use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::state::AppState;

// ---------------------------------------------------------------------------
// DB models
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ValidatorActivityRow {
    pub id: i64,
    pub registry_pubkey: String,
    pub validator_pubkey: String,
    pub last_active: i64,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct EmergencyInjectionRow {
    pub id: i64,
    pub registry_pubkey: String,
    pub validator_pubkey: String,
    pub injected_by: String,
    pub queued_at: i64,
    pub execute_after: i64,
    pub executed: bool,
    pub created_at: chrono::NaiveDateTime,
}

// ---------------------------------------------------------------------------
// Request types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct CreateActivityRequest {
    pub registry_pubkey: String,
    pub validator_pubkey: String,
}

#[derive(Debug, Deserialize)]
pub struct QueueEmergencyInjectionRequest {
    pub registry_pubkey: String,
    pub validator_pubkey: String,
    pub injected_by: String,
}

// ---------------------------------------------------------------------------
// SQL constants
// ---------------------------------------------------------------------------

const ACTIVITY_SELECT: &str =
    "SELECT id, registry_pubkey, validator_pubkey, last_active, created_at, updated_at FROM validator_activities";
const EMERGENCY_INJECTION_SELECT: &str =
    "SELECT id, registry_pubkey, validator_pubkey, injected_by, queued_at, execute_after, executed, created_at FROM emergency_injections";

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

async fn list_activities(
    State(state): State<AppState>,
) -> Result<Json<Vec<ValidatorActivityRow>>, AppError> {
    let rows = sqlx::query_as::<_, ValidatorActivityRow>(ACTIVITY_SELECT)
        .fetch_all(&state.pool)
        .await?;
    Ok(Json(rows))
}

async fn get_activity(
    State(state): State<AppState>,
    Path((registry_pubkey, validator_pubkey)): Path<(String, String)>,
) -> Result<Json<ValidatorActivityRow>, AppError> {
    let row = sqlx::query_as::<_, ValidatorActivityRow>(&format!(
        "{ACTIVITY_SELECT} WHERE registry_pubkey = $1 AND validator_pubkey = $2"
    ))
    .bind(&registry_pubkey)
    .bind(&validator_pubkey)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::not_found("validator activity not found"))?;
    Ok(Json(row))
}

async fn upsert_activity(
    State(state): State<AppState>,
    Json(req): Json<CreateActivityRequest>,
) -> Result<Json<ValidatorActivityRow>, AppError> {
    let row = sqlx::query_as::<_, ValidatorActivityRow>(
        "INSERT INTO validator_activities (registry_pubkey, validator_pubkey, last_active)
         VALUES ($1, $2, EXTRACT(EPOCH FROM NOW())::BIGINT)
         ON CONFLICT (registry_pubkey, validator_pubkey)
         DO UPDATE SET last_active = EXTRACT(EPOCH FROM NOW())::BIGINT, updated_at = NOW()
         RETURNING id, registry_pubkey, validator_pubkey, last_active, created_at, updated_at",
    )
    .bind(&req.registry_pubkey)
    .bind(&req.validator_pubkey)
    .fetch_one(&state.pool)
    .await?;
    Ok(Json(row))
}

async fn list_injections(
    State(state): State<AppState>,
) -> Result<Json<Vec<EmergencyInjectionRow>>, AppError> {
    let rows = sqlx::query_as::<_, EmergencyInjectionRow>(EMERGENCY_INJECTION_SELECT)
        .fetch_all(&state.pool)
        .await?;
    Ok(Json(rows))
}

async fn queue_injection(
    State(state): State<AppState>,
    Json(req): Json<QueueEmergencyInjectionRequest>,
) -> Result<(axum::http::StatusCode, Json<EmergencyInjectionRow>), AppError> {
    let row = sqlx::query_as::<_, EmergencyInjectionRow>(
        "INSERT INTO emergency_injections (registry_pubkey, validator_pubkey, injected_by, queued_at, execute_after)
         VALUES ($1, $2, $3, EXTRACT(EPOCH FROM NOW())::BIGINT, EXTRACT(EPOCH FROM NOW())::BIGINT + 172800)
         RETURNING id, registry_pubkey, validator_pubkey, injected_by, queued_at, execute_after, executed, created_at",
    )
    .bind(&req.registry_pubkey)
    .bind(&req.validator_pubkey)
    .bind(&req.injected_by)
    .fetch_one(&state.pool)
    .await?;
    Ok((axum::http::StatusCode::CREATED, Json(row)))
}

async fn get_injection(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<EmergencyInjectionRow>, AppError> {
    let row = sqlx::query_as::<_, EmergencyInjectionRow>(&format!(
        "{EMERGENCY_INJECTION_SELECT} WHERE id = $1"
    ))
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::not_found("emergency injection not found"))?;
    Ok(Json(row))
}

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/activities", get(list_activities).post(upsert_activity))
        .route("/activities/{registry_pubkey}/{validator_pubkey}", get(get_activity))
        .route("/emergency-injections", get(list_injections).post(queue_injection))
        .route("/emergency-injections/{id}", get(get_injection))
}
