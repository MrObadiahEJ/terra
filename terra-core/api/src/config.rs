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
    /// Solana RPC URL for the chain indexer (Sprint C). Defaults to localnet.
    pub rpc_url: String,
    /// Whether the background chain indexer task runs at all. Defaults on;
    /// set `TERRA_INDEXER_ENABLED=false` to stop metered RPC traffic without
    /// changing code (R-045).
    pub indexer_enabled: bool,
    /// Seconds between successful indexer polls. Default 60 (was a hard-coded
    /// 15 s poll of every full program account).
    pub indexer_poll_interval_secs: u64,
    /// Per-RPC-call timeout for indexer requests. Default 30 s.
    pub indexer_rpc_timeout_secs: u64,
    /// Ceiling for exponential backoff after a failed poll. Default 300 s.
    pub indexer_max_backoff_secs: u64,
    /// Max pubkeys per `getMultipleAccounts` batch in phase 2 of a scan.
    pub indexer_account_batch_size: usize,
    /// Ed25519 public key (base58) that may sign privileged requests.
    /// When absent, privileged endpoints reject all callers (fail-closed).
    pub api_authority_pubkey: Option<ed25519_dalek::VerifyingKey>,
    /// Ed25519 public key (base58) of the VRF oracle authority for storage
    /// proof-of-possession. When set, `prove_replica` verifies that the
    /// submitted proof is a valid Ed25519 signature from this authority over
    /// the challenge + content_address bytes. When absent, the API fails
    /// closed: no challenges are issued and `prove_replica` returns 503. The
    /// legacy SHA-256(challenge + content_address) fallback was removed — a
    /// digest never proved continued possession of the bytes.
    pub vrf_oracle_pubkey: Option<ed25519_dalek::VerifyingKey>,
    /// Maximum accepted request body size in bytes (tower-http limit).
    /// Default 64 MiB: covers multipart OSM ingest (PBF extracts) and photo
    /// evidence uploads; JSON API payloads are far smaller. Override via env.
    pub max_body_bytes: usize,
    /// Per-request timeout (seconds) applied by tower-http `TimeoutLayer`.
    /// Default 60 s; slow handlers should be lifted via env for heavy ingest.
    pub request_timeout_secs: u64,
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
        let vrf_oracle_pubkey = match std::env::var("VRF_ORACLE_PUBKEY") {
            Ok(key_b58) => {
                let bytes = bs58::decode(&key_b58)
                    .into_vec()
                    .context("VRF_ORACLE_PUBKEY is not valid base58")?;
                let arr: [u8; 32] = bytes.try_into().map_err(|_| {
                    anyhow::anyhow!("VRF_ORACLE_PUBKEY must be 32 bytes (ed25519 pubkey)")
                })?;
                Some(
                    ed25519_dalek::VerifyingKey::from_bytes(&arr)
                        .context("invalid ed25519 public key in VRF_ORACLE_PUBKEY")?,
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
            max_body_bytes: std::env::var("MAX_BODY_BYTES")
                .ok()
                .and_then(|v| v.parse::<usize>().ok())
                .unwrap_or(64 * 1024 * 1024),
            request_timeout_secs: std::env::var("REQUEST_TIMEOUT_SECS")
                .ok()
                .and_then(|v| v.parse::<u64>().ok())
                .unwrap_or(60),
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
            vrf_oracle_pubkey,
            rpc_url: std::env::var("TERRA_RPC_URL")
                .unwrap_or_else(|_| "http://127.0.0.1:8899".to_string()),
            indexer_enabled: parse_bool_env("TERRA_INDEXER_ENABLED", true),
            indexer_poll_interval_secs: parse_u64_env("TERRA_INDEXER_POLL_SECS", 60),
            indexer_rpc_timeout_secs: parse_u64_env("TERRA_INDEXER_RPC_TIMEOUT_SECS", 30),
            indexer_max_backoff_secs: parse_u64_env("TERRA_INDEXER_MAX_BACKOFF_SECS", 300),
            indexer_account_batch_size: parse_usize_env("TERRA_INDEXER_BATCH_SIZE", 100),
        })
    }
}

/// Parse an env bool. Invalid values fall back to the documented default
/// rather than making the API fail to boot on a typo.
fn parse_bool(value: Option<String>, default: bool) -> bool {
    match value {
        Some(value) => match value.trim().to_ascii_lowercase().as_str() {
            "1" | "true" | "yes" | "on" => true,
            "0" | "false" | "no" | "off" => false,
            _ => default,
        },
        None => default,
    }
}

fn parse_bool_env(name: &str, default: bool) -> bool {
    parse_bool(std::env::var(name).ok(), default)
}

fn parse_u64(value: Option<String>, default: u64) -> u64 {
    value
        .and_then(|v| v.trim().parse::<u64>().ok())
        .filter(|v| *v > 0)
        .unwrap_or(default)
}

fn parse_u64_env(name: &str, default: u64) -> u64 {
    parse_u64(std::env::var(name).ok(), default)
}

fn parse_usize(value: Option<String>, default: usize) -> usize {
    value
        .and_then(|v| v.trim().parse::<usize>().ok())
        .filter(|v| *v > 0)
        .unwrap_or(default)
}

fn parse_usize_env(name: &str, default: usize) -> usize {
    parse_usize(std::env::var(name).ok(), default)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn indexer_env_defaults_are_conservative() {
        // Parse helpers directly so a shared test process's env cannot change
        // the assertion. Env lookups are a thin wrapper over these helpers.
        assert!(parse_bool(None, true));
        assert_eq!(parse_u64(None, 60), 60);
        assert_eq!(parse_u64(None, 30), 30);
        assert_eq!(parse_u64(None, 300), 300);
        assert_eq!(parse_usize(None, 100), 100);
    }

    #[test]
    fn indexer_env_rejects_invalid_and_zero_values() {
        assert!(!parse_bool(Some("FALSE".into()), true));
        assert!(parse_bool(Some("yes".into()), false));
        assert!(parse_bool(Some("bogus".into()), true));
        assert_eq!(parse_u64(Some("0".into()), 60), 60);
        assert_eq!(parse_u64(Some("bogus".into()), 60), 60);
        assert_eq!(parse_u64(Some(" 90 ".into()), 60), 90);
        assert_eq!(parse_usize(Some("not-a-number".into()), 100), 100);
        assert_eq!(parse_usize(Some("250".into()), 100), 250);
    }
}
