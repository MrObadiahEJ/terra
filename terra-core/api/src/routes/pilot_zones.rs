use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::error::AppError;
use crate::state::AppState;

const ASSET_TYPES: &[&str] = &["orthophoto", "point_cloud", "dsm", "dtm", "mesh", "other"];

#[derive(Debug, Deserialize)]
pub struct NewPilotZone {
    pub name: String,
    pub description: Option<String>,
    /// GeoJSON geometry (Polygon) in EPSG:4326.
    pub geometry: serde_json::Value,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct PilotZoneRow {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub geometry: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct NewAsset {
    pub asset_type: String,
    pub name: String,
    pub format: Option<String>,
    pub file_path: Option<String>,
    pub resolution_m: Option<f64>,
    pub point_count: Option<i64>,
    pub metadata: Option<Value>,
    pub geometry: Option<Value>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct AssetRow {
    pub id: Uuid,
    pub pilot_zone_id: Uuid,
    pub asset_type: String,
    pub name: String,
    pub format: Option<String>,
    pub file_path: Option<String>,
    pub resolution_m: Option<f64>,
    pub point_count: Option<i64>,
    pub metadata: Option<Value>,
    pub geometry: Option<String>,
    pub created_at: DateTime<Utc>,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", post(create_pilot_zone))
        .route("/", get(list_pilot_zones))
        .route("/{id}", get(get_pilot_zone))
        .route("/{id}/assets", get(list_assets))
        .route("/{id}/assets", post(create_asset))
}

async fn create_pilot_zone(
    State(state): State<AppState>,
    Json(params): Json<NewPilotZone>,
) -> Result<(axum::http::StatusCode, Json<PilotZoneRow>), AppError> {
    let geojson = params.geometry.to_string();
    let zone = sqlx::query_as::<_, PilotZoneRow>(
        "INSERT INTO pilot_zones (name, description, geometry)
         VALUES ($1, $2, ST_GeomFromGeoJSON($3))
         RETURNING id, name, description, ST_AsGeoJSON(geometry)::text AS geometry, created_at",
    )
    .bind(&params.name)
    .bind(&params.description)
    .bind(&geojson)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| {
        if matches!(
            e,
            sqlx::Error::Database(ref de)
                if matches!(de.code().as_deref(), Some("XX000") | Some("22023") | Some("22P02"))
        ) {
            AppError::bad_request("geometry must be a valid GeoJSON Polygon in EPSG:4326")
        } else {
            AppError::from(e)
        }
    })?;
    Ok((axum::http::StatusCode::CREATED, Json(zone)))
}

async fn list_pilot_zones(
    State(state): State<AppState>,
) -> Result<Json<Vec<PilotZoneRow>>, AppError> {
    let rows = sqlx::query_as::<_, PilotZoneRow>(
        "SELECT id, name, description, ST_AsGeoJSON(geometry)::text AS geometry, created_at
         FROM pilot_zones ORDER BY created_at DESC",
    )
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(rows))
}

async fn get_pilot_zone(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<PilotZoneRow>, AppError> {
    let row = sqlx::query_as::<_, PilotZoneRow>(
        "SELECT id, name, description, ST_AsGeoJSON(geometry)::text AS geometry, created_at
         FROM pilot_zones WHERE id = $1",
    )
    .bind(id)
    .fetch_one(&state.pool)
    .await?;
    Ok(Json(row))
}

async fn list_assets(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<AssetRow>>, AppError> {
    let rows = sqlx::query_as::<_, AssetRow>(
        "SELECT id, pilot_zone_id, asset_type, name, format, file_path, resolution_m,
                point_count, metadata, ST_AsGeoJSON(geometry)::text AS geometry, created_at
         FROM photogrammetry_assets WHERE pilot_zone_id = $1 ORDER BY created_at DESC",
    )
    .bind(id)
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(rows))
}

async fn create_asset(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(params): Json<NewAsset>,
) -> Result<(axum::http::StatusCode, Json<AssetRow>), AppError> {
    if !ASSET_TYPES.contains(&params.asset_type.as_str()) {
        return Err(AppError::bad_request(format!(
            "asset_type must be one of: {}",
            ASSET_TYPES.join(", ")
        )));
    }
    if params.name.is_empty() {
        return Err(AppError::bad_request("name is required"));
    }

    let geojson = params.geometry.map(|g| g.to_string());
    let metadata = params.metadata.unwrap_or_else(|| json!({}));

    let row = sqlx::query_as::<_, AssetRow>(
        "INSERT INTO photogrammetry_assets
            (pilot_zone_id, asset_type, name, format, file_path, resolution_m, point_count, metadata, geometry)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8::jsonb,
                 CASE WHEN $9::text IS NOT NULL THEN ST_GeomFromGeoJSON($9) ELSE NULL END)
         RETURNING id, pilot_zone_id, asset_type, name, format, file_path, resolution_m,
                   point_count, metadata, ST_AsGeoJSON(geometry)::text AS geometry, created_at",
    )
    .bind(id)
    .bind(&params.asset_type)
    .bind(&params.name)
    .bind(&params.format)
    .bind(&params.file_path)
    .bind(params.resolution_m)
    .bind(params.point_count)
    .bind(metadata)
    .bind(&geojson)
    .fetch_one(&state.pool)
    .await?;

    Ok((axum::http::StatusCode::CREATED, Json(row)))
}
