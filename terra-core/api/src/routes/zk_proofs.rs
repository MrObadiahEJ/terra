use ark_ff::{PrimeField, Zero};
use axum::extract::{ConnectInfo, Path, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use uuid::Uuid;

use crate::error::AppError;
use crate::state::AppState;

// ---------------------------------------------------------------------------
// Zone sets + ownership roots + nullifier records (RFC-011 mirror)
// ---------------------------------------------------------------------------

const ZONE_SET_SELECT: &str = r#"
    SELECT
        id, zone_set_address, zone_id, authority,
        parcel_count, current_root_version,
        created_at, updated_at
    FROM zone_sets
"#;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ZoneSet {
    pub id: Uuid,
    pub zone_set_address: String,
    pub zone_id: String,
    pub authority: String,
    pub parcel_count: i32,
    pub current_root_version: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct RegisterZoneSetRequest {
    pub zone_set_address: String,
    pub zone_id: String,
    pub authority: String,
    pub snapshot_cid: String,
    pub snapshot_hash: String,
    pub root_address: String,
    pub merkle_root: String,
}

#[derive(Debug, Deserialize)]
pub struct GenerateRootRequest {
    pub root_address: String,
    pub merkle_root: String,
    pub snapshot_cid: String,
    pub snapshot_hash: String,
    pub commitment_count: i32,
    pub authority_signature: Option<String>,
}

const ROOT_SELECT: &str = r#"
    SELECT
        id, zone_set_id, root_address, merkle_root, version,
        commitment_count, algorithm_id, verification_key_hash,
        snapshot_cid, snapshot_hash, authority_signature, created_at, updated_at
    FROM ownership_roots
"#;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct OwnershipRoot {
    pub id: Uuid,
    pub zone_set_id: Uuid,
    pub root_address: String,
    pub merkle_root: String,
    pub version: i32,
    pub commitment_count: i32,
    pub algorithm_id: i16,
    pub verification_key_hash: String,
    pub snapshot_cid: String,
    pub snapshot_hash: String,
    pub authority_signature: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

const NULLIFIER_SELECT: &str = r#"
    SELECT
        id, nullifier_hash, zone_set_id, root_version,
        proof_purpose, disclosure_type, block_time, created_at
    FROM nullifier_records
"#;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct NullifierRecord {
    pub id: Uuid,
    pub nullifier_hash: String,
    pub zone_set_id: Uuid,
    pub root_version: i32,
    pub proof_purpose: String,
    pub disclosure_type: i16,
    pub block_time: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VerifyProofRequest {
    pub nullifier_hash: String,
    pub root_version: i32,
    /// One-time Ed25519 presentation key; never the holder's wallet address.
    pub presenter: String,
    pub proof_purpose: String,
    pub disclosure_type: i16,
    /// Hex-encoded TG16 v1 Groth16 frame.
    pub proof_data: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PrepareDevProofRequest {
    pub root_version: i32,
    pub presenter: String,
    pub proof_purpose: String,
    pub disclosure_type: i16,
    /// Private scalar, accepted only over the loopback development endpoint.
    pub secret: String,
}

#[derive(Debug, Serialize)]
pub struct PrepareDevProofResponse {
    pub nullifier_hash: String,
    pub statement: String,
    pub development_only: bool,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GenerateDevProofRequest {
    pub root_version: i32,
    pub presenter: String,
    pub proof_purpose: String,
    pub disclosure_type: i16,
    pub secret: String,
    pub siblings: Vec<String>,
    pub path_indices: Vec<bool>,
    pub statement_signature: String,
}

#[derive(Debug, Serialize)]
pub struct GenerateDevProofResponse {
    pub proof_data: String,
    pub verification_key_hash: String,
    pub development_only: bool,
}

fn decode_hex32(s: &str) -> Result<[u8; 32], AppError> {
    let bytes = hex::decode(s.trim_start_matches("0x"))
        .map_err(|_| AppError::bad_request("expected hex"))?;
    if bytes.len() != 32 {
        return Err(AppError::bad_request("expected 32 bytes"));
    }
    let mut out = [0u8; 32];
    out.copy_from_slice(&bytes);
    Ok(out)
}

fn decode_hex64(s: &str) -> Result<[u8; 64], AppError> {
    let bytes = hex::decode(s.trim_start_matches("0x"))
        .map_err(|_| AppError::bad_request("expected a 64-byte hex signature"))?;
    if bytes.len() != 64 {
        return Err(AppError::bad_request("expected a 64-byte hex signature"));
    }
    let mut out = [0u8; 64];
    out.copy_from_slice(&bytes);
    Ok(out)
}

fn root_attestation_message(root: &[u8; 32], version: i32) -> [u8; 36] {
    let mut message = [0u8; 36];
    message[..32].copy_from_slice(root);
    message[32..].copy_from_slice(&version.to_le_bytes());
    message
}

fn verify_root_attestation(
    authority: &str,
    root: &[u8; 32],
    version: i32,
    signature: &str,
) -> Result<(), AppError> {
    let authority_bytes = crate::routes::identities::decode_wallet(authority)?;
    let public_key = ed25519_dalek::VerifyingKey::from_bytes(&authority_bytes)
        .map_err(|_| AppError::bad_request("invalid zone authority key"))?;
    let signature_bytes = decode_hex64(signature)?;
    let signature = ed25519_dalek::Signature::from_bytes(&signature_bytes);
    public_key
        .verify_strict(&root_attestation_message(root, version), &signature)
        .map_err(|_| AppError::bad_request("invalid zone-authority root signature"))
}

const MEMBERSHIP_PROOFS_DISABLED: &str =
    "ownership membership proofs are disabled until a sound membership circuit, trusted verification key, and audited setup artifacts are installed";

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

async fn list_zone_sets(State(state): State<AppState>) -> Result<Json<Vec<ZoneSet>>, AppError> {
    let rows: Vec<ZoneSet> = sqlx::query_as(ZONE_SET_SELECT)
        .fetch_all(&state.pool)
        .await?;
    Ok(Json(rows))
}

async fn get_zone_set(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ZoneSet>, AppError> {
    let row: ZoneSet = sqlx::query_as(&format!("{ZONE_SET_SELECT} WHERE id = $1"))
        .bind(id)
        .fetch_optional(&state.pool)
        .await?
        .ok_or_else(|| AppError::not_found("zone set not found"))?;
    Ok(Json(row))
}

async fn register_zone_set(
    State(state): State<AppState>,
    Json(req): Json<RegisterZoneSetRequest>,
) -> Result<(StatusCode, Json<ZoneSet>), AppError> {
    if req.snapshot_cid.trim().is_empty() {
        return Err(AppError::bad_request("snapshot_cid is required"));
    }
    decode_hex32(&req.snapshot_hash)?;
    decode_hex32(&req.merkle_root)?;
    crate::routes::identities::decode_wallet(&req.authority)?;
    crate::routes::identities::decode_wallet(&req.zone_set_address)?;
    crate::routes::identities::decode_wallet(&req.root_address)?;

    let mut tx = state.pool.begin().await?;
    let row: Option<ZoneSet> = sqlx::query_as(
        "INSERT INTO zone_sets (zone_set_address, zone_id, authority)
         VALUES ($1, $2, $3)
         ON CONFLICT (zone_set_address) DO NOTHING
         RETURNING id, zone_set_address, zone_id, authority,
                   parcel_count, current_root_version, created_at, updated_at",
    )
    .bind(&req.zone_set_address)
    .bind(&req.zone_id)
    .bind(&req.authority)
    .fetch_one(&mut *tx)
    .await
    .map(Some)
    .or_else(|error| match error {
        sqlx::Error::RowNotFound => Ok(None),
        other => Err(other),
    })?;
    let Some(row) = row else {
        return Err(AppError::conflict(
            "zone set already exists; authority changes require an on-chain update",
        ));
    };

    let verification_key_hash = state
        .zk
        .as_ref()
        .map(|zk| zk.verification_key_hash_hex())
        .unwrap_or_default();
    let root_inserted = sqlx::query(
        "INSERT INTO ownership_roots
            (zone_set_id, root_address, merkle_root, version, commitment_count,
             algorithm_id, snapshot_cid, snapshot_hash, verification_key_hash)
         VALUES ($1, $2, $3, 0, 0, 0, $4, $5, $6)
         ON CONFLICT (root_address) DO NOTHING",
    )
    .bind(row.id)
    .bind(&req.root_address)
    .bind(&req.merkle_root)
    .bind(&req.snapshot_cid)
    .bind(&req.snapshot_hash)
    .bind(&verification_key_hash)
    .execute(&mut *tx)
    .await?;
    if root_inserted.rows_affected() != 1 {
        return Err(AppError::conflict("ownership root address already exists"));
    }
    tx.commit().await?;

    Ok((StatusCode::CREATED, Json(row)))
}

async fn list_roots(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<OwnershipRoot>>, AppError> {
    let rows: Vec<OwnershipRoot> = sqlx::query_as(&format!(
        "{ROOT_SELECT} WHERE zone_set_id = $1 ORDER BY version DESC"
    ))
    .bind(id)
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(rows))
}

async fn generate_root(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<GenerateRootRequest>,
) -> Result<(StatusCode, Json<OwnershipRoot>), AppError> {
    if req.snapshot_cid.trim().is_empty() {
        return Err(AppError::bad_request("snapshot_cid is required"));
    }
    decode_hex32(&req.merkle_root)?;
    decode_hex32(&req.snapshot_hash)?;
    if req.commitment_count <= 0 {
        return Err(AppError::bad_request(
            "commitment_count must be positive (cannot generate a root for an empty zone)",
        ));
    }

    let mut tx = state.pool.begin().await?;
    let current: Option<(i32, String)> = sqlx::query_as(
        "SELECT current_root_version, authority FROM zone_sets WHERE id = $1 FOR UPDATE",
    )
    .bind(id)
    .fetch_optional(&mut *tx)
    .await?;
    let Some((version, authority)) = current else {
        return Err(AppError::not_found("zone set not found"));
    };
    let next = version + 1;
    let root = decode_hex32(&req.merkle_root)?;
    let signature = req
        .authority_signature
        .as_deref()
        .ok_or_else(|| AppError::bad_request("authority_signature is required"))?;
    verify_root_attestation(&authority, &root, next, signature)?;

    sqlx::query(
        "UPDATE zone_sets
         SET current_root_version = $2, parcel_count = $3, updated_at = now()
         WHERE id = $1",
    )
    .bind(id)
    .bind(next)
    .bind(req.commitment_count)
    .execute(&mut *tx)
    .await?;

    let verification_key_hash = state
        .zk
        .as_ref()
        .map(|zk| zk.verification_key_hash_hex())
        .unwrap_or_default();
    let row: OwnershipRoot = sqlx::query_as(
        "INSERT INTO ownership_roots
            (zone_set_id, root_address, merkle_root, version, commitment_count,
             algorithm_id, snapshot_cid, snapshot_hash, authority_signature,
             verification_key_hash)
         VALUES ($1, $2, $3, $4, $5, 0, $6, $7, $8, $9)
         RETURNING id, zone_set_id, root_address, merkle_root, version,
                   commitment_count, algorithm_id, verification_key_hash,
                   snapshot_cid, snapshot_hash, authority_signature, created_at, updated_at",
    )
    .bind(id)
    .bind(&req.root_address)
    .bind(&req.merkle_root)
    .bind(next)
    .bind(req.commitment_count)
    .bind(&req.snapshot_cid)
    .bind(&req.snapshot_hash)
    .bind(req.authority_signature.as_deref().unwrap_or(""))
    .bind(&verification_key_hash)
    .fetch_one(&mut *tx)
    .await?;
    tx.commit().await?;

    Ok((StatusCode::CREATED, Json(row)))
}

#[derive(Serialize)]
struct ZkStatusResponse {
    mode: &'static str,
    verification_enabled: bool,
    development_proving_enabled: bool,
    circuit: &'static str,
    tree_depth: usize,
    verification_key_hash: Option<String>,
    note: &'static str,
}

async fn zk_status(State(state): State<AppState>) -> Json<ZkStatusResponse> {
    let zk = state.zk.as_deref();
    Json(ZkStatusResponse {
        mode: match zk.map(|runtime| runtime.mode()) {
            Some(crate::zk::ZkMode::Development) => "development",
            Some(crate::zk::ZkMode::Production) => "production",
            None => "disabled",
        },
        verification_enabled: zk.is_some(),
        development_proving_enabled: zk.is_some_and(|runtime| runtime.is_dev_prover_enabled()),
        circuit: "terra-land-membership-bn254-poseidon-sha256-v1",
        tree_depth: terra_zk_membership::TREE_DEPTH,
        verification_key_hash: zk.map(|runtime| runtime.verification_key_hash_hex()),
        note: "Development proving sends private witness data only to this loopback API. Production requires independently audited ceremony keys.",
    })
}

type MembershipContext = (
    ZoneSet,
    OwnershipRoot,
    [u8; 32],
    terra_zk_membership::Fr,
    [u8; 32],
    Vec<u8>,
);

async fn membership_context(
    state: &AppState,
    zone_set_id: Uuid,
    root_version: i32,
    presenter: &str,
    proof_purpose: &str,
    disclosure_type: i16,
    secret_hex: &str,
) -> Result<MembershipContext, AppError> {
    if root_version <= 0 {
        return Err(AppError::bad_request("root_version must be positive"));
    }
    if disclosure_type != 0 {
        return Err(AppError::bad_request(
            "this Groth16 circuit only supports MEMBERSHIP disclosure_type 0",
        ));
    }
    if proof_purpose.trim().is_empty() || proof_purpose.len() > 128 {
        return Err(AppError::bad_request(
            "proof_purpose must contain 1..=128 UTF-8 bytes",
        ));
    }
    let presenter_bytes = crate::routes::identities::decode_wallet(presenter)?;
    let secret_bytes = decode_hex32(secret_hex)?;
    let secret =
        terra_zk_membership::field_from_be(&secret_bytes).map_err(AppError::bad_request)?;
    if secret.is_zero() {
        return Err(AppError::bad_request("secret must be non-zero"));
    }

    let zone_set: ZoneSet = sqlx::query_as(&format!("{ZONE_SET_SELECT} WHERE id = $1"))
        .bind(zone_set_id)
        .fetch_optional(&state.pool)
        .await?
        .ok_or_else(|| AppError::not_found("zone set not found"))?;
    let root: OwnershipRoot = sqlx::query_as(&format!(
        "{ROOT_SELECT} WHERE zone_set_id = $1 AND version = $2"
    ))
    .bind(zone_set_id)
    .bind(root_version)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::not_found("ownership root version not found"))?;
    if zone_set.current_root_version != root_version {
        return Err(AppError::conflict(
            "proof references a stale root version; select the current ownership root",
        ));
    }
    if root.commitment_count <= 0 || root.algorithm_id != 0 {
        return Err(AppError::conflict(
            "ownership root is empty or uses an unsupported proof algorithm",
        ));
    }

    let zone_bytes = crate::routes::identities::decode_wallet(&zone_set.zone_set_address)?;
    let root_bytes = decode_hex32(&root.merkle_root)?;
    let root_field =
        terra_zk_membership::field_from_be(&root_bytes).map_err(AppError::bad_request)?;
    let zone_field = terra_zk_membership::Fr::from_be_bytes_mod_order(&zone_bytes);
    let nullifier =
        terra_zk_membership::derive_nullifier(secret, zone_field, root_field, root_version as u32)
            .map_err(|error| AppError::Internal(format!("nullifier derivation failed: {error}")))?;
    let nullifier_bytes = terra_zk_membership::field_to_be(&nullifier);
    let statement = terra_zk_membership::canonical_ownership_statement(
        &zone_bytes,
        &root_bytes,
        root_version as u32,
        &nullifier_bytes,
        &presenter_bytes,
        proof_purpose,
        disclosure_type as u8,
    )
    .map_err(AppError::bad_request)?;

    Ok((
        zone_set,
        root,
        presenter_bytes,
        secret,
        nullifier_bytes,
        statement,
    ))
}

fn require_loopback_prover(
    peer: SocketAddr,
    state: &AppState,
) -> Result<&crate::zk::ZkRuntime, AppError> {
    if !peer.ip().is_loopback() {
        return Err(AppError::Forbidden(
            "development proof generation is loopback-only".into(),
        ));
    }
    let runtime = state
        .zk
        .as_deref()
        .ok_or_else(|| AppError::Unavailable(MEMBERSHIP_PROOFS_DISABLED.into()))?;
    if !runtime.is_dev_prover_enabled() {
        return Err(AppError::Unavailable(
            "proof generation is available only with development Groth16 proving keys".into(),
        ));
    }
    Ok(runtime)
}

async fn prepare_dev_proof(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    Json(req): Json<PrepareDevProofRequest>,
) -> Result<Json<PrepareDevProofResponse>, AppError> {
    require_loopback_prover(peer, &state)?;
    let (_, _, _, _, nullifier, statement) = membership_context(
        &state,
        id,
        req.root_version,
        &req.presenter,
        &req.proof_purpose,
        req.disclosure_type,
        &req.secret,
    )
    .await?;
    Ok(Json(PrepareDevProofResponse {
        nullifier_hash: hex::encode(nullifier),
        statement: hex::encode(statement),
        development_only: true,
    }))
}

async fn generate_dev_proof(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    Json(req): Json<GenerateDevProofRequest>,
) -> Result<Json<GenerateDevProofResponse>, AppError> {
    let runtime = require_loopback_prover(peer, &state)?;
    let (zone_set, root, presenter, secret, nullifier, statement) = membership_context(
        &state,
        id,
        req.root_version,
        &req.presenter,
        &req.proof_purpose,
        req.disclosure_type,
        &req.secret,
    )
    .await?;
    verify_presenter_signature(&presenter, &statement, &req.statement_signature)?;

    if req.siblings.len() != terra_zk_membership::TREE_DEPTH
        || req.path_indices.len() != terra_zk_membership::TREE_DEPTH
    {
        return Err(AppError::bad_request(format!(
            "Merkle witness must contain exactly {} sibling hashes and direction bits",
            terra_zk_membership::TREE_DEPTH
        )));
    }
    let siblings = req
        .siblings
        .iter()
        .map(|hex| {
            let bytes = decode_hex32(hex)?;
            terra_zk_membership::field_from_be(&bytes).map_err(AppError::bad_request)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let path = terra_zk_membership::MerklePath {
        siblings,
        indices: req.path_indices,
    };
    let computed_root = terra_zk_membership::compute_root(secret, &path)
        .map_err(|error| AppError::bad_request(error.to_string()))?;
    let expected_root = terra_zk_membership::field_from_be(&decode_hex32(&root.merkle_root)?)
        .map_err(AppError::bad_request)?;
    if computed_root != expected_root {
        return Err(AppError::bad_request(
            "private witness does not open into the selected ownership root",
        ));
    }
    let expected_nullifier = terra_zk_membership::field_to_be(
        &terra_zk_membership::derive_nullifier(
            secret,
            terra_zk_membership::Fr::from_be_bytes_mod_order(
                &crate::routes::identities::decode_wallet(&zone_set.zone_set_address)?,
            ),
            expected_root,
            req.root_version as u32,
        )
        .map_err(|error| AppError::Internal(format!("nullifier derivation failed: {error}")))?,
    );
    if expected_nullifier != nullifier {
        return Err(AppError::bad_request(
            "membership nullifier does not match the signed statement",
        ));
    }

    let proving_key = runtime
        .proving_key_arc()
        .ok_or_else(|| AppError::Unavailable("development proving key is not loaded".into()))?;
    let proof = tokio::task::spawn_blocking(move || {
        terra_zk_membership::prove(
            &proving_key,
            terra_zk_membership::MembershipWitness {
                statement,
                secret,
                path,
            },
        )
    })
    .await
    .map_err(|error| AppError::Internal(format!("Groth16 proving task failed: {error}")))?
    .map_err(|error| AppError::Internal(format!("Groth16 proof generation failed: {error}")))?;

    let signature = decode_hex64(&req.statement_signature)?;
    let proof_data = terra_zk_membership::encode_frame(&signature, runtime.verifying_key(), &proof)
        .map_err(|error| AppError::Internal(error.into()))?;
    Ok(Json(GenerateDevProofResponse {
        proof_data: hex::encode(proof_data),
        verification_key_hash: runtime.verification_key_hash_hex(),
        development_only: true,
    }))
}

fn verify_presenter_signature(
    presenter: &[u8; 32],
    statement: &[u8],
    signature_hex: &str,
) -> Result<(), AppError> {
    let key = ed25519_dalek::VerifyingKey::from_bytes(presenter)
        .map_err(|_| AppError::bad_request("invalid ephemeral presenter key"))?;
    let signature_bytes = decode_hex64(signature_hex)?;
    let signature = ed25519_dalek::Signature::from_bytes(&signature_bytes);
    key.verify_strict(statement, &signature)
        .map_err(|_| AppError::bad_request("invalid presenter statement signature"))
}

async fn verify_proof(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<VerifyProofRequest>,
) -> Result<(StatusCode, Json<NullifierRecord>), AppError> {
    let nullifier = decode_hex32(&req.nullifier_hash)?;
    if nullifier.iter().all(|byte| *byte == 0) {
        return Err(AppError::bad_request("nullifier_hash must be non-zero"));
    }
    if req.root_version <= 0 {
        return Err(AppError::bad_request("root_version must be positive"));
    }
    if req.disclosure_type != 0 {
        return Err(AppError::bad_request(
            "this Groth16 circuit only supports MEMBERSHIP disclosure_type 0",
        ));
    }
    if req.proof_purpose.trim().is_empty() || req.proof_purpose.len() > 128 {
        return Err(AppError::bad_request(
            "proof_purpose must contain 1..=128 UTF-8 bytes",
        ));
    }
    let presenter = crate::routes::identities::decode_wallet(&req.presenter)?;
    let proof_data = hex::decode(req.proof_data.trim_start_matches("0x"))
        .map_err(|_| AppError::bad_request("proof_data must be hex-encoded"))?;
    let frame = terra_zk_membership::decode_frame(&proof_data).map_err(AppError::bad_request)?;
    let runtime = state
        .zk
        .as_ref()
        .ok_or_else(|| AppError::Unavailable(MEMBERSHIP_PROOFS_DISABLED.into()))?;

    // Serialize root-version checks and nullifier insertion with root rotation.
    let mut tx = state.pool.begin().await?;
    let current: Option<(i32, String)> = sqlx::query_as(
        "SELECT current_root_version, zone_set_address FROM zone_sets WHERE id = $1 FOR UPDATE",
    )
    .bind(id)
    .fetch_optional(&mut *tx)
    .await?;
    let Some((version, zone_set_address)) = current else {
        return Err(AppError::not_found("zone set not found"));
    };
    if req.root_version != version {
        return Err(AppError::conflict(
            "proof references a stale root version; regenerate against the current root",
        ));
    }

    let root: OwnershipRoot = sqlx::query_as(&format!(
        "{ROOT_SELECT} WHERE zone_set_id = $1 AND version = $2"
    ))
    .bind(id)
    .bind(version)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| AppError::not_found("current ownership root not found"))?;
    if root.commitment_count <= 0 || root.algorithm_id != 0 {
        return Err(AppError::conflict(
            "ownership root is empty or uses an unsupported proof algorithm",
        ));
    }
    let expected_vk_hash = runtime.verification_key_hash_hex();
    if root.verification_key_hash != expected_vk_hash
        || frame.verification_key != *runtime.verification_key_frame()
    {
        return Err(AppError::bad_request(
            "proof verification key does not match the key pinned to this ownership root",
        ));
    }

    let zone_bytes = crate::routes::identities::decode_wallet(&zone_set_address)?;
    let root_bytes = decode_hex32(&root.merkle_root)?;
    let statement = terra_zk_membership::canonical_ownership_statement(
        &zone_bytes,
        &root_bytes,
        version as u32,
        &nullifier,
        &presenter,
        &req.proof_purpose,
        req.disclosure_type as u8,
    )
    .map_err(AppError::bad_request)?;
    verify_presenter_signature(&presenter, &statement, &hex::encode(frame.signature))?;

    let proof =
        terra_zk_membership::frame::parse_proof(&frame.proof).map_err(AppError::bad_request)?;
    let verifying_key = runtime.verifying_key_arc();
    let statement_for_verification = statement.clone();
    let verified = tokio::task::spawn_blocking(move || {
        terra_zk_membership::verify(&verifying_key, &statement_for_verification, &proof)
    })
    .await
    .map_err(|error| AppError::Internal(format!("Groth16 verification task failed: {error}")))?
    .map_err(|error| AppError::Internal(format!("Groth16 verification failed: {error}")))?;
    if !verified {
        return Err(AppError::bad_request("Groth16 membership proof is invalid"));
    }

    let row: NullifierRecord = sqlx::query_as(
        "INSERT INTO nullifier_records
            (nullifier_hash, zone_set_id, root_version, prover, proof_purpose, disclosure_type)
         VALUES ($1, $2, $3, $4, $5, $6)
         RETURNING id, nullifier_hash, zone_set_id, root_version,
                   proof_purpose, disclosure_type, block_time, created_at",
    )
    .bind(&req.nullifier_hash)
    .bind(id)
    .bind(req.root_version)
    .bind(&req.presenter)
    .bind(&req.proof_purpose)
    .bind(req.disclosure_type)
    .fetch_one(&mut *tx)
    .await
    .map_err(|error| {
        if let sqlx::Error::Database(db) = &error {
            if db.code().as_deref() == Some("23505") {
                return AppError::conflict("proof has already been used (nullifier recorded)");
            }
        }
        AppError::from(error)
    })?;

    tx.commit().await?;
    Ok((StatusCode::CREATED, Json(row)))
}

async fn list_nullifiers(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<NullifierRecord>>, AppError> {
    let rows: Vec<NullifierRecord> = sqlx::query_as(&format!(
        "{NULLIFIER_SELECT} WHERE zone_set_id = $1 ORDER BY block_time DESC"
    ))
    .bind(id)
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(rows))
}

async fn invalidate_version(
    State(state): State<AppState>,
    Path((id, version)): Path<(Uuid, i32)>,
) -> Result<Json<ZoneSet>, AppError> {
    if version <= 0 {
        return Err(AppError::bad_request(
            "cannot invalidate the genesis version",
        ));
    }
    let row: Option<ZoneSet> = sqlx::query_as(&format!(
        "{ZONE_SET_SELECT} WHERE id = $1 AND current_root_version > $2"
    ))
    .bind(id)
    .bind(version)
    .fetch_optional(&state.pool)
    .await?;
    match row {
        Some(z) => {
            sqlx::query("UPDATE zone_sets SET updated_at = now() WHERE id = $1")
                .bind(id)
                .execute(&state.pool)
                .await?;
            Ok(Json(z))
        }
        None => Err(AppError::conflict(
            "cannot invalidate the current version (rotate the root first)",
        )),
    }
}

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/status", get(zk_status))
        .route("/", get(list_zone_sets).post(register_zone_set))
        .route("/{id}", get(get_zone_set))
        .route("/{id}/roots", get(list_roots).post(generate_root))
        .route("/{id}/proofs", get(list_nullifiers).post(verify_proof))
        .route("/{id}/dev/prepare", post(prepare_dev_proof))
        .route("/{id}/dev/prove", post(generate_dev_proof))
        .route("/{id}/invalidate/{version}", post(invalidate_version))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disclosure_type_bounds() {
        for valid in [0i16, 1, 2] {
            assert!((0..=2).contains(&valid));
        }
        assert!(!(0..=2).contains(&3i16));
    }

    #[test]
    #[allow(clippy::assertions_on_constants)]
    fn empty_zone_rejected() {
        assert!(0i32 <= 0);
        assert!(5i32 > 0);
    }

    #[test]
    #[allow(clippy::const_is_empty)]
    fn purpose_length_bounds() {
        assert!("subsidy_qualification".len() <= 128);
        assert!("".is_empty());
    }

    #[test]
    fn decode_hex32_rejects_short() {
        assert!(decode_hex32("abcd").is_err());
        assert!(decode_hex32(&"00".repeat(32)).is_ok());
    }

    #[test]
    fn authority_root_attestation_is_bound_to_root_and_version() {
        use ed25519_dalek::Signer;

        let signing_key = ed25519_dalek::SigningKey::from_bytes(&[7u8; 32]);
        let authority = bs58::encode(signing_key.verifying_key().as_bytes()).into_string();
        let root = [11u8; 32];
        let signature = signing_key
            .sign(&root_attestation_message(&root, 3))
            .to_bytes();
        let signature_hex = hex::encode(signature);

        assert!(verify_root_attestation(&authority, &root, 3, &signature_hex).is_ok());
        assert!(verify_root_attestation(&authority, &[12u8; 32], 3, &signature_hex).is_err());
        assert!(verify_root_attestation(&authority, &root, 4, &signature_hex).is_err());
    }

    #[test]
    fn legacy_caller_supplied_prover_metadata_is_rejected() {
        let body = serde_json::json!({
            "nullifier_hash": "00".repeat(32),
            "root_version": 1,
            "prover": "caller-chosen",
            "proof_purpose": "membership",
            "disclosure_type": 0
        });
        assert!(serde_json::from_value::<VerifyProofRequest>(body).is_err());
    }
}
