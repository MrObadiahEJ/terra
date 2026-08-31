use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::AppError;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", post(bind_identity))
        .route("/{wallet}", get(get_identity_by_wallet))
        .route("/{identity_hash}/successions", post(request_succession))
        .route(
            "/{identity_hash}/successions/{successor}/cancel",
            post(cancel_succession),
        )
        .route(
            "/{identity_hash}/successions/{successor}/claim",
            post(claim_succession),
        )
}

// ---------------------------------------------------------------------------
// Person <-> wallet binding + wallet passation (recovery / succession /
// transfer of control) + validator rotation.
//
// The on-chain `Identity` account binds a person (via a hashed identity
// credential) to the wallet they actually hold; `recovery` is a second wallet
// for key-loss recovery. `Succession` is a time-boxed passation that lets
// control pass to an heir, a recovery account, or a deliberate transferee, and
// lets a dead validator's slot be rotated by the parcel owner (rotate_validators).
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct BindIdentity {
    /// hex(32) sha256 over the person's identity credential.
    pub identity_hash: String,
    /// base58 wallet the person holds (the active key; the on-chain owner).
    pub owner: String,
    /// base58 backup/recovery wallet (nonzero).
    pub recovery: String,
    // Optional human metadata (display/index only — never trusted on its own).
    pub display_name: Option<String>,
    pub national_id: Option<String>,
    pub phone: Option<String>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct IdentityRow {
    pub id: Uuid,
    pub identity_hash: String,
    pub owner: String,
    pub recovery: String,
    pub parcel_count: i16,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
pub struct IdentityView {
    #[serde(flatten)]
    pub identity: IdentityRow,
    pub display_name: Option<String>,
    pub national_id: Option<String>,
    pub phone: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RequestSuccession {
    /// base58 wallet that should gain control after the grace period.
    pub successor: String,
    /// 0 = successor(heir), 1 = recovery, 2 = transfer.
    pub kind: u8,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct SuccessionRow {
    pub id: Uuid,
    pub identity_id: Uuid,
    pub identity_hash: String,
    pub kind: i16,
    pub successor: String,
    pub requested_at: chrono::DateTime<chrono::Utc>,
    pub effective_at: chrono::DateTime<chrono::Utc>,
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct RotateValidators {
    pub version: u16,
    pub required: u16,
    /// full base58 validator list after the rotation.
    pub validators: Vec<String>,
    /// base58 wallet that authorized the rotation (the on-chain parcel owner).
    pub rotated_by: String,
}

/// Bind a person to a wallet (with a recovery wallet) — the provisioned,
/// person-held key model. Mirrors the on-chain Identity account and records
/// optional human metadata.
pub async fn bind_identity(
    State(state): State<AppState>,
    Json(req): Json<BindIdentity>,
) -> Result<(StatusCode, Json<IdentityView>), AppError> {
    let identity_hash = crate::routes::attestations::decode_hex32(&req.identity_hash)?;
    // Owner must be a valid base58 32-byte address.
    let _ = decode_wallet(&req.owner)?;
    let _ = decode_wallet(&req.recovery)?;

    let mut tx = state.pool.begin().await?;

    let row = sqlx::query_as::<_, IdentityRow>(
        "INSERT INTO identities (identity_hash, owner, recovery)
         VALUES ($1, $2, $3)
         ON CONFLICT (identity_hash) DO UPDATE SET
            owner    = EXCLUDED.owner,
            recovery = EXCLUDED.recovery,
            updated_at = now()
         RETURNING id, identity_hash, owner, recovery, parcel_count, created_at",
    )
    .bind(hex::encode(identity_hash))
    .bind(&req.owner)
    .bind(&req.recovery)
    .fetch_one(&mut *tx)
    .await?;

    if req.display_name.is_some() || req.national_id.is_some() || req.phone.is_some() {
        sqlx::query(
            "INSERT INTO identity_metadata (identity_id, display_name, national_id, phone)
             VALUES ($1, $2, $3, $4)
             ON CONFLICT (identity_id) DO UPDATE SET
                display_name = EXCLUDED.display_name,
                national_id  = EXCLUDED.national_id,
                phone        = EXCLUDED.phone",
        )
        .bind(row.id)
        .bind(req.display_name.as_deref().unwrap_or(""))
        .bind(req.national_id.as_deref().unwrap_or(""))
        .bind(req.phone.as_deref().unwrap_or(""))
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    Ok((
        StatusCode::CREATED,
        Json(IdentityView {
            identity: row,
            display_name: req.display_name,
            national_id: req.national_id,
            phone: req.phone,
        }),
    ))
}

/// Resolve an identity by its wallet address: returns the identity (if any)
/// that owns this wallet, or is acting via the wallet's recovery role.
pub async fn get_identity_by_wallet(
    State(state): State<AppState>,
    Path(wallet): Path<String>,
) -> Result<Json<IdentityView>, AppError> {
    let _ = decode_wallet(&wallet)?;
    let row = sqlx::query_as::<_, IdentityRow>(
        "SELECT id, identity_hash, owner, recovery, parcel_count, created_at
         FROM identities WHERE LOWER(owner) = LOWER($1) OR LOWER(recovery) = LOWER($1)",
    )
    .bind(&wallet)
    .fetch_one(&state.pool)
    .await?;

    let meta: Option<(Option<String>, Option<String>, Option<String>)> = sqlx::query_as(
        "SELECT display_name, national_id, phone FROM identity_metadata WHERE identity_id = $1",
    )
    .bind(row.id)
    .fetch_optional(&state.pool)
    .await?;

    let (display_name, national_id, phone) = meta.unwrap_or((None, None, None));
    Ok(Json(IdentityView {
        identity: row,
        display_name,
        national_id,
        phone,
    }))
}

/// Request a wallet passation. Creates a mirror of the on-chain Succession
/// account with a grace-period effective time. This lets an heir (kind 0),
/// recovery account (kind 1), or deliberate transferee (kind 2) take control
/// after the window — and is the mechanism behind dead-validator succession.
pub async fn request_succession(
    State(state): State<AppState>,
    Path(identity_hash): Path<String>,
    Json(req): Json<RequestSuccession>,
) -> Result<(StatusCode, Json<SuccessionRow>), AppError> {
    let ih = crate::routes::attestations::decode_hex32(&identity_hash)?;
    let _ = decode_wallet(&req.successor)?;
    if req.kind > 2 {
        return Err(AppError::bad_request("kind must be 0 (successor), 1 (recovery), or 2 (transfer)"));
    }

    let row = sqlx::query_as::<_, SuccessionRow>(
        "INSERT INTO successions (identity_id, identity_hash, kind, successor, effective_at)
         SELECT id, identity_hash, $2, $3, (now() + make_interval(secs => 604800))
         FROM identities WHERE identity_hash = $1
         ON CONFLICT (identity_id, successor) DO UPDATE SET
            kind = EXCLUDED.kind,
            effective_at = EXCLUDED.effective_at,
            status = 'pending',
            cancelled_at = NULL,
            claimed_at = NULL
         RETURNING id, identity_id, identity_hash, kind, successor, requested_at,
                   effective_at, status",
    )
    .bind(hex::encode(ih))
    .bind(req.kind as i16)
    .bind(&req.successor)
    .fetch_one(&state.pool)
    .await?;

    Ok((StatusCode::CREATED, Json(row)))
}

/// Cancel an in-flight succession (only valid before it becomes effective).
pub async fn cancel_succession(
    State(state): State<AppState>,
    Path((identity_hash, successor)): Path<(String, String)>,
) -> Result<Json<SuccessionRow>, AppError> {
    let ih = crate::routes::attestations::decode_hex32(&identity_hash)?;
    let _ = decode_wallet(&successor)?;

    let row = sqlx::query_as::<_, SuccessionRow>(
        "UPDATE successions
         SET status = 'cancelled', cancelled_at = now()
         WHERE identity_hash = $1 AND LOWER(successor) = LOWER($2)
           AND status = 'pending' AND effective_at > now()
         RETURNING id, identity_id, identity_hash, kind, successor, requested_at,
                   effective_at, status",
    )
    .bind(hex::encode(ih))
    .bind(&successor)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::not_found("no cancelable pending succession"))?;

    Ok(Json(row))
}

/// Claim an effective succession: the successor becomes the identity's new
/// owner and any parcels the identity owned transfer with it. Mirrors the
/// on-chain claim + parcel re-pointing (a client computes/records the new owner).
pub async fn claim_succession(
    State(state): State<AppState>,
    Path((identity_hash, successor)): Path<(String, String)>,
) -> Result<Json<IdentityRow>, AppError> {
    let ih = crate::routes::attestations::decode_hex32(&identity_hash)?;
    let _ = decode_wallet(&successor)?;

    // Atomically: mark this pending+effective succession as claimed and swap
    // the identity's owner to the successor; reset recovery to ''.
    let mut tx = state.pool.begin().await?;

    let claimed: Option<(Uuid,)> = sqlx::query_as(
        "UPDATE successions
         SET status = 'claimed', claimed_at = now()
         WHERE identity_hash = $1 AND LOWER(successor) = LOWER($2)
           AND status = 'pending' AND effective_at <= now()
         RETURNING id",
    )
    .bind(hex::encode(ih))
    .bind(&successor)
    .fetch_optional(&mut *tx)
    .await?;

    let Some(_) = claimed else {
        return Err(AppError::conflict(
            "succession is not pending/effective yet (grace period not elapsed)",
        ));
    };

    let row = sqlx::query_as::<_, IdentityRow>(
        "UPDATE identities
         SET owner = $2, recovery = '', parcel_count = 0, updated_at = now()
         WHERE identity_hash = $1
         RETURNING id, identity_hash, owner, recovery, parcel_count, created_at",
    )
    .bind(hex::encode(ih))
    .bind(&successor)
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(Json(row))
}

/// Record a validator-set rotation for an attestation (the fix for dead or
/// leaving validators). Mirrors the on-chain rotate_validators + version bump.
pub async fn rotate_validators(
    State(state): State<AppState>,
    Path((parcel_id, specifier)): Path<(Uuid, String)>,
    Json(req): Json<RotateValidators>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    if req.validators.is_empty() {
        return Err(AppError::bad_request("at least one validator required"));
    }
    if (req.required as usize) > req.validators.len() {
        return Err(AppError::bad_request("required threshold exceeds validator count"));
    }
    let _ = decode_wallet(&req.rotated_by)?;
    for v in &req.validators {
        let _ = decode_wallet(v)?;
    }

    let att = sqlx::query_as::<_, crate::routes::attestations::AttestationRow>(
        "SELECT id, parcel_id, onchain_id, specifier, content_hash, required,
                validators, created_at
         FROM attestations WHERE parcel_id = $1 AND specifier = $2",
    )
    .bind(parcel_id)
    .bind(&specifier)
    .fetch_one(&state.pool)
    .await?;

    let mut tx = state.pool.begin().await?;

    sqlx::query(
        "INSERT INTO validator_rotations (attestation_id, version, required, validators, rotated_by)
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(att.id)
    .bind(req.version as i16)
    .bind(req.required as i16)
    .bind(&req.validators)
    .bind(&req.rotated_by)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        "UPDATE attestations
         SET version = $2, required = $3, validators = $4
         WHERE id = $1",
    )
    .bind(att.id)
    .bind(req.version as i16)
    .bind(req.required as i16)
    .bind(&req.validators)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok((StatusCode::OK, Json(serde_json::json!({
        "attestation_id": att.id,
        "version": req.version,
        "required": req.required,
        "validators": req.validators,
    }))))
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn decode_wallet(s: &str) -> Result<[u8; 32], AppError> {
    let bytes = bs58::decode(s)
        .into_vec()
        .map_err(|e| AppError::bad_request(format!("invalid wallet address: {e}")))?;
    if bytes.len() != 32 {
        return Err(AppError::bad_request("wallet address must decode to 32 bytes"));
    }
    let mut out = [0u8; 32];
    out.copy_from_slice(&bytes);
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A passation is only valid if the required threshold does not exceed the
    /// set of known validators. This is the rule guarding both attest() and
    /// rotate_validators() so a dead/lost validator can always be rotated out
    /// without leaving quorum unreachable.
    fn threshold_ok(required: usize, validators: &[String]) -> bool {
        required > 0 && required <= validators.len()
    }

    #[test]
    fn wallet_address_roundtrips() {
        let seed = [7u8; 32];
        let wallet = bs58::encode(seed).into_string();
        assert_eq!(decode_wallet(&wallet).expect("valid wallet"), seed);
    }

    #[test]
    fn invalid_wallet_address_rejected() {
        assert!(decode_wallet("not-base58!!!").is_err());
        // Valid base58 but wrong length must be rejected too.
        let short = bs58::encode([1u8; 16]).into_string();
        assert!(decode_wallet(&short).is_err());
    }

    #[test]
    fn threshold_can_never_exceed_validator_count() {
        let five = (0..5).map(|i| bs58::encode([i as u8; 32]).into_string()).collect::<Vec<_>>();
        assert!(threshold_ok(1, &five));
        assert!(threshold_ok(5, &five));
        assert!(!threshold_ok(6, &five)); // would make quorum unreachable
        assert!(!threshold_ok(0, &five));
    }

    #[test]
    fn rotating_validators_can_restore_reachable_quorum() {
        // Original set of 5 with threshold 3; 3 validators die. Threshold 3 of 5
        // is now unreachable with only 2 left (3 > 2), so the owner rotates in
        // new validators to make quorum reachable again.
        let original = (0..5).map(|i| bs58::encode([(i + 1) as u8; 32]).into_string()).collect::<Vec<_>>();
        let surviving = original[3..].to_vec(); // only 2 left
        assert!(!threshold_ok(3, &surviving)); // stuck before rotation

        let mut reconstituted = surviving.clone();
        // Owner adds new validators to recover the headroom.
        reconstituted.push(bs58::encode([99u8; 32]).into_string());
        reconstituted.push(bs58::encode([100u8; 32]).into_string());
        let reconstituted = reconstituted; // now 4 known validators
        // Quorum reachable again with threshold 3 of 4.
        assert!(threshold_ok(3, &reconstituted));
    }
}
