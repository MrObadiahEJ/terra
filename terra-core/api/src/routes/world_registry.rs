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
pub struct WorldRegistryRow {
    pub id: i64,
    pub pubkey: String,
    pub admin: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct CountryAllocationRow {
    pub id: i64,
    pub world_registry_pubkey: String,
    pub country_code: String,
    pub approved_admin: String,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct GenesisRequestRow {
    pub id: i64,
    pub world_registry_pubkey: String,
    pub genesis_request_pubkey: String,
    pub country_code: String,
    pub requested_by: String,
    pub confirmations_count: i32,
    pub distinct_countries: i32,
    pub finalized: bool,
    pub created_at: chrono::NaiveDateTime,
}

// ---------------------------------------------------------------------------
// Request types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct CreateWorldRegistryRequest {
    pub pubkey: String,
    pub admin: String,
}

#[derive(Debug, Deserialize)]
pub struct AllocateCountryRequest {
    pub country_code: String,
    pub approved_admin: String,
}

// ---------------------------------------------------------------------------
// SQL constants
// ---------------------------------------------------------------------------

const WORLD_REGISTRY_SELECT: &str =
    "SELECT id, pubkey, admin, created_at, updated_at FROM world_registries";
const COUNTRY_ALLOCATION_SELECT: &str =
    "SELECT id, world_registry_pubkey, country_code, approved_admin, created_at FROM country_allocations";
const GENESIS_REQUEST_SELECT: &str =
    "SELECT id, world_registry_pubkey, genesis_request_pubkey, country_code, requested_by, confirmations_count, distinct_countries, finalized, created_at FROM genesis_requests";

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

async fn list_registries(
    State(state): State<AppState>,
) -> Result<Json<Vec<WorldRegistryRow>>, AppError> {
    let rows = sqlx::query_as::<_, WorldRegistryRow>(WORLD_REGISTRY_SELECT)
        .fetch_all(&state.pool)
        .await?;
    Ok(Json(rows))
}

async fn get_registry(
    State(state): State<AppState>,
    Path(pubkey): Path<String>,
) -> Result<Json<WorldRegistryRow>, AppError> {
    let row = sqlx::query_as::<_, WorldRegistryRow>(&format!(
        "{WORLD_REGISTRY_SELECT} WHERE pubkey = $1"
    ))
    .bind(&pubkey)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::not_found("world registry not found"))?;
    Ok(Json(row))
}

async fn create_registry(
    State(state): State<AppState>,
    Json(req): Json<CreateWorldRegistryRequest>,
) -> Result<(axum::http::StatusCode, Json<WorldRegistryRow>), AppError> {
    let row = sqlx::query_as::<_, WorldRegistryRow>(
        "INSERT INTO world_registries (pubkey, admin) VALUES ($1, $2) RETURNING id, pubkey, admin, created_at, updated_at",
    )
    .bind(&req.pubkey)
    .bind(&req.admin)
    .fetch_one(&state.pool)
    .await?;
    Ok((axum::http::StatusCode::CREATED, Json(row)))
}

async fn list_allocations(
    State(state): State<AppState>,
    Path(registry_pubkey): Path<String>,
) -> Result<Json<Vec<CountryAllocationRow>>, AppError> {
    let rows = sqlx::query_as::<_, CountryAllocationRow>(&format!(
        "{COUNTRY_ALLOCATION_SELECT} WHERE world_registry_pubkey = $1 ORDER BY created_at"
    ))
    .bind(&registry_pubkey)
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(rows))
}

async fn create_allocation(
    State(state): State<AppState>,
    Path(registry_pubkey): Path<String>,
    Json(req): Json<AllocateCountryRequest>,
) -> Result<(axum::http::StatusCode, Json<CountryAllocationRow>), AppError> {
    let row = sqlx::query_as::<_, CountryAllocationRow>(
        "INSERT INTO country_allocations (world_registry_pubkey, country_code, approved_admin) VALUES ($1, $2, $3) RETURNING id, world_registry_pubkey, country_code, approved_admin, created_at",
    )
    .bind(&registry_pubkey)
    .bind(&req.country_code)
    .bind(&req.approved_admin)
    .fetch_one(&state.pool)
    .await?;
    Ok((axum::http::StatusCode::CREATED, Json(row)))
}

async fn list_genesis_requests(
    State(state): State<AppState>,
) -> Result<Json<Vec<GenesisRequestRow>>, AppError> {
    let rows = sqlx::query_as::<_, GenesisRequestRow>(GENESIS_REQUEST_SELECT)
        .fetch_all(&state.pool)
        .await?;
    Ok(Json(rows))
}

async fn get_genesis_request(
    State(state): State<AppState>,
    Path(country_code): Path<String>,
) -> Result<Json<GenesisRequestRow>, AppError> {
    let row = sqlx::query_as::<_, GenesisRequestRow>(&format!(
        "{GENESIS_REQUEST_SELECT} WHERE country_code = $1"
    ))
    .bind(&country_code)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::not_found("genesis request not found"))?;
    Ok(Json(row))
}

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_registries).post(create_registry))
        .route("/{pubkey}", get(get_registry))
        .route("/{pubkey}/allocations", get(list_allocations).post(create_allocation))
        .route("/genesis-requests", get(list_genesis_requests))
        .route("/genesis-requests/{country_code}", get(get_genesis_request))
}
