mod auth;
mod config;
mod db;
mod error;
mod geoutil;
mod routes;
mod state;
mod storage;
mod zk;

use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;

use anyhow::{Context, Result};
use axum::extract::Request;
use axum::http::{HeaderValue, Method};
use axum::middleware::{self, Next};
use axum::response::Response;
use axum::routing::get;
use axum::Router;
use tower_http::cors::CorsLayer;
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::timeout::TimeoutLayer;
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

    let storage = Arc::new(storage::from_env()?);
    let zk = zk::ZkRuntime::from_env()?.map(Arc::new);
    if let Some(zk) = &zk {
        tracing::warn!(
            mode = ?zk.mode(),
            proving_enabled = zk.is_dev_prover_enabled(),
            "Groth16 ZK service configured; validate ceremony provenance before production use"
        );
    } else {
        tracing::info!("Groth16 ZK verification disabled: no key configuration");
    }

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
        .layer(middleware::from_fn(security_headers))
        .layer(RequestBodyLimitLayer::new(config.max_body_bytes))
        .layer(TimeoutLayer::new(std::time::Duration::from_secs(config.request_timeout_secs)))
        .with_state(AppState {
            pool,
            geo,
            api_authority: auth::ApiAuthority(config.api_authority_pubkey),
            storage,
            zk,
        });

    let addr = format!("{}:{}", config.host, config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("terra-api listening on {addr}");
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;
    Ok(())
}

/// Security response headers applied to every API response (Sprint 012).
///
/// Defense-in-depth on the unauthenticated mirror: browsers must not be able to
/// MIME-sniff API responses (`nosniff`), frame them (`DENY`), leak referrer
/// context to third parties (`no-referrer`), or exercise browser capabilities
/// the product does not use (`Permissions-Policy`). The API surface is JSON-only
/// (plus multipart evidence/OSM ingest), so no Content-Security-Policy applies.
async fn security_headers(
    req: Request,
    next: Next,
) -> Response {
    let mut response = next.run(req).await;
    response
        .headers_mut()
        .insert("x-content-type-options", HeaderValue::from_static("nosniff"));
    response
        .headers_mut()
        .insert("x-frame-options", HeaderValue::from_static("DENY"));
    response
        .headers_mut()
        .insert("referrer-policy", HeaderValue::from_static("no-referrer"));
    response.headers_mut().insert(
        "permissions-policy",
        HeaderValue::from_static("camera=(), microphone=(), geolocation=()"),
    );
    response
}

async fn load_geo(path: Option<&Path>) -> Result<Option<Arc<GeoData>>> {
    let Some(path) = path else {
        tracing::warn!("OSM_PBF_PATH not set, geo endpoints disabled");
        return Ok(None);
    };

    let path = path.to_path_buf();
    tracing::info!(path = %path.display(), "loading OSM road network");
    let (data, graph, building_bounds, network) = tokio::task::spawn_blocking(move || {
        let data = terra_geo::read_osm_pbf(&path)
            .with_context(|| format!("failed to parse OSM data from {}", path.display()))?;
        let graph = terra_geo::build_graph(&data);
        let network = terra_geo::NetworkGraph::build(&graph);
        let building_bounds = data.buildings.iter().map(building_bounds).collect();
        tracing::info!(
            nodes = data.nodes.len(),
            roads = data.roads.len(),
            pois = data.pois.len(),
            "OSM data loaded"
        );
        Ok::<_, anyhow::Error>((data, graph, building_bounds, network))
    })
    .await??;

    Ok(Some(Arc::new(GeoData {
        data,
        graph,
        building_bounds,
        network,
    })))
}

fn building_bounds(building: &terra_geo::BuildingFootprint) -> Option<geo::Rect<f64>> {
    let mut points = building.ring.iter();
    let first = *points.next()?;
    let (min, max) = points.fold((first, first), |(min, max), point| {
        (
            geo::Coord {
                x: min.x.min(point.x),
                y: min.y.min(point.y),
            },
            geo::Coord {
                x: max.x.max(point.x),
                y: max.y.max(point.y),
            },
        )
    });
    Some(geo::Rect::new(min, max))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::StatusCode;
    use axum::routing::get;
    use axum::Router;
    use tower::ServiceExt;

    use crate::auth::ApiAuthority;
    use crate::routes::AppState;
    use crate::storage::StorageBackend;

    /// Minimal router with only the security-headers middleware applied.
    /// Mirrors the production composition (sans CORS/Trace/DB-backed handlers).
    fn test_router() -> Router {
        let pool = sqlx::PgPool::connect_lazy("postgres://x:x@localhost/x").unwrap();
        let state = AppState {
            pool,
            geo: None,
            api_authority: ApiAuthority(None),
            storage: Arc::new(StorageBackend::Local {
                root: std::path::PathBuf::from("/tmp/terra-test"),
            }),
            zk: None,
        };
        Router::new()
            .route("/__security_headers_probe", get(|| async { "ok" }))
            .route(
                "/__security_headers_slow",
                get(|| async {
                    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                    "ok"
                }),
            )
            .layer(middleware::from_fn(security_headers))
            .layer(RequestBodyLimitLayer::new(8))
            .layer(TimeoutLayer::new(std::time::Duration::from_millis(1)))
            .with_state(state)
    }

    async fn probe(path: &str) -> axum::response::Response {
        test_router()
            .oneshot(
                axum::http::Request::builder()
                    .uri(path)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn ok_response_carries_security_headers() {
        let response = probe("/__security_headers_probe").await;
        assert_eq!(response.status(), StatusCode::OK);

        assert_eq!(
            response
                .headers()
                .get("x-content-type-options")
                .and_then(|v| v.to_str().ok()),
            Some("nosniff")
        );
        assert_eq!(
            response
                .headers()
                .get("x-frame-options")
                .and_then(|v| v.to_str().ok()),
            Some("DENY")
        );
        assert_eq!(
            response
                .headers()
                .get("referrer-policy")
                .and_then(|v| v.to_str().ok()),
            Some("no-referrer")
        );
        assert_eq!(
            response
                .headers()
                .get("permissions-policy")
                .and_then(|v| v.to_str().ok()),
            Some("camera=(), microphone=(), geolocation=()")
        );
    }

    #[tokio::test]
    async fn error_response_also_carries_security_headers() {
        let response = probe("/__security_headers_does_not_exist").await;
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        assert_eq!(
            response
                .headers()
                .get("x-content-type-options")
                .and_then(|v| v.to_str().ok()),
            Some("nosniff")
        );
    }

    #[tokio::test]
    async fn oversized_content_length_rejected_with_413() {
        let response = test_router()
            .oneshot(
                axum::http::Request::builder()
                    .uri("/__security_headers_probe")
                    .header("content-length", "9")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
    }

    #[tokio::test]
    async fn within_limit_is_accepted() {
        let response = test_router()
            .oneshot(
                axum::http::Request::builder()
                    .uri("/__security_headers_probe")
                    .header("content-length", "2")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn slow_request_times_out_with_408() {
        let response = test_router()
            .oneshot(
                axum::http::Request::builder()
                    .uri("/__security_headers_slow")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::REQUEST_TIMEOUT);
    }
}
