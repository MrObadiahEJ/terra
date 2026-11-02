//! Chapter 12 — World Representation Manifest API (REQ-027).
//!
//! Endpoints:
//!   POST   /manifests                          create a manifest version
//!   GET    /manifests/{id}                     get a manifest with its objects
//!   GET    /manifests                          list manifests (filter by region)
//!   PUT    /manifests/{id}/objects             update/replace objects in a manifest
//!
//! The manifest stores references and relationships, not geometry.

use axum::extract::{Path, Query, State};
use axum::routing::{get, post, put};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::AppError;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", post(create_manifest).get(list_manifests))
        .route("/{id}", get(get_manifest))
        .route("/{id}/objects", put(update_manifest_objects))
}

// ---------------------------------------------------------------------------
// Request / response types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct CreateManifestRequest {
    pub region_id: String,
    pub region_name: String,
    pub parent_manifest_id: Option<Uuid>,
    pub neighbor_manifest_ids: Option<Vec<Uuid>>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ManifestObjectInput {
    pub object_id: String,
    pub component_version_id: String,
    pub assembly_version_id: Option<String>,
    pub spatial_extent: serde_json::Value,
    pub storage_reference: String,
    pub available_lods: Option<Vec<i32>>,
    pub validation_state: Option<String>,
    pub access_requirements: Option<serde_json::Value>,
    pub update_history: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateManifestObjectsRequest {
    pub objects: Vec<ManifestObjectInput>,
}

#[derive(Debug, Deserialize)]
pub struct ListManifestsParams {
    pub region_id: Option<String>,
    #[serde(default = "default_limit")]
    pub limit: usize,
}

fn default_limit() -> usize {
    50
}

// ---------------------------------------------------------------------------
// Row types (sqlx::FromRow + Serialize)
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ManifestRow {
    pub id: Uuid,
    pub region_id: String,
    pub region_name: String,
    pub version: i32,
    pub parent_manifest_id: Option<Uuid>,
    pub neighbor_manifest_ids: Vec<Uuid>,
    pub description: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ManifestObjectRow {
    pub id: Uuid,
    pub manifest_id: Uuid,
    pub object_id: String,
    pub component_version_id: String,
    pub assembly_version_id: Option<String>,
    pub spatial_extent: serde_json::Value,
    pub storage_reference: String,
    pub available_lods: Vec<i32>,
    pub validation_state: String,
    pub access_requirements: serde_json::Value,
    pub update_history: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

// ---------------------------------------------------------------------------
// POST /manifests
// ---------------------------------------------------------------------------

pub async fn create_manifest(
    State(state): State<AppState>,
    Json(req): Json<CreateManifestRequest>,
) -> Result<(axum::http::StatusCode, Json<ManifestRow>), AppError> {
    // Determine next version for the region.
    let next_version: i32 = sqlx::query_scalar(
        "SELECT COALESCE(MAX(version), 0) + 1 FROM world_manifests WHERE region_id = $1",
    )
    .bind(&req.region_id)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| AppError::internal(format!("version query failed: {e}")))?;

    let neighbor_ids = req.neighbor_manifest_ids.unwrap_or_default();

    let row = sqlx::query_as::<_, ManifestRow>(
        r#"
        INSERT INTO world_manifests (
            region_id, region_name, version, parent_manifest_id,
            neighbor_manifest_ids, description
        )
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id, region_id, region_name, version, parent_manifest_id,
                  neighbor_manifest_ids, description, created_at, updated_at
        "#,
    )
    .bind(&req.region_id)
    .bind(&req.region_name)
    .bind(next_version)
    .bind(&req.parent_manifest_id)
    .bind(&neighbor_ids)
    .bind(&req.description)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| AppError::internal(format!("insert failed: {e}")))?;

    Ok((axum::http::StatusCode::CREATED, Json(row)))
}

// ---------------------------------------------------------------------------
// GET /manifests/{id}
// ---------------------------------------------------------------------------

const MANIFEST_SELECT: &str = "SELECT id, region_id, region_name, version, \
    parent_manifest_id, neighbor_manifest_ids, description, \
    created_at, updated_at FROM world_manifests WHERE id = $1";

const OBJECTS_SELECT: &str = "SELECT id, manifest_id, object_id, \
    component_version_id, assembly_version_id, spatial_extent, \
    storage_reference, available_lods, validation_state, \
    access_requirements, update_history, created_at \
    FROM manifest_objects WHERE manifest_id = $1 ORDER BY object_id";

pub async fn get_manifest(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ManifestWithObjects>, AppError> {
    let manifest = sqlx::query_as::<_, ManifestRow>(MANIFEST_SELECT)
        .bind(id)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| AppError::internal(format!("query failed: {e}")))?
        .ok_or_else(|| AppError::not_found(format!("manifest {id} not found")))?;

    let objects = sqlx::query_as::<_, ManifestObjectRow>(OBJECTS_SELECT)
        .bind(id)
        .fetch_all(&state.pool)
        .await
        .map_err(|e| AppError::internal(format!("query failed: {e}")))?;

    Ok(Json(ManifestWithObjects {
        manifest,
        objects,
    }))
}

#[derive(Debug, Serialize)]
pub struct ManifestWithObjects {
    pub manifest: ManifestRow,
    pub objects: Vec<ManifestObjectRow>,
}

// ---------------------------------------------------------------------------
// GET /manifests?region_id=...
// ---------------------------------------------------------------------------

const LIST_MANIFESTS: &str = "SELECT id, region_id, region_name, version, \
    parent_manifest_id, neighbor_manifest_ids, description, \
    created_at, updated_at FROM world_manifests";

pub async fn list_manifests(
    State(state): State<AppState>,
    Query(params): Query<ListManifestsParams>,
) -> Result<Json<Vec<ManifestRow>>, AppError> {
    let limit = params.limit.min(500);
    let rows: Vec<ManifestRow> = match &params.region_id {
        Some(rid) => sqlx::query_as::<_, ManifestRow>(
            format!("{LIST_MANIFESTS} WHERE region_id = $1 ORDER BY version DESC LIMIT $2").as_str(),
        )
        .bind(rid)
        .bind(limit as i64)
        .fetch_all(&state.pool)
        .await
        .map_err(|e| AppError::internal(format!("query failed: {e}")))?,
        None => sqlx::query_as::<_, ManifestRow>(
            format!("{LIST_MANIFESTS} ORDER BY region_id, version DESC LIMIT $1").as_str(),
        )
        .bind(limit as i64)
        .fetch_all(&state.pool)
        .await
        .map_err(|e| AppError::internal(format!("query failed: {e}")))?,
    };

    Ok(Json(rows))
}

// ---------------------------------------------------------------------------
// PUT /manifests/{id}/objects
// ---------------------------------------------------------------------------

pub async fn update_manifest_objects(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateManifestObjectsRequest>,
) -> Result<Json<Vec<ManifestObjectRow>>, AppError> {
    // Verify manifest exists.
    let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM world_manifests WHERE id = $1)")
        .bind(id)
        .fetch_one(&state.pool)
        .await
        .map_err(|e| AppError::internal(format!("exists check failed: {e}")))?;
    if !exists {
        return Err(AppError::not_found(format!("manifest {id} not found")));
    }

    // Replace all objects for this manifest (idempotent upsert per object).
    let mut updated = Vec::with_capacity(req.objects.len());
    for obj in &req.objects {
        let lods = obj.available_lods.clone().unwrap_or_default();
        let validation = obj.validation_state.clone().unwrap_or_else(|| "pending".into());
        let access = obj.access_requirements.clone().unwrap_or(serde_json::json!({}));
        let history = obj.update_history.clone().unwrap_or(serde_json::json!([]));

        let row = sqlx::query_as::<_, ManifestObjectRow>(
            r#"
            INSERT INTO manifest_objects (
                manifest_id, object_id, component_version_id,
                assembly_version_id, spatial_extent, storage_reference,
                available_lods, validation_state, access_requirements,
                update_history
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            ON CONFLICT (manifest_id, object_id) DO UPDATE SET
                component_version_id = EXCLUDED.component_version_id,
                assembly_version_id = EXCLUDED.assembly_version_id,
                spatial_extent = EXCLUDED.spatial_extent,
                storage_reference = EXCLUDED.storage_reference,
                available_lods = EXCLUDED.available_lods,
                validation_state = EXCLUDED.validation_state,
                access_requirements = EXCLUDED.access_requirements,
                update_history = EXCLUDED.update_history
            RETURNING id, manifest_id, object_id, component_version_id,
                      assembly_version_id, spatial_extent, storage_reference,
                      available_lods, validation_state, access_requirements,
                      update_history, created_at
            "#,
        )
        .bind(id)
        .bind(&obj.object_id)
        .bind(&obj.component_version_id)
        .bind(&obj.assembly_version_id)
        .bind(&serde_json::to_value(&obj.spatial_extent).unwrap_or(serde_json::Value::Null))
        .bind(&obj.storage_reference)
        .bind(&lods)
        .bind(validation)
        .bind(&access)
        .bind(&history)
        .fetch_one(&state.pool)
        .await
        .map_err(|e| AppError::internal(format!("upsert failed: {e}")))?;
        updated.push(row);
    }

    Ok(Json(updated))
}
