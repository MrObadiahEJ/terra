use axum::Router;

pub mod activity;
pub mod authority_registry;
pub mod cross_border;
pub mod demo_scenarios;
pub mod disputes;
pub mod escrows;
pub mod evidence;
pub mod fusion;
pub mod geo;
pub mod health;
pub mod identities;
pub mod parcels;
pub mod pilot_zones;
pub mod reconstruction;
pub mod recovery;
pub mod rights;
pub mod spatial;
pub mod spatial_accounts;
pub mod staking;
pub mod stories;
pub mod subdivision;
pub mod threshold_credentials;
pub mod tx_prep;
pub mod vaults;
pub mod world_registry;
pub mod zk_proofs;

pub use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .nest("/parcels", parcels::router())
        .nest("/identities", identities::router())
        .nest("/authority-registry", authority_registry::router())
        .nest("/vaults", vaults::router())
        .nest("/disputes", disputes::router())
        .nest("/escrows", escrows::router())
        .nest("/rights", rights::router())
        .nest("/cross-border", cross_border::router())
        .nest("/subdivision", subdivision::router())
        .nest("/staking", staking::router())
        .nest("/zk", zk_proofs::router())
        .nest("/spatial", spatial::router())
        .nest("/spatial-accounts", spatial_accounts::router())
        // stories.rs owns absolute paths (/stories, /exports/{id}).
        .merge(stories::router())
        .nest("/geo", geo::router())
        .nest("/fusion", fusion::router())
        .nest("/pilot-zones", pilot_zones::router())
        .nest("/world-registry", world_registry::router())
        .nest("/recovery", recovery::router())
        .nest("/credentials", threshold_credentials::router())
        .nest("/evidence", evidence::router())
        .nest("/reconstruction", reconstruction::router())
        .nest("/tx", tx_prep::router())
        .nest("/demo", demo_scenarios::router())
        .nest("/activity", activity::router())
}
