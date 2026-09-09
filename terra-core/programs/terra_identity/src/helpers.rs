use super::constants::*;

/// Returns true for the two guardianship succession kinds (3, 4).
pub fn is_guardianship_kind(kind: u8) -> bool {
    kind == succession_kind::GUARDIANSHIP || kind == succession_kind::COURT_APPOINTED_GUARDIAN
}

/// Normalize a requested grace period for a guardianship kind.
/// 0 => default 180d; otherwise must be >= 90d, clamped to the global max.
pub fn normalize_guardianship_grace(grace_secs: i64) -> i64 {
    if grace_secs == 0 {
        DEFAULT_GUARDIANSHIP_GRACE_SECS
    } else {
        grace_secs.clamp(MIN_GUARDIANSHIP_GRACE_SECS, MAX_SUCCESSION_GRACE_SECS)
    }
}

/// Validate the endorsement threshold for a guardianship request.
pub fn validate_guardianship_threshold(required_validations: u8, declared: usize) -> bool {
    (required_validations as usize) >= MIN_GUARDIANSHIP_VALIDATIONS as usize
        && (required_validations as usize) <= declared
}
