//! Chapter 12 — Storage provider discovery (REQ-028).
//!
//! Endpoints:
//!   POST   /providers                          register a storage provider
//!   GET    /providers                          list providers (filter by type/active/region)
//!   GET    /providers/{id}                     get a provider
//!   GET    /providers/{id}/replicas            list a provider's replicas
//!   POST   /providers/{id}/replicas            register a component replica

use axum::extract::{Path, Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::error::AppError;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", post(create_provider).get(list_providers))
        .route("/{id}", get(get_provider))
        .route("/{id}/replicas", post(create_replica).get(list_replicas))
        .route(
            "/{provider_id}/replicas/{replica_id}/challenge",
            post(challenge_replica),
        )
        .route(
            "/{provider_id}/replicas/{replica_id}/prove",
            post(prove_replica),
        )
}

// ---------------------------------------------------------------------------
// Request / response types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct CreateProviderRequest {
    pub provider_id: String,
    pub name: String,
    pub provider_type: String,
    pub base_url: String,
    pub capabilities: Option<serde_json::Value>,
    pub geographic_scope: Option<Vec<String>>,
    pub reliability_score: Option<f64>,
    pub cost_per_gb: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct CreateReplicaRequest {
    pub component_id: String,
    pub component_version: String,
    pub storage_reference: String,
    pub version: String,
    pub size_bytes: Option<i64>,
    pub checksum: Option<String>,
    pub latency_ms: Option<i32>,
    pub is_available: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct ListProvidersParams {
    pub provider_type: Option<String>,
    pub is_active: Option<bool>,
    pub region: Option<String>,
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
pub struct ProviderRow {
    pub id: Uuid,
    pub provider_id: String,
    pub name: String,
    pub provider_type: String,
    pub base_url: String,
    pub capabilities: serde_json::Value,
    pub geographic_scope: Vec<String>,
    pub is_active: bool,
    pub reliability_score: f64,
    pub cost_per_gb: Option<f64>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ReplicaRow {
    pub id: Uuid,
    pub provider_id: Uuid,
    pub component_id: String,
    pub component_version: String,
    pub storage_reference: String,
    pub version: String,
    pub size_bytes: Option<i64>,
    pub checksum: Option<String>,
    pub latency_ms: Option<i32>,
    pub is_available: bool,
    pub last_checked_at: chrono::DateTime<chrono::Utc>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

// ---------------------------------------------------------------------------
// POST /providers
// ---------------------------------------------------------------------------

pub async fn create_provider(
    State(state): State<AppState>,
    Json(req): Json<CreateProviderRequest>,
) -> Result<(axum::http::StatusCode, Json<ProviderRow>), AppError> {
    let capabilities = req.capabilities.unwrap_or(serde_json::json!({}));
    let geo_scope = req.geographic_scope.unwrap_or_default();
    let reliability = req.reliability_score.unwrap_or(1.0);
    let cost = req.cost_per_gb;

    let row = sqlx::query_as::<_, ProviderRow>(
        r#"
        INSERT INTO storage_providers (
            provider_id, name, provider_type, base_url, capabilities,
            geographic_scope, reliability_score, cost_per_gb
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING id, provider_id, name, provider_type, base_url, capabilities,
                  geographic_scope, is_active, reliability_score, cost_per_gb,
                  created_at, updated_at
        "#,
    )
    .bind(&req.provider_id)
    .bind(&req.name)
    .bind(&req.provider_type)
    .bind(&req.base_url)
    .bind(&capabilities)
    .bind(&geo_scope)
    .bind(reliability)
    .bind(cost)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| AppError::internal(format!("insert failed: {e}")))?;

    Ok((axum::http::StatusCode::CREATED, Json(row)))
}

// ---------------------------------------------------------------------------
// GET /providers
// ---------------------------------------------------------------------------

const PROVIDERS_SELECT: &str = "SELECT id, provider_id, name, provider_type, \
    base_url, capabilities, geographic_scope, is_active, reliability_score, \
    cost_per_gb, created_at, updated_at FROM storage_providers";

pub async fn list_providers(
    State(state): State<AppState>,
    Query(params): Query<ListProvidersParams>,
) -> Result<Json<Vec<ProviderRow>>, AppError> {
    let limit = params.limit.min(500);
    let rows: Vec<ProviderRow> = match (&params.provider_type, &params.is_active, &params.region) {
        (Some(t), Some(a), Some(r)) => sqlx::query_as::<_, ProviderRow>(
            format!("{PROVIDERS_SELECT} WHERE provider_type = $1 AND is_active = $2 AND $3 = ANY(geographic_scope) ORDER BY reliability_score DESC LIMIT $4").as_str(),
        )
        .bind(t)
        .bind(a)
        .bind(r)
        .bind(limit as i64)
        .fetch_all(&state.pool)
        .await
        .map_err(|e| AppError::internal(format!("query failed: {e}")))?,
        (Some(t), Some(a), None) => sqlx::query_as::<_, ProviderRow>(
            format!("{PROVIDERS_SELECT} WHERE provider_type = $1 AND is_active = $2 ORDER BY reliability_score DESC LIMIT $3").as_str(),
        )
        .bind(t)
        .bind(a)
        .bind(limit as i64)
        .fetch_all(&state.pool)
        .await
        .map_err(|e| AppError::internal(format!("query failed: {e}")))?,
        (Some(t), None, Some(r)) => sqlx::query_as::<_, ProviderRow>(
            format!("{PROVIDERS_SELECT} WHERE provider_type = $1 AND $2 = ANY(geographic_scope) ORDER BY reliability_score DESC LIMIT $3").as_str(),
        )
        .bind(t)
        .bind(r)
        .bind(limit as i64)
        .fetch_all(&state.pool)
        .await
        .map_err(|e| AppError::internal(format!("query failed: {e}")))?,
        (None, Some(a), Some(r)) => sqlx::query_as::<_, ProviderRow>(
            format!("{PROVIDERS_SELECT} WHERE is_active = $1 AND $2 = ANY(geographic_scope) ORDER BY reliability_score DESC LIMIT $3").as_str(),
        )
        .bind(a)
        .bind(r)
        .bind(limit as i64)
        .fetch_all(&state.pool)
        .await
        .map_err(|e| AppError::internal(format!("query failed: {e}")))?,
        (Some(t), None, None) => sqlx::query_as::<_, ProviderRow>(
            format!("{PROVIDERS_SELECT} WHERE provider_type = $1 ORDER BY reliability_score DESC LIMIT $2").as_str(),
        )
        .bind(t)
        .bind(limit as i64)
        .fetch_all(&state.pool)
        .await
        .map_err(|e| AppError::internal(format!("query failed: {e}")))?,
        (None, Some(a), None) => sqlx::query_as::<_, ProviderRow>(
            format!("{PROVIDERS_SELECT} WHERE is_active = $1 ORDER BY reliability_score DESC LIMIT $2").as_str(),
        )
        .bind(a)
        .bind(limit as i64)
        .fetch_all(&state.pool)
        .await
        .map_err(|e| AppError::internal(format!("query failed: {e}")))?,
        (None, None, Some(r)) => sqlx::query_as::<_, ProviderRow>(
            format!("{PROVIDERS_SELECT} WHERE $1 = ANY(geographic_scope) ORDER BY reliability_score DESC LIMIT $2").as_str(),
        )
        .bind(r)
        .bind(limit as i64)
        .fetch_all(&state.pool)
        .await
        .map_err(|e| AppError::internal(format!("query failed: {e}")))?,
        (None, None, None) => sqlx::query_as::<_, ProviderRow>(
            format!("{PROVIDERS_SELECT} ORDER BY reliability_score DESC LIMIT $1").as_str(),
        )
        .bind(limit as i64)
        .fetch_all(&state.pool)
        .await
        .map_err(|e| AppError::internal(format!("query failed: {e}")))?,
    };

    Ok(Json(rows))
}

// ---------------------------------------------------------------------------
// GET /providers/{id}
// ---------------------------------------------------------------------------

const PROVIDER_SELECT: &str = "SELECT id, provider_id, name, provider_type, \
    base_url, capabilities, geographic_scope, is_active, reliability_score, \
    cost_per_gb, created_at, updated_at FROM storage_providers WHERE id = $1";

pub async fn get_provider(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ProviderRow>, AppError> {
    let row = sqlx::query_as::<_, ProviderRow>(PROVIDER_SELECT)
        .bind(id)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| AppError::internal(format!("query failed: {e}")))?
        .ok_or_else(|| AppError::not_found(format!("provider {id} not found")))?;

    Ok(Json(row))
}

// ---------------------------------------------------------------------------
// POST /providers/{id}/replicas
// ---------------------------------------------------------------------------

pub async fn create_replica(
    State(state): State<AppState>,
    Path(provider_id): Path<Uuid>,
    Json(req): Json<CreateReplicaRequest>,
) -> Result<(axum::http::StatusCode, Json<ReplicaRow>), AppError> {
    // Verify provider exists and is active.
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM storage_providers WHERE id = $1 AND is_active = true)",
    )
    .bind(provider_id)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| AppError::internal(format!("provider check failed: {e}")))?;
    if !exists {
        return Err(AppError::not_found(format!(
            "active provider {provider_id} not found"
        )));
    }

    let row = sqlx::query_as::<_, ReplicaRow>(
        r#"
        INSERT INTO provider_replicas (
            provider_id, component_id, component_version, storage_reference,
            version, size_bytes, checksum, latency_ms, is_available
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        RETURNING id, provider_id, component_id, component_version,
                  storage_reference, version, size_bytes, checksum,
                  latency_ms, is_available, last_checked_at, created_at
        "#,
    )
    .bind(provider_id)
    .bind(&req.component_id)
    .bind(&req.component_version)
    .bind(&req.storage_reference)
    .bind(&req.version)
    .bind(req.size_bytes)
    .bind(&req.checksum)
    .bind(req.latency_ms)
    .bind(req.is_available.unwrap_or(true))
    .fetch_one(&state.pool)
    .await
    .map_err(|e| AppError::internal(format!("insert failed: {e}")))?;

    Ok((axum::http::StatusCode::CREATED, Json(row)))
}

// ---------------------------------------------------------------------------
// GET /providers/{id}/replicas
// ---------------------------------------------------------------------------

const REPLICAS_SELECT: &str = "SELECT id, provider_id, component_id, \
    component_version, storage_reference, version, size_bytes, checksum, \
    latency_ms, is_available, last_checked_at, created_at \
    FROM provider_replicas WHERE provider_id = $1 ORDER BY component_id, component_version";

pub async fn list_replicas(
    State(state): State<AppState>,
    Path(provider_id): Path<Uuid>,
) -> Result<Json<Vec<ReplicaRow>>, AppError> {
    let rows = sqlx::query_as::<_, ReplicaRow>(REPLICAS_SELECT)
        .bind(provider_id)
        .fetch_all(&state.pool)
        .await
        .map_err(|e| AppError::internal(format!("query failed: {e}")))?;

    Ok(Json(rows))
}

// ---------------------------------------------------------------------------
// POST /providers/{id}/replicas/{replica_id}/challenge
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct ChallengeResponse {
    pub challenge: String,
}

pub async fn challenge_replica(
    State(state): State<AppState>,
    Path((_provider_id, replica_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<ChallengeResponse>, AppError> {
    // Verify replica exists.
    let _replica: (Uuid,) = sqlx::query_as("SELECT id FROM provider_replicas WHERE id = $1")
        .bind(replica_id)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| AppError::internal(format!("query failed: {e}")))?
        .ok_or_else(|| AppError::not_found(format!("replica {replica_id} not found")))?;

    // Generate a random challenge.
    let challenge = Uuid::new_v4().to_string();

    // Store the challenge and clear any previous proof timestamp.
    sqlx::query(
        "UPDATE provider_replicas SET last_challenge = $1, last_proof_at = NULL WHERE id = $2",
    )
    .bind(&challenge)
    .bind(replica_id)
    .execute(&state.pool)
    .await
    .map_err(|e| AppError::internal(format!("store challenge failed: {e}")))?;

    Ok(Json(ChallengeResponse { challenge }))
}

// ---------------------------------------------------------------------------
// POST /providers/{id}/replicas/{replica_id}/prove
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct ProveRequest {
    pub proof: String,
}

pub async fn prove_replica(
    State(state): State<AppState>,
    Path((_provider_id, replica_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<ProveRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    // Verify replica exists and has a pending challenge.
    let (content_address, last_challenge): (String, Option<String>) = sqlx::query_as(
        "SELECT content_address, last_challenge FROM provider_replicas WHERE id = $1",
    )
    .bind(replica_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| AppError::internal(format!("query failed: {e}")))?
    .ok_or_else(|| AppError::not_found(format!("replica {replica_id} not found")))?;

    let challenge = last_challenge
        .ok_or_else(|| AppError::bad_request("no pending challenge; call /challenge first"))?;

    // Compute expected proof: SHA-256(challenge + content_address).
    let expected_proof = format!(
        "{:x}",
        Sha256::digest(format!("{challenge}{content_address}").as_bytes())
    );

    if req.proof != expected_proof {
        return Err(AppError::bad_request(
            "proof does not match expected digest",
        ));
    }

    // Update last_proof_at and clear the challenge.
    sqlx::query(
        "UPDATE provider_replicas SET last_proof_at = now(), last_challenge = NULL WHERE id = $1",
    )
    .bind(replica_id)
    .execute(&state.pool)
    .await
    .map_err(|e| AppError::internal(format!("update failed: {e}")))?;

    Ok(Json(serde_json::json!({
        "status": "verified",
        "replica_id": replica_id,
        "proven_at": chrono::Utc::now(),
    })))
}
