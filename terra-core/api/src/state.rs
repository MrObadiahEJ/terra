use std::sync::Arc;

use sqlx::PgPool;
use terra_geo::{OsmData, RoadGraph};

/// Shared application state passed to all handlers.
#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    /// Loaded OSM road network + POIs, if a PBF path is configured.
    pub geo: Option<Arc<GeoData>>,
}

pub struct GeoData {
    pub data: OsmData,
    pub graph: RoadGraph,
}
