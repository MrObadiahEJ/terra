mod config;
mod db;
mod error;
mod geoutil;
mod routes;
mod state;

use std::path::Path;
use std::sync::Arc;

use anyhow::{Context, Result};
use axum::http::{HeaderValue, Method};
use axum::routing::get;
use axum::Router;
use tower_http::cors::CorsLayer;
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

    // CORS locked to configured origins (default: local Vite dev server).
    // The mirror API is unauthenticated by design for pilot deployments
    // behind a gateway; exposing it cross-origin to the whole web would let
    // any site drive state-changing endpoints from a visitor's browser.
    let cors = if config.cors_allowed_origins.iter().any(|o| o == "*") {
        tracing::warn!("CORS wide open (*) — only for local development");
        CorsLayer::permissive()
    } else {
        let origins: Vec<HeaderValue> = config
            .cors_allowed_origins
            .iter()
            .filter_map(|o| o.parse().ok())
            .collect();
        CorsLayer::new()
            .allow_origin(origins)
            .allow_methods([
                Method::GET,
                Method::POST,
                Method::PUT,
                Method::PATCH,
                Method::DELETE,
            ])
            .allow_headers([axum::http::header::CONTENT_TYPE])
    };

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
        let data = terra_geo::read_osm_pbf(&path)
            .with_context(|| format!("failed to parse OSM data from {}", path.display()))?;
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
