//! RFC-015 — Spatial Component Package API (REQ-026).
//!
//! Endpoints:
//!   POST   /components          submit + conformance-validate a package
//!   GET    /components/{id}     retrieve a package by id
//!   GET    /components          list packages (filter by physical_entity_id)

use axum::extract::{Path, Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::error::AppError;
use crate::state::AppState;

use terra_geo::component_package::{validate, ComponentPackage};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", post(submit_component).get(list_components))
        .route("/{id}", get(get_component))
}

// ---------------------------------------------------------------------------
// POST /api/v1/components
//
// Submit a Spatial Component Package. The package is conformance-validated
// (RFC-015 §4) before insert; invalid packages are rejected with 400.
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ComponentRow {
    pub id: Uuid,
    pub component_id: String,
    pub physical_entity_id: String,
    pub component_type: String,
    pub validation_state: String,
    pub geometry_digest: String,
    pub digest_algorithm_version: String,
    pub crs: String,
    pub elevation_ref: String,
    pub vertical_units: String,
    pub bounding_volume: serde_json::Value,
    pub spatial_extent: serde_json::Value,
    pub resolution: f64,
    pub lod: i32,
    pub reconstruction_engine_id: String,
    pub reconstruction_engine_version: String,
    pub creation_timestamp: chrono::DateTime<chrono::Utc>,
    pub provenance_classification: String,
    pub privacy_classification: String,
    pub access_policy: serde_json::Value,
    pub version_lineage: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct SubmitComponent {
    #[serde(flatten)]
    pub package: ComponentPackage,
}

pub async fn submit_component(
    State(state): State<AppState>,
    Json(input): Json<SubmitComponent>,
) -> Result<(axum::http::StatusCode, Json<ComponentRow>), AppError> {
    let pkg = &input.package;

    // RFC-015 §4 conformance (rules 2–8). Rule 1 (digest integrity) is
    // checked when the geometry artifact is later resolved via GET.
    validate(pkg).map_err(|e| AppError::bad_request(e.to_string()))?;

    let row = sqlx::query_as::<_, ComponentRow>(
        r#"
        INSERT INTO component_packages (
            component_id, physical_entity_id, parent_id, component_type,
            geometry_artifact, format, format_version, geometry_digest,
            digest_algorithm_version, crs, elevation_ref, vertical_units,
            local_frame, global_transform, bounding_volume, spatial_extent,
            resolution, lod, source_evidence_refs,
            reconstruction_engine_id, reconstruction_engine_version,
            processing_configuration, creation_timestamp,
            resource_usage_receipt, accuracy, uncertainty, coverage,
            confidence, provenance_classification, dependencies, overlap_ids,
            privacy_classification, access_policy, validation_state,
            version_lineage
        )
        VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,
                $18,$19,$20,$21,$22,$23,$24,$25,$26,$27,$28,$29,$30,$31,$32,
                $33,$34,$35,$36)
        RETURNING id, component_id, physical_entity_id, component_type,
                  validation_state, geometry_digest, digest_algorithm_version,
                  crs, elevation_ref, vertical_units, bounding_volume,
                  spatial_extent, resolution, lod,
                  reconstruction_engine_id, reconstruction_engine_version,
                  creation_timestamp, provenance_classification,
                  privacy_classification, access_policy, version_lineage,
                  created_at
        "#,
    )
    .bind(&pkg.component_id)
    .bind(&pkg.physical_entity_id)
    .bind(&pkg.parent_id)
    .bind(&pkg.component_type)
    .bind(&pkg.geometry_artifact)
    .bind(&pkg.format)
    .bind(&pkg.format_version)
    .bind(&pkg.geometry_digest)
    .bind(&pkg.digest_algorithm_version)
    .bind(&pkg.crs)
    .bind(&pkg.elevation_ref)
    .bind(&pkg.vertical_units)
    .bind(&serde_json::to_value(&pkg.local_frame).unwrap_or(Value::Null))
    .bind(&serde_json::to_value(&pkg.global_transform).unwrap_or(Value::Null))
    .bind(&serde_json::to_value(&pkg.bounding_volume).unwrap_or(Value::Null))
    .bind(&serde_json::to_value(&pkg.spatial_extent).unwrap_or(Value::Null))
    .bind(pkg.resolution)
    .bind(pkg.lod)
    .bind(&pkg.source_evidence_refs)
    .bind(&pkg.reconstruction_engine_id)
    .bind(&pkg.reconstruction_engine_version)
    .bind(&serde_json::to_value(&pkg.processing_configuration).unwrap_or(Value::Null))
    .bind(&pkg.creation_timestamp)
    .bind(&serde_json::to_value(&pkg.resource_usage_receipt).unwrap_or(Value::Null))
    .bind(pkg.accuracy)
    .bind(pkg.uncertainty)
    .bind(&serde_json::to_value(&pkg.coverage).unwrap_or(Value::Null))
    .bind(pkg.confidence)
    .bind(&pkg.provenance_classification)
    .bind(&pkg.dependencies)
    .bind(&pkg.overlap_ids)
    .bind(&pkg.privacy_classification)
    .bind(&serde_json::to_value(&pkg.access_policy).unwrap_or(Value::Null))
    .bind(&pkg.validation_state)
    .bind(&serde_json::to_value(&pkg.version_lineage).unwrap_or(Value::Null))
    .fetch_one(&state.pool)
    .await
    .map_err(|e| AppError::internal(format!("insert failed: {e}")))?;

    Ok((axum::http::StatusCode::CREATED, Json(row)))
}

// ---------------------------------------------------------------------------
// GET /api/v1/components/{id}
// ---------------------------------------------------------------------------

const SELECT_COMPONENT: &str = "SELECT id, component_id, physical_entity_id, \
    component_type, validation_state, geometry_digest, \
    digest_algorithm_version, crs, elevation_ref, vertical_units, \
    bounding_volume, spatial_extent, resolution, lod, \
    reconstruction_engine_id, reconstruction_engine_version, \
    creation_timestamp, provenance_classification, privacy_classification, \
    access_policy, version_lineage, created_at \
    FROM component_packages WHERE id = $1";

pub async fn get_component(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ComponentRow>, AppError> {
    let row = sqlx::query_as::<_, ComponentRow>(SELECT_COMPONENT)
        .bind(id)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| AppError::internal(format!("query failed: {e}")))?
        .ok_or_else(|| AppError::not_found(format!("component {id} not found")))?;

    Ok(Json(row))
}

// ---------------------------------------------------------------------------
// GET /api/v1/components?physical_entity_id=...
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct ListComponentsParams {
    pub physical_entity_id: Option<String>,
    #[serde(default = "default_limit")]
    pub limit: usize,
}

fn default_limit() -> usize {
    50
}

const LIST_COMPONENTS: &str = "SELECT id, component_id, physical_entity_id, \
    component_type, validation_state, geometry_digest, \
    digest_algorithm_version, crs, elevation_ref, vertical_units, \
    bounding_volume, spatial_extent, resolution, lod, \
    reconstruction_engine_id, reconstruction_engine_version, \
    creation_timestamp, provenance_classification, privacy_classification, \
    access_policy, version_lineage, created_at \
    FROM component_packages";

pub async fn list_components(
    State(state): State<AppState>,
    Query(params): Query<ListComponentsParams>,
) -> Result<Json<Vec<ComponentRow>>, AppError> {
    let limit = params.limit.min(500);
    let rows: Vec<ComponentRow> = match &params.physical_entity_id {
        Some(peid) => sqlx::query_as::<_, ComponentRow>(
            format!("{LIST_COMPONENTS} WHERE physical_entity_id = $1 ORDER BY created_at DESC LIMIT $2").as_str(),
        )
        .bind(peid)
        .bind(limit as i64)
        .fetch_all(&state.pool)
        .await
        .map_err(|e| AppError::internal(format!("query failed: {e}")))?,
        None => sqlx::query_as::<_, ComponentRow>(
            format!("{LIST_COMPONENTS} ORDER BY created_at DESC LIMIT $1").as_str(),
        )
        .bind(limit as i64)
        .fetch_all(&state.pool)
        .await
        .map_err(|e| AppError::internal(format!("query failed: {e}")))?,
    };

    Ok(Json(rows))
}
