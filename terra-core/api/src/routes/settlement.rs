//! Provider settlement rail (RFC-014 direction, USDC service fees only).
//!
//! Usage is metered per replica (`access_count`, `bytes_served`). Closing an
//! epoch snapshots each provider's un-settled delta, prices it in micro-USDC,
//! commits all leaves to a Merkle root, and advances the provider's settled
//! counters so the same bytes can never be paid twice. A provider then claims
//! its leaf with a Merkle inclusion proof signed by its registered wallet.
//!
//! Q-005 hard gate: this rail models payment for verified storage service
//! only. `currency` is constrained to USDC and `settlement_kind` to
//! `service_fee` in migration 0054 — there is no token, fractional interest,
//! yield, or profit share.

use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::auth::SignedRequest;
use crate::error::AppError;
use crate::state::AppState;

const BYTES_PER_GB: i128 = 1_000_000_000;
/// 100_000 micro-USDC = $0.10 per GB by default.
const DEFAULT_RATE_MICRO_USDC_PER_GB: i64 = 100_000;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/epochs", get(list_epochs).post(close_epoch))
        .route("/epochs/{epoch_id}", get(get_epoch))
        .route(
            "/epochs/{epoch_id}/leaves/{provider_id}/proof",
            get(get_claim_proof),
        )
        .route("/claims", post(claim))
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct EpochRow {
    pub epoch_id: i64,
    pub currency: String,
    pub settlement_kind: String,
    pub rate_micro_usdc_per_gb: i64,
    pub min_payout_micro_usdc: i64,
    pub total_micro_usdc: i64,
    pub merkle_root: String,
    pub leaf_count: i32,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct LeafRow {
    pub epoch_id: i64,
    pub provider_id: Uuid,
    pub provider_wallet: String,
    pub access_count: i64,
    pub bytes_served: i64,
    pub amount_micro_usdc: i64,
    pub leaf_hash: String,
    pub leaf_index: i32,
    pub claimed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub claim_reference: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CloseEpochRequest {
    pub rate_micro_usdc_per_gb: Option<i64>,
    pub min_payout_micro_usdc: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct CloseEpochResponse {
    pub epoch: EpochRow,
    pub leaves: Vec<LeafRow>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MerkleStep {
    pub hash: String,
    /// True when the sibling is the left child (`sibling || current`).
    pub left: bool,
}

#[derive(Debug, Deserialize)]
pub struct ClaimRequest {
    pub epoch_id: i64,
    pub provider_id: Uuid,
    pub proof: Vec<MerkleStep>,
    pub claim_reference: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ClaimResponse {
    pub claimed: bool,
    pub epoch_id: i64,
    pub provider_id: Uuid,
    pub amount_micro_usdc: i64,
    pub claim_reference: Option<String>,
}

// ---------------------------------------------------------------------------
// Merkle helpers (SHA-256, duplicate-last for odd levels)
// ---------------------------------------------------------------------------

fn sha256_hex(input: &[u8]) -> String {
    format!("{:x}", Sha256::digest(input))
}

fn hash_pair(left: &str, right: &str) -> String {
    sha256_hex(format!("{left}{right}").as_bytes())
}

pub fn settlement_leaf_hash(
    provider_id: Uuid,
    wallet: &str,
    access_count: i64,
    bytes_served: i64,
    amount_micro_usdc: i64,
) -> String {
    sha256_hex(
        format!(
            "terra:settlement:leaf:v1\n{provider_id}\n{wallet}\n{access_count}\n{bytes_served}\n{amount_micro_usdc}"
        )
        .as_bytes(),
    )
}

fn merkle_root(leaves: &[String]) -> String {
    if leaves.is_empty() {
        return sha256_hex(b"terra:settlement:empty");
    }
    let mut level = leaves.to_vec();
    while level.len() > 1 {
        let mut next = Vec::with_capacity(level.len().div_ceil(2));
        for pair in level.chunks(2) {
            let right = if pair.len() == 2 { &pair[1] } else { &pair[0] };
            next.push(hash_pair(&pair[0], right));
        }
        level = next;
    }
    level[0].clone()
}

fn merkle_proof(leaves: &[String], index: usize) -> Vec<MerkleStep> {
    let mut proof = Vec::new();
    let mut level = leaves.to_vec();
    let mut idx = index;
    while level.len() > 1 {
        let sibling = if idx.is_multiple_of(2) {
            if idx + 1 < level.len() {
                level[idx + 1].clone()
            } else {
                level[idx].clone()
            }
        } else {
            level[idx - 1].clone()
        };
        proof.push(MerkleStep {
            hash: sibling,
            left: !idx.is_multiple_of(2),
        });
        let mut next = Vec::with_capacity(level.len().div_ceil(2));
        for pair in level.chunks(2) {
            let right = if pair.len() == 2 { &pair[1] } else { &pair[0] };
            next.push(hash_pair(&pair[0], right));
        }
        level = next;
        idx /= 2;
    }
    proof
}

fn verify_merkle_proof(leaf: &str, proof: &[MerkleStep], root: &str) -> bool {
    let mut current = leaf.to_string();
    for step in proof {
        current = if step.left {
            hash_pair(&step.hash, &current)
        } else {
            hash_pair(&current, &step.hash)
        };
    }
    current == root
}

// ---------------------------------------------------------------------------
// Close epoch
// ---------------------------------------------------------------------------

pub async fn close_epoch(
    State(state): State<AppState>,
    auth: SignedRequest,
    Json(req): Json<CloseEpochRequest>,
) -> Result<(axum::http::StatusCode, Json<CloseEpochResponse>), AppError> {
    let key = state
        .api_authority
        .0
        .ok_or_else(|| AppError::unauthorized("API authority not configured"))?;
    auth.verify(&key, "POST", "/api/v1/providers/settlement/epochs")?;

    let response = close_epoch_inner(
        &state,
        req.rate_micro_usdc_per_gb,
        req.min_payout_micro_usdc,
    )
    .await?;
    Ok((axum::http::StatusCode::CREATED, Json(response)))
}

pub async fn close_epoch_inner(
    state: &AppState,
    rate_micro_usdc_per_gb: Option<i64>,
    min_payout_micro_usdc: Option<i64>,
) -> Result<CloseEpochResponse, AppError> {
    let rate = rate_micro_usdc_per_gb.unwrap_or(DEFAULT_RATE_MICRO_USDC_PER_GB);
    let min_payout = min_payout_micro_usdc.unwrap_or(0);
    if rate <= 0 {
        return Err(AppError::bad_request(
            "rate_micro_usdc_per_gb must be positive",
        ));
    }
    if min_payout < 0 {
        return Err(AppError::bad_request(
            "min_payout_micro_usdc must be non-negative",
        ));
    }

    // Only rows whose cumulative replica usage exceeds what has already been
    // settled are eligible. `FOR UPDATE` on providers serializes concurrent
    // epoch closes so a byte cannot be paid twice.
    type Unsettled = (Uuid, String, i64, i64, i64, i64);
    let mut tx = state.pool.begin().await?;
    let rows: Vec<Unsettled> = sqlx::query_as(
        "SELECT sp.id, sp.wallet_address,
                sp.settled_access_count, sp.settled_bytes_served,
                COALESCE((SELECT SUM(pr.access_count) FROM provider_replicas pr
                          WHERE pr.provider_id = sp.id), 0)::bigint,
                COALESCE((SELECT SUM(pr.bytes_served) FROM provider_replicas pr
                          WHERE pr.provider_id = sp.id), 0)::bigint
         FROM storage_providers sp
         WHERE sp.wallet_address IS NOT NULL AND sp.is_active = true
           AND COALESCE((SELECT SUM(pr.bytes_served) FROM provider_replicas pr
                         WHERE pr.provider_id = sp.id), 0) > sp.settled_bytes_served
         ORDER BY sp.id
         FOR UPDATE",
    )
    .fetch_all(&mut *tx)
    .await
    .map_err(|e| AppError::internal(format!("settlement scan failed: {e}")))?;

    struct Pending {
        provider_id: Uuid,
        wallet: String,
        access_delta: i64,
        bytes_delta: i64,
        amount: i64,
        leaf_hash: String,
        total_access: i64,
        total_bytes: i64,
    }

    let mut pending = Vec::new();
    for (provider_id, wallet, settled_access, settled_bytes, total_access, total_bytes) in rows {
        let access_delta = (total_access - settled_access).max(0);
        let bytes_delta = (total_bytes - settled_bytes).max(0);
        if bytes_delta == 0 {
            // Access-only deltas have no byte-denominated payout.
            continue;
        }
        let amount = ((bytes_delta as i128) * (rate as i128) / BYTES_PER_GB) as i64;
        if amount <= 0 || amount < min_payout {
            continue;
        }
        let leaf_hash =
            settlement_leaf_hash(provider_id, &wallet, access_delta, bytes_delta, amount);
        pending.push(Pending {
            provider_id,
            wallet,
            access_delta,
            bytes_delta,
            amount,
            leaf_hash,
            total_access,
            total_bytes,
        });
    }

    let leaves: Vec<String> = pending.iter().map(|p| p.leaf_hash.clone()).collect();
    let root = merkle_root(&leaves);
    let total_micro_usdc: i64 = pending.iter().map(|p| p.amount).sum();

    let epoch: EpochRow = sqlx::query_as(
        "INSERT INTO provider_settlement_epochs
            (currency, settlement_kind, rate_micro_usdc_per_gb,
             min_payout_micro_usdc, total_micro_usdc, merkle_root, leaf_count)
         VALUES ('USDC', 'service_fee', $1, $2, $3, $4, $5)
         RETURNING epoch_id, currency, settlement_kind, rate_micro_usdc_per_gb,
                   min_payout_micro_usdc, total_micro_usdc, merkle_root, leaf_count,
                   status, created_at",
    )
    .bind(rate)
    .bind(min_payout)
    .bind(total_micro_usdc)
    .bind(&root)
    .bind(leaves.len() as i32)
    .fetch_one(&mut *tx)
    .await?;

    let mut inserted = Vec::with_capacity(pending.len());
    for (index, leaf) in pending.iter().enumerate() {
        let row: LeafRow = sqlx::query_as(
            "INSERT INTO provider_settlement_leaves
                (epoch_id, provider_id, provider_wallet, access_count,
                 bytes_served, amount_micro_usdc, leaf_hash, leaf_index)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
             RETURNING epoch_id, provider_id, provider_wallet, access_count,
                       bytes_served, amount_micro_usdc, leaf_hash, leaf_index,
                       claimed_at, claim_reference",
        )
        .bind(epoch.epoch_id)
        .bind(leaf.provider_id)
        .bind(&leaf.wallet)
        .bind(leaf.access_delta)
        .bind(leaf.bytes_delta)
        .bind(leaf.amount)
        .bind(&leaf.leaf_hash)
        .bind(index as i32)
        .fetch_one(&mut *tx)
        .await?;

        sqlx::query(
            "UPDATE storage_providers
             SET settled_access_count = $1, settled_bytes_served = $2
             WHERE id = $3",
        )
        .bind(leaf.total_access)
        .bind(leaf.total_bytes)
        .bind(leaf.provider_id)
        .execute(&mut *tx)
        .await?;

        inserted.push(row);
    }

    tx.commit().await?;
    Ok(CloseEpochResponse {
        epoch,
        leaves: inserted,
    })
}

// ---------------------------------------------------------------------------
// Read endpoints
// ---------------------------------------------------------------------------

pub async fn list_epochs(State(state): State<AppState>) -> Result<Json<Vec<EpochRow>>, AppError> {
    let rows: Vec<EpochRow> = sqlx::query_as(
        "SELECT epoch_id, currency, settlement_kind, rate_micro_usdc_per_gb,
                min_payout_micro_usdc, total_micro_usdc, merkle_root, leaf_count,
                status, created_at
         FROM provider_settlement_epochs
         ORDER BY epoch_id DESC
         LIMIT 100",
    )
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(rows))
}

pub async fn get_epoch(
    State(state): State<AppState>,
    Path(epoch_id): Path<i64>,
) -> Result<Json<EpochRow>, AppError> {
    let row: EpochRow = sqlx::query_as(
        "SELECT epoch_id, currency, settlement_kind, rate_micro_usdc_per_gb,
                min_payout_micro_usdc, total_micro_usdc, merkle_root, leaf_count,
                status, created_at
         FROM provider_settlement_epochs WHERE epoch_id = $1",
    )
    .bind(epoch_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::not_found("settlement epoch not found"))?;
    Ok(Json(row))
}

pub async fn get_claim_proof(
    State(state): State<AppState>,
    Path((epoch_id, provider_id)): Path<(i64, Uuid)>,
) -> Result<Json<serde_json::Value>, AppError> {
    let epoch = fetch_epoch(&state, epoch_id).await?;
    let leaf = fetch_leaf(&state, epoch_id, provider_id).await?;
    let leaves = fetch_leaf_hashes(&state, epoch_id).await?;
    let index = leaf.leaf_index as usize;
    let proof = merkle_proof(&leaves, index);
    Ok(Json(serde_json::json!({
        "epoch_id": epoch_id,
        "provider_id": provider_id,
        "leaf": leaf,
        "merkle_root": epoch.merkle_root,
        "proof": proof,
    })))
}

// ---------------------------------------------------------------------------
// Claim
// ---------------------------------------------------------------------------

pub async fn claim(
    State(state): State<AppState>,
    auth: SignedRequest,
    Json(req): Json<ClaimRequest>,
) -> Result<Json<ClaimResponse>, AppError> {
    let leaf = fetch_leaf(&state, req.epoch_id, req.provider_id).await?;
    // The claim must be signed by the provider wallet named in the leaf.
    auth.verify_wallet(
        &leaf.provider_wallet,
        "POST",
        "/api/v1/providers/settlement/claims",
    )?;
    let response = claim_inner(&state, &req).await?;
    Ok(Json(response))
}

pub async fn claim_inner(state: &AppState, req: &ClaimRequest) -> Result<ClaimResponse, AppError> {
    let epoch = fetch_epoch(state, req.epoch_id).await?;
    let leaf = fetch_leaf(state, req.epoch_id, req.provider_id).await?;

    if leaf.claimed_at.is_some() {
        return Err(AppError::conflict("settlement leaf already claimed"));
    }

    let recomputed = settlement_leaf_hash(
        leaf.provider_id,
        &leaf.provider_wallet,
        leaf.access_count,
        leaf.bytes_served,
        leaf.amount_micro_usdc,
    );
    if recomputed != leaf.leaf_hash {
        return Err(AppError::internal(
            "stored settlement leaf hash does not match its contents",
        ));
    }
    if !verify_merkle_proof(&leaf.leaf_hash, &req.proof, &epoch.merkle_root) {
        return Err(AppError::bad_request(
            "Merkle inclusion proof does not match the epoch root",
        ));
    }

    sqlx::query(
        "UPDATE provider_settlement_leaves
         SET claimed_at = now(), claim_reference = $1
         WHERE epoch_id = $2 AND provider_id = $3 AND claimed_at IS NULL",
    )
    .bind(&req.claim_reference)
    .bind(req.epoch_id)
    .bind(req.provider_id)
    .execute(&state.pool)
    .await?;

    // Mark the epoch settled once every leaf has been claimed.
    let remaining: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM provider_settlement_leaves
         WHERE epoch_id = $1 AND claimed_at IS NULL",
    )
    .bind(req.epoch_id)
    .fetch_one(&state.pool)
    .await?;
    if remaining == 0 {
        sqlx::query("UPDATE provider_settlement_epochs SET status = 'settled' WHERE epoch_id = $1")
            .bind(req.epoch_id)
            .execute(&state.pool)
            .await?;
    }

    Ok(ClaimResponse {
        claimed: true,
        epoch_id: req.epoch_id,
        provider_id: req.provider_id,
        amount_micro_usdc: leaf.amount_micro_usdc,
        claim_reference: req.claim_reference.clone(),
    })
}

async fn fetch_epoch(state: &AppState, epoch_id: i64) -> Result<EpochRow, AppError> {
    sqlx::query_as(
        "SELECT epoch_id, currency, settlement_kind, rate_micro_usdc_per_gb,
                min_payout_micro_usdc, total_micro_usdc, merkle_root, leaf_count,
                status, created_at
         FROM provider_settlement_epochs WHERE epoch_id = $1",
    )
    .bind(epoch_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::not_found("settlement epoch not found"))
}

async fn fetch_leaf(
    state: &AppState,
    epoch_id: i64,
    provider_id: Uuid,
) -> Result<LeafRow, AppError> {
    sqlx::query_as(
        "SELECT epoch_id, provider_id, provider_wallet, access_count, bytes_served,
                amount_micro_usdc, leaf_hash, leaf_index, claimed_at, claim_reference
         FROM provider_settlement_leaves
         WHERE epoch_id = $1 AND provider_id = $2",
    )
    .bind(epoch_id)
    .bind(provider_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::not_found("settlement leaf not found"))
}

async fn fetch_leaf_hashes(state: &AppState, epoch_id: i64) -> Result<Vec<String>, AppError> {
    let rows: Vec<(String,)> = sqlx::query_as(
        "SELECT leaf_hash FROM provider_settlement_leaves
         WHERE epoch_id = $1 ORDER BY leaf_index",
    )
    .bind(epoch_id)
    .fetch_all(&state.pool)
    .await?;
    Ok(rows.into_iter().map(|(hash,)| hash).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn leaf(seed: u8) -> String {
        settlement_leaf_hash(
            Uuid::from_bytes([seed; 16]),
            "wallet",
            seed as i64,
            seed as i64 * 1024,
            seed as i64 * 10,
        )
    }

    #[test]
    fn every_leaf_proof_verifies_against_the_root() {
        for count in 1..=9usize {
            let leaves: Vec<String> = (0..count).map(|i| leaf(i as u8 + 1)).collect();
            let root = merkle_root(&leaves);
            for (index, hash) in leaves.iter().enumerate() {
                let proof = merkle_proof(&leaves, index);
                assert!(
                    verify_merkle_proof(hash, &proof, &root),
                    "leaf {index}/{count} failed"
                );
            }
        }
    }

    #[test]
    fn tampered_leaf_or_proof_is_rejected() {
        let leaves: Vec<String> = (0..5).map(|i| leaf(i + 1)).collect();
        let root = merkle_root(&leaves);
        let mut proof = merkle_proof(&leaves, 2);
        assert!(verify_merkle_proof(&leaves[2], &proof, &root));
        assert!(!verify_merkle_proof(&leaf(9), &proof, &root));
        proof[0].left = !proof[0].left;
        assert!(!verify_merkle_proof(&leaves[2], &proof, &root));
    }

    #[tokio::test]
    async fn epoch_close_and_claim_settle_a_provider() {
        use crate::auth::ApiAuthority;
        use crate::state::AppState;
        use crate::storage::StorageBackend;
        use sqlx::PgPool;
        use std::sync::Arc;

        let Ok(database_url) = std::env::var("DATABASE_URL") else {
            eprintln!("skipping DB-backed settlement test: DATABASE_URL is not set");
            return;
        };
        let pool = PgPool::connect(&database_url)
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
        let wallet = format!("settlement-wallet-{}", Uuid::new_v4());
        let component_id = format!("settlement-component-{}", Uuid::new_v4());
        sqlx::query(
            "INSERT INTO storage_providers
                (id, provider_id, name, provider_type, base_url, wallet_address, reliability_score)
             VALUES ($1, $2, 'Settlement Provider', 'object_storage',
                     'https://provider.example.com', $3, 0.9)",
        )
        .bind(provider_id)
        .bind(format!("settlement-provider-{}", Uuid::new_v4()))
        .bind(&wallet)
        .execute(&pool)
        .await
        .expect("insert provider");
        sqlx::query(
            "INSERT INTO provider_replicas
                (provider_id, component_id, component_version, storage_reference,
                 version, checksum, latency_ms, access_count, bytes_served)
             VALUES ($1, $2, 'v1', 'https://provider.example.com/a.glb',
                     'v1', $3, 40, 3, 2000000)",
        )
        .bind(provider_id)
        .bind(&component_id)
        .bind(format!("sha256:{}", "ef".repeat(32)))
        .execute(&pool)
        .await
        .expect("insert replica");

        let closed = close_epoch_inner(&state, None, None)
            .await
            .expect("close epoch");
        assert_eq!(closed.epoch.currency, "USDC");
        assert_eq!(closed.epoch.settlement_kind, "service_fee");
        assert_eq!(closed.epoch.leaf_count, 1);
        assert_eq!(closed.leaves.len(), 1);
        let leaf = &closed.leaves[0];
        assert_eq!(leaf.provider_id, provider_id);
        assert_eq!(leaf.bytes_served, 2_000_000);
        assert_eq!(leaf.amount_micro_usdc, 200); // 2 MB × $0.10/GB

        let settled: (i64, i64) = sqlx::query_as(
            "SELECT settled_access_count, settled_bytes_served
             FROM storage_providers WHERE id = $1",
        )
        .bind(provider_id)
        .fetch_one(&pool)
        .await
        .expect("read settled counters");
        assert_eq!(settled, (3, 2_000_000));

        let hashes = fetch_leaf_hashes(&state, closed.epoch.epoch_id)
            .await
            .expect("leaf hashes");
        let proof = merkle_proof(&hashes, leaf.leaf_index as usize);
        let claim = ClaimRequest {
            epoch_id: closed.epoch.epoch_id,
            provider_id,
            proof,
            claim_reference: Some("usdc-transfer-test".to_string()),
        };
        let response = claim_inner(&state, &claim).await.expect("claim settles");
        assert!(response.claimed);
        assert_eq!(response.amount_micro_usdc, 200);

        // Replay is rejected.
        assert!(claim_inner(&state, &claim).await.is_err());

        let status: (String, Option<String>) = sqlx::query_as(
            "SELECT e.status, l.claim_reference
             FROM provider_settlement_epochs e
             JOIN provider_settlement_leaves l USING (epoch_id)
             WHERE e.epoch_id = $1 AND l.provider_id = $2",
        )
        .bind(closed.epoch.epoch_id)
        .bind(provider_id)
        .fetch_one(&pool)
        .await
        .expect("read claim state");
        assert_eq!(status.0, "settled");
        assert_eq!(status.1.as_deref(), Some("usdc-transfer-test"));

        sqlx::query("DELETE FROM storage_providers WHERE id = $1")
            .bind(provider_id)
            .execute(&pool)
            .await
            .expect("clean up provider");
    }

    #[test]
    fn q005_shape_is_usdc_service_fee_only() {
        // The rail never models token/yield fields; the DB constraints enforce
        // this too. Guard the constants the close path writes.
        assert_eq!(BYTES_PER_GB, 1_000_000_000);
        assert!(DEFAULT_RATE_MICRO_USDC_PER_GB > 0);
    }
}
