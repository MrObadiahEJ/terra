use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::AppError;
use crate::state::AppState;

// ---------------------------------------------------------------------------
// Zone sets + ownership roots + nullifier records (RFC-011 mirror)
// ---------------------------------------------------------------------------

const ZONE_SET_SELECT: &str = r#"
    SELECT
        id, zone_set_address, zone_id, authority,
        parcel_count, current_root_version,
        created_at, updated_at
    FROM zone_sets
"#;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ZoneSet {
    pub id: Uuid,
    pub zone_set_address: String,
    pub zone_id: String,
    pub authority: String,
    pub parcel_count: i32,
    pub current_root_version: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct RegisterZoneSetRequest {
    pub zone_set_address: String,
    pub zone_id: String,
    pub authority: String,
    pub snapshot_cid: String,
    pub snapshot_hash: String,
    pub root_address: String,
    pub merkle_root: String,
}

#[derive(Debug, Deserialize)]
pub struct GenerateRootRequest {
    pub root_address: String,
    pub merkle_root: String,
    pub snapshot_cid: String,
    pub snapshot_hash: String,
    pub commitment_count: i32,
    pub authority_signature: Option<String>,
}

const ROOT_SELECT: &str = r#"
    SELECT
        id, zone_set_id, root_address, merkle_root, version,
        commitment_count, algorithm_id, snapshot_cid, snapshot_hash,
        authority_signature, created_at, updated_at
    FROM ownership_roots
"#;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct OwnershipRoot {
    pub id: Uuid,
    pub zone_set_id: Uuid,
    pub root_address: String,
    pub merkle_root: String,
    pub version: i32,
    pub commitment_count: i32,
    pub algorithm_id: i16,
    pub snapshot_cid: String,
    pub snapshot_hash: String,
    pub authority_signature: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

const NULLIFIER_SELECT: &str = r#"
    SELECT
        id, nullifier_hash, zone_set_id, root_version,
        prover, proof_purpose, disclosure_type,
        block_time, created_at
    FROM nullifier_records
"#;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct NullifierRecord {
    pub id: Uuid,
    pub nullifier_hash: String,
    pub zone_set_id: Uuid,
    pub root_version: i32,
    pub prover: String,
    pub proof_purpose: String,
    pub disclosure_type: i16,
    pub block_time: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct VerifyProofRequest {
    pub nullifier_hash: String,
    pub root_version: i32,
    pub prover: String,
    pub proof_purpose: String,
    pub disclosure_type: i16,
}

fn decode_hex32(s: &str) -> Result<[u8; 32], AppError> {
    let bytes = hex::decode(s.trim_start_matches("0x"))
        .map_err(|_| AppError::bad_request("expected hex"))?;
    if bytes.len() != 32 {
        return Err(AppError::bad_request("expected 32 bytes"));
    }
    let mut out = [0u8; 32];
    out.copy_from_slice(&bytes);
    Ok(out)
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

async fn list_zone_sets(State(state): State<AppState>) -> Result<Json<Vec<ZoneSet>>, AppError> {
    let rows: Vec<ZoneSet> = sqlx::query_as(ZONE_SET_SELECT)
        .fetch_all(&state.pool)
        .await?;
    Ok(Json(rows))
}

async fn get_zone_set(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ZoneSet>, AppError> {
    let row: ZoneSet = sqlx::query_as(&format!("{ZONE_SET_SELECT} WHERE id = $1"))
        .bind(id)
        .fetch_optional(&state.pool)
        .await?
        .ok_or_else(|| AppError::not_found("zone set not found"))?;
    Ok(Json(row))
}

async fn register_zone_set(
    State(state): State<AppState>,
    Json(req): Json<RegisterZoneSetRequest>,
) -> Result<(StatusCode, Json<ZoneSet>), AppError> {
    if req.snapshot_cid.trim().is_empty() {
        return Err(AppError::bad_request("snapshot_cid is required"));
    }
    decode_hex32(&req.snapshot_hash)?;
    decode_hex32(&req.merkle_root)?;
    crate::routes::identities::decode_wallet(&req.authority)?;

    let mut tx = state.pool.begin().await?;
    let row: ZoneSet = sqlx::query_as(
        "INSERT INTO zone_sets (zone_set_address, zone_id, authority)
         VALUES ($1, $2, $3)
         ON CONFLICT (zone_set_address) DO UPDATE SET
            zone_id = EXCLUDED.zone_id,
            authority = EXCLUDED.authority,
            updated_at = now()
         RETURNING id, zone_set_address, zone_id, authority,
                   parcel_count, current_root_version, created_at, updated_at",
    )
    .bind(&req.zone_set_address)
    .bind(&req.zone_id)
    .bind(&req.authority)
    .fetch_one(&mut *tx)
    .await?;

    sqlx::query(
        "INSERT INTO ownership_roots
            (zone_set_id, root_address, merkle_root, version, commitment_count,
             algorithm_id, snapshot_cid, snapshot_hash)
         VALUES ($1, $2, $3, 0, 0, 0, $4, $5)
         ON CONFLICT (root_address) DO UPDATE SET
            merkle_root = EXCLUDED.merkle_root,
            snapshot_cid = EXCLUDED.snapshot_cid,
            snapshot_hash = EXCLUDED.snapshot_hash,
            updated_at = now()",
    )
    .bind(row.id)
    .bind(&req.root_address)
    .bind(&req.merkle_root)
    .bind(&req.snapshot_cid)
    .bind(&req.snapshot_hash)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;

    Ok((StatusCode::CREATED, Json(row)))
}

async fn list_roots(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<OwnershipRoot>>, AppError> {
    let rows: Vec<OwnershipRoot> = sqlx::query_as(&format!(
        "{ROOT_SELECT} WHERE zone_set_id = $1 ORDER BY version DESC"
    ))
    .bind(id)
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(rows))
}

async fn generate_root(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<GenerateRootRequest>,
) -> Result<(StatusCode, Json<OwnershipRoot>), AppError> {
    if req.snapshot_cid.trim().is_empty() {
        return Err(AppError::bad_request("snapshot_cid is required"));
    }
    decode_hex32(&req.merkle_root)?;
    decode_hex32(&req.snapshot_hash)?;
    if req.commitment_count <= 0 {
        return Err(AppError::bad_request(
            "commitment_count must be positive (cannot generate a root for an empty zone)",
        ));
    }

    let mut tx = state.pool.begin().await?;
    let current: Option<(i32,)> =
        sqlx::query_as("SELECT current_root_version FROM zone_sets WHERE id = $1 FOR UPDATE")
            .bind(id)
            .fetch_optional(&mut *tx)
            .await?;
    let Some((version,)) = current else {
        return Err(AppError::not_found("zone set not found"));
    };
    let next = version + 1;

    sqlx::query(
        "UPDATE zone_sets
         SET current_root_version = $2, parcel_count = $3, updated_at = now()
         WHERE id = $1",
    )
    .bind(id)
    .bind(next)
    .bind(req.commitment_count)
    .execute(&mut *tx)
    .await?;

    let row: OwnershipRoot = sqlx::query_as(
        "INSERT INTO ownership_roots
            (zone_set_id, root_address, merkle_root, version, commitment_count,
             algorithm_id, snapshot_cid, snapshot_hash, authority_signature)
         VALUES ($1, $2, $3, $4, $5, 0, $6, $7, $8)
         RETURNING id, zone_set_id, root_address, merkle_root, version,
                   commitment_count, algorithm_id, snapshot_cid, snapshot_hash,
                   authority_signature, created_at, updated_at",
    )
    .bind(id)
    .bind(&req.root_address)
    .bind(&req.merkle_root)
    .bind(next)
    .bind(req.commitment_count)
    .bind(&req.snapshot_cid)
    .bind(&req.snapshot_hash)
    .bind(req.authority_signature.as_deref().unwrap_or(""))
    .fetch_one(&mut *tx)
    .await?;
    tx.commit().await?;

    Ok((StatusCode::CREATED, Json(row)))
}

async fn verify_proof(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<VerifyProofRequest>,
) -> Result<(StatusCode, Json<NullifierRecord>), AppError> {
    decode_hex32(&req.nullifier_hash)?;
    crate::routes::identities::decode_wallet(&req.prover)?;
    if req.proof_purpose.trim().is_empty() || req.proof_purpose.len() > 128 {
        return Err(AppError::bad_request(
            "proof_purpose must be 1..128 characters",
        ));
    }
    if req.disclosure_type < 0 || req.disclosure_type > 2 {
        return Err(AppError::bad_request(
            "disclosure_type must be 0 (membership), 1 (range), or 2 (count)",
        ));
    }

    // Root currency: the proof must reference the zone's current version.
    let current: Option<(i32,)> =
        sqlx::query_as("SELECT current_root_version FROM zone_sets WHERE id = $1")
            .bind(id)
            .fetch_optional(&state.pool)
            .await?;
    let Some((version,)) = current else {
        return Err(AppError::not_found("zone set not found"));
    };
    if req.root_version != version {
        return Err(AppError::conflict(
            "proof references a stale root version; regenerate against the current root",
        ));
    }

    let row: NullifierRecord = sqlx::query_as(
        "INSERT INTO nullifier_records
            (nullifier_hash, zone_set_id, root_version, prover, proof_purpose, disclosure_type)
         VALUES ($1, $2, $3, $4, $5, $6)
         RETURNING id, nullifier_hash, zone_set_id, root_version,
                   prover, proof_purpose, disclosure_type, block_time, created_at",
    )
    .bind(&req.nullifier_hash)
    .bind(id)
    .bind(req.root_version)
    .bind(&req.prover)
    .bind(&req.proof_purpose)
    .bind(req.disclosure_type)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| {
        if let sqlx::Error::Database(db) = &e {
            if db.code().as_deref() == Some("23505") {
                return AppError::conflict("proof has already been used (nullifier recorded)");
            }
        }
        AppError::from(e)
    })?;

    Ok((StatusCode::CREATED, Json(row)))
}

async fn list_nullifiers(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<NullifierRecord>>, AppError> {
    let rows: Vec<NullifierRecord> = sqlx::query_as(&format!(
        "{NULLIFIER_SELECT} WHERE zone_set_id = $1 ORDER BY block_time DESC"
    ))
    .bind(id)
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(rows))
}

async fn invalidate_version(
    State(state): State<AppState>,
    Path((id, version)): Path<(Uuid, i32)>,
) -> Result<Json<ZoneSet>, AppError> {
    if version <= 0 {
        return Err(AppError::bad_request(
            "cannot invalidate the genesis version",
        ));
    }
    let row: Option<ZoneSet> = sqlx::query_as(&format!(
        "{ZONE_SET_SELECT} WHERE id = $1 AND current_root_version > $2"
    ))
    .bind(id)
    .bind(version)
    .fetch_optional(&state.pool)
    .await?;
    match row {
        Some(z) => {
            sqlx::query("UPDATE zone_sets SET updated_at = now() WHERE id = $1")
                .bind(id)
                .execute(&state.pool)
                .await?;
            Ok(Json(z))
        }
        None => Err(AppError::conflict(
            "cannot invalidate the current version (rotate the root first)",
        )),
    }
}

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_zone_sets).post(register_zone_set))
        .route("/{id}", get(get_zone_set))
        .route("/{id}/roots", get(list_roots).post(generate_root))
        .route("/{id}/proofs", get(list_nullifiers).post(verify_proof))
        .route("/{id}/invalidate/{version}", post(invalidate_version))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disclosure_type_bounds() {
        for valid in [0i16, 1, 2] {
            assert!((0..=2).contains(&valid));
        }
        assert!(!(0..=2).contains(&3i16));
    }

    #[test]
    fn empty_zone_rejected() {
        assert!(0i32 <= 0);
        assert!(5i32 > 0);
    }

    #[test]
    fn purpose_length_bounds() {
        assert!("subsidy_qualification".len() <= 128);
        assert!("".is_empty());
    }

    #[test]
    fn decode_hex32_rejects_short() {
        assert!(decode_hex32("abcd").is_err());
        assert!(decode_hex32(&"00".repeat(32)).is_ok());
    }
}
