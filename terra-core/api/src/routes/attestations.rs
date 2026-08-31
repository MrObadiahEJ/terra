use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use ed25519_dalek::{Signature, VerifyingKey};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::AppError;
use crate::state::AppState;

// ---------------------------------------------------------------------------
// Canonical signed message for every validator signature:
//   message = bytes( content_hash[32] || onchain_id[32] )
// A validator signs this 64-byte payload with their Ed25519 key. Anyone can
// re-derive it and verify the signature against the validator's public key —
// this is what makes a signer unable to deny having validated a transaction,
// including when several validators must approve a land purchase.
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct RegisterAttestation {
    /// hex(32) on-chain parcel PDA seed.
    pub onchain_id: String,
    /// hex(32) attestation specifier (PDA seed component).
    pub specifier: String,
    /// hex(32) sha256 over the off-chain payload (documents, deed, survey).
    pub content_hash: String,
    pub required: u16,
    pub count: u16,
    /// base58 wallet list recorded in the on-chain Attestation account.
    pub validators: Vec<String>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct AttestationRow {
    pub id: Uuid,
    pub parcel_id: Uuid,
    pub onchain_id: String,
    pub specifier: String,
    pub content_hash: String,
    pub required: i16,
    pub validators: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct SubmitValidation {
    /// base58 validator wallet doing the signing.
    pub validator: String,
    /// hex(64) Ed25519 signature over content_hash || onchain_id.
    pub signature: String,
    /// hex(32) content hash the validator is attesting to.
    pub content_hash: String,
}

#[derive(Debug, Serialize)]
pub struct ValidationView {
    pub validator: String,
    pub signature: String,
    pub valid: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
pub struct AttestationDetail {
    #[serde(flatten)]
    pub attestation: AttestationRow,
    pub has_quorum: bool,
    pub signatories: usize,
    pub required: i16,
    pub validations: Vec<ValidationView>,
}

#[derive(Debug, Deserialize)]
pub struct RegisterDocument {
    pub title: String,
    pub category: String,
    pub content_hash: String,
    pub storage_ref: String,
    pub owner: String,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct DocumentRow {
    pub id: Uuid,
    pub parcel_id: Uuid,
    pub title: String,
    pub category: String,
    pub content_hash: String,
    pub storage_ref: String,
    pub owner: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Register an attestation mirror row from the on-chain Attestation account.
pub async fn register_attestation(
    State(state): State<AppState>,
    Path(parcel_id): Path<Uuid>,
    Json(req): Json<RegisterAttestation>,
) -> Result<Json<AttestationRow>, AppError> {
    let onchain_id = decode_hex32(&req.onchain_id)?;
    let specifier = decode_hex32(&req.specifier)?;
    let content_hash = decode_hex32(&req.content_hash)?;
    let _ = onchain_id; // stored as hex

    let row = sqlx::query_as::<_, AttestationRow>(
        "INSERT INTO attestations
            (parcel_id, onchain_id, specifier, content_hash, required, count, validators)
         VALUES ($1, $2, $3, $4, $5, $6, $7)
         ON CONFLICT (parcel_id, specifier) DO UPDATE SET
            content_hash = EXCLUDED.content_hash,
            required     = EXCLUDED.required,
            count        = EXCLUDED.count,
            validators   = EXCLUDED.validators
         RETURNING id, parcel_id, onchain_id, specifier, content_hash, required,
                   validators, created_at",
    )
    .bind(parcel_id)
    .bind(hex::encode(onchain_id))
    .bind(hex::encode(specifier))
    .bind(hex::encode(content_hash))
    .bind(req.required as i16)
    .bind(req.count as i16)
    .bind(&req.validators)
    .fetch_one(&state.pool)
    .await?;
    Ok(Json(row))
}

/// Fetch an attestation and its validations, recomputing signature validity.
pub async fn get_attestation(
    State(state): State<AppState>,
    Path((parcel_id, specifier)): Path<(Uuid, String)>,
) -> Result<Json<AttestationDetail>, AppError> {
    let att = sqlx::query_as::<_, AttestationRow>(
        "SELECT id, parcel_id, onchain_id, specifier, content_hash, required,
                validators, created_at
         FROM attestations
         WHERE parcel_id = $1 AND specifier = $2",
    )
    .bind(parcel_id)
    .bind(&specifier)
    .fetch_one(&state.pool)
    .await?;

    let content_hash = decode_hex32(&att.content_hash)?;
    let onchain_id = decode_hex32(&att.onchain_id)?;

    let rows: Vec<(String, String, chrono::DateTime<chrono::Utc>)> = sqlx::query_as(
        "SELECT validator, signature, created_at FROM validations
         WHERE attestation_id = $1 ORDER BY created_at",
    )
    .bind(att.id)
    .fetch_all(&state.pool)
    .await?;

    let mut validations = Vec::with_capacity(rows.len());
    for (validator, signature, created_at) in rows {
        let valid = verify_ed25519(&validator, &content_hash, &onchain_id, &signature)
            .unwrap_or(false);
        validations.push(ValidationView {
            validator,
            signature,
            valid,
            created_at,
        });
    }

    let signatories = validations.iter().filter(|v| v.valid).count();
    let has_quorum = (signatories as i16) >= att.required;

    Ok(Json(AttestationDetail {
        has_quorum,
        signatories,
        required: att.required,
        validations,
        attestation: att,
    }))
}

/// Record and cryptographically verify a validator's signature.
pub async fn submit_validation(
    State(state): State<AppState>,
    Path((parcel_id, specifier)): Path<(Uuid, String)>,
    Json(req): Json<SubmitValidation>,
) -> Result<(StatusCode, Json<ValidationView>), AppError> {
    let att = sqlx::query_as::<_, AttestationRow>(
        "SELECT id, parcel_id, onchain_id, specifier, content_hash, required,
                validators, created_at
         FROM attestations
         WHERE parcel_id = $1 AND specifier = $2",
    )
    .bind(parcel_id)
    .bind(&specifier)
    .fetch_one(&state.pool)
    .await?;

    let content_hash = decode_hex32(&att.content_hash)?;
    let onchain_id = decode_hex32(&att.onchain_id)?;
    let req_hash = decode_hex32(&req.content_hash)?;
    if req_hash != content_hash {
        return Err(AppError::bad_request("content_hash does not match the attestation"));
    }
    // The signer's wallet must be one of the anchored validators.
    if !att
        .validators
        .iter()
        .any(|v| v.eq_ignore_ascii_case(&req.validator))
    {
        return Err(AppError::bad_request("validator is not in the attestation set"));
    }

    let valid = verify_ed25519(&req.validator, &content_hash, &onchain_id, &req.signature)?;

    let created_at: chrono::DateTime<chrono::Utc> = sqlx::query_scalar(
        "INSERT INTO validations (attestation_id, validator, signature, valid)
         VALUES ($1, $2, $3, $4)
         ON CONFLICT (attestation_id, validator) DO UPDATE SET
            signature = EXCLUDED.signature,
            valid     = EXCLUDED.valid,
            created_at = EXCLUDED.created_at
         RETURNING created_at",
    )
    .bind(att.id)
    .bind(&req.validator)
    .bind(&req.signature)
    .bind(valid)
    .fetch_one(&state.pool)
    .await?;

    Ok((StatusCode::OK, Json(ValidationView {
        validator: req.validator,
        signature: req.signature,
        valid,
        created_at,
    })))
}

/// Bind an off-chain document to a parcel under an owner wallet.
pub async fn register_document(
    State(state): State<AppState>,
    Path(parcel_id): Path<Uuid>,
    Json(req): Json<RegisterDocument>,
) -> Result<(StatusCode, Json<DocumentRow>), AppError> {
    decode_hex32(&req.content_hash)?;
    let doc = sqlx::query_as::<_, DocumentRow>(
        "INSERT INTO documents
            (parcel_id, title, category, content_hash, storage_ref, owner)
         VALUES ($1, $2, $3, $4, $5, $6)
         RETURNING id, parcel_id, title, category, content_hash, storage_ref, owner, created_at",
    )
    .bind(parcel_id)
    .bind(&req.title)
    .bind(&req.category)
    .bind(&req.content_hash)
    .bind(&req.storage_ref)
    .bind(&req.owner)
    .fetch_one(&state.pool)
    .await?;
    Ok((StatusCode::CREATED, Json(doc)))
}

/// List documents bound to a parcel.
pub async fn list_documents(
    State(state): State<AppState>,
    Path(parcel_id): Path<Uuid>,
) -> Result<Json<Vec<DocumentRow>>, AppError> {
    let docs = sqlx::query_as::<_, DocumentRow>(
        "SELECT id, parcel_id, title, category, content_hash, storage_ref, owner, created_at
         FROM documents WHERE parcel_id = $1 ORDER BY created_at",
    )
    .bind(parcel_id)
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(docs))
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

pub fn decode_hex32(s: &str) -> Result<[u8; 32], AppError> {
    let bytes = hex::decode(s).map_err(|_| AppError::bad_request("expected hex"))?;
    if bytes.len() != 32 {
        return Err(AppError::bad_request("expected 32 bytes"));
    }
    let mut out = [0u8; 32];
    out.copy_from_slice(&bytes);
    Ok(out)
}

/// Verify an Ed25519 signature over `content_hash || onchain_id` made by the
/// Ed25519 public key derived from a Solana wallet address.
fn verify_ed25519(
    validator_base58: &str,
    content_hash: &[u8; 32],
    onchain_id: &[u8; 32],
    signature_hex: &str,
) -> Result<bool, AppError> {
    let vk_bytes = bs58::decode(validator_base58)
        .into_vec()
        .map_err(|e| AppError::bad_request(format!("invalid validator address: {e}")))?;
    if vk_bytes.len() != 32 {
        return Err(AppError::bad_request("validator address must decode to 32 bytes"));
    }
    let mut vk_arr = [0u8; 32];
    vk_arr.copy_from_slice(&vk_bytes);
    let vk = VerifyingKey::from_bytes(&vk_arr)
        .map_err(|e| AppError::bad_request(format!("invalid verifying key: {e}")))?;

    let sig_bytes = hex::decode(signature_hex)
        .map_err(|e| AppError::bad_request(format!("signature must be hex: {e}")))?;
    if sig_bytes.len() != 64 {
        return Err(AppError::bad_request("signature must be 64 bytes"));
    }
    let sig = Signature::from_slice(&sig_bytes)
        .map_err(|e| AppError::bad_request(format!("invalid signature: {e}")))?;

    let mut message = Vec::with_capacity(64);
    message.extend_from_slice(content_hash);
    message.extend_from_slice(onchain_id);

    Ok(vk.verify_strict(&message, &sig).is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signer as _, SigningKey};

    fn sample() -> (SigningKey, [u8; 32], [u8; 32]) {
        // Deterministic secret so tests are reproducible.
        let mut seed = [0u8; 32];
        seed[0] = 7;
        let sk = SigningKey::from_bytes(&seed);
        let content_hash = [1u8; 32];
        let onchain_id = [2u8; 32];
        (sk, content_hash, onchain_id)
    }

    fn sign(sk: &SigningKey, content_hash: &[u8; 32], onchain_id: &[u8; 32]) -> String {
        let mut message = Vec::with_capacity(64);
        message.extend_from_slice(content_hash);
        message.extend_from_slice(onchain_id);
        let sig = sk.sign(&message);
        hex::encode(sig.to_bytes())
    }

    #[test]
    fn valid_signature_verifies_for_wallet_address() {
        let (sk, ch, oid) = sample();
        let vk_bytes = sk.verifying_key().to_bytes();

        // The Solana wallet address IS this 32-byte ed25519 public key base58-encoded.
        let wallet = bs58::encode(vk_bytes).into_string();
        let sig = sign(&sk, &ch, &oid);

        assert!(verify_ed25519(&wallet, &ch, &oid, &sig).expect("should not error"));
    }

    #[test]
    fn wrong_content_hash_is_rejected() {
        let (sk, ch, oid) = sample();
        let vk_bytes = sk.verifying_key().to_bytes();
        let wallet = bs58::encode(vk_bytes).into_string();
        let sig = sign(&sk, &ch, &oid);

        // Signer approved a *different* payload than the one being checked.
        let other_hash = [9u8; 32];
        assert!(!verify_ed25519(&wallet, &other_hash, &oid, &sig).expect("should not error"));
    }

    #[test]
    fn tampered_signature_is_rejected() {
        let (sk, ch, oid) = sample();
        let vk_bytes = sk.verifying_key().to_bytes();
        let wallet = bs58::encode(vk_bytes).into_string();
        let mut sig_bytes = hex::decode(sign(&sk, &ch, &oid)).expect("valid hex");
        sig_bytes[0] ^= 0xff; // flip a byte in the actual 64-byte signature
        assert!(!verify_ed25519(&wallet, &ch, &oid, &hex::encode(sig_bytes)).expect("should not error"));
    }

    #[test]
    fn another_wallet_cannot_impersonate() {
        let (sk, ch, oid) = sample();
        let other_sk = SigningKey::from_bytes(&[8u8; 32]);
        let sig = sign(&other_sk, &ch, &oid);

        // Signature by a different key verified against the intended wallet.
        let vk_bytes = sk.verifying_key().to_bytes();
        let wallet = bs58::encode(vk_bytes).into_string();
        assert!(!verify_ed25519(&wallet, &ch, &oid, &sig).expect("should not error"));
    }
}
