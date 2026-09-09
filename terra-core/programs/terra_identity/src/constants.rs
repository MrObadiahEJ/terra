// ---------------------------------------------------------------------------
// Succession kind constants
// ---------------------------------------------------------------------------

pub mod succession_kind {
    pub const SUCCESSOR: u8 = 0;
    pub const RECOVERY: u8 = 1;
    pub const TRANSFER: u8 = 2;
    pub const GUARDIANSHIP: u8 = 3;
    pub const COURT_APPOINTED_GUARDIAN: u8 = 4;
    pub const MAX: u8 = COURT_APPOINTED_GUARDIAN;
}

// ---------------------------------------------------------------------------
// Grace period constants
// ---------------------------------------------------------------------------

pub const MIN_SUCCESSION_GRACE_SECS: i64 = 7 * 24 * 3600;
pub const MAX_SUCCESSION_GRACE_SECS: i64 = 180 * 24 * 3600;
pub const DEFAULT_SUCCESSION_GRACE_SECS: i64 = 30 * 24 * 3600;
pub const MIN_SUCCESSION_VALIDATIONS: u8 = 1;

// ---------------------------------------------------------------------------
// Guardianship constants (RFC-010)
// ---------------------------------------------------------------------------

pub const MIN_GUARDIANSHIP_GRACE_SECS: i64 = 90 * 24 * 3600;
pub const DEFAULT_GUARDIANSHIP_GRACE_SECS: i64 = 180 * 24 * 3600;
pub const MIN_GUARDIANSHIP_VALIDATIONS: u8 = 3;
pub const GUARDIANSHIP_REVOKE_TIMELOCK_SECS: i64 = 48 * 3600;
pub const MAX_VALIDATORS: usize = 8;

/// Maximum scope-notes length for court guardianship.
pub const MAX_SCOPE_NOTES_LEN: usize = 128;
