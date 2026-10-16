pub mod access;
pub mod canonical;
pub mod canonical3d;
pub mod graph;
pub mod osm;
pub mod poi;
pub mod reach;

pub use access::{nearest_road_access, road_access_along_boundary, AccessPoint};
pub use canonical::{canonicalize as canonicalize_geometry, geometry_digest, geometry_digest_hex};
pub use canonical3d::{
    canonicalize_v2 as canonicalize_geometry_v2, geometry_digest_hex_v2, GEOMETRY_DIGEST_VERSION_V2,
};
pub use graph::{build_graph, RoadGraph, RoadSegment};
pub use osm::{read_osm_pbf, BuildingFootprint, OsmData, Poi, RoadWay};
pub use poi::{pois_near, pois_within};
pub use reach::{
    access_digest, analyze, encode_metrics, is_sealed_highway, NetworkGraph, ReachabilityReport,
    FLAG_ROAD_ACCESS, ROAD_ACCESS_THRESHOLD_M,
};
