use axum::extract::{Query, State};
use axum::routing::{get, post};
use axum::Json;
use axum::Router;
use chrono::{DateTime, Utc};
use geo::LineString;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::{Postgres, QueryBuilder};

use crate::error::AppError;
use crate::geoutil::geojson_polygon;
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

    let geo = std::sync::Arc::clone(geo);
    let report = tokio::task::spawn_blocking(move || {
        terra_geo::analyze(&geo.network, &geo.graph, &polygon, &id_bytes)
    })
    .await
    .map_err(|error| AppError::Internal(error.to_string()))?;

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
    for batch in geo.graph.segments.chunks(UPSERT_BATCH_SIZE) {
        let result = road_upsert_query(batch).build().execute(&mut *tx).await?;
        road_count += result.rows_affected() as i64;
    }

    let mut poi_count = 0i64;
    for batch in geo.data.pois.chunks(UPSERT_BATCH_SIZE) {
        let mut query = poi_upsert_query(batch)?;
        let result = query.build().execute(&mut *tx).await?;
        poi_count += result.rows_affected() as i64;
    }

    tx.commit().await?;

    Ok(Json(json!({
        "roads_upserted": road_count,
        "pois_upserted": poi_count,
    })))
}

const UPSERT_BATCH_SIZE: usize = 500;

fn road_upsert_query<'a>(segments: &'a [terra_geo::RoadSegment]) -> QueryBuilder<'a, Postgres> {
    let mut query =
        QueryBuilder::new("INSERT INTO roads (id, name, highway, oneway, geometry, length_m) ");
    query.push_values(segments, |mut row, segment| {
        row.push_bind(segment.id)
            .push_bind(segment.name.as_deref())
            .push_bind(segment.highway.as_str())
            .push_bind(segment.oneway)
            .push_unseparated(", ST_GeomFromText(")
            .push_bind_unseparated(linestring_wkt(&segment.line))
            .push_unseparated(", 4326)")
            .push_bind(segment.length_m);
    });
    query.push(
        " ON CONFLICT (id) DO UPDATE SET
            name = EXCLUDED.name,
            highway = EXCLUDED.highway,
            oneway = EXCLUDED.oneway,
            geometry = EXCLUDED.geometry,
            length_m = EXCLUDED.length_m",
    );
    query
}

fn poi_upsert_query<'a>(
    pois: &'a [terra_geo::Poi],
) -> Result<QueryBuilder<'a, Postgres>, AppError> {
    let tags = pois
        .iter()
        .map(|poi| serde_json::to_value(&poi.tags))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| AppError::bad_request(format!("invalid poi tags: {error}")))?;
    let mut query =
        QueryBuilder::new("INSERT INTO pois (id, name, category, kind, tags, geometry) ");
    query.push_values(pois.iter().zip(tags), |mut row, (poi, tags)| {
        row.push_bind(poi.id)
            .push_bind(poi.name.as_deref())
            .push_bind(poi.category.as_str())
            .push_bind(poi.kind.as_str())
            .push_bind(tags)
            .push_unseparated("::jsonb, ST_GeomFromText(")
            .push_bind_unseparated(point_wkt(poi.coord.x, poi.coord.y))
            .push_unseparated(", 4326)");
    });
    query.push(
        " ON CONFLICT (id) DO UPDATE SET
            name = EXCLUDED.name,
            category = EXCLUDED.category,
            kind = EXCLUDED.kind,
            tags = EXCLUDED.tags,
            geometry = EXCLUDED.geometry",
    );
    Ok(query)
}

async fn list_roads(
    State(state): State<AppState>,
    Query(params): Query<BboxParams>,
) -> Result<Json<Vec<RoadRow>>, AppError> {
    let (sql, rows): (String, Vec<RoadRow>) =
        match (params.minx, params.miny, params.maxx, params.maxy) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use geo::{line_string, Coord};
    use sqlx::Execute as _;
    use std::collections::HashMap;

    #[test]
    fn roads_are_encoded_as_a_bounded_multi_row_upsert() {
        let segments = vec![
            terra_geo::RoadSegment {
                id: 1,
                name: Some("First".into()),
                highway: "residential".into(),
                oneway: false,
                line: line_string![(x: 0.0, y: 0.0), (x: 1.0, y: 1.0)],
                length_m: 100.0,
            },
            terra_geo::RoadSegment {
                id: 2,
                name: None,
                highway: "track".into(),
                oneway: true,
                line: line_string![(x: 2.0, y: 2.0), (x: 3.0, y: 3.0)],
                length_m: 200.0,
            },
        ];
        let sql = road_upsert_query(&segments).build().sql().to_owned();

        assert!(sql.starts_with("INSERT INTO roads"));
        assert!(
            sql.contains("($1, $2, $3, $4, ST_GeomFromText($5, 4326), $6)"),
            "{sql}"
        );
        assert!(sql.contains("($7, $8, $9, $10, ST_GeomFromText($11, 4326), $12)"));
        assert!(sql.contains("ON CONFLICT (id) DO UPDATE"));
        assert!(UPSERT_BATCH_SIZE * 6 < 65_535);
    }

    #[test]
    fn pois_are_encoded_as_jsonb_multi_row_upserts() {
        let pois = vec![terra_geo::Poi {
            id: 3,
            name: Some("Clinic".into()),
            category: "amenity".into(),
            kind: "clinic".into(),
            tags: HashMap::from([("amenity".into(), "clinic".into())]),
            coord: Coord { x: 1.0, y: 2.0 },
        }];
        let query = poi_upsert_query(&pois).unwrap();
        let sql = query.sql().to_owned();

        assert!(sql.contains("VALUES ($1, $2, $3, $4, $5::jsonb, ST_GeomFromText($6, 4326))"));
        assert!(sql.contains("ON CONFLICT (id) DO UPDATE"));
    }
}
