use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::state::AppState;

// ---------------------------------------------------------------------------
// DB models
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct CredentialRequestRow {
    pub id: i64,
    pub request_pubkey: String,
    pub holder_pubkey: String,
    pub zone_id: String,
    pub request_hash: String,
    pub status: String,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ThresholdCredentialRow {
    pub id: i64,
    pub credential_pubkey: String,
    pub holder_pubkey: String,
    pub zone_id: String,
    pub issued_at: i64,
    pub expires_at: i64,
    pub revoked: bool,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct CredentialNullifierRow {
    pub id: i64,
    pub nullifier_pubkey: String,
    pub credential_pubkey: String,
    pub nullified_at: i64,
    pub created_at: chrono::NaiveDateTime,
}

// ---------------------------------------------------------------------------
// Request types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct CreateCredentialRequestRequest {
    pub request_pubkey: String,
    pub holder_pubkey: String,
    pub zone_id: String,
    pub request_hash: String,
}

#[derive(Debug, Deserialize)]
pub struct FinalizeCredentialRequest {
    pub request_pubkey: String,
    pub credential_pubkey: String,
    pub holder_pubkey: String,
    pub zone_id: String,
    pub issued_at: i64,
    pub expires_at: i64,
}

#[derive(Debug, Deserialize)]
pub struct NullifyCredentialRequest {
    pub nullifier_pubkey: String,
    pub credential_pubkey: String,
}

// ---------------------------------------------------------------------------
// SQL constants
// ---------------------------------------------------------------------------

const CREDENTIAL_REQUEST_SELECT: &str =
    "SELECT id, request_pubkey, holder_pubkey, zone_id, request_hash, status, created_at FROM credential_requests";
const THRESHOLD_CREDENTIAL_SELECT: &str =
    "SELECT id, credential_pubkey, holder_pubkey, zone_id, issued_at, expires_at, revoked, created_at FROM threshold_credentials";
const CREDENTIAL_NULLIFIER_SELECT: &str =
    "SELECT id, nullifier_pubkey, credential_pubkey, nullified_at, created_at FROM credential_nullifiers";

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

async fn list_credential_requests(
    State(state): State<AppState>,
) -> Result<Json<Vec<CredentialRequestRow>>, AppError> {
    let rows = sqlx::query_as::<_, CredentialRequestRow>(CREDENTIAL_REQUEST_SELECT)
        .fetch_all(&state.pool)
        .await?;
    Ok(Json(rows))
}

async fn create_credential_request(
    State(state): State<AppState>,
    Json(req): Json<CreateCredentialRequestRequest>,
) -> Result<(axum::http::StatusCode, Json<CredentialRequestRow>), AppError> {
    let row = sqlx::query_as::<_, CredentialRequestRow>(
        "INSERT INTO credential_requests (request_pubkey, holder_pubkey, zone_id, request_hash, status)
         VALUES ($1, $2, $3, $4, 'pending')
         RETURNING id, request_pubkey, holder_pubkey, zone_id, request_hash, status, created_at",
    )
    .bind(&req.request_pubkey)
    .bind(&req.holder_pubkey)
    .bind(&req.zone_id)
    .bind(&req.request_hash)
    .fetch_one(&state.pool)
    .await?;
    Ok((axum::http::StatusCode::CREATED, Json(row)))
}

async fn finalize_credential(
    State(state): State<AppState>,
    Json(req): Json<FinalizeCredentialRequest>,
) -> Result<Json<ThresholdCredentialRow>, AppError> {
    // Mark request as signed
    sqlx::query("UPDATE credential_requests SET status = 'signed' WHERE request_pubkey = $1")
        .bind(&req.request_pubkey)
        .execute(&state.pool)
        .await?;

    let row = sqlx::query_as::<_, ThresholdCredentialRow>(
        "INSERT INTO threshold_credentials (credential_pubkey, holder_pubkey, zone_id, issued_at, expires_at)
         VALUES ($1, $2, $3, $4, $5)
         RETURNING id, credential_pubkey, holder_pubkey, zone_id, issued_at, expires_at, revoked, created_at",
    )
    .bind(&req.credential_pubkey)
    .bind(&req.holder_pubkey)
    .bind(&req.zone_id)
    .bind(req.issued_at)
    .bind(req.expires_at)
    .fetch_one(&state.pool)
    .await?;
    Ok(Json(row))
}

async fn list_credentials(
    State(state): State<AppState>,
) -> Result<Json<Vec<ThresholdCredentialRow>>, AppError> {
    let rows = sqlx::query_as::<_, ThresholdCredentialRow>(THRESHOLD_CREDENTIAL_SELECT)
        .fetch_all(&state.pool)
        .await?;
    Ok(Json(rows))
}

async fn get_credential(
    State(state): State<AppState>,
    Path(credential_pubkey): Path<String>,
) -> Result<Json<ThresholdCredentialRow>, AppError> {
    let row = sqlx::query_as::<_, ThresholdCredentialRow>(&format!(
        "{THRESHOLD_CREDENTIAL_SELECT} WHERE credential_pubkey = $1"
    ))
    .bind(&credential_pubkey)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::not_found("credential not found"))?;
    Ok(Json(row))
}

async fn nullify_credential(
    State(state): State<AppState>,
    Json(req): Json<NullifyCredentialRequest>,
) -> Result<(axum::http::StatusCode, Json<CredentialNullifierRow>), AppError> {
    // Mark credential as revoked
    sqlx::query("UPDATE threshold_credentials SET revoked = true WHERE credential_pubkey = $1")
        .bind(&req.credential_pubkey)
        .execute(&state.pool)
        .await?;

    let row = sqlx::query_as::<_, CredentialNullifierRow>(
        "INSERT INTO credential_nullifiers (nullifier_pubkey, credential_pubkey, nullified_at)
         VALUES ($1, $2, EXTRACT(EPOCH FROM NOW())::BIGINT)
         RETURNING id, nullifier_pubkey, credential_pubkey, nullified_at, created_at",
    )
    .bind(&req.nullifier_pubkey)
    .bind(&req.credential_pubkey)
    .fetch_one(&state.pool)
    .await?;
    Ok((axum::http::StatusCode::CREATED, Json(row)))
}

async fn list_nullifiers(
    State(state): State<AppState>,
) -> Result<Json<Vec<CredentialNullifierRow>>, AppError> {
    let rows = sqlx::query_as::<_, CredentialNullifierRow>(CREDENTIAL_NULLIFIER_SELECT)
        .fetch_all(&state.pool)
        .await?;
    Ok(Json(rows))
}

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/requests", get(list_credential_requests).post(create_credential_request))
        .route("/finalize", post(finalize_credential))
        .route("/", get(list_credentials))
        .route("/{credential_pubkey}", get(get_credential))
        .route("/nullify", post(nullify_credential))
        .route("/nullifiers", get(list_nullifiers))
}
