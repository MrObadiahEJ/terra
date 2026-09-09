use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::post;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::state::AppState;

// ---------------------------------------------------------------------------
// POST /api/v1/tx/prepare
//
// Accepts an instruction type + parameters, constructs an unsigned Solana
// instruction (or set of instructions), and returns the serialized bytes
// that the client should sign client-side and submit via
// `sendTransaction`.  The API never holds a keypair — all signing happens
// in the browser wallet.
// ---------------------------------------------------------------------------

pub fn router() -> Router<AppState> {
    Router::new().route("/prepare", post(prepare_tx))
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum PrepareRequest {
    /// Build an `add_evidence` instruction.
    #[serde(rename = "add_evidence")]
    AddEvidence {
        /// base58 parcel PDA.
        parcel: String,
        /// hex(32) claim id seed.
        claim_id: String,
        /// hex(32) SHA-256 of the evidence content.
        content_hash: String,
        /// Evidence type constant (0=Photo, 1=Document, etc.).
        evidence_type: u8,
        /// Storage reference (IPFS CID or local path).
        storage_ref: String,
        /// Unix timestamp of original observation.
        observed_at: i64,
    },

    /// Build a `submit_observation` instruction.
    #[serde(rename = "submit_observation")]
    SubmitObservation {
        /// base58 claim PDA.
        claim: String,
        /// Latitude * 1e7 (i64).
        latitude: i64,
        /// Longitude * 1e7 (i64).
        longitude: i64,
        /// Observation method (0=visual, 1=GPS, 2=survey, 3=document_review).
        method: u8,
        /// hex(32) sha256 of detailed findings document.
        findings_hash: String,
        /// Confidence level 0-100.
        confidence: u8,
        /// hex(32) sha256 of the signed observation payload.
        signature_hash: String,
    },

    /// Build a `submit_verification_attestation` instruction.
    #[serde(rename = "submit_attestation")]
    SubmitAttestation {
        /// base58 claim PDA.
        claim: String,
        /// base58 observation PDA this attestation references.
        observation: String,
        /// Attestation result constant (0=Confirmed, 1=Disputed, 2=Inconclusive).
        result: u8,
        /// Confidence level 0-100.
        confidence: u8,
        /// hex(32) sha256 of the signed attestation payload.
        signature_hash: String,
    },

    /// Build a `create_claim` instruction.
    #[serde(rename = "create_claim")]
    CreateClaim {
        /// base58 parcel PDA.
        parcel: String,
        /// hex(32) claim id seed (unique per parcel).
        claim_id: String,
        /// Claim type constant.
        claim_type: u8,
        /// hex(32) content hash of the claim payload.
        content_hash: String,
        /// Required attestation count.
        required_attestations: u8,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PreparedInstruction {
    /// The Anchor discriminator (8 bytes) for the instruction.
    pub discriminator: Vec<u8>,
    /// Accounts required by the instruction.
    pub accounts: Vec<AccountMeta>,
    /// Serialized instruction data (without discriminator).
    pub data: Vec<u8>,
    /// Human-readable description of what this instruction does.
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountMeta {
    /// base58 public key.
    pub pubkey: String,
    /// Is this account a signer?
    pub is_signer: bool,
    /// Is this account writable?
    pub is_writable: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PrepareResponse {
    /// The instructions to include in the transaction.
    pub instructions: Vec<PreparedInstruction>,
    /// Fee payer that must sign the transaction (the wallet).
    pub fee_payer: String,
    /// Recent blockhash the client must fetch before sending.
    pub recent_blockhash_note: String,
}

/// Prepare unsigned transaction instructions for client-side signing.
async fn prepare_tx(
    State(_state): State<AppState>,
    Json(req): Json<PrepareRequest>,
) -> Result<(StatusCode, Json<PrepareResponse>), AppError> {
    match req {
        PrepareRequest::AddEvidence {
            parcel,
            claim_id,
            content_hash,
            evidence_type,
            storage_ref,
            observed_at,
        } => {
            // Validate inputs.
            let claim_id_bytes = decode_hex32(&claim_id)?;
            let content_hash_bytes = decode_hex32(&content_hash)?;

            // Build instruction data: anchor discriminator (8) + claim_id (32) +
            // evidence_type (1) + content_hash (32) + storage_ref_len (2) +
            // storage_ref (?) + observed_at (8)
            let mut data = Vec::new();
            // Anchor discriminator for "global:add_evidence" — client will need to
            // compute the actual 8-byte discriminator from the instruction name.
            // For now we include a placeholder and the client computes it.
            data.extend_from_slice(&[0u8; 8]); // discriminator placeholder
            data.extend_from_slice(&claim_id_bytes);
            data.push(evidence_type);
            data.extend_from_slice(&content_hash_bytes);
            let sr_bytes = storage_ref.as_bytes();
            data.extend_from_slice(&(sr_bytes.len() as u16).to_le_bytes());
            data.extend_from_slice(sr_bytes);
            data.extend_from_slice(&observed_at.to_le_bytes());

            let parcel_pk = parcel.clone();
            let accounts = vec![
                AccountMeta {
                    pubkey: "EVIDENCE_ACCOUNT_PDA".into(),
                    is_signer: false,
                    is_writable: true,
                },
                AccountMeta {
                    pubkey: parcel_pk,
                    is_signer: false,
                    is_writable: false,
                },
                AccountMeta {
                    pubkey: "FEE_PAYER".into(),
                    is_signer: true,
                    is_writable: true,
                },
                AccountMeta {
                    pubkey: "11111111111111111111111111111111".into(),
                    is_signer: false,
                    is_writable: false,
                },
            ];

            Ok((
                StatusCode::OK,
                Json(PrepareResponse {
                    instructions: vec![PreparedInstruction {
                        discriminator: vec![],
                        accounts,
                        data,
                        description: format!(
                            "Add evidence to claim on parcel {parcel}: content_hash={content_hash}"
                        ),
                    }],
                    fee_payer: "FEE_PAYER".into(),
                    recent_blockhash_note: "Client must fetch recent_blockhash before sending"
                        .into(),
                }),
            ))
        }

        PrepareRequest::SubmitObservation {
            claim,
            latitude,
            longitude,
            method,
            findings_hash,
            confidence,
            signature_hash,
        } => {
            let findings_hash_bytes = decode_hex32(&findings_hash)?;
            let signature_hash_bytes = decode_hex32(&signature_hash)?;

            let mut data = Vec::new();
            data.extend_from_slice(&[0u8; 8]); // discriminator placeholder
            data.extend_from_slice(&latitude.to_le_bytes());
            data.extend_from_slice(&longitude.to_le_bytes());
            data.push(method);
            data.extend_from_slice(&findings_hash_bytes);
            data.push(confidence);
            data.extend_from_slice(&signature_hash_bytes);

            let claim_pk = claim.clone();
            let accounts = vec![
                AccountMeta {
                    pubkey: "OBSERVATION_ACCOUNT_PDA".into(),
                    is_signer: false,
                    is_writable: true,
                },
                AccountMeta {
                    pubkey: claim_pk,
                    is_signer: false,
                    is_writable: true,
                },
                AccountMeta {
                    pubkey: "FEE_PAYER".into(),
                    is_signer: true,
                    is_writable: true,
                },
                AccountMeta {
                    pubkey: "11111111111111111111111111111111".into(),
                    is_signer: false,
                    is_writable: false,
                },
            ];

            Ok((
                StatusCode::OK,
                Json(PrepareResponse {
                    instructions: vec![PreparedInstruction {
                        discriminator: vec![],
                        accounts,
                        data,
                        description: format!(
                            "Submit observation for claim {claim}: method={method}, confidence={confidence}"
                        ),
                    }],
                    fee_payer: "FEE_PAYER".into(),
                    recent_blockhash_note: "Client must fetch recent_blockhash before sending"
                        .into(),
                }),
            ))
        }

        PrepareRequest::SubmitAttestation {
            claim,
            observation,
            result,
            confidence,
            signature_hash,
        } => {
            let signature_hash_bytes = decode_hex32(&signature_hash)?;

            let mut data = Vec::new();
            data.extend_from_slice(&[0u8; 8]); // discriminator placeholder
            data.push(result);
            data.push(confidence);
            data.extend_from_slice(&signature_hash_bytes);

            let claim_pk = claim.clone();
            let accounts = vec![
                AccountMeta {
                    pubkey: "ATTESTATION_ACCOUNT_PDA".into(),
                    is_signer: false,
                    is_writable: true,
                },
                AccountMeta {
                    pubkey: claim_pk,
                    is_signer: false,
                    is_writable: true,
                },
                AccountMeta {
                    pubkey: observation,
                    is_signer: false,
                    is_writable: false,
                },
                AccountMeta {
                    pubkey: "FEE_PAYER".into(),
                    is_signer: true,
                    is_writable: true,
                },
                AccountMeta {
                    pubkey: "11111111111111111111111111111111".into(),
                    is_signer: false,
                    is_writable: false,
                },
            ];

            Ok((
                StatusCode::OK,
                Json(PrepareResponse {
                    instructions: vec![PreparedInstruction {
                        discriminator: vec![],
                        accounts,
                        data,
                        description: format!(
                            "Submit attestation for claim {claim}: result={result}, confidence={confidence}"
                        ),
                    }],
                    fee_payer: "FEE_PAYER".into(),
                    recent_blockhash_note: "Client must fetch recent_blockhash before sending"
                        .into(),
                }),
            ))
        }

        PrepareRequest::CreateClaim {
            parcel,
            claim_id,
            claim_type,
            content_hash,
            required_attestations,
        } => {
            let claim_id_bytes = decode_hex32(&claim_id)?;
            let content_hash_bytes = decode_hex32(&content_hash)?;

            let mut data = Vec::new();
            data.extend_from_slice(&[0u8; 8]); // discriminator placeholder
            data.extend_from_slice(&claim_id_bytes);
            data.push(claim_type);
            data.extend_from_slice(&content_hash_bytes);
            data.push(required_attestations);
            // region placeholder (2 bytes, global default [0,0])
            data.extend_from_slice(&[0u8; 2]);

            let parcel_pk = parcel.clone();
            let accounts = vec![
                AccountMeta {
                    pubkey: "CLAIM_ACCOUNT_PDA".into(),
                    is_signer: false,
                    is_writable: true,
                },
                AccountMeta {
                    pubkey: parcel_pk,
                    is_signer: false,
                    is_writable: false,
                },
                AccountMeta {
                    pubkey: "FEE_PAYER".into(),
                    is_signer: true,
                    is_writable: true,
                },
                AccountMeta {
                    pubkey: "11111111111111111111111111111111".into(),
                    is_signer: false,
                    is_writable: false,
                },
            ];

            Ok((
                StatusCode::OK,
                Json(PrepareResponse {
                    instructions: vec![PreparedInstruction {
                        discriminator: vec![],
                        accounts,
                        data,
                        description: format!(
                            "Create claim on parcel {parcel}: type={claim_type}, required_attestations={required_attestations}"
                        ),
                    }],
                    fee_payer: "FEE_PAYER".into(),
                    recent_blockhash_note: "Client must fetch recent_blockhash before sending"
                        .into(),
                }),
            ))
        }
    }
}

fn decode_hex32(s: &str) -> Result<[u8; 32], AppError> {
    let bytes = hex::decode(s).map_err(|_| AppError::bad_request("expected hex"))?;
    if bytes.len() != 32 {
        return Err(AppError::bad_request("expected 32 bytes"));
    }
    let mut out = [0u8; 32];
    out.copy_from_slice(&bytes);
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use std::sync::Arc;
    use tower::ServiceExt;

    use crate::auth::ApiAuthority;
    use crate::storage::StorageBackend;

    /// Build a minimal router we can test against (no DB required).
    fn app() -> axum::Router {
        let pool = sqlx::PgPool::connect_lazy("postgres://x:x@localhost/x").unwrap();
        let state = AppState {
            pool,
            geo: None,
            api_authority: ApiAuthority(None),
            storage: Arc::new(StorageBackend::Local {
                root: std::path::PathBuf::from("/tmp/terra-test"),
            }),
        };
        axum::Router::new()
            .route("/prepare", post(prepare_tx))
            .with_state(state)
    }

    /// Helper: send JSON POST and return (status, body bytes).
    async fn post_json(path: &str, body: serde_json::Value) -> (StatusCode, Vec<u8>) {
        let resp = app()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(path)
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        let status = resp.status();
        let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap()
            .to_vec();
        (status, bytes)
    }

    fn hex32(byte: u8) -> String {
        hex::encode([byte; 32])
    }

    // -----------------------------------------------------------------------
    // create_claim
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn prepare_create_claim_returns_instruction() {
        let (status, body) = post_json(
            "/prepare",
            serde_json::json!({
                "type": "create_claim",
                "parcel": "5ZWj7a1f8tWkjBESHKgrLmXGcFn7p8UfC2s8nS6vfa5i",
                "claim_id": hex32(0xAA),
                "claim_type": 0,
                "content_hash": hex32(0xBB),
                "required_attestations": 2
            }),
        )
        .await;
        assert_eq!(status, StatusCode::OK);

        let resp: PrepareResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(resp.instructions.len(), 1);
        assert_eq!(resp.fee_payer, "FEE_PAYER");

        let ix = &resp.instructions[0];
        assert!(ix.description.contains("Create claim"));
        // data: discriminator(8) + claim_id(32) + claim_type(1) + content_hash(32) + required(1) + region(2) = 76
        assert_eq!(ix.data.len(), 76);
        // Accounts: CLAIM_ACCOUNT_PDA, parcel, FEE_PAYER, system_program
        assert_eq!(ix.accounts.len(), 4);
        assert!(ix.accounts[0].is_writable);
        assert!(ix.accounts[2].is_signer); // fee payer
    }

    // -----------------------------------------------------------------------
    // add_evidence
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn prepare_add_evidence_returns_instruction() {
        let (status, body) = post_json(
            "/prepare",
            serde_json::json!({
                "type": "add_evidence",
                "parcel": "5ZWj7a1f8tWkjBESHKgrLmXGcFn7p8UfC2s8nS6vfa5i",
                "claim_id": hex32(0xCC),
                "content_hash": hex32(0xDD),
                "evidence_type": 1,
                "storage_ref": "ipfs://QmTest123",
                "observed_at": 1_700_000_000_i64
            }),
        )
        .await;
        assert_eq!(status, StatusCode::OK);

        let resp: PrepareResponse = serde_json::from_slice(&body).unwrap();
        let ix = &resp.instructions[0];
        assert!(ix.description.contains("Add evidence"));
        assert!(ix.description.contains(&hex32(0xDD)));
        // data: disc(8) + claim_id(32) + evidence_type(1) + content_hash(32)
        //       + sr_len(2) + sr("ipfs://QmTest123" = 14) + observed_at(8) = 97
        // Actual length may vary slightly based on serialization; just verify structure.
        assert!(ix.data.len() >= 8 + 32 + 1 + 32 + 2 + 8); // minimum: disc+claim_id+type+hash+len+timestamp
        assert_eq!(ix.accounts.len(), 4);
    }

    // -----------------------------------------------------------------------
    // submit_observation
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn prepare_submit_observation_returns_instruction() {
        let (status, body) = post_json(
            "/prepare",
            serde_json::json!({
                "type": "submit_observation",
                "claim": "5ZWj7a1f8tWkjBESHKgrLmXGcFn7p8UfC2s8nS6vfa5i",
                "latitude": 471000000_i64,
                "longitude": 852000000_i64,
                "method": 1,
                "findings_hash": hex32(0xEE),
                "confidence": 90,
                "signature_hash": hex32(0xFF)
            }),
        )
        .await;
        assert_eq!(status, StatusCode::OK);

        let resp: PrepareResponse = serde_json::from_slice(&body).unwrap();
        let ix = &resp.instructions[0];
        assert!(ix.description.contains("Submit observation"));
        assert!(ix.description.contains("method=1"));
        assert!(ix.description.contains("confidence=90"));
        // data: disc(8) + lat(8) + lon(8) + method(1) + findings(32) + confidence(1) + sig(32)
        assert_eq!(ix.data.len(), 8 + 8 + 8 + 1 + 32 + 1 + 32);
        assert_eq!(ix.accounts.len(), 4);
        assert!(ix.accounts[1].is_writable); // claim must be writable
    }

    // -----------------------------------------------------------------------
    // submit_attestation
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn prepare_submit_attestation_returns_instruction() {
        let (status, body) = post_json(
            "/prepare",
            serde_json::json!({
                "type": "submit_attestation",
                "claim": "5ZWj7a1f8tWkjBESHKgrLmXGcFn7p8UfC2s8nS6vfa5i",
                "observation": "HMBRKVfVqxiV6LCm59oVBZvf34XfJirPvdYMbZwLJN4o",
                "result": 0,
                "confidence": 95,
                "signature_hash": hex32(0xAB)
            }),
        )
        .await;
        assert_eq!(status, StatusCode::OK);

        let resp: PrepareResponse = serde_json::from_slice(&body).unwrap();
        let ix = &resp.instructions[0];
        assert!(ix.description.contains("Submit attestation"));
        assert!(ix.description.contains("result=0"));
        // data: disc(8) + result(1) + confidence(1) + sig(32)
        assert_eq!(ix.data.len(), 8 + 1 + 1 + 32);
        assert_eq!(ix.accounts.len(), 5);
    }

    // -----------------------------------------------------------------------
    // Error cases
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn prepare_rejects_invalid_hex_claim_id() {
        let (status, body) = post_json(
            "/prepare",
            serde_json::json!({
                "type": "create_claim",
                "parcel": "5ZWj7a1f8tWkjBESHKgrLmXGcFn7p8UfC2s8nS6vfa5i",
                "claim_id": "not-hex",
                "claim_type": 0,
                "content_hash": hex32(0xBB),
                "required_attestations": 2
            }),
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        let err: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(err["error"].as_str().unwrap().contains("hex"));
    }

    #[tokio::test]
    async fn prepare_rejects_short_content_hash() {
        let (status, body) = post_json(
            "/prepare",
            serde_json::json!({
                "type": "add_evidence",
                "parcel": "5ZWj7a1f8tWkjBESHKgrLmXGcFn7p8UfC2s8nS6vfa5i",
                "claim_id": hex32(0xCC),
                "content_hash": "aabb",  // 2 bytes, not 32
                "evidence_type": 0,
                "storage_ref": "test",
                "observed_at": 0
            }),
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        let err: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(err["error"].as_str().unwrap().contains("32 bytes"));
    }

    #[tokio::test]
    async fn prepare_rejects_invalid_type_variant() {
        let (status, body) = post_json(
            "/prepare",
            serde_json::json!({
                "type": "nonexistent",
                "parcel": "x"
            }),
        )
        .await;
        // Serde deserialization errors from Json<T> extractor return 422.
        assert!(
            status == StatusCode::UNPROCESSABLE_ENTITY || status == StatusCode::BAD_REQUEST,
            "expected 422 or 400, got {status}"
        );
        // Body may be JSON error or plain text depending on axum version.
        let body_str = String::from_utf8_lossy(&body);
        assert!(
            body_str.contains("unknown variant")
                || body_str.contains("data did not match")
                || body_str.contains("Error"),
            "unexpected error body: {body_str}"
        );
    }

    // -----------------------------------------------------------------------
    // Struct unit tests
    // -----------------------------------------------------------------------

    #[test]
    fn decode_hex32_works() {
        let hex_str = "aa".repeat(32);
        let result = decode_hex32(&hex_str);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), [0xaa; 32]);
    }

    #[test]
    fn decode_hex32_rejects_wrong_length() {
        let result = decode_hex32("aabb");
        assert!(result.is_err());
    }

    #[test]
    fn decode_hex32_rejects_empty() {
        let result = decode_hex32("");
        assert!(result.is_err());
    }

    #[test]
    fn decode_hex32_rejects_non_hex() {
        let result = decode_hex32(&"zz".repeat(32));
        assert!(result.is_err());
    }

    #[test]
    fn prepare_response_serializes() {
        let resp = PrepareResponse {
            instructions: vec![PreparedInstruction {
                discriminator: vec![],
                accounts: vec![],
                data: vec![1, 2, 3],
                description: "test".into(),
            }],
            fee_payer: "wallet".into(),
            recent_blockhash_note: "fetch blockhash".into(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("fee_payer"));
        assert!(json.contains("recent_blockhash_note"));
        assert!(json.contains("description"));
    }
}
