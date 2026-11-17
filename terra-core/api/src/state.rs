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
/// Ed25519 public key of the Switchboard/Pyth VRF oracle authority.
/// When set, `prove_replica` verifies that the submitted proof is a valid
/// signature from this authority; when absent, the legacy SHA-256 comparison
/// is used for backward compatibility.
pub vrf_oracle_pubkey: Option<ed25519_dalek::VerifyingKey>,
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
