//! Chapter 11 — Spatial Assembly API (REQ-026 dependent).
//!
//! Assembly connects accepted, independently produced components into coherent
//! representations of buildings, parcels, districts. It records component
//! version relationships, registration transforms, and conflict detection.
//!
//! Endpoints:
//!   POST   /assemblies                          create an assembly
//!   GET    /assemblies/{id}                     get an assembly with objects + conflicts
//!   GET    /assemblies                          list assemblies (filter by type/root_object/status)
//!   POST   /assemblies/{id}/objects             add a component to an assembly
//!   POST   /assemblies/{id}/conflicts           record a conflict between components
//!   PUT    /assemblies/{id}/status              update assembly status (validate/complete/reject)

use axum::extract::{Path, Query, State};
use axum::routing::{get, post, put};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::AppError;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", post(create_assembly).get(list_assemblies))
        .route("/{id}", get(get_assembly))
        .route("/{id}/objects", post(add_assembly_object))
        .route("/{id}/conflicts", post(record_conflict))
        .route("/{id}/status", put(update_assembly_status))
}

// ---------------------------------------------------------------------------
// Request / response types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct CreateAssemblyRequest {
    pub assembly_id: String,
    pub name: String,
    pub object_type: String,
    pub root_object_id: String,
}

#[derive(Debug, Deserialize)]
pub struct AddAssemblyObjectRequest {
    pub object_id: String,
    pub component_version_id: String,
    pub relationship_type: String,
    pub transform: Option<serde_json::Value>,
    pub position_in_assembly: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct RecordConflictRequest {
    pub object_a_id: String,
    pub object_b_id: String,
    pub conflict_type: String,
    pub severity: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAssemblyStatusRequest {
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct ListAssembliesParams {
    pub object_type: Option<String>,
    pub root_object_id: Option<String>,
    pub status: Option<String>,
    #[serde(default = "default_limit")]
    pub limit: usize,
}

fn default_limit() -> usize {
    50
}

// ---------------------------------------------------------------------------
// Row types
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct AssemblyRow {
    pub id: Uuid,
    pub assembly_id: String,
    pub name: String,
    pub object_type: String,
    pub root_object_id: String,
    pub version: i32,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct AssemblyObjectRow {
    pub id: Uuid,
    pub assembly_id: Uuid,
    pub object_id: String,
    pub component_version_id: String,
    pub relationship_type: String,
    pub transform: Option<serde_json::Value>,
    pub position_in_assembly: Option<serde_json::Value>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct AssemblyConflictRow {
    pub id: Uuid,
    pub assembly_id: Uuid,
    pub object_a_id: String,
    pub object_b_id: String,
    pub conflict_type: String,
    pub severity: String,
    pub status: String,
    pub description: Option<String>,
    pub resolution: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub resolved_at: Option<chrono::DateTime<chrono::Utc>>,
}

// ---------------------------------------------------------------------------
// POST /assemblies
// ---------------------------------------------------------------------------

pub async fn create_assembly(
    State(state): State<AppState>,
    Json(req): Json<CreateAssemblyRequest>,
) -> Result<(axum::http::StatusCode, Json<AssemblyRow>), AppError> {
    let row = sqlx::query_as::<_, AssemblyRow>(
        r#"
        INSERT INTO assemblies (assembly_id, name, object_type, root_object_id)
        VALUES ($1, $2, $3, $4)
        RETURNING id, assembly_id, name, object_type, root_object_id, version,
                  status, created_at, updated_at
        "#,
    )
    .bind(&req.assembly_id)
    .bind(&req.name)
    .bind(&req.object_type)
    .bind(&req.root_object_id)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| AppError::internal(format!("insert failed: {e}")))?;

    Ok((axum::http::StatusCode::CREATED, Json(row)))
}

// ---------------------------------------------------------------------------
// GET /assemblies/{id}
// ---------------------------------------------------------------------------

const ASSEMBLY_SELECT: &str = "SELECT id, assembly_id, name, object_type, \
    root_object_id, version, status, created_at, updated_at \
    FROM assemblies WHERE id = $1";

const ASSEMBLY_OBJECTS_SELECT: &str = "SELECT id, assembly_id, object_id, \
    component_version_id, relationship_type, transform, position_in_assembly, \
    created_at FROM assembly_objects WHERE assembly_id = $1 ORDER BY object_id";

const ASSEMBLY_CONFLICTS_SELECT: &str = "SELECT id, assembly_id, object_a_id, \
    object_b_id, conflict_type, severity, status, description, resolution, \
    created_at, resolved_at FROM assembly_conflicts WHERE assembly_id = $1 \
    ORDER BY created_at";

#[derive(Debug, Serialize)]
pub struct AssemblyWithDetails {
    pub assembly: AssemblyRow,
    pub objects: Vec<AssemblyObjectRow>,
    pub conflicts: Vec<AssemblyConflictRow>,
}

pub async fn get_assembly(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<AssemblyWithDetails>, AppError> {
    let assembly = sqlx::query_as::<_, AssemblyRow>(ASSEMBLY_SELECT)
        .bind(id)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| AppError::internal(format!("query failed: {e}")))?
        .ok_or_else(|| AppError::not_found(format!("assembly {id} not found")))?;

    let objects = sqlx::query_as::<_, AssemblyObjectRow>(ASSEMBLY_OBJECTS_SELECT)
        .bind(id)
        .fetch_all(&state.pool)
        .await
        .map_err(|e| AppError::internal(format!("query failed: {e}")))?;

    let conflicts = sqlx::query_as::<_, AssemblyConflictRow>(ASSEMBLY_CONFLICTS_SELECT)
        .bind(id)
        .fetch_all(&state.pool)
        .await
        .map_err(|e| AppError::internal(format!("query failed: {e}")))?;

    Ok(Json(AssemblyWithDetails {
        assembly,
        objects,
        conflicts,
    }))
}

// ---------------------------------------------------------------------------
// GET /assemblies
// ---------------------------------------------------------------------------

const ASSEMBLIES_SELECT: &str = "SELECT id, assembly_id, name, object_type, \
    root_object_id, version, status, created_at, updated_at FROM assemblies";

pub async fn list_assemblies(
    State(state): State<AppState>,
    Query(params): Query<ListAssembliesParams>,
) -> Result<Json<Vec<AssemblyRow>>, AppError> {
    let limit = params.limit.min(500);
    let rows: Vec<AssemblyRow> = match (
        &params.object_type,
        &params.root_object_id,
        &params.status,
    ) {
        (Some(t), Some(r), Some(s)) => sqlx::query_as::<_, AssemblyRow>(
            format!("{ASSEMBLIES_SELECT} WHERE object_type = $1 AND root_object_id = $2 AND status = $3 ORDER BY version DESC LIMIT $4").as_str(),
        )
        .bind(t)
        .bind(r)
        .bind(s)
        .bind(limit as i64)
        .fetch_all(&state.pool)
        .await
        .map_err(|e| AppError::internal(format!("query failed: {e}")))?,
        (Some(t), Some(r), None) => sqlx::query_as::<_, AssemblyRow>(
            format!("{ASSEMBLIES_SELECT} WHERE object_type = $1 AND root_object_id = $2 ORDER BY version DESC LIMIT $3").as_str(),
        )
        .bind(t)
        .bind(r)
        .bind(limit as i64)
        .fetch_all(&state.pool)
        .await
        .map_err(|e| AppError::internal(format!("query failed: {e}")))?,
        (Some(t), None, Some(s)) => sqlx::query_as::<_, AssemblyRow>(
            format!("{ASSEMBLIES_SELECT} WHERE object_type = $1 AND status = $2 ORDER BY version DESC LIMIT $3").as_str(),
        )
        .bind(t)
        .bind(s)
        .bind(limit as i64)
        .fetch_all(&state.pool)
        .await
        .map_err(|e| AppError::internal(format!("query failed: {e}")))?,
        (None, Some(r), Some(s)) => sqlx::query_as::<_, AssemblyRow>(
            format!("{ASSEMBLIES_SELECT} WHERE root_object_id = $1 AND status = $2 ORDER BY version DESC LIMIT $3").as_str(),
        )
        .bind(r)
        .bind(s)
        .bind(limit as i64)
        .fetch_all(&state.pool)
        .await
        .map_err(|e| AppError::internal(format!("query failed: {e}")))?,
        (Some(t), None, None) => sqlx::query_as::<_, AssemblyRow>(
            format!("{ASSEMBLIES_SELECT} WHERE object_type = $1 ORDER BY version DESC LIMIT $2").as_str(),
        )
        .bind(t)
        .bind(limit as i64)
        .fetch_all(&state.pool)
        .await
        .map_err(|e| AppError::internal(format!("query failed: {e}")))?,
        (None, Some(r), None) => sqlx::query_as::<_, AssemblyRow>(
            format!("{ASSEMBLIES_SELECT} WHERE root_object_id = $1 ORDER BY version DESC LIMIT $2").as_str(),
        )
        .bind(r)
        .bind(limit as i64)
        .fetch_all(&state.pool)
        .await
        .map_err(|e| AppError::internal(format!("query failed: {e}")))?,
        (None, None, Some(s)) => sqlx::query_as::<_, AssemblyRow>(
            format!("{ASSEMBLIES_SELECT} WHERE status = $1 ORDER BY version DESC LIMIT $2").as_str(),
        )
        .bind(s)
        .bind(limit as i64)
        .fetch_all(&state.pool)
        .await
        .map_err(|e| AppError::internal(format!("query failed: {e}")))?,
        (None, None, None) => sqlx::query_as::<_, AssemblyRow>(
            format!("{ASSEMBLIES_SELECT} ORDER BY root_object_id, version DESC LIMIT $1").as_str(),
        )
        .bind(limit as i64)
        .fetch_all(&state.pool)
        .await
        .map_err(|e| AppError::internal(format!("query failed: {e}")))?,
    };

    Ok(Json(rows))
}

// ---------------------------------------------------------------------------
// POST /assemblies/{id}/objects
// ---------------------------------------------------------------------------

pub async fn add_assembly_object(
    State(state): State<AppState>,
    Path(assembly_id): Path<Uuid>,
    Json(req): Json<AddAssemblyObjectRequest>,
) -> Result<(axum::http::StatusCode, Json<AssemblyObjectRow>), AppError> {
    let row = sqlx::query_as::<_, AssemblyObjectRow>(
        r#"
        INSERT INTO assembly_objects (
            assembly_id, object_id, component_version_id, relationship_type,
            transform, position_in_assembly
        )
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id, assembly_id, object_id, component_version_id,
                  relationship_type, transform, position_in_assembly, created_at
        "#,
    )
    .bind(assembly_id)
    .bind(&req.object_id)
    .bind(&req.component_version_id)
    .bind(&req.relationship_type)
    .bind(serde_json::to_value(&req.transform).unwrap_or(serde_json::Value::Null))
    .bind(serde_json::to_value(&req.position_in_assembly).unwrap_or(serde_json::Value::Null))
    .fetch_one(&state.pool)
    .await
    .map_err(|e| AppError::internal(format!("insert failed: {e}")))?;

    Ok((axum::http::StatusCode::CREATED, Json(row)))
}

// ---------------------------------------------------------------------------
// POST /assemblies/{id}/conflicts
// ---------------------------------------------------------------------------

pub async fn record_conflict(
    State(state): State<AppState>,
    Path(assembly_id): Path<Uuid>,
    Json(req): Json<RecordConflictRequest>,
) -> Result<(axum::http::StatusCode, Json<AssemblyConflictRow>), AppError> {
    let severity = req.severity.unwrap_or_else(|| "medium".to_string());

    let row = sqlx::query_as::<_, AssemblyConflictRow>(
        r#"
        INSERT INTO assembly_conflicts (
            assembly_id, object_a_id, object_b_id, conflict_type,
            severity, description
        )
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id, assembly_id, object_a_id, object_b_id, conflict_type,
                  severity, status, description, resolution, created_at,
                  resolved_at
        "#,
    )
    .bind(assembly_id)
    .bind(&req.object_a_id)
    .bind(&req.object_b_id)
    .bind(&req.conflict_type)
    .bind(severity)
    .bind(&req.description)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| AppError::internal(format!("insert failed: {e}")))?;

    Ok((axum::http::StatusCode::CREATED, Json(row)))
}

// ---------------------------------------------------------------------------
// PUT /assemblies/{id}/status
// ---------------------------------------------------------------------------

pub async fn update_assembly_status(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateAssemblyStatusRequest>,
) -> Result<Json<AssemblyRow>, AppError> {
    let row = sqlx::query_as::<_, AssemblyRow>(
        "UPDATE assemblies SET status = $1, updated_at = now() WHERE id = $2 \
         RETURNING id, assembly_id, name, object_type, root_object_id, version,
                  status, created_at, updated_at",
    )
    .bind(&req.status)
    .bind(id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| AppError::internal(format!("update failed: {e}")))?
    .ok_or_else(|| AppError::not_found(format!("assembly {id} not found")))?;

    Ok(Json(row))
}
