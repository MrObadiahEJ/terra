//! Chapter 12 — Storage provider discovery (REQ-028).
//!
//! Endpoints:
//!   POST   /providers                          register a storage provider
//!   GET    /providers                          list providers (filter by type/active/region)
//!   GET    /providers/{id}                     get a provider
//!   GET    /providers/{id}/replicas            list a provider's replicas
//!   POST   /providers/{id}/replicas            register a component replica

use axum::extract::{Path, Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use ed25519_dalek::Verifier;
use hex;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::auth::SignedRequest;
use crate::error::AppError;
use crate::state::AppState;

use chrono::{Duration, Utc};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", post(create_provider).get(list_providers))
        .route("/{id}", get(get_provider))
        .route("/{id}/replicas", post(create_replica).get(list_replicas))
        .route("/{id}/payout-currency", post(set_payout_currency))
        .route(
            "/{provider_id}/replicas/{replica_id}/challenge",
            post(challenge_replica),
        )
        .route(
            "/{provider_id}/replicas/{replica_id}/prove",
            post(prove_replica),
        )
        .route("/replication-audit", get(replication_audit))
        .route("/authority-registries", get(list_authority_registries))
        .route(
            "/authority-registries/{pubkey}",
            get(get_authority_registry),
        )
        .route("/ipfs-docs", get(list_ipfs_docs))
        .route("/ipfs-docs/{cid}", get(get_ipfs_doc))
}

// ---------------------------------------------------------------------------
// Request / response types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct CreateProviderRequest {
    pub provider_id: String,
    pub name: String,
    pub provider_type: String,
    pub base_url: String,
    pub capabilities: Option<serde_json::Value>,
    pub geographic_scope: Option<Vec<String>>,
    pub reliability_score: Option<f64>,
    pub cost_per_gb: Option<f64>,
    /// Refundable anti-spam bond (raw units); slashed on failed PoP.
    pub stake_bond: Option<i64>,
    /// Wallet address for bond settlement and reward payouts.
    pub wallet_address: Option<String>,
    /// Preferred settlement asset (`settlement_assets.currency`). Omit to use
    /// the registry default (SOL); the provider is never forced onto USD.
    pub payout_currency: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateReplicaRequest {
    pub component_id: String,
    pub component_version: String,
    pub storage_reference: String,
    pub version: String,
    pub size_bytes: Option<i64>,
    pub checksum: Option<String>,
    pub content_address: Option<String>,
    pub latency_ms: Option<i32>,
    pub is_available: Option<bool>,
}

/// Premium category for artifact retrieval. Providers with a stake_bond > 0
/// are eligible for premium categories (priority retrieval, higher reward).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PremiumCategory {
    /// Standard retrieval — any provider can serve.
    Standard,
    /// Premium — only bonded providers (stake_bond > 0) can serve.
    Premium(String), // category name
}

#[derive(Debug, Deserialize)]
pub struct ListProvidersParams {
    pub provider_type: Option<String>,
    pub is_active: Option<bool>,
    pub region: Option<String>,
    /// If set, only return providers eligible for this premium category
    /// (i.e. stake_bond > 0).
    pub premium_category: Option<String>,
    #[serde(default = "default_limit")]
    pub limit: usize,
}

fn default_limit() -> usize {
    50
}

// ---------------------------------------------------------------------------
// Row types
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ProviderRow {
    pub id: Uuid,
    pub provider_id: String,
    pub name: String,
    pub provider_type: String,
    pub base_url: String,
    pub capabilities: serde_json::Value,
    pub geographic_scope: Vec<String>,
    pub is_active: bool,
    pub reliability_score: f64,
    pub cost_per_gb: Option<f64>,
    /// Refundable anti-spam bond (raw units); slashed on failed PoP.
    pub stake_bond: i64,
    /// Wallet address for bond settlement and reward payouts.
    pub wallet_address: Option<String>,
    /// Preferred settlement asset (`settlement_assets.currency`); NULL uses the
    /// registry default (SOL).
    pub payout_currency: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ReplicaRow {
    pub id: Uuid,
    pub provider_id: Uuid,
    pub component_id: String,
    pub component_version: String,
    pub storage_reference: String,
    pub version: String,
    pub size_bytes: Option<i64>,
    pub checksum: Option<String>,
    pub content_address: Option<String>,
    pub latency_ms: Option<i32>,
    pub is_available: bool,
    pub last_checked_at: chrono::DateTime<chrono::Utc>,
    pub last_challenge: Option<String>,
    pub last_proof_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Usage metering: number of times this replica has been retrieved.
    pub access_count: i64,
    /// Usage metering: total bytes served from this replica.
    pub bytes_served: i64,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

// ---------------------------------------------------------------------------
// POST /providers
// ---------------------------------------------------------------------------

pub async fn create_provider(
    State(state): State<AppState>,
    Json(req): Json<CreateProviderRequest>,
) -> Result<(axum::http::StatusCode, Json<ProviderRow>), AppError> {
    let capabilities = req.capabilities.unwrap_or(serde_json::json!({}));
    let geo_scope = req.geographic_scope.unwrap_or_default();
    let reliability = req.reliability_score.unwrap_or(1.0);
    let cost = req.cost_per_gb;
    let stake_bond = req.stake_bond.unwrap_or(0);
    let wallet_address = req.wallet_address.clone();
    ensure_enabled_settlement_asset(&state, req.payout_currency.as_deref()).await?;

    let row = sqlx::query_as::<_, ProviderRow>(
        r#"
        INSERT INTO storage_providers (
            provider_id, name, provider_type, base_url, capabilities,
            geographic_scope, reliability_score, cost_per_gb,
            stake_bond, wallet_address, payout_currency
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        RETURNING id, provider_id, name, provider_type, base_url, capabilities,
                  geographic_scope, is_active, reliability_score, cost_per_gb,
                  stake_bond, wallet_address, payout_currency, created_at, updated_at
        "#,
    )
    .bind(&req.provider_id)
    .bind(&req.name)
    .bind(&req.provider_type)
    .bind(&req.base_url)
    .bind(&capabilities)
    .bind(&geo_scope)
    .bind(reliability)
    .bind(cost)
    .bind(stake_bond)
    .bind(&wallet_address)
    .bind(&req.payout_currency)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| AppError::internal(format!("insert failed: {e}")))?;

    Ok((axum::http::StatusCode::CREATED, Json(row)))
}

/// Reject a payout currency that is unknown or disabled before it reaches the
/// foreign key, so callers get a 400 instead of a 500.
async fn ensure_enabled_settlement_asset(
    state: &AppState,
    currency: Option<&str>,
) -> Result<(), AppError> {
    let Some(currency) = currency else {
        return Ok(());
    };
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM settlement_assets
                       WHERE currency = $1 AND is_enabled = true)",
    )
    .bind(currency)
    .fetch_one(&state.pool)
    .await?;
    if !exists {
        return Err(AppError::bad_request(format!(
            "unknown or disabled settlement asset: {currency}"
        )));
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// GET /providers
// ---------------------------------------------------------------------------

const PROVIDERS_SELECT: &str = "SELECT id, provider_id, name, provider_type, \
    base_url, capabilities, geographic_scope, is_active, reliability_score, \
    cost_per_gb, stake_bond, wallet_address, payout_currency, created_at, updated_at \
    FROM storage_providers";

pub async fn list_providers(
    State(state): State<AppState>,
    Query(params): Query<ListProvidersParams>,
) -> Result<Json<Vec<ProviderRow>>, AppError> {
    let limit = params.limit.min(500);
    // Premium category filter: only bonded providers (stake_bond > 0).
    let premium_category = match &params.premium_category {
        Some(name) => PremiumCategory::Premium(name.clone()),
        None => PremiumCategory::Standard,
    };
    let premium_clause = match premium_category {
        PremiumCategory::Premium(_) => " AND stake_bond > 0".to_string(),
        PremiumCategory::Standard => String::new(),
    };
    let rows: Vec<ProviderRow> = match (&params.provider_type, &params.is_active, &params.region)
    {
        (Some(t), Some(a), Some(r)) => sqlx::query_as::<_, ProviderRow>(
            format!(
                "{PROVIDERS_SELECT} WHERE provider_type = $1 AND is_active = $2 AND $3 = ANY(geographic_scope){premium_clause} ORDER BY reliability_score DESC LIMIT $4"
            )
            .as_str(),
        )
        .bind(t)
        .bind(a)
        .bind(r)
        .bind(limit as i64)
        .fetch_all(&state.pool)
        .await
        .map_err(|e| AppError::internal(format!("query failed: {e}")))?,
        (Some(t), Some(a), None) => sqlx::query_as::<_, ProviderRow>(
            format!(
                "{PROVIDERS_SELECT} WHERE provider_type = $1 AND is_active = $2{premium_clause} ORDER BY reliability_score DESC LIMIT $3"
            )
            .as_str(),
        )
        .bind(t)
        .bind(a)
        .bind(limit as i64)
        .fetch_all(&state.pool)
        .await
        .map_err(|e| AppError::internal(format!("query failed: {e}")))?,
        (Some(t), None, Some(r)) => sqlx::query_as::<_, ProviderRow>(
            format!(
                "{PROVIDERS_SELECT} WHERE provider_type = $1 AND $2 = ANY(geographic_scope){premium_clause} ORDER BY reliability_score DESC LIMIT $3"
            )
            .as_str(),
        )
        .bind(t)
        .bind(r)
        .bind(limit as i64)
        .fetch_all(&state.pool)
        .await
        .map_err(|e| AppError::internal(format!("query failed: {e}")))?,
        (None, Some(a), Some(r)) => sqlx::query_as::<_, ProviderRow>(
            format!(
                "{PROVIDERS_SELECT} WHERE is_active = $1 AND $2 = ANY(geographic_scope){premium_clause} ORDER BY reliability_score DESC LIMIT $3"
            )
            .as_str(),
        )
        .bind(a)
        .bind(r)
        .bind(limit as i64)
        .fetch_all(&state.pool)
        .await
        .map_err(|e| AppError::internal(format!("query failed: {e}")))?,
        (Some(t), None, None) => sqlx::query_as::<_, ProviderRow>(
            format!(
                "{PROVIDERS_SELECT} WHERE provider_type = $1{premium_clause} ORDER BY reliability_score DESC LIMIT $2"
            )
            .as_str(),
        )
        .bind(t)
        .bind(limit as i64)
        .fetch_all(&state.pool)
        .await
        .map_err(|e| AppError::internal(format!("query failed: {e}")))?,
        (None, Some(a), None) => sqlx::query_as::<_, ProviderRow>(
            format!(
                "{PROVIDERS_SELECT} WHERE is_active = $1{premium_clause} ORDER BY reliability_score DESC LIMIT $2"
            )
            .as_str(),
        )
        .bind(a)
        .bind(limit as i64)
        .fetch_all(&state.pool)
        .await
        .map_err(|e| AppError::internal(format!("query failed: {e}")))?,
        (None, None, Some(r)) => sqlx::query_as::<_, ProviderRow>(
            format!(
                "{PROVIDERS_SELECT} WHERE $1 = ANY(geographic_scope){premium_clause} ORDER BY reliability_score DESC LIMIT $2"
            )
            .as_str(),
        )
        .bind(r)
        .bind(limit as i64)
        .fetch_all(&state.pool)
        .await
        .map_err(|e| AppError::internal(format!("query failed: {e}")))?,
        (None, None, None) => sqlx::query_as::<_, ProviderRow>(
            format!(
                "{PROVIDERS_SELECT}{premium_clause} ORDER BY reliability_score DESC LIMIT $1"
            )
            .as_str(),
        )
        .bind(limit as i64)
        .fetch_all(&state.pool)
        .await
        .map_err(|e| AppError::internal(format!("query failed: {e}")))?,
    };

    Ok(Json(rows))
}

// ---------------------------------------------------------------------------
// GET /providers/{id}
// ---------------------------------------------------------------------------

const PROVIDER_SELECT: &str = "SELECT id, provider_id, name, provider_type, \
    base_url, capabilities, geographic_scope, is_active, reliability_score, \
    cost_per_gb, stake_bond, wallet_address, payout_currency, created_at, updated_at \
    FROM storage_providers WHERE id = $1";

pub async fn get_provider(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ProviderRow>, AppError> {
    let row = sqlx::query_as::<_, ProviderRow>(PROVIDER_SELECT)
        .bind(id)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| AppError::internal(format!("query failed: {e}")))?
        .ok_or_else(|| AppError::not_found(format!("provider {id} not found")))?;

    Ok(Json(row))
}

#[derive(Debug, Deserialize)]
pub struct SetPayoutCurrencyRequest {
    /// Asset symbol from `settlement_assets`; null resets to the default.
    pub payout_currency: Option<String>,
}

/// Let a provider choose its settlement rail. Signed by the registered wallet
/// so a third party cannot redirect someone else's payouts. No rail is
/// mandatory: `null` means "pay me in the registry default".
pub async fn set_payout_currency(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    auth: SignedRequest,
    Json(req): Json<SetPayoutCurrencyRequest>,
) -> Result<Json<ProviderRow>, AppError> {
    ensure_enabled_settlement_asset(&state, req.payout_currency.as_deref()).await?;

    let wallet: Option<String> =
        sqlx::query_scalar("SELECT wallet_address FROM storage_providers WHERE id = $1")
            .bind(id)
            .fetch_optional(&state.pool)
            .await?
            .flatten();
    let wallet = wallet.ok_or_else(|| {
        AppError::bad_request("provider has no wallet address; cannot authorize payout change")
    })?;
    auth.verify_wallet(
        &wallet,
        "POST",
        &format!("/api/v1/providers/{id}/payout-currency"),
    )?;

    let updated = sqlx::query(
        "UPDATE storage_providers
         SET payout_currency = $1, updated_at = now()
         WHERE id = $2",
    )
    .bind(&req.payout_currency)
    .bind(id)
    .execute(&state.pool)
    .await?;
    if updated.rows_affected() == 0 {
        return Err(AppError::not_found(format!("provider {id} not found")));
    }

    let row = sqlx::query_as::<_, ProviderRow>(PROVIDER_SELECT)
        .bind(id)
        .fetch_one(&state.pool)
        .await?;
    Ok(Json(row))
}

// ---------------------------------------------------------------------------
// POST /providers/{id}/replicas
// ---------------------------------------------------------------------------

pub async fn create_replica(
    State(state): State<AppState>,
    Path(provider_id): Path<Uuid>,
    Json(req): Json<CreateReplicaRequest>,
) -> Result<(axum::http::StatusCode, Json<ReplicaRow>), AppError> {
    // Verify provider exists and is active.
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM storage_providers WHERE id = $1 AND is_active = true)",
    )
    .bind(provider_id)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| AppError::internal(format!("provider check failed: {e}")))?;
    if !exists {
        return Err(AppError::not_found(format!(
            "active provider {provider_id} not found"
        )));
    }

    let row = sqlx::query_as::<_, ReplicaRow>(
        r#"
        INSERT INTO provider_replicas (
            provider_id, component_id, component_version, storage_reference,
            version, size_bytes, checksum, content_address, latency_ms,
            is_available, access_count, bytes_served
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, 0, 0)
        RETURNING id, provider_id, component_id, component_version,
                  storage_reference, version, size_bytes, checksum,
                  content_address, latency_ms, is_available,
                  last_checked_at, last_challenge, last_proof_at,
                  access_count, bytes_served, created_at
        "#,
    )
    .bind(provider_id)
    .bind(&req.component_id)
    .bind(&req.component_version)
    .bind(&req.storage_reference)
    .bind(&req.version)
    .bind(req.size_bytes)
    .bind(&req.checksum)
    .bind(&req.content_address)
    .bind(req.latency_ms)
    .bind(req.is_available.unwrap_or(true))
    .fetch_one(&state.pool)
    .await
    .map_err(|e| AppError::internal(format!("insert failed: {e}")))?;

    Ok((axum::http::StatusCode::CREATED, Json(row)))
}

// ---------------------------------------------------------------------------
// GET /providers/{id}/replicas
// ---------------------------------------------------------------------------

const REPLICAS_SELECT: &str = "SELECT id, provider_id, component_id, \
    component_version, storage_reference, version, size_bytes, checksum, \
    content_address, latency_ms, is_available, last_checked_at, \
    last_challenge, last_proof_at, access_count, bytes_served, created_at \
    FROM provider_replicas WHERE provider_id = $1 ORDER BY component_id, component_version";

pub async fn list_replicas(
    State(state): State<AppState>,
    Path(provider_id): Path<Uuid>,
) -> Result<Json<Vec<ReplicaRow>>, AppError> {
    let rows = sqlx::query_as::<_, ReplicaRow>(REPLICAS_SELECT)
        .bind(provider_id)
        .fetch_all(&state.pool)
        .await
        .map_err(|e| AppError::internal(format!("query failed: {e}")))?;

    Ok(Json(rows))
}

// ---------------------------------------------------------------------------
// Reliability derivation (Q-008: reliability from challenge results, not
// self-declaration).
//
// A provider's reliability_score is recomputed from its replicas' proof-of-
// possession results. A replica with a recent successful proof counts as
// reliable; one with no proof or a stale proof counts as unreliable.
// The provider score is the fraction of its replicas that have proved within
// the proof window, floored at 0.05 so an unproved provider is never trusted.
fn reliability_from_proofs(last_proof_at: Option<chrono::DateTime<chrono::Utc>>) -> f64 {
    match last_proof_at {
        Some(proved_at) => {
            let age = Utc::now() - proved_at;
            if age <= Duration::hours(24) {
                1.0
            } else if age <= Duration::days(7) {
                0.5
            } else if age <= Duration::days(30) {
                0.2
            } else {
                0.05
            }
        }
        None => 0.05,
    }
}

/// Recompute a provider's reliability_score from all its replicas' proof
/// - results, then persist it to storage_providers.
async fn recompute_provider_reliability(
    state: &AppState,
    provider_id: Uuid,
) -> Result<f64, AppError> {
    let proofs: Vec<Option<chrono::DateTime<chrono::Utc>>> =
        sqlx::query_scalar("SELECT last_proof_at FROM provider_replicas WHERE provider_id = $1")
            .bind(provider_id)
            .fetch_all(&state.pool)
            .await
            .map_err(|e| AppError::internal(format!("replica proof query failed: {e}")))?;

    let score = if proofs.is_empty() {
        0.05
    } else {
        proofs
            .iter()
            .map(|p| reliability_from_proofs(*p))
            .sum::<f64>()
            / proofs.len() as f64
    };

    sqlx::query(
        "UPDATE storage_providers SET reliability_score = $1, updated_at = now() WHERE id = $2",
    )
    .bind(score)
    .bind(provider_id)
    .execute(&state.pool)
    .await
    .map_err(|e| AppError::internal(format!("reliability update failed: {e}")))?;

    Ok(score)
}

#[derive(Debug, Serialize)]
pub struct ChallengeResponse {
    pub challenge: String,
}

pub async fn challenge_replica(
    State(state): State<AppState>,
    Path((_provider_id, replica_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<ChallengeResponse>, AppError> {
    // Verify replica exists.
    let _replica: (Uuid,) = sqlx::query_as("SELECT id FROM provider_replicas WHERE id = $1")
        .bind(replica_id)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| AppError::internal(format!("query failed: {e}")))?
        .ok_or_else(|| AppError::not_found(format!("replica {replica_id} not found")))?;

    // Generate a random challenge.
    let challenge = Uuid::new_v4().to_string();

    // Store the challenge and clear any previous proof timestamp.
    sqlx::query(
        "UPDATE provider_replicas SET last_challenge = $1, last_proof_at = NULL WHERE id = $2",
    )
    .bind(&challenge)
    .bind(replica_id)
    .execute(&state.pool)
    .await
    .map_err(|e| AppError::internal(format!("store challenge failed: {e}")))?;

    Ok(Json(ChallengeResponse { challenge }))
}

/// Issue PoP challenges for replicas that are due (scheduled maintenance).
///
/// A replica is due when it is active/available, has no pending challenge, and
/// has not proven within the last hour (or never). Returns the number of
/// challenges issued. Called from the API's maintenance loop; the manual
/// `POST /providers/{id}/replicas/{id}/challenge` endpoint remains available.
pub async fn schedule_pop_challenges(state: &AppState, batch: i64) -> Result<u64, AppError> {
    let due: Vec<(Uuid,)> = sqlx::query_as(
        "SELECT pr.id FROM provider_replicas pr \
         JOIN storage_providers sp ON sp.id = pr.provider_id \
         WHERE pr.is_available = true AND sp.is_active = true \
           AND pr.last_challenge IS NULL \
           AND (pr.last_proof_at IS NULL OR pr.last_proof_at < now() - interval '1 hour') \
         ORDER BY pr.last_proof_at ASC NULLS FIRST \
         LIMIT $1",
    )
    .bind(batch)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| AppError::internal(format!("scheduled challenge query failed: {e}")))?;

    let mut issued = 0u64;
    for (replica_id,) in due {
        let challenge = Uuid::new_v4().to_string();
        let result = sqlx::query(
            "UPDATE provider_replicas SET last_challenge = $1, last_proof_at = NULL WHERE id = $2",
        )
        .bind(&challenge)
        .bind(replica_id)
        .execute(&state.pool)
        .await
        .map_err(|e| AppError::internal(format!("scheduled challenge store failed: {e}")))?;
        issued += result.rows_affected();
    }
    Ok(issued)
}

// ---------------------------------------------------------------------------
// POST /providers/{id}/replicas/{replica_id}/prove
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct ProveRequest {
    pub proof: String,
}

pub async fn prove_replica(
    State(state): State<AppState>,
    auth: SignedRequest,
    Path((_provider_id, replica_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<ProveRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    // Only the API authority may submit PoP proofs — prevents anyone who can
    // read the replica row from forging a SHA-256 legacy proof.
    if let Some(key) = &state.api_authority.0 {
        auth.verify(
            key,
            "POST",
            &format!("/api/v1/providers/{_provider_id}/replicas/{replica_id}/prove"),
        )?;
    } else {
        return Err(AppError::unauthorized("API authority not configured"));
    }

    // Verify replica exists and has a pending challenge.
    let (provider_id, content_address, last_challenge): (Uuid, Option<String>, Option<String>) =
        sqlx::query_as(
            "SELECT provider_id, content_address, last_challenge FROM provider_replicas WHERE id = $1",
        )
        .bind(replica_id)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| AppError::internal(format!("query failed: {e}")))?
        .ok_or_else(|| AppError::not_found(format!("replica {replica_id} not found")))?;

    let challenge = last_challenge
        .ok_or_else(|| AppError::bad_request("no pending challenge; call /challenge first"))?;

    // Compute expected payload: challenge + content_address bytes.
    let addr = content_address
        .ok_or_else(|| AppError::bad_request("replica has no content address; re-register it"))?;
    let payload = format!("{challenge}{addr}");

    // Verify the proof as an Ed25519 signature from the VRF oracle authority
    // over the payload bytes. When no oracle key is configured, fall back to
    // the legacy SHA-256(challenge + content_address) comparison.
    if let Some(oracle_key) = &state.vrf_oracle_pubkey {
        let sig_bytes =
            hex::decode(&req.proof).map_err(|_| AppError::bad_request("proof is not valid hex"))?;
        let sig: ed25519_dalek::Signature = sig_bytes
            .as_slice()
            .try_into()
            .map_err(|_| AppError::bad_request("proof is not a valid Ed25519 signature"))?;
        oracle_key.verify(payload.as_bytes(), &sig).map_err(|_| {
            AppError::bad_request("proof signature does not verify against oracle key")
        })?;
    } else {
        // Legacy fallback: SHA-256(challenge + content_address) comparison.
        let expected_proof = format!("{:x}", Sha256::digest(payload.as_bytes()));
        if req.proof != expected_proof {
            return Err(AppError::bad_request(
                "proof does not match expected digest",
            ));
        }
    }

    // Update last_proof_at and clear the challenge, then derive the
    // provider's reliability_score from challenge results (Q-008).
    sqlx::query(
        "UPDATE provider_replicas SET last_proof_at = now(), last_challenge = NULL WHERE id = $1",
    )
    .bind(replica_id)
    .execute(&state.pool)
    .await
    .map_err(|e| AppError::internal(format!("update failed: {e}")))?;

    let _score = recompute_provider_reliability(&state, provider_id).await?;

    Ok(Json(serde_json::json!({
        "status": "verified",
        "replica_id": replica_id,
        "proven_at": chrono::Utc::now(),
        "provider_reliability": _score,
    })))
}

// ---------------------------------------------------------------------------
// GET /providers/replication-audit — N-of-M replication compliance (Q-008)
//
// Audits every (component_id, component_version) pair and reports whether it
// is replicated across at least 2 independent provider classes. A pair with
// fewer than 2 distinct provider types is non-compliant: if that provider
// - fails, the artifact is unavailable.
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct ReplicationAuditEntry {
    pub component_id: String,
    pub component_version: String,
    pub replica_count: i64,
    pub provider_class_count: i64,
    pub provider_types: Vec<String>,
    pub compliant: bool,
}

#[derive(Debug, Serialize)]
pub struct ReplicationAuditReport {
    pub total_pairs: i64,
    pub compliant_pairs: i64,
    pub non_compliant_pairs: i64,
    pub entries: Vec<ReplicationAuditEntry>,
}

pub async fn replication_audit(
    State(state): State<AppState>,
) -> Result<Json<ReplicationAuditReport>, AppError> {
    let pairs: Vec<(String, String)> =
        sqlx::query_as("SELECT DISTINCT component_id, component_version FROM provider_replicas")
            .fetch_all(&state.pool)
            .await
            .map_err(|e| AppError::internal(format!("replica pair query failed: {e}")))?;

    let mut entries = Vec::new();
    let mut compliant = 0i64;
    let mut non_compliant = 0i64;

    for (component_id, component_version) in &pairs {
        let row: Option<(i64, i64, Vec<String>)> = sqlx::query_as(
            "SELECT \
                 count(*) AS replica_count, \
                 count(DISTINCT p.provider_type) AS provider_class_count, \
                 coalesce(array_agg(DISTINCT p.provider_type) FILTER (WHERE p.provider_type IS NOT NULL), '{}') AS provider_types \
             FROM provider_replicas r \
             JOIN storage_providers p ON p.id = r.provider_id \
             WHERE r.component_id = $1 AND r.component_version = $2 \
             GROUP BY r.component_id, r.component_version",
        )
        .bind(component_id)
        .bind(component_version)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| AppError::internal(format!("replication audit query failed: {e}")))?;

        let Some((replica_count, provider_class_count, provider_types)) = row else {
            // No replicas found for this pair (shouldn't happen since we got
            // the pair from the DISTINCT query, but skip defensively).
            continue;
        };
        let is_compliant = provider_class_count >= 2;
        if is_compliant {
            compliant += 1;
        } else {
            non_compliant += 1;
        }

        entries.push(ReplicationAuditEntry {
            component_id: component_id.clone(),
            component_version: component_version.clone(),
            replica_count,
            provider_class_count,
            provider_types,
            compliant: is_compliant,
        });
    }

    entries.sort_by_key(|a| std::cmp::Reverse(a.provider_class_count));

    Ok(Json(ReplicationAuditReport {
        total_pairs: pairs.len() as i64,
        compliant_pairs: compliant,
        non_compliant_pairs: non_compliant,
        entries,
    }))
}

// ---------------------------------------------------------------------------
// GET /authority-registries — IPFS mirror: authority registry list
// GET /authority-registries/{pubkey} — IPFS mirror: single registry
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct AuthorityRegistryRow {
    pub id: i64,
    pub pubkey: String,
    pub admin: String,
    pub validators: Vec<String>,
    pub required_endorsements: i16,
    pub mode: i16,
    pub version: i32,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

pub async fn list_authority_registries(
    State(state): State<AppState>,
) -> Result<Json<Vec<AuthorityRegistryRow>>, AppError> {
    let rows = sqlx::query_as::<_, AuthorityRegistryRow>(
        "SELECT id, pubkey, admin, validators, required_endorsements, mode, version, \
         created_at, updated_at FROM authority_registries ORDER BY created_at DESC",
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| AppError::internal(format!("authority registry query failed: {e}")))?;
    Ok(Json(rows))
}

pub async fn get_authority_registry(
    State(state): State<AppState>,
    Path(pubkey): Path<String>,
) -> Result<Json<AuthorityRegistryRow>, AppError> {
    let row = sqlx::query_as::<_, AuthorityRegistryRow>(
        "SELECT id, pubkey, admin, validators, required_endorsements, mode, version, \
         created_at, updated_at FROM authority_registries WHERE pubkey = $1",
    )
    .bind(&pubkey)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| AppError::internal(format!("authority registry query failed: {e}")))?
    .ok_or_else(|| AppError::not_found(format!("authority registry {pubkey} not found")))?;
    Ok(Json(row))
}

// ---------------------------------------------------------------------------
// GET /ipfs-docs — IPFS mirror: document anchor list
// GET /ipfs-docs/{cid} — IPFS mirror: single document anchor
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct IpfsDocRow {
    pub id: i64,
    pub attestation_pubkey: String,
    pub cid: String,
    pub content_hash: String,
    pub category: String,
    pub registered_by: String,
    pub registered_at: chrono::DateTime<chrono::Utc>,
}

pub async fn list_ipfs_docs(
    State(state): State<AppState>,
) -> Result<Json<Vec<IpfsDocRow>>, AppError> {
    let rows = sqlx::query_as::<_, IpfsDocRow>(
        "SELECT id, attestation_pubkey, cid, content_hash, category, registered_by, \
         registered_at FROM document_anchors ORDER BY registered_at DESC",
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| AppError::internal(format!("ipfs doc query failed: {e}")))?;
    Ok(Json(rows))
}

pub async fn get_ipfs_doc(
    State(state): State<AppState>,
    Path(cid): Path<String>,
) -> Result<Json<IpfsDocRow>, AppError> {
    let row = sqlx::query_as::<_, IpfsDocRow>(
        "SELECT id, attestation_pubkey, cid, content_hash, category, registered_by, \
         registered_at FROM document_anchors WHERE cid = $1",
    )
    .bind(&cid)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| AppError::internal(format!("ipfs doc query failed: {e}")))?
    .ok_or_else(|| AppError::not_found(format!("ipfs doc {cid} not found")))?;
    Ok(Json(row))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::ApiAuthority;
    use crate::storage::StorageBackend;
    use chrono::{Duration, Utc};
    use ed25519_dalek::{Signer, SigningKey, VerifyingKey};
    use std::sync::Arc;

    #[test]
    fn reliability_from_proofs_recent_proof_is_full() {
        let now = Utc::now();
        assert_eq!(
            reliability_from_proofs(Some(now)),
            1.0,
            "proof within 24h should be 1.0"
        );
    }

    #[test]
    fn reliability_from_proofs_week_old_proof_is_half() {
        let now = Utc::now();
        let week_ago = now - Duration::days(3) - Duration::hours(1);
        assert_eq!(
            reliability_from_proofs(Some(week_ago)),
            0.5,
            "proof 3-7 days old should be 0.5"
        );
    }

    #[test]
    fn reliability_from_proofs_month_old_proof_is_low() {
        let now = Utc::now();
        let weeks_ago = now - Duration::days(15);
        assert_eq!(
            reliability_from_proofs(Some(weeks_ago)),
            0.2,
            "proof 7-30 days old should be 0.2"
        );
    }

    #[test]
    fn reliability_from_proofs_stale_proof_is_minimal() {
        let now = Utc::now();
        let months_ago = now - Duration::days(60);
        assert_eq!(
            reliability_from_proofs(Some(months_ago)),
            0.05,
            "proof older than 30 days should be 0.05"
        );
    }

    #[test]
    fn reliability_from_proofs_no_proof_is_minimal() {
        assert_eq!(
            reliability_from_proofs(None),
            0.05,
            "no proof should be 0.05"
        );
    }

    #[tokio::test]
    async fn replication_audit_reports_compliance() {
        let Ok(database_url) = std::env::var("DATABASE_URL") else {
            eprintln!("skipping DB-backed replication audit test: DATABASE_URL is not set");
            return;
        };
        let pool = sqlx::PgPool::connect(&database_url)
            .await
            .expect("connect test database");

        // Clean up any leftover data from previous failed runs.
        sqlx::query("DELETE FROM provider_replicas WHERE component_id LIKE 'comp-audit-%'")
            .execute(&pool)
            .await
            .expect("clean up leftover replicas");
        sqlx::query("DELETE FROM storage_providers WHERE provider_id LIKE 'provider-a-%' OR provider_id LIKE 'provider-b-%'")
            .execute(&pool)
            .await
            .expect("clean up leftover providers");

        // Create two providers of different classes.
        let provider_a_id = Uuid::new_v4();
        let provider_b_id = Uuid::new_v4();
        let provider_a_pid = format!("provider-a-{}", provider_a_id);
        let provider_b_pid = format!("provider-b-{}", provider_b_id);
        let provider_a = sqlx::query_as::<_, ProviderRow>(
            "INSERT INTO storage_providers (id, provider_id, name, provider_type, base_url, capabilities, geographic_scope, reliability_score, cost_per_gb) \
             VALUES ($1, $2, $3, 'object_storage', $4, $5, $6, 1.0, NULL) \
             RETURNING id, provider_id, name, provider_type, base_url, capabilities, geographic_scope, is_active, reliability_score, cost_per_gb, stake_bond, wallet_address, payout_currency, created_at, updated_at",
        )
        .bind(provider_a_id)
        .bind(&provider_a_pid)
        .bind("Provider A")
        .bind("https://storage-a.example.com")
        .bind(serde_json::json!({}))
        .bind(&Vec::<String>::new())
        .fetch_one(&pool)
        .await
        .expect("insert provider A");

        let provider_b = sqlx::query_as::<_, ProviderRow>(
            "INSERT INTO storage_providers (id, provider_id, name, provider_type, base_url, capabilities, geographic_scope, reliability_score, cost_per_gb) \
             VALUES ($1, $2, $3, 'ipfs', $4, $5, $6, 1.0, NULL) \
             RETURNING id, provider_id, name, provider_type, base_url, capabilities, geographic_scope, is_active, reliability_score, cost_per_gb, stake_bond, wallet_address, payout_currency, created_at, updated_at",
        )
        .bind(provider_b_id)
        .bind(&provider_b_pid)
        .bind("Provider B")
        .bind("https://ipfs-b.example.com")
        .bind(serde_json::json!({}))
        .bind(&Vec::<String>::new())
        .fetch_one(&pool)
        .await
        .expect("insert provider B");

        // Register component-1 on both providers (compliant: 2 classes).
        let comp1_id = format!("comp-audit-1-{}", Uuid::new_v4());
        let comp2_id = format!("comp-audit-2-{}", Uuid::new_v4());
        for provider in [&provider_a, &provider_b] {
            sqlx::query(
                "INSERT INTO provider_replicas (provider_id, component_id, component_version, storage_reference, version, size_bytes, checksum, content_address, latency_ms, is_available) \
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, true)",
            )
            .bind(provider.id)
            .bind(&comp1_id)
            .bind("v1")
            .bind("https://storage.example.com/comp-1.glb")
            .bind("1")
            .bind(1000i64)
            .bind("sha256:abcdef")
            .bind("sha256:abcdef")
            .bind(50i32)
            .execute(&pool)
            .await
            .expect("insert replica");
        }

        // Register component-2 on only provider A (non-compliant: 1 class).
        sqlx::query(
            "INSERT INTO provider_replicas (provider_id, component_id, component_version, storage_reference, version, size_bytes, checksum, content_address, latency_ms, is_available) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, true)",
        )
        .bind(provider_a.id)
        .bind(&comp2_id)
        .bind("v1")
        .bind("https://storage.example.com/comp-2.glb")
        .bind("1")
        .bind(1000i64)
        .bind("sha256:bbbbbb")
        .bind("sha256:bbbbbb")
        .bind(50i32)
        .execute(&pool)
        .await
        .expect("insert replica");

        // Run the audit.
        let dir = tempfile::tempdir().expect("temp storage dir");
        let state = AppState {
            pool: pool.clone(),
            geo: None,
            api_authority: ApiAuthority(Some(VerifyingKey::from(&SigningKey::from_bytes(
                &[13u8; 32],
            )))),
            vrf_oracle_pubkey: None,
            storage: Arc::new(StorageBackend::Local {
                root: dir.path().to_path_buf(),
            }),
            zk: None,
        };

        let report = replication_audit(State(state))
            .await
            .expect("replication audit");

        // Find our two test components in the report (other tests may also
        // have replicas, so we look up by our component IDs rather than
        // asserting on total counts).
        let comp1 = report
            .entries
            .iter()
            .find(|e| e.component_id == comp1_id)
            .unwrap();
        assert!(
            comp1.compliant,
            "comp-1 should be compliant (2 provider classes)"
        );
        assert_eq!(comp1.provider_class_count, 2);

        let comp2 = report
            .entries
            .iter()
            .find(|e| e.component_id == comp2_id)
            .unwrap();
        assert!(
            !comp2.compliant,
            "comp-2 should be non-compliant (1 provider class)"
        );
        assert_eq!(comp2.provider_class_count, 1);

        // Clean up.
        sqlx::query("DELETE FROM provider_replicas WHERE component_id IN ($1, $2)")
            .bind(&comp1_id)
            .bind(&comp2_id)
            .execute(&pool)
            .await
            .expect("clean up replicas");
        sqlx::query("DELETE FROM storage_providers WHERE id IN ($1, $2)")
            .bind(provider_a_id)
            .bind(provider_b_id)
            .execute(&pool)
            .await
            .expect("clean up providers");
    }

    #[tokio::test]
    async fn prove_replica_updates_provider_reliability() {
        let Ok(database_url) = std::env::var("DATABASE_URL") else {
            eprintln!("skipping DB-backed prove replica test: DATABASE_URL is not set");
            return;
        };
        let pool = sqlx::PgPool::connect(&database_url)
            .await
            .expect("connect test database");

        // Clean up leftovers from a previous failed run. `provider_id` is
        // UNIQUE, so reusing a fixed id would fail on the second run.
        sqlx::query(
            "DELETE FROM provider_replicas WHERE component_id LIKE 'component-reliability%'",
        )
        .execute(&pool)
        .await
        .expect("clean up leftover replicas");
        sqlx::query("DELETE FROM storage_providers WHERE provider_id LIKE 'provider-reliability%'")
            .execute(&pool)
            .await
            .expect("clean up leftover providers");

        // Create a provider.
        let provider_id = Uuid::new_v4();
        let provider_pid = format!("provider-reliability-{}", provider_id);
        let provider = sqlx::query_as::<_, ProviderRow>(
            "INSERT INTO storage_providers (id, provider_id, name, provider_type, base_url, capabilities, geographic_scope, reliability_score, cost_per_gb) \
             VALUES ($1, $2, $3, 'object_storage', $4, $5, $6, 1.0, NULL) \
              RETURNING id, provider_id, name, provider_type, base_url, capabilities, geographic_scope, is_active, reliability_score, cost_per_gb, stake_bond, wallet_address, payout_currency, created_at, updated_at",
        )
        .bind(provider_id)
        .bind(&provider_pid)
        .bind("Reliability Provider")
        .bind("https://storage.example.com")
        .bind(serde_json::json!({}))
        .bind(&Vec::<String>::new())
        .fetch_one(&pool)
        .await
        .expect("insert provider");

        // Create a replica with content_address and a pending challenge.
        let replica = sqlx::query_as::<_, ReplicaRow>(
            "INSERT INTO provider_replicas (provider_id, component_id, component_version, storage_reference, version, size_bytes, checksum, content_address, latency_ms, is_available, last_challenge) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, true, $10) \
             RETURNING id, provider_id, component_id, component_version, storage_reference, version, size_bytes, checksum, content_address, latency_ms, is_available, last_checked_at, last_challenge, last_proof_at, access_count, bytes_served, created_at",
        )
        .bind(provider.id)
        .bind("component-reliability")
        .bind("v1")
        .bind("https://storage.example.com/comp.glb")
        .bind("1")
        .bind(1000i64)
        .bind("sha256:expected")
        .bind("sha256:expected")
        .bind(50i32)
        .bind("challenge-123")
        .fetch_one(&pool)
        .await
        .expect("insert replica");

        let expected_proof = format!(
            "{:x}",
            Sha256::digest("challenge-123sha256:expected".as_bytes())
        );

        let dir = tempfile::tempdir().expect("temp storage dir");
        let state = AppState {
            pool: pool.clone(),
            geo: None,
            api_authority: ApiAuthority(Some(VerifyingKey::from(&SigningKey::from_bytes(
                &[13u8; 32],
            )))),
            vrf_oracle_pubkey: None,
            storage: Arc::new(StorageBackend::Local {
                root: dir.path().to_path_buf(),
            }),
            zk: None,
        };

        // Prove the replica (requires a valid SignedRequest against the API
        // authority key configured in the test state).
        let body = format!(r#"{{"proof":"{}"}}"#, expected_proof);
        let body_hash = Sha256::digest(body.as_bytes());
        let body_hash_hex = hex::encode(body_hash);
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time before epoch")
            .as_secs() as i64;
        let payload = format!(
            "POST\n/api/v1/providers/{}/replicas/{}/prove\n{}\n{}",
            provider.id, replica.id, body_hash_hex, timestamp
        );
        let signing_key = SigningKey::from_bytes(&[13u8; 32]);
        let signature = signing_key.sign(payload.as_bytes());
        let auth = SignedRequest {
            signature,
            timestamp,
            body_hash_hex,
        };

        let result = prove_replica(
            State(state.clone()),
            auth,
            Path((provider.id, replica.id)),
            Json(ProveRequest {
                proof: expected_proof.clone(),
            }),
        )
        .await
        .expect("prove replica");

        let status = result.get("status").unwrap().as_str().unwrap();
        assert_eq!(status, "verified");

        // Verify provider reliability was updated (should be 1.0 with a recent proof).
        let updated: Option<f64> =
            sqlx::query_scalar("SELECT reliability_score FROM storage_providers WHERE id = $1")
                .bind(provider.id)
                .fetch_optional(&pool)
                .await
                .expect("fetch provider reliability");
        assert!(
            updated.unwrap() >= 0.9,
            "provider reliability should be high after successful proof, got {:?}",
            updated
        );

        // Verify last_proof_at was set.
        let proof_at: Option<chrono::DateTime<chrono::Utc>> =
            sqlx::query_scalar("SELECT last_proof_at FROM provider_replicas WHERE id = $1")
                .bind(replica.id)
                .fetch_optional(&pool)
                .await
                .expect("fetch last_proof_at");
        assert!(
            proof_at.is_some(),
            "last_proof_at should be set after successful proof"
        );

        // Clean up.
        sqlx::query("DELETE FROM provider_replicas WHERE component_id = $1")
            .bind("component-reliability")
            .execute(&pool)
            .await
            .expect("clean up replica");
        sqlx::query("DELETE FROM storage_providers WHERE id = $1")
            .bind(provider_id)
            .execute(&pool)
            .await
            .expect("clean up provider");
    }

    #[tokio::test]
    async fn scheduled_pop_challenges_challenge_due_replicas_once() {
        let Ok(database_url) = std::env::var("DATABASE_URL") else {
            eprintln!("skipping DB-backed scheduled PoP test: DATABASE_URL is not set");
            return;
        };
        let pool = sqlx::PgPool::connect(&database_url)
            .await
            .expect("connect test database");
        let dir = tempfile::tempdir().expect("temp storage dir");
        let state = AppState {
            pool: pool.clone(),
            geo: None,
            api_authority: ApiAuthority(None),
            vrf_oracle_pubkey: None,
            storage: Arc::new(StorageBackend::Local {
                root: dir.path().to_path_buf(),
            }),
            zk: None,
        };

        let provider_id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO storage_providers
                (id, provider_id, name, provider_type, base_url, reliability_score)
             VALUES ($1, $2, 'PoP Scheduler Provider', 'object_storage',
                     'https://provider.example.com', 0.9)",
        )
        .bind(provider_id)
        .bind(format!("pop-schedule-provider-{}", Uuid::new_v4()))
        .execute(&pool)
        .await
        .expect("insert provider");

        // Never proven and no pending challenge -> due for a challenge.
        let replica_id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO provider_replicas
                (id, provider_id, component_id, component_version, storage_reference,
                 version, checksum, latency_ms, is_available, last_challenge, last_proof_at)
             VALUES ($1, $2, $3, 'v1', 'https://provider.example.com/pop.glb',
                     'v1', $4, 40, true, NULL, NULL)",
        )
        .bind(replica_id)
        .bind(provider_id)
        .bind(format!("pop-schedule-component-{}", Uuid::new_v4()))
        .bind(format!("sha256:{}", "ab".repeat(32)))
        .execute(&pool)
        .await
        .expect("insert replica");

        let issued = schedule_pop_challenges(&state, 500)
            .await
            .expect("schedule PoP challenges");
        assert!(issued >= 1, "a due replica must receive a challenge");

        let challenge: Option<String> =
            sqlx::query_scalar("SELECT last_challenge FROM provider_replicas WHERE id = $1")
                .bind(replica_id)
                .fetch_one(&pool)
                .await
                .expect("read challenge");
        assert!(
            challenge.is_some(),
            "scheduled maintenance must set last_challenge"
        );

        // A second sweep must not reissue a challenge while one is pending.
        let again = schedule_pop_challenges(&state, 500)
            .await
            .expect("second schedule sweep");
        let unchanged: Option<String> =
            sqlx::query_scalar("SELECT last_challenge FROM provider_replicas WHERE id = $1")
                .bind(replica_id)
                .fetch_one(&pool)
                .await
                .expect("read challenge again");
        assert_eq!(
            unchanged, challenge,
            "pending challenges must not be replaced (issued {} in second sweep)",
            again
        );

        sqlx::query("DELETE FROM storage_providers WHERE id = $1")
            .bind(provider_id)
            .execute(&pool)
            .await
            .expect("clean up provider");
    }

    #[tokio::test]
    async fn payout_currency_must_be_an_enabled_asset() {
        use crate::auth::ApiAuthority;
        use crate::state::AppState;
        use crate::storage::StorageBackend;
        use sqlx::PgPool;
        use std::sync::Arc;

        let Ok(database_url) = std::env::var("DATABASE_URL") else {
            eprintln!("skipping DB-backed provider test: DATABASE_URL is not set");
            return;
        };
        let pool = PgPool::connect(&database_url)
            .await
            .expect("connect test database");
        let dir = tempfile::tempdir().expect("temp storage dir");
        let state = AppState {
            pool,
            geo: None,
            api_authority: ApiAuthority(None),
            vrf_oracle_pubkey: None,
            storage: Arc::new(StorageBackend::Local {
                root: dir.path().to_path_buf(),
            }),
            zk: None,
        };

        // The native default and explicit registry rows are accepted.
        ensure_enabled_settlement_asset(&state, None)
            .await
            .expect("null means default");
        ensure_enabled_settlement_asset(&state, Some("SOL"))
            .await
            .expect("SOL is enabled");
        // Unknown symbols are rejected before they reach the foreign key.
        assert!(
            ensure_enabled_settlement_asset(&state, Some("NOTAREALASSET"))
                .await
                .is_err()
        );
    }
}
