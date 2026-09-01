use axum::Router;

pub mod attestations;
pub mod fusion;
pub mod geo;
pub mod health;
pub mod identities;
pub mod parcels;
pub mod pilot_zones;
pub mod vaults;

pub use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .nest("/parcels", parcels::router())
        .nest("/identities", identities::router())
        .nest("/vaults", vaults::router())
        .nest("/geo", geo::router())
        .nest("/fusion", fusion::router())
        .nest("/pilot-zones", pilot_zones::router())
}
