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
// Subdivision records
// ---------------------------------------------------------------------------

const SUBDIV_SELECT: &str = r#"
    SELECT
        id, original_parcel_id, sub_parcel_id,
        original_geometry_hash, new_geometry_hash,
        surveyor_attestation_id, rights_migrated,
        attestations_migrated, initiated_by,
        created_at, completed_at, status
    FROM subdivision_records
"#;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct SubdivisionRecord {
    pub id: Uuid,
    pub original_parcel_id: Uuid,
    pub sub_parcel_id: Uuid,
    pub original_geometry_hash: String,
    pub new_geometry_hash: String,
    pub surveyor_attestation_id: Option<Uuid>,
    pub rights_migrated: bool,
    pub attestations_migrated: bool,
    pub initiated_by: String,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateSubdivisionRequest {
    pub original_parcel_id: Uuid,
    pub sub_parcel_id: Uuid,
    pub original_geometry_hash: String,
    pub new_geometry_hash: String,
    pub surveyor_attestation_id: Option<Uuid>,
    pub initiated_by: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSubdivisionRequest {
    pub rights_migrated: Option<bool>,
    pub attestations_migrated: Option<bool>,
    pub status: Option<String>,
}

// ---------------------------------------------------------------------------
// Amalgamation records
// ---------------------------------------------------------------------------

const AMALG_SELECT: &str = r#"
    SELECT
        id, result_parcel_id, source_parcel_id,
        source_geometry_hash, result_geometry_hash,
        rights_merged, initiated_by,
        created_at, completed_at, status
    FROM amalgamation_records
"#;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct AmalgamationRecord {
    pub id: Uuid,
    pub result_parcel_id: Uuid,
    pub source_parcel_id: Uuid,
    pub source_geometry_hash: String,
    pub result_geometry_hash: String,
    pub rights_merged: bool,
    pub initiated_by: String,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateAmalgamationRequest {
    pub result_parcel_id: Uuid,
    pub source_parcel_id: Uuid,
    pub source_geometry_hash: String,
    pub result_geometry_hash: String,
    pub initiated_by: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAmalgamationRequest {
    pub rights_merged: Option<bool>,
    pub status: Option<String>,
}

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/subdivisions", get(list_subdivisions).post(create_subdivision))
        .route("/subdivisions/{id}", get(get_subdivision).patch(update_subdivision))
        .route("/amalgamations", get(list_amalgamations).post(create_amalgamation))
        .route("/amalgamations/{id}", get(get_amalgamation).patch(update_amalgamation))
}

// ---------------------------------------------------------------------------
// Handlers — Subdivisions
// ---------------------------------------------------------------------------

async fn list_subdivisions(
    State(state): State<AppState>,
) -> Result<Json<Vec<SubdivisionRecord>>, AppError> {
    let rows: Vec<SubdivisionRecord> = sqlx::query_as(SUBDIV_SELECT)
        .fetch_all(&state.pool)
        .await?;
    Ok(Json(rows))
}

async fn create_subdivision(
    State(state): State<AppState>,
    Json(req): Json<CreateSubdivisionRequest>,
) -> Result<(StatusCode, Json<SubdivisionRecord>), AppError> {
    if req.original_geometry_hash.is_empty() {
        return Err(AppError::bad_request("original_geometry_hash is required"));
    }
    if req.new_geometry_hash.is_empty() {
        return Err(AppError::bad_request("new_geometry_hash is required"));
    }
    if req.initiated_by.is_empty() {
        return Err(AppError::bad_request("initiated_by is required"));
    }

    let row: SubdivisionRecord = sqlx::query_as(
        r#"INSERT INTO subdivision_records (
            original_parcel_id, sub_parcel_id,
            original_geometry_hash, new_geometry_hash,
            surveyor_attestation_id, initiated_by
        ) VALUES ($1,$2,$3,$4,$5,$6)
        RETURNING id, original_parcel_id, sub_parcel_id,
            original_geometry_hash, new_geometry_hash,
            surveyor_attestation_id, rights_migrated,
            attestations_migrated, initiated_by,
            created_at, completed_at, status"#,
    )
    .bind(req.original_parcel_id)
    .bind(req.sub_parcel_id)
    .bind(&req.original_geometry_hash)
    .bind(&req.new_geometry_hash)
    .bind(req.surveyor_attestation_id)
    .bind(&req.initiated_by)
    .fetch_one(&state.pool)
    .await?;
    Ok((StatusCode::CREATED, Json(row)))
}

async fn get_subdivision(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<SubdivisionRecord>, AppError> {
    let row: SubdivisionRecord = sqlx::query_as(&format!("{SUBDIV_SELECT} WHERE id = $1"))
        .bind(id)
        .fetch_optional(&state.pool)
        .await?
        .ok_or_else(|| AppError::not_found("subdivision record"))?;
    Ok(Json(row))
}

async fn update_subdivision(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateSubdivisionRequest>,
) -> Result<Json<SubdivisionRecord>, AppError> {
    let existing: SubdivisionRecord =
        sqlx::query_as(&format!("{SUBDIV_SELECT} WHERE id = $1"))
            .bind(id)
            .fetch_optional(&state.pool)
            .await?
            .ok_or_else(|| AppError::not_found("subdivision record"))?;

    let rm = req.rights_migrated.unwrap_or(existing.rights_migrated);
    let am = req.attestations_migrated.unwrap_or(existing.attestations_migrated);
    let st = req.status.unwrap_or(existing.status);

    let row: SubdivisionRecord = sqlx::query_as(
        r#"UPDATE subdivision_records SET
            rights_migrated = $1, attestations_migrated = $2,
            status = $3,
            completed_at = CASE WHEN $3 = 'completed' THEN now() ELSE completed_at END
        WHERE id = $4
        RETURNING id, original_parcel_id, sub_parcel_id,
            original_geometry_hash, new_geometry_hash,
            surveyor_attestation_id, rights_migrated,
            attestations_migrated, initiated_by,
            created_at, completed_at, status"#,
    )
    .bind(rm)
    .bind(am)
    .bind(&st)
    .bind(id)
    .fetch_one(&state.pool)
    .await?;
    Ok(Json(row))
}

// ---------------------------------------------------------------------------
// Handlers — Amalgamations
// ---------------------------------------------------------------------------

async fn list_amalgamations(
    State(state): State<AppState>,
) -> Result<Json<Vec<AmalgamationRecord>>, AppError> {
    let rows: Vec<AmalgamationRecord> = sqlx::query_as(AMALG_SELECT)
        .fetch_all(&state.pool)
        .await?;
    Ok(Json(rows))
}

async fn create_amalgamation(
    State(state): State<AppState>,
    Json(req): Json<CreateAmalgamationRequest>,
) -> Result<(StatusCode, Json<AmalgamationRecord>), AppError> {
    if req.source_geometry_hash.is_empty() {
        return Err(AppError::bad_request("source_geometry_hash is required"));
    }
    if req.result_geometry_hash.is_empty() {
        return Err(AppError::bad_request("result_geometry_hash is required"));
    }
    if req.initiated_by.is_empty() {
        return Err(AppError::bad_request("initiated_by is required"));
    }

    let row: AmalgamationRecord = sqlx::query_as(
        r#"INSERT INTO amalgamation_records (
            result_parcel_id, source_parcel_id,
            source_geometry_hash, result_geometry_hash,
            initiated_by
        ) VALUES ($1,$2,$3,$4,$5)
        RETURNING id, result_parcel_id, source_parcel_id,
            source_geometry_hash, result_geometry_hash,
            rights_merged, initiated_by,
            created_at, completed_at, status"#,
    )
    .bind(req.result_parcel_id)
    .bind(req.source_parcel_id)
    .bind(&req.source_geometry_hash)
    .bind(&req.result_geometry_hash)
    .bind(&req.initiated_by)
    .fetch_one(&state.pool)
    .await?;
    Ok((StatusCode::CREATED, Json(row)))
}

async fn get_amalgamation(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<AmalgamationRecord>, AppError> {
    let row: AmalgamationRecord = sqlx::query_as(&format!("{AMALG_SELECT} WHERE id = $1"))
        .bind(id)
        .fetch_optional(&state.pool)
        .await?
        .ok_or_else(|| AppError::not_found("amalgamation record"))?;
    Ok(Json(row))
}

async fn update_amalgamation(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateAmalgamationRequest>,
) -> Result<Json<AmalgamationRecord>, AppError> {
    let existing: AmalgamationRecord =
        sqlx::query_as(&format!("{AMALG_SELECT} WHERE id = $1"))
            .bind(id)
            .fetch_optional(&state.pool)
            .await?
            .ok_or_else(|| AppError::not_found("amalgamation record"))?;

    let rm = req.rights_merged.unwrap_or(existing.rights_merged);
    let st = req.status.unwrap_or(existing.status);

    let row: AmalgamationRecord = sqlx::query_as(
        r#"UPDATE amalgamation_records SET
            rights_merged = $1, status = $2,
            completed_at = CASE WHEN $2 = 'completed' THEN now() ELSE completed_at END
        WHERE id = $3
        RETURNING id, result_parcel_id, source_parcel_id,
            source_geometry_hash, result_geometry_hash,
            rights_merged, initiated_by,
            created_at, completed_at, status"#,
    )
    .bind(rm)
    .bind(&st)
    .bind(id)
    .fetch_one(&state.pool)
    .await?;
    Ok(Json(row))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_subdivision_request_defaults() {
        let req = CreateSubdivisionRequest {
            original_parcel_id: Uuid::new_v4(),
            sub_parcel_id: Uuid::new_v4(),
            original_geometry_hash: "abc".into(),
            new_geometry_hash: "def".into(),
            surveyor_attestation_id: None,
            initiated_by: "wallet1".into(),
        };
        assert!(req.surveyor_attestation_id.is_none());
    }

    #[test]
    fn create_amalgamation_request_defaults() {
        let req = CreateAmalgamationRequest {
            result_parcel_id: Uuid::new_v4(),
            source_parcel_id: Uuid::new_v4(),
            source_geometry_hash: "abc".into(),
            result_geometry_hash: "def".into(),
            initiated_by: "wallet1".into(),
        };
        assert!(!req.initiated_by.is_empty());
    }
}
