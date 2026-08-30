use geo::algorithm::line_intersection::{line_intersection, LineIntersection};
use geo::{Closest, ClosestPoint, Coord, Distance, Haversine, Line, LineString, Point, Polygon};

use crate::graph::RoadGraph;

/// A point where a parcel can reach the road network.
#[derive(Debug, Clone, PartialEq)]
pub struct AccessPoint {
    pub point: Coord<f64>,
    pub distance_m: f64,
    pub road_name: Option<String>,
    pub highway: String,
}

/// Find the `limit` nearest road segments to `origin`, returning the closest
/// point on each road plus its straight-line (haversine) distance.
pub fn nearest_road_access(
    graph: &RoadGraph,
    origin: Coord<f64>,
    limit: usize,
) -> Vec<AccessPoint> {
    let origin_pt = Point::new(origin.x, origin.y);

    let mut hits: Vec<AccessPoint> = graph
        .segments
        .iter()
        .filter_map(|seg| {
            closest_point_on_line(&seg.line, origin_pt).map(|closest| AccessPoint {
                point: closest,
                distance_m: Haversine::distance(origin_pt, Point::new(closest.x, closest.y)),
                road_name: seg.name.clone(),
                highway: seg.highway.clone(),
            })
        })
        .collect();

    hits.sort_by(|a, b| a.distance_m.total_cmp(&b.distance_m));
    hits.truncate(limit);
    hits
}

/// Find where roads cross the boundary of `parcel`. These are the edges a
/// plot physically touches (or is crossed by) the road network.
pub fn road_access_along_boundary(
    graph: &RoadGraph,
    parcel: &Polygon<f64>,
    limit: usize,
) -> Vec<AccessPoint> {
    let boundary = parcel.exterior();
    let boundary_lines: Vec<Line<f64>> = line_string_segments(boundary);

    let mut hits: Vec<AccessPoint> = Vec::new();

    for seg in &graph.segments {
        for road_line in line_string_segments(&seg.line) {
            for boundary_line in &boundary_lines {
                if let Some(point) = line_intersection_point(&road_line, boundary_line) {
                    hits.push(AccessPoint {
                        point,
                        distance_m: 0.0,
                        road_name: seg.name.clone(),
                        highway: seg.highway.clone(),
                    });
                    break; // one hit per boundary edge is enough
                }
            }
        }
    }

    hits.truncate(limit);
    hits
}

/// Closest point on a LineString to `origin`, if any.
fn closest_point_on_line(line: &LineString<f64>, origin: Point<f64>) -> Option<Coord<f64>> {
    match line.closest_point(&origin) {
        Closest::SinglePoint(p) | Closest::Intersection(p) => Some(p.0),
        Closest::Indeterminate => None,
    }
}

/// First point where two lines cross, if they do.
fn line_intersection_point(a: &Line<f64>, b: &Line<f64>) -> Option<Coord<f64>> {
    match line_intersection(*a, *b) {
        Some(LineIntersection::SinglePoint { intersection, .. }) => Some(intersection),
        _ => None,
    }
}

fn line_string_segments(line: &LineString<f64>) -> Vec<Line<f64>> {
    let coords: Vec<Coord<f64>> = line.coords().copied().collect();
    coords
        .windows(2)
        .map(|w| Line::new(w[0], w[1]))
        .collect()
}
