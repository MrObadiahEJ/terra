use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet, VecDeque};

use geo::{Centroid as _, Coord, Distance, Haversine, Point};
use sha2::{Digest, Sha256};

use crate::graph::RoadGraph;
use crate::access::AccessPoint;

/// Road classes considered "sealed" (connect to the wider paved network).
pub fn is_sealed_highway(highway: &str) -> bool {
    matches!(
        highway,
        "motorway"
            | "motorway_link"
            | "trunk"
            | "trunk_link"
            | "primary"
            | "primary_link"
            | "secondary"
            | "secondary_link"
            | "tertiary"
            | "tertiary_link"
    )
}

/// Maximum straight-line distance from parcel edge to a road that still counts
/// as having physical road access.
pub const ROAD_ACCESS_THRESHOLD_M: f64 = 50.0;

/// Index of `infra_flag::ROAD_ACCESS` (must match the on-chain program).
pub const FLAG_ROAD_ACCESS: u16 = 1 << 5;

/// A graph of the road network in which vertices are coordinates and edges are
/// straight polyline pieces, so true network distances can be computed.
pub struct NetworkGraph {
    adjacency: Vec<Vec<(usize, f64)>>,
    /// Global vertex ids belonging to each road segment.
    segment_vertices: Vec<Vec<usize>>,
    sealed_segment: Vec<bool>,
}

impl NetworkGraph {
    pub fn build(graph: &RoadGraph) -> Self {
        let mut vertices: Vec<Coord<f64>> = Vec::new();
        let mut vertex_index: HashMap<(u64, u64), usize> = HashMap::new();
        let mut adjacency: Vec<Vec<(usize, f64)>> = Vec::new();
        let mut segment_vertices = Vec::with_capacity(graph.segments.len());
        let mut sealed_segment = Vec::with_capacity(graph.segments.len());

        for seg in &graph.segments {
            let coords: Vec<Coord<f64>> = seg.line.coords().copied().collect();
            let mut ids: Vec<usize> = Vec::with_capacity(coords.len());

            for c in &coords {
                let key = (c.x.to_bits(), c.y.to_bits());
                let idx = match vertex_index.get(&key) {
                    Some(&i) => i,
                    None => {
                        let i = vertices.len();
                        vertices.push(*c);
                        adjacency.push(Vec::new());
                        vertex_index.insert(key, i);
                        i
                    }
                };
                if let Some(&prev) = ids.last() {
                    let dist = Haversine::distance(
                        geo::Point::new(vertices[prev].x, vertices[prev].y),
                        geo::Point::new(vertices[idx].x, vertices[idx].y),
                    );
                    adjacency[prev].push((idx, dist));
                    adjacency[idx].push((prev, dist));
                }
                if ids.last() != Some(&idx) {
                    ids.push(idx);
                }
            }

            if !ids.is_empty() {
                segment_vertices.push(ids);
                sealed_segment.push(is_sealed_highway(&seg.highway));
            }
        }

        NetworkGraph {
            adjacency,
            segment_vertices,
            sealed_segment,
        }
    }
}

/// Reachability analysis of a single parcel against the road network.
#[derive(Debug, Clone)]
pub struct ReachabilityReport {
    /// Straight-line distance from the parcel centroid to the nearest road.
    pub nearest_road_m: f64,
    /// Number of boundary crossings with the road network.
    pub boundary_accesses: usize,
    /// Name/highway of the nearest road, if any.
    pub nearest_road: Option<AccessPoint>,
    /// Total road length (km) reachable from the parcel's access point within
    /// the same connected component of the network.
    pub component_km: f64,
    /// Whether a sealed (paved/main) road is in the same connected component.
    pub sealed_reachable: bool,
    /// Network distance along roads from the parcel's access roads to the
    /// nearest sealed road, if reachable.
    pub sealed_network_m: Option<f64>,
    /// Derived infrastructure flags (road-access bit is set when the parcel is
    /// physically adjacent to a road).
    pub flags: u16,
    /// sha-256 digest over the canonical flag payload (see `access_digest`).
    pub access_hash: [u8; 32],
}

/// Analyze a parcel's road reachability and build the canonical flag digest.
pub fn analyze(
    network: &NetworkGraph,
    graph: &RoadGraph,
    parcel: &geo::Polygon<f64>,
    parcel_id: &[u8; 32],
) -> ReachabilityReport {
    // 1. Physical adjacency: roads the parcel boundary actually touches.
    let boundary = access_points_on_boundary(graph, parcel);
    let boundary_accesses = boundary.len();

    // Nearest road to the parcel centroid (fallback when no boundary hits).
    let centroid = parcel.centroid().unwrap_or(Point::new(0.0, 0.0));
    let nearest = crate::access::nearest_road_access(graph, centroid.0, 1)
        .into_iter()
        .next();
    let nearest_road_m = nearest
        .as_ref()
        .map(|a| a.distance_m)
        .unwrap_or(f64::INFINITY);

    let access_roads: Vec<usize> = if !boundary.is_empty() {
        segments_touching_roads(graph, &boundary)
    } else if nearest_road_m <= ROAD_ACCESS_THRESHOLD_M {
        // No boundary contacts, but an actual road lies at the threshold:
        // fall back to the segment of the nearest road.
        nearest
            .as_ref()
            .and_then(|hit| nearest_segment_for(graph, hit))
            .into_iter()
            .collect()
    } else {
        Vec::new()
    };

    // 2. Reachability within the connected component of the access roads. Only
    //    meaningful when the parcel is physically adjacent to the network.
    let (sealed_reachable, sealed_network_m, component_km) =
        reachability_within_component(network, &access_roads);

    // 3. Flags: road access is real when the parcel touches a road with the
    //    network, or an actual road lies within the threshold.
    let mut flags: u16 = 0;
    if !boundary.is_empty() || nearest_road_m <= ROAD_ACCESS_THRESHOLD_M {
        flags |= FLAG_ROAD_ACCESS;
    }

    // 4. Canonical digest binding flags + metrics to the parcel.
    let report = ReachabilityReport {
        nearest_road_m,
        boundary_accesses,
        nearest_road: nearest.clone(),
        component_km: component_km / 1000.0,
        sealed_reachable,
        sealed_network_m,
        flags,
        access_hash: [0u8; 32],
    };
    let metrics = encode_metrics(&report);
    let access_hash = access_digest(parcel_id, flags, &metrics);

    ReachabilityReport { access_hash, ..report }
}

fn access_points_on_boundary(
    graph: &RoadGraph,
    parcel: &geo::Polygon<f64>,
) -> Vec<AccessPoint> {
    crate::access::road_access_along_boundary(graph, parcel, 16)
}

/// Find road segment indices that match a set of boundary access points, by
/// picking the segment whose geometry lies closest to each point.
fn segments_touching_roads(graph: &RoadGraph, hits: &[AccessPoint]) -> Vec<usize> {
    let mut out = HashSet::new();
    for hit in hits {
        if let Some(idx) = nearest_segment_for(graph, hit) {
            out.insert(idx);
        }
    }
    out.into_iter().collect()
}

fn nearest_segment_for(graph: &RoadGraph, hit: &AccessPoint) -> Option<usize> {
    let p = geo::Point::new(hit.point.x, hit.point.y);
    let mut best: Option<(usize, f64)> = None;
    for (i, seg) in graph.segments.iter().enumerate() {
        // Prefer an exact highway/name match when possible.
        let same_highway = seg.highway == hit.highway
            && (hit.road_name.is_none()
                || seg.name.as_ref() == hit.road_name.as_ref());
        if !same_highway {
            continue;
        }
        let d = point_to_linestring_m(&p, &seg.line);
        if best.map(|(_, bd)| d < bd).unwrap_or(true) {
            best = Some((i, d));
        }
    }
    best.map(|(i, _)| i)
}

fn point_to_linestring_m(p: &geo::Point<f64>, line: &geo::LineString<f64>) -> f64 {
    use geo::ClosestPoint as _;
    match line.closest_point(p) {
        geo::Closest::SinglePoint(q) | geo::Closest::Intersection(q) => {
            Haversine::distance(*p, q)
        }
        geo::Closest::Indeterminate => f64::INFINITY,
    }
}

/// Find the connected component reachable from `start_segments` and compute,
/// via Dijkstra, the network distance to the nearest sealed road in that
/// component. Returns `(sealed_reachable, sealed_network_m, component_length_m)`.
fn reachability_within_component(
    network: &NetworkGraph,
    start_segments: &[usize],
) -> (bool, Option<f64>, f64) {
    if start_segments.is_empty() {
        return (false, None, 0.0);
    }

    let n = network.adjacency.len();
    let mut dist: Vec<Option<f64>> = vec![None; n];
    let mut heap = BinaryHeap::new();
    let mut start_vertices = HashSet::new();

    for &si in start_segments {
        for &v in &network.segment_vertices[si] {
            start_vertices.insert(v);
        }
    }
    for &v in &start_vertices {
        dist[v] = Some(0.0);
        heap.push(HeapEntry { d: 0.0, v });
    }

    // BFS over the connected component, marking membership.
    let mut visited = vec![false; n];
    let mut ordered_starts: Vec<usize> = start_vertices.iter().copied().collect();
    ordered_starts.sort_unstable();
    let mut queue: VecDeque<usize> = ordered_starts.iter().copied().collect();
    for &v in &ordered_starts {
        visited[v] = true;
    }
    while let Some(v) = queue.pop_front() {
        for &(w, _) in &network.adjacency[v] {
            if !visited[w] {
                visited[w] = true;
                queue.push_back(w);
            }
        }
    }

    // Sum every undirected edge inside the component exactly once (v < w), so
    // the total is independent of traversal order and the resulting digest is
    // deterministic.
    let mut component_length = 0.0;
    for v in 0..n {
        if !visited[v] {
            continue;
        }
        for &(w, wl) in &network.adjacency[v] {
            if w > v && visited[w] {
                component_length += wl;
            }
        }
    }

    // Dijkstra to the nearest sealed vertex.
    let mut best_sealed: Option<f64> = None;
    while let Some(HeapEntry { d, v }) = heap.pop() {
        if dist[v].map(|cur| cur < d).unwrap_or(true) {
            continue;
        }
        if is_sealed_vertex(network, v) {
            best_sealed = Some(d);
            break;
        }
        for &(w, wl) in &network.adjacency[v] {
            let nd = d + wl;
            if dist[w].map(|cur| cur <= nd).unwrap_or(false) {
                continue;
            }
            dist[w] = Some(nd);
            heap.push(HeapEntry { d: nd, v: w });
        }
    }

    (best_sealed.is_some(), best_sealed, component_length)
}

fn is_sealed_vertex(network: &NetworkGraph, v: usize) -> bool {
    network
        .sealed_segment
        .iter()
        .enumerate()
        .any(|(si, sealed)| *sealed && network.segment_vertices[si].contains(&v))
}

struct HeapEntry {
    d: f64,
    v: usize,
}

impl PartialEq for HeapEntry {
    fn eq(&self, other: &Self) -> bool {
        self.d == other.d
    }
}
impl Eq for HeapEntry {}
impl PartialOrd for HeapEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for HeapEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse so BinaryHeap behaves as a min-heap.
        other.d.total_cmp(&self.d)
    }
}

/// Serialize reachability metrics deterministically for hashing.
pub fn encode_metrics(report: &ReachabilityReport) -> Vec<u8> {
    let mut out = Vec::with_capacity(40);
    out.extend_from_slice(&report.nearest_road_m.to_bits().to_le_bytes());
    out.extend_from_slice(&(report.boundary_accesses as u32).to_le_bytes());
    out.extend_from_slice(&report.component_km.to_bits().to_le_bytes());
    out.extend_from_slice(&[report.sealed_reachable as u8]);
    out.extend_from_slice(
        &report
            .sealed_network_m
            .unwrap_or(f64::INFINITY)
            .to_bits()
            .to_le_bytes(),
    );
    out
}

/// Canonical payload binding parcel identity + flags + reachability metrics
/// into a single verifiable digest.
pub fn access_digest(parcel_id: &[u8; 32], flags: u16, metrics: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(b"terra-access-v1");
    hasher.update(parcel_id);
    hasher.update(flags.to_le_bytes());
    hasher.update(metrics.len().to_le_bytes());
    hasher.update(metrics);
    hasher.finalize().into()
}
#[cfg(test)]
mod tests {
    use super::*;
    use geo::{line_string, polygon};

    fn sample_graph() -> RoadGraph {
        // Sealed road AB, connected via shared vertex B to an unsealed track BC.
        let sealed = crate::graph::RoadSegment {
            id: 1,
            name: Some("Main St".into()),
            highway: "primary".into(),
            oneway: false,
            line: line_string![
                (x: 10.0, y: 0.0),
                (x: 11.0, y: 0.0)
            ],
            length_m: 111_000.0,
        };
        let track = crate::graph::RoadSegment {
            id: 2,
            name: Some("Private Track".into()),
            highway: "track".into(),
            oneway: false,
            line: line_string![
                (x: 11.0, y: 0.0),
                (x: 12.0, y: 0.0)
            ],
            length_m: 111_000.0,
        };
        let mut g = RoadGraph::default();
        g.segments.push(sealed);
        g.segments.push(track);
        g
    }

    #[test]
    fn parcel_touching_track_reaches_sealed_network() {
        let graph = sample_graph();
        let network = NetworkGraph::build(&graph);

        // Parcel adjacent to the track (at ~11.8,0), i.e. network-connected to sealed.
        let parcel = polygon![
            (x: 11.8, y: -0.01),
            (x: 11.9, y: -0.01),
            (x: 11.9, y: 0.01),
            (x: 11.8, y: 0.01),
        ];
        let report = analyze(&network, &graph, &parcel, &[7u8; 32]);
        assert!(report.boundary_accesses >= 1);
        assert!(report.flags & FLAG_ROAD_ACCESS != 0);
        assert!(report.sealed_reachable, "track is attached to a sealed road");
        assert!(report.sealed_network_m.is_some());
        assert_eq!(report.access_hash.len(), 32);
    }

    #[test]
    fn isolated_parcel_has_no_road_access() {
        let graph = sample_graph();
        let network = NetworkGraph::build(&graph);

        // Far away from any road.
        let parcel = polygon![
            (x: 40.0, y: 40.0),
            (x: 40.1, y: 40.0),
            (x: 40.1, y: 40.1),
            (x: 40.0, y: 40.1),
        ];
        let report = analyze(&network, &graph, &parcel, &[9u8; 32]);
        assert_eq!(report.boundary_accesses, 0);
        assert!(report.flags & FLAG_ROAD_ACCESS == 0);
        assert!(!report.sealed_reachable);
    }

    #[test]
    fn digest_is_deterministic_and_sensitive() {
        let graph = sample_graph();
        let network = NetworkGraph::build(&graph);
        let parcel = polygon![
            (x: 11.8, y: -0.01),
            (x: 11.9, y: -0.01),
            (x: 11.9, y: 0.01),
            (x: 11.8, y: 0.01),
        ];
        let a = analyze(&network, &graph, &parcel, &[1u8; 32]);
        let b = analyze(&network, &graph, &parcel, &[1u8; 32]);
        let c = analyze(&network, &graph, &parcel, &[2u8; 32]);
        assert_eq!(a.access_hash, b.access_hash);
        assert_ne!(a.access_hash, c.access_hash, "parcel id must change the digest");
    }
}
