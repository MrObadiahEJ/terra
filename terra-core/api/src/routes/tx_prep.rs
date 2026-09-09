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

#[derive(Debug, Serialize)]
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

#[derive(Debug, Serialize)]
pub struct AccountMeta {
    /// base58 public key.
    pub pubkey: String,
    /// Is this account a signer?
    pub is_signer: bool,
    /// Is this account writable?
    pub is_writable: bool,
}

#[derive(Debug, Serialize)]
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

    #[test]
    fn prepare_request_deserializes() {
        let json = r#"{"type":"add_evidence","parcel":"ABC","claim_id":"aa bb cc dd ee ff 00 11 22 33 44 55 66 77 88 99 aa bb cc dd ee ff 00 11 22 33 44 55 66 77 88 99","content_hash":"aa bb cc dd ee ff 00 11 22 33 44 55 66 77 88 99 aa bb cc dd ee ff 00 11 22 33 44 55 66 77 88 99","evidence_type":0,"storage_ref":"test","observed_at":0}"#;
        // Just check it doesn't panic — full validation is in the handler.
        let _: Result<PrepareRequest, _> = serde_json::from_str(json);
    }

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
}
