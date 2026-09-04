use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::AppError;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", post(create_vault).get(list_vaults))
        .route("/{vault_pubkey}", get(get_vault))
        .route("/{vault_pubkey}/access", post(authorize_vault_access))
        .route("/{vault_pubkey}/rotations", post(initiate_shard_rotation))
        .route(
            "/{vault_pubkey}/rotations/{rotation_pubkey}/endorse",
            post(endorse_shard_rotation),
        )
        .route(
            "/{vault_pubkey}/rotations/{rotation_pubkey}/execute",
            post(execute_shard_rotation),
        )
        .route(
            "/{vault_pubkey}/rotations/{rotation_pubkey}/cancel",
            post(cancel_shard_rotation),
        )
        .route("/{vault_pubkey}/ping", post(ping_shard))
        .route("/{vault_pubkey}/access-logs", get(list_access_logs))
}

// ---------------------------------------------------------------------------
// Vault shard protocol mirror endpoints (RFC-003)
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct CreateVault {
    pub subject_pubkey: String,
    pub ciphertext_cid: String,
    pub ciphertext_hash: String,
    pub algorithm_id: u16,
    pub storage_uris: Vec<String>,
    pub shard_holders: Vec<String>,
    pub threshold: u16,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct VaultRow {
    pub id: i64,
    pub subject_pubkey: String,
    pub vault_pubkey: String,
    pub ciphertext_cid: String,
    pub ciphertext_hash: String,
    pub algorithm_id: i16,
    pub storage_uris: Vec<String>,
    pub shard_holders: Vec<String>,
    pub threshold: i16,
    pub version: i32,
    pub last_ping_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct AuthorizeAccess {
    pub authority: String,
    pub purpose: String,
    pub expiry: chrono::DateTime<chrono::Utc>,
    pub nonce: String,
}

#[derive(Debug, Deserialize)]
pub struct InitiateRotation {
    pub initiator: String,
    pub old_ciphertext_hash: String,
    pub new_ciphertext_hash: String,
    pub new_shard_holders: Vec<String>,
    pub new_threshold: u16,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct RotationRow {
    pub id: i64,
    pub rotation_pubkey: String,
    pub vault_pubkey: String,
    pub old_ciphertext_hash: String,
    pub new_ciphertext_hash: String,
    pub new_shard_holders: Vec<String>,
    pub new_threshold: i16,
    pub initiated_by: String,
    pub endorsements: Vec<String>,
    pub required_endorsements: i16,
    pub initiated_at: chrono::DateTime<chrono::Utc>,
    pub effective_at: chrono::DateTime<chrono::Utc>,
    pub status: i16,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct EndorseRotation {
    pub validator: String,
}

#[derive(Debug, Deserialize)]
pub struct PingShard {
    pub validator: String,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct AccessLogRow {
    pub id: i64,
    pub vault_pubkey: String,
    pub subject_pubkey: String,
    pub authority: String,
    pub purpose: String,
    pub expiry: chrono::DateTime<chrono::Utc>,
    pub nonce: String,
    pub block_time: chrono::DateTime<chrono::Utc>,
}

fn decode_hex64(s: &str) -> Result<Vec<u8>, AppError> {
    hex::decode(s).map_err(|e| AppError::bad_request(format!("invalid hex: {e}")))
}

/// Create a vault mirror row from the on-chain VaultRecord.
pub async fn create_vault(
    State(state): State<AppState>,
    Json(req): Json<CreateVault>,
) -> Result<(StatusCode, Json<VaultRow>), AppError> {
    let ciphertext_hash = decode_hex64(&req.ciphertext_hash)?;
    if ciphertext_hash.len() != 32 {
        return Err(AppError::bad_request("ciphertext_hash must be 32 bytes"));
    }
    if req.ciphertext_cid.is_empty() {
        return Err(AppError::bad_request("ciphertext_cid is required"));
    }
    if req.threshold == 0 {
        return Err(AppError::bad_request("threshold must be > 0"));
    }
    if req.shard_holders.is_empty() {
        return Err(AppError::bad_request("shard_holders cannot be empty"));
    }
    if req.shard_holders.len() > 8 {
        return Err(AppError::bad_request("shard_holders cannot exceed 8"));
    }
    if req.threshold as usize > req.shard_holders.len() {
        return Err(AppError::bad_request(
            "threshold exceeds shard holder count",
        ));
    }

    let row = sqlx::query_as::<_, VaultRow>(
        "INSERT INTO vaults
            (subject_pubkey, vault_pubkey, ciphertext_cid, ciphertext_hash,
             algorithm_id, storage_uris, shard_holders, threshold)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
         ON CONFLICT (vault_pubkey) DO UPDATE SET
            ciphertext_cid   = EXCLUDED.ciphertext_cid,
            ciphertext_hash  = EXCLUDED.ciphertext_hash,
            algorithm_id     = EXCLUDED.algorithm_id,
            storage_uris     = EXCLUDED.storage_uris,
            shard_holders    = EXCLUDED.shard_holders,
            threshold        = EXCLUDED.threshold,
            updated_at       = now()
         RETURNING id, subject_pubkey, vault_pubkey, ciphertext_cid,
                   encode(ciphertext_hash, 'hex') AS ciphertext_hash, algorithm_id, storage_uris, shard_holders,
                   threshold, version, last_ping_at, created_at, updated_at",
    )
    .bind(&req.subject_pubkey)
    .bind(&req.subject_pubkey) // vault_pubkey mirror — set from on-chain in prod
    .bind(&req.ciphertext_cid)
    .bind(&ciphertext_hash)
    .bind(req.algorithm_id as i16)
    .bind(&req.storage_uris)
    .bind(&req.shard_holders)
    .bind(req.threshold as i16)
    .fetch_one(&state.pool)
    .await?;
    Ok((StatusCode::CREATED, Json(row)))
}

/// Fetch a single vault by its on-chain pubkey.
pub async fn get_vault(
    State(state): State<AppState>,
    Path(vault_pubkey): Path<String>,
) -> Result<Json<VaultRow>, AppError> {
    let row = sqlx::query_as::<_, VaultRow>(
        "SELECT id, subject_pubkey, vault_pubkey, ciphertext_cid, encode(ciphertext_hash, 'hex') AS ciphertext_hash,
                algorithm_id, storage_uris, shard_holders, threshold, version,
                last_ping_at, created_at, updated_at
         FROM vaults WHERE vault_pubkey = $1",
    )
    .bind(&vault_pubkey)
    .fetch_one(&state.pool)
    .await?;
    Ok(Json(row))
}

/// List all vaults (paginated in future).
pub async fn list_vaults(State(state): State<AppState>) -> Result<Json<Vec<VaultRow>>, AppError> {
    let rows = sqlx::query_as::<_, VaultRow>(
        "SELECT id, subject_pubkey, vault_pubkey, ciphertext_cid, encode(ciphertext_hash, 'hex') AS ciphertext_hash,
                algorithm_id, storage_uris, shard_holders, threshold, version,
                last_ping_at, created_at, updated_at
         FROM vaults ORDER BY created_at DESC LIMIT 100",
    )
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(rows))
}

/// Record a vault access authorization log entry.
pub async fn authorize_vault_access(
    State(state): State<AppState>,
    Path(vault_pubkey): Path<String>,
    Json(req): Json<AuthorizeAccess>,
) -> Result<(StatusCode, Json<AccessLogRow>), AppError> {
    if req.purpose.is_empty() {
        return Err(AppError::bad_request("purpose is required"));
    }
    let nonce = decode_hex64(&req.nonce)?;
    if nonce.iter().all(|&b| b == 0) {
        return Err(AppError::bad_request("nonce cannot be all zeros"));
    }

    let row = sqlx::query_as::<_, AccessLogRow>(
        "INSERT INTO vault_access_logs
            (vault_pubkey, subject_pubkey, authority, purpose, expiry, nonce)
         SELECT $1, subject_pubkey, $2, $3, $4, $5
         FROM vaults WHERE vault_pubkey = $1
         RETURNING id, vault_pubkey, subject_pubkey, authority, purpose,
                   expiry, encode(nonce, 'hex') AS nonce, block_time",
    )
    .bind(&vault_pubkey)
    .bind(&req.authority)
    .bind(&req.purpose)
    .bind(req.expiry)
    .bind(&nonce)
    .fetch_one(&state.pool)
    .await?;
    Ok((StatusCode::CREATED, Json(row)))
}

/// Record a shard rotation initiation.
pub async fn initiate_shard_rotation(
    State(state): State<AppState>,
    Path(vault_pubkey): Path<String>,
    Json(req): Json<InitiateRotation>,
) -> Result<(StatusCode, Json<RotationRow>), AppError> {
    let old_hash = decode_hex64(&req.old_ciphertext_hash)?;
    let new_hash = decode_hex64(&req.new_ciphertext_hash)?;
    if old_hash.len() != 32 || new_hash.len() != 32 {
        return Err(AppError::bad_request("ciphertext hashes must be 32 bytes"));
    }
    if req.new_threshold == 0 {
        return Err(AppError::bad_request("new_threshold must be > 0"));
    }

    // Ensure no pending rotation already exists for this vault.
    let existing: Option<(i16,)> = sqlx::query_as(
        "SELECT status FROM vault_shard_rotations
         WHERE vault_pubkey = $1 AND status = 0 LIMIT 1",
    )
    .bind(&vault_pubkey)
    .fetch_optional(&state.pool)
    .await?;
    if existing.is_some() {
        return Err(AppError::conflict(
            "a pending rotation already exists for this vault",
        ));
    }

    let effective_at = chrono::Utc::now() + chrono::Duration::seconds(7 * 24 * 3600);
    let row = sqlx::query_as::<_, RotationRow>(
        "INSERT INTO vault_shard_rotations
            (rotation_pubkey, vault_pubkey, old_ciphertext_hash, new_ciphertext_hash,
             new_shard_holders, new_threshold, initiated_by, required_endorsements,
             effective_at, status)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, 0)
         RETURNING id, rotation_pubkey, vault_pubkey, encode(old_ciphertext_hash, 'hex') AS old_ciphertext_hash,
                   encode(new_ciphertext_hash, 'hex') AS new_ciphertext_hash, new_shard_holders, new_threshold,
                   initiated_by, endorsements, required_endorsements,
                   initiated_at, effective_at, status, created_at",
    )
    .bind(format!("rot_{uuid}", uuid = Uuid::new_v4()))
    .bind(&vault_pubkey)
    .bind(&old_hash)
    .bind(&new_hash)
    .bind(&req.new_shard_holders)
    .bind(req.new_threshold as i16)
    .bind(&req.initiator)
    .bind(0i16) // required_endorsements set from on-chain
    .bind(effective_at)
    .fetch_one(&state.pool)
    .await?;
    Ok((StatusCode::CREATED, Json(row)))
}

/// Endorse a pending shard rotation.
pub async fn endorse_shard_rotation(
    State(state): State<AppState>,
    Path((vault_pubkey, rotation_pubkey)): Path<(String, String)>,
    Json(req): Json<EndorseRotation>,
) -> Result<Json<RotationRow>, AppError> {
    let row = sqlx::query_as::<_, RotationRow>(
        "UPDATE vault_shard_rotations
         SET endorsements = array_append(endorsements, $3)
         WHERE rotation_pubkey = $2 AND vault_pubkey = $1 AND status = 0
           AND NOT ($3 = ANY(endorsements))
         RETURNING id, rotation_pubkey, vault_pubkey, encode(old_ciphertext_hash, 'hex') AS old_ciphertext_hash,
                   encode(new_ciphertext_hash, 'hex') AS new_ciphertext_hash, new_shard_holders, new_threshold,
                   initiated_by, endorsements, required_endorsements,
                   initiated_at, effective_at, status, created_at",
    )
    .bind(&vault_pubkey)
    .bind(&rotation_pubkey)
    .bind(&req.validator)
    .fetch_one(&state.pool)
    .await?;
    Ok(Json(row))
}

/// Execute a shard rotation after time lock and quorum.
pub async fn execute_shard_rotation(
    State(state): State<AppState>,
    Path((vault_pubkey, rotation_pubkey)): Path<(String, String)>,
) -> Result<Json<VaultRow>, AppError> {
    // Verify the rotation is pending and effective.
    let rot = sqlx::query_as::<_, RotationRow>(
        "SELECT id, rotation_pubkey, vault_pubkey, encode(old_ciphertext_hash, 'hex') AS old_ciphertext_hash,
                encode(new_ciphertext_hash, 'hex') AS new_ciphertext_hash, new_shard_holders, new_threshold,
                initiated_by, endorsements, required_endorsements,
                initiated_at, effective_at, status, created_at
         FROM vault_shard_rotations
         WHERE rotation_pubkey = $1 AND vault_pubkey = $2 AND status = 0",
    )
    .bind(&rotation_pubkey)
    .bind(&vault_pubkey)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::not_found("no pending rotation found"))?;

    if chrono::Utc::now() < rot.effective_at {
        return Err(AppError::bad_request(
            "rotation time lock has not yet expired",
        ));
    }
    let endorsement_count = rot.endorsements.len() as i16;
    if endorsement_count < rot.required_endorsements {
        return Err(AppError::bad_request(format!(
            "insufficient endorsements: {endorsement_count}/{}",
            rot.required_endorsements
        )));
    }

    // Update the vault with the new state.
    let old_bytes = decode_hex64(&rot.old_ciphertext_hash)?;
    let new_bytes = decode_hex64(&rot.new_ciphertext_hash)?;
    let row = sqlx::query_as::<_, VaultRow>(
        "UPDATE vaults
         SET ciphertext_hash = $3,
             shard_holders    = $4,
             threshold        = $5,
             version          = version + 1,
             updated_at       = now()
         WHERE vault_pubkey = $1 AND ciphertext_hash = $2
         RETURNING id, subject_pubkey, vault_pubkey, ciphertext_cid,
                   encode(ciphertext_hash, 'hex') AS ciphertext_hash, algorithm_id, storage_uris, shard_holders,
                   threshold, version, last_ping_at, created_at, updated_at",
    )
    .bind(&vault_pubkey)
    .bind(&old_bytes)
    .bind(&new_bytes)
    .bind(&rot.new_shard_holders)
    .bind(rot.new_threshold)
    .fetch_one(&state.pool)
    .await?;

    // Mark rotation as executed.
    sqlx::query("UPDATE vault_shard_rotations SET status = 1 WHERE rotation_pubkey = $1")
        .bind(&rotation_pubkey)
        .execute(&state.pool)
        .await?;

    Ok(Json(row))
}

/// Cancel a pending shard rotation.
pub async fn cancel_shard_rotation(
    State(state): State<AppState>,
    Path((vault_pubkey, rotation_pubkey)): Path<(String, String)>,
) -> Result<Json<RotationRow>, AppError> {
    let row = sqlx::query_as::<_, RotationRow>(
        "UPDATE vault_shard_rotations
         SET status = 2
         WHERE rotation_pubkey = $1 AND vault_pubkey = $2 AND status = 0
         RETURNING id, rotation_pubkey, vault_pubkey, encode(old_ciphertext_hash, 'hex') AS old_ciphertext_hash,
                   encode(new_ciphertext_hash, 'hex') AS new_ciphertext_hash, new_shard_holders, new_threshold,
                   initiated_by, endorsements, required_endorsements,
                   initiated_at, effective_at, status, created_at",
    )
    .bind(&rotation_pubkey)
    .bind(&vault_pubkey)
    .fetch_one(&state.pool)
    .await?;
    Ok(Json(row))
}

/// Record a shard liveness ping.
pub async fn ping_shard(
    State(state): State<AppState>,
    Path(vault_pubkey): Path<String>,
    Json(req): Json<PingShard>,
) -> Result<Json<VaultRow>, AppError> {
    let row = sqlx::query_as::<_, VaultRow>(
        "UPDATE vaults
         SET last_ping_at = now(), updated_at = now()
         WHERE vault_pubkey = $1
         RETURNING id, subject_pubkey, vault_pubkey, ciphertext_cid,
                   encode(ciphertext_hash, 'hex') AS ciphertext_hash, algorithm_id, storage_uris, shard_holders,
                   threshold, version, last_ping_at, created_at, updated_at",
    )
    .bind(&vault_pubkey)
    .fetch_one(&state.pool)
    .await?;

    if !row.shard_holders.iter().any(|h| h == &req.validator) {
        return Err(AppError::bad_request(
            "validator is not a shard holder for this vault",
        ));
    }

    Ok(Json(row))
}

/// List access log entries for a vault.
pub async fn list_access_logs(
    State(state): State<AppState>,
    Path(vault_pubkey): Path<String>,
) -> Result<Json<Vec<AccessLogRow>>, AppError> {
    let rows = sqlx::query_as::<_, AccessLogRow>(
        "SELECT id, vault_pubkey, subject_pubkey, authority, purpose,
                expiry, encode(nonce, 'hex') AS nonce, block_time
         FROM vault_access_logs
         WHERE vault_pubkey = $1
         ORDER BY block_time DESC
         LIMIT 100",
    )
    .bind(&vault_pubkey)
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(rows))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_hex64_rejects_odd_length() {
        assert!(decode_hex64("abc").is_err());
    }

    #[test]
    fn decode_hex64_accepts_valid() {
        assert_eq!(decode_hex64("00ff").unwrap(), vec![0x00, 0xff]);
    }

    #[test]
    fn decode_hex64_empty_is_ok() {
        assert_eq!(decode_hex64("").unwrap(), Vec::<u8>::new());
    }
}
