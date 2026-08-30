use axum::extract::{Query, State};
use axum::routing::{get, post};
use axum::Json;
use axum::Router;
use chrono::{DateTime, Utc};
use geo::{Coord, LineString};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::error::AppError;
use crate::state::AppState;
use sqlx::Connection as _;

#[derive(Debug, Deserialize)]
pub struct BboxParams {
    pub minx: Option<f64>,
    pub miny: Option<f64>,
    pub maxx: Option<f64>,
    pub maxy: Option<f64>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct RoadRow {
    pub id: i64,
    pub name: Option<String>,
    pub highway: String,
    pub oneway: bool,
    pub length_m: f64,
    pub geometry: Option<String>,
    pub ingested_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct PoiRow {
    pub id: i64,
    pub name: Option<String>,
    pub category: String,
    pub kind: String,
    pub tags: Option<Value>,
    pub geometry: Option<String>,
    pub ingested_at: DateTime<Utc>,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/ingest", post(ingest_osm))
        .route("/roads", get(list_roads))
        .route("/pois", get(list_pois))
        .route("/stats", get(stats))
        .route("/reachability", post(reachability))
}

#[derive(Debug, Deserialize)]
pub struct ReachabilityRequest {
    /// 32-byte parcel id as hex.
    pub parcel_id: String,
    /// GeoJSON Polygon in EPSG:4326.
    pub geometry: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct ReachabilityResponse {
    pub nearest_road_m: f64,
    pub boundary_accesses: usize,
    pub component_km: f64,
    pub sealed_reachable: bool,
    pub sealed_network_m: Option<f64>,
    pub flags: u16,
    pub access_hash: String,
}

/// Run the off-chain road-access validation for a parcel and return the
/// derived flags plus the canonical digest to anchor on-chain.
async fn reachability(
    State(state): State<AppState>,
    Json(req): Json<ReachabilityRequest>,
) -> Result<Json<ReachabilityResponse>, AppError> {
    let geo = state
        .geo
        .as_ref()
        .ok_or_else(|| AppError::bad_request("OSM data not loaded (set OSM_PBF_PATH)"))?;

    let parcel_id = hex::decode(&req.parcel_id)
        .map_err(|_| AppError::bad_request("parcel_id must be 32-byte hex"))?;
    if parcel_id.len() != 32 {
        return Err(AppError::bad_request("parcel_id must be 32 bytes"));
    }
    let id_bytes: [u8; 32] = parcel_id
        .try_into()
        .map_err(|_| AppError::bad_request("parcel_id must be 32 bytes"))?;

    let polygon = geojson_polygon(&req.geometry)?;

    let network = terra_geo::NetworkGraph::build(&geo.graph);
    let report = terra_geo::analyze(&network, &geo.graph, &polygon, &id_bytes);

    Ok(Json(ReachabilityResponse {
        nearest_road_m: report.nearest_road_m,
        boundary_accesses: report.boundary_accesses,
        component_km: report.component_km,
        sealed_reachable: report.sealed_reachable,
        sealed_network_m: report.sealed_network_m,
        flags: report.flags,
        access_hash: hex::encode(report.access_hash),
    }))
}

/// Convert a GeoJSON Polygon (EPSG:4326) into a `geo::Polygon`, validating the
/// ring structure and coordinate bounds.
fn geojson_polygon(value: &Value) -> Result<geo::Polygon<f64>, AppError> {
    if value["type"].as_str() != Some("Polygon") {
        return Err(AppError::bad_request("geometry must be a GeoJSON Polygon"));
    }
    let rings = value["coordinates"]
        .as_array()
        .ok_or_else(|| AppError::bad_request("Polygon.coordinates is required"))?;
    let mut parsed = Vec::with_capacity(rings.len());

    for ring in rings {
        let ring = ring
            .as_array()
            .ok_or_else(|| AppError::bad_request("ring must be an array of positions"))?;
        let mut coords: Vec<Coord<f64>> = Vec::with_capacity(ring.len());
        for pos in ring {
            let pos = pos
                .as_array()
                .ok_or_else(|| AppError::bad_request("position must be [lon, lat]"))?;
            let lon = pos
                .first()
                .and_then(|v| v.as_f64())
                .ok_or_else(|| AppError::bad_request("position[0] (lon) required"))?;
            let lat = pos
                .get(1)
                .and_then(|v| v.as_f64())
                .ok_or_else(|| AppError::bad_request("position[1] (lat) required"))?;
            if !(-180.0..=180.0).contains(&lon) || !(-90.0..=90.0).contains(&lat) {
                return Err(AppError::bad_request("coordinate out of EPSG:4326 range"));
            }
            coords.push(Coord { x: lon, y: lat });
        }
        if coords.len() < 4 {
            return Err(AppError::bad_request("ring must have at least 4 positions"));
        }
        if coords[0] != *coords.last().expect("checked len >= 4") {
            coords.push(coords[0]);
        }
        parsed.push(coords.into());
    }

    let exterior = parsed.remove(0);
    let polygon = geo::Polygon::new(
        exterior,
        parsed.into_iter().collect::<Vec<_>>(),
    );
    Ok(polygon)
}

/// Persist the in-memory OSM road graph + POIs into the PostGIS fusion database.
/// Idempotent: rows are upserted by their OSM id.
async fn ingest_osm(State(state): State<AppState>) -> Result<Json<Value>, AppError> {
    let geo = state
        .geo
        .as_ref()
        .ok_or_else(|| AppError::bad_request("OSM data not loaded (set OSM_PBF_PATH)"))?;

    let mut pool = state.pool.acquire().await?;
    let mut tx = pool.begin().await?;

    let mut road_count = 0i64;
    for segment in &geo.graph.segments {
        let wkt = linestring_wkt(&segment.line);
        let res = sqlx::query(
            "INSERT INTO roads (id, name, highway, oneway, geometry, length_m)
             VALUES ($1, $2, $3, $4, ST_GeomFromText($5, 4326), $6)
             ON CONFLICT (id) DO UPDATE SET
               name = EXCLUDED.name,
               highway = EXCLUDED.highway,
               oneway = EXCLUDED.oneway,
               geometry = EXCLUDED.geometry,
               length_m = EXCLUDED.length_m",
        )
        .bind(segment.id)
        .bind(&segment.name)
        .bind(&segment.highway)
        .bind(segment.oneway)
        .bind(&wkt)
        .bind(segment.length_m)
        .execute(&mut *tx)
        .await?;
        road_count += res.rows_affected() as i64;
    }

    let mut poi_count = 0i64;
    for poi in &geo.data.pois {
        let tags = serde_json::to_value(&poi.tags)
            .map_err(|e| AppError::bad_request(format!("invalid poi tags: {e}")))?;
        let wkt = point_wkt(poi.coord.x, poi.coord.y);
        let res = sqlx::query(
            "INSERT INTO pois (id, name, category, kind, tags, geometry)
             VALUES ($1, $2, $3, $4, $5::jsonb, ST_GeomFromText($6, 4326))
             ON CONFLICT (id) DO UPDATE SET
               name = EXCLUDED.name,
               category = EXCLUDED.category,
               kind = EXCLUDED.kind,
               tags = EXCLUDED.tags,
               geometry = EXCLUDED.geometry",
        )
        .bind(poi.id)
        .bind(&poi.name)
        .bind(&poi.category)
        .bind(&poi.kind)
        .bind(tags)
        .bind(&wkt)
        .execute(&mut *tx)
        .await?;
        poi_count += res.rows_affected() as i64;
    }

    tx.commit().await?;

    Ok(Json(json!({
        "roads_upserted": road_count,
        "pois_upserted": poi_count,
    })))
}

async fn list_roads(
    State(state): State<AppState>,
    Query(params): Query<BboxParams>,
) -> Result<Json<Vec<RoadRow>>, AppError> {
    let (sql, rows): (String, Vec<RoadRow>) = match (
        params.minx,
        params.miny,
        params.maxx,
        params.maxy,
    ) {
        (Some(minx), Some(miny), Some(maxx), Some(maxy)) => (
            "SELECT id, name, highway, oneway, length_m,
                    ST_AsGeoJSON(geometry)::text AS geometry, ingested_at
             FROM roads
             WHERE ST_Intersects(geometry, ST_MakeEnvelope($1, $2, $3, $4, 4326))
             ORDER BY length_m DESC"
                .to_string(),
            sqlx::query_as::<_, RoadRow>(
                "SELECT id, name, highway, oneway, length_m,
                        ST_AsGeoJSON(geometry)::text AS geometry, ingested_at
                 FROM roads
                 WHERE ST_Intersects(geometry, ST_MakeEnvelope($1, $2, $3, $4, 4326))
                 ORDER BY length_m DESC",
            )
            .bind(minx)
            .bind(miny)
            .bind(maxx)
            .bind(maxy)
            .fetch_all(&state.pool)
            .await?,
        ),
        _ => (
            "SELECT id, name, highway, oneway, length_m,
                    ST_AsGeoJSON(geometry)::text AS geometry, ingested_at
             FROM roads
             ORDER BY length_m DESC"
                .to_string(),
            sqlx::query_as::<_, RoadRow>(
                "SELECT id, name, highway, oneway, length_m,
                        ST_AsGeoJSON(geometry)::text AS geometry, ingested_at
                 FROM roads
                 ORDER BY length_m DESC",
            )
            .fetch_all(&state.pool)
            .await?,
        ),
    };
    let _ = sql;
    Ok(Json(rows))
}

async fn list_pois(
    State(state): State<AppState>,
    Query(params): Query<BboxParams>,
) -> Result<Json<Vec<PoiRow>>, AppError> {
    match (params.minx, params.miny, params.maxx, params.maxy) {
        (Some(minx), Some(miny), Some(maxx), Some(maxy)) => {
            let rows = sqlx::query_as::<_, PoiRow>(
                "SELECT id, name, category, kind, tags, ST_AsGeoJSON(geometry)::text AS geometry, ingested_at
                 FROM pois
                 WHERE ST_Intersects(geometry, ST_MakeEnvelope($1, $2, $3, $4, 4326))
                 ORDER BY name NULLS LAST",
            )
            .bind(minx)
            .bind(miny)
            .bind(maxx)
            .bind(maxy)
            .fetch_all(&state.pool)
            .await?;
            Ok(Json(rows))
        }
        _ => {
            let rows = sqlx::query_as::<_, PoiRow>(
                "SELECT id, name, category, kind, tags, ST_AsGeoJSON(geometry)::text AS geometry, ingested_at
                 FROM pois
                 ORDER BY name NULLS LAST",
            )
            .fetch_all(&state.pool)
            .await?;
            Ok(Json(rows))
        }
    }
}

async fn stats(State(state): State<AppState>) -> Result<Json<Value>, AppError> {
    let row: (Option<i64>, Option<i64>, Option<i64>, Option<i64>) = sqlx::query_as(
        "SELECT
            (SELECT count(*) FROM roads),
            (SELECT count(*) FROM pois),
            (SELECT count(*) FROM pilot_zones),
            (SELECT count(*) FROM photogrammetry_assets)",
    )
    .fetch_one(&state.pool)
    .await?;
    Ok(Json(json!({
        "roads": row.0.unwrap_or(0),
        "pois": row.1.unwrap_or(0),
        "pilot_zones": row.2.unwrap_or(0),
        "photogrammetry_assets": row.3.unwrap_or(0),
    })))
}

fn linestring_wkt(line: &LineString<f64>) -> String {
    let pts: Vec<String> = line
        .coords()
        .map(|c| format!("{:.6} {:.6}", c.x, c.y))
        .collect();
    format!("LINESTRING({})", pts.join(","))
}

fn point_wkt(lon: f64, lat: f64) -> String {
    format!("POINT({lon:.6} {lat:.6})")
}
