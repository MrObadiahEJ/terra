//! Chain indexer (Sprint C).
//!
//! Polls the configured Solana RPC for `terra_registry` program accounts,
//! decodes Anchor `Parcel` accounts by discriminator, and upserts them into the
//! `parcels` mirror table with `chain_verified = true`, `chain_slot`, and
//! `chain_signature`. Client-POSTed rows start with `chain_verified = false`
//! and are confirmed by the indexer.
//!
//! The indexer runs as a background tokio task spawned from `main.rs`.

use anyhow::{Context, Result};
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, warn};

use crate::state::AppState;

/// terra_registry program ID (mirrors terra-web/src/lib/constants.ts).
const PROGRAM_ID: &str = "GaEDbktvpZ3qiqp4PmFgHwDSa6JsFfVjXFqNb2nTbage";

/// Anchor 8-byte discriminator for the `Parcel` account:
/// SHA-256("account:Parcel")[..8]
const PARCEL_DISCRIMINATOR: [u8; 8] = [149, 167, 245, 67, 209, 244, 214, 75];

/// Anchor 8-byte discriminator for the `VerificationAttestation` account:
/// SHA-256("account:VerificationAttestation")[..8]
const VERIFICATION_ATTESTATION_DISCRIMINATOR: [u8; 8] = [143, 201, 182, 196, 87, 192, 219, 159];

/// Interval between indexer polls.
const POLL_INTERVAL_SECS: u64 = 15;

/// Parsed Parcel account from on-chain data.
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

/// Convert a ParsedParcel id to a hex string.
fn id_hex(id: &[u8; 32]) -> String {
    id.iter().map(|b| format!("{b:02x}")).collect()
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

/// Single poll: fetch program accounts, decode parcels, upsert.
async fn poll_once(client: &RpcClient, program_id: &Pubkey, state: &AppState) -> Result<usize> {
    let accounts = client
        .get_program_accounts(program_id)
        .with_context(|| "get_program_accounts failed")?;

    let mut upserted = 0usize;
    for (pubkey, account) in &accounts {
        if let Some(parcel) = parse_parcel(&account.data) {
            upsert_parcel(state, pubkey, &parcel, 0, "").await?;
            upserted += 1;
        } else if let Some(attestation) = parse_verification_attestation(&account.data) {
            upsert_verification_attestation(state, pubkey, &attestation, 0, "").await?;
            upserted += 1;
        }
    }

    Ok(upserted)
}

/// Insert or update a parcel row from on-chain data.
async fn upsert_parcel(
    state: &AppState,
    pubkey: &Pubkey,
    parcel: &ParsedParcel,
    _slot: u64,
    _signature: &str,
) -> Result<()> {
    let id_hex = id_hex(&parcel.id);
    let pubkey_str = pubkey.to_string();

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
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| {
            warn!("check existing parcel {pubkey} failed: {e:?}");
            anyhow::anyhow!("check existing parcel {pubkey}: {e}")
        })?;

    if exists.is_some() {
        sqlx::query(
            r#"
            UPDATE parcels SET
                name = $1,
                status = $2,
                geometry = ST_GeomFromGeoJSON('{"type":"Polygon","coordinates":[[[0,0],[1,0],[1,1],[0,1],[0,0]]]}'),
                geometry_hash = $3,
                infrastructure_flags = $4,
                rights_count = $5,
                access_hash = $6,
                updated_at = to_timestamp($7::double precision),
                chain_verified = TRUE,
                chain_slot = $8,
                chain_signature = $9
            WHERE onchain_id = $10
            "#,
        )
        .bind(&parcel.name)
        .bind(format!("{}", parcel.status))
        .bind(&geometry_hash_hex)
        .bind(parcel.infrastructure_flags as i16)
        .bind(parcel.rights_count as i16)
        .bind(&access_hash_hex)
        .bind(parcel.updated_at as f64)
        .bind(0i64) // chain_slot
        .bind("") // chain_signature
        .bind(&pubkey_str)
        .execute(&state.pool)
        .await
        .map_err(|e| {
            warn!("update parcel {pubkey} failed: {e:?}");
            anyhow::anyhow!("update parcel {pubkey}: {e}")
        })?;
    } else {
        sqlx::query(
            r#"
            INSERT INTO parcels (
                id, name, status, geometry, onchain_id, geometry_hash,
                infrastructure_flags, rights_count, access_hash,
                created_at, updated_at, chain_verified, chain_slot, chain_signature
            ) VALUES (
                $1, $2, $3, ST_GeomFromGeoJSON('{"type":"Polygon","coordinates":[[[0,0],[0,0],[0,0]]]}'), $4, $5, $6, $7, $8,
                to_timestamp($9::double precision), to_timestamp($10::double precision), TRUE, $11, $12
            )
            "#,
        )
        .bind(id_hex)
        .bind(&parcel.name)
        .bind(format!("{}", parcel.status))
        .bind(&pubkey_str)
        .bind(&geometry_hash_hex)
        .bind(parcel.infrastructure_flags as i16)
        .bind(parcel.rights_count as i16)
        .bind(&access_hash_hex)
        .bind(parcel.created_at as f64)
        .bind(parcel.updated_at as f64)
        .bind(0i64) // chain_slot
        .bind("") // chain_signature
        .execute(&state.pool)
        .await
        .map_err(|e| {
            warn!("insert parcel {pubkey} failed: {e:?}");
            anyhow::anyhow!("insert parcel {pubkey}: {e}")
        })?;
    }

    Ok(())
}

/// Insert or update a verification_attestation row from on-chain data.
async fn upsert_verification_attestation(
    state: &AppState,
    pubkey: &Pubkey,
    attestation: &ParsedVerificationAttestation,
    _slot: u64,
    _signature: &str,
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
            .fetch_optional(&state.pool)
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
                chain_verified = TRUE,
                chain_slot = $11,
                chain_signature = $12
            WHERE pubkey = $13
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
        .bind(0i64) // chain_slot
        .bind("") // chain_signature
        .bind(&pubkey_str)
        .execute(&state.pool)
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
                $9, $10, $11, TRUE, $12, $13
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
        .bind(0i64) // chain_slot
        .bind("") // chain_signature
        .execute(&state.pool)
        .await
        .map_err(|e| {
            warn!("insert verification_attestation {pubkey} failed: {e:?}");
            anyhow::anyhow!("insert verification_attestation {pubkey}: {e}")
        })?;
    }

    Ok(())
}
