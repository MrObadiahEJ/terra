use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::AppError;
use crate::state::AppState;

const PARCEL_SELECT: &str = r#"
    SELECT
        id,
        name,
        owner,
        status,
        ST_AsGeoJSON(geometry)::text AS geometry,
        ST_Area(geometry::geography)::float8 AS area_m2,
        created_at,
        updated_at
    FROM parcels
"#;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Parcel {
    pub id: Uuid,
    pub name: String,
    pub owner: String,
    pub status: String,
    pub geometry: Option<String>,
    pub area_m2: Option<f64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct ListParams {
    pub minx: Option<f64>,
    pub miny: Option<f64>,
    pub maxx: Option<f64>,
    pub maxy: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct NewParcel {
    pub name: String,
    pub owner: String,
    #[serde(default)]
    pub status: String,
    pub geometry: serde_json::Value,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list))
        .route("/", post(create_parcel))
        .route("/{id}", get(get_by_id))
        .route("/{id}", delete(delete_parcel))
}

async fn list(
    State(state): State<AppState>,
    Query(params): Query<ListParams>,
) -> Result<Json<Vec<Parcel>>, AppError> {
    let parcels = match (params.minx, params.miny, params.maxx, params.maxy) {
        (Some(minx), Some(miny), Some(maxx), Some(maxy)) => sqlx::query_as::<_, Parcel>(&format!(
            "{PARCEL_SELECT}
             WHERE ST_Intersects(geometry, ST_MakeEnvelope($1, $2, $3, $4, 4326))
             ORDER BY created_at DESC"
        ))
        .bind(minx)
        .bind(miny)
        .bind(maxx)
        .bind(maxy)
        .fetch_all(&state.pool)
        .await?,
        _ => sqlx::query_as::<_, Parcel>(&format!(
            "{PARCEL_SELECT}
             ORDER BY created_at DESC"
        ))
        .fetch_all(&state.pool)
        .await?,
    };
    Ok(Json(parcels))
}

async fn get_by_id(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Parcel>, AppError> {
    let parcel = sqlx::query_as::<_, Parcel>(&format!(
        "{PARCEL_SELECT}
         WHERE id = $1"
    ))
    .bind(id)
    .fetch_one(&state.pool)
    .await?;
    Ok(Json(parcel))
}

async fn create_parcel(
    State(state): State<AppState>,
    Json(params): Json<NewParcel>,
) -> Result<(StatusCode, Json<Parcel>), AppError> {
    let geojson = params.geometry.to_string();
    let status = if params.status.is_empty() {
        "pending".to_string()
    } else {
        params.status
    };

    let parcel = sqlx::query_as::<_, Parcel>(&format!(
        "INSERT INTO parcels (name, owner, status, geometry)
         VALUES ($1, $2, $3, ST_GeomFromGeoJSON($4))
         RETURNING id, name, owner, status, ST_AsGeoJSON(geometry)::text AS geometry,
                   ST_Area(geometry::geography)::float8 AS area_m2, created_at, updated_at"
    ))
    .bind(&params.name)
    .bind(&params.owner)
    .bind(&status)
    .bind(&geojson)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| {
        if is_geometry_error(&e) {
            AppError::bad_request("geometry must be a valid GeoJSON Polygon in EPSG:4326")
        } else {
            AppError::from(e)
        }
    })?;

    Ok((StatusCode::CREATED, Json(parcel)))
}

async fn delete_parcel(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let result = sqlx::query("DELETE FROM parcels WHERE id = $1")
        .bind(id)
        .execute(&state.pool)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("parcel not found".into()));
    }
    Ok(StatusCode::NO_CONTENT)
}

fn is_geometry_error(err: &sqlx::Error) -> bool {
    matches!(
        err,
        sqlx::Error::Database(db_err)
            if matches!(
                db_err.code().as_deref(),
                Some("XX000") | Some("22023") | Some("22P02")
            )
    )
}
