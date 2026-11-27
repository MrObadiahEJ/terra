//! Mirror audit (closes RISK R-004: mirror drift).
//!
//! Every off-chain mirror row carries a `mirror_hash` (SHA-256 hex over the
//! canonical serialization of its mirrored fields), computed by a database
//! trigger on every INSERT/UPDATE. `GET /mirror/audit` recomputes the hash
//! for every row and reports:
//!
//! - rows whose stored `mirror_hash` no longer matches the row contents
//!   (manual DB edit or trigger bypass — the hash column itself is tamper-evident)
//! - anchored rows (with `onchain_id`) whose on-chain commitment columns are
//!   missing (incomplete anchoring)
//!
//! The chain remains the source of truth; this endpoint is the read-side
//! consistency check that detects drift between mirror and on-chain state.

use axum::extract::{Query, State};
use axum::routing::get;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::error::AppError;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct AuditParams {
    /// Maximum number of drifted rows to return per table (default 50).
    #[serde(default = "default_limit")]
    pub limit: usize,
}

fn default_limit() -> usize {
    50
}

#[derive(Debug, Serialize)]
pub struct TableAudit {
    pub table: String,
    pub total_rows: i64,
    pub hashed_rows: i64,
    pub unhashed_rows: i64,
    pub unverified_rows: i64,
    pub drifted_rows: i64,
    pub anchored_rows: i64,
    pub unanchored_rows: i64,
    pub drift: Vec<DriftedRow>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct DriftedRow {
    pub id: String,
    pub kind: String, // "hash-mismatch" | "missing-onchain-commitment"
    pub detail: String,
}

#[derive(Debug, Serialize)]
pub struct AuditReport {
    pub tables: Vec<TableAudit>,
    pub total_drifted: i64,
    pub ok: bool,
}

pub fn router() -> Router<AppState> {
    Router::new().route("/audit", get(audit))
}

async fn audit(
    State(state): State<AppState>,
    Query(params): Query<AuditParams>,
) -> Result<Json<AuditReport>, AppError> {
    let limit = params.limit.clamp(1, 500);
    let mut tables = Vec::new();
    let mut total_drifted = 0i64;
    let mut ok = true;

    // --- parcels ---
    let parcel_audit = audit_parcels(&state, limit).await?;
    total_drifted += parcel_audit.drifted_rows;
    if parcel_audit.drifted_rows > 0
        || parcel_audit.unhashed_rows > 0
        || parcel_audit.unanchored_rows > 0
        || parcel_audit.unverified_rows > 0
    {
        ok = false;
    }
    tables.push(parcel_audit);

    // --- rights ---
    let rights_audit = audit_rights(&state, limit).await?;
    total_drifted += rights_audit.drifted_rows;
    if rights_audit.drifted_rows > 0
        || rights_audit.unhashed_rows > 0
        || rights_audit.unverified_rows > 0
    {
        ok = false;
    }
    tables.push(rights_audit);

    // --- spatial_assets_onchain ---
    let sa_audit = audit_spatial_assets(&state, limit).await?;
    total_drifted += sa_audit.drifted_rows;
    if sa_audit.drifted_rows > 0 || sa_audit.unhashed_rows > 0 || sa_audit.unverified_rows > 0 {
        ok = false;
    }
    tables.push(sa_audit);

    // --- geometry_versions_onchain ---
    let gv_audit = audit_geometry_versions(&state, limit).await?;
    total_drifted += gv_audit.drifted_rows;
    if gv_audit.drifted_rows > 0 || gv_audit.unhashed_rows > 0 || gv_audit.unverified_rows > 0 {
        ok = false;
    }
    tables.push(gv_audit);

    // --- three_d_models_onchain ---
    let td_audit = audit_three_d_models(&state, limit).await?;
    total_drifted += td_audit.drifted_rows;
    if td_audit.drifted_rows > 0 || td_audit.unhashed_rows > 0 || td_audit.unverified_rows > 0 {
        ok = false;
    }
    tables.push(td_audit);

    // --- spatial_snapshots_onchain ---
    let ss_audit = audit_spatial_snapshots(&state, limit).await?;
    total_drifted += ss_audit.drifted_rows;
    if ss_audit.drifted_rows > 0 || ss_audit.unhashed_rows > 0 || ss_audit.unverified_rows > 0 {
        ok = false;
    }
    tables.push(ss_audit);

    // --- verification_attestations_onchain ---
    let va_audit = audit_verification_attestations(&state, limit).await?;
    total_drifted += va_audit.drifted_rows;
    if va_audit.drifted_rows > 0 || va_audit.unhashed_rows > 0 || va_audit.unverified_rows > 0 {
        ok = false;
    }
    tables.push(va_audit);

    // --- claims_onchain (migration 0057) ---
    let claims_audit = audit_claims(&state, limit).await?;
    total_drifted += claims_audit.drifted_rows;
    if claims_audit.drifted_rows > 0
        || claims_audit.unhashed_rows > 0
        || claims_audit.unverified_rows > 0
    {
        ok = false;
    }
    tables.push(claims_audit);

    // --- chain_accounts (migration 0060, Q-033 stage 1) ---
    let generic_audit = audit_chain_accounts(&state, limit).await?;
    total_drifted += generic_audit.drifted_rows;
    if generic_audit.drifted_rows > 0
        || generic_audit.unhashed_rows > 0
        || generic_audit.unverified_rows > 0
    {
        ok = false;
    }
    tables.push(generic_audit);

    // --- parcel_geometry_artifacts (migration 0061, Q-035(a)) ---
    let artifact_audit = audit_parcel_geometry_artifacts(&state, limit).await?;
    total_drifted += artifact_audit.drifted_rows;
    if artifact_audit.drifted_rows > 0
        || artifact_audit.unhashed_rows > 0
        || artifact_audit.unverified_rows > 0
    {
        ok = false;
    }
    tables.push(artifact_audit);

    Ok(Json(AuditReport {
        tables,
        total_drifted,
        ok,
    }))
}

// Recompute the parcels mirror hash in SQL and compare with the stored value.
// Also flags anchored rows (onchain_id set) whose geometry_hash/access_hash
// are missing (incomplete on-chain anchoring).
async fn audit_parcels(state: &AppState, limit: usize) -> Result<TableAudit, AppError> {
    let total: i64 = sqlx::query_scalar("SELECT count(*) FROM parcels")
        .fetch_one(&state.pool)
        .await?;
    let hashed: i64 =
        sqlx::query_scalar("SELECT count(*) FROM parcels WHERE mirror_hash IS NOT NULL")
            .fetch_one(&state.pool)
            .await?;
    let anchored: i64 =
        sqlx::query_scalar("SELECT count(*) FROM parcels WHERE onchain_id IS NOT NULL")
            .fetch_one(&state.pool)
            .await?;
    let unverified: i64 =
        sqlx::query_scalar("SELECT count(*) FROM parcels WHERE chain_verified = FALSE")
            .fetch_one(&state.pool)
            .await?;

    // Recompute hash and compare.
    let drifted = sqlx::query_as::<_, DriftedRow>(
        "SELECT id::text AS id, 'hash-mismatch' AS kind, \
                'stored mirror_hash does not match recomputed row hash' AS detail \
         FROM parcels \
         WHERE mirror_hash IS NOT NULL \
           AND mirror_hash <> encode(digest( \
                 coalesce(name, '') || '|' || \
                 coalesce(status, '') || '|' || \
                 coalesce(ST_AsEWKB(geometry), '') || '|' || \
                 coalesce(source_geometry, '') || '|' || \
                 coalesce(onchain_id, '') || '|' || \
                 coalesce(geometry_hash, '') || '|' || \
                 coalesce(access_hash, '') || '|' || \
                 infrastructure_flags::text || '|' || \
                 rights_count::text, 'sha256'), 'hex') \
         LIMIT $1",
    )
    .bind(limit as i32)
    .fetch_all(&state.pool)
    .await?;

    // Anchored rows missing on-chain commitment columns.
    let unanchored = sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM parcels \
         WHERE onchain_id IS NOT NULL \
           AND (geometry_hash IS NULL OR access_hash IS NULL)",
    )
    .fetch_one(&state.pool)
    .await?;

    // Unverified rows: not yet confirmed by the indexer against the chain.
    let unverified_drift = sqlx::query_as::<_, DriftedRow>(
        "SELECT id::text AS id, 'chain-unverified' AS kind, \
                'mirror row not yet confirmed against on-chain state' AS detail \
         FROM parcels \
         WHERE chain_verified = FALSE \
         LIMIT $1",
    )
    .bind(limit as i32)
    .fetch_all(&state.pool)
    .await?;

    let mut all_drift = drifted;
    all_drift.extend(unverified_drift);

    Ok(TableAudit {
        table: "parcels".into(),
        total_rows: total,
        hashed_rows: hashed,
        unhashed_rows: total - hashed,
        unverified_rows: unverified,
        drifted_rows: all_drift.len() as i64,
        anchored_rows: anchored,
        unanchored_rows: unanchored,
        drift: all_drift,
    })
}

async fn audit_rights(state: &AppState, limit: usize) -> Result<TableAudit, AppError> {
    let total: i64 = sqlx::query_scalar("SELECT count(*) FROM rights")
        .fetch_one(&state.pool)
        .await?;
    let hashed: i64 =
        sqlx::query_scalar("SELECT count(*) FROM rights WHERE mirror_hash IS NOT NULL")
            .fetch_one(&state.pool)
            .await?;
    let unverified: i64 =
        sqlx::query_scalar("SELECT count(*) FROM rights WHERE chain_verified = FALSE")
            .fetch_one(&state.pool)
            .await?;

    let drifted = sqlx::query_as::<_, DriftedRow>(
        "SELECT id::text AS id, 'hash-mismatch' AS kind, \
                'stored mirror_hash does not match recomputed row hash' AS detail \
         FROM rights \
         WHERE mirror_hash IS NOT NULL \
           AND mirror_hash <> encode(digest( \
                 coalesce(onchain_id, '') || '|' || \
                 coalesce(parcel_id::text, '') || '|' || \
                 rights_kind::text || '|' || \
                 coalesce(holder, '') || '|' || \
                 coalesce(granter, '') || '|' || \
                 coalesce(notes, '') || '|' || \
                 coalesce(to_char(created_at, 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'), '') || '|' || \
                 coalesce(to_char(expires_at, 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'), ''), 'sha256'), 'hex') \
          LIMIT $1",
    )
    .bind(limit as i32)
    .fetch_all(&state.pool)
    .await?;

    let unverified_drift = sqlx::query_as::<_, DriftedRow>(
        "SELECT id::text AS id, 'chain-unverified' AS kind, \
                'mirror row not yet confirmed against on-chain state' AS detail \
         FROM rights \
         WHERE chain_verified = FALSE \
         LIMIT $1",
    )
    .bind(limit as i32)
    .fetch_all(&state.pool)
    .await?;

    let mut all_drift = drifted;
    all_drift.extend(unverified_drift);

    Ok(TableAudit {
        table: "rights".into(),
        total_rows: total,
        hashed_rows: hashed,
        unhashed_rows: total - hashed,
        unverified_rows: unverified,
        drifted_rows: all_drift.len() as i64,
        anchored_rows: 0,
        unanchored_rows: 0,
        drift: all_drift,
    })
}

async fn audit_spatial_assets(state: &AppState, limit: usize) -> Result<TableAudit, AppError> {
    let total: i64 = sqlx::query_scalar("SELECT count(*) FROM spatial_assets_onchain")
        .fetch_one(&state.pool)
        .await?;
    let hashed: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM spatial_assets_onchain WHERE mirror_hash IS NOT NULL",
    )
    .fetch_one(&state.pool)
    .await?;
    let unverified: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM spatial_assets_onchain WHERE chain_verified = FALSE",
    )
    .fetch_one(&state.pool)
    .await?;

    let drifted = sqlx::query_as::<_, DriftedRow>(
        "SELECT id::text AS id, 'hash-mismatch' AS kind, \
                'stored mirror_hash does not match recomputed row hash' AS detail \
         FROM spatial_assets_onchain \
         WHERE mirror_hash IS NOT NULL \
           AND mirror_hash <> encode(digest( \
                 coalesce(pubkey, '') || '|' || \
                 coalesce(parcel_pubkey, '') || '|' || \
                 coalesce(authority, '') || '|' || \
                 dimensionality::text || '|' || \
                 elevation_min_mm::text || '|' || \
                 elevation_max_mm::text || '|' || \
                 geometry_version_count::text || '|' || \
                 coalesce(latest_geometry_hash, '') || '|' || \
                 coalesce(to_char(chain_created_at, 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'), '') || '|' || \
                 coalesce(to_char(chain_updated_at, 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'), ''), 'sha256'), 'hex') \
         LIMIT $1",
    )
    .bind(limit as i32)
    .fetch_all(&state.pool)
    .await?;

    let unverified_drift = sqlx::query_as::<_, DriftedRow>(
        "SELECT id::text AS id, 'chain-unverified' AS kind, \
                'mirror row not yet confirmed against on-chain state' AS detail \
         FROM spatial_assets_onchain \
         WHERE chain_verified = FALSE \
         LIMIT $1",
    )
    .bind(limit as i32)
    .fetch_all(&state.pool)
    .await?;

    let mut all_drift = drifted;
    all_drift.extend(unverified_drift);

    Ok(TableAudit {
        table: "spatial_assets_onchain".into(),
        total_rows: total,
        hashed_rows: hashed,
        unhashed_rows: total - hashed,
        unverified_rows: unverified,
        drifted_rows: all_drift.len() as i64,
        anchored_rows: 0,
        unanchored_rows: 0,
        drift: all_drift,
    })
}

async fn audit_geometry_versions(state: &AppState, limit: usize) -> Result<TableAudit, AppError> {
    let total: i64 = sqlx::query_scalar("SELECT count(*) FROM geometry_versions_onchain")
        .fetch_one(&state.pool)
        .await?;
    let hashed: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM geometry_versions_onchain WHERE mirror_hash IS NOT NULL",
    )
    .fetch_one(&state.pool)
    .await?;
    let unverified: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM geometry_versions_onchain WHERE chain_verified = FALSE",
    )
    .fetch_one(&state.pool)
    .await?;

    let drifted = sqlx::query_as::<_, DriftedRow>(
        "SELECT id::text AS id, 'hash-mismatch' AS kind, \
                'stored mirror_hash does not match recomputed row hash' AS detail \
         FROM geometry_versions_onchain \
         WHERE mirror_hash IS NOT NULL \
           AND mirror_hash <> encode(digest( \
                 coalesce(pubkey, '') || '|' || \
                 coalesce(asset_pubkey, '') || '|' || \
                 coalesce(parcel_pubkey, '') || '|' || \
                 version::text || '|' || \
                 coalesce(geometry_hash, '') || '|' || \
                 source::text || '|' || \
                 dimension::text || '|' || \
                 coalesce(storage_reference, '') || '|' || \
                 coalesce(submitted_by, '') || '|' || \
                 verified::text || '|' || \
                 coalesce(verified_by, '') || '|' || \
                 elevation_min_mm::text || '|' || \
                 elevation_max_mm::text || '|' || \
                 elevation_source::text || '|' || \
                 required::text || '|' || \
                 attest_count::text || '|' || \
                 coalesce(array_to_string(attestors, ','), ''), 'sha256'), 'hex') \
         LIMIT $1",
    )
    .bind(limit as i32)
    .fetch_all(&state.pool)
    .await?;

    let unverified_drift = sqlx::query_as::<_, DriftedRow>(
        "SELECT id::text AS id, 'chain-unverified' AS kind, \
                'mirror row not yet confirmed against on-chain state' AS detail \
         FROM geometry_versions_onchain \
         WHERE chain_verified = FALSE \
         LIMIT $1",
    )
    .bind(limit as i32)
    .fetch_all(&state.pool)
    .await?;

    let mut all_drift = drifted;
    all_drift.extend(unverified_drift);

    Ok(TableAudit {
        table: "geometry_versions_onchain".into(),
        total_rows: total,
        hashed_rows: hashed,
        unhashed_rows: total - hashed,
        unverified_rows: unverified,
        drifted_rows: all_drift.len() as i64,
        anchored_rows: 0,
        unanchored_rows: 0,
        drift: all_drift,
    })
}

async fn audit_three_d_models(state: &AppState, limit: usize) -> Result<TableAudit, AppError> {
    let total: i64 = sqlx::query_scalar("SELECT count(*) FROM three_d_models_onchain")
        .fetch_one(&state.pool)
        .await?;
    let hashed: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM three_d_models_onchain WHERE mirror_hash IS NOT NULL",
    )
    .fetch_one(&state.pool)
    .await?;
    let unverified: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM three_d_models_onchain WHERE chain_verified = FALSE",
    )
    .fetch_one(&state.pool)
    .await?;

    let drifted = sqlx::query_as::<_, DriftedRow>(
        "SELECT id::text AS id, 'hash-mismatch' AS kind, \
                'stored mirror_hash does not match recomputed row hash' AS detail \
         FROM three_d_models_onchain \
         WHERE mirror_hash IS NOT NULL \
           AND mirror_hash <> encode(digest( \
                 coalesce(pubkey, '') || '|' || \
                 coalesce(asset_pubkey, '') || '|' || \
                 coalesce(parcel_pubkey, '') || '|' || \
                 model_version::text || '|' || \
                 geometry_version::text || '|' || \
                 coalesce(model_hash, '') || '|' || \
                 coalesce(storage_reference, '') || '|' || \
                 model_format::text || '|' || \
                 coalesce(submitted_by, '') || '|' || \
                 verified::text || '|' || \
                 coalesce(verified_by, ''), 'sha256'), 'hex') \
         LIMIT $1",
    )
    .bind(limit as i32)
    .fetch_all(&state.pool)
    .await?;

    let unverified_drift = sqlx::query_as::<_, DriftedRow>(
        "SELECT id::text AS id, 'chain-unverified' AS kind, \
                'mirror row not yet confirmed against on-chain state' AS detail \
         FROM three_d_models_onchain \
         WHERE chain_verified = FALSE \
         LIMIT $1",
    )
    .bind(limit as i32)
    .fetch_all(&state.pool)
    .await?;

    let mut all_drift = drifted;
    all_drift.extend(unverified_drift);

    Ok(TableAudit {
        table: "three_d_models_onchain".into(),
        total_rows: total,
        hashed_rows: hashed,
        unhashed_rows: total - hashed,
        unverified_rows: unverified,
        drifted_rows: all_drift.len() as i64,
        anchored_rows: 0,
        unanchored_rows: 0,
        drift: all_drift,
    })
}

async fn audit_spatial_snapshots(state: &AppState, limit: usize) -> Result<TableAudit, AppError> {
    let total: i64 = sqlx::query_scalar("SELECT count(*) FROM spatial_snapshots_onchain")
        .fetch_one(&state.pool)
        .await?;
    let hashed: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM spatial_snapshots_onchain WHERE mirror_hash IS NOT NULL",
    )
    .fetch_one(&state.pool)
    .await?;
    let unverified: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM spatial_snapshots_onchain WHERE chain_verified = FALSE",
    )
    .fetch_one(&state.pool)
    .await?;

    let drifted = sqlx::query_as::<_, DriftedRow>(
        "SELECT id::text AS id, 'hash-mismatch' AS kind, \
                'stored mirror_hash does not match recomputed row hash' AS detail \
         FROM spatial_snapshots_onchain \
         WHERE mirror_hash IS NOT NULL \
           AND mirror_hash <> encode(digest( \
                 coalesce(pubkey, '') || '|' || \
                 coalesce(asset_pubkey, '') || '|' || \
                 coalesce(parcel_pubkey, '') || '|' || \
                 snapshot_index::text || '|' || \
                 geometry_version_count::text || '|' || \
                 coalesce(latest_geometry_hash, '') || '|' || \
                 dimensionality::text || '|' || \
                 elevation_min_mm::text || '|' || \
                 elevation_max_mm::text || '|' || \
                 coalesce(label, '') || '|' || \
                 coalesce(created_by, '') || '|' || \
                 coalesce(to_char(chain_created_at, 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'), ''), 'sha256'), 'hex') \
         LIMIT $1",
    )
    .bind(limit as i32)
    .fetch_all(&state.pool)
    .await?;

    let unverified_drift = sqlx::query_as::<_, DriftedRow>(
        "SELECT id::text AS id, 'chain-unverified' AS kind, \
                'mirror row not yet confirmed against on-chain state' AS detail \
         FROM spatial_snapshots_onchain \
         WHERE chain_verified = FALSE \
         LIMIT $1",
    )
    .bind(limit as i32)
    .fetch_all(&state.pool)
    .await?;

    let mut all_drift = drifted;
    all_drift.extend(unverified_drift);

    Ok(TableAudit {
        table: "spatial_snapshots_onchain".into(),
        total_rows: total,
        hashed_rows: hashed,
        unhashed_rows: total - hashed,
        unverified_rows: unverified,
        drifted_rows: all_drift.len() as i64,
        anchored_rows: 0,
        unanchored_rows: 0,
        drift: all_drift,
    })
}

async fn audit_verification_attestations(
    state: &AppState,
    limit: usize,
) -> Result<TableAudit, AppError> {
    let total: i64 = sqlx::query_scalar("SELECT count(*) FROM verification_attestations_onchain")
        .fetch_one(&state.pool)
        .await?;
    let hashed: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM verification_attestations_onchain WHERE mirror_hash IS NOT NULL",
    )
    .fetch_one(&state.pool)
    .await?;
    let unverified: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM verification_attestations_onchain WHERE chain_verified = FALSE",
    )
    .fetch_one(&state.pool)
    .await?;

    let drifted = sqlx::query_as::<_, DriftedRow>(
        "SELECT id::text AS id, 'hash-mismatch' AS kind, \
                'stored mirror_hash does not match recomputed row hash' AS detail \
         FROM verification_attestations_onchain \
         WHERE mirror_hash IS NOT NULL \
           AND mirror_hash <> encode(digest( \
                 coalesce(claim_pubkey, '') || '|' || \
                 coalesce(validator_pubkey, '') || '|' || \
                 coalesce(observation_pubkey, '') || '|' || \
                 coalesce(credential_subject, '') || '|' || \
                 vc_type || '|' || \
                 to_char(issuance_date, 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"') || '|' || \
                 to_char(expiration_date, 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"') || '|' || \
                 sd_jwt_digest || '|' || \
                 selective_disclosure::text, 'sha256'), 'hex') \
         LIMIT $1",
    )
    .bind(limit as i32)
    .fetch_all(&state.pool)
    .await?;

    let unverified_drift = sqlx::query_as::<_, DriftedRow>(
        "SELECT id::text AS id, 'chain-unverified' AS kind, \
                'mirror row not yet confirmed against on-chain state' AS detail \
         FROM verification_attestations_onchain \
         WHERE chain_verified = FALSE \
         LIMIT $1",
    )
    .bind(limit as i32)
    .fetch_all(&state.pool)
    .await?;

    let mut all_drift = drifted;
    all_drift.extend(unverified_drift);

    Ok(TableAudit {
        table: "verification_attestations_onchain".into(),
        total_rows: total,
        hashed_rows: hashed,
        unhashed_rows: total - hashed,
        unverified_rows: unverified,
        drifted_rows: all_drift.len() as i64,
        anchored_rows: 0,
        unanchored_rows: 0,
        drift: all_drift,
    })
}

/// Claims mirrored from on-chain `Claim` accounts (migration 0057).
/// The hash expression below is intentionally identical to
/// `compute_claims_onchain_mirror_hash()` in 0057; a mismatch here would
/// report every honest row as drifted.
async fn audit_claims(state: &AppState, limit: usize) -> Result<TableAudit, AppError> {
    let total: i64 = sqlx::query_scalar("SELECT count(*) FROM claims_onchain")
        .fetch_one(&state.pool)
        .await?;
    let hashed: i64 =
        sqlx::query_scalar("SELECT count(*) FROM claims_onchain WHERE mirror_hash IS NOT NULL")
            .fetch_one(&state.pool)
            .await?;
    let unverified: i64 =
        sqlx::query_scalar("SELECT count(*) FROM claims_onchain WHERE chain_verified = FALSE")
            .fetch_one(&state.pool)
            .await?;

    let drifted = sqlx::query_as::<_, DriftedRow>(
        "SELECT id::text AS id, 'hash-mismatch' AS kind, \
                'stored mirror_hash does not match recomputed row hash' AS detail \
         FROM claims_onchain \
         WHERE mirror_hash IS NOT NULL \
           AND mirror_hash <> encode(digest( \
                 coalesce(pubkey, '') || '|' || \
                 coalesce(claim_id, '') || '|' || \
                 coalesce(parcel_pubkey, '') || '|' || \
                 claim_type::text || '|' || \
                 coalesce(submitted_by, '') || '|' || \
                 status::text || '|' || \
                 coalesce(statement_hash, '') || '|' || \
                 version::text || '|' || \
                 evidence_count::text || '|' || \
                 observation_count::text || '|' || \
                 attestation_count::text || '|' || \
                 required_attestations::text || '|' || \
                 coalesce(to_char(chain_created_at, 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'), '') || '|' || \
                 coalesce(to_char(chain_updated_at, 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'), ''), 'sha256'), 'hex') \
         LIMIT $1",
    )
    .bind(limit as i32)
    .fetch_all(&state.pool)
    .await?;

    let unverified_drift = sqlx::query_as::<_, DriftedRow>(
        "SELECT id::text AS id, 'chain-unverified' AS kind, \
                'mirror row not yet confirmed against on-chain state' AS detail \
         FROM claims_onchain \
         WHERE chain_verified = FALSE \
         LIMIT $1",
    )
    .bind(limit as i32)
    .fetch_all(&state.pool)
    .await?;

    let drifted_rows = drifted.len() + unverified_drift.len();
    let mut all_drift = drifted;
    all_drift.extend(unverified_drift);

    Ok(TableAudit {
        table: "claims_onchain".into(),
        total_rows: total,
        hashed_rows: hashed,
        unhashed_rows: total - hashed,
        unverified_rows: unverified,
        drifted_rows: drifted_rows as i64,
        anchored_rows: 0,
        unanchored_rows: 0,
        drift: all_drift,
    })
}

/// Generic all-IDL-account mirror (Q-033 stage 1). Every row must have a
/// trigger-maintained hash and, per ADR-026, a real transaction provenance.
async fn audit_chain_accounts(state: &AppState, limit: usize) -> Result<TableAudit, AppError> {
    let total: i64 = sqlx::query_scalar("SELECT count(*) FROM chain_accounts")
        .fetch_one(&state.pool)
        .await?;
    let hashed: i64 =
        sqlx::query_scalar("SELECT count(*) FROM chain_accounts WHERE mirror_hash IS NOT NULL")
            .fetch_one(&state.pool)
            .await?;
    let unverified: i64 =
        sqlx::query_scalar("SELECT count(*) FROM chain_accounts WHERE chain_verified = FALSE")
            .fetch_one(&state.pool)
            .await?;

    let drifted = sqlx::query_as::<_, DriftedRow>(
        "SELECT pubkey AS id, 'hash-mismatch' AS kind, \
                'stored mirror_hash does not match recomputed row hash' AS detail \
         FROM chain_accounts \
         WHERE mirror_hash IS NOT NULL \
           AND mirror_hash <> encode(digest( \
                 pubkey || '|' || account_type || '|' || fields::text, 'sha256'), 'hex') \
         LIMIT $1",
    )
    .bind(limit as i32)
    .fetch_all(&state.pool)
    .await?;

    let unverified_drift = sqlx::query_as::<_, DriftedRow>(
        "SELECT pubkey AS id, 'chain-unverified' AS kind, \
                'mirror row not yet confirmed against on-chain state' AS detail \
         FROM chain_accounts WHERE chain_verified = FALSE LIMIT $1",
    )
    .bind(limit as i32)
    .fetch_all(&state.pool)
    .await?;

    let drifted_rows = (drifted.len() + unverified_drift.len()) as i64;
    let mut all_drift = drifted;
    all_drift.extend(unverified_drift);

    Ok(TableAudit {
        table: "chain_accounts".into(),
        total_rows: total,
        hashed_rows: hashed,
        unhashed_rows: total - hashed,
        unverified_rows: unverified,
        drifted_rows,
        anchored_rows: 0,
        unanchored_rows: 0,
        drift: all_drift,
    })
}

/// Typed mirror for the Q-035(a) content-addressed artifact PDA.
async fn audit_parcel_geometry_artifacts(
    state: &AppState,
    limit: usize,
) -> Result<TableAudit, AppError> {
    let total: i64 = sqlx::query_scalar("SELECT count(*) FROM parcel_geometry_artifacts")
        .fetch_one(&state.pool)
        .await?;
    let hashed: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM parcel_geometry_artifacts WHERE mirror_hash IS NOT NULL",
    )
    .fetch_one(&state.pool)
    .await?;
    let unverified: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM parcel_geometry_artifacts WHERE chain_verified = FALSE",
    )
    .fetch_one(&state.pool)
    .await?;

    let drifted = sqlx::query_as::<_, DriftedRow>(
        "SELECT pubkey AS id, 'hash-mismatch' AS kind, \
                'stored mirror_hash does not match recomputed row hash' AS detail \
         FROM parcel_geometry_artifacts \
         WHERE mirror_hash IS NOT NULL \
           AND mirror_hash <> encode(digest( \
                 pubkey || '|' || parcel_pubkey || '|' || content_hash || '|' || \
                 storage_uri || '|' || registrant || '|' || \
                 coalesce(to_char(chain_created_at, 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'), ''), \
                 'sha256'), 'hex') \
         LIMIT $1",
    )
    .bind(limit as i32)
    .fetch_all(&state.pool)
    .await?;

    let unverified_drift = sqlx::query_as::<_, DriftedRow>(
        "SELECT pubkey AS id, 'chain-unverified' AS kind, \
                'mirror row not yet confirmed against on-chain state' AS detail \
         FROM parcel_geometry_artifacts WHERE chain_verified = FALSE LIMIT $1",
    )
    .bind(limit as i32)
    .fetch_all(&state.pool)
    .await?;

    let drifted_rows = (drifted.len() + unverified_drift.len()) as i64;
    let mut all_drift = drifted;
    all_drift.extend(unverified_drift);

    Ok(TableAudit {
        table: "parcel_geometry_artifacts".into(),
        total_rows: total,
        hashed_rows: hashed,
        unhashed_rows: total - hashed,
        unverified_rows: unverified,
        drifted_rows,
        anchored_rows: 0,
        unanchored_rows: 0,
        drift: all_drift,
    })
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    #[tokio::test]
    async fn mirror_audit_sql_recomputes_parcel_hash() {
        let Ok(database_url) = std::env::var("DATABASE_URL") else {
            eprintln!("skipping mirror audit test: DATABASE_URL is not set");
            return;
        };
        let pool = sqlx::PgPool::connect(&database_url)
            .await
            .expect("connect test database");

        // Insert a parcel, verify its mirror_hash is auto-computed by the trigger.
        let id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO parcels (id, name, status, geometry) \
             VALUES ($1, $2, 'pending', ST_GeomFromGeoJSON($3))",
        )
        .bind(id)
        .bind("audit-test-parcel")
        .bind(serde_json::json!({
            "type": "Polygon",
            "coordinates": [[[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0], [0.0, 0.0]]]
        }))
        .execute(&pool)
        .await
        .expect("insert test parcel");

        let stored: Option<String> =
            sqlx::query_scalar("SELECT mirror_hash FROM parcels WHERE id = $1")
                .bind(id)
                .fetch_optional(&pool)
                .await
                .expect("fetch mirror_hash");
        assert!(
            stored.is_some(),
            "trigger must compute mirror_hash on insert"
        );
        let hash = stored.unwrap();
        assert_eq!(hash.len(), 64, "mirror_hash must be 64-char hex");

        // Recompute the same hash in SQL and verify it matches the stored value.
        let recomputed: Option<String> = sqlx::query_scalar(
            "SELECT encode(digest( \
                 coalesce(name, '') || '|' || \
                 coalesce(status, '') || '|' || \
                 coalesce(ST_AsEWKB(geometry), '') || '|' || \
                 coalesce(source_geometry, '') || '|' || \
                 coalesce(onchain_id, '') || '|' || \
                 coalesce(geometry_hash, '') || '|' || \
                 coalesce(access_hash, '') || '|' || \
                 infrastructure_flags::text || '|' || \
                 rights_count::text, 'sha256'), 'hex') \
             FROM parcels WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&pool)
        .await
        .expect("recompute parcel hash");
        assert_eq!(
            recomputed,
            Some(hash.clone()),
            "recomputed hash must equal stored hash"
        );

        // Update the parcel name -> trigger must recompute mirror_hash -> hash must change.
        sqlx::query("UPDATE parcels SET name = $1 WHERE id = $2")
            .bind("audit-test-parcel-updated")
            .bind(id)
            .execute(&pool)
            .await
            .expect("update test parcel");

        let updated: Option<String> =
            sqlx::query_scalar("SELECT mirror_hash FROM parcels WHERE id = $1")
                .bind(id)
                .fetch_optional(&pool)
                .await
                .expect("fetch updated mirror_hash");
        assert_ne!(
            updated,
            Some(hash),
            "mirror_hash must change when row contents change"
        );
        assert!(updated.is_some() && updated.as_deref().unwrap().len() == 64);

        // Clean up.
        sqlx::query("DELETE FROM parcels WHERE id = $1")
            .bind(id)
            .execute(&pool)
            .await
            .expect("delete test parcel");
    }

    #[tokio::test]
    async fn mirror_audit_claims_hash_matches_trigger() {
        let Ok(database_url) = std::env::var("DATABASE_URL") else {
            eprintln!("skipping mirror audit test: DATABASE_URL is not set");
            return;
        };
        let pool = sqlx::PgPool::connect(&database_url)
            .await
            .expect("connect test database");

        let pubkey = format!("claims-audit-{}", Uuid::new_v4());
        sqlx::query(
            "INSERT INTO claims_onchain (pubkey, claim_type, status, version, \
             evidence_count, observation_count, attestation_count, required_attestations) \
             VALUES ($1, 1, 2, 3, 4, 5, 6, 7)",
        )
        .bind(&pubkey)
        .execute(&pool)
        .await
        .expect("insert test claim");

        // Same expression the audit endpoint uses (migration 0057 trigger).
        let (stored, recomputed): (Option<String>, Option<String>) = sqlx::query_as(
            "SELECT mirror_hash, encode(digest( \
                 coalesce(pubkey, '') || '|' || \
                 coalesce(claim_id, '') || '|' || \
                 coalesce(parcel_pubkey, '') || '|' || \
                 claim_type::text || '|' || \
                 coalesce(submitted_by, '') || '|' || \
                 status::text || '|' || \
                 coalesce(statement_hash, '') || '|' || \
                 version::text || '|' || \
                 evidence_count::text || '|' || \
                 observation_count::text || '|' || \
                 attestation_count::text || '|' || \
                 required_attestations::text || '|' || \
                 coalesce(to_char(chain_created_at, 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'), '') || '|' || \
                 coalesce(to_char(chain_updated_at, 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"'), ''), 'sha256'), 'hex') \
             FROM claims_onchain WHERE pubkey = $1",
        )
        .bind(&pubkey)
        .fetch_one(&pool)
        .await
        .expect("recompute claim hash");
        assert!(stored.is_some(), "trigger must compute claims mirror_hash");
        assert_eq!(stored, recomputed, "claims audit hash must match trigger");

        sqlx::query("DELETE FROM claims_onchain WHERE pubkey = $1")
            .bind(&pubkey)
            .execute(&pool)
            .await
            .expect("delete test claim");
        pool.close().await;
    }

    #[tokio::test]
    async fn verification_attestation_defaults_are_honest() {
        let Ok(database_url) = std::env::var("DATABASE_URL") else {
            eprintln!("skipping mirror audit test: DATABASE_URL is not set");
            return;
        };
        let pool = sqlx::PgPool::connect(&database_url)
            .await
            .expect("connect test database");

        // `column_default` is SQL NULL for an explicit `DEFAULT NULL`, so
        // keep it nullable here: both NULL and the literal "NULL" are honest.
        let defaults: Vec<(String, Option<String>)> = sqlx::query_as(
            "SELECT column_name, column_default FROM information_schema.columns \
             WHERE table_name = 'verification_attestations_onchain' \
               AND column_name IN ('chain_verified', 'chain_slot', 'chain_signature')",
        )
        .fetch_all(&pool)
        .await
        .expect("query attestation defaults");
        assert_eq!(defaults.len(), 3, "migration 0058 must be applied");
        let verified_default = defaults
            .iter()
            .find(|(name, _)| name == "chain_verified")
            .and_then(|(_, value)| value.as_deref())
            .expect("chain_verified default");
        assert!(
            verified_default.contains("false"),
            "ADR-026: new attestations must start unverified (got {verified_default})"
        );
        for column in ["chain_slot", "chain_signature"] {
            let value = defaults
                .iter()
                .find(|(name, _)| name == column)
                .and_then(|(_, value)| value.as_deref());
            assert!(
                value.is_none() || value == Some("NULL"),
                "{column} must default to NULL, not a fake value (got {value:?})"
            );
        }
        pool.close().await;
    }

    #[tokio::test]
    async fn mirror_audit_endpoint_shape() {
        let Ok(database_url) = std::env::var("DATABASE_URL") else {
            eprintln!("skipping mirror audit endpoint test: DATABASE_URL is not set");
            return;
        };
        let pool = sqlx::PgPool::connect(&database_url)
            .await
            .expect("connect test database");

        // Exercise the audit SQL directly (same queries the endpoint uses).
        // Count both in one query to avoid races with concurrent tests.
        let (total, hashed): (i64, i64) = sqlx::query_as(
            "SELECT count(*), count(*) FILTER (WHERE mirror_hash IS NOT NULL) FROM parcels",
        )
        .fetch_one(&pool)
        .await
        .expect("count parcels");
        assert!(hashed <= total, "hashed rows cannot exceed total rows");
        let _ = pool;
    }
}
