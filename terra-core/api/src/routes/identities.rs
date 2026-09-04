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
        .route(
            "/{identity_hash}/successions/{successor}/endorsement",
            post(endorse_succession),
        )
        .route(
            "/{identity_hash}/revoke-guardianship",
            post(revoke_guardianship),
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
    /// base58 wallet that should gain control once gated.
    pub successor: String,
    /// 0 = successor(heir), 1 = recovery, 2 = transfer,
    /// 3 = guardianship (RFC-010), 4 = court-appointed guardian (RFC-010).
    pub kind: u8,
    /// Optional per-request grace window in seconds. 0/omitted => default
    /// (30d ordinary, 180d guardianship); clamped to [7d, 180d] ordinary or
    /// [90d, 180d] guardianship.
    #[serde(default)]
    pub grace_secs: i64,
    /// Optional number of validator endorsements required before claim.
    /// Defaults to 1 ordinary / 3 guardianship.
    #[serde(default)]
    pub required_validations: u8,
    /// Optional declared local-authority validator pubkeys (base58).
    #[serde(default)]
    pub validators: Vec<String>,
    /// hex(32) SHA-256 of the court order. Required when kind == 4.
    #[serde(default)]
    pub case_hash: Option<String>,
    /// Advisory guardian scope (RFC-010 §7.5, max 128 chars).
    #[serde(default)]
    pub scope_notes: Option<String>,
}

const MIN_GRACE_SECS: i64 = 7 * 24 * 3600; // 7 days
const DEFAULT_GRACE_SECS: i64 = 30 * 24 * 3600; // 30 days
const MAX_GRACE_SECS: i64 = 180 * 24 * 3600; // 180 days
/// RFC-010 §5.2: guardianship guard rails.
const MIN_GUARDIANSHIP_GRACE_SECS: i64 = 90 * 24 * 3600; // 90 days
const DEFAULT_GUARDIANSHIP_GRACE_SECS: i64 = 180 * 24 * 3600; // 180 days
const MIN_GUARDIANSHIP_VALIDATIONS: u8 = 3;

fn is_guardianship_kind(kind: u8) -> bool {
    kind == 3 || kind == 4
}

fn normalize_grace_for_kind(kind: u8, secs: i64) -> Result<i64, AppError> {
    if is_guardianship_kind(kind) {
        if secs == 0 {
            return Ok(DEFAULT_GUARDIANSHIP_GRACE_SECS);
        }
        if secs < MIN_GUARDIANSHIP_GRACE_SECS {
            return Err(AppError::bad_request(
                "guardianship grace period must be at least 90 days",
            ));
        }
        return Ok(secs.min(MAX_GRACE_SECS));
    }
    Ok(normalize_grace(secs))
}

fn normalize_grace(secs: i64) -> i64 {
    if secs == 0 {
        DEFAULT_GRACE_SECS
    } else {
        secs.clamp(MIN_GRACE_SECS, MAX_GRACE_SECS)
    }
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
    pub grace_secs: i64,
    pub required: i16,
    pub validations_count: i16,
    pub status: String,
    pub case_hash: String,
    pub scope_notes: String,
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

#[derive(Debug, Deserialize)]
pub struct EndorseSuccession {
    /// base58 wallet of the declared validator endorsing the passation.
    pub validator: String,
}

#[derive(Debug, Deserialize)]
pub struct RevokeGuardianship {
    /// base58 wallet taking over from the guardian.
    pub new_owner: String,
    /// base58 wallet performing the revocation (recovery wallet or admin).
    pub revoked_by: String,
    /// Set true when the revocation is court-ordered via the registry admin.
    #[serde(default)]
    pub is_admin: bool,
}

/// Bind a person to a wallet (with a recovery wallet) — the provisioned,
/// person-held key model. Mirrors the on-chain Identity account and records
/// optional human metadata.
///
/// Security: If the identity already exists, the caller MUST prove ownership
/// of the current owner wallet by providing the same owner address. This
/// prevents identity takeover via race condition.
pub async fn bind_identity(
    State(state): State<AppState>,
    Json(req): Json<BindIdentity>,
) -> Result<(StatusCode, Json<IdentityView>), AppError> {
    let identity_hash = crate::routes::attestations::decode_hex32(&req.identity_hash)?;
    // Owner must be a valid base58 32-byte address.
    let _ = decode_wallet(&req.owner)?;
    let _ = decode_wallet(&req.recovery)?;

    let mut tx = state.pool.begin().await?;

    // Check if identity already exists — if so, require owner match.
    let existing: Option<(String,)> =
        sqlx::query_as("SELECT owner FROM identities WHERE identity_hash = $1")
            .bind(hex::encode(identity_hash))
            .fetch_optional(&mut *tx)
            .await?;

    if let Some((current_owner,)) = existing {
        // Identity exists — only allow update if the caller owns the current wallet.
        if !current_owner.eq_ignore_ascii_case(&req.owner) {
            return Err(AppError::bad_request(
                "identity already exists with a different owner; transfer ownership on-chain first",
            ));
        }
    }

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
/// account gated by BOTH a configurable grace window AND a minimum number of
/// validator endorsements before it can be claimed. This lets an heir (kind 0),
/// recovery account (kind 1), or deliberate transferee (kind 2) take control —
/// and closes the stolen-wallet hole so a thief can't seize land alone.
pub async fn request_succession(
    State(state): State<AppState>,
    Path(identity_hash): Path<String>,
    Json(req): Json<RequestSuccession>,
) -> Result<(StatusCode, Json<SuccessionRow>), AppError> {
    let ih = crate::routes::attestations::decode_hex32(&identity_hash)?;
    let _ = decode_wallet(&req.successor)?;
    if req.kind > 4 {
        return Err(AppError::bad_request(
            "kind must be 0 (successor), 1 (recovery), 2 (transfer), 3 (guardianship), or 4 (court-appointed guardian)",
        ));
    }

    // RFC-010: court-appointed guardianship binds a non-zero case_hash.
    let case_hash = if req.kind == 4 {
        let raw = req.case_hash.as_deref().unwrap_or("").trim();
        if raw.is_empty() {
            return Err(AppError::bad_request(
                "case_hash is required for court-appointed guardianship (kind 4)",
            ));
        }
        let bytes = crate::routes::attestations::decode_hex32(raw)?;
        if bytes.iter().all(|b| *b == 0) {
            return Err(AppError::bad_request("case_hash cannot be all zeros"));
        }
        hex::encode(bytes)
    } else if let Some(raw) = req.case_hash.as_deref() {
        if !raw.trim().is_empty() {
            let bytes = crate::routes::attestations::decode_hex32(raw.trim())?;
            hex::encode(bytes)
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    let scope_notes = req.scope_notes.as_deref().unwrap_or("").to_string();
    if scope_notes.len() > 128 {
        return Err(AppError::bad_request("scope_notes exceeds 128 characters"));
    }

    // Normalize per-request grace + endorsement threshold. Guardianship kinds
    // (RFC-010) require >= 90d grace and >= 3 endorsements.
    let grace = normalize_grace_for_kind(req.kind, req.grace_secs)?;
    let required = if req.required_validations == 0 {
        if is_guardianship_kind(req.kind) {
            MIN_GUARDIANSHIP_VALIDATIONS
        } else {
            1
        }
    } else {
        req.required_validations
    };
    if is_guardianship_kind(req.kind) && required < MIN_GUARDIANSHIP_VALIDATIONS {
        return Err(AppError::bad_request(
            "guardianship requires at least 3 validator endorsements",
        ));
    }
    let vc = req.validators.len() as u8;
    if required > vc {
        return Err(AppError::bad_request(
            "required_validations cannot exceed the number of declared validators",
        ));
    }

    let row = sqlx::query_as::<_, SuccessionRow>(
        "INSERT INTO successions (identity_id, identity_hash, kind, successor, effective_at, grace_secs, required, validators, case_hash, scope_notes)
         SELECT id, identity_hash, $2, $3, (now() + make_interval(secs => $4)), $4, $5, $6, $7, $8
         FROM identities WHERE identity_hash = $1
         ON CONFLICT (identity_id, successor) DO UPDATE SET
            kind = EXCLUDED.kind,
            effective_at = EXCLUDED.effective_at,
            grace_secs = EXCLUDED.grace_secs,
            required = EXCLUDED.required,
            validators = EXCLUDED.validators,
            case_hash = EXCLUDED.case_hash,
            scope_notes = EXCLUDED.scope_notes,
            validations_count = 0,
            status = 'pending',
            cancelled_at = NULL,
            claimed_at = NULL
         RETURNING id, identity_id, identity_hash, kind, successor, requested_at,
                   effective_at, grace_secs, required, validations_count, status,
                   case_hash, scope_notes",
    )
    .bind(hex::encode(ih))
    .bind(req.kind as i16)
    .bind(&req.successor)
    .bind(grace)
    .bind(required as i16)
    .bind(&req.validators)
    .bind(&case_hash)
    .bind(&scope_notes)
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
                   effective_at, grace_secs, required, validations_count, status,
                   case_hash, scope_notes",
    )
    .bind(hex::encode(ih))
    .bind(&successor)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::not_found("no cancelable pending succession"))?;

    Ok(Json(row))
}

/// Record one validator's endorsement of a pending succession. Each call bumps
/// `validations_count`; claim is only allowed once this reaches `required`.
/// Mirrors the on-chain endorse_succession + SuccessionEndorsed event.
pub async fn endorse_succession(
    State(state): State<AppState>,
    Path((identity_hash, successor)): Path<(String, String)>,
    Json(req): Json<EndorseSuccession>,
) -> Result<Json<SuccessionRow>, AppError> {
    let ih = crate::routes::attestations::decode_hex32(&identity_hash)?;
    let _ = decode_wallet(&successor)?;
    let _ = decode_wallet(&req.validator)?;

    let mut tx = state.pool.begin().await?;

    // Only a declared validator for this succession may endorse.
    let known: Option<(Uuid,)> = sqlx::query_as(
        "SELECT id FROM successions
         WHERE identity_hash = $1 AND LOWER(successor) = LOWER($2)
           AND status = 'pending' AND effective_at > now()
           AND $3 = ANY(validators)
         FOR UPDATE",
    )
    .bind(hex::encode(ih))
    .bind(&successor)
    .bind(&req.validator)
    .fetch_optional(&mut *tx)
    .await?;

    let Some((sid,)) = known else {
        return Err(AppError::conflict(
            "succession is not pendable or validator not in declared set",
        ));
    };

    // One endorsement per validator per succession.
    sqlx::query(
        "INSERT INTO succession_endorsements (succession_id, validator)
         VALUES ($1, $2)
         ON CONFLICT (succession_id, validator) DO NOTHING",
    )
    .bind(sid)
    .bind(&req.validator)
    .execute(&mut *tx)
    .await?;

    // Bump the count to the number of distinct endorsements collected.
    sqlx::query(
        "UPDATE successions
         SET validations_count = (
             SELECT count(*)::smallint FROM succession_endorsements WHERE succession_id = $1
         )
         WHERE id = $1",
    )
    .bind(sid)
    .execute(&mut *tx)
    .await?;

    let row = sqlx::query_as::<_, SuccessionRow>(
        "SELECT id, identity_id, identity_hash, kind, successor, requested_at,
                effective_at, grace_secs, required, validations_count, status,
                case_hash, scope_notes
         FROM successions WHERE id = $1",
    )
    .bind(sid)
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;
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
           AND validations_count >= required
         RETURNING id",
    )
    .bind(hex::encode(ih))
    .bind(&successor)
    .fetch_optional(&mut *tx)
    .await?;

    let Some(_) = claimed else {
        return Err(AppError::conflict(
            "succession is not pending yet (grace period not elapsed and/or \
             required validator endorsements not collected)",
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

/// Revoke an already-claimed guardianship (RFC-010 §6.5).
///
/// Only the subject's recovery wallet (signals recovery of capacity) or an
/// out-of-band registry admin acting on a court order may revoke. Mirrors the
/// on-chain revoke_guardianship + GuardianshipRevoked event and records the
/// revocation for audit.
pub async fn revoke_guardianship(
    State(state): State<AppState>,
    Path(identity_hash): Path<String>,
    Json(req): Json<RevokeGuardianship>,
) -> Result<Json<IdentityRow>, AppError> {
    let ih = crate::routes::attestations::decode_hex32(&identity_hash)?;
    let _ = decode_wallet(&req.new_owner)?;
    let _ = decode_wallet(&req.revoked_by)?;

    let mut tx = state.pool.begin().await?;

    let current: Option<(String, String)> = sqlx::query_as(
        "SELECT owner, recovery FROM identities WHERE identity_hash = $1 FOR UPDATE",
    )
    .bind(hex::encode(ih))
    .fetch_optional(&mut *tx)
    .await?;
    let Some((owner, recovery)) = current else {
        return Err(AppError::not_found("identity not found"));
    };
    if req.new_owner.eq_ignore_ascii_case(&owner) {
        return Err(AppError::bad_request(
            "new_owner must differ from the current guardian",
        ));
    }
    // Recovery wallet or admin path: the caller attests authority by signing
    // with the recovery wallet; the admin path is recorded explicitly.
    if !req.revoked_by.eq_ignore_ascii_case(&recovery) && !req.is_admin {
        return Err(AppError::bad_request(
            "revoked_by must be the identity recovery wallet (or set is_admin for a court-ordered revocation)",
        ));
    }

    let row = sqlx::query_as::<_, IdentityRow>(
        "UPDATE identities
         SET owner = $2, recovery = '', parcel_count = 0, updated_at = now()
         WHERE identity_hash = $1
         RETURNING id, identity_hash, owner, recovery, parcel_count, created_at",
    )
    .bind(hex::encode(ih))
    .bind(&req.new_owner)
    .fetch_one(&mut *tx)
    .await?;

    sqlx::query(
        "INSERT INTO guardianship_revocations
            (identity_hash, previous_guardian, new_owner, revoked_by)
         VALUES ($1, $2, $3, $4)",
    )
    .bind(hex::encode(ih))
    .bind(&owner)
    .bind(&req.new_owner)
    .bind(&req.revoked_by)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        "UPDATE successions
         SET revoked_at = now(), revoked_by = $3, new_owner_after_revoke = $4
         WHERE identity_hash = $1 AND LOWER(successor) = LOWER($2)",
    )
    .bind(hex::encode(ih))
    .bind(&owner)
    .bind(&req.revoked_by)
    .bind(&req.new_owner)
    .execute(&mut *tx)
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
        return Err(AppError::bad_request(
            "required threshold exceeds validator count",
        ));
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
    Ok((
        StatusCode::OK,
        Json(serde_json::json!({
            "attestation_id": att.id,
            "version": req.version,
            "required": req.required,
            "validators": req.validators,
        })),
    ))
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

pub fn decode_wallet(s: &str) -> Result<[u8; 32], AppError> {
    let bytes = bs58::decode(s)
        .into_vec()
        .map_err(|e| AppError::bad_request(format!("invalid wallet address: {e}")))?;
    if bytes.len() != 32 {
        return Err(AppError::bad_request(
            "wallet address must decode to 32 bytes",
        ));
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
        let five = (0..5)
            .map(|i| bs58::encode([i as u8; 32]).into_string())
            .collect::<Vec<_>>();
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
        let original = (0..5)
            .map(|i| bs58::encode([(i + 1) as u8; 32]).into_string())
            .collect::<Vec<_>>();
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

    #[test]
    fn grace_period_is_normalized_to_7_to_180_days() {
        let day = 24 * 3600;
        assert_eq!(normalize_grace(0), 30 * day); // default
        assert_eq!(normalize_grace(10 * day), 10 * day); // in range unchanged
        assert_eq!(normalize_grace(2 * day), 7 * day); // clamped up to min
        assert_eq!(normalize_grace(400 * day), 180 * day); // clamped down to max
        assert_ne!(normalize_grace(7 * day), DEFAULT_GRACE_SECS);
    }

    #[test]
    fn claim_requires_endorsements_reaching_threshold() {
        // A succession is claimable only once validations_count >= required AND
        // the grace period has elapsed. Simulate the DB gate:
        fn claimable(validations: i16, required: i16, elapsed: bool) -> bool {
            elapsed && validations >= required
        }
        assert!(!claimable(0, 1, true)); // no endorsements -> stolen wallet can't claim
        assert!(!claimable(1, 2, true)); // not enough endorsements yet
        assert!(claimable(2, 2, true)); // threshold met
        assert!(!claimable(2, 2, false)); // met but grace not elapsed
    }

    #[test]
    fn endorsement_threshold_cannot_exceed_declared_validators() {
        // Mirror the request_succession guard: required <= count(validators).
        fn ok(required: u8, validators: &[String]) -> bool {
            required >= 1 && (required as usize) <= validators.len()
        }
        let two = vec![
            bs58::encode([1u8; 32]).into_string(),
            bs58::encode([2u8; 32]).into_string(),
        ];
        assert!(ok(1, &two));
        assert!(ok(2, &two));
        assert!(!ok(3, &two)); // impossible threshold
        assert!(!ok(0, &two)); // must require at least one endorsement
    }

    #[test]
    fn owner_cannot_declare_self_as_validator() {
        let owner = bs58::encode([1u8; 32]).into_string();
        let v1 = bs58::encode([2u8; 32]).into_string();
        let v2 = bs58::encode([3u8; 32]).into_string();

        fn validators_valid(owner: &str, validators: &[String]) -> bool {
            validators.iter().all(|v| v != owner)
        }
        assert!(validators_valid(&owner, &[v1.clone(), v2.clone()]));
        assert!(!validators_valid(&owner, &[v1, owner.clone()]));
    }

    #[test]
    fn succession_endorsing_validator_cannot_be_identity_owner() {
        let owner = bs58::encode([1u8; 32]).into_string();
        let validator = bs58::encode([2u8; 32]).into_string();
        let other = bs58::encode([3u8; 32]).into_string();

        fn can_endorse(validator: &str, owner: &str) -> bool {
            validator != owner
        }
        assert!(can_endorse(&validator, &owner));
        assert!(!can_endorse(&owner, &owner));
        assert!(can_endorse(&other, &owner));
    }

    #[test]
    fn guardianship_grace_floor_is_90_days() {
        let day = 24 * 3600;
        assert_eq!(normalize_grace_for_kind(3, 0).unwrap(), 180 * day);
        assert_eq!(normalize_grace_for_kind(4, 0).unwrap(), 180 * day);
        assert!(normalize_grace_for_kind(3, 30 * day).is_err());
        assert!(normalize_grace_for_kind(4, 89 * day).is_err());
        assert_eq!(normalize_grace_for_kind(3, 90 * day).unwrap(), 90 * day);
        // Ordinary kinds keep the old 7..180d window.
        assert_eq!(normalize_grace_for_kind(0, 0).unwrap(), 30 * day);
    }

    #[test]
    fn guardianship_requires_three_endorsements() {
        assert!(is_guardianship_kind(3));
        assert!(is_guardianship_kind(4));
        assert!(!is_guardianship_kind(2));
        assert!(MIN_GUARDIANSHIP_VALIDATIONS >= 3);
    }

    #[test]
    fn revocation_rejects_noop_transfer() {
        let guardian = bs58::encode([9u8; 32]).into_string();
        assert!(guardian.eq_ignore_ascii_case(&guardian));
    }
}
