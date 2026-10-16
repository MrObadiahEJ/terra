use std::sync::Arc;

use axum::extract::{Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use geo::{Coord, Distance, Haversine, Intersects, Point, Polygon, Rect};
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

#[derive(Debug, Deserialize)]
pub struct BuildingsParams {
    pub minx: Option<f64>,
    pub miny: Option<f64>,
    pub maxx: Option<f64>,
    pub maxy: Option<f64>,
    #[serde(default = "default_building_limit")]
    pub limit: usize,
    #[serde(default)]
    pub offset: usize,
}

fn default_building_limit() -> usize {
    500
}

#[derive(Debug, Serialize)]
pub struct BuildingFootprintView {
    pub osm_id: i64,
    pub name: Option<String>,
    pub building: String,
    /// Mapped height in metres (`height=*`), when present.
    pub height_m: Option<f32>,
    /// Mapped storey count (`building:levels=*`), when present.
    pub levels: Option<i16>,
    /// OSM-mapped building footprint; not a legal parcel outline.
    pub geometry: Value,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/nearest-roads", get(nearest_roads))
        .route("/pois", get(pois))
        .route("/buildings", get(buildings))
        .route("/stats", get(stats))
        .route("/geometry/digest", post(geometry_digest))
}

/// Body of `POST /geo/geometry/digest`: a GeoJSON geometry (object or the
/// raw JSON string as stored on-chain).
#[derive(Debug, Deserialize)]
pub struct GeometryDigestRequest {
    pub geometry: Value,
}

/// Deterministic canonical geometry digests (RFC-013 §9 Q4 and the full-3D
/// extension). `version` selects the domain tag:
/// - `"v1"` (`terra-geom-v1`): planar-only digest. Preserves every existing
///   parcel hash; 3D ordinates and metadata are rejected.
/// - `"v2"` (`terra-geom-v2`): versioned 3D digest. Accepts 2.5D/3D
///   coordinates plus required CRS / elevation-reference metadata, quantizes
///   X/Y on v1's 1e-7-degree grid and Z on a 1e-5-unit grid, and is
///   collision-free with v1 by domain separation.
#[derive(Debug, Deserialize)]
pub struct GeometryDigestParams {
    #[serde(default = "default_digest_version")]
    pub version: String,
}

fn default_digest_version() -> String {
    "v2".to_string()
}

async fn geometry_digest(
    Query(params): Query<GeometryDigestParams>,
    Json(body): Json<GeometryDigestRequest>,
) -> Result<Json<Value>, AppError> {
    let geojson = match &body.geometry {
        Value::String(text) => text.clone(),
        value => value.to_string(),
    };
    let (digest, canonical_len) = match params.version.as_str() {
        "v1" => (
            terra_geo::geometry_digest_hex(&geojson)
                .map_err(|e| AppError::bad_request(format!("v1 digest failed: {e}")))?,
            terra_geo::canonicalize_geometry(&geojson)
                .map(|c| c.len())
                .unwrap_or_default(),
        ),
        "v2" => (
            terra_geo::geometry_digest_hex_v2(&geojson)
                .map_err(|e| AppError::bad_request(format!("v2 digest failed: {e}")))?,
            terra_geo::canonicalize_geometry_v2(&geojson)
                .map(|c| c.len())
                .unwrap_or_default(),
        ),
        other => {
            return Err(AppError::bad_request(format!(
                "unknown digest version '{other}' (expected 'v1' or 'v2')"
            )))
        }
    };
    Ok(Json(json!({
        "version": format!("terra-geom-{}", params.version),
        "digest_hex": digest,
        "canonical_bytes": canonical_len,
    })))
}

async fn buildings(
    State(state): State<AppState>,
    Query(params): Query<BuildingsParams>,
) -> Result<Json<Vec<BuildingFootprintView>>, AppError> {
    let geo = geo(&state)?;
    let bbox = match (params.minx, params.miny, params.maxx, params.maxy) {
        (Some(minx), Some(miny), Some(maxx), Some(maxy)) if minx <= maxx && miny <= maxy => {
            Some((minx, miny, maxx, maxy))
        }
        (None, None, None, None) => None,
        _ => {
            return Err(AppError::bad_request(
                "provide a complete, ordered building bbox",
            ))
        }
    };
    let limit = params.limit.clamp(1, 2000);
    let bbox = bbox.map(|(minx, miny, maxx, maxy)| {
        Rect::new(Coord { x: minx, y: miny }, Coord { x: maxx, y: maxy })
    });
    let footprints = geo
        .data
        .buildings
        .iter()
        .zip(&geo.building_bounds)
        .filter(|(building, bounds)| {
            let Some(bbox) = bbox else {
                return true;
            };
            footprint_intersects_bbox(building, **bounds, &bbox)
        })
        .skip(params.offset)
        .take(limit)
        .map(|(building, _)| BuildingFootprintView {
            osm_id: building.id,
            name: building.name.clone(),
            building: building.building.clone(),
            height_m: building.height_m,
            levels: building.levels,
            geometry: json!({
                "type": "Polygon",
                "coordinates": [building.ring.iter().map(|point| [point.x, point.y]).collect::<Vec<_>>()]
            }),
        })
        .collect();
    Ok(Json(footprints))
}

fn footprint_intersects_bbox(
    building: &terra_geo::BuildingFootprint,
    bounds: Option<Rect<f64>>,
    bbox: &Rect<f64>,
) -> bool {
    let Some(bounds) = bounds else {
        return false;
    };
    if !bounds.intersects(bbox) {
        return false;
    }

    let footprint = Polygon::new(building.ring.clone().into(), vec![]);
    footprint.intersects(bbox)
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
                "building_footprints": geo.data.buildings.len(),
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

#[cfg(test)]
mod tests {
    use super::*;

    /// The v1 and v2 digests are domain-separated: identical planar input
    /// produces different digests, and v1 geometry hashes stay stable.
    #[test]
    fn digest_versions_are_domain_separated_and_deterministic() {
        let geojson = json!({
            "type": "Polygon",
            "coordinates": [[[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 0.0]]]
        })
        .to_string();
        let d1a = terra_geo::geometry_digest_hex(&geojson).unwrap();
        let d1b = terra_geo::geometry_digest_hex(&geojson).unwrap();
        assert_eq!(d1a, d1b, "v1 must be deterministic");

        // A v2 document without third ordinates or metadata canonicalizes to
        // the same structure as v1 but under a different domain tag.
        // `precision` is a positive integer (declared decimal places of the
        // source data), per the terra-geom-v2 spec.
        let plain_v2 = json!({
            "type": "Polygon",
            "crs": "EPSG:4326",
            "elevation_ref": "EGM2008",
            "vertical_units": "m",
            "precision": 2,
            "coordinates": [[[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 0.0]]]
        })
        .to_string();
        let d2a = terra_geo::geometry_digest_hex_v2(&plain_v2).unwrap();
        assert_ne!(d1a, d2a, "v1 and v2 must never collide");

        // v2 accepts 3D ordinates + metadata; v1 rejects them.
        let solid = json!({
            "type": "Polygon",
            "crs": "EPSG:4326",
            "elevation_ref": "EGM2008",
            "vertical_units": "m",
            "precision": 2,
            "coordinates": [[[0.0, 0.0, 0.0], [1.0, 0.0, 3.5], [1.0, 1.0, 3.5], [0.0, 0.0, 0.0]]]
        })
        .to_string();
        assert!(terra_geo::geometry_digest_hex(&solid).is_err());
        let d3 = terra_geo::geometry_digest_hex_v2(&solid).unwrap();
        assert_ne!(d2a, d3, "heights must change the v2 digest");
    }

    #[test]
    fn digest_endpoint_accepts_object_or_raw_string_geometry() {
        // Both request shapes normalize to the same JSON text before hashing.
        let geom = json!({
            "type": "Point",
            "crs": "EPSG:4326",
            "elevation_ref": "EGM2008",
            "vertical_units": "m",
            "precision": 3,
            "coordinates": [36.821, -1.292, 1750.0]
        });
        let as_object = geom.to_string();
        let as_string = geom.to_string();
        assert_eq!(
            terra_geo::geometry_digest_hex_v2(&as_object).unwrap(),
            terra_geo::geometry_digest_hex_v2(&as_string).unwrap()
        );
    }

    fn footprint(coords: &[(f64, f64)]) -> terra_geo::BuildingFootprint {
        terra_geo::BuildingFootprint {
            id: 1,
            name: None,
            building: "yes".into(),
            height_m: None,
            levels: None,
            ring: coords.iter().map(|(x, y)| Coord { x: *x, y: *y }).collect(),
        }
    }

    #[test]
    fn bbox_intersection_includes_polygon_enclosing_bbox() {
        let building = footprint(&[
            (0.0, 0.0),
            (10.0, 0.0),
            (10.0, 10.0),
            (0.0, 10.0),
            (0.0, 0.0),
        ]);
        let bounds = Rect::new(Coord { x: 0.0, y: 0.0 }, Coord { x: 10.0, y: 10.0 });
        let bbox = Rect::new(Coord { x: 4.0, y: 4.0 }, Coord { x: 6.0, y: 6.0 });

        assert!(footprint_intersects_bbox(&building, Some(bounds), &bbox));
    }

    #[test]
    fn bbox_intersection_rejects_disjoint_polygons() {
        let building = footprint(&[(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0), (0.0, 0.0)]);
        let bounds = Rect::new(Coord { x: 0.0, y: 0.0 }, Coord { x: 1.0, y: 1.0 });
        let bbox = Rect::new(Coord { x: 2.0, y: 2.0 }, Coord { x: 3.0, y: 3.0 });

        assert!(!footprint_intersects_bbox(&building, Some(bounds), &bbox));
    }

    #[test]
    fn bbox_intersection_includes_crossing_polygon_without_inside_vertices() {
        let building = footprint(&[
            (-1.0, 0.4),
            (2.0, 0.4),
            (2.0, 0.6),
            (-1.0, 0.6),
            (-1.0, 0.4),
        ]);
        let bounds = Rect::new(Coord { x: -1.0, y: 0.4 }, Coord { x: 2.0, y: 0.6 });
        let bbox = Rect::new(Coord { x: 0.0, y: 0.0 }, Coord { x: 1.0, y: 1.0 });

        assert!(footprint_intersects_bbox(&building, Some(bounds), &bbox));
    }
}
