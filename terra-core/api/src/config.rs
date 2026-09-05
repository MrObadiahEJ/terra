use std::path::PathBuf;

use anyhow::Result;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub host: String,
    pub port: u16,
    pub osm_pbf_path: Option<PathBuf>,
    /// Comma-separated list of allowed CORS origins. Empty = same-origin
    /// only (safest default); `*` restores the old permissive behavior for
    /// local development behind a gateway.
    pub cors_allowed_origins: Vec<String>,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok();
        Ok(Self {
            database_url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgres://terra:terra@localhost:5432/terra_dev".to_string()),
            host: std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: std::env::var("PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(8080),
            osm_pbf_path: std::env::var("OSM_PBF_PATH").ok().map(PathBuf::from),
            cors_allowed_origins: std::env::var("CORS_ALLOWED_ORIGINS")
                .map(|v| {
                    v.split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect()
                })
                .unwrap_or_else(|_| {
                    vec![
                        "http://localhost:5173".to_string(),
                        "http://127.0.0.1:5173".to_string(),
                    ]
                }),
        })
    }
}
