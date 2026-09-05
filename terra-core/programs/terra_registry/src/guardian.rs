use anchor_lang::prelude::*;

use crate::TerraError;

// ---------------------------------------------------------------------------
// Constants (RFC-010 §5.2)
// ---------------------------------------------------------------------------

/// Minimum grace period for any guardianship claim: 90 days.
pub const MIN_GUARDIANSHIP_GRACE_SECS: i64 = 90 * 24 * 3600;
/// Default grace when a requester passes 0: 180 days.
pub const DEFAULT_GUARDIANSHIP_GRACE_SECS: i64 = 180 * 24 * 3600;
/// Minimum validator endorsements for guardianship: 3.
pub const MIN_GUARDIANSHIP_VALIDATIONS: u8 = 3;
/// Maximum scope-notes length (mirrors the 128-byte Rights notes bound).
pub const MAX_SCOPE_NOTES_LEN: usize = 128;

/// Returns true for the two guardianship succession kinds (3, 4).
pub fn is_guardianship_kind(kind: u8) -> bool {
    kind == crate::succession_kind::GUARDIANSHIP
        || kind == crate::succession_kind::COURT_APPOINTED_GUARDIAN
}

/// Normalize a requested grace period for a guardianship kind.
/// 0 => default 180d; otherwise must be >= 90d, clamped to the global max.
pub fn normalize_guardianship_grace(grace_secs: i64) -> Result<i64> {
    if grace_secs == 0 {
        return Ok(DEFAULT_GUARDIANSHIP_GRACE_SECS);
    }
    require!(
        grace_secs >= MIN_GUARDIANSHIP_GRACE_SECS,
        TerraError::GuardianshipGraceTooShort
    );
    Ok(grace_secs.min(crate::MAX_SUCCESSION_GRACE_SECS))
}

/// Validate the endorsement threshold for a guardianship request.
pub fn validate_guardianship_threshold(required_validations: u8, declared: usize) -> Result<()> {
    require!(
        required_validations >= MIN_GUARDIANSHIP_VALIDATIONS,
        TerraError::GuardianshipThresholdTooLow
    );
    require!(
        (required_validations as usize) <= declared,
        TerraError::InvalidThreshold
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// Request a court-appointed guardianship with an explicit `case_hash` binding.
///
/// This is the backward-compatible form of RFC-010 §6.1.1 option 1: it
/// initializes the same `Succession` PDA (`["succession", identity, successor]`)
/// with `kind = COURT_APPOINTED_GUARDIAN` (4), enforces the elevated
/// guardianship guards, and anchors the off-chain court order via `case_hash`.
/// The hash is emitted in the event for auditability; the `scope_notes` carry
/// the advisory scope convention (e.g. "limited_to_parcel_<b58>").
pub fn request_court_guardianship(
    ctx: Context<super::RequestCourtGuardianship>,
    successor: Pubkey,
    grace_secs: i64,
    required_validations: u8,
    validators: [Pubkey; crate::MAX_VALIDATORS],
    case_hash: [u8; 32],
    scope_notes: String,
) -> Result<()> {
    require!(successor != Pubkey::default(), TerraError::EmptySuccessor);
    require!(
        !case_hash.iter().all(|b| *b == 0),
        TerraError::EmptyCaseHash
    );
    require!(
        scope_notes.len() <= MAX_SCOPE_NOTES_LEN,
        TerraError::NotesTooLong
    );

    let identity = &ctx.accounts.identity;
    let signer = ctx.accounts.signer.key();
    require!(
        signer == identity.owner || signer == identity.recovery,
        TerraError::NotAuthorized
    );
    require!(successor != identity.owner, TerraError::SuccessorIsOwner);

    let mut count: u8 = 0;
    for &v in validators.iter() {
        if v == Pubkey::default() {
            continue;
        }
        require!(v != identity.owner, TerraError::ValidatorOwnsAsset);
        count += 1;
    }
    require!(count > 0, TerraError::NoValidators);
    validate_guardianship_threshold(required_validations, count as usize)?;

    let grace = normalize_guardianship_grace(grace_secs)?;

    let now = Clock::get()?.unix_timestamp;
    let succession = &mut ctx.accounts.succession;
    succession.identity = identity.key();
    succession.successor = successor;
    succession.kind = crate::succession_kind::COURT_APPOINTED_GUARDIAN;
    succession.requested_at = now;
    succession.grace_secs = grace;
    succession.effective_at = now.saturating_add(grace);
    succession.required = required_validations;
    succession.validations_count = 0;
    succession.validators = validators;

    emit!(CourtGuardianshipRequested {
        identity: identity.key(),
        successor,
        grace_secs: grace,
        required: required_validations,
        count,
        effective_at: succession.effective_at,
        case_hash,
        scope_notes,
    });
    Ok(())
}

/// Revoke an already-claimed guardianship (RFC-010 §6.5).
///
/// Only the subject's recovery wallet (signals recovery of capacity) or the
/// registry admin (acting on a court order) may revoke. The caller names the
/// wallet that takes over — the subject's new active wallet or a new guardian.
///
/// Trust model (explicit): recovery and admin are emergency-brake roles with
/// unilateral power by design — identical in kind to key-recovery in any
/// social-recovery wallet. Revocation requires no validator quorum because
/// the recovery wallet IS the subject's voice; every revocation is fully
/// described by the GuardianshipRevoked event for off-chain audit.
pub fn revoke_guardianship(
    ctx: Context<super::RevokeGuardianship>,
    new_owner: Pubkey,
) -> Result<()> {
    require!(new_owner != Pubkey::default(), TerraError::EmptySuccessor);

    let revoker = ctx.accounts.revoker.key();
    let previous = ctx.accounts.identity.owner;
    require!(new_owner != previous, TerraError::SuccessorIsOwner);

    let is_recovery = revoker == ctx.accounts.identity.recovery;
    let is_admin = revoker == ctx.accounts.registry.admin;
    require!(is_recovery || is_admin, TerraError::NotAuthorized);

    let now = Clock::get()?.unix_timestamp;
    let identity = &mut ctx.accounts.identity;
    identity.owner = new_owner;
    identity.recovery = Pubkey::default();
    identity.updated_at = now;

    emit!(GuardianshipRevoked {
        identity: identity.key(),
        previous_guardian: previous,
        new_owner,
        revoked_by: revoker,
        block_time: now,
    });
    Ok(())
}

// ---------------------------------------------------------------------------
// Events
// ---------------------------------------------------------------------------

#[event]
pub struct CourtGuardianshipRequested {
    pub identity: Pubkey,
    pub successor: Pubkey,
    pub grace_secs: i64,
    pub required: u8,
    pub count: u8,
    pub effective_at: i64,
    pub case_hash: [u8; 32],
    pub scope_notes: String,
}

#[event]
pub struct GuardianshipRevoked {
    pub identity: Pubkey,
    pub previous_guardian: Pubkey,
    pub new_owner: Pubkey,
    pub revoked_by: Pubkey,
    pub block_time: i64,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn guardianship_grace_floor_is_90_days() {
        assert_eq!(MIN_GUARDIANSHIP_GRACE_SECS, 90 * 24 * 3600);
    }

    #[test]
    fn guardianship_default_grace_is_180_days() {
        assert_eq!(DEFAULT_GUARDIANSHIP_GRACE_SECS, 180 * 24 * 3600);
    }

    #[test]
    fn guardianship_min_validations_is_3() {
        assert_eq!(MIN_GUARDIANSHIP_VALIDATIONS, 3);
    }

    #[test]
    fn guardianship_kind_detection() {
        assert!(is_guardianship_kind(3));
        assert!(is_guardianship_kind(4));
        assert!(!is_guardianship_kind(0));
        assert!(!is_guardianship_kind(1));
        assert!(!is_guardianship_kind(2));
    }

    #[test]
    fn guardianship_default_grace_on_zero() {
        assert_eq!(
            normalize_guardianship_grace(0).unwrap(),
            DEFAULT_GUARDIANSHIP_GRACE_SECS
        );
    }

    #[test]
    fn guardianship_short_grace_rejected() {
        assert!(normalize_guardianship_grace(30 * 24 * 3600).is_err());
        assert!(normalize_guardianship_grace(MIN_GUARDIANSHIP_GRACE_SECS).is_ok());
    }

    #[test]
    fn guardianship_threshold_floor() {
        assert!(validate_guardianship_threshold(3, 3).is_ok());
        assert!(validate_guardianship_threshold(2, 3).is_err());
        assert!(validate_guardianship_threshold(4, 3).is_err());
    }
}
