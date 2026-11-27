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
use chrono::{DateTime, Utc};
use sha2::{Digest, Sha256};
use solana_account_decoder::{UiAccountEncoding, UiDataSliceConfig};
use solana_client::rpc_client::{GetConfirmedSignaturesForAddress2Config, RpcClient};
use solana_client::rpc_config::{RpcAccountInfoConfig, RpcProgramAccountsConfig};
use solana_client::rpc_response::RpcConfirmedTransactionStatusWithSignature;
use solana_sdk::pubkey::Pubkey;
use sqlx::PgPool;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::indexer_decoders::*;
use crate::state::AppState;

/// terra_registry program ID (mirrors terra-web/src/lib/constants.ts).
const PROGRAM_ID: &str = "GaEDbktvpZ3qiqp4PmFgHwDSa6JsFfVjXFqNb2nTbage";

/// Rights account kind: OWNERSHIP = 0 (terra_registry `right_kind`).
const RIGHTS_KIND_OWNERSHIP: u8 = 0;

/// Runtime knobs for the metered hosted RPC. Defaults are deliberately
/// conservative; all four are env-overridable in `config.rs`.
#[derive(Debug, Clone)]
pub struct IndexerOptions {
    pub poll_interval: Duration,
    pub rpc_timeout: Duration,
    pub max_backoff: Duration,
    pub account_batch_size: usize,
}

impl Default for IndexerOptions {
    fn default() -> Self {
        Self {
            poll_interval: Duration::from_secs(60),
            rpc_timeout: Duration::from_secs(30),
            max_backoff: Duration::from_secs(300),
            account_batch_size: 100,
        }
    }
}

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
        Self {
            slot: None,
            signature: None,
        }
    }

    /// ADR-026: an empty (or whitespace-only) signature is not a real
    /// transaction. Normalize it to the fully-unverified state so every
    /// provenance-binding query stores FALSE/NULL/NULL, never `Some("")`.
    fn verified(slot: u64, signature: String) -> Self {
        let signature = signature.trim().to_string();
        if signature.is_empty() {
            return Self::unverified();
        }
        Self {
            slot: Some(slot),
            signature: Some(signature),
        }
    }

    fn is_verified(&self) -> bool {
        self.slot.is_some()
            && self
                .signature
                .as_ref()
                .is_some_and(|signature| !signature.trim().is_empty())
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
fn resolve_provenance(client: &RpcClient, pubkey: &Pubkey) -> AccountProvenance {
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

/// A decoded account that has a mirror table. Byte layouts and discriminators
/// come from the IDL-generated decoders in `indexer_decoders.rs`; this file
/// never hand-rolls a Borsh offset (R-042).
enum MirroredAccount {
    Parcel(ParsedParcel),
    Attestation(ParsedVerificationAttestation),
    Rights(ParsedRights),
    Claim(ParsedClaim),
    SpatialAsset(ParsedSpatialAsset),
    // Boxed: this decoded account is much larger than the other variants and
    // is only held for one mirror write (clippy::large_enum_variant).
    GeometryVersion(Box<ParsedGeometryVersion>),
    ThreeDModel(ParsedThreeDModel),
    SpatialSnapshot(ParsedSpatialSnapshot),
    /// Q-035(a) typed projection of the content-addressed geometry artifact.
    GeometryArtifact(ParsedParcelGeometryArtifact),
}

fn classify_account(data: &[u8]) -> Option<MirroredAccount> {
    if let Some(parcel) = parse_parcel(data) {
        Some(MirroredAccount::Parcel(parcel))
    } else if let Some(attestation) = parse_verification_attestation(data) {
        Some(MirroredAccount::Attestation(attestation))
    } else if let Some(rights) = parse_rights(data) {
        Some(MirroredAccount::Rights(rights))
    } else if let Some(claim) = parse_claim(data) {
        Some(MirroredAccount::Claim(claim))
    } else if let Some(asset) = parse_spatial_asset(data) {
        Some(MirroredAccount::SpatialAsset(asset))
    } else if let Some(version) = parse_geometry_version(data) {
        Some(MirroredAccount::GeometryVersion(Box::new(version)))
    } else if let Some(model) = parse_three_d_model(data) {
        Some(MirroredAccount::ThreeDModel(model))
    } else if let Some(snapshot) = parse_spatial_snapshot(data) {
        Some(MirroredAccount::SpatialSnapshot(snapshot))
    } else {
        parse_parcel_geometry_artifact(data).map(MirroredAccount::GeometryArtifact)
    }
}

/// Accounts whose mirror rows have a foreign key to another mirrored row
/// (parcel or spatial asset). They are mirrored in the second poll phase so
/// the parent row exists regardless of RPC account order.
fn is_dependent(account: &MirroredAccount) -> bool {
    matches!(
        account,
        MirroredAccount::Rights(_)
            | MirroredAccount::GeometryVersion(_)
            | MirroredAccount::ThreeDModel(_)
            | MirroredAccount::SpatialSnapshot(_)
    )
}

/// Header-only discriminator filter for phase 1 of the RPC read.
fn is_mirrored_header(header: &[u8]) -> bool {
    header.len() == 8 && INDEXER_DISCRIMINATORS.iter().any(|d| &d[..] == header)
}

/// Mirror one decoded account. Errors are returned to the caller, which
/// isolates them per account instead of aborting the whole poll.
///
/// `dependent_phase = false` mirrors standalone accounts; `true` mirrors
/// accounts that need a parcel/spatial-asset row written in phase 1.
async fn mirror_account(
    pool: &PgPool,
    storage: &crate::storage::StorageBackend,
    client: &RpcClient,
    pubkey: &Pubkey,
    data: &[u8],
    parcel_holder: &HashMap<[u8; 32], [u8; 32]>,
    dependent_phase: bool,
) -> Result<bool> {
    // Q-033 stage 1: every IDL account type gets a generic, provenance-tagged
    // mirror row. The typed projections below are an additional stage-0 view
    // for the eight business-critical types; an account with no typed table is
    // still chain-derived, not client-trust only.
    let Some((account_name, fields)) = decode_any(data) else {
        return Ok(false);
    };
    let specialized = classify_account(data);
    if specialized.as_ref().is_some_and(is_dependent) != dependent_phase {
        return Ok(false);
    }
    let provenance = resolve_provenance(client, pubkey);
    upsert_chain_account(pool, pubkey, account_name, &fields, &provenance).await?;
    let Some(decoded) = specialized else {
        return Ok(true);
    };
    match decoded {
        MirroredAccount::Parcel(parcel) => {
            // `Rights.parcel` is the Parcel PDA pubkey (terra_registry sets
            // `rights.parcel = parcel.key()`), and `pubkey` here IS that PDA.
            // The old code looked up `parcel.id` (a caller-derived hash), so
            // it never matched and fell back to the PDA as a fake wallet.
            let holder = parcel_holder.get(&pubkey.to_bytes()).copied();
            // Q-035(a): a verified artifact is the only honest source of
            // geometry for a parcel that was never submitted through the API.
            let artifact = verified_geometry_artifact(
                pool,
                storage,
                pubkey,
                &hex::encode(parcel.geometry_hash),
            )
            .await?;
            upsert_parcel(
                pool,
                pubkey,
                &parcel,
                holder.as_ref(),
                &provenance,
                artifact.as_ref(),
            )
            .await?;
        }
        MirroredAccount::Attestation(attestation) => {
            upsert_verification_attestation(pool, pubkey, &attestation, &provenance).await?;
        }
        MirroredAccount::Rights(rights) => {
            let parcel_pda = Pubkey::new_from_array(rights.parcel);
            let Some(parcel_id) = parcel_id_for_pda(pool, &parcel_pda).await? else {
                warn!(%parcel_pda, "rights mirror skipped: no parcel row with this onchain_id");
                return Ok(false);
            };
            upsert_rights(pool, pubkey, &rights, parcel_id, &provenance).await?;
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
        MirroredAccount::ThreeDModel(model) => {
            upsert_three_d_model(pool, pubkey, &model, &provenance).await?;
        }
        MirroredAccount::SpatialSnapshot(snapshot) => {
            upsert_spatial_snapshot(pool, pubkey, &snapshot, &provenance).await?;
        }
        MirroredAccount::GeometryArtifact(artifact) => {
            upsert_parcel_geometry_artifact(pool, pubkey, &artifact, &provenance).await?;
        }
    }
    Ok(true)
}

/// Run the indexer loop: poll RPC, decode accounts, upsert mirror rows.
///
/// `getProgramAccounts` has no cursor/offset in Solana RPC 3.1.12, so the
/// read is two-phase: a header-only scan (8-byte discriminator) followed by
/// batched `getMultipleAccounts` for the matching pubkeys. Failures back off
/// exponentially up to `max_backoff`; successful polls reset the backoff.
pub async fn run_indexer(
    rpc_url: String,
    state: Arc<AppState>,
    options: IndexerOptions,
) -> Result<()> {
    let client = RpcClient::new_with_timeout(rpc_url.clone(), options.rpc_timeout);
    let program_id = Pubkey::from_str(PROGRAM_ID)
        .with_context(|| format!("invalid program id: {PROGRAM_ID}"))?;

    info!(
        %PROGRAM_ID,
        %rpc_url,
        poll_secs = options.poll_interval.as_secs(),
        batch_size = options.account_batch_size,
        "chain indexer started"
    );

    let mut consecutive_failures = 0u32;
    loop {
        match poll_once(&client, &program_id, &state, options.account_batch_size).await {
            Ok(count) => {
                consecutive_failures = 0;
                if count > 0 {
                    info!(count, "indexer poll: accounts upserted");
                }
                sleep(options.poll_interval).await;
            }
            Err(err) => {
                consecutive_failures = consecutive_failures.saturating_add(1);
                // poll_interval * 2^(n-1), capped. Shift is capped at 8 so
                // even a long outage cannot overflow the duration.
                let shift = consecutive_failures.saturating_sub(1).min(8);
                let delay = options
                    .poll_interval
                    .saturating_mul(1u32 << shift)
                    .min(options.max_backoff);
                warn!(
                    %err,
                    consecutive_failures,
                    backoff_secs = delay.as_secs(),
                    "indexer poll failed; backing off"
                );
                sleep(delay).await;
            }
        }
    }
}

/// Read only the 8-byte Anchor discriminator for every program-owned account,
/// then fetch full data in bounded batches for the accounts we mirror.
///
/// This avoids pulling every byte of unrelated program accounts on each poll.
/// True cursor pagination is not available in Solana RPC 3.1.12; the missing
/// capability is recorded as a founder/vendor decision (Q-034).
fn fetch_mirrored_accounts(
    client: &RpcClient,
    program_id: &Pubkey,
    batch_size: usize,
) -> Result<Vec<(Pubkey, Vec<u8>)>> {
    let config = RpcProgramAccountsConfig {
        filters: None,
        account_config: RpcAccountInfoConfig {
            encoding: Some(UiAccountEncoding::Base64),
            data_slice: Some(UiDataSliceConfig {
                offset: 0,
                length: 8,
            }),
            ..Default::default()
        },
        with_context: None,
        sort_results: None,
    };
    let headers = client
        .get_program_ui_accounts_with_config(program_id, config)
        .with_context(|| "header-only program account scan failed")?;

    let mut keys = Vec::with_capacity(headers.len());
    for (pubkey, ui_account) in headers {
        if ui_account
            .data
            .decode()
            .is_some_and(|header| is_mirrored_header(&header))
        {
            keys.push(pubkey);
        }
    }

    let batch_size = batch_size.max(1);
    let mut accounts = Vec::with_capacity(keys.len());
    for chunk in keys.chunks(batch_size) {
        let fetched = client
            .get_multiple_accounts(chunk)
            .with_context(|| "batched full-account fetch failed")?;
        for (pubkey, account) in chunk.iter().zip(fetched) {
            if let Some(account) = account {
                accounts.push((*pubkey, account.data));
            }
        }
    }
    Ok(accounts)
}

/// Single poll: fetch program accounts, mirror each decoded account with
/// transaction-level provenance.
///
/// Poll-level RPC failures still abort the poll (nothing can be mirrored).
/// Per-account decode/DB failures are isolated: they are logged and skipped
/// so later accounts keep mirroring (R-039).
async fn poll_once(
    client: &RpcClient,
    program_id: &Pubkey,
    state: &AppState,
    batch_size: usize,
) -> Result<usize> {
    let accounts = fetch_mirrored_accounts(client, program_id, batch_size)?;

    // Rights.parcel is the parcel PDA. Build the ownership map from the
    // on-chain Rights accounts so parcels get a real wallet holder, never
    // the PDA address as a fake wallet (R-043).
    let mut parcel_holder: HashMap<[u8; 32], [u8; 32]> = HashMap::new();
    for (_pubkey, data) in &accounts {
        if let Some(rights) = parse_rights(data)
            && rights.rights_kind == RIGHTS_KIND_OWNERSHIP
        {
            parcel_holder.insert(rights.parcel, rights.holder);
        }
    }

    let mut upserted = 0usize;
    let mut failed = 0usize;
    // Phase 1: standalone rows. Phase 2: rows with FK dependencies.
    for dependent_phase in [false, true] {
        for (pubkey, data) in &accounts {
            match mirror_account(
                &state.pool,
                &state.storage,
                client,
                pubkey,
                data,
                &parcel_holder,
                dependent_phase,
            )
            .await
            {
                Ok(true) => upserted += 1,
                Ok(false) => {}
                Err(err) => {
                    warn!(
                        %pubkey,
                        account = mirrored_account_name(data).unwrap_or("unknown"),
                        error = %err,
                        "indexer account mirror failed; continuing poll"
                    );
                    failed += 1;
                }
            }
        }
    }
    if failed > 0 {
        info!(upserted, failed, "indexer poll: isolated account failures");
    }

    Ok(upserted)
}

/// Recompute the digest the registration client anchored on-chain.
///
/// `RegisterParcelPanel` computes `sha256(JSON.stringify(ring))` where `ring`
/// is `geometry.coordinates[0]`. We keep the submitted GeoJSON verbatim
/// (migration 0059) because PostGIS normalizes numbers and drops the original
/// bytes; `ST_AsGeoJSON` can never reproduce the client's hash. serde_json
/// emits the same compact numeric-array form as `JSON.stringify` for these
/// longitude/latitude values.
pub(crate) fn submitted_ring_hash(source_geometry: &str) -> Option<[u8; 32]> {
    let value: serde_json::Value = serde_json::from_str(source_geometry).ok()?;
    let ring = value.get("coordinates")?.get(0)?;
    if !ring.is_array() {
        return None;
    }
    let compact = serde_json::to_string(ring).ok()?;
    Some(Sha256::digest(compact.as_bytes()).into())
}

/// Upsert the generic mirror row for any IDL account type (Q-033 stage 1).
///
/// `fields` is the IDL-derived JSON view emitted by `decode_any()`. Its
/// canonical `jsonb` form is hash-triggered in migration 0060. Provenance
/// follows ADR-026: without a real transaction the row stays unverified with
/// NULL slot/signature.
async fn upsert_chain_account(
    pool: &PgPool,
    pubkey: &Pubkey,
    account_name: &str,
    fields: &serde_json::Value,
    provenance: &AccountProvenance,
) -> Result<()> {
    let pubkey_str = pubkey.to_string();
    let fields_json = fields.to_string();
    let verified = provenance.is_verified();
    let chain_slot = if verified {
        provenance.slot.map(|s| s as i64)
    } else {
        None
    };
    let chain_signature = if verified {
        provenance.signature.as_deref()
    } else {
        None
    };

    sqlx::query(
        r#"
        INSERT INTO chain_accounts (
            pubkey, account_type, fields, chain_verified, chain_slot,
            chain_signature, mirrored_at
        ) VALUES ($1, $2, $3::jsonb, $4, $5, $6, now())
        ON CONFLICT (pubkey) DO UPDATE SET
            account_type = EXCLUDED.account_type,
            fields = EXCLUDED.fields,
            chain_verified = EXCLUDED.chain_verified,
            chain_slot = EXCLUDED.chain_slot,
            chain_signature = EXCLUDED.chain_signature,
            mirrored_at = now()
        "#,
    )
    .bind(&pubkey_str)
    .bind(account_name)
    .bind(&fields_json)
    .bind(verified)
    .bind(chain_slot)
    .bind(chain_signature)
    .execute(pool)
    .await
    .with_context(|| format!("generic chain mirror failed for {pubkey} ({account_name})"))?;
    Ok(())
}

/// A geometry artifact that passed every content-address check. Only this
/// shape may be used to create a chain-only parcel row.
#[derive(Debug, Clone)]
struct VerifiedGeometryArtifact {
    source_geometry: String,
}

/// Resolve the on-chain artifact URI through the API's storage backend and
/// re-derive both digests. Returns `None` (fail closed) unless:
///   1. the artifact PDA is chain-verified with a real tx signature;
///   2. the on-chain URI is exactly the stored content-addressed ref;
///   3. sha256(bytes) == the on-chain content_hash;
///   4. sha256(JSON.stringify(coordinates[0])) == Parcel.geometry_hash.
async fn verified_geometry_artifact(
    pool: &PgPool,
    storage: &crate::storage::StorageBackend,
    parcel_pda: &Pubkey,
    expected_geometry_hash: &str,
) -> Result<Option<VerifiedGeometryArtifact>> {
    let pubkey_str = parcel_pda.to_string();
    let pubkey_hex = hex::encode(parcel_pda.to_bytes());
    let row: Option<(String, String, String)> = sqlx::query_as(
        r#"
        SELECT u.storage_ref, u.content_hash, u.geometry_hash
        FROM parcel_geometry_uploads u
        JOIN parcel_geometry_artifacts a
          ON a.parcel_pubkey = u.parcel_pubkey
         AND a.content_hash = u.content_hash
        WHERE (u.parcel_pubkey = $1 OR u.parcel_pubkey = $2)
          AND a.chain_verified = TRUE
          AND a.storage_uri = u.storage_ref
        ORDER BY u.created_at DESC
        LIMIT 1
        "#,
    )
    .bind(&pubkey_str)
    .bind(&pubkey_hex)
    .fetch_optional(pool)
    .await
    .with_context(|| format!("geometry artifact lookup failed for {parcel_pda}"))?;
    let Some((storage_ref, content_hash, uploaded_ring_hash)) = row else {
        return Ok(None);
    };

    let Some(bytes) = storage
        .load(&storage_ref)
        .await
        .with_context(|| format!("geometry artifact bytes unavailable for {parcel_pda}"))?
    else {
        warn!(%parcel_pda, %storage_ref, "geometry artifact bytes missing");
        return Ok(None);
    };

    let actual_content_hash = hex::encode(Sha256::digest(&bytes));
    if actual_content_hash != content_hash {
        warn!(
            %parcel_pda,
            expected = %content_hash,
            actual = %actual_content_hash,
            "geometry artifact content hash mismatch; refusing materialization"
        );
        return Ok(None);
    }

    let Ok(source) = std::str::from_utf8(&bytes) else {
        warn!(%parcel_pda, "geometry artifact is not UTF-8 GeoJSON");
        return Ok(None);
    };
    let Some(ring) = submitted_ring_hash(source) else {
        warn!(%parcel_pda, "geometry artifact has no exterior ring");
        return Ok(None);
    };
    let actual_ring_hash = hex::encode(ring);
    if actual_ring_hash != expected_geometry_hash || uploaded_ring_hash != actual_ring_hash {
        warn!(
            %parcel_pda,
            onchain = %expected_geometry_hash,
            uploaded = %uploaded_ring_hash,
            actual = %actual_ring_hash,
            "geometry artifact ring hash mismatch; refusing materialization"
        );
        return Ok(None);
    }

    debug!(%parcel_pda, %content_hash, "verified geometry artifact for chain-only parcel");
    Ok(Some(VerifiedGeometryArtifact {
        source_geometry: source.to_string(),
    }))
}

/// Typed mirror for Q-035(a)'s `ParcelGeometryArtifact` PDA. The generic
/// `chain_accounts` row is written for every IDL type; this row additionally
/// exposes the fields the upload/indexer materialization path needs.
async fn upsert_parcel_geometry_artifact(
    pool: &PgPool,
    pubkey: &Pubkey,
    artifact: &ParsedParcelGeometryArtifact,
    provenance: &AccountProvenance,
) -> Result<()> {
    let pubkey_str = pubkey.to_string();
    let parcel_pubkey = Pubkey::new_from_array(artifact.parcel).to_string();
    let registrant = Pubkey::new_from_array(artifact.registrant).to_string();
    let content_hash = hex::encode(artifact.content_hash);
    let verified = provenance.is_verified();
    let chain_slot = if verified {
        provenance.slot.map(|slot| slot as i64)
    } else {
        None
    };
    let chain_signature = if verified {
        provenance.signature.as_deref()
    } else {
        None
    };

    sqlx::query(
        r#"
        INSERT INTO parcel_geometry_artifacts (
            pubkey, parcel_pubkey, content_hash, storage_uri, registrant,
            chain_created_at, chain_verified, chain_slot, chain_signature
        ) VALUES ($1, $2, $3, $4, $5, to_timestamp($6::double precision), $7, $8, $9)
        ON CONFLICT (pubkey) DO UPDATE SET
            parcel_pubkey = EXCLUDED.parcel_pubkey,
            content_hash = EXCLUDED.content_hash,
            storage_uri = EXCLUDED.storage_uri,
            registrant = EXCLUDED.registrant,
            chain_created_at = EXCLUDED.chain_created_at,
            chain_verified = EXCLUDED.chain_verified,
            chain_slot = EXCLUDED.chain_slot,
            chain_signature = EXCLUDED.chain_signature,
            mirrored_at = now()
        "#,
    )
    .bind(&pubkey_str)
    .bind(&parcel_pubkey)
    .bind(&content_hash)
    .bind(&artifact.storage_uri)
    .bind(&registrant)
    .bind(artifact.created_at as f64)
    .bind(verified)
    .bind(chain_slot)
    .bind(chain_signature)
    .execute(pool)
    .await
    .with_context(|| format!("geometry artifact mirror failed for {pubkey}"))?;
    Ok(())
}

/// Insert or update a parcel row from on-chain data.
///
/// The on-chain Parcel account carries a 32-byte id and a geometry hash but
/// no polygon; `parcels.geometry` is `NOT NULL` and write-guarded. An existing
/// client-supplied polygon is confirmed only when recomputing the client's
/// ring digest from stored source GeoJSON yields the on-chain `geometry_hash`
/// (R-044). A parcel with no prior row is created ONLY from a Q-035(a) artifact
/// that passed `verified_geometry_artifact`; no bytes are ever fabricated.
/// Either way ADR-026 still requires a real transaction for `chain_verified`.
async fn upsert_parcel(
    pool: &PgPool,
    pubkey: &Pubkey,
    parcel: &ParsedParcel,
    holder: Option<&[u8; 32]>,
    provenance: &AccountProvenance,
    artifact: Option<&VerifiedGeometryArtifact>,
) -> Result<()> {
    let pubkey_str = pubkey.to_string();
    let pubkey_hex = hex::encode(pubkey.to_bytes());

    // Client writes and reconcile() use both base58 and hex onchain_id forms.
    let parcel_id: Option<Uuid> = sqlx::query_scalar(
        "SELECT id FROM parcels WHERE onchain_id = $1 OR onchain_id = $2 LIMIT 1",
    )
    .bind(&pubkey_str)
    .bind(&pubkey_hex)
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        warn!("check existing parcel {pubkey} failed: {e:?}");
        anyhow::anyhow!("check existing parcel {pubkey}: {e}")
    })?;

    let geometry_hash_hex = hex::encode(parcel.geometry_hash);
    let access_hash_hex = hex::encode(parcel.access_hash);

    let Some(parcel_id) = parcel_id else {
        // Chain-only materialization: the artifact has already been checked
        // against its on-chain PDA, content hash and this parcel's ring hash.
        let Some(artifact) = artifact else {
            warn!(
                %pubkey,
                "parcel mirror skipped: no stored polygon and no verified geometry artifact"
            );
            return Ok(());
        };
        let verified = provenance.is_verified();
        let chain_slot = if verified {
            provenance.slot.map(|slot| slot as i64)
        } else {
            None
        };
        let chain_signature = if verified {
            provenance.signature.as_deref()
        } else {
            None
        };

        let new_id: Uuid = sqlx::query_scalar(
            r#"
            INSERT INTO parcels (
                name, status, geometry, onchain_id, geometry_hash, access_hash,
                infrastructure_flags, rights_count, source_geometry,
                chain_verified, chain_slot, chain_signature, updated_at
            ) VALUES (
                $1, $2, ST_GeomFromGeoJSON($3), $4, $5, $6, $7, $8, $9,
                $10, $11, $12, to_timestamp($13::double precision)
            )
            RETURNING id
            "#,
        )
        .bind(&parcel.name)
        .bind(format!("{}", parcel.status))
        .bind(&artifact.source_geometry)
        .bind(&pubkey_str)
        .bind(&geometry_hash_hex)
        .bind(&access_hash_hex)
        .bind(parcel.infrastructure_flags as i16)
        .bind(parcel.rights_count as i16)
        .bind(&artifact.source_geometry)
        .bind(verified)
        .bind(chain_slot)
        .bind(chain_signature)
        .bind(parcel.updated_at as f64)
        .fetch_one(pool)
        .await
        .map_err(|e| {
            warn!("chain-only parcel insert {pubkey} failed: {e:?}");
            anyhow::anyhow!("chain-only parcel insert {pubkey}: {e}")
        })?;

        if let Some(holder) = holder {
            let holder_b58 = Pubkey::new_from_array(*holder).to_string();
            sqlx::query(
                "INSERT INTO parcel_ownership (parcel_id, holder) VALUES ($1, $2) \
                 ON CONFLICT (parcel_id) DO UPDATE SET holder = EXCLUDED.holder, updated_at = now()",
            )
            .bind(new_id)
            .bind(&holder_b58)
            .execute(pool)
            .await
            .map_err(|e| {
                warn!("chain-only parcel ownership {pubkey} failed: {e:?}");
                anyhow::anyhow!("chain-only parcel ownership {pubkey}: {e}")
            })?;
        }
        info!(%pubkey, "materialized chain-only parcel from verified geometry artifact");
        return Ok(());
    };

    let source_geometry: Option<String> =
        sqlx::query_scalar("SELECT source_geometry FROM parcels WHERE id = $1")
            .bind(parcel_id)
            .fetch_one(pool)
            .await
            .map_err(|e| {
                warn!("read parcel source geometry {pubkey} failed: {e:?}");
                anyhow::anyhow!("read parcel source geometry {pubkey}: {e}")
            })?;
    let stored_digest = source_geometry
        .as_deref()
        .and_then(submitted_ring_hash)
        .map(hex::encode);
    // An artifact that passed `verified_geometry_artifact` already proved the
    // same ring hash against bytes fetched from its on-chain URI.
    let geometry_matches =
        artifact.is_some() || stored_digest.as_deref() == Some(geometry_hash_hex.as_str());
    if !geometry_matches {
        warn!(
            %pubkey,
            onchain_hash = %geometry_hash_hex,
            stored_digest = ?stored_digest,
            "parcel source geometry cannot confirm on-chain geometry_hash; refusing chain_verified"
        );
    }

    // ADR-026: verification needs BOTH a real transaction and a matching
    // polygon. Never publish a signature/slot on an unverified row.
    let verified = geometry_matches && provenance.is_verified();
    let chain_slot = if verified {
        provenance.slot.map(|s| s as i64)
    } else {
        None
    };
    let chain_signature = if verified {
        provenance.signature.as_deref()
    } else {
        None
    };
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
            chain_signature = $10,
            source_geometry = COALESCE($12, source_geometry),
            geometry = CASE
                WHEN $12::text IS NOT NULL THEN ST_GeomFromGeoJSON($12)
                ELSE geometry
            END
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
    .bind(verified)
    .bind(chain_slot)
    .bind(chain_signature)
    .bind(parcel_id)
    .bind(artifact.map(|value| value.source_geometry.as_str()))
    .execute(pool)
    .await
    .map_err(|e| {
        warn!("update parcel {pubkey} failed: {e:?}");
        anyhow::anyhow!("update parcel {pubkey}: {e}")
    })?;

    if let Some(holder) = holder {
        // The Rights holder is a wallet pubkey: base58, like the rest of the
        // API (createParcel writes base58). Never write the parcel PDA here.
        let holder_b58 = Pubkey::new_from_array(*holder).to_string();
        sqlx::query(
            "INSERT INTO parcel_ownership (parcel_id, holder) VALUES ($1, $2) \
             ON CONFLICT (parcel_id) DO UPDATE SET holder = EXCLUDED.holder, updated_at = now()",
        )
        .bind(parcel_id)
        .bind(&holder_b58)
        .execute(pool)
        .await
        .map_err(|e| {
            warn!("update parcel_ownership {pubkey} failed: {e:?}");
            anyhow::anyhow!("update parcel_ownership {pubkey}: {e}")
        })?;
    } else {
        warn!(
            %pubkey,
            "ownership holder unresolved: no ownership Rights account; leaving parcel_ownership untouched"
        );
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
    // Generated decoder returns the fixed `[Pubkey; 8]` array; skip the
    // zero-filled unused slots instead of writing them as fake attestors.
    let attestors_hex: Vec<String> = version
        .attestors
        .iter()
        .filter(|pk| pk.iter().any(|b| *b != 0))
        .map(hex::encode)
        .collect();

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

/// Resolve a parcel PDA to the mirror row's UUID. `reconcile()` writes hex
/// onchain_id while the client write path writes base58; both are accepted.
async fn parcel_id_for_pda(pool: &PgPool, pda: &Pubkey) -> Result<Option<Uuid>> {
    let base58 = pda.to_string();
    let hex = hex::encode(pda.to_bytes());
    sqlx::query_scalar("SELECT id FROM parcels WHERE onchain_id = $1 OR onchain_id = $2 LIMIT 1")
        .bind(&base58)
        .bind(&hex)
        .fetch_optional(pool)
        .await
        .with_context(|| format!("parcel lookup for {pda} failed"))
}

/// `terra_registry::right_status` constants → the text values the API/UI use.
fn rights_status_text(status: u8) -> &'static str {
    match status {
        1 => "expiring",
        2 => "expired",
        3 => "grace",
        4 => "renewed",
        5 => "revoked",
        _ => "active",
    }
}

fn timestamp_from_unix(secs: i64) -> Option<DateTime<Utc>> {
    DateTime::<Utc>::from_timestamp(secs, 0)
}

/// Mirror an on-chain `Rights` account into the off-chain `rights` table.
/// `onchain_id` (forward migration 0058) is the Rights PDA, so successive
/// polls upsert one row per PDA rather than duplicating it.
async fn upsert_rights(
    pool: &PgPool,
    pubkey: &Pubkey,
    rights: &ParsedRights,
    parcel_id: Uuid,
    provenance: &AccountProvenance,
) -> Result<()> {
    let onchain_id = pubkey.to_string();
    let holder = Pubkey::new_from_array(rights.holder).to_string();
    let granter = Pubkey::new_from_array(rights.granter).to_string();
    let created_at = timestamp_from_unix(rights.created_at).unwrap_or_else(Utc::now);
    let expires_at = (rights.expires_at != 0)
        .then(|| timestamp_from_unix(rights.expires_at))
        .flatten();
    let status = rights_status_text(rights.status);

    let existing: Option<Uuid> = sqlx::query_scalar("SELECT id FROM rights WHERE onchain_id = $1")
        .bind(&onchain_id)
        .fetch_optional(pool)
        .await
        .with_context(|| format!("rights lookup for {pubkey} failed"))?;

    if let Some(id) = existing {
        sqlx::query(
            r#"
            UPDATE rights SET
                parcel_id = $1,
                rights_kind = $2,
                holder = $3,
                granter = $4,
                created_at = $5,
                expires_at = $6,
                notes = $7,
                status = $8,
                grace_period_secs = $9,
                chain_verified = $10,
                chain_slot = $11,
                chain_signature = $12
            WHERE id = $13
            "#,
        )
        .bind(parcel_id)
        .bind(rights.rights_kind as i16)
        .bind(&holder)
        .bind(&granter)
        .bind(created_at)
        .bind(expires_at)
        .bind(&rights.notes)
        .bind(status)
        .bind(rights.grace_period_secs)
        .bind(provenance.is_verified())
        .bind(provenance.slot.map(|s| s as i64))
        .bind(provenance.signature.as_deref())
        .bind(id)
        .execute(pool)
        .await
        .with_context(|| format!("rights update for {pubkey} failed"))?;
    } else {
        sqlx::query(
            r#"
            INSERT INTO rights (
                onchain_id, parcel_id, rights_kind, holder, granter,
                created_at, expires_at, notes, status, grace_period_secs,
                chain_verified, chain_slot, chain_signature
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            "#,
        )
        .bind(&onchain_id)
        .bind(parcel_id)
        .bind(rights.rights_kind as i16)
        .bind(&holder)
        .bind(&granter)
        .bind(created_at)
        .bind(expires_at)
        .bind(&rights.notes)
        .bind(status)
        .bind(rights.grace_period_secs)
        .bind(provenance.is_verified())
        .bind(provenance.slot.map(|s| s as i64))
        .bind(provenance.signature.as_deref())
        .execute(pool)
        .await
        .with_context(|| format!("rights insert for {pubkey} failed"))?;
    }
    Ok(())
}

/// Mirror an on-chain `ThreeDModel` account. `asset_pubkey` must be base58:
/// it is an FK to `spatial_assets_onchain.pubkey`.
async fn upsert_three_d_model(
    pool: &PgPool,
    pubkey: &Pubkey,
    model: &ParsedThreeDModel,
    provenance: &AccountProvenance,
) -> Result<()> {
    let pubkey_str = pubkey.to_string();
    let asset_pubkey = Pubkey::new_from_array(model.asset).to_string();
    let parcel_pubkey = hex::encode(model.parcel);
    let model_hash = hex::encode(model.model_hash);
    let submitted_by = hex::encode(model.submitted_by);
    let verified_by = if model.verified_by.iter().all(|b| *b == 0) {
        None
    } else {
        Some(hex::encode(model.verified_by))
    };
    let chain_verified_at = (model.verified_at != 0)
        .then(|| timestamp_from_unix(model.verified_at))
        .flatten();

    let exists: Option<i64> =
        sqlx::query_scalar("SELECT id FROM three_d_models_onchain WHERE pubkey = $1")
            .bind(&pubkey_str)
            .fetch_optional(pool)
            .await
            .with_context(|| format!("3D model lookup for {pubkey} failed"))?;

    if exists.is_some() {
        sqlx::query(
            r#"
            UPDATE three_d_models_onchain SET
                asset_pubkey = $1,
                parcel_pubkey = $2,
                model_version = $3,
                geometry_version = $4,
                model_hash = $5,
                storage_reference = $6,
                model_format = $7,
                submitted_by = $8,
                chain_submitted_at = to_timestamp($9::double precision),
                verified = $10,
                verified_by = $11,
                chain_verified_at = $12,
                chain_verified = $13,
                chain_slot = $14,
                chain_signature = $15,
                mirrored_at = now()
            WHERE pubkey = $16
            "#,
        )
        .bind(&asset_pubkey)
        .bind(&parcel_pubkey)
        .bind(model.model_version as i32)
        .bind(model.geometry_version as i32)
        .bind(&model_hash)
        .bind(&model.storage_reference)
        .bind(model.model_format as i16)
        .bind(&submitted_by)
        .bind(model.submitted_at as f64)
        .bind(model.verified)
        .bind(verified_by.as_deref())
        .bind(chain_verified_at)
        .bind(provenance.is_verified())
        .bind(provenance.slot.map(|s| s as i64))
        .bind(provenance.signature.as_deref())
        .bind(&pubkey_str)
        .execute(pool)
        .await
        .with_context(|| format!("3D model update for {pubkey} failed"))?;
    } else {
        sqlx::query(
            r#"
            INSERT INTO three_d_models_onchain (
                pubkey, asset_pubkey, parcel_pubkey, model_version,
                geometry_version, model_hash, storage_reference, model_format,
                submitted_by, chain_submitted_at, verified, verified_by,
                chain_verified_at, chain_verified, chain_slot, chain_signature,
                mirrored_at
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9,
                to_timestamp($10::double precision), $11, $12, $13,
                $14, $15, $16, now()
            )
            "#,
        )
        .bind(&pubkey_str)
        .bind(&asset_pubkey)
        .bind(&parcel_pubkey)
        .bind(model.model_version as i32)
        .bind(model.geometry_version as i32)
        .bind(&model_hash)
        .bind(&model.storage_reference)
        .bind(model.model_format as i16)
        .bind(&submitted_by)
        .bind(model.submitted_at as f64)
        .bind(model.verified)
        .bind(verified_by.as_deref())
        .bind(chain_verified_at)
        .bind(provenance.is_verified())
        .bind(provenance.slot.map(|s| s as i64))
        .bind(provenance.signature.as_deref())
        .execute(pool)
        .await
        .with_context(|| format!("3D model insert for {pubkey} failed"))?;
    }
    Ok(())
}

/// Mirror an on-chain `SpatialSnapshot` account.
async fn upsert_spatial_snapshot(
    pool: &PgPool,
    pubkey: &Pubkey,
    snapshot: &ParsedSpatialSnapshot,
    provenance: &AccountProvenance,
) -> Result<()> {
    let pubkey_str = pubkey.to_string();
    let asset_pubkey = Pubkey::new_from_array(snapshot.asset).to_string();
    let parcel_pubkey = hex::encode(snapshot.parcel);
    let latest_geometry = hex::encode(snapshot.latest_geometry);
    let created_by = hex::encode(snapshot.created_by);

    let exists: Option<i64> =
        sqlx::query_scalar("SELECT id FROM spatial_snapshots_onchain WHERE pubkey = $1")
            .bind(&pubkey_str)
            .fetch_optional(pool)
            .await
            .with_context(|| format!("snapshot lookup for {pubkey} failed"))?;

    if exists.is_some() {
        sqlx::query(
            r#"
            UPDATE spatial_snapshots_onchain SET
                asset_pubkey = $1,
                parcel_pubkey = $2,
                snapshot_index = $3,
                geometry_version_count = $4,
                latest_geometry_hash = $5,
                dimensionality = $6,
                elevation_min_mm = $7,
                elevation_max_mm = $8,
                label = $9,
                created_by = $10,
                chain_created_at = to_timestamp($11::double precision),
                chain_verified = $12,
                chain_slot = $13,
                chain_signature = $14,
                mirrored_at = now()
            WHERE pubkey = $15
            "#,
        )
        .bind(&asset_pubkey)
        .bind(&parcel_pubkey)
        .bind(snapshot.snapshot_index as i32)
        .bind(snapshot.geometry_version_count as i32)
        .bind(&latest_geometry)
        .bind(snapshot.dimensionality as i16)
        .bind(snapshot.elevation_min_mm as i32)
        .bind(snapshot.elevation_max_mm as i32)
        .bind(&snapshot.label)
        .bind(&created_by)
        .bind(snapshot.created_at as f64)
        .bind(provenance.is_verified())
        .bind(provenance.slot.map(|s| s as i64))
        .bind(provenance.signature.as_deref())
        .bind(&pubkey_str)
        .execute(pool)
        .await
        .with_context(|| format!("snapshot update for {pubkey} failed"))?;
    } else {
        sqlx::query(
            r#"
            INSERT INTO spatial_snapshots_onchain (
                pubkey, asset_pubkey, parcel_pubkey, snapshot_index,
                geometry_version_count, latest_geometry_hash, dimensionality,
                elevation_min_mm, elevation_max_mm, label, created_by,
                chain_created_at, chain_verified, chain_slot, chain_signature,
                mirrored_at
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11,
                to_timestamp($12::double precision), $13, $14, $15, now()
            )
            "#,
        )
        .bind(&pubkey_str)
        .bind(&asset_pubkey)
        .bind(&parcel_pubkey)
        .bind(snapshot.snapshot_index as i32)
        .bind(snapshot.geometry_version_count as i32)
        .bind(&latest_geometry)
        .bind(snapshot.dimensionality as i16)
        .bind(snapshot.elevation_min_mm as i32)
        .bind(snapshot.elevation_max_mm as i32)
        .bind(&snapshot.label)
        .bind(&created_by)
        .bind(snapshot.created_at as f64)
        .bind(provenance.is_verified())
        .bind(provenance.slot.map(|s| s as i64))
        .bind(provenance.signature.as_deref())
        .execute(pool)
        .await
        .with_context(|| format!("snapshot insert for {pubkey} failed"))?;
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

    // Independent minimal Borsh encoder: string lengths are u32 LE, exactly
    // as Anchor/Borsh writes them. If a hand-rolled u8/u16 parser returns,
    // these round-trips fail.
    fn put32(out: &mut Vec<u8>, byte: u8) {
        out.extend_from_slice(&[byte; 32]);
    }
    fn put_str(out: &mut Vec<u8>, value: &str) {
        out.extend_from_slice(&(value.len() as u32).to_le_bytes());
        out.extend_from_slice(value.as_bytes());
    }
    fn account(disc: [u8; 8], body: Vec<u8>) -> Vec<u8> {
        let mut data = disc.to_vec();
        data.extend_from_slice(&body);
        data
    }

    #[test]
    fn claim_discriminator_matches_sha256_account_claim() {
        let digest = Sha256::digest(b"account:Claim");
        assert_eq!(
            CLAIM_DISCRIMINATOR,
            [
                digest[0], digest[1], digest[2], digest[3], digest[4], digest[5], digest[6],
                digest[7],
            ],
            "generated decoder must match the real Anchor Claim discriminator"
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
    fn empty_signature_never_becomes_verified_provenance() {
        let provenance = AccountProvenance::verified(9999, String::new());
        assert!(!provenance.is_verified());
        assert_eq!(provenance.slot, None);
        assert_eq!(provenance.signature, None);

        let whitespace = AccountProvenance::verified(9999, "   ".to_string());
        assert!(!whitespace.is_verified());
        assert_eq!(whitespace.slot, None);
        assert_eq!(whitespace.signature, None);
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

    #[test]
    fn header_filter_accepts_only_mirrored_discriminators() {
        for disc in INDEXER_DISCRIMINATORS {
            assert!(is_mirrored_header(disc));
        }
        assert!(!is_mirrored_header(&[0u8; 8]));
        assert!(!is_mirrored_header(&[1u8, 2, 3]));
    }

    #[test]
    fn idl_decoders_round_trip_all_mirrored_accounts() {
        // Parcel: string length > 255 would break the old one-byte parser.
        let mut body = Vec::new();
        put32(&mut body, 1);
        put_str(&mut body, "Parcel α");
        put32(&mut body, 2);
        body.push(3); // status
        body.push(2); // rights_count
        body.extend_from_slice(&0x0102u16.to_le_bytes());
        put32(&mut body, 3); // access_hash
        body.extend_from_slice(&1_700_000_000i64.to_le_bytes());
        body.extend_from_slice(&1_700_000_100i64.to_le_bytes());
        assert_eq!(
            parse_parcel(&account(PARCEL_DISCRIMINATOR, body)),
            Some(ParsedParcel {
                id: [1; 32],
                name: "Parcel α".to_string(),
                geometry_hash: [2; 32],
                status: 3,
                rights_count: 2,
                infrastructure_flags: 0x0102,
                access_hash: [3; 32],
                created_at: 1_700_000_000,
                updated_at: 1_700_000_100,
            })
        );

        // Rights: the 300-byte note exposes the old u8 length prefix.
        let long_note = "n".repeat(300);
        let mut body = Vec::new();
        put32(&mut body, 1);
        body.push(1); // rights_kind
        put32(&mut body, 2); // holder
        put32(&mut body, 3); // granter
        body.extend_from_slice(&1_700_000_000i64.to_le_bytes());
        body.extend_from_slice(&1_700_000_100i64.to_le_bytes());
        put_str(&mut body, &long_note);
        body.push(4); // status
        body.extend_from_slice(&3_600i64.to_le_bytes());
        assert_eq!(
            parse_rights(&account(RIGHTS_DISCRIMINATOR, body)),
            Some(ParsedRights {
                parcel: [1; 32],
                rights_kind: 1,
                holder: [2; 32],
                granter: [3; 32],
                created_at: 1_700_000_000,
                expires_at: 1_700_000_100,
                notes: long_note,
                status: 4,
                grace_period_secs: 3_600,
            })
        );

        // VerificationAttestation.
        let mut body = Vec::new();
        for byte in 1..=4u8 {
            put32(&mut body, byte);
        }
        put_str(&mut body, "https://example.org/vc");
        body.extend_from_slice(&1_700_000_000i64.to_le_bytes());
        body.extend_from_slice(&1_700_000_100i64.to_le_bytes());
        put32(&mut body, 5);
        body.push(1); // selective_disclosure
        body.extend_from_slice(&1_700_000_200i64.to_le_bytes());
        body.push(2); // protocol_version
        assert_eq!(
            parse_verification_attestation(&account(VERIFICATION_ATTESTATION_DISCRIMINATOR, body)),
            Some(ParsedVerificationAttestation {
                claim: [1; 32],
                validator: [2; 32],
                observation: [3; 32],
                credential_subject: [4; 32],
                vc_type: "https://example.org/vc".to_string(),
                issuance_date: 1_700_000_000,
                expiration_date: 1_700_000_100,
                sd_jwt_digest: [5; 32],
                selective_disclosure: true,
                created_at: 1_700_000_200,
                protocol_version: 2,
            })
        );

        // Claim.
        let mut body = Vec::new();
        put32(&mut body, 1);
        put32(&mut body, 2);
        body.push(3);
        put32(&mut body, 4);
        body.push(5);
        put32(&mut body, 6);
        for value in [7u8, 8, 9, 10, 11] {
            body.push(value);
        }
        body.extend_from_slice(&1_700_000_000i64.to_le_bytes());
        body.extend_from_slice(&1_700_000_100i64.to_le_bytes());
        assert_eq!(
            parse_claim(&account(CLAIM_DISCRIMINATOR, body)),
            Some(ParsedClaim {
                claim_id: [1; 32],
                parcel: [2; 32],
                claim_type: 3,
                submitted_by: [4; 32],
                status: 5,
                statement_hash: [6; 32],
                version: 7,
                evidence_count: 8,
                observation_count: 9,
                attestation_count: 10,
                required_attestations: 11,
                created_at: 1_700_000_000,
                updated_at: 1_700_000_100,
            })
        );

        // SpatialAsset.
        let mut body = Vec::new();
        put32(&mut body, 1);
        put32(&mut body, 2);
        body.push(3);
        body.extend_from_slice(&100i64.to_le_bytes());
        body.extend_from_slice(&900i64.to_le_bytes());
        body.extend_from_slice(&7u32.to_le_bytes());
        put32(&mut body, 4);
        body.extend_from_slice(&1_700_000_000i64.to_le_bytes());
        body.extend_from_slice(&1_700_000_100i64.to_le_bytes());
        assert_eq!(
            parse_spatial_asset(&account(SPATIAL_ASSET_DISCRIMINATOR, body)),
            Some(ParsedSpatialAsset {
                parcel: [1; 32],
                authority: [2; 32],
                dimensionality: 3,
                elevation_min_mm: 100,
                elevation_max_mm: 900,
                geometry_version_count: 7,
                latest_geometry: [4; 32],
                created_at: 1_700_000_000,
                updated_at: 1_700_000_100,
            })
        );

        // GeometryVersion: u32 string length, bool, 8 fixed pubkeys.
        let mut body = Vec::new();
        put32(&mut body, 1);
        put32(&mut body, 2);
        body.extend_from_slice(&3u32.to_le_bytes());
        put32(&mut body, 4);
        body.push(5);
        body.push(6);
        put_str(&mut body, "ipfs://geometry");
        put32(&mut body, 7);
        body.extend_from_slice(&1_700_000_000i64.to_le_bytes());
        body.push(1);
        put32(&mut body, 8);
        body.extend_from_slice(&1_700_000_100i64.to_le_bytes());
        put32(&mut body, 9);
        body.extend_from_slice(&100i64.to_le_bytes());
        body.extend_from_slice(&900i64.to_le_bytes());
        body.push(1);
        body.push(2);
        for i in 0..8 {
            if i == 0 {
                put32(&mut body, 10);
            } else {
                put32(&mut body, 0);
            }
        }
        body.push(1);
        let mut attestors = [[0u8; 32]; 8];
        attestors[0] = [10; 32];
        assert_eq!(
            parse_geometry_version(&account(GEOMETRY_VERSION_DISCRIMINATOR, body)),
            Some(ParsedGeometryVersion {
                asset: [1; 32],
                parcel: [2; 32],
                version: 3,
                geometry_hash: [4; 32],
                source: 5,
                dimension: 6,
                storage_reference: "ipfs://geometry".to_string(),
                submitted_by: [7; 32],
                submitted_at: 1_700_000_000,
                verified: true,
                verified_by: [8; 32],
                verified_at: 1_700_000_100,
                evidence_manifest: [9; 32],
                elevation_min_mm: 100,
                elevation_max_mm: 900,
                elevation_source: 1,
                required: 2,
                attestors,
                attest_count: 1,
            })
        );

        // ThreeDModel.
        let mut body = Vec::new();
        put32(&mut body, 1);
        put32(&mut body, 2);
        body.extend_from_slice(&3u32.to_le_bytes());
        body.extend_from_slice(&4u32.to_le_bytes());
        put32(&mut body, 5);
        put_str(&mut body, "ipfs://model.glb");
        body.push(6);
        put32(&mut body, 7);
        body.extend_from_slice(&1_700_000_000i64.to_le_bytes());
        body.push(1);
        put32(&mut body, 8);
        body.extend_from_slice(&1_700_000_100i64.to_le_bytes());
        assert_eq!(
            parse_three_d_model(&account(THREE_D_MODEL_DISCRIMINATOR, body)),
            Some(ParsedThreeDModel {
                asset: [1; 32],
                parcel: [2; 32],
                model_version: 3,
                geometry_version: 4,
                model_hash: [5; 32],
                storage_reference: "ipfs://model.glb".to_string(),
                model_format: 6,
                submitted_by: [7; 32],
                submitted_at: 1_700_000_000,
                verified: true,
                verified_by: [8; 32],
                verified_at: 1_700_000_100,
            })
        );

        // SpatialSnapshot.
        let mut body = Vec::new();
        put32(&mut body, 1);
        put32(&mut body, 2);
        body.extend_from_slice(&3u32.to_le_bytes());
        body.extend_from_slice(&4u32.to_le_bytes());
        put32(&mut body, 5);
        body.push(2);
        body.extend_from_slice(&100i64.to_le_bytes());
        body.extend_from_slice(&900i64.to_le_bytes());
        put_str(&mut body, "snapshot λ");
        put32(&mut body, 6);
        body.extend_from_slice(&1_700_000_000i64.to_le_bytes());
        assert_eq!(
            parse_spatial_snapshot(&account(SPATIAL_SNAPSHOT_DISCRIMINATOR, body)),
            Some(ParsedSpatialSnapshot {
                asset: [1; 32],
                parcel: [2; 32],
                snapshot_index: 3,
                geometry_version_count: 4,
                latest_geometry: [5; 32],
                dimensionality: 2,
                elevation_min_mm: 100,
                elevation_max_mm: 900,
                label: "snapshot λ".to_string(),
                created_by: [6; 32],
                created_at: 1_700_000_000,
            })
        );
    }

    /// Random pubkeys avoid collisions with rows left behind by a previous
    /// interrupted test run (Solana's `new_unique()` is deterministic per
    /// process, so restarts reuse the same keys).
    fn random_test_pubkey() -> Pubkey {
        let mut bytes = [0u8; 32];
        bytes[..16].copy_from_slice(uuid::Uuid::new_v4().as_bytes());
        bytes[16..].copy_from_slice(uuid::Uuid::new_v4().as_bytes());
        Pubkey::new_from_array(bytes)
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
    async fn generic_chain_mirror_covers_every_idl_account_type() {
        let Some(pool) = test_pool().await else {
            return;
        };
        // Q-033 acceptance: one DB row per IDL account type, all with real
        // transaction provenance and a trigger-computed mirror hash.
        for name in ALL_ACCOUNT_NAMES {
            let pubkey = random_test_pubkey();
            let provenance = AccountProvenance::verified(1700, format!("test-signature-{name}"));
            upsert_chain_account(
                &pool,
                &pubkey,
                name,
                &serde_json::json!({"account": name}),
                &provenance,
            )
            .await
            .expect("generic mirror upsert");

            let (stored_type, verified, slot, signature, hash): (
                String,
                bool,
                Option<i64>,
                Option<String>,
                Option<String>,
            ) = sqlx::query_as(
                "SELECT account_type, chain_verified, chain_slot, chain_signature, mirror_hash \
                 FROM chain_accounts WHERE pubkey = $1",
            )
            .bind(pubkey.to_string())
            .fetch_one(&pool)
            .await
            .expect("read generic mirror row");
            assert_eq!(&stored_type, name);
            assert!(verified, "{name} must carry real provenance");
            assert_eq!(slot, Some(1700));
            assert_eq!(
                signature.as_deref(),
                Some(format!("test-signature-{name}").as_str())
            );
            assert!(hash.is_some_and(|h| h.len() == 64), "{name} mirror_hash");

            sqlx::query("DELETE FROM chain_accounts WHERE pubkey = $1")
                .bind(pubkey.to_string())
                .execute(&pool)
                .await
                .expect("delete test generic mirror row");
        }
        pool.close().await;
    }

    #[tokio::test]
    async fn generic_chain_mirror_rejects_provenance_without_signature() {
        let Some(pool) = test_pool().await else {
            return;
        };
        let pubkey = random_test_pubkey();
        upsert_chain_account(
            &pool,
            &pubkey,
            "ZoneSet",
            &serde_json::json!({"zone_id": [1]}),
            &AccountProvenance::verified(9999, String::new()),
        )
        .await
        .expect("upsert unverified generic row");
        let (verified, slot, signature): (bool, Option<i64>, Option<String>) = sqlx::query_as(
            "SELECT chain_verified, chain_slot, chain_signature FROM chain_accounts WHERE pubkey = $1",
        )
        .bind(pubkey.to_string())
        .fetch_one(&pool)
        .await
        .expect("read unverified generic row");
        assert!(!verified);
        assert_eq!(slot, None);
        assert_eq!(signature, None);
        // An empty signature is not a real transaction: ADR-026 requires the
        // three provenance columns to be FALSE/NULL/NULL.
        sqlx::query("DELETE FROM chain_accounts WHERE pubkey = $1")
            .bind(pubkey.to_string())
            .execute(&pool)
            .await
            .expect("delete test row");
        pool.close().await;
    }

    // Exactly the GeoJSON the registration panel sends: the client anchors
    // sha256(JSON.stringify(coordinates[0])), not a terra-geom-v2 digest.
    const TEST_GEOJSON: &str =
        r#"{"type":"Polygon","coordinates":[[[0,0],[1,0],[1,1],[0,1],[0,0]]]}"#;

    #[tokio::test]
    async fn claims_mirror_has_provenance_columns() {
        let Some(pool) = test_pool().await else {
            return;
        };
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
        let Some(pool) = test_pool().await else {
            return;
        };
        let pubkey = random_test_pubkey();
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
            attestors: [[0u8; 32]; 8],
            attest_count: 0,
        }
    }

    #[tokio::test]
    async fn spatial_mirrors_set_chain_verified_from_transaction_provenance() {
        let Some(pool) = test_pool().await else {
            return;
        };
        let asset_key = random_test_pubkey();
        let version_key = random_test_pubkey();
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
            assert!(
                verified,
                "{table} must be chain-verified when a tx is resolved"
            );
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

    #[tokio::test]
    async fn rights_mirror_uses_parcel_pda_and_real_wallet_holder() {
        let Some(pool) = test_pool().await else {
            return;
        };
        let parcel_pda = random_test_pubkey();
        let rights_pda = random_test_pubkey();
        let parcel_id = Uuid::new_v4();
        let parcel_key = parcel_pda.to_string();
        let rights_key = rights_pda.to_string();

        sqlx::query(
            "INSERT INTO parcels (id, name, status, geometry, onchain_id) \
             VALUES ($1, 'rights-test', 'registered', ST_GeomFromGeoJSON($2), $3)",
        )
        .bind(parcel_id)
        .bind(TEST_GEOJSON)
        .bind(&parcel_key)
        .execute(&pool)
        .await
        .expect("insert test parcel for rights");

        let holder = [21u8; 32];
        let rights = ParsedRights {
            parcel: parcel_pda.to_bytes(),
            rights_kind: 1,
            holder,
            granter: [22u8; 32],
            created_at: 1_700_000_000,
            expires_at: 0,
            notes: "usage right".to_string(),
            status: 0,
            grace_period_secs: 0,
        };
        let provenance = AccountProvenance::verified(1500, "test-rights-sig".to_string());
        upsert_rights(&pool, &rights_pda, &rights, parcel_id, &provenance)
            .await
            .expect("upsert rights");

        let (db_parcel_id, db_holder, verified, slot, signature): (
            Uuid,
            String,
            bool,
            Option<i64>,
            Option<String>,
        ) = sqlx::query_as(
            "SELECT parcel_id, holder, chain_verified, chain_slot, chain_signature \
             FROM rights WHERE onchain_id = $1",
        )
        .bind(&rights_key)
        .fetch_one(&pool)
        .await
        .expect("read mirrored rights");
        assert_eq!(db_parcel_id, parcel_id);
        assert_eq!(db_holder, Pubkey::new_from_array(holder).to_string());
        assert!(verified);
        assert_eq!(slot, Some(1500));
        assert_eq!(signature.as_deref(), Some("test-rights-sig"));

        sqlx::query("DELETE FROM rights WHERE onchain_id = $1")
            .bind(&rights_key)
            .execute(&pool)
            .await
            .expect("delete test rights");
        sqlx::query("DELETE FROM parcels WHERE id = $1")
            .bind(parcel_id)
            .execute(&pool)
            .await
            .expect("delete test parcel");
        pool.close().await;
    }

    #[tokio::test]
    async fn three_d_and_snapshot_mirrors_set_provenance() {
        let Some(pool) = test_pool().await else {
            return;
        };
        let asset_key = random_test_pubkey();
        let asset_str = asset_key.to_string();
        let model_key = random_test_pubkey();
        let snapshot_key = random_test_pubkey();
        let model_str = model_key.to_string();
        let snapshot_str = snapshot_key.to_string();

        sqlx::query(
            "INSERT INTO spatial_assets_onchain (pubkey, parcel_pubkey, authority) \
             VALUES ($1, $2, $3)",
        )
        .bind(&asset_str)
        .bind(hex::encode([5u8; 32]))
        .bind(hex::encode([6u8; 32]))
        .execute(&pool)
        .await
        .expect("insert parent spatial asset");

        let provenance = AccountProvenance::verified(1600, "test-model-sig".to_string());
        let model = ParsedThreeDModel {
            asset: asset_key.to_bytes(),
            parcel: [5u8; 32],
            model_version: 1,
            geometry_version: 2,
            model_hash: [7u8; 32],
            storage_reference: "test://model".to_string(),
            model_format: 1,
            submitted_by: [8u8; 32],
            submitted_at: 1_700_000_000,
            verified: true,
            verified_by: [9u8; 32],
            verified_at: 1_700_000_100,
        };
        upsert_three_d_model(&pool, &model_key, &model, &provenance)
            .await
            .expect("upsert 3D model");
        let snapshot = ParsedSpatialSnapshot {
            asset: asset_key.to_bytes(),
            parcel: [5u8; 32],
            snapshot_index: 1,
            geometry_version_count: 2,
            latest_geometry: [10u8; 32],
            dimensionality: 3,
            elevation_min_mm: 0,
            elevation_max_mm: 100,
            label: "test snapshot".to_string(),
            created_by: [11u8; 32],
            created_at: 1_700_000_000,
        };
        upsert_spatial_snapshot(&pool, &snapshot_key, &snapshot, &provenance)
            .await
            .expect("upsert spatial snapshot");

        for (table, key) in [
            ("three_d_models_onchain", &model_str),
            ("spatial_snapshots_onchain", &snapshot_str),
        ] {
            let query = format!(
                "SELECT chain_verified, chain_slot, chain_signature FROM {table} WHERE pubkey = $1"
            );
            let (verified, slot, signature): (bool, Option<i64>, Option<String>) =
                sqlx::query_as(&query)
                    .bind(key)
                    .fetch_one(&pool)
                    .await
                    .expect("read model/snapshot provenance");
            assert!(
                verified,
                "{table} must be chain-verified when a tx is resolved"
            );
            assert_eq!(slot, Some(1600));
            assert_eq!(signature.as_deref(), Some("test-model-sig"));
        }

        sqlx::query("DELETE FROM spatial_snapshots_onchain WHERE pubkey = $1")
            .bind(&snapshot_str)
            .execute(&pool)
            .await
            .expect("delete test snapshot");
        sqlx::query("DELETE FROM three_d_models_onchain WHERE pubkey = $1")
            .bind(&model_str)
            .execute(&pool)
            .await
            .expect("delete test 3D model");
        sqlx::query("DELETE FROM spatial_assets_onchain WHERE pubkey = $1")
            .bind(&asset_str)
            .execute(&pool)
            .await
            .expect("delete test spatial asset");
        pool.close().await;
    }

    #[tokio::test]
    async fn parcel_verification_requires_matching_geometry_digest() {
        let Some(pool) = test_pool().await else {
            return;
        };
        let parcel_pda = random_test_pubkey();
        let parcel_id = Uuid::new_v4();
        let parcel_key = parcel_pda.to_string();
        let geometry_hash = submitted_ring_hash(TEST_GEOJSON).expect("test ring digest");
        let holder = [31u8; 32];

        sqlx::query(
            "INSERT INTO parcels (id, name, status, geometry, source_geometry, onchain_id) \
             VALUES ($1, 'digest-test', 'registered', ST_GeomFromGeoJSON($2), $2, $3)",
        )
        .bind(parcel_id)
        .bind(TEST_GEOJSON)
        .bind(&parcel_key)
        .execute(&pool)
        .await
        .expect("insert parcel for digest test");

        let mut parcel = ParsedParcel {
            id: [0u8; 32],
            name: "digest-test".to_string(),
            geometry_hash: [0xFFu8; 32],
            status: 1,
            rights_count: 0,
            infrastructure_flags: 0,
            access_hash: [0u8; 32],
            created_at: 1_700_000_000,
            updated_at: 1_700_000_100,
        };
        let provenance = AccountProvenance::verified(1700, "test-parcel-sig".to_string());

        // A real signature must NOT verify a polygon with a different hash.
        upsert_parcel(
            &pool,
            &parcel_pda,
            &parcel,
            Some(&holder),
            &provenance,
            None,
        )
        .await
        .expect("upsert parcel with mismatched hash");
        let (verified, slot, signature): (bool, Option<i64>, Option<String>) = sqlx::query_as(
            "SELECT chain_verified, chain_slot, chain_signature FROM parcels WHERE id = $1",
        )
        .bind(parcel_id)
        .fetch_one(&pool)
        .await
        .expect("read parcel provenance");
        assert!(
            !verified,
            "mismatched geometry must never be chain_verified"
        );
        assert_eq!(slot, None);
        assert_eq!(signature, None);

        // With the real digest, the same signature verifies the row and the
        // ownership holder is the wallet from Rights, base58 — not the PDA.
        parcel.geometry_hash = geometry_hash;
        upsert_parcel(
            &pool,
            &parcel_pda,
            &parcel,
            Some(&holder),
            &provenance,
            None,
        )
        .await
        .expect("upsert parcel with matching hash");
        let (verified, slot, signature, db_holder): (bool, Option<i64>, Option<String>, String) =
            sqlx::query_as(
                "SELECT p.chain_verified, p.chain_slot, p.chain_signature, o.holder \
             FROM parcels p JOIN parcel_ownership o ON o.parcel_id = p.id WHERE p.id = $1",
            )
            .bind(parcel_id)
            .fetch_one(&pool)
            .await
            .expect("read verified parcel and holder");
        assert!(verified);
        assert_eq!(slot, Some(1700));
        assert_eq!(signature.as_deref(), Some("test-parcel-sig"));
        assert_eq!(db_holder, Pubkey::new_from_array(holder).to_string());

        sqlx::query("DELETE FROM parcels WHERE id = $1")
            .bind(parcel_id)
            .execute(&pool)
            .await
            .expect("delete test parcel");
        pool.close().await;
    }

    fn test_geometry_artifact(parcel: &Pubkey) -> ParsedParcelGeometryArtifact {
        ParsedParcelGeometryArtifact {
            parcel: parcel.to_bytes(),
            content_hash: [42u8; 32],
            storage_uri: "content/sha256/2a/2a/2a2a2a2a".to_string(),
            registrant: [23u8; 32],
            created_at: 1_700_000_000,
        }
    }

    #[tokio::test]
    async fn parcel_geometry_artifact_mirror_writes_transaction_provenance() {
        let Some(pool) = test_pool().await else {
            return;
        };
        let artifact_pda = random_test_pubkey();
        let parcel = random_test_pubkey();
        let artifact = test_geometry_artifact(&parcel);
        let artifact_key = artifact_pda.to_string();

        upsert_parcel_geometry_artifact(
            &pool,
            &artifact_pda,
            &artifact,
            &AccountProvenance::verified(1888, "test-artifact-sig".to_string()),
        )
        .await
        .expect("upsert artifact with provenance");

        let (parcel_pubkey, content_hash, storage_uri, verified, slot, signature, mirror_hash): (
            String,
            String,
            String,
            bool,
            Option<i64>,
            Option<String>,
            Option<String>,
        ) = sqlx::query_as(
            "SELECT parcel_pubkey, content_hash, storage_uri, chain_verified, \
             chain_slot, chain_signature, mirror_hash \
             FROM parcel_geometry_artifacts WHERE pubkey = $1",
        )
        .bind(&artifact_key)
        .fetch_one(&pool)
        .await
        .expect("read artifact provenance");
        assert_eq!(parcel_pubkey, parcel.to_string());
        assert_eq!(content_hash, hex::encode([42u8; 32]));
        assert_eq!(storage_uri, artifact.storage_uri);
        assert!(verified);
        assert_eq!(slot, Some(1888));
        assert_eq!(signature.as_deref(), Some("test-artifact-sig"));
        assert!(mirror_hash.is_some_and(|hash| hash.len() == 64));

        // A failed signature lookup must downgrade and clear all three fields.
        upsert_parcel_geometry_artifact(
            &pool,
            &artifact_pda,
            &artifact,
            &AccountProvenance::unverified(),
        )
        .await
        .expect("downgrade artifact provenance");
        let (verified, slot, signature): (bool, Option<i64>, Option<String>) = sqlx::query_as(
            "SELECT chain_verified, chain_slot, chain_signature \
             FROM parcel_geometry_artifacts WHERE pubkey = $1",
        )
        .bind(&artifact_key)
        .fetch_one(&pool)
        .await
        .expect("read downgraded artifact provenance");
        assert!(!verified);
        assert_eq!(slot, None);
        assert_eq!(signature, None);

        sqlx::query("DELETE FROM parcel_geometry_artifacts WHERE pubkey = $1")
            .bind(&artifact_key)
            .execute(&pool)
            .await
            .expect("delete test artifact mirror");
        pool.close().await;
    }

    #[tokio::test]
    async fn chain_only_parcel_materializes_from_verified_artifact() {
        let Some(pool) = test_pool().await else {
            return;
        };
        let dir = tempfile::tempdir().expect("temporary artifact root");
        let storage = crate::storage::StorageBackend::Local {
            root: dir.path().to_path_buf(),
        };
        let stored = storage
            .store_content_addressed("application/geo+json", TEST_GEOJSON.as_bytes())
            .await
            .expect("store test geometry artifact");

        let parcel_pda = random_test_pubkey();
        let artifact_pda = random_test_pubkey();
        let parcel_key = parcel_pda.to_string();
        let artifact_key = artifact_pda.to_string();
        let content_hash = hex::encode(stored.content_hash);
        let ring_hash = submitted_ring_hash(TEST_GEOJSON).expect("test ring digest");
        let geometry_hash = hex::encode(ring_hash);
        let holder = [31u8; 32];

        // No verified artifact row: a chain-only parcel must not be created.
        assert!(
            verified_geometry_artifact(&pool, &storage, &parcel_pda, &geometry_hash)
                .await
                .expect("artifact lookup without a row")
                .is_none()
        );

        sqlx::query(
            "INSERT INTO parcel_geometry_uploads (\
                parcel_pubkey, content_hash, geometry_hash, storage_ref, \
                storage_backend, content_type, size_bytes \
             ) VALUES ($1, $2, $3, $4, 'local', 'application/geo+json', $5)",
        )
        .bind(&parcel_key)
        .bind(&content_hash)
        .bind(&geometry_hash)
        .bind(&stored.storage_ref)
        .bind(stored.size_bytes as i64)
        .execute(&pool)
        .await
        .expect("insert test geometry upload");

        sqlx::query(
            "INSERT INTO parcel_geometry_artifacts (\
                pubkey, parcel_pubkey, content_hash, storage_uri, registrant, \
                chain_created_at, chain_verified, chain_slot, chain_signature \
             ) VALUES ($1, $2, $3, $4, $5, to_timestamp($6::double precision), TRUE, $7, $8)",
        )
        .bind(&artifact_key)
        .bind(&parcel_key)
        .bind(&content_hash)
        .bind(&stored.storage_ref)
        .bind(Pubkey::new_from_array([23u8; 32]).to_string())
        .bind(1_700_000_000f64)
        .bind(1900i64)
        .bind("test-chain-only-sig")
        .execute(&pool)
        .await
        .expect("insert verified test artifact mirror");

        let artifact = verified_geometry_artifact(&pool, &storage, &parcel_pda, &geometry_hash)
            .await
            .expect("verified artifact lookup")
            .expect("chain-verified artifact must resolve");
        assert_eq!(artifact.source_geometry, TEST_GEOJSON);

        let parcel = ParsedParcel {
            id: [0u8; 32],
            name: "chain-only".to_string(),
            geometry_hash: ring_hash,
            status: 1,
            rights_count: 0,
            infrastructure_flags: 0,
            access_hash: [0u8; 32],
            created_at: 1_700_000_000,
            updated_at: 1_700_000_100,
        };
        upsert_parcel(
            &pool,
            &parcel_pda,
            &parcel,
            Some(&holder),
            &AccountProvenance::verified(1900, "test-chain-only-sig".to_string()),
            Some(&artifact),
        )
        .await
        .expect("materialize chain-only parcel");

        let (name, source_geometry, verified, slot, signature, db_holder): (
            String,
            String,
            bool,
            Option<i64>,
            Option<String>,
            String,
        ) = sqlx::query_as(
            "SELECT p.name, p.source_geometry, p.chain_verified, p.chain_slot, \
             p.chain_signature, o.holder \
             FROM parcels p JOIN parcel_ownership o ON o.parcel_id = p.id \
             WHERE p.onchain_id = $1",
        )
        .bind(&parcel_key)
        .fetch_one(&pool)
        .await
        .expect("read materialized parcel");
        assert_eq!(name, "chain-only");
        assert_eq!(source_geometry, TEST_GEOJSON);
        assert!(verified);
        assert_eq!(slot, Some(1900));
        assert_eq!(signature.as_deref(), Some("test-chain-only-sig"));
        assert_eq!(db_holder, Pubkey::new_from_array(holder).to_string());

        // Changing only the on-chain URI invalidates the artifact even though
        // the uploaded bytes are untouched: fail closed on mirror drift.
        sqlx::query("UPDATE parcel_geometry_artifacts SET storage_uri = $2 WHERE pubkey = $1")
            .bind(&artifact_key)
            .bind("content/sha256/ff/ff/tampered")
            .execute(&pool)
            .await
            .expect("tamper artifact URI");
        assert!(
            verified_geometry_artifact(&pool, &storage, &parcel_pda, &geometry_hash)
                .await
                .expect("lookup after URI tamper")
                .is_none(),
            "an on-chain URI that does not match stored bytes must fail closed"
        );

        sqlx::query("DELETE FROM parcels WHERE onchain_id = $1")
            .bind(&parcel_key)
            .execute(&pool)
            .await
            .expect("delete materialized parcel");
        sqlx::query("DELETE FROM parcel_geometry_uploads WHERE parcel_pubkey = $1")
            .bind(&parcel_key)
            .execute(&pool)
            .await
            .expect("delete test upload");
        sqlx::query("DELETE FROM parcel_geometry_artifacts WHERE pubkey = $1")
            .bind(&artifact_key)
            .execute(&pool)
            .await
            .expect("delete test artifact mirror");
        pool.close().await;
    }
}
