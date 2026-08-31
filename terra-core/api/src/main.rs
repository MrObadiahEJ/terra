mod config;
mod db;
mod error;
mod geoutil;
mod routes;
mod state;

use std::path::Path;
use std::sync::Arc;

use anyhow::{Context, Result};
use axum::routing::get;
use axum::Router;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

use routes::AppState;
use state::GeoData;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,tower_http=info")),
        )
        .init();

    let config = config::Config::from_env()?;
    tracing::info!(database_url = %config.database_url, "connecting to database");
    let pool = db::connect(&config.database_url).await?;
    tracing::info!("migrations applied, database ready");

    let geo = load_geo(config.osm_pbf_path.as_deref()).await?;

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app: Router = Router::new()
        .route("/health", get(routes::health::health))
        .nest("/api/v1", routes::router())
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(AppState { pool, geo });

    let addr = format!("{}:{}", config.host, config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("terra-api listening on {addr}");
    axum::serve(listener, app).await?;
    Ok(())
}

async fn load_geo(path: Option<&Path>) -> Result<Option<Arc<GeoData>>> {
    let Some(path) = path else {
        tracing::warn!("OSM_PBF_PATH not set, geo endpoints disabled");
        return Ok(None);
    };

    let path = path.to_path_buf();
    tracing::info!(path = %path.display(), "loading OSM road network");
    let (data, graph) = tokio::task::spawn_blocking(move || {
        let data = terra_geo::read_osm_pbf(&path).with_context(|| {
            format!("failed to parse OSM data from {}", path.display())
        })?;
        let graph = terra_geo::build_graph(&data);
        tracing::info!(
            nodes = data.nodes.len(),
            roads = data.roads.len(),
            pois = data.pois.len(),
            "OSM data loaded"
        );
        Ok::<_, anyhow::Error>((data, graph))
    })
    .await??;

    Ok(Some(Arc::new(GeoData { data, graph })))
}
