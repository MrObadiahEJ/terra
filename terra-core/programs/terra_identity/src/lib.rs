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
