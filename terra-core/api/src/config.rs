use std::path::PathBuf;

use anyhow::{Context, Result};

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
    /// Ed25519 public key (base58) that may sign privileged requests.
    /// When absent, privileged endpoints reject all callers (fail-closed).
    pub api_authority_pubkey: Option<ed25519_dalek::VerifyingKey>,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok();
        let api_authority_pubkey = match std::env::var("API_AUTHORITY_PUBKEY") {
            Ok(key_b58) => {
                let bytes = bs58::decode(&key_b58)
                    .into_vec()
                    .context("API_AUTHORITY_PUBKEY is not valid base58")?;
                let arr: [u8; 32] = bytes.try_into().map_err(|_| {
                    anyhow::anyhow!("API_AUTHORITY_PUBKEY must be 32 bytes (ed25519 pubkey)")
                })?;
                Some(
                    ed25519_dalek::VerifyingKey::from_bytes(&arr)
                        .context("invalid ed25519 public key in API_AUTHORITY_PUBKEY")?,
                )
            }
            Err(_) => None,
        };
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
            api_authority_pubkey,
        })
    }
}
