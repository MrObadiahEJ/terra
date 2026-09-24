use anchor_lang::prelude::*;

use crate::TerraError;

// ---------------------------------------------------------------------------
// RFC-012 Phase 2 — generalized validator PDAs
//
// Migration map (RFC-012 §9): `ValidatorRegistry.validators` stays as the
// index; attributes move onto per-wallet PDAs. Existing ValidatorReputation
// continues as the legacy score (`reputation_score` is the continuity field).
// ---------------------------------------------------------------------------

/// Maximum characters stored in free-text profile fields.
pub const MAX_PROFILE_NOTE_LEN: usize = 128;
/// Presence expires after this many seconds if not refreshed (24h).
pub const PRESENCE_TTL_SECS: i64 = 24 * 3600;
/// Minimum presence confidence to accept an update (basis points).
pub const MIN_PRESENCE_CONFIDENCE_BPS: u16 = 100;

pub mod profile_tier {
    /// Permissionless start — low confidence, not yet endorsed.
    pub const NEW: u8 = 0;
    /// Undergoing nomination / early activity.
    pub const PROBATIONARY: u8 = 1;
    /// Sustained good performance.
    pub const ESTABLISHED: u8 = 2;
    /// High multi-dimensional trust.
    pub const TRUSTED: u8 = 3;
    /// Peak capability band (survey, legal, high-value).
    pub const HIGH_CAPABILITY: u8 = 4;
    pub const MAX: u8 = HIGH_CAPABILITY;
}

pub mod availability_status {
    pub const ONLINE: u8 = 0;
    pub const AVAILABLE: u8 = 1;
    pub const BUSY: u8 = 2;
    pub const OFFLINE: u8 = 3;
    pub const TEMPORARILY_UNAVAILABLE: u8 = 4;
    /// Protocol suspension (admin/governance) — not the same as jail.
    pub const SUSPENDED: u8 = 5;
    pub const MAX: u8 = SUSPENDED;
}

pub mod capability_level {
    /// Self-declared only; no independent confirmation.
    pub const DECLARED: u8 = 0;
    /// Seen in observations / workflows.
    pub const OBSERVED: u8 = 1;
    /// Verified by registry admin or attestation path.
    pub const VERIFIED: u8 = 2;
    /// Highest trust band for this capability code.
    pub const TRUSTED: u8 = 3;
    pub const MAX: u8 = TRUSTED;
}

/// Initial capability taxonomy (RFC-012 §12 open question — provisional codes).
pub mod capability_code {
    pub const GNSS: u8 = 0;
    pub const IMAGERY: u8 = 1;
    pub const DOCUMENT: u8 = 2;
    pub const SURVEY: u8 = 3;
    pub const LEGAL: u8 = 4;
    pub const PHYSICAL: u8 = 5;
    pub const REMOTE: u8 = 6;
    pub const MAX: u8 = REMOTE;
}

pub mod presence_provenance {
    pub const SELF_REPORTED: u8 = 0;
    pub const DEVICE_GNSS: u8 = 1;
    pub const MULTI_DEVICE: u8 = 2;
    pub const LOCAL_VALIDATORS: u8 = 3;
    pub const SURVEY_GRADE: u8 = 4;
    pub const SATELLITE_CONFIRMED: u8 = 5;
    pub const MAX: u8 = SATELLITE_CONFIRMED;
}

pub mod relationship_edge_type {
    /// Peer endorsement of competence / identity.
    pub const ENDORSEMENT: u8 = 0;
    /// Co-validation history (shared tasks).
    pub const CO_VALIDATION: u8 = 1;
    /// Shared infrastructure (nodes, storage, devices).
    pub const SHARED_INFRASTRUCTURE: u8 = 2;
    pub const MAX: u8 = SHARED_INFRASTRUCTURE;
}

// ---------------------------------------------------------------------------
// Accounts (PDA seeds per RFC-012 §10)
// ---------------------------------------------------------------------------

/// Core validator profile — identity + tier attributes (not location-bound).
///
/// PDA: `["validator_profile", wallet]`
#[account]
#[derive(InitSpace)]
pub struct ValidatorProfile {
    /// The validator's wallet (mobile participant — NOT a region).
    pub wallet: Pubkey,
    /// Trust tier (`profile_tier`). Starts NEW; admin/governance promotes.
    pub tier: u8,
    /// Optional link to an off-chain/on-chain identity hash (0 = unset).
    pub identity_hash: [u8; 32],
    /// Free-form operator note (device class, org, …) — not used for selection.
    #[max_len(128)]
    pub note: String,
    pub created_at: i64,
    pub updated_at: i64,
}

/// Dynamic geographic presence — timestamped, confidence-scored, expiring.
///
/// Never treat as immutable identity. PDA: `["validator_presence", wallet]`
#[account]
#[derive(InitSpace)]
pub struct ValidatorPresence {
    pub wallet: Pubkey,
    /// Latitude in degrees * 1e7 (i32) — compact fixed-point.
    pub latitude_e7: i32,
    /// Longitude in degrees * 1e7.
    pub longitude_e7: i32,
    /// Horizontal accuracy in meters.
    pub accuracy_m: u16,
    /// How the fix was obtained (`presence_provenance`).
    pub provenance: u8,
    /// Confidence in this presence fix (0–10000 bps).
    pub confidence_bps: u16,
    /// When this fix was recorded.
    pub observed_at: i64,
    /// After this time the presence is stale (observed_at + TTL).
    pub expires_at: i64,
    pub updated_at: i64,
}

/// Current availability state (distinct from reputation jail).
///
/// PDA: `["validator_availability", wallet]`
#[account]
#[derive(InitSpace)]
pub struct ValidatorAvailability {
    pub wallet: Pubkey,
    /// One of `availability_status`.
    pub status: u8,
    pub updated_at: i64,
}

/// Declared/verified capability for one capability code.
///
/// PDA: `["validator_capability", wallet, capability_code]`
#[account]
#[derive(InitSpace)]
pub struct ValidatorCapability {
    pub wallet: Pubkey,
    /// Capability code (`capability_code`).
    pub capability_code: u8,
    /// Confidence level (`capability_level`).
    pub level: u8,
    /// Optional evidence hash backing a VERIFIED/TRUSTED claim (0 = none).
    pub evidence_hash: [u8; 32],
    pub declared_at: i64,
    /// 0 until verified; set when level reaches VERIFIED or above.
    pub verified_at: i64,
    pub updated_at: i64,
}

/// Directed relationship graph edge (endorsement / co-validation / infra).
///
/// Correlation ≠ fraud — used for review, not auto-punishment (RFC-012 §6.12).
/// PDA: `["validator_edge", from, to, edge_type]`
#[account]
#[derive(InitSpace)]
pub struct ValidatorRelationshipEdge {
    pub from: Pubkey,
    pub to: Pubkey,
    /// One of `relationship_edge_type`.
    pub edge_type: u8,
    /// Optional strength (0–10000 bps); default 10000 for binary endorsement.
    pub weight_bps: u16,
    pub created_at: i64,
    pub updated_at: i64,
}

// ---------------------------------------------------------------------------
// Pure helpers (unit-testable)
// ---------------------------------------------------------------------------

/// True when `code` is a known capability code.
pub fn is_valid_capability_code(code: u8) -> bool {
    code <= capability_code::MAX
}

/// True when `level` is a known capability level.
pub fn is_valid_capability_level(level: u8) -> bool {
    level <= capability_level::MAX
}

/// True when `status` is a known availability status.
pub fn is_valid_availability(status: u8) -> bool {
    status <= availability_status::MAX
}

/// True when `prov` is a known presence provenance.
pub fn is_valid_provenance(prov: u8) -> bool {
    prov <= presence_provenance::MAX
}

/// True when `edge` is a known relationship edge type.
pub fn is_valid_edge_type(edge: u8) -> bool {
    edge <= relationship_edge_type::MAX
}

/// True when `tier` is a known profile tier.
pub fn is_valid_tier(tier: u8) -> bool {
    tier <= profile_tier::MAX
}

/// Self-declared capabilities may only claim DECLARED.
/// OBSERVED+ requires admin verification (or a future observation path).
pub fn self_declare_max_level() -> u8 {
    capability_level::DECLARED
}

/// Presence is stale after `expires_at`.
pub fn presence_is_stale(now: i64, expires_at: i64) -> bool {
    now >= expires_at
}

// ---------------------------------------------------------------------------
// Instruction handlers
// ---------------------------------------------------------------------------

/// Create a validator profile. The wallet must sign (self-onboarding);
/// permissionless start at tier NEW (RFC-012 §6.2).
pub fn init_validator_profile(
    ctx: Context<crate::InitValidatorProfile>,
    identity_hash: [u8; 32],
    note: String,
) -> Result<()> {
    require!(
        note.len() <= MAX_PROFILE_NOTE_LEN,
        TerraError::StringTooLong
    );
    let now = Clock::get()?.unix_timestamp;
    let p = &mut ctx.accounts.profile;
    p.wallet = ctx.accounts.wallet.key();
    p.tier = profile_tier::NEW;
    p.identity_hash = identity_hash;
    p.note = note;
    p.created_at = now;
    p.updated_at = now;

    emit!(crate::ValidatorProfileInitialized {
        wallet: p.wallet,
        tier: p.tier,
        initialized_at: now,
    });
    Ok(())
}

/// Update profile note / identity hash. Tier changes are admin-only via
/// a separate path so a wallet cannot self-promote to TRUSTED.
pub fn update_validator_profile(
    ctx: Context<crate::UpdateValidatorProfile>,
    identity_hash: [u8; 32],
    note: String,
) -> Result<()> {
    require!(
        note.len() <= MAX_PROFILE_NOTE_LEN,
        TerraError::StringTooLong
    );
    let now = Clock::get()?.unix_timestamp;
    let p = &mut ctx.accounts.profile;
    p.identity_hash = identity_hash;
    p.note = note;
    p.updated_at = now;

    emit!(crate::ValidatorProfileUpdated {
        wallet: p.wallet,
        tier: p.tier,
        updated_at: now,
    });
    Ok(())
}

/// Admin-only tier promotion/demotion (governance path; no self-promote).
pub fn set_validator_profile_tier(
    ctx: Context<crate::SetValidatorProfileTier>,
    tier: u8,
) -> Result<()> {
    require!(is_valid_tier(tier), TerraError::InvalidProfileTier);
    let now = Clock::get()?.unix_timestamp;
    let p = &mut ctx.accounts.profile;
    let old = p.tier;
    p.tier = tier;
    p.updated_at = now;

    emit!(crate::ValidatorProfileTierChanged {
        wallet: p.wallet,
        old_tier: old,
        new_tier: tier,
        updated_at: now,
    });
    Ok(())
}

/// Record / refresh geographic presence. Only the profile wallet may publish.
/// Stale presence (`expires_at` in the past) is simply overwritten on update.
pub fn set_validator_presence(
    ctx: Context<crate::SetValidatorPresence>,
    latitude_e7: i32,
    longitude_e7: i32,
    accuracy_m: u16,
    provenance: u8,
    confidence_bps: u16,
) -> Result<()> {
    require!(
        is_valid_provenance(provenance),
        TerraError::InvalidPresenceProvenance
    );
    require!(
        (MIN_PRESENCE_CONFIDENCE_BPS..=10_000).contains(&confidence_bps),
        TerraError::InvalidConfidence
    );
    // Basic geographic sanity: |lat| <= 90e7, |lon| <= 180e7.
    require!(
        latitude_e7.abs() <= 900_000_000 && longitude_e7.abs() <= 1_800_000_000,
        TerraError::InvalidPresenceFix
    );

    let now = Clock::get()?.unix_timestamp;
    let pr = &mut ctx.accounts.presence;
    pr.wallet = ctx.accounts.profile.wallet;
    pr.latitude_e7 = latitude_e7;
    pr.longitude_e7 = longitude_e7;
    pr.accuracy_m = accuracy_m;
    pr.provenance = provenance;
    pr.confidence_bps = confidence_bps;
    pr.observed_at = now;
    pr.expires_at = now.saturating_add(PRESENCE_TTL_SECS);
    pr.updated_at = now;

    emit!(crate::ValidatorPresenceUpdated {
        wallet: pr.wallet,
        provenance,
        confidence_bps,
        expires_at: pr.expires_at,
    });
    Ok(())
}

/// Update availability for the profile wallet. SUSPENDED is reserved for
/// admin (`suspend_validator_availability`).
pub fn set_validator_availability(
    ctx: Context<crate::SetValidatorAvailability>,
    status: u8,
) -> Result<()> {
    require!(
        is_valid_availability(status),
        TerraError::InvalidAvailabilityStatus
    );
    require!(
        status != availability_status::SUSPENDED,
        TerraError::NotAuthorized
    );
    require!(
        ctx.accounts.wallet.key() == ctx.accounts.profile.wallet,
        TerraError::NotAuthorized
    );

    let now = Clock::get()?.unix_timestamp;
    let av = &mut ctx.accounts.availability;
    av.wallet = ctx.accounts.profile.wallet;
    av.status = status;
    av.updated_at = now;

    emit!(crate::ValidatorAvailabilityChanged {
        wallet: av.wallet,
        status,
        updated_at: now,
    });
    Ok(())
}

/// Admin-only: force availability to SUSPENDED (governance action).
pub fn suspend_validator_availability(
    ctx: Context<crate::SuspendValidatorAvailability>,
) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let av = &mut ctx.accounts.availability;
    av.wallet = ctx.accounts.profile.wallet;
    av.status = availability_status::SUSPENDED;
    av.updated_at = now;

    emit!(crate::ValidatorAvailabilityChanged {
        wallet: av.wallet,
        status: availability_status::SUSPENDED,
        updated_at: now,
    });
    Ok(())
}

/// Declare a capability. Self may only claim DECLARED; VERIFIED/TRUSTED
/// requires registry admin (`AdminDeclareValidatorCapability`).
pub fn declare_validator_capability(
    ctx: Context<crate::DeclareValidatorCapability>,
    capability_code: u8,
    level: u8,
    evidence_hash: [u8; 32],
) -> Result<()> {
    require!(
        is_valid_capability_code(capability_code),
        TerraError::InvalidCapabilityCode
    );
    require!(
        is_valid_capability_level(level),
        TerraError::InvalidCapabilityLevel
    );
    require!(level <= self_declare_max_level(), TerraError::NotAuthorized);
    require!(
        ctx.accounts.wallet.key() == ctx.accounts.profile.wallet,
        TerraError::NotAuthorized
    );

    let now = Clock::get()?.unix_timestamp;
    let cap = &mut ctx.accounts.capability;
    cap.wallet = ctx.accounts.profile.wallet;
    cap.capability_code = capability_code;
    cap.level = level;
    cap.evidence_hash = evidence_hash;
    if cap.declared_at == 0 {
        cap.declared_at = now;
    }
    cap.verified_at = 0;
    cap.updated_at = now;

    emit!(crate::ValidatorCapabilityDeclared {
        wallet: cap.wallet,
        capability_code,
        level,
        verified_at: cap.verified_at,
    });
    Ok(())
}

/// Admin-only: set capability level to VERIFIED or TRUSTED with evidence.
pub fn admin_verify_validator_capability(
    ctx: Context<crate::AdminVerifyValidatorCapability>,
    capability_code: u8,
    level: u8,
    evidence_hash: [u8; 32],
) -> Result<()> {
    require!(
        is_valid_capability_code(capability_code),
        TerraError::InvalidCapabilityCode
    );
    require!(
        is_valid_capability_level(level),
        TerraError::InvalidCapabilityLevel
    );
    require!(
        level >= capability_level::VERIFIED,
        TerraError::InvalidCapabilityLevel
    );
    require!(
        ctx.accounts.capability.capability_code == capability_code,
        TerraError::InvalidCapabilityCode
    );

    let now = Clock::get()?.unix_timestamp;
    let cap = &mut ctx.accounts.capability;
    cap.level = level;
    cap.evidence_hash = evidence_hash;
    if cap.declared_at == 0 {
        cap.declared_at = now;
    }
    cap.verified_at = now;
    cap.updated_at = now;

    emit!(crate::ValidatorCapabilityDeclared {
        wallet: cap.wallet,
        capability_code,
        level,
        verified_at: cap.verified_at,
    });
    Ok(())
}

/// Create a directed relationship edge. `from` must sign; self-edges rejected.
pub fn create_validator_relationship_edge(
    ctx: Context<crate::CreateValidatorRelationshipEdge>,
    edge_type: u8,
    weight_bps: u16,
) -> Result<()> {
    require!(is_valid_edge_type(edge_type), TerraError::InvalidEdgeType);
    require!(weight_bps <= 10_000, TerraError::InvalidConfidence);

    let from = ctx.accounts.from_profile.wallet;
    let to = ctx.accounts.to_profile.wallet;
    require!(from != to, TerraError::SelfRelationshipEdge);

    let now = Clock::get()?.unix_timestamp;
    let edge = &mut ctx.accounts.edge;
    edge.from = from;
    edge.to = to;
    edge.edge_type = edge_type;
    edge.weight_bps = weight_bps;
    if edge.created_at == 0 {
        edge.created_at = now;
    }
    edge.updated_at = now;

    emit!(crate::ValidatorRelationshipEdgeCreated {
        from,
        to,
        edge_type,
        weight_bps,
    });
    Ok(())
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_tier_constants_are_contiguous() {
        assert_eq!(profile_tier::NEW, 0);
        assert_eq!(profile_tier::PROBATIONARY, 1);
        assert_eq!(profile_tier::ESTABLISHED, 2);
        assert_eq!(profile_tier::TRUSTED, 3);
        assert_eq!(profile_tier::HIGH_CAPABILITY, 4);
        assert_eq!(profile_tier::MAX, 4);
        for t in 0..=profile_tier::MAX {
            assert!(is_valid_tier(t));
        }
        assert!(!is_valid_tier(profile_tier::MAX + 1));
    }

    #[test]
    fn availability_status_constants_are_contiguous() {
        assert_eq!(availability_status::ONLINE, 0);
        assert_eq!(availability_status::AVAILABLE, 1);
        assert_eq!(availability_status::BUSY, 2);
        assert_eq!(availability_status::OFFLINE, 3);
        assert_eq!(availability_status::TEMPORARILY_UNAVAILABLE, 4);
        assert_eq!(availability_status::SUSPENDED, 5);
        for s in 0..=availability_status::MAX {
            assert!(is_valid_availability(s));
        }
        assert!(!is_valid_availability(6));
    }

    #[test]
    fn capability_level_and_code_bounds() {
        for l in 0..=capability_level::MAX {
            assert!(is_valid_capability_level(l));
        }
        assert!(!is_valid_capability_level(capability_level::MAX + 1));
        for c in 0..=capability_code::MAX {
            assert!(is_valid_capability_code(c));
        }
        assert!(!is_valid_capability_code(capability_code::MAX + 1));
        assert_eq!(self_declare_max_level(), capability_level::DECLARED);
    }

    #[test]
    fn presence_provenance_and_staleness() {
        for p in 0..=presence_provenance::MAX {
            assert!(is_valid_provenance(p));
        }
        assert!(!is_valid_provenance(presence_provenance::MAX + 1));
        assert!(!presence_is_stale(100, 200));
        assert!(presence_is_stale(200, 200));
        assert!(presence_is_stale(201, 200));
    }

    #[test]
    fn edge_types_cover_rfc_relationship_kinds() {
        assert_eq!(relationship_edge_type::ENDORSEMENT, 0);
        assert_eq!(relationship_edge_type::CO_VALIDATION, 1);
        assert_eq!(relationship_edge_type::SHARED_INFRASTRUCTURE, 2);
        for e in 0..=relationship_edge_type::MAX {
            assert!(is_valid_edge_type(e));
        }
        assert!(!is_valid_edge_type(3));
    }

    #[test]
    fn note_length_guard_matches_const() {
        let ok = "x".repeat(MAX_PROFILE_NOTE_LEN);
        let bad = "x".repeat(MAX_PROFILE_NOTE_LEN + 1);
        assert!(ok.as_bytes().len() <= MAX_PROFILE_NOTE_LEN);
        assert!(bad.as_bytes().len() > MAX_PROFILE_NOTE_LEN);
    }
}
