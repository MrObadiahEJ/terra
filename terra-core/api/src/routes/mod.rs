use axum::Router;

pub mod attestations;
pub mod authority_registry;
pub mod cross_border;
pub mod disputes;
pub mod escrows;
pub mod fusion;
pub mod geo;
pub mod health;
pub mod identities;
pub mod ipfs_docs;
pub mod parcels;
pub mod pilot_zones;
pub mod rights;
pub mod spatial;
pub mod staking;
pub mod subdivision;
pub mod vaults;
pub mod zk_proofs;

pub use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .nest("/parcels", parcels::router())
        .nest("/identities", identities::router())
        .nest("/authority-registry", authority_registry::router())
        .nest("/ipfs-docs", ipfs_docs::router())
        .nest("/vaults", vaults::router())
        .nest("/disputes", disputes::router())
        .nest("/escrows", escrows::router())
        .nest("/rights", rights::router())
        .nest("/cross-border", cross_border::router())
        .nest("/subdivision", subdivision::router())
        .nest("/staking", staking::router())
        .nest("/zk", zk_proofs::router())
        .nest("/spatial", spatial::router())
        .nest("/geo", geo::router())
        .nest("/fusion", fusion::router())
        .nest("/pilot-zones", pilot_zones::router())
}
