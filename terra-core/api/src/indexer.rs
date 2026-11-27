//! Chain indexer (Sprint C; P0 hardening 2026-11-27).
//!
//! Polls the configured Solana RPC for `terra_registry` program accounts,
//! decodes Anchor accounts by discriminator, and upserts them into their
//! mirror tables. Client-POSTed rows start with `chain_verified = FALSE` and
//! are confirmed by the indexer.
//!
//! Provenance semantics (R-041): `chain_verified = TRUE` means the row was
//! decoded from an on-chain account **and** a real transaction signature was
//! resolved for that account. `chain_slot` is the slot of that transaction,
//! `chain_signature` is the signature itself. When no signature can be
//! resolved, all three stay honest (`FALSE`/`NULL`/`NULL`) — never the
//! poll-time slot and never an empty-string signature.
//!
//! Per-account isolation (R-039): one bad or undecodable account only skips
//! that account. A failed mirror cannot abort the poll and starve later
//! accounts (notably parcels) in RPC order.
//!
//! The indexer runs as a background tokio task spawned from `main.rs`.

use anyhow::{Context, Result};
use solana_client::rpc_client::{GetConfirmedSignaturesForAddress2Config, RpcClient};
use solana_client::rpc_response::RpcConfirmedTransactionStatusWithSignature;
use solana_sdk::pubkey::Pubkey;
use sqlx::PgPool;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, warn};
use uuid::Uuid;

use crate::state::AppState;

/// terra_registry program ID (mirrors terra-web/src/lib/constants.ts).
const PROGRAM_ID: &str = "GaEDbktvpZ3qiqp4PmFgHwDSa6JsFfVjXFqNb2nTbage";

/// Anchor 8-byte discriminator for the `Parcel` account:
/// SHA-256("account:Parcel")[..8]
const PARCEL_DISCRIMINATOR: [u8; 8] = [149, 167, 245, 67, 209, 244, 214, 75];

/// Anchor 8-byte discriminator for the `VerificationAttestation` account:
/// SHA-256("account:VerificationAttestation")[..8]
const VERIFICATION_ATTESTATION_DISCRIMINATOR: [u8; 8] = [231, 126, 92, 51, 84, 178, 81, 242];

/// Anchor 8-byte discriminator for the `Rights` account:
/// SHA-256("account:Rights")[..8] = 4fbb5655f3d567ea
const RIGHTS_DISCRIMINATOR: [u8; 8] = [0x4f, 0xbb, 0x56, 0x55, 0xf3, 0xd5, 0x67, 0xea];

/// Anchor 8-byte discriminator for the `Claim` account:
/// SHA-256("account:Claim")[..8] = 9b4616b07bd7f666
const CLAIM_DISCRIMINATOR: [u8; 8] = [0x9b, 0x46, 0x16, 0xb0, 0x7b, 0xd7, 0xf6, 0x66];

/// Anchor 8-byte discriminator for the `SpatialAsset` account:
/// SHA-256("account:SpatialAsset")[..8] = 8f85940476320aaa
const SPATIAL_ASSET_DISCRIMINATOR: [u8; 8] = [0x8f, 0x85, 0x94, 0x04, 0x76, 0x32, 0x0a, 0xaa];

/// Anchor 8-byte discriminator for the `GeometryVersion` account:
/// SHA-256("account:GeometryVersion")[..8] = 4e8f23c261d5c3d8
const GEOMETRY_VERSION_DISCRIMINATOR: [u8; 8] = [0x4e, 0x8f, 0x23, 0xc2, 0x61, 0xd5, 0xc3, 0xd8];

/// Rights account kind: OWNERSHIP = 0
const RIGHTS_KIND_OWNERSHIP: u8 = 0;

/// Interval between indexer polls.
const POLL_INTERVAL_SECS: u64 = 15;

/// Transaction-level provenance for one mirrored account.
///
/// `signature` is None unless `getSignaturesForAddress` returned a real,
/// non-failing transaction. `slot` is that transaction's slot — never the
/// slot at which the poll happened.
#[derive(Debug, Clone, PartialEq, Eq)]
struct AccountProvenance {
    slot: Option<u64>,
    signature: Option<String>,
}

impl AccountProvenance {
    /// No confirming transaction resolved: the row is not chain-verified.
    fn unverified() -> Self {
        Self { slot: None, signature: None }
    }

    fn verified(slot: u64, signature: String) -> Self {
        Self { slot: Some(slot), signature: Some(signature) }
    }

    fn is_verified(&self) -> bool {
        self.signature.is_some()
    }
}

/// Pick the newest successful transaction from `getSignaturesForAddress`.
/// Entries are returned newest-first; failed transactions never prove state.
fn provenance_from_signatures(
    entries: &[RpcConfirmedTransactionStatusWithSignature],
) -> AccountProvenance {
    for entry in entries {
        if entry.err.is_none() {
            return AccountProvenance::verified(entry.slot, entry.signature.clone());
        }
    }
    AccountProvenance::unverified()
}

/// Resolve transaction provenance for a single account.
///
/// RPC failures are non-fatal: they yield an unverified provenance so one
/// rate-limited lookup cannot starve the rest of the poll.
fn resolve_provenance(
    client: &RpcClient,
    pubkey: &Pubkey,
) -> AccountProvenance {
    // Bounded lookback: enough to skip a few recent failed transactions
    // without pulling the account's entire history every poll.
    let config = GetConfirmedSignaturesForAddress2Config {
        limit: Some(5),
        ..Default::default()
    };
    match client.get_signatures_for_address_with_config(pubkey, config) {
        Ok(entries) => provenance_from_signatures(&entries),
        Err(err) => {
            warn!(%pubkey, "signature lookup failed; mirroring without chain verification: {err}");
            AccountProvenance::unverified()
        }
    }
}

/// A decoded account that has a mirror table. Unknown/discriminator-mismatched
/// accounts are ignored; classification never touches the database.
enum MirroredAccount {
    Parcel(ParsedParcel),
    Attestation(ParsedVerificationAttestation),
    Claim(ParsedClaim),
    SpatialAsset(ParsedSpatialAsset),
    GeometryVersion(ParsedGeometryVersion),
}

fn classify_account(data: &[u8]) -> Option<MirroredAccount> {
    if let Some(parcel) = parse_parcel(data) {
        Some(MirroredAccount::Parcel(parcel))
    } else if let Some(attestation) = parse_verification_attestation(data) {
        Some(MirroredAccount::Attestation(attestation))
    } else if let Some(claim) = parse_claim(data) {
        Some(MirroredAccount::Claim(claim))
    } else if let Some(asset) = parse_spatial_asset(data) {
        Some(MirroredAccount::SpatialAsset(asset))
    } else if let Some(version) = parse_geometry_version(data) {
        Some(MirroredAccount::GeometryVersion(version))
    } else {
        None
    }
}

/// Mirror one account. Errors are returned to the caller, which isolates
/// them per account instead of aborting the whole poll.
async fn mirror_account(
    pool: &PgPool,
    client: &RpcClient,
    pubkey: &Pubkey,
    data: &[u8],
    parcel_holder: &std::collections::HashMap<[u8; 32], [u8; 32]>,
) -> Result<bool> {
    let Some(decoded) = classify_account(data) else {
        return Ok(false);
    };
    let provenance = resolve_provenance(client, pubkey);
    match decoded {
        MirroredAccount::Parcel(parcel) => {
            let holder = parcel_holder
                .get(&parcel.id)
                .copied()
                .unwrap_or_else(|| pubkey.to_bytes());
            upsert_parcel(pool, pubkey, &parcel, &holder, &provenance).await?;
        }
        MirroredAccount::Attestation(attestation) => {
            upsert_verification_attestation(pool, pubkey, &attestation, &provenance).await?;
        }
        MirroredAccount::Claim(claim) => {
            upsert_claim(pool, pubkey, &claim, &provenance).await?;
        }
        MirroredAccount::SpatialAsset(asset) => {
            upsert_spatial_asset(pool, pubkey, &asset, &provenance).await?;
        }
        MirroredAccount::GeometryVersion(version) => {
            upsert_geometry_version(pool, pubkey, &version, &provenance).await?;
        }
    }
    Ok(true)
}

/// Parsed Rights account from on-chain data.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ParsedRights {
    pub parcel: [u8; 32],
    pub rights_kind: u8,
    pub holder: [u8; 32],
    pub granter: [u8; 32],
    pub created_at: i64,
    pub expires_at: i64,
    pub notes: String,
    pub status: u8,
    pub grace_period_secs: i64,
}

/// Decode a raw Anchor account buffer into a ParsedRights.
///
/// Anchor account layout:
///   [0..8]   discriminator (8 bytes)
///   [8..40]  parcel [u8; 32]
///   [40]     rights_kind u8
///   [41..73] holder [u8; 32]
///   [73..105] granter [u8; 32]
///   [105..113] created_at i64 LE
///   [113..121] expires_at i64 LE
///   [121] notes length (u8)
///   [122..] notes bytes (UTF-8)
///   then status u8, grace_period_secs i64 LE
fn parse_rights(data: &[u8]) -> Option<ParsedRights> {
    if data.len() < 8 {
        return None;
    }
    if data[..8] != RIGHTS_DISCRIMINATOR {
        return None;
    }

    let mut offset = 8;

    // parcel [u8; 32]
    if data.len() < offset + 32 {
        return None;
    }
    let mut parcel = [0u8; 32];
    parcel.copy_from_slice(&data[offset..offset + 32]);
    offset += 32;

    // rights_kind u8
    if data.len() < offset + 1 {
        return None;
    }
    let rights_kind = data[offset];
    offset += 1;

    // holder [u8; 32]
    if data.len() < offset + 32 {
        return None;
    }
    let mut holder = [0u8; 32];
    holder.copy_from_slice(&data[offset..offset + 32]);
    offset += 32;

    // granter [u8; 32]
    if data.len() < offset + 32 {
        return None;
    }
    let mut granter = [0u8; 32];
    granter.copy_from_slice(&data[offset..offset + 32]);
    offset += 32;

    // created_at i64 LE
    if data.len() < offset + 8 {
        return None;
    }
    let created_at = i64::from_le_bytes([
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
        data[offset + 4],
        data[offset + 5],
        data[offset + 6],
        data[offset + 7],
    ]);
    offset += 8;

    // expires_at i64 LE
    if data.len() < offset + 8 {
        return None;
    }
    let expires_at = i64::from_le_bytes([
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
        data[offset + 4],
        data[offset + 5],
        data[offset + 6],
        data[offset + 7],
    ]);
    offset += 8;

    // notes length (u8)
    if data.len() < offset + 1 {
        return None;
    }
    let notes_len = data[offset] as usize;
    offset += 1;

    // notes (UTF-8)
    if data.len() < offset + notes_len {
        return None;
    }
    let notes = String::from_utf8_lossy(&data[offset..offset + notes_len]).to_string();
    offset += notes_len;

    // status u8
    if data.len() < offset + 1 {
        return None;
    }
    let status = data[offset];
    offset += 1;

    // grace_period_secs i64 LE
    if data.len() < offset + 8 {
        return None;
    }
    let grace_period_secs = i64::from_le_bytes([
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
        data[offset + 4],
        data[offset + 5],
        data[offset + 6],
        data[offset + 7],
    ]);

    Some(ParsedRights {
        parcel,
        rights_kind,
        holder,
        granter,
        created_at,
        expires_at,
        notes,
        status,
        grace_period_secs,
    })
}

/// Parsed Parcel account from on-chain data.
///
/// `id` is part of the account layout and kept for completeness/debugging
/// even though the mirror uses `onchain_id` (the PDA address as base58 text)
/// as the natural key.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ParsedParcel {
    pub id: [u8; 32],
    pub name: String,
    pub geometry_hash: [u8; 32],
    pub status: u8,
    pub rights_count: u8,
    pub infrastructure_flags: u16,
    pub access_hash: [u8; 32],
    pub created_at: i64,
    pub updated_at: i64,
}

/// Parsed Claim account from on-chain data.
///
/// PDA: `["claim", parcel, claim_id]`.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ParsedClaim {
    pub claim_id: [u8; 32],
    pub parcel: [u8; 32],
    pub claim_type: u8,
    pub submitted_by: [u8; 32],
    pub status: u8,
    pub statement_hash: [u8; 32],
    pub version: u8,
    pub evidence_count: u8,
    pub observation_count: u8,
    pub attestation_count: u8,
    pub required_attestations: u8,
    pub created_at: i64,
    pub updated_at: i64,
}

/// Decode a raw Anchor account buffer into a ParsedClaim.
///
/// Anchor account layout:
///   [0..8]   discriminator (8 bytes)
///   [8..40]  claim_id [u8; 32]
///   [40..72] parcel [u8; 32]
///   [72]     claim_type u8
///   [73..105] submitted_by [u8; 32]
///   [105]    status u8
///   [106..138] statement_hash [u8; 32]
///   [138]    version u8
///   [139]    evidence_count u8
///   [140]    observation_count u8
///   [141]    attestation_count u8
///   [142]    required_attestations u8
///   [143..151] created_at i64 LE
///   [151..159] updated_at i64 LE
fn parse_claim(data: &[u8]) -> Option<ParsedClaim> {
    if data.len() < 8 {
        return None;
    }
    if data[..8] != CLAIM_DISCRIMINATOR {
        return None;
    }

    let mut offset = 8;

    // claim_id [u8; 32]
    if data.len() < offset + 32 {
        return None;
    }
    let mut claim_id = [0u8; 32];
    claim_id.copy_from_slice(&data[offset..offset + 32]);
    offset += 32;

    // parcel [u8; 32]
    if data.len() < offset + 32 {
        return None;
    }
    let mut parcel = [0u8; 32];
    parcel.copy_from_slice(&data[offset..offset + 32]);
    offset += 32;

    // claim_type u8
    if data.len() < offset + 1 {
        return None;
    }
    let claim_type = data[offset];
    offset += 1;

    // submitted_by [u8; 32]
    if data.len() < offset + 32 {
        return None;
    }
    let mut submitted_by = [0u8; 32];
    submitted_by.copy_from_slice(&data[offset..offset + 32]);
    offset += 32;

    // status u8
    if data.len() < offset + 1 {
        return None;
    }
    let status = data[offset];
    offset += 1;

    // statement_hash [u8; 32]
    if data.len() < offset + 32 {
        return None;
    }
    let mut statement_hash = [0u8; 32];
    statement_hash.copy_from_slice(&data[offset..offset + 32]);
    offset += 32;

    // version u8
    if data.len() < offset + 1 {
        return None;
    }
    let version = data[offset];
    offset += 1;

    // evidence_count u8
    if data.len() < offset + 1 {
        return None;
    }
    let evidence_count = data[offset];
    offset += 1;

    // observation_count u8
    if data.len() < offset + 1 {
        return None;
    }
    let observation_count = data[offset];
    offset += 1;

    // attestation_count u8
    if data.len() < offset + 1 {
        return None;
    }
    let attestation_count = data[offset];
    offset += 1;

    // required_attestations u8
    if data.len() < offset + 1 {
        return None;
    }
    let required_attestations = data[offset];
    offset += 1;

    // created_at i64 LE
    if data.len() < offset + 8 {
        return None;
    }
    let created_at = i64::from_le_bytes([
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
        data[offset + 4],
        data[offset + 5],
        data[offset + 6],
        data[offset + 7],
    ]);
    offset += 8;

    // updated_at i64 LE
    if data.len() < offset + 8 {
        return None;
    }
    let updated_at = i64::from_le_bytes([
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
        data[offset + 4],
        data[offset + 5],
        data[offset + 6],
        data[offset + 7],
    ]);

    Some(ParsedClaim {
        claim_id,
        parcel,
        claim_type,
        submitted_by,
        status,
        statement_hash,
        version,
        evidence_count,
        observation_count,
        attestation_count,
        required_attestations,
        created_at,
        updated_at,
    })
}

/// Parsed SpatialAsset account from on-chain data.
///
/// PDA: `["spatial_asset", parcel]`.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ParsedSpatialAsset {
    pub parcel: [u8; 32],
    pub authority: [u8; 32],
    pub dimensionality: u8,
    pub elevation_min_mm: i64,
    pub elevation_max_mm: i64,
    pub geometry_version_count: u32,
    pub latest_geometry: [u8; 32],
    pub created_at: i64,
    pub updated_at: i64,
}

/// Decode a raw Anchor account buffer into a ParsedSpatialAsset.
///
/// Anchor account layout:
///   [0..8]   discriminator (8 bytes)
///   [8..40]  parcel [u8; 32]
///   [40..72] authority [u8; 32]
///   [72]     dimensionality u8
///   [73..81] elevation_min_mm i64 LE
///   [81..89] elevation_max_mm i64 LE
///   [89..93] geometry_version_count u32 LE
///   [93..125] latest_geometry [u8; 32]
///   [125..133] created_at i64 LE
///   [133..141] updated_at i64 LE
fn parse_spatial_asset(data: &[u8]) -> Option<ParsedSpatialAsset> {
    if data.len() < 8 {
        return None;
    }
    if data[..8] != SPATIAL_ASSET_DISCRIMINATOR {
        return None;
    }

    let mut offset = 8;

    // parcel [u8; 32]
    if data.len() < offset + 32 {
        return None;
    }
    let mut parcel = [0u8; 32];
    parcel.copy_from_slice(&data[offset..offset + 32]);
    offset += 32;

    // authority [u8; 32]
    if data.len() < offset + 32 {
        return None;
    }
    let mut authority = [0u8; 32];
    authority.copy_from_slice(&data[offset..offset + 32]);
    offset += 32;

    // dimensionality u8
    if data.len() < offset + 1 {
        return None;
    }
    let dimensionality = data[offset];
    offset += 1;

    // elevation_min_mm i64 LE
    if data.len() < offset + 8 {
        return None;
    }
    let elevation_min_mm = i64::from_le_bytes([
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
        data[offset + 4],
        data[offset + 5],
        data[offset + 6],
        data[offset + 7],
    ]);
    offset += 8;

    // elevation_max_mm i64 LE
    if data.len() < offset + 8 {
        return None;
    }
    let elevation_max_mm = i64::from_le_bytes([
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
        data[offset + 4],
        data[offset + 5],
        data[offset + 6],
        data[offset + 7],
    ]);
    offset += 8;

    // geometry_version_count u32 LE
    if data.len() < offset + 4 {
        return None;
    }
    let geometry_version_count = u32::from_le_bytes([
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
    ]);
    offset += 4;

    // latest_geometry [u8; 32]
    if data.len() < offset + 32 {
        return None;
    }
    let mut latest_geometry = [0u8; 32];
    latest_geometry.copy_from_slice(&data[offset..offset + 32]);
    offset += 32;

    // created_at i64 LE
    if data.len() < offset + 8 {
        return None;
    }
    let created_at = i64::from_le_bytes([
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
        data[offset + 4],
        data[offset + 5],
        data[offset + 6],
        data[offset + 7],
    ]);
    offset += 8;

    // updated_at i64 LE
    if data.len() < offset + 8 {
        return None;
    }
    let updated_at = i64::from_le_bytes([
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
        data[offset + 4],
        data[offset + 5],
        data[offset + 6],
        data[offset + 7],
    ]);

    Some(ParsedSpatialAsset {
        parcel,
        authority,
        dimensionality,
        elevation_min_mm,
        elevation_max_mm,
        geometry_version_count,
        latest_geometry,
        created_at,
        updated_at,
    })
}

/// Parsed GeometryVersion account from on-chain data.
///
/// PDA: `["geometry_version", asset, version_le_bytes]`.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ParsedGeometryVersion {
    pub asset: [u8; 32],
    pub parcel: [u8; 32],
    pub version: u32,
    pub geometry_hash: [u8; 32],
    pub source: u8,
    pub dimension: u8,
    pub storage_reference: String,
    pub submitted_by: [u8; 32],
    pub submitted_at: i64,
    pub verified: bool,
    pub verified_by: [u8; 32],
    pub verified_at: i64,
    pub evidence_manifest: [u8; 32],
    pub elevation_min_mm: i64,
    pub elevation_max_mm: i64,
    pub elevation_source: u8,
    pub required: u8,
    pub attestors: Vec<String>,
    pub attest_count: u8,
}

/// Decode a raw Anchor account buffer into a ParsedGeometryVersion.
///
/// Anchor account layout:
///   [0..8]   discriminator (8 bytes)
///   [8..40]  asset [u8; 32]
///   [40..72] parcel [u8; 32]
///   [72..76] version u32 LE
///   [76..108] geometry_hash [u8; 32]
///   [108]    source u8
///   [109]    dimension u8
///   [110]    storage_reference length (u16 LE)
///   [112..]  storage_reference bytes (UTF-8)
///   then     submitted_by [u8; 32]
///   then     submitted_at i64 LE
///   then     verified u8
///   then     verified_by [u8; 32]
///   then     verified_at i64 LE
///   then     evidence_manifest [u8; 32]
///   then     elevation_min_mm i64 LE
///   then     elevation_max_mm i64 LE
///   then     elevation_source u8
///   then     required u8
///   then     attestors [Pubkey; MAX_VALIDATORS] (32 bytes each)
///   then     attest_count u8
fn parse_geometry_version(data: &[u8]) -> Option<ParsedGeometryVersion> {
    if data.len() < 8 {
        return None;
    }
    if data[..8] != GEOMETRY_VERSION_DISCRIMINATOR {
        return None;
    }

    let mut offset = 8;

    // asset [u8; 32]
    if data.len() < offset + 32 {
        return None;
    }
    let mut asset = [0u8; 32];
    asset.copy_from_slice(&data[offset..offset + 32]);
    offset += 32;

    // parcel [u8; 32]
    if data.len() < offset + 32 {
        return None;
    }
    let mut parcel = [0u8; 32];
    parcel.copy_from_slice(&data[offset..offset + 32]);
    offset += 32;

    // version u32 LE
    if data.len() < offset + 4 {
        return None;
    }
    let version = u32::from_le_bytes([
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
    ]);
    offset += 4;

    // geometry_hash [u8; 32]
    if data.len() < offset + 32 {
        return None;
    }
    let mut geometry_hash = [0u8; 32];
    geometry_hash.copy_from_slice(&data[offset..offset + 32]);
    offset += 32;

    // source u8
    if data.len() < offset + 1 {
        return None;
    }
    let source = data[offset];
    offset += 1;

    // dimension u8
    if data.len() < offset + 1 {
        return None;
    }
    let dimension = data[offset];
    offset += 1;

    // storage_reference: u16 LE length + bytes
    if data.len() < offset + 2 {
        return None;
    }
    let storage_ref_len = u16::from_le_bytes([data[offset], data[offset + 1]]) as usize;
    offset += 2;
    if data.len() < offset + storage_ref_len {
        return None;
    }
    let storage_reference =
        String::from_utf8_lossy(&data[offset..offset + storage_ref_len]).to_string();
    offset += storage_ref_len;

    // submitted_by [u8; 32]
    if data.len() < offset + 32 {
        return None;
    }
    let mut submitted_by = [0u8; 32];
    submitted_by.copy_from_slice(&data[offset..offset + 32]);
    offset += 32;

    // submitted_at i64 LE
    if data.len() < offset + 8 {
        return None;
    }
    let submitted_at = i64::from_le_bytes([
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
        data[offset + 4],
        data[offset + 5],
        data[offset + 6],
        data[offset + 7],
    ]);
    offset += 8;

    // verified u8
    if data.len() < offset + 1 {
        return None;
    }
    let verified = data[offset] != 0;
    offset += 1;

    // verified_by [u8; 32]
    if data.len() < offset + 32 {
        return None;
    }
    let mut verified_by = [0u8; 32];
    verified_by.copy_from_slice(&data[offset..offset + 32]);
    offset += 32;

    // verified_at i64 LE
    if data.len() < offset + 8 {
        return None;
    }
    let verified_at = i64::from_le_bytes([
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
        data[offset + 4],
        data[offset + 5],
        data[offset + 6],
        data[offset + 7],
    ]);
    offset += 8;

    // evidence_manifest [u8; 32]
    if data.len() < offset + 32 {
        return None;
    }
    let mut evidence_manifest = [0u8; 32];
    evidence_manifest.copy_from_slice(&data[offset..offset + 32]);
    offset += 32;

    // elevation_min_mm i64 LE
    if data.len() < offset + 8 {
        return None;
    }
    let elevation_min_mm = i64::from_le_bytes([
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
        data[offset + 4],
        data[offset + 5],
        data[offset + 6],
        data[offset + 7],
    ]);
    offset += 8;

    // elevation_max_mm i64 LE
    if data.len() < offset + 8 {
        return None;
    }
    let elevation_max_mm = i64::from_le_bytes([
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
        data[offset + 4],
        data[offset + 5],
        data[offset + 6],
        data[offset + 7],
    ]);
    offset += 8;

    // elevation_source u8
    if data.len() < offset + 1 {
        return None;
    }
    let elevation_source = data[offset];
    offset += 1;

    // required u8
    if data.len() < offset + 1 {
        return None;
    }
    let required = data[offset];
    offset += 1;

    // attestors [Pubkey; MAX_VALIDATORS] - 8 validators per lib.rs
    const MAX_VALIDATORS: usize = 8;
    if data.len() < offset + MAX_VALIDATORS * 32 {
        return None;
    }
    let mut attestors = Vec::with_capacity(MAX_VALIDATORS);
    for i in 0..MAX_VALIDATORS {
        let start = offset + i * 32;
        let pk = &data[start..start + 32];
        if pk.iter().all(|&b| b == 0) {
            continue; // skip zero-filled slots
        }
        attestors.push(pk.iter().map(|b| format!("{b:02x}")).collect());
    }
    offset += MAX_VALIDATORS * 32;

    // attest_count u8
    if data.len() < offset + 1 {
        return None;
    }
    let attest_count = data[offset];

    Some(ParsedGeometryVersion {
        asset,
        parcel,
        version,
        geometry_hash,
        source,
        dimension,
        storage_reference,
        submitted_by,
        submitted_at,
        verified,
        verified_by,
        verified_at,
        evidence_manifest,
        elevation_min_mm,
        elevation_max_mm,
        elevation_source,
        required,
        attestors,
        attest_count,
    })
}

/// Decode a raw Anchor account buffer into a ParsedParcel.
///
/// Anchor account layout:
///   [0..8]   discriminator (8 bytes)
///   [8..40]  id [u8; 32]
///   [40]     name length (u8, max_len <= 255)
///   [41..]   name bytes (UTF-8)
///   then     geometry_hash [u8; 32], status u8, rights_count u8,
///            infrastructure_flags u16 LE, access_hash [u8; 32],
///            created_at i64 LE, updated_at i64 LE
fn parse_parcel(data: &[u8]) -> Option<ParsedParcel> {
    if data.len() < 8 {
        return None;
    }
    if data[..8] != PARCEL_DISCRIMINATOR {
        return None;
    }

    let mut offset = 8;

    // id [u8; 32]
    if data.len() < offset + 32 {
        return None;
    }
    let mut id = [0u8; 32];
    id.copy_from_slice(&data[offset..offset + 32]);
    offset += 32;

    // name length (u8)
    if data.len() < offset + 1 {
        return None;
    }
    let name_len = data[offset] as usize;
    offset += 1;

    // name (UTF-8)
    if data.len() < offset + name_len {
        return None;
    }
    let name = String::from_utf8_lossy(&data[offset..offset + name_len]).to_string();
    offset += name_len;

    // geometry_hash [u8; 32]
    if data.len() < offset + 32 {
        return None;
    }
    let mut geometry_hash = [0u8; 32];
    geometry_hash.copy_from_slice(&data[offset..offset + 32]);
    offset += 32;

    // status u8
    if data.len() < offset + 1 {
        return None;
    }
    let status = data[offset];
    offset += 1;

    // rights_count u8
    if data.len() < offset + 1 {
        return None;
    }
    let rights_count = data[offset];
    offset += 1;

    // infrastructure_flags u16 LE
    if data.len() < offset + 2 {
        return None;
    }
    let infrastructure_flags = u16::from_le_bytes([data[offset], data[offset + 1]]);
    offset += 2;

    // access_hash [u8; 32]
    if data.len() < offset + 32 {
        return None;
    }
    let mut access_hash = [0u8; 32];
    access_hash.copy_from_slice(&data[offset..offset + 32]);
    offset += 32;

    // created_at i64 LE
    if data.len() < offset + 8 {
        return None;
    }
    let created_at = i64::from_le_bytes([
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
        data[offset + 4],
        data[offset + 5],
        data[offset + 6],
        data[offset + 7],
    ]);
    offset += 8;

    // updated_at i64 LE
    if data.len() < offset + 8 {
        return None;
    }
    let updated_at = i64::from_le_bytes([
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
        data[offset + 4],
        data[offset + 5],
        data[offset + 6],
        data[offset + 7],
    ]);

    Some(ParsedParcel {
        id,
        name,
        geometry_hash,
        status,
        rights_count,
        infrastructure_flags,
        access_hash,
        created_at,
        updated_at,
    })
}

/// Parsed VerificationAttestation account from on-chain data.
///
/// `id` and `created_at` are part of the account layout and kept for
/// completeness/debugging even though the mirror only needs the fields below.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ParsedVerificationAttestation {
    pub id: [u8; 32],
    pub claim: [u8; 32],
    pub validator: [u8; 32],
    pub observation: [u8; 32],
    pub credential_subject: [u8; 32],
    pub vc_type: String,
    pub issuance_date: i64,
    pub expiration_date: i64,
    pub sd_jwt_digest: [u8; 32],
    pub selective_disclosure: bool,
    pub created_at: i64,
    pub protocol_version: u8,
}

/// Decode a raw Anchor account buffer into a ParsedVerificationAttestation.
///
/// Anchor account layout:
///   [0..8]   discriminator (8 bytes)
///   [8..40]  claim [u8; 32]
///   [40..72] validator [u8; 32]
///   [72..104] observation [u8; 32]
///   [104..136] credential_subject [u8; 32]
///   [136] vc_type length (u8)
///   [137..] vc_type bytes (UTF-8)
///   then issuance_date i64 LE, expiration_date i64 LE
///   then sd_jwt_digest [u8; 32]
///   then selective_disclosure u8
///   then created_at i64 LE
///   then protocol_version u8
fn parse_verification_attestation(data: &[u8]) -> Option<ParsedVerificationAttestation> {
    if data.len() < 8 {
        return None;
    }
    if data[..8] != VERIFICATION_ATTESTATION_DISCRIMINATOR {
        return None;
    }

    let mut offset = 8;

    // claim [u8; 32]
    if data.len() < offset + 32 {
        return None;
    }
    let mut claim = [0u8; 32];
    claim.copy_from_slice(&data[offset..offset + 32]);
    offset += 32;

    // validator [u8; 32]
    if data.len() < offset + 32 {
        return None;
    }
    let mut validator = [0u8; 32];
    validator.copy_from_slice(&data[offset..offset + 32]);
    offset += 32;

    // observation [u8; 32]
    if data.len() < offset + 32 {
        return None;
    }
    let mut observation = [0u8; 32];
    observation.copy_from_slice(&data[offset..offset + 32]);
    offset += 32;

    // credential_subject [u8; 32]
    if data.len() < offset + 32 {
        return None;
    }
    let mut credential_subject = [0u8; 32];
    credential_subject.copy_from_slice(&data[offset..offset + 32]);
    offset += 32;

    // vc_type length (u8)
    if data.len() < offset + 1 {
        return None;
    }
    let vc_type_len = data[offset] as usize;
    offset += 1;

    // vc_type (UTF-8)
    if data.len() < offset + vc_type_len {
        return None;
    }
    let vc_type = String::from_utf8_lossy(&data[offset..offset + vc_type_len]).to_string();
    offset += vc_type_len;

    // issuance_date i64 LE
    if data.len() < offset + 8 {
        return None;
    }
    let issuance_date = i64::from_le_bytes([
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
        data[offset + 4],
        data[offset + 5],
        data[offset + 6],
        data[offset + 7],
    ]);
    offset += 8;

    // expiration_date i64 LE
    if data.len() < offset + 8 {
        return None;
    }
    let expiration_date = i64::from_le_bytes([
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
        data[offset + 4],
        data[offset + 5],
        data[offset + 6],
        data[offset + 7],
    ]);
    offset += 8;

    // sd_jwt_digest [u8; 32]
    if data.len() < offset + 32 {
        return None;
    }
    let mut sd_jwt_digest = [0u8; 32];
    sd_jwt_digest.copy_from_slice(&data[offset..offset + 32]);
    offset += 32;

    // selective_disclosure u8
    if data.len() < offset + 1 {
        return None;
    }
    let selective_disclosure = data[offset] != 0;
    offset += 1;

    // created_at i64 LE
    if data.len() < offset + 8 {
        return None;
    }
    let created_at = i64::from_le_bytes([
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
        data[offset + 4],
        data[offset + 5],
        data[offset + 6],
        data[offset + 7],
    ]);
    offset += 8;

    // protocol_version u8
    if data.len() < offset + 1 {
        return None;
    }
    let protocol_version = data[offset];

    Some(ParsedVerificationAttestation {
        id: [0u8; 32], // Will be set from the pubkey
        claim,
        validator,
        observation,
        credential_subject,
        vc_type,
        issuance_date,
        expiration_date,
        sd_jwt_digest,
        selective_disclosure,
        created_at,
        protocol_version,
    })
}

/// Run the indexer loop: poll RPC, decode accounts, upsert mirror rows.
pub async fn run_indexer(rpc_url: String, state: Arc<AppState>) -> Result<()> {
    let client = RpcClient::new(rpc_url.clone());
    let program_id = Pubkey::from_str(PROGRAM_ID)
        .with_context(|| format!("invalid program id: {PROGRAM_ID}"))?;

    info!("chain indexer started: program={PROGRAM_ID} rpc={rpc_url} poll={POLL_INTERVAL_SECS}s");

    loop {
        match poll_once(&client, &program_id, &state).await {
            Ok(count) => {
                if count > 0 {
                    info!("indexer polled: {count} parcel accounts upserted");
                }
            }
            Err(err) => {
                warn!("indexer poll failed: {err}");
            }
        }
        sleep(Duration::from_secs(POLL_INTERVAL_SECS)).await;
    }
}

/// Single poll: fetch program accounts, mirror each decoded account with
/// transaction-level provenance.
///
/// Poll-level RPC failures still abort the poll (nothing can be mirrored).
/// Per-account decode/DB failures are isolated: they are logged and skipped
/// so later accounts — including parcels — keep mirroring.
async fn poll_once(client: &RpcClient, program_id: &Pubkey, state: &AppState) -> Result<usize> {
    // Keep the cheap slot probe as a poll-health check; provenance itself no
    // longer uses this scan-time slot (R-041).
    let _scan_slot = client.get_slot().with_context(|| "get_slot failed")?;

    let accounts = client
        .get_program_accounts(program_id)
        .with_context(|| "get_program_accounts failed")?;

    // Decode all Rights accounts first, then map parcel -> holder for
    // ownership rights (rights_kind == 0). This lets us mirror the real
    // wallet holder instead of the PDA address.
    let mut parcel_holder: std::collections::HashMap<[u8; 32], [u8; 32]> =
        std::collections::HashMap::new();
    for (_pubkey, account) in &accounts {
        if let Some(rights) = parse_rights(&account.data)
            && rights.rights_kind == RIGHTS_KIND_OWNERSHIP
        {
            parcel_holder.insert(rights.parcel, rights.holder);
        }
    }

    let mut upserted = 0usize;
    let mut failed = 0usize;
    for (pubkey, account) in &accounts {
        match mirror_account(
            &state.pool,
            client,
            pubkey,
            &account.data,
            &parcel_holder,
        )
        .await
        {
            Ok(true) => upserted += 1,
            Ok(false) => {}
            Err(err) => {
                warn!(%pubkey, error = %err, "indexer account mirror failed; continuing poll");
                failed += 1;
            }
        }
    }
    if failed > 0 {
        info!("indexer poll: {upserted} accounts upserted, {failed} isolated failures");
    }

    Ok(upserted)
}

/// Insert or update a parcel row from on-chain data.
///
/// The on-chain `Parcel` account carries a 32-byte id and a geometry hash but
/// no owner — ownership lives in a separate Rights PDA. Off-chain we mirror
/// this with `parcels.id` (auto-generated UUID) + `onchain_id` (the PDA
/// address as base58 text) plus a `parcel_ownership` row so `PARCEL_SELECT`'s
/// JOIN finds indexed parcels.
///
/// On UPDATE we deliberately do NOT touch `geometry` — the on-chain Parcel
/// account only carries a geometry hash, not the full GeoJSON, so overwriting
/// the mirror's geometry with NULL would destroy client-supplied polygon data.
async fn upsert_parcel(
    pool: &PgPool,
    pubkey: &Pubkey,
    parcel: &ParsedParcel,
    holder: &[u8; 32],
    provenance: &AccountProvenance,
) -> Result<()> {
    let pubkey_str = pubkey.to_string();
    let holder_hex: String = holder.iter().map(|b| format!("{b:02x}")).collect();

    // geometry_hash as text (hex)
    let geometry_hash_hex: String = parcel
        .geometry_hash
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    let access_hash_hex: String = parcel
        .access_hash
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();

    // Check whether the row already exists (onchain_id is the natural key).
    let exists: Option<i64> = sqlx::query_scalar("SELECT id FROM parcels WHERE onchain_id = $1")
        .bind(&pubkey_str)
        .fetch_optional(pool)
        .await
        .map_err(|e| {
            warn!("check existing parcel {pubkey} failed: {e:?}");
            anyhow::anyhow!("check existing parcel {pubkey}: {e}")
        })?;

    if let Some(parcel_id) = exists {
        // UPDATE: preserve existing geometry (don't set to NULL).
        sqlx::query(
            r#"
            UPDATE parcels SET
                name = $1,
                status = $2,
                geometry_hash = $3,
                infrastructure_flags = $4,
                rights_count = $5,
                access_hash = $6,
                updated_at = to_timestamp($7::double precision),
                chain_verified = $8,
                chain_slot = $9,
                chain_signature = $10
            WHERE id = $11
            "#,
        )
        .bind(&parcel.name)
        .bind(format!("{}", parcel.status))
        .bind(&geometry_hash_hex)
        .bind(parcel.infrastructure_flags as i16)
        .bind(parcel.rights_count as i16)
        .bind(&access_hash_hex)
        .bind(parcel.updated_at as f64)
        .bind(provenance.is_verified())
        .bind(provenance.slot.map(|s| s as i64))
        .bind(provenance.signature.as_deref())
        .bind(parcel_id)
        .execute(pool)
        .await
        .map_err(|e| {
            warn!("update parcel {pubkey} failed: {e:?}");
            anyhow::anyhow!("update parcel {pubkey}: {e}")
        })?;

        // Update the holder to the real wallet from the Rights PDA.
        sqlx::query("UPDATE parcel_ownership SET holder = $1 WHERE parcel_id = $2")
            .bind(&holder_hex)
            .bind(parcel_id)
            .execute(pool)
            .await
            .map_err(|e| {
                warn!("update parcel_ownership {pubkey} failed: {e:?}");
                anyhow::anyhow!("update parcel_ownership {pubkey}: {e}")
            })?;
    } else {
        let parcel_id: Uuid = sqlx::query_scalar(
            r#"
            INSERT INTO parcels (
                name, status, geometry, onchain_id, geometry_hash,
                infrastructure_flags, rights_count, access_hash,
                created_at, updated_at, chain_verified, chain_slot, chain_signature
            ) VALUES (
                $1, $2, NULL, $3, $4, $5, $6, $7,
                to_timestamp($8::double precision), to_timestamp($9::double precision), $10, $11, $12
            )
            RETURNING id
            "#,
        )
        .bind(&parcel.name)
        .bind(format!("{}", parcel.status))
        .bind(&pubkey_str)
        .bind(&geometry_hash_hex)
        .bind(parcel.infrastructure_flags as i16)
        .bind(parcel.rights_count as i16)
        .bind(&access_hash_hex)
        .bind(parcel.created_at as f64)
        .bind(parcel.updated_at as f64)
        .bind(provenance.is_verified())
        .bind(provenance.slot.map(|s| s as i64))
        .bind(provenance.signature.as_deref())
        .fetch_one(pool)
        .await
        .map_err(|e| {
            warn!("insert parcel {pubkey} failed: {e:?}");
            anyhow::anyhow!("insert parcel {pubkey}: {e}")
        })?;

        // Mirror the on-chain ownership into parcel_ownership with the real
        // wallet holder from the Rights PDA.
        sqlx::query("INSERT INTO parcel_ownership (parcel_id, holder) VALUES ($1, $2)")
            .bind(parcel_id)
            .bind(&holder_hex)
            .execute(pool)
            .await
            .map_err(|e| {
                warn!("insert parcel_ownership {pubkey} failed: {e:?}");
                anyhow::anyhow!("insert parcel_ownership {pubkey}: {e}")
            })?;
    }

    Ok(())
}

/// Insert or update a verification_attestation row from on-chain data.
async fn upsert_verification_attestation(
    pool: &PgPool,
    pubkey: &Pubkey,
    attestation: &ParsedVerificationAttestation,
    provenance: &AccountProvenance,
) -> Result<()> {
    let pubkey_str = pubkey.to_string();

    // Convert [u8; 32] to hex strings
    let claim_hex: String = attestation
        .claim
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    let validator_hex: String = attestation
        .validator
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    let observation_hex: String = attestation
        .observation
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    let credential_subject_hex: String = attestation
        .credential_subject
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    let sd_jwt_digest_hex: String = attestation
        .sd_jwt_digest
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();

    // Check whether the row already exists (pubkey is the natural key).
    let exists: Option<i64> =
        sqlx::query_scalar("SELECT id FROM verification_attestations_onchain WHERE pubkey = $1")
            .bind(&pubkey_str)
            .fetch_optional(pool)
            .await
            .map_err(|e| {
                warn!("check existing verification_attestation {pubkey} failed: {e:?}");
                anyhow::anyhow!("check existing verification_attestation {pubkey}: {e}")
            })?;

    if exists.is_some() {
        sqlx::query(
            r#"
            UPDATE verification_attestations_onchain SET
                claim_pubkey = $1,
                validator_pubkey = $2,
                observation_pubkey = $3,
                credential_subject = $4,
                vc_type = $5,
                issuance_date = to_timestamp($6::double precision),
                expiration_date = to_timestamp($7::double precision),
                sd_jwt_digest = $8,
                selective_disclosure = $9,
                protocol_version = $10,
                chain_verified = $11,
                chain_slot = $12,
                chain_signature = $13
            WHERE pubkey = $14
            "#,
        )
        .bind(&claim_hex)
        .bind(&validator_hex)
        .bind(&observation_hex)
        .bind(&credential_subject_hex)
        .bind(&attestation.vc_type)
        .bind(attestation.issuance_date as f64)
        .bind(attestation.expiration_date as f64)
        .bind(&sd_jwt_digest_hex)
        .bind(attestation.selective_disclosure)
        .bind(attestation.protocol_version as i16)
        .bind(provenance.is_verified())
        .bind(provenance.slot.map(|s| s as i64))
        .bind(provenance.signature.as_deref())
        .bind(&pubkey_str)
        .execute(pool)
        .await
        .map_err(|e| {
            warn!("update verification_attestation {pubkey} failed: {e:?}");
            anyhow::anyhow!("update verification_attestation {pubkey}: {e}")
        })?;
    } else {
        sqlx::query(
            r#"
            INSERT INTO verification_attestations_onchain (
                pubkey, claim_pubkey, validator_pubkey, observation_pubkey,
                credential_subject, vc_type, issuance_date, expiration_date,
                sd_jwt_digest, selective_disclosure, protocol_version,
                chain_verified, chain_slot, chain_signature
            ) VALUES (
                $1, $2, $3, $4, $5, $6,
                to_timestamp($7::double precision), to_timestamp($8::double precision),
                $9, $10, $11, $12, $13, $14
            )
            "#,
        )
        .bind(&pubkey_str)
        .bind(&claim_hex)
        .bind(&validator_hex)
        .bind(&observation_hex)
        .bind(&credential_subject_hex)
        .bind(&attestation.vc_type)
        .bind(attestation.issuance_date as f64)
        .bind(attestation.expiration_date as f64)
        .bind(&sd_jwt_digest_hex)
        .bind(attestation.selective_disclosure)
        .bind(attestation.protocol_version as i16)
        .bind(provenance.is_verified())
        .bind(provenance.slot.map(|s| s as i64))
        .bind(provenance.signature.as_deref())
        .execute(pool)
        .await
        .map_err(|e| {
            warn!("insert verification_attestation {pubkey} failed: {e:?}");
            anyhow::anyhow!("insert verification_attestation {pubkey}: {e}")
        })?;
    }

    Ok(())
}

/// Insert or update a claim row from on-chain data.
///
/// The mirror table is created by migration `0057_claims_onchain_mirror.sql`.
async fn upsert_claim(
    pool: &PgPool,
    pubkey: &Pubkey,
    claim: &ParsedClaim,
    provenance: &AccountProvenance,
) -> Result<()> {
    let pubkey_str = pubkey.to_string();
    let claim_id_hex: String = claim.claim_id.iter().map(|b| format!("{b:02x}")).collect();
    let parcel_hex: String = claim.parcel.iter().map(|b| format!("{b:02x}")).collect();
    let submitted_by_hex: String = claim
        .submitted_by
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    let statement_hash_hex: String = claim
        .statement_hash
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();

    let exists: Option<i64> = sqlx::query_scalar("SELECT id FROM claims_onchain WHERE pubkey = $1")
        .bind(&pubkey_str)
        .fetch_optional(pool)
        .await
        .map_err(|e| {
            warn!("check existing claim {pubkey} failed: {e:?}");
            anyhow::anyhow!("check existing claim {pubkey}: {e}")
        })?;

    if let Some(_id) = exists {
        sqlx::query(
            r#"
            UPDATE claims_onchain SET
                claim_id = $1,
                parcel_pubkey = $2,
                claim_type = $3,
                submitted_by = $4,
                status = $5,
                statement_hash = $6,
                version = $7,
                evidence_count = $8,
                observation_count = $9,
                attestation_count = $10,
                required_attestations = $11,
                chain_created_at = to_timestamp($12::double precision),
                chain_updated_at = to_timestamp($13::double precision),
                chain_verified = $14,
                chain_slot = $15,
                chain_signature = $16,
                mirrored_at = now()
            WHERE pubkey = $17
            "#,
        )
        .bind(&claim_id_hex)
        .bind(&parcel_hex)
        .bind(claim.claim_type as i16)
        .bind(&submitted_by_hex)
        .bind(claim.status as i16)
        .bind(&statement_hash_hex)
        .bind(claim.version as i16)
        .bind(claim.evidence_count as i16)
        .bind(claim.observation_count as i16)
        .bind(claim.attestation_count as i16)
        .bind(claim.required_attestations as i16)
        .bind(claim.created_at as f64)
        .bind(claim.updated_at as f64)
        .bind(provenance.is_verified())
        .bind(provenance.slot.map(|s| s as i64))
        .bind(provenance.signature.as_deref())
        .bind(&pubkey_str)
        .execute(pool)
        .await
        .map_err(|e| {
            warn!("update claim {pubkey} failed: {e:?}");
            anyhow::anyhow!("update claim {pubkey}: {e}")
        })?;
    } else {
        sqlx::query(
            r#"
            INSERT INTO claims_onchain (
                pubkey, claim_id, parcel_pubkey, claim_type, submitted_by,
                status, statement_hash, version, evidence_count,
                observation_count, attestation_count, required_attestations,
                chain_created_at, chain_updated_at,
                chain_verified, chain_slot, chain_signature, mirrored_at
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12,
                to_timestamp($13::double precision), to_timestamp($14::double precision),
                $15, $16, $17, now()
            )
            "#,
        )
        .bind(&pubkey_str)
        .bind(&claim_id_hex)
        .bind(&parcel_hex)
        .bind(claim.claim_type as i16)
        .bind(&submitted_by_hex)
        .bind(claim.status as i16)
        .bind(&statement_hash_hex)
        .bind(claim.version as i16)
        .bind(claim.evidence_count as i16)
        .bind(claim.observation_count as i16)
        .bind(claim.attestation_count as i16)
        .bind(claim.required_attestations as i16)
        .bind(claim.created_at as f64)
        .bind(claim.updated_at as f64)
        .bind(provenance.is_verified())
        .bind(provenance.slot.map(|s| s as i64))
        .bind(provenance.signature.as_deref())
        .execute(pool)
        .await
        .map_err(|e| {
            warn!("insert claim {pubkey} failed: {e:?}");
            anyhow::anyhow!("insert claim {pubkey}: {e}")
        })?;
    }

    Ok(())
}

/// Insert or update a spatial_asset row from on-chain data.
async fn upsert_spatial_asset(
    pool: &PgPool,
    pubkey: &Pubkey,
    asset: &ParsedSpatialAsset,
    provenance: &AccountProvenance,
) -> Result<()> {
    let pubkey_str = pubkey.to_string();
    let parcel_hex: String = asset.parcel.iter().map(|b| format!("{b:02x}")).collect();
    let authority_hex: String = asset.authority.iter().map(|b| format!("{b:02x}")).collect();
    let latest_geometry_hex: String = asset
        .latest_geometry
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();

    let exists: Option<i64> =
        sqlx::query_scalar("SELECT id FROM spatial_assets_onchain WHERE pubkey = $1")
            .bind(&pubkey_str)
            .fetch_optional(pool)
            .await
            .map_err(|e| {
                warn!("check existing spatial_asset {pubkey} failed: {e:?}");
                anyhow::anyhow!("check existing spatial_asset {pubkey}: {e}")
            })?;

    if let Some(_id) = exists {
        sqlx::query(
            r#"
            UPDATE spatial_assets_onchain SET
                parcel_pubkey = $1,
                authority = $2,
                dimensionality = $3,
                elevation_min_mm = $4,
                elevation_max_mm = $5,
                geometry_version_count = $6,
                latest_geometry_hash = $7,
                chain_created_at = to_timestamp($8::double precision),
                chain_updated_at = to_timestamp($9::double precision),
                chain_verified = $10,
                chain_slot = $11,
                chain_signature = $12,
                mirrored_at = now()
            WHERE pubkey = $13
            "#,
        )
        .bind(&parcel_hex)
        .bind(&authority_hex)
        .bind(asset.dimensionality as i16)
        .bind(asset.elevation_min_mm as i32)
        .bind(asset.elevation_max_mm as i32)
        .bind(asset.geometry_version_count as i32)
        .bind(&latest_geometry_hex)
        .bind(asset.created_at as f64)
        .bind(asset.updated_at as f64)
        .bind(provenance.is_verified())
        .bind(provenance.slot.map(|s| s as i64))
        .bind(provenance.signature.as_deref())
        .bind(&pubkey_str)
        .execute(pool)
        .await
        .map_err(|e| {
            warn!("update spatial_asset {pubkey} failed: {e:?}");
            anyhow::anyhow!("update spatial_asset {pubkey}: {e}")
        })?;
    } else {
        sqlx::query(
            r#"
            INSERT INTO spatial_assets_onchain (
                pubkey, parcel_pubkey, authority, dimensionality,
                elevation_min_mm, elevation_max_mm, geometry_version_count,
                latest_geometry_hash, chain_created_at, chain_updated_at,
                chain_verified, chain_slot, chain_signature, mirrored_at
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8,
                to_timestamp($9::double precision), to_timestamp($10::double precision),
                $11, $12, $13, now()
            )
            "#,
        )
        .bind(&pubkey_str)
        .bind(&parcel_hex)
        .bind(&authority_hex)
        .bind(asset.dimensionality as i16)
        .bind(asset.elevation_min_mm as i32)
        .bind(asset.elevation_max_mm as i32)
        .bind(asset.geometry_version_count as i32)
        .bind(&latest_geometry_hex)
        .bind(asset.created_at as f64)
        .bind(asset.updated_at as f64)
        .bind(provenance.is_verified())
        .bind(provenance.slot.map(|s| s as i64))
        .bind(provenance.signature.as_deref())
        .execute(pool)
        .await
        .map_err(|e| {
            warn!("insert spatial_asset {pubkey} failed: {e:?}");
            anyhow::anyhow!("insert spatial_asset {pubkey}: {e}")
        })?;
    }

    Ok(())
}

/// Insert or update a geometry_version row from on-chain data.
async fn upsert_geometry_version(
    pool: &PgPool,
    pubkey: &Pubkey,
    version: &ParsedGeometryVersion,
    provenance: &AccountProvenance,
) -> Result<()> {
    let pubkey_str = pubkey.to_string();
    // `geometry_versions_onchain.asset_pubkey` is an FK to
    // `spatial_assets_onchain.pubkey`, which is base58. The on-chain field is
    // raw bytes; hex encoding here silently broke every geometry insert.
    let asset_pubkey = Pubkey::new_from_array(version.asset).to_string();
    let parcel_hex: String = version.parcel.iter().map(|b| format!("{b:02x}")).collect();
    let geometry_hash_hex: String = version
        .geometry_hash
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    let submitted_by_hex: String = version
        .submitted_by
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    let verified_by_hex: String = version
        .verified_by
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    let evidence_manifest_hex: String = version
        .evidence_manifest
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    let attestors_hex: Vec<String> = version.attestors.to_vec();

    let exists: Option<i64> =
        sqlx::query_scalar("SELECT id FROM geometry_versions_onchain WHERE pubkey = $1")
            .bind(&pubkey_str)
            .fetch_optional(pool)
            .await
            .map_err(|e| {
                warn!("check existing geometry_version {pubkey} failed: {e:?}");
                anyhow::anyhow!("check existing geometry_version {pubkey}: {e}")
            })?;

    if let Some(_id) = exists {
        sqlx::query(
            r#"
            UPDATE geometry_versions_onchain SET
                asset_pubkey = $1,
                parcel_pubkey = $2,
                version = $3,
                geometry_hash = $4,
                source = $5,
                dimension = $6,
                storage_reference = $7,
                submitted_by = $8,
                chain_submitted_at = to_timestamp($9::double precision),
                verified = $10,
                verified_by = $11,
                chain_verified_at = to_timestamp($12::double precision),
                evidence_manifest = $13,
                elevation_min_mm = $14,
                elevation_max_mm = $15,
                elevation_source = $16,
                required = $17,
                attest_count = $18,
                attestors = $19,
                chain_verified = $20,
                chain_slot = $21,
                chain_signature = $22,
                mirrored_at = now()
            WHERE pubkey = $23
            "#,
        )
        .bind(&asset_pubkey)
        .bind(&parcel_hex)
        .bind(version.version as i32)
        .bind(&geometry_hash_hex)
        .bind(version.source as i16)
        .bind(version.dimension as i16)
        .bind(&version.storage_reference)
        .bind(&submitted_by_hex)
        .bind(version.submitted_at as f64)
        .bind(version.verified)
        .bind(&verified_by_hex)
        .bind(version.verified_at as f64)
        .bind(&evidence_manifest_hex)
        .bind(version.elevation_min_mm as i32)
        .bind(version.elevation_max_mm as i32)
        .bind(version.elevation_source as i16)
        .bind(version.required as i16)
        .bind(version.attest_count as i16)
        .bind(&attestors_hex)
        .bind(provenance.is_verified())
        .bind(provenance.slot.map(|s| s as i64))
        .bind(provenance.signature.as_deref())
        .bind(&pubkey_str)
        .execute(pool)
        .await
        .map_err(|e| {
            warn!("update geometry_version {pubkey} failed: {e:?}");
            anyhow::anyhow!("update geometry_version {pubkey}: {e}")
        })?;
    } else {
        sqlx::query(
            r#"
            INSERT INTO geometry_versions_onchain (
                pubkey, asset_pubkey, parcel_pubkey, version, geometry_hash,
                source, dimension, storage_reference, submitted_by,
                chain_submitted_at, verified, verified_by,
                chain_verified_at, evidence_manifest, elevation_min_mm,
                elevation_max_mm, elevation_source, required, attest_count,
                attestors, chain_verified, chain_slot, chain_signature, mirrored_at
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9,
                to_timestamp($10::double precision), $11, $12,
                to_timestamp($13::double precision), $14, $15, $16, $17, $18, $19,
                $20, $21, $22, $23, now()
            )
            "#,
        )
        .bind(&pubkey_str)
        .bind(&asset_pubkey)
        .bind(&parcel_hex)
        .bind(version.version as i32)
        .bind(&geometry_hash_hex)
        .bind(version.source as i16)
        .bind(version.dimension as i16)
        .bind(&version.storage_reference)
        .bind(&submitted_by_hex)
        .bind(version.submitted_at as f64)
        .bind(version.verified)
        .bind(&verified_by_hex)
        .bind(version.verified_at as f64)
        .bind(&evidence_manifest_hex)
        .bind(version.elevation_min_mm as i32)
        .bind(version.elevation_max_mm as i32)
        .bind(version.elevation_source as i16)
        .bind(version.required as i16)
        .bind(version.attest_count as i16)
        .bind(&attestors_hex)
        .bind(provenance.is_verified())
        .bind(provenance.slot.map(|s| s as i64))
        .bind(provenance.signature.as_deref())
        .execute(pool)
        .await
        .map_err(|e| {
            warn!("insert geometry_version {pubkey} failed: {e:?}");
            anyhow::anyhow!("insert geometry_version {pubkey}: {e}")
        })?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sha2::{Digest, Sha256};

    fn signature_entry(
        signature: &str,
        slot: u64,
        failed: bool,
    ) -> RpcConfirmedTransactionStatusWithSignature {
        RpcConfirmedTransactionStatusWithSignature {
            signature: signature.to_string(),
            slot,
            err: if failed {
                Some(
                    serde_json::from_value(serde_json::json!("AccountNotFound"))
                        .expect("UiTransactionError deserializes from variant name"),
                )
            } else {
                None
            },
            memo: None,
            block_time: None,
            confirmation_status: None,
        }
    }

    #[test]
    fn claim_discriminator_matches_sha256_account_claim() {
        let digest = Sha256::digest(b"account:Claim");
        assert_eq!(
            CLAIM_DISCRIMINATOR,
            [
                digest[0], digest[1], digest[2], digest[3],
                digest[4], digest[5], digest[6], digest[7],
            ],
            "indexer must match the real Anchor Claim discriminator"
        );
    }

    #[test]
    fn provenance_skips_failed_transactions_and_uses_their_slot() {
        // RPC returns newest-first; the newest entry failed, so it cannot
        // prove the mirrored state. The next successful entry is the anchor.
        let entries = vec![
            signature_entry("failed-sig", 900, true),
            signature_entry("real-sig", 842, false),
        ];
        let provenance = provenance_from_signatures(&entries);
        assert!(provenance.is_verified());
        assert_eq!(provenance.signature.as_deref(), Some("real-sig"));
        assert_eq!(provenance.slot, Some(842));
    }

    #[test]
    fn provenance_without_signatures_is_unverified() {
        let provenance = provenance_from_signatures(&[]);
        assert!(!provenance.is_verified());
        assert_eq!(provenance.signature, None);
        assert_eq!(provenance.slot, None);
    }

    #[test]
    fn classify_account_ignores_unknown_and_truncated_buffers() {
        assert!(classify_account(b"not an anchor account").is_none());
        // A valid discriminator with a truncated body must return None rather
        // than panicking: a malformed account must be isolated, not crash a poll.
        let mut truncated = CLAIM_DISCRIMINATOR.to_vec();
        truncated.extend_from_slice(&[0u8; 7]);
        assert!(classify_account(&truncated).is_none());
    }

    async fn test_pool() -> Option<PgPool> {
        let Ok(database_url) = std::env::var("DATABASE_URL") else {
            eprintln!("skipping DB-backed indexer test: DATABASE_URL is not set");
            return None;
        };
        Some(
            sqlx::PgPool::connect(&database_url)
                .await
                .expect("connect test database"),
        )
    }

    #[tokio::test]
    async fn claims_mirror_has_provenance_columns() {
        let Some(pool) = test_pool().await else { return };
        let columns: Vec<String> = sqlx::query_scalar(
            "SELECT column_name FROM information_schema.columns \
             WHERE table_name = 'claims_onchain'",
        )
        .fetch_all(&pool)
        .await
        .expect("query claims_onchain columns");
        for column in [
            "pubkey",
            "chain_verified",
            "chain_slot",
            "chain_signature",
            "mirror_hash",
        ] {
            assert!(
                columns.iter().any(|c| c == column),
                "claims_onchain must have a {column} column (migration 0057)"
            );
        }
        pool.close().await;
    }

    fn test_claim() -> ParsedClaim {
        ParsedClaim {
            claim_id: [1u8; 32],
            parcel: [2u8; 32],
            claim_type: 1,
            submitted_by: [3u8; 32],
            status: 1,
            statement_hash: [4u8; 32],
            version: 2,
            evidence_count: 3,
            observation_count: 4,
            attestation_count: 5,
            required_attestations: 6,
            created_at: 1_700_000_000,
            updated_at: 1_700_000_100,
        }
    }

    #[tokio::test]
    async fn upsert_claim_writes_real_transaction_provenance() {
        let Some(pool) = test_pool().await else { return };
        let pubkey = Pubkey::new_unique();
        let key = pubkey.to_string();
        let claim = test_claim();
        let verified = AccountProvenance::verified(842, "test-claim-signature".to_string());

        upsert_claim(&pool, &pubkey, &claim, &verified)
            .await
            .expect("upsert claim with provenance");

        let (chain_verified, chain_slot, chain_signature): (bool, Option<i64>, Option<String>) =
            sqlx::query_as(
                "SELECT chain_verified, chain_slot, chain_signature \
                 FROM claims_onchain WHERE pubkey = $1",
            )
            .bind(&key)
            .fetch_one(&pool)
            .await
            .expect("read claim provenance");
        assert!(chain_verified);
        assert_eq!(chain_slot, Some(842));
        assert_eq!(chain_signature.as_deref(), Some("test-claim-signature"));

        // A failed signature lookup must downgrade, not fabricate provenance.
        upsert_claim(&pool, &pubkey, &claim, &AccountProvenance::unverified())
            .await
            .expect("upsert claim without provenance");
        let (chain_verified, chain_slot, chain_signature): (bool, Option<i64>, Option<String>) =
            sqlx::query_as(
                "SELECT chain_verified, chain_slot, chain_signature \
                 FROM claims_onchain WHERE pubkey = $1",
            )
            .bind(&key)
            .fetch_one(&pool)
            .await
            .expect("read downgraded claim provenance");
        assert!(!chain_verified);
        assert_eq!(chain_slot, None);
        assert_eq!(chain_signature, None);

        sqlx::query("DELETE FROM claims_onchain WHERE pubkey = $1")
            .bind(&key)
            .execute(&pool)
            .await
            .expect("delete test claim");
        pool.close().await;
    }

    fn test_spatial_asset() -> ParsedSpatialAsset {
        ParsedSpatialAsset {
            parcel: [5u8; 32],
            authority: [6u8; 32],
            dimensionality: 2,
            elevation_min_mm: 100,
            elevation_max_mm: 900,
            geometry_version_count: 1,
            latest_geometry: [7u8; 32],
            created_at: 1_700_000_000,
            updated_at: 1_700_000_100,
        }
    }

    fn test_geometry_version(asset: &Pubkey) -> ParsedGeometryVersion {
        ParsedGeometryVersion {
            asset: asset.to_bytes(),
            parcel: [5u8; 32],
            version: 1,
            geometry_hash: [8u8; 32],
            source: 1,
            dimension: 2,
            storage_reference: "test://geometry".to_string(),
            submitted_by: [9u8; 32],
            submitted_at: 1_700_000_000,
            verified: true,
            verified_by: [10u8; 32],
            verified_at: 1_700_000_100,
            evidence_manifest: [11u8; 32],
            elevation_min_mm: 100,
            elevation_max_mm: 900,
            elevation_source: 1,
            required: 1,
            attestors: vec![],
            attest_count: 0,
        }
    }

    #[tokio::test]
    async fn spatial_mirrors_set_chain_verified_from_transaction_provenance() {
        let Some(pool) = test_pool().await else { return };
        let asset_key = Pubkey::new_unique();
        let version_key = Pubkey::new_unique();
        let asset_str = asset_key.to_string();
        let version_str = version_key.to_string();
        let provenance = AccountProvenance::verified(1234, "test-spatial-signature".to_string());

        let asset = test_spatial_asset();
        upsert_spatial_asset(&pool, &asset_key, &asset, &provenance)
            .await
            .expect("upsert spatial asset");
        let version = test_geometry_version(&asset_key);
        upsert_geometry_version(&pool, &version_key, &version, &provenance)
            .await
            .expect("upsert geometry version");

        for (table, key) in [
            ("spatial_assets_onchain", &asset_str),
            ("geometry_versions_onchain", &version_str),
        ] {
            let query = format!(
                "SELECT chain_verified, chain_slot, chain_signature FROM {table} WHERE pubkey = $1"
            );
            let (verified, slot, signature): (bool, Option<i64>, Option<String>) =
                sqlx::query_as(&query)
                    .bind(key)
                    .fetch_one(&pool)
                    .await
                    .expect("read spatial provenance");
            assert!(verified, "{table} must be chain-verified when a tx is resolved");
            assert_eq!(slot, Some(1234), "{table} must store the tx slot");
            assert_eq!(signature.as_deref(), Some("test-spatial-signature"));
        }

        sqlx::query("DELETE FROM geometry_versions_onchain WHERE pubkey = $1")
            .bind(&version_str)
            .execute(&pool)
            .await
            .expect("delete test geometry version");
        sqlx::query("DELETE FROM spatial_assets_onchain WHERE pubkey = $1")
            .bind(&asset_str)
            .execute(&pool)
            .await
            .expect("delete test spatial asset");
        pool.close().await;
    }
}
