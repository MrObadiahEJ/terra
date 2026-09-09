use std::sync::Arc;

use sqlx::PgPool;
use terra_geo::{OsmData, RoadGraph};

use crate::auth::ApiAuthority;
use crate::storage::StorageBackend;

/// Shared application state passed to all handlers.
#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    /// Loaded OSM road network + POIs, if a PBF path is configured.
    pub geo: Option<Arc<GeoData>>,
    /// Ed25519 authority for privileged API endpoints (reconcile, forfeiture).
    pub api_authority: ApiAuthority,
    /// Storage backend for evidence uploads.
    pub storage: Arc<StorageBackend>,
}

pub struct GeoData {
    pub data: OsmData,
    pub graph: RoadGraph,
}
