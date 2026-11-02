//! Stage 5: Distributed providers — worker registration + job assignment.
//!
//! Endpoints:
//!   POST   /workers                          register a worker with capabilities
//!   GET    /workers                          list workers (filter by status/capability)
//!   GET    /workers/{id}                     get a worker
//!   PUT    /workers/{id}                     update worker capabilities/status
//!   POST   /workers/{id}/heartbeat           refresh worker heartbeat
//!   GET    /workers/{id}/assignments         list jobs assigned to a worker
//!   POST   /assignments/{job_id}/complete    mark an assignment complete with resource receipt

use axum::extract::{Path, Query, State};
use axum::routing::{get, post, put};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::AppError;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", post(register_worker).get(list_workers))
        .route("/{id}", get(get_worker).put(update_worker))
        .route("/{id}/heartbeat", post(heartbeat_worker))
        .route("/workers/reap-stale", post(reap_stale_workers))
        .route("/{id}/assignments", get(list_worker_assignments))
        .route("/assignments/{job_id}/complete", post(complete_assignment))
}

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct WorkerRow {
    pub id: Uuid,
    pub worker_id: String,
    pub wallet_address: String,
    pub capabilities: serde_json::Value,
    pub status: String,
    pub last_heartbeat_at: chrono::DateTime<chrono::Utc>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct RegisterWorkerRequest {
    pub worker_id: String,
    pub wallet_address: String,
    pub capabilities: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateWorkerRequest {
    pub capabilities: Option<serde_json::Value>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListWorkersParams {
    pub status: Option<String>,
    #[serde(default = "default_limit")]
    pub limit: usize,
}

fn default_limit() -> usize {
    50
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct AssignmentRow {
    pub id: Uuid,
    pub job_id: String,
    pub worker_id: String,
    pub assigned_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub result_uri: Option<String>,
    pub resource_receipt: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct CompleteAssignmentRequest {
    pub result_uri: Option<String>,
    pub resource_receipt: Option<serde_json::Value>,
}

// Validate a resource usage receipt: must be a JSON object with numeric,
// non-negative values for cpu_seconds, gpu_seconds, storage_mb, bandwidth_mb.
fn validate_resource_receipt(receipt: &serde_json::Value) -> Result<(), AppError> {
    let obj = receipt.as_object().ok_or_else(|| {
        AppError::bad_request("resource_receipt must be a JSON object")
    })?;
    let allowed_keys = [
        "cpu_seconds",
        "gpu_seconds",
        "storage_mb",
        "bandwidth_mb",
        "energy_wh",
    ];
    for (key, value) in obj {
        if !allowed_keys.contains(&key.as_str()) {
            return Err(AppError::bad_request(format!(
                "resource_receipt contains unknown key: {key}"
            )));
        }
        let num = value.as_f64().ok_or_else(|| {
            AppError::bad_request(format!("resource_receipt.{key} must be a number"))
        })?;
        if num < 0.0 {
            return Err(AppError::bad_request(format!(
                "resource_receipt.{key} must be non-negative"
            )));
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// POST /workers
// ---------------------------------------------------------------------------

pub async fn register_worker(
    State(state): State<AppState>,
    Json(req): Json<RegisterWorkerRequest>,
) -> Result<(axum::http::StatusCode, Json<WorkerRow>), AppError> {
    let capabilities = req.capabilities.unwrap_or(serde_json::json!({}));
    let row = sqlx::query_as::<_, WorkerRow>(
        r#"
        INSERT INTO worker_registrations (worker_id, wallet_address, capabilities)
        VALUES ($1, $2, $3)
        ON CONFLICT (worker_id) DO UPDATE SET
            wallet_address = EXCLUDED.wallet_address,
            capabilities = EXCLUDED.capabilities,
            status = 'active',
            last_heartbeat_at = now(),
            updated_at = now()
        RETURNING id, worker_id, wallet_address, capabilities, status,
                  last_heartbeat_at, created_at, updated_at
        "#,
    )
    .bind(&req.worker_id)
    .bind(&req.wallet_address)
    .bind(&capabilities)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| AppError::internal(format!("register worker failed: {e}")))?;

    Ok((axum::http::StatusCode::CREATED, Json(row)))
}

// ---------------------------------------------------------------------------
// GET /workers
// ---------------------------------------------------------------------------

const WORKERS_SELECT: &str = "SELECT id, worker_id, wallet_address, capabilities, \
    status, last_heartbeat_at, created_at, updated_at FROM worker_registrations";

pub async fn list_workers(
    State(state): State<AppState>,
    Query(params): Query<ListWorkersParams>,
) -> Result<Json<Vec<WorkerRow>>, AppError> {
    let limit = params.limit.min(500);
    let rows: Vec<WorkerRow> = match &params.status {
        Some(s) => sqlx::query_as::<_, WorkerRow>(
            format!("{WORKERS_SELECT} WHERE status = $1 ORDER BY last_heartbeat_at DESC LIMIT $2").as_str(),
        )
        .bind(s)
        .bind(limit as i64)
        .fetch_all(&state.pool)
        .await
        .map_err(|e| AppError::internal(format!("query failed: {e}")))?,
        None => sqlx::query_as::<_, WorkerRow>(
            format!("{WORKERS_SELECT} ORDER BY last_heartbeat_at DESC LIMIT $1").as_str(),
        )
        .bind(limit as i64)
        .fetch_all(&state.pool)
        .await
        .map_err(|e| AppError::internal(format!("query failed: {e}")))?,
    };

    Ok(Json(rows))
}

// ---------------------------------------------------------------------------
// GET /workers/{id}
// ---------------------------------------------------------------------------

const WORKER_SELECT: &str = "SELECT id, worker_id, wallet_address, capabilities, \
    status, last_heartbeat_at, created_at, updated_at FROM worker_registrations \
    WHERE id = $1";

pub async fn get_worker(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<WorkerRow>, AppError> {
    let row = sqlx::query_as::<_, WorkerRow>(WORKER_SELECT)
        .bind(id)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| AppError::internal(format!("query failed: {e}")))?
        .ok_or_else(|| AppError::not_found(format!("worker {id} not found")))?;

    Ok(Json(row))
}

// ---------------------------------------------------------------------------
// PUT /workers/{id}
// ---------------------------------------------------------------------------

pub async fn update_worker(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateWorkerRequest>,
) -> Result<Json<WorkerRow>, AppError> {
    let capabilities = req.capabilities.unwrap_or(serde_json::json!({}));
    let status = req.status.unwrap_or_else(|| "active".to_string());

    let row = sqlx::query_as::<_, WorkerRow>(
        r#"
        UPDATE worker_registrations
        SET capabilities = $2, status = $3, updated_at = now()
        WHERE id = $1
        RETURNING id, worker_id, wallet_address, capabilities, status,
                  last_heartbeat_at, created_at, updated_at
        "#,
    )
    .bind(id)
    .bind(&capabilities)
    .bind(&status)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| AppError::internal(format!("update worker failed: {e}")))?
    .ok_or_else(|| AppError::not_found(format!("worker {id} not found")))?;

    Ok(Json(row))
}

// ---------------------------------------------------------------------------
// POST /workers/{id}/heartbeat
// ---------------------------------------------------------------------------

pub async fn heartbeat_worker(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<WorkerRow>, AppError> {
    let row = sqlx::query_as::<_, WorkerRow>(
        r#"
        UPDATE worker_registrations
        SET last_heartbeat_at = now(), updated_at = now()
        WHERE id = $1
        RETURNING id, worker_id, wallet_address, capabilities, status,
                  last_heartbeat_at, created_at, updated_at
        "#,
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| AppError::internal(format!("heartbeat failed: {e}")))?
    .ok_or_else(|| AppError::not_found(format!("worker {id} not found")))?;

    Ok(Json(row))
}

/// Mark workers as inactive if they have not heartbeated within the staleness
/// window (default 30 minutes). Called from the lease path so that stale
/// workers do not keep being considered for new jobs.
pub async fn reap_stale_workers(
    State(state): State<AppState>,
    Query(params): Query<StaleWorkerParams>,
) -> Result<Json<serde_json::Value>, AppError> {
    let stale_minutes: i64 = params.minutes.unwrap_or(30);
    let result = sqlx::query(
        r#"
        UPDATE worker_registrations
        SET status = 'inactive', updated_at = now()
        WHERE status = 'active'
          AND last_heartbeat_at < now() - make_interval(secs => $1::double precision)
        "#,
    )
    .bind(stale_minutes as f64)
    .execute(&state.pool)
    .await
    .map_err(|e| AppError::internal(format!("reap stale workers failed: {e}")))?;

    Ok(Json(serde_json::json!({
        "marked_inactive": result.rows_affected()
    })))
}

#[derive(Debug, Deserialize)]
pub struct StaleWorkerParams {
    pub minutes: Option<i64>,
}

// ---------------------------------------------------------------------------
// GET /workers/{id}/assignments
// ---------------------------------------------------------------------------

const ASSIGNMENTS_SELECT: &str = "SELECT id, job_id, worker_id, assigned_at, \
    completed_at, result_uri, resource_receipt FROM job_assignments \
    WHERE worker_id = $1 ORDER BY assigned_at DESC";

pub async fn list_worker_assignments(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<AssignmentRow>>, AppError> {
    let rows = sqlx::query_as::<_, AssignmentRow>(ASSIGNMENTS_SELECT)
        .bind(id)
        .fetch_all(&state.pool)
        .await
        .map_err(|e| AppError::internal(format!("query failed: {e}")))?;

    Ok(Json(rows))
}

// ---------------------------------------------------------------------------
// POST /assignments/{job_id}/complete
// ---------------------------------------------------------------------------

pub async fn complete_assignment(
    State(state): State<AppState>,
    Path(job_id): Path<String>,
    Json(req): Json<CompleteAssignmentRequest>,
) -> Result<Json<AssignmentRow>, AppError> {
    // Validate the resource receipt if one was provided, before writing.
    if let Some(ref receipt) = req.resource_receipt {
        validate_resource_receipt(receipt)?;
    }

    let row = sqlx::query_as::<_, AssignmentRow>(
        r#"
        UPDATE job_assignments
        SET completed_at = now(), result_uri = $2, resource_receipt = $3
        WHERE job_id = $1
        RETURNING id, job_id, worker_id, assigned_at, completed_at,
                  result_uri, resource_receipt
        "#,
    )
    .bind(&job_id)
    .bind(&req.result_uri)
    .bind(&req.resource_receipt)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| AppError::internal(format!("complete assignment failed: {e}")))?
    .ok_or_else(|| AppError::not_found(format!("assignment for job {job_id} not found")))?;

    Ok(Json(row))
}
