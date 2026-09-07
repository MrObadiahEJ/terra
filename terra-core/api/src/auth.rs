use crate::error::AppError;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::StatusCode;
use ed25519_dalek::{Signature, Verifier, VerifyingKey};

/// Authority public key loaded from `API_AUTHORITY_PUBKEY` env var.
/// When `None`, privileged endpoints reject all requests (fail-closed).
#[derive(Clone)]
pub struct ApiAuthority(pub Option<VerifyingKey>);

/// Parsed and verified Ed25519-signed request.
///
/// Expected `Authorization` header format:
/// ```text
/// Signature <base64_signature>
/// ```
///
/// Required custom headers:
/// - `x-timestamp`: epoch seconds (replay window: ±60 s)
/// - `x-body-hash`: hex-encoded SHA-256 of the request body
///
/// The signed payload is:
/// ```text
/// {METHOD}\n{PATH}\n{BODY_SHA256_HEX}\n{TIMESTAMP_EPOCH_SECS}
/// ```
pub struct SignedRequest {
    pub signature: Signature,
    pub timestamp: i64,
    pub body_hash_hex: String,
}

impl SignedRequest {
    /// Verify the signature against the given authority key.
    pub fn verify(&self, key: &VerifyingKey, method: &str, path: &str) -> Result<(), AppError> {
        let payload = format!(
            "{method}\n{path}\n{}\n{}",
            self.body_hash_hex, self.timestamp
        );
        key.verify(payload.as_bytes(), &self.signature)
            .map_err(|_| AppError::unauthorized("invalid signature"))
    }

    /// Verify the signature against a base58-encoded wallet address (Ed25519
    /// public key). This is the pattern used when the caller must prove
    /// ownership of a specific wallet, e.g. identity updates.
    pub fn verify_wallet(&self, wallet_b58: &str, method: &str, path: &str) -> Result<(), AppError> {
        let pubkey_bytes = bs58::decode(wallet_b58)
            .into_vec()
            .map_err(|_| AppError::bad_request("invalid base58 wallet address"))?;
        if pubkey_bytes.len() != 32 {
            return Err(AppError::bad_request("wallet address must be 32 bytes"));
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&pubkey_bytes);
        let key = VerifyingKey::from_bytes(&arr)
            .map_err(|_| AppError::bad_request("invalid Ed25519 public key"))?;
        self.verify(&key, method, path)
    }
}

impl<S> FromRequestParts<S> for SignedRequest
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Extract Authorization header.
        let auth_header = parts
            .headers
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .ok_or(StatusCode::UNAUTHORIZED)?;

        // Parse "Signature <base64>" format.
        let sig_b64 = auth_header
            .strip_prefix("Signature ")
            .ok_or(StatusCode::UNAUTHORIZED)?;

        let sig_bytes = base64_decode(sig_b64).map_err(|_| StatusCode::UNAUTHORIZED)?;
        if sig_bytes.len() != 64 {
            return Err(StatusCode::UNAUTHORIZED);
        }
        let mut sig_arr = [0u8; 64];
        sig_arr.copy_from_slice(&sig_bytes);
        let signature = Signature::from_bytes(&sig_arr);

        // Extract the timestamp.
        let timestamp_str = parts
            .headers
            .get("x-timestamp")
            .and_then(|v| v.to_str().ok())
            .ok_or(StatusCode::UNAUTHORIZED)?;
        let timestamp: i64 = timestamp_str
            .parse()
            .map_err(|_| StatusCode::UNAUTHORIZED)?;

        // Replay protection: reject requests older than 60 seconds.
        let now = chrono::Utc::now().timestamp();
        if (now - timestamp).abs() > 60 {
            return Err(StatusCode::UNAUTHORIZED);
        }

        // Body hash header.
        let body_hash_hex = parts
            .headers
            .get("x-body-hash")
            .and_then(|v| v.to_str().ok())
            .ok_or(StatusCode::UNAUTHORIZED)?
            .to_string();

        Ok(SignedRequest {
            signature,
            timestamp,
            body_hash_hex,
        })
    }
}

/// Minimal base64 decode (only STANDARD encoding).
fn base64_decode(input: &str) -> Result<Vec<u8>, ()> {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD
        .decode(input)
        .map_err(|_| ())
}
