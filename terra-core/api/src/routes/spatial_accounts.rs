use axum::extract::{Path, Query, State};
use axum::routing::get;
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::state::AppState;

// ---------------------------------------------------------------------------
// Spatial account mirror read API (RFC-013 Phase E).
//
// Serves the on-chain SpatialAsset / GeometryVersion / ThreeDModel /
// SpatialSnapshot mirrors (migration 0030) to the globe client (Phase D):
// version history with provenance, registered 3D models, and point-in-time
// snapshot reads (find newest snapshot at T, list versions below its copied
// cursor). The indexer owns the tables; this layer is read-only.
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct ListParams {
    pub parcel: Option<String>,
    #[serde(default = "default_limit")]
    pub limit: i64,
}

#[derive(Debug, Deserialize)]
pub struct HistoryParams {
    /// Unix seconds: read the newest snapshot with `chain_created_at <= at`.
    pub at: i64,
}

fn default_limit() -> i64 {
    50
}

/// Mirror keys are base58 pubkeys / hex hashes; reject obvious junk early.
pub fn validate_pubkey(s: &str) -> Result<(), AppError> {
    let ok = !s.is_empty() && s.len() <= 64 && s.chars().all(|c| c.is_ascii_alphanumeric());
    if ok {
        Ok(())
    } else {
        Err(AppError::bad_request("invalid pubkey"))
    }
}

pub fn clamp_limit(limit: i64) -> i64 {
    limit.clamp(1, 200)
}

pub fn validate_unix_at(at: i64) -> Result<(), AppError> {
    // 2000-01-01 .. 2100-01-01, loose sanity bounds.
    if (946_684_800..=4_102_444_800).contains(&at) {
        Ok(())
    } else {
        Err(AppError::bad_request(
            "at must be a unix timestamp between 2000 and 2100",
        ))
    }
}

// ---------------------------------------------------------------------------
// Rows
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct AssetRow {
    pub pubkey: String,
    pub parcel_pubkey: String,
    pub authority: String,
    pub dimensionality: i16,
    pub elevation_min_mm: i32,
    pub elevation_max_mm: i32,
    pub geometry_version_count: i32,
    pub latest_geometry_hash: String,
    pub chain_created_at: Option<DateTime<Utc>>,
    pub chain_updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct VersionRow {
    pub pubkey: String,
    pub asset_pubkey: String,
    pub parcel_pubkey: String,
    pub version: i32,
    pub geometry_hash: String,
    pub source: i16,
    pub dimension: i16,
    pub storage_reference: String,
    pub submitted_by: String,
    pub chain_submitted_at: Option<DateTime<Utc>>,
    pub verified: bool,
    pub verified_by: Option<String>,
    pub chain_verified_at: Option<DateTime<Utc>>,
    pub evidence_manifest: Option<String>,
    pub elevation_min_mm: i32,
    pub elevation_max_mm: i32,
    pub elevation_source: i16,
    pub required: i16,
    pub attest_count: i16,
    pub attestors: Vec<String>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ModelRow {
    pub pubkey: String,
    pub asset_pubkey: String,
    pub parcel_pubkey: String,
    pub model_version: i32,
    pub geometry_version: i32,
    pub model_hash: String,
    pub storage_reference: String,
    pub model_format: i16,
    pub submitted_by: String,
    pub chain_submitted_at: Option<DateTime<Utc>>,
    pub verified: bool,
    pub verified_by: Option<String>,
    pub chain_verified_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct SnapshotRow {
    pub pubkey: String,
    pub asset_pubkey: String,
    pub parcel_pubkey: String,
    pub snapshot_index: i32,
    pub geometry_version_count: i32,
    pub latest_geometry_hash: String,
    pub dimensionality: i16,
    pub elevation_min_mm: i32,
    pub elevation_max_mm: i32,
    pub label: String,
    pub created_by: String,
    pub chain_created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
pub struct PointInTimeHistory {
    pub snapshot: SnapshotRow,
    pub versions: Vec<VersionRow>,
}

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/assets", get(list_assets))
        .route("/assets/{pubkey}", get(get_asset))
        .route("/assets/{pubkey}/versions", get(list_versions))
        .route("/assets/{pubkey}/models", get(list_models))
        .route("/assets/{pubkey}/snapshots", get(list_snapshots))
        .route("/assets/{pubkey}/history", get(point_in_time_history))
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// Mirrored spatial assets, optionally filtered by parcel PDA.
async fn list_assets(
    State(state): State<AppState>,
    Query(params): Query<ListParams>,
) -> Result<Json<Vec<AssetRow>>, AppError> {
    let limit = clamp_limit(params.limit);
    if let Some(parcel) = params.parcel.as_deref() {
        validate_pubkey(parcel)?;
        let rows: Vec<AssetRow> = sqlx::query_as(
            "SELECT pubkey, parcel_pubkey, authority, dimensionality,
                    elevation_min_mm, elevation_max_mm, geometry_version_count,
                    latest_geometry_hash, chain_created_at, chain_updated_at
             FROM spatial_assets_onchain
             WHERE parcel_pubkey = $1
             ORDER BY pubkey
             LIMIT $2",
        )
        .bind(parcel)
        .bind(limit)
        .fetch_all(&state.pool)
        .await?;
        return Ok(Json(rows));
    }
    let rows: Vec<AssetRow> = sqlx::query_as(
        "SELECT pubkey, parcel_pubkey, authority, dimensionality,
                elevation_min_mm, elevation_max_mm, geometry_version_count,
                latest_geometry_hash, chain_created_at, chain_updated_at
         FROM spatial_assets_onchain
         ORDER BY pubkey
         LIMIT $1",
    )
    .bind(limit)
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(rows))
}

/// One mirrored spatial asset by PDA pubkey.
async fn get_asset(
    State(state): State<AppState>,
    Path(pubkey): Path<String>,
) -> Result<Json<AssetRow>, AppError> {
    validate_pubkey(&pubkey)?;
    let row: AssetRow = sqlx::query_as(
        "SELECT pubkey, parcel_pubkey, authority, dimensionality,
                elevation_min_mm, elevation_max_mm, geometry_version_count,
                latest_geometry_hash, chain_created_at, chain_updated_at
         FROM spatial_assets_onchain WHERE pubkey = $1",
    )
    .bind(&pubkey)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::not_found("spatial asset not found"))?;
    Ok(Json(row))
}

/// Full geometry-version history for an asset (append order).
async fn list_versions(
    State(state): State<AppState>,
    Path(pubkey): Path<String>,
) -> Result<Json<Vec<VersionRow>>, AppError> {
    validate_pubkey(&pubkey)?;
    let rows: Vec<VersionRow> = sqlx::query_as(
        "SELECT pubkey, asset_pubkey, parcel_pubkey, version, geometry_hash,
                source, dimension, storage_reference, submitted_by,
                chain_submitted_at, verified, verified_by, chain_verified_at,
                evidence_manifest, elevation_min_mm, elevation_max_mm,
                elevation_source, required, attest_count, attestors
         FROM geometry_versions_onchain
         WHERE asset_pubkey = $1
         ORDER BY version ASC",
    )
    .bind(&pubkey)
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(rows))
}

/// Registered 3D models for an asset (append order).
async fn list_models(
    State(state): State<AppState>,
    Path(pubkey): Path<String>,
) -> Result<Json<Vec<ModelRow>>, AppError> {
    validate_pubkey(&pubkey)?;
    let rows: Vec<ModelRow> = sqlx::query_as(
        "SELECT pubkey, asset_pubkey, parcel_pubkey, model_version,
                geometry_version, model_hash, storage_reference, model_format,
                submitted_by, chain_submitted_at, verified, verified_by,
                chain_verified_at
         FROM three_d_models_onchain
         WHERE asset_pubkey = $1
         ORDER BY model_version ASC",
    )
    .bind(&pubkey)
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(rows))
}

/// Point-in-time snapshots for an asset (append order).
async fn list_snapshots(
    State(state): State<AppState>,
    Path(pubkey): Path<String>,
) -> Result<Json<Vec<SnapshotRow>>, AppError> {
    validate_pubkey(&pubkey)?;
    let rows: Vec<SnapshotRow> = sqlx::query_as(
        "SELECT pubkey, asset_pubkey, parcel_pubkey, snapshot_index,
                geometry_version_count, latest_geometry_hash, dimensionality,
                elevation_min_mm, elevation_max_mm, label, created_by,
                chain_created_at
         FROM spatial_snapshots_onchain
         WHERE asset_pubkey = $1
         ORDER BY snapshot_index ASC",
    )
    .bind(&pubkey)
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(rows))
}

/// "What did this asset look like at time T?" — newest snapshot with
/// `chain_created_at <= T`, then every version below its copied cursor
/// (RFC-013 §8.5: snapshots copy the index; appends never rewrite history).
async fn point_in_time_history(
    State(state): State<AppState>,
    Path(pubkey): Path<String>,
    Query(params): Query<HistoryParams>,
) -> Result<Json<PointInTimeHistory>, AppError> {
    validate_pubkey(&pubkey)?;
    validate_unix_at(params.at)?;

    let snapshot: SnapshotRow = sqlx::query_as(
        "SELECT pubkey, asset_pubkey, parcel_pubkey, snapshot_index,
                geometry_version_count, latest_geometry_hash, dimensionality,
                elevation_min_mm, elevation_max_mm, label, created_by,
                chain_created_at
         FROM spatial_snapshots_onchain
         WHERE asset_pubkey = $1
           AND chain_created_at IS NOT NULL
           AND chain_created_at <= to_timestamp($2)
         ORDER BY chain_created_at DESC, snapshot_index DESC
         LIMIT 1",
    )
    .bind(&pubkey)
    .bind(params.at)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::not_found("no snapshot at or before the requested time"))?;

    let versions: Vec<VersionRow> = sqlx::query_as(
        "SELECT pubkey, asset_pubkey, parcel_pubkey, version, geometry_hash,
                source, dimension, storage_reference, submitted_by,
                chain_submitted_at, verified, verified_by, chain_verified_at,
                evidence_manifest, elevation_min_mm, elevation_max_mm,
                elevation_source, required, attest_count, attestors
         FROM geometry_versions_onchain
         WHERE asset_pubkey = $1
           AND version < $2
         ORDER BY version ASC",
    )
    .bind(&pubkey)
    .bind(snapshot.geometry_version_count)
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(PointInTimeHistory { snapshot, versions }))
}

// ---------------------------------------------------------------------------
// Tests (pure validation logic; DB-backed queries are covered by CI PostGIS)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pubkey_validation() {
        assert!(validate_pubkey("GaEDbktvpZ3qiqp4PmFgHwDSa6JsFfVjXFqNb2nTbage").is_ok());
        assert!(validate_pubkey("").is_err());
        assert!(validate_pubkey("bad key!").is_err());
        assert!(validate_pubkey(&"a".repeat(65)).is_err());
    }

    #[test]
    fn limit_clamped_1_to_200() {
        assert_eq!(clamp_limit(0), 1);
        assert_eq!(clamp_limit(50), 50);
        assert_eq!(clamp_limit(500), 200);
    }

    #[test]
    fn at_timestamp_sanity_bounds() {
        assert!(validate_unix_at(1_000_000_000).is_ok()); // 2001-09-09
        assert!(validate_unix_at(2_000_000_000).is_ok()); // 2033-05-18
        assert!(validate_unix_at(0).is_err());
        assert!(validate_unix_at(946_684_799).is_err());
        assert!(validate_unix_at(4_102_444_801).is_err());
    }
}
