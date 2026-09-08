use anchor_lang::prelude::*;

/// Succession kind constants shared across the Terra identity layer.
///
/// These constants define the types of wallet passation supported by the
/// protocol. They are used by both the identity program and the registry
/// program to ensure consistent interpretation of the `kind` field on
/// Succession accounts.
pub mod succession_kind {
    /// Simple successor (e.g., named heir).
    pub const SUCCESSOR: u8 = 0;
    /// Recovery passation (recovery wallet reclaims control).
    pub const RECOVERY: u8 = 1;
    /// Deliberate transfer to a new wallet.
    pub const TRANSFER: u8 = 2;
    /// Guardian appointment (RFC-010): requires ≥3 validators, ≥90-day grace.
    pub const GUARDIANSHIP: u8 = 3;
    /// Court-appointed guardian (RFC-010): requires case_hash, ≥3 validators, ≥90-day grace.
    pub const COURT_APPOINTED_GUARDIAN: u8 = 4;
    /// Highest valid kind value.
    pub const MAX: u8 = COURT_APPOINTED_GUARDIAN;
}

// ---------------------------------------------------------------------------
// Grace period constants
// ---------------------------------------------------------------------------

/// Minimum grace period for any succession claim: 7 days.
pub const MIN_SUCCESSION_GRACE_SECS: i64 = 7 * 24 * 3600;
/// Maximum grace period for any succession claim: 180 days.
pub const MAX_SUCCESSION_GRACE_SECS: i64 = 180 * 24 * 3600;
/// Default grace when a requester passes 0: 30 days.
pub const DEFAULT_SUCCESSION_GRACE_SECS: i64 = 30 * 24 * 3600;

/// Minimum validator endorsements required for any succession: 1.
pub const MIN_SUCCESSION_VALIDATIONS: u8 = 1;

// ---------------------------------------------------------------------------
// Guardianship constants (RFC-010)
// ---------------------------------------------------------------------------

/// Minimum grace period for any guardianship claim: 90 days.
pub const MIN_GUARDIANSHIP_GRACE_SECS: i64 = 90 * 24 * 3600;
/// Default grace when a requester passes 0 for guardianship: 180 days.
pub const DEFAULT_GUARDIANSHIP_GRACE_SECS: i64 = 180 * 24 * 3600;
/// Minimum validator endorsements for guardianship: 3.
pub const MIN_GUARDIANSHIP_VALIDATIONS: u8 = 3;
/// Timelock before a recovery wallet's revocation request can be executed.
/// 48 hours — enough for the owner to react, short enough for emergency use.
pub const GUARDIANSHIP_REVOKE_TIMELOCK_SECS: i64 = 48 * 3600;

/// Maximum number of validators in an identity passation.
pub const MAX_VALIDATORS: usize = 8;

// ---------------------------------------------------------------------------
// Account types
// ---------------------------------------------------------------------------

/// Binds a person (via a hashed identity credential) to a wallet the person
/// actually holds, plus a recovery wallet. This is the resolvable on-chain link
/// behind "who owns this."
///
/// PDA: `["identity", identity_hash]`.
#[derive(anchor_lang::AnchorSerialize, anchor_lang::AnchorDeserialize, Clone, anchor_lang::InitSpace)]
pub struct Identity {
    /// 32-byte hash over the person's identity credential (e.g. national ID),
    /// so the credential itself never lives on-chain.
    pub identity_hash: [u8; 32],
    /// The active wallet acting on behalf of this identity.
    pub owner: Pubkey,
    /// A separate wallet the person also controls (backup / recovery).
    pub recovery: Pubkey,
    /// Number of parcels currently owned by this identity.
    pub parcel_count: u16,
    pub created_at: i64,
    pub updated_at: i64,
    /// When true, a recovery wallet has requested revocation but the timelock
    /// has not yet expired.
    pub pending_revocation: bool,
    /// The approved target wallet for a pending revocation.
    pub pending_new_owner: Pubkey,
    /// Unix timestamp after which a pending revocation may be executed.
    pub revoke_after: i64,
}

/// An in-flight passation of wallet control, gated by BOTH a configurable grace
/// period AND a minimum number of validator endorsements.
///
/// PDA: `["succession", identity, successor]`.
#[derive(anchor_lang::AnchorSerialize, anchor_lang::AnchorDeserialize, Clone, anchor_lang::InitSpace)]
pub struct Succession {
    /// The Identity whose control is being passed.
    pub identity: Pubkey,
    /// The wallet that will take over once gated.
    pub successor: Pubkey,
    /// succession_kind.
    pub kind: u8,
    pub requested_at: i64,
    /// effective = requested_at + grace_secs.
    pub effective_at: i64,
    /// Configurable per-request grace (0 => DEFAULT_SUCCESSION_GRACE_SECS).
    pub grace_secs: i64,
    /// Number of validator endorsements required before claim.
    pub required: u8,
    /// Number of endorsements collected so far.
    pub validations_count: u8,
    /// Declared local-authority validator set acting as testifiers.
    pub validators: [Pubkey; MAX_VALIDATORS],
}

// ---------------------------------------------------------------------------
// Events
// ---------------------------------------------------------------------------

#[derive(anchor_lang::AnchorSerialize, anchor_lang::AnchorDeserialize, Clone)]
pub struct IdentityBound {
    pub identity: Pubkey,
    pub identity_hash: [u8; 32],
    pub owner: Pubkey,
    pub recovery: Pubkey,
}

#[derive(anchor_lang::AnchorSerialize, anchor_lang::AnchorDeserialize, Clone)]
pub struct ParcelAttached {
    pub identity: Pubkey,
    pub parcel: Pubkey,
    pub owner: Pubkey,
}

#[derive(anchor_lang::AnchorSerialize, anchor_lang::AnchorDeserialize, Clone)]
pub struct SuccessionRequested {
    pub identity: Pubkey,
    pub successor: Pubkey,
    pub kind: u8,
    pub grace_secs: i64,
    pub required: u8,
    pub count: u8,
    pub effective_at: i64,
}

#[derive(anchor_lang::AnchorSerialize, anchor_lang::AnchorDeserialize, Clone)]
pub struct SuccessionEndorsed {
    pub identity: Pubkey,
    pub successor: Pubkey,
    pub validator: Pubkey,
    pub validations_count: u8,
    pub required: u8,
}

#[derive(anchor_lang::AnchorSerialize, anchor_lang::AnchorDeserialize, Clone)]
pub struct SuccessionCancelled {
    pub identity: Pubkey,
    pub successor: Pubkey,
    pub kind: u8,
}

#[derive(anchor_lang::AnchorSerialize, anchor_lang::AnchorDeserialize, Clone)]
pub struct SuccessionClaimed {
    pub identity: Pubkey,
    pub from: Pubkey,
    pub to: Pubkey,
    pub kind: u8,
    pub parcels_repointed: u8,
}

#[derive(anchor_lang::AnchorSerialize, anchor_lang::AnchorDeserialize, Clone)]
pub struct GuardianshipRevoked {
    pub identity: Pubkey,
    pub from: Pubkey,
    pub to: Pubkey,
}

// ---------------------------------------------------------------------------
// Helper functions
// ---------------------------------------------------------------------------

/// Returns true for the two guardianship succession kinds (3, 4).
pub fn is_guardianship_kind(kind: u8) -> bool {
    kind == succession_kind::GUARDIANSHIP || kind == succession_kind::COURT_APPOINTED_GUARDIAN
}

/// Normalize a requested grace period for a guardianship kind.
/// 0 => default 180d; otherwise clamped to [MIN_GUARDIANSHIP_GRACE_SECS, MAX_SUCCESSION_GRACE_SECS].
pub fn normalize_guardianship_grace(grace_secs: i64) -> i64 {
    if grace_secs == 0 {
        DEFAULT_GUARDIANSHIP_GRACE_SECS
    } else {
        grace_secs.clamp(MIN_GUARDIANSHIP_GRACE_SECS, MAX_SUCCESSION_GRACE_SECS)
    }
}

/// Returns true if the endorsement threshold meets the guardianship minimum.
pub fn validate_guardianship_threshold(required_validations: u8, declared: usize) -> bool {
    (required_validations as usize) >= MIN_GUARDIANSHIP_VALIDATIONS as usize
        && (required_validations as usize) <= declared
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn succession_kind_reserved_variants_are_contiguous() {
        assert_eq!(succession_kind::SUCCESSOR, 0);
        assert_eq!(succession_kind::RECOVERY, 1);
        assert_eq!(succession_kind::TRANSFER, 2);
        assert_eq!(succession_kind::GUARDIANSHIP, 3);
        assert_eq!(succession_kind::COURT_APPOINTED_GUARDIAN, 4);
        assert_eq!(
            succession_kind::MAX,
            succession_kind::COURT_APPOINTED_GUARDIAN
        );
    }

    #[test]
    fn is_guardianship_kind_works() {
        assert!(!is_guardianship_kind(succession_kind::SUCCESSOR));
        assert!(!is_guardianship_kind(succession_kind::RECOVERY));
        assert!(!is_guardianship_kind(succession_kind::TRANSFER));
        assert!(is_guardianship_kind(succession_kind::GUARDIANSHIP));
        assert!(is_guardianship_kind(succession_kind::COURT_APPOINTED_GUARDIAN));
    }

    #[test]
    fn normalize_guardianship_grace_defaults_to_180d() {
        assert_eq!(normalize_guardianship_grace(0), DEFAULT_GUARDIANSHIP_GRACE_SECS);
    }

    #[test]
    fn normalize_guardianship_grace_clamps() {
        // 120 days is within [90d, 180d] — passes through unchanged
        assert_eq!(
            normalize_guardianship_grace(120 * 24 * 3600),
            120 * 24 * 3600
        );
        // 50 days is below 90-day minimum — clamped up
        assert_eq!(
            normalize_guardianship_grace(50 * 24 * 3600),
            MIN_GUARDIANSHIP_GRACE_SECS
        );
        // 200 days exceeds 180-day maximum — clamped down
        assert_eq!(
            normalize_guardianship_grace(200 * 24 * 3600),
            MAX_SUCCESSION_GRACE_SECS
        );
    }

    #[test]
    fn validate_guardianship_threshold_works() {
        assert!(validate_guardianship_threshold(3, 3));
        assert!(validate_guardianship_threshold(3, 5));
        assert!(!validate_guardianship_threshold(2, 3));
        assert!(!validate_guardianship_threshold(4, 3));
    }
}
