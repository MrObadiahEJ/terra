use std::sync::Arc;

use sqlx::PgPool;
use terra_geo::{OsmData, RoadGraph};

use crate::auth::ApiAuthority;
use crate::storage::StorageBackend;
use crate::zk::ZkRuntime;

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
    /// Optional Groth16 verifier and development-only local prover.
    pub zk: Option<Arc<ZkRuntime>>,
}

pub struct GeoData {
    pub data: OsmData,
    pub graph: RoadGraph,
    /// Precomputed bounds aligned with `data.buildings`.
    pub building_bounds: Vec<Option<geo::Rect<f64>>>,
    /// Road-network topology used by reachability analysis.
    pub network: terra_geo::NetworkGraph,
}
