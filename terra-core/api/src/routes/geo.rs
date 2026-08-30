use std::sync::Arc;

use axum::extract::{Query, State};
use axum::routing::get;
use axum::{Json, Router};
use geo::{Coord, Distance, Haversine, Point};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::error::AppError;
use crate::state::{AppState, GeoData};

#[derive(Debug, Deserialize)]
pub struct NearestRoadsParams {
    pub lon: f64,
    pub lat: f64,
    #[serde(default = "default_limit")]
    pub limit: usize,
}

fn default_limit() -> usize {
    5
}

#[derive(Debug, Serialize)]
pub struct RoadAccess {
    pub lon: f64,
    pub lat: f64,
    pub distance_m: f64,
    pub road_name: Option<String>,
    pub highway: String,
}

#[derive(Debug, Deserialize)]
pub struct PoisParams {
    pub lon: f64,
    pub lat: f64,
    #[serde(default = "default_radius")]
    pub radius: f64,
    #[serde(default = "default_limit")]
    pub limit: usize,
    pub category: Option<String>,
}

fn default_radius() -> f64 {
    1000.0
}

#[derive(Debug, Serialize)]
pub struct Poi {
    pub id: i64,
    pub name: Option<String>,
    pub category: String,
    pub kind: String,
    pub lon: f64,
    pub lat: f64,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/nearest-roads", get(nearest_roads))
        .route("/pois", get(pois))
        .route("/stats", get(stats))
}

async fn nearest_roads(
    State(state): State<AppState>,
    Query(params): Query<NearestRoadsParams>,
) -> Result<Json<Vec<RoadAccess>>, AppError> {
    let geo = geo(&state)?;
    let origin = Coord {
        x: params.lon,
        y: params.lat,
    };
    let hits = terra_geo::access::nearest_road_access(&geo.graph, origin, params.limit)
        .into_iter()
        .map(|h| RoadAccess {
            lon: h.point.x,
            lat: h.point.y,
            distance_m: h.distance_m,
            road_name: h.road_name,
            highway: h.highway,
        })
        .collect();
    Ok(Json(hits))
}

async fn pois(
    State(state): State<AppState>,
    Query(params): Query<PoisParams>,
) -> Result<Json<Vec<Poi>>, AppError> {
    let geo = geo(&state)?;
    let center = Coord {
        x: params.lon,
        y: params.lat,
    };

    let mut within = terra_geo::poi::pois_within(&geo.data, center, params.radius);
    within.sort_by(|a, b| {
        let da = Haversine::distance(
            Point::new(a.coord.x, a.coord.y),
            Point::new(center.x, center.y),
        );
        let db = Haversine::distance(
            Point::new(b.coord.x, b.coord.y),
            Point::new(center.x, center.y),
        );
        da.total_cmp(&db)
    });
    if let Some(category) = params.category.as_deref() {
        within.retain(|p| p.category == category);
    }
    within.truncate(params.limit);

    let hits = within
        .into_iter()
        .map(|p| Poi {
            id: p.id,
            name: p.name.clone(),
            category: p.category.clone(),
            kind: p.kind.clone(),
            lon: p.coord.x,
            lat: p.coord.y,
        })
        .collect();
    Ok(Json(hits))
}

async fn stats(State(state): State<AppState>) -> Result<Json<Value>, AppError> {
    match &state.geo {
        Some(geo) => {
            let bbox = geo.graph.bbox.map(|r| {
                json!({
                    "min_lon": r.min().x,
                    "min_lat": r.min().y,
                    "max_lon": r.max().x,
                    "max_lat": r.max().y,
                })
            });
            Ok(Json(json!({
                "nodes": geo.data.nodes.len(),
                "roads": geo.data.roads.len(),
                "road_segments": geo.graph.segment_count(),
                "road_length_km": geo.graph.total_length_m() / 1000.0,
                "pois": geo.data.pois.len(),
                "bbox": bbox,
            })))
        }
        None => Ok(Json(json!({ "loaded": false }))),
    }
}

fn geo(state: &AppState) -> Result<&Arc<GeoData>, AppError> {
    state.geo.as_ref().ok_or_else(|| {
        AppError::bad_request("OSM data not loaded (set OSM_PBF_PATH on the server)")
    })
}
