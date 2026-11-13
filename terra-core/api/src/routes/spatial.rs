use axum::extract::{Path, Query, State};
use axum::routing::get;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use terra_geo::{covering_cells as geo_covering_cells, GeoBounds, QuadCell};
use uuid::Uuid;

use crate::error::AppError;
use crate::state::AppState;

// ---------------------------------------------------------------------------
// PostGIS spatial architecture endpoints.
//
// Canonical facts come from the 0017 views:
//   parcel_spatial_stats (area, centroid, bbox per parcel)
//   zone_parcel_counts   (count + total area per pilot zone)
// Radius search uses the maintained `parcels.centroid` column; zone
// membership uses ST_Intersects against `pilot_zones.geometry`.
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct NearParams {
    pub lon: f64,
    pub lat: f64,
    #[serde(default = "default_radius")]
    pub radius_m: f64,
    #[serde(default = "default_limit")]
    pub limit: i64,
}

fn default_radius() -> f64 {
    1000.0
}

fn default_limit() -> i64 {
    20
}

fn validate_lon_lat(lon: f64, lat: f64) -> Result<(), AppError> {
    if !(-180.0..=180.0).contains(&lon) || !(-90.0..=90.0).contains(&lat) {
        return Err(AppError::bad_request("coordinate out of EPSG:4326 range"));
    }
    Ok(())
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ParcelSpatialStats {
    pub id: Uuid,
    pub name: String,
    pub holder: String,
    pub status: String,
    pub onchain_id: Option<String>,
    pub area_m2: Option<f64>,
    pub geometry: Option<String>,
    pub centroid: Option<String>,
    pub minx: Option<f64>,
    pub miny: Option<f64>,
    pub maxx: Option<f64>,
    pub maxy: Option<f64>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct NearParcel {
    pub id: Uuid,
    pub name: String,
    pub holder: String,
    pub status: String,
    pub area_m2: Option<f64>,
    pub distance_m: Option<f64>,
    pub centroid: Option<String>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ZoneParcelCount {
    pub zone_id: Uuid,
    pub zone_name: String,
    pub parcel_count: Option<i64>,
    pub total_area_m2: Option<f64>,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/cell", get(cell_at))
        .route("/cover", get(cover_cells))
        .route("/parcels/near", get(parcels_near))
        .route("/parcels/{id}/stats", get(parcel_stats))
        .route("/zones/stats", get(zone_stats))
        .route("/zones/{zone_id}/parcels", get(parcels_within_zone))
}

// ---------------------------------------------------------------------------
// Quadtree discovery cells (Chapter 12 §12.4, ADR-020).
//
// These are control-plane partition lookups, not PostGIS proximity queries.
// They do not touch the database and are safe when the API runs without a
// populated geo mirror.
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct CellParams {
    pub lon: f64,
    pub lat: f64,
    #[serde(default = "default_cell_level")]
    pub level: u8,
}

fn default_cell_level() -> u8 {
    12
}

#[derive(Debug, Serialize)]
pub struct CellBoundsResponse {
    pub min_lon: f64,
    pub min_lat: f64,
    pub max_lon: f64,
    pub max_lat: f64,
}

#[derive(Debug, Serialize)]
pub struct CellResponse {
    pub id: String,
    pub level: u8,
    pub x: u32,
    pub y: u32,
    pub bounds: CellBoundsResponse,
}

impl From<QuadCell> for CellResponse {
    fn from(cell: QuadCell) -> Self {
        let bounds = cell.bounds();
        Self {
            id: cell.to_string(),
            level: cell.level(),
            x: cell.x(),
            y: cell.y(),
            bounds: CellBoundsResponse {
                min_lon: bounds.min_lon,
                min_lat: bounds.min_lat,
                max_lon: bounds.max_lon,
                max_lat: bounds.max_lat,
            },
        }
    }
}

/// `GET /spatial/cell?lon=&lat=&level=` — the quadtree cell containing a point.
pub async fn cell_at(Query(params): Query<CellParams>) -> Result<Json<CellResponse>, AppError> {
    validate_lon_lat(params.lon, params.lat)?;
    let cell = QuadCell::from_point(params.lon, params.lat, params.level)
        .map_err(|e| AppError::bad_request(format!("invalid cell request: {e}")))?;
    Ok(Json(cell.into()))
}

#[derive(Debug, Deserialize)]
pub struct CoverParams {
    pub min_lon: f64,
    pub min_lat: f64,
    pub max_lon: f64,
    pub max_lat: f64,
    #[serde(default = "default_cover_level")]
    pub level: u8,
}

fn default_cover_level() -> u8 {
    10
}

#[derive(Debug, Serialize)]
pub struct CoverResponse {
    pub level: u8,
    pub count: usize,
    pub cells: Vec<CellResponse>,
}

/// `GET /spatial/cover?min_lon=&min_lat=&max_lon=&max_lat=&level=` — the
/// quadtree cells intersecting a bounds rectangle, in south-to-north,
/// west-to-east order.
pub async fn cover_cells(
    Query(params): Query<CoverParams>,
) -> Result<Json<CoverResponse>, AppError> {
    let bounds = GeoBounds::new(
        params.min_lon,
        params.min_lat,
        params.max_lon,
        params.max_lat,
    )
    .map_err(|e| AppError::bad_request(format!("invalid bounds: {e}")))?;
    let cells = geo_covering_cells(&bounds, params.level)
        .map_err(|e| AppError::bad_request(format!("invalid cover request: {e}")))?;
    let count = cells.len();
    Ok(Json(CoverResponse {
        level: params.level,
        count,
        cells: cells.into_iter().map(CellResponse::from).collect(),
    }))
}

/// Parcels whose centroid falls within `radius_m` of (lon, lat),
/// ordered by distance. Uses the maintained centroid + GIST index.
async fn parcels_near(
    State(state): State<AppState>,
    Query(params): Query<NearParams>,
) -> Result<Json<Vec<NearParcel>>, AppError> {
    validate_lon_lat(params.lon, params.lat)?;
    if !(params.radius_m > 0.0 && params.radius_m <= 100_000.0) {
        return Err(AppError::bad_request("radius_m must be in (0, 100000]"));
    }
    let limit = params.limit.clamp(1, 100);

    let rows: Vec<NearParcel> = sqlx::query_as(
        "SELECT
            p.id, p.name, o.holder, p.status,
            ST_Area(p.geometry::geography)::float8 AS area_m2,
            ST_Distance(p.centroid::geography, ST_SetSRID(ST_MakePoint($1, $2), 4326)::geography)::float8 AS distance_m,
            ST_AsGeoJSON(p.centroid)::text AS centroid
         FROM parcels p
         JOIN parcel_ownership o ON o.parcel_id = p.id
         WHERE p.centroid IS NOT NULL
           AND ST_DWithin(p.centroid::geography, ST_SetSRID(ST_MakePoint($1, $2), 4326)::geography, $3)
         ORDER BY distance_m ASC
         LIMIT $4",
    )
    .bind(params.lon)
    .bind(params.lat)
    .bind(params.radius_m)
    .bind(limit)
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(rows))
}

/// Canonical spatial facts for one parcel (area, centroid, bbox).
async fn parcel_stats(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ParcelSpatialStats>, AppError> {
    let row: ParcelSpatialStats = sqlx::query_as(
        "SELECT id, name, holder, status, onchain_id, area_m2, geometry,
                centroid, minx, miny, maxx, maxy
         FROM parcel_spatial_stats WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::not_found("parcel not found"))?;
    Ok(Json(row))
}

/// Parcel count + total area per pilot zone.
async fn zone_stats(State(state): State<AppState>) -> Result<Json<Vec<ZoneParcelCount>>, AppError> {
    let rows: Vec<ZoneParcelCount> = sqlx::query_as(
        "SELECT zone_id, zone_name, parcel_count, total_area_m2 FROM zone_parcel_counts",
    )
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(rows))
}

/// Parcels intersecting a pilot zone, ordered by area descending.
async fn parcels_within_zone(
    State(state): State<AppState>,
    Path(zone_id): Path<Uuid>,
) -> Result<Json<Vec<ParcelSpatialStats>>, AppError> {
    let rows: Vec<ParcelSpatialStats> = sqlx::query_as(
        "SELECT s.id, s.name, s.holder, s.status, s.onchain_id, s.area_m2,
                s.geometry, s.centroid, s.minx, s.miny, s.maxx, s.maxy
         FROM parcel_spatial_stats s
         JOIN parcels p ON p.id = s.id
         JOIN pilot_zones z ON z.id = $1 AND ST_Intersects(p.geometry, z.geometry)
         ORDER BY s.area_m2 DESC NULLS LAST",
    )
    .bind(zone_id)
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(rows))
}

// ---------------------------------------------------------------------------
// Tests (pure validation logic; DB-backed queries are covered by CI PostGIS)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lon_lat_validation() {
        assert!(validate_lon_lat(11.6, 3.98).is_ok());
        assert!(validate_lon_lat(200.0, 0.0).is_err());
        assert!(validate_lon_lat(0.0, -100.0).is_err());
    }

    #[test]
    fn radius_bounds() {
        fn ok(r: f64) -> bool {
            r > 0.0 && r <= 100_000.0
        }
        assert!(ok(1000.0));
        assert!(!ok(0.0));
        assert!(!ok(-5.0));
        assert!(!ok(200_000.0));
    }

    #[test]
    fn limit_clamped_1_to_100() {
        assert_eq!(0i64.clamp(1, 100), 1);
        assert_eq!(500i64.clamp(1, 100), 100);
        assert_eq!(20i64.clamp(1, 100), 20);
    }

    #[test]
    fn default_radius_and_limit() {
        assert_eq!(default_radius(), 1000.0);
        assert_eq!(default_limit(), 20);
    }

    #[tokio::test]
    async fn cell_endpoint_returns_stable_cell() {
        let Json(cell) = cell_at(Query(CellParams {
            lon: 11.5,
            lat: 3.85,
            level: 8,
        }))
        .await
        .unwrap();
        assert_eq!(cell.level, 8);
        assert_eq!(cell.id, format!("q/8/{}/{}", cell.x, cell.y));
        assert!(cell.bounds.min_lon <= 11.5 && 11.5 <= cell.bounds.max_lon);
        assert!(cell.bounds.min_lat <= 3.85 && 3.85 <= cell.bounds.max_lat);
    }

    #[tokio::test]
    async fn cell_endpoint_rejects_bad_level_and_coordinates() {
        let err = cell_at(Query(CellParams {
            lon: 0.0,
            lat: 0.0,
            level: 31,
        }))
        .await
        .unwrap_err();
        assert!(matches!(err, AppError::BadRequest(_)));

        let err = cell_at(Query(CellParams {
            lon: 200.0,
            lat: 0.0,
            level: 4,
        }))
        .await
        .unwrap_err();
        assert!(matches!(err, AppError::BadRequest(_)));
    }

    #[tokio::test]
    async fn cover_endpoint_returns_ordered_cells() {
        let Json(cover) = cover_cells(Query(CoverParams {
            min_lon: 11.49,
            min_lat: 3.84,
            max_lon: 11.53,
            max_lat: 3.88,
            level: 10,
        }))
        .await
        .unwrap();
        assert!(cover.count > 0);
        assert_eq!(cover.count, cover.cells.len());
        assert!(cover
            .cells
            .windows(2)
            .all(|pair| (pair[0].y, pair[0].x) <= (pair[1].y, pair[1].x)));
    }

    #[tokio::test]
    async fn cover_endpoint_rejects_oversized_query() {
        let err = cover_cells(Query(CoverParams {
            min_lon: -180.0,
            min_lat: -90.0,
            max_lon: 180.0,
            max_lat: 90.0,
            level: 20,
        }))
        .await
        .unwrap_err();
        assert!(matches!(err, AppError::BadRequest(_)));
    }
}
