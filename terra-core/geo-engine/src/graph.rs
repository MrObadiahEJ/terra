use geo::prelude::BoundingRect;
use geo::{Coord, Haversine, Length, LineString, Rect};

use crate::osm::{OsmData, RoadWay};

/// A single linear road segment with resolved coordinates and node topology.
#[derive(Debug, Clone)]
pub struct RoadSegment {
    pub id: i64,
    pub name: Option<String>,
    pub highway: String,
    pub oneway: bool,
    /// OSM node id at the start of the segment (from the way's first node ref).
    pub from_node_id: i64,
    /// OSM node id at the end of the segment (from the way's last node ref).
    pub to_node_id: i64,
    pub line: LineString<f64>,
    pub length_m: f64,
}

/// The road network graph for an area.
#[derive(Debug, Default)]
pub struct RoadGraph {
    pub segments: Vec<RoadSegment>,
    pub bbox: Option<Rect<f64>>,
}

impl RoadGraph {
    pub fn total_length_m(&self) -> f64 {
        self.segments.iter().map(|s| s.length_m).sum()
    }

    pub fn segment_count(&self) -> usize {
        self.segments.len()
    }
}

/// Build a road graph from parsed OSM data, dropping ways with no resolvable geometry.
pub fn build_graph(data: &OsmData) -> RoadGraph {
    let mut graph = RoadGraph::default();
    let mut bbox: Option<Rect<f64>> = None;

    for way in &data.roads {
        if let Some(segment) = build_segment(way, data) {
            bbox = merge_bbox(bbox, segment.line.bounding_rect());
            graph.segments.push(segment);
        }
    }

    graph.bbox = bbox;
    graph
}

fn build_segment(way: &RoadWay, data: &OsmData) -> Option<RoadSegment> {
    let coords: Vec<Coord<f64>> = way
        .nodes
        .iter()
        .filter_map(|id| data.nodes.get(id).copied())
        .collect();

    if coords.len() < 2 {
        return None;
    }

    let line = LineString::new(coords);
    let length_m = line.length::<Haversine>();

    let from_node_id = *way.nodes.first()?;
    let to_node_id = *way.nodes.last()?;

    Some(RoadSegment {
        id: way.id,
        name: way.name.clone(),
        highway: way.highway.clone(),
        oneway: way.oneway,
        from_node_id,
        to_node_id,
        line,
        length_m,
    })
}

fn merge_bbox(current: Option<Rect<f64>>, incoming: Option<Rect<f64>>) -> Option<Rect<f64>> {
    match (current, incoming) {
        (Some(a), Some(b)) => Some(Rect::new(
            Coord {
                x: a.min().x.min(b.min().x),
                y: a.min().y.min(b.min().y),
            },
            Coord {
                x: a.max().x.max(b.max().x),
                y: a.max().y.max(b.max().y),
            },
        )),
        (a, b) => a.or(b),
    }
}
