use anchor_lang::prelude::*;
use solana_program::hash::hash as sha256_hash;

use crate::TerraError;

// ---------------------------------------------------------------------------
// Jurisdiction status
// ---------------------------------------------------------------------------

pub mod jurisdiction_status {
    pub const ACTIVE: u8 = 0;
    pub const SUSPENDED: u8 = 1;
    pub const WITHDRAWN: u8 = 2;
}

// ---------------------------------------------------------------------------
// Algorithm IDs
// ---------------------------------------------------------------------------

pub mod algorithm_id {
    pub const GROTH16: u8 = 0;
    pub const FRI_STARK: u8 = 1;
}

/// Maximum proof size in bytes.
pub const MAX_PROOF_LEN: usize = 512;
/// Maximum jurisdiction name length.
pub const MAX_JURISDICTION_NAME_LEN: usize = 64;
/// Maximum credential schema CID length.
pub const MAX_SCHEMA_CID_LEN: usize = 128;
/// Maximum revocation reason length.
pub const MAX_REASON_LEN: usize = 128;

// ---------------------------------------------------------------------------
// Accounts
// ---------------------------------------------------------------------------

/// A registered jurisdiction (country/zone) in the cross-border identity system.
///
/// PDA seed: `["jurisdiction", country_code]`.
#[account]
#[derive(InitSpace)]
pub struct Jurisdiction {
    /// ISO 3166-1 alpha-2 padded to 16 bytes (e.g. b"KE\x00..." for Kenya).
    pub country_code: [u8; 16],
    /// The jurisdiction authority wallet (issues/revokes credentials).
    pub authority: Pubkey,
    /// Human-readable name (max 64 chars).
    #[max_len(64)]
    pub jurisdiction_name: String,
    /// IPFS CID of the W3C Verifiable Credential schema.
    #[max_len(128)]
    pub credential_schema_cid: String,
    /// On-chain or oracle reference for revocation checks.
    pub revocation_registry: Pubkey,
    /// SHA-256 of the ZK verification key for this jurisdiction's circuit.
    pub verification_key_hash: [u8; 32],
    /// 0 = Groth16, 1 = FRI-STARK.
    pub algorithm_id: u8,
    /// 0 = Active, 1 = Suspended, 2 = Withdrawn.
    pub status: u8,
    pub created_at: i64,
    pub updated_at: i64,
}

/// Binds a person (via identity_hash) to a jurisdiction with a ZK proof.
///
/// PDA seed: `["cross_border_identity", jurisdiction_key, identity_hash]`.
#[account]
#[derive(InitSpace)]
pub struct JurisdictionBinding {
    /// The identity_hash from the existing Identity account.
    pub identity_hash: [u8; 32],
    /// The Jurisdiction PDA this binding belongs to.
    pub jurisdiction_key: Pubkey,
    /// Pedersen commitment to the credential.
    pub credential_commitment: [u8; 32],
    /// Derived nullifier to prevent double-binding and proof reuse.
    pub nullifier: [u8; 32],
    /// The serialized ZK proof (max 512 bytes).
    #[max_len(512)]
    pub proof_data: Vec<u8>,
    /// Version of the proof circuit (for future upgrades).
    pub proof_version: u8,
    /// 0 = Groth16, 1 = FRI-STARK.
    pub algorithm_id: u8,
    /// Whether this binding has been revoked.
    pub revoked: bool,
    /// Timestamp of revocation (0 if not revoked).
    pub revoked_at: i64,
    /// Who revoked this binding.
    pub revoked_by: Pubkey,
    /// When the binding was created.
    pub bound_at: i64,
    /// Optional expiry (0 = no expiry).
    pub expires_at: i64,
    /// Whether a validator has attested this binding's proof off-chain.
    /// Binding is a permissionless *claim*; only verified bindings should be
    /// relied upon (circuit verification is deferred to audit; see RFC-006).
    pub verified: bool,
    /// Validator that last verified this binding.
    pub verified_by: Pubkey,
    /// Monotonic counter, bumped on re-verification.
    pub version: u32,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// Register a new jurisdiction. Only the registry admin can register.
pub fn register_jurisdiction(
    ctx: Context<super::RegisterJurisdiction>,
    country_code: [u8; 16],
    jurisdiction_name: String,
    credential_schema_cid: String,
    revocation_registry: Pubkey,
    verification_key_hash: [u8; 32],
    algorithm_id: u8,
) -> Result<()> {
    crate::validator_registry::require_not_paused(&ctx.accounts.registry)?;
    require!(!country_code.iter().all(|b| *b == 0), TerraError::InvalidId);
    require!(
        !jurisdiction_name.is_empty() && jurisdiction_name.len() <= MAX_JURISDICTION_NAME_LEN,
        TerraError::NotesTooLong
    );
    require!(
        !credential_schema_cid.is_empty() && credential_schema_cid.len() <= MAX_SCHEMA_CID_LEN,
        TerraError::CidRequired
    );
    require!(
        !verification_key_hash.iter().all(|b| *b == 0),
        TerraError::EmptyGeometryHash
    );
    require!(
        algorithm_id == algorithm_id::GROTH16 || algorithm_id == algorithm_id::FRI_STARK,
        TerraError::AlgorithmNotSupported
    );

    // Only registry admin can register jurisdictions.
    let registry = &ctx.accounts.registry;
    require!(
        ctx.accounts.authority.key() == registry.admin,
        TerraError::NotAuthorized
    );

    let now = Clock::get()?.unix_timestamp;
    let jurisdiction = &mut ctx.accounts.jurisdiction;
    jurisdiction.country_code = country_code;
    jurisdiction.authority = ctx.accounts.authority.key();
    jurisdiction.jurisdiction_name = jurisdiction_name;
    jurisdiction.credential_schema_cid = credential_schema_cid;
    jurisdiction.revocation_registry = revocation_registry;
    jurisdiction.verification_key_hash = verification_key_hash;
    jurisdiction.algorithm_id = algorithm_id;
    jurisdiction.status = jurisdiction_status::ACTIVE;
    jurisdiction.created_at = now;
    jurisdiction.updated_at = now;

    emit!(JurisdictionRegistered {
        jurisdiction: jurisdiction.key(),
        country_code,
        authority: jurisdiction.authority,
        algorithm_id,
    });
    Ok(())
}

/// Update jurisdiction parameters (rotate verification key, change status).
pub fn update_jurisdiction(
    ctx: Context<super::UpdateJurisdiction>,
    new_verification_key_hash: Option<[u8; 32]>,
    new_revocation_registry: Option<Pubkey>,
    new_status: Option<u8>,
) -> Result<()> {
    let jurisdiction = &mut ctx.accounts.jurisdiction;
    require!(
        ctx.accounts.authority.key() == jurisdiction.authority,
        TerraError::NotAuthorized
    );

    let has_update = new_verification_key_hash.is_some()
        || new_revocation_registry.is_some()
        || new_status.is_some();
    require!(has_update, TerraError::InvalidStatus);

    if let Some(vk_hash) = new_verification_key_hash {
        require!(
            !vk_hash.iter().all(|b| *b == 0),
            TerraError::EmptyGeometryHash
        );
        jurisdiction.verification_key_hash = vk_hash;
    }
    if let Some(registry) = new_revocation_registry {
        jurisdiction.revocation_registry = registry;
    }
    if let Some(status) = new_status {
        require!(
            status <= jurisdiction_status::WITHDRAWN,
            TerraError::InvalidStatus
        );
        jurisdiction.status = status;
    }

    jurisdiction.updated_at = Clock::get()?.unix_timestamp;

    emit!(JurisdictionUpdated {
        jurisdiction: jurisdiction.key(),
        updated_by: ctx.accounts.authority.key(),
    });
    Ok(())
}

/// Bind an identity to a jurisdiction with a ZK proof.
pub fn bind_cross_border_identity(
    ctx: Context<super::BindCrossBorderIdentity>,
    credential_commitment: [u8; 32],
    proof_data: Vec<u8>,
    nullifier_nonce: [u8; 32],
    expires_at: i64,
    identity_hash: [u8; 32],
) -> Result<()> {
    require!(
        !proof_data.is_empty() && proof_data.len() <= MAX_PROOF_LEN,
        TerraError::InvalidProofData
    );
    require!(
        !credential_commitment.iter().all(|b| *b == 0),
        TerraError::EmptyGeometryHash
    );
    require!(
        !nullifier_nonce.iter().all(|b| *b == 0),
        TerraError::NonceRequired
    );

    let now = Clock::get()?.unix_timestamp;
    if expires_at != 0 {
        require!(expires_at > now, TerraError::InvalidExpiry);
    }

    let jurisdiction = &ctx.accounts.jurisdiction;
    require!(
        jurisdiction.status == jurisdiction_status::ACTIVE,
        TerraError::InvalidJurisdictionStatus
    );

    // Derive nullifier = SHA-256(credential_commitment || jurisdiction_key || nullifier_nonce).
    let jurisdiction_bytes = ctx.accounts.jurisdiction.key().to_bytes();
    let mut input = Vec::with_capacity(96);
    input.extend_from_slice(&credential_commitment);
    input.extend_from_slice(&jurisdiction_bytes);
    input.extend_from_slice(&nullifier_nonce);
    let nullifier: [u8; 32] = sha256_hash(&input).to_bytes();

    let binding = &mut ctx.accounts.binding;
    binding.identity_hash = identity_hash;
    binding.jurisdiction_key = jurisdiction.key();
    binding.credential_commitment = credential_commitment;
    binding.nullifier = nullifier;
    binding.proof_data = proof_data;
    binding.proof_version = 0;
    binding.algorithm_id = jurisdiction.algorithm_id;
    binding.revoked = false;
    binding.revoked_at = 0;
    binding.bound_at = now;
    binding.expires_at = expires_at;
    // Fresh claims are unverified until a validator attests via
    // verify_jurisdiction_membership.
    binding.verified = false;
    binding.verified_by = Pubkey::default();
    binding.version = 0;

    emit!(CrossBorderIdentityBound {
        binding: binding.key(),
        identity_hash: binding.identity_hash,
        jurisdiction: binding.jurisdiction_key,
        bound_at: now,
    });
    Ok(())
}

/// A validator verifies that a binding is valid and the credential has not been revoked.
pub fn verify_jurisdiction_membership(
    ctx: Context<super::VerifyJurisdictionMembership>,
    _off_chain_nonce: [u8; 32],
) -> Result<()> {
    crate::validator_registry::require_not_paused(&ctx.accounts.registry)?;
    let binding = &mut ctx.accounts.binding;
    require!(!binding.revoked, TerraError::BindingRevoked);

    let now = Clock::get()?.unix_timestamp;
    if binding.expires_at != 0 {
        require!(binding.expires_at > now, TerraError::BindingExpired);
    }

    binding.version = binding.version.saturating_add(1);
    binding.verified = true;
    binding.verified_by = ctx.accounts.validator.key();

    emit!(JurisdictionMembershipVerified {
        binding: binding.key(),
        identity_hash: binding.identity_hash,
        jurisdiction: binding.jurisdiction_key,
        validator: ctx.accounts.validator.key(),
        version: binding.version,
        block_time: now,
    });
    Ok(())
}

/// Revoke a binding if the original credential has been revoked.
pub fn revoke_jurisdictional_identity(
    ctx: Context<super::RevokeJurisdictionalIdentity>,
    reason: String,
) -> Result<()> {
    require!(reason.len() <= MAX_REASON_LEN, TerraError::NotesTooLong);

    let binding = &mut ctx.accounts.binding;
    require!(!binding.revoked, TerraError::BindingAlreadyRevoked);

    let now = Clock::get()?.unix_timestamp;
    binding.revoked = true;
    binding.revoked_at = now;
    binding.revoked_by = ctx.accounts.authority.key();
    binding.version = binding.version.saturating_add(1);

    emit!(JurisdictionIdentityRevoked {
        binding: binding.key(),
        identity_hash: binding.identity_hash,
        jurisdiction: binding.jurisdiction_key,
        reason,
        revoked_by: binding.revoked_by,
        block_time: now,
    });
    Ok(())
}

/// Re-bind after verification key rotation or credential refresh.
pub fn rebind_cross_border_identity(
    ctx: Context<super::RebindCrossBorderIdentity>,
    credential_commitment: [u8; 32],
    proof_data: Vec<u8>,
    nullifier_nonce: [u8; 32],
    expires_at: i64,
) -> Result<()> {
    require!(
        !proof_data.is_empty() && proof_data.len() <= MAX_PROOF_LEN,
        TerraError::InvalidProofData
    );
    require!(
        !credential_commitment.iter().all(|b| *b == 0),
        TerraError::EmptyGeometryHash
    );
    require!(
        !nullifier_nonce.iter().all(|b| *b == 0),
        TerraError::NonceRequired
    );

    let now = Clock::get()?.unix_timestamp;
    if expires_at != 0 {
        require!(expires_at > now, TerraError::InvalidExpiry);
    }

    let old_binding = &ctx.accounts.old_binding;
    require!(!old_binding.revoked, TerraError::BindingRevoked);

    let jurisdiction = &ctx.accounts.jurisdiction;
    require!(
        jurisdiction.status == jurisdiction_status::ACTIVE,
        TerraError::InvalidJurisdictionStatus
    );

    let jurisdiction_bytes = ctx.accounts.jurisdiction.key().to_bytes();
    let mut input = Vec::with_capacity(96);
    input.extend_from_slice(&credential_commitment);
    input.extend_from_slice(&jurisdiction_bytes);
    input.extend_from_slice(&nullifier_nonce);
    let nullifier: [u8; 32] = sha256_hash(&input).to_bytes();

    let new_binding = &mut ctx.accounts.new_binding;
    new_binding.identity_hash = old_binding.identity_hash;
    new_binding.jurisdiction_key = jurisdiction.key();
    new_binding.credential_commitment = credential_commitment;
    new_binding.nullifier = nullifier;
    new_binding.proof_data = proof_data;
    new_binding.proof_version = 0;
    new_binding.algorithm_id = jurisdiction.algorithm_id;
    new_binding.revoked = false;
    new_binding.revoked_at = 0;
    new_binding.bound_at = now;
    new_binding.expires_at = expires_at;
    // Fresh claims are unverified until a validator attests via
    // verify_jurisdiction_membership.
    new_binding.verified = false;
    new_binding.verified_by = Pubkey::default();
    new_binding.version = 0;

    emit!(CrossBorderIdentityRebound {
        old_binding: old_binding.key(),
        new_binding: new_binding.key(),
        identity_hash: new_binding.identity_hash,
        jurisdiction: new_binding.jurisdiction_key,
        bound_at: now,
    });
    Ok(())
}

// ---------------------------------------------------------------------------
// RFC-012 Phase 10 — cross-border jurisdiction links & spanning verifications
// ---------------------------------------------------------------------------

/// Lifecycle of a jurisdiction-to-jurisdiction link (`CrossBorderBinding`).
pub mod binding_status {
    pub const ACTIVE: u8 = 0;
    pub const SUSPENDED: u8 = 1;
    /// Terminal — no further transitions.
    pub const REVOKED: u8 = 2;
    pub const MAX: u8 = REVOKED;
}

/// Canonical ordering of a country pair: lexicographic (min, max).
/// Keeps the PDA `["cross_border_binding", min, max]` unique per unordered pair.
pub fn order_country_pair(a: [u8; 2], b: [u8; 2]) -> ([u8; 2], [u8; 2]) {
    if a <= b {
        (a, b)
    } else {
        (b, a)
    }
}

pub fn is_valid_binding_status(status: u8) -> bool {
    status <= binding_status::MAX
}

/// ACTIVE ↔ SUSPENDED, both may be REVOKED; REVOKED is terminal.
/// Setting the same status is an idempotent no-op update.
pub fn can_transition_binding(from: u8, to: u8) -> bool {
    use binding_status::*;
    if from == REVOKED {
        return false;
    }
    if to > MAX {
        return false;
    }
    matches!(
        (from, to),
        (ACTIVE, ACTIVE)
            | (ACTIVE, SUSPENDED)
            | (ACTIVE, REVOKED)
            | (SUSPENDED, SUSPENDED)
            | (SUSPENDED, ACTIVE)
            | (SUSPENDED, REVOKED)
    )
}

/// `expires_at == 0` means no expiry.
pub fn binding_is_expired(expires_at: i64, now: i64) -> bool {
    expires_at != 0 && now >= expires_at
}

/// Jurisdiction-to-jurisdiction recognition link (RFC-012 §5 `CrossBorderBinding`).
///
/// PDA: `["cross_border_binding", country_min, country_max]` — the country pair
/// is stored canonically ordered, so one PDA exists per unordered pair and
/// routing can derive it from `TaskRequirement.jurisdiction` + profile
/// jurisdiction without extra lookups.
#[account]
#[derive(InitSpace)]
pub struct CrossBorderBinding {
    /// Smaller of the two ISO 3166-1 alpha-2 codes (2 bytes).
    pub country_min: [u8; 2],
    /// Larger of the two codes.
    pub country_max: [u8; 2],
    /// Jurisdiction PDA of `country_min`.
    pub jurisdiction_min: Pubkey,
    /// Jurisdiction PDA of `country_max`.
    pub jurisdiction_max: Pubkey,
    /// `binding_status` — ACTIVE / SUSPENDED / REVOKED (terminal).
    pub status: u8,
    /// Unix expiry (0 = none).
    pub expires_at: i64,
    pub created_at: i64,
    pub updated_at: i64,
}

/// A verification that spanned two jurisdictions (RFC-012 §5
/// `CrossBorderVerification`) — the auditable record that a validator from
/// `country_home` produced a result for a task requiring `country_required`
/// under an ACTIVE `CrossBorderBinding`.
///
/// PDA: `["cross_border_verification", task_id, validator]`.
#[account]
#[derive(InitSpace)]
pub struct CrossBorderSpanRecord {
    pub task_id: [u8; 32],
    pub validator: Pubkey,
    pub req_index: u8,
    /// The validator's declared `ValidatorProfile.jurisdiction`.
    pub country_home: [u8; 2],
    /// The task requirement's jurisdiction.
    pub country_required: [u8; 2],
    /// The `CrossBorderBinding` PDA that authorized the span.
    pub binding: Pubkey,
    pub recorded_at: i64,
}

/// Both jurisdictions sign (treaty semantics): authority of each Jurisdiction
/// PDA must sign; countries must be distinct and canonically ordered; both
/// must be ACTIVE; expiry must be in the future (or 0).
pub fn create_cross_border_binding(
    ctx: Context<crate::CreateCrossBorderBinding>,
    country_min: [u8; 2],
    country_max: [u8; 2],
    expires_at: i64,
) -> Result<()> {
    require!(
        country_min != country_max && country_min < country_max,
        TerraError::InvalidJurisdictionPair
    );
    let now = Clock::get()?.unix_timestamp;
    require!(
        expires_at == 0 || expires_at > now,
        TerraError::InvalidBindingExpiry
    );
    let jm = &ctx.accounts.jurisdiction_min;
    let jx = &ctx.accounts.jurisdiction_max;
    require!(jm.key() != jx.key(), TerraError::InvalidJurisdictionPair);
    require!(
        jm.status == jurisdiction_status::ACTIVE && jx.status == jurisdiction_status::ACTIVE,
        TerraError::InvalidJurisdictionStatus
    );
    require!(
        [jm.country_code[0], jm.country_code[1]] == country_min,
        TerraError::InvalidJurisdictionPair
    );
    require!(
        [jx.country_code[0], jx.country_code[1]] == country_max,
        TerraError::InvalidJurisdictionPair
    );

    let b = &mut ctx.accounts.binding;
    b.country_min = country_min;
    b.country_max = country_max;
    b.jurisdiction_min = jm.key();
    b.jurisdiction_max = jx.key();
    b.status = binding_status::ACTIVE;
    b.expires_at = expires_at;
    b.created_at = now;
    b.updated_at = now;

    emit!(crate::CrossBorderBindingCreated {
        binding: b.key(),
        country_min,
        country_max,
        expires_at,
        created_at: now,
    });
    Ok(())
}

/// Suspend / revoke / reactivate a binding. Either authority may act;
/// REVOKED is terminal. Context pins both Jurisdiction accounts to the
/// ones stored on the binding so a foreign authority cannot pass two of
/// its own jurisdictions to satisfy the OR-signer check.
pub fn set_cross_border_binding_status(
    ctx: Context<crate::SetCrossBorderBindingStatus>,
    status: u8,
) -> Result<()> {
    require!(
        is_valid_binding_status(status),
        TerraError::InvalidBindingStatus
    );
    let b = &mut ctx.accounts.binding;
    require!(
        can_transition_binding(b.status, status),
        TerraError::BindingAlreadyRevoked
    );
    let now = Clock::get()?.unix_timestamp;
    b.status = status;
    b.updated_at = now;

    emit!(crate::CrossBorderBindingStatusChanged {
        binding: b.key(),
        status,
        updated_by: ctx.accounts.authority.key(),
        updated_at: now,
    });
    Ok(())
}

/// Permissionless auditable record: anyone may pay rent to record that a
/// completed/submitted assignment spanned two jurisdictions under an ACTIVE
/// binding (the accounts prove every fact — no trusted recorder).
pub fn record_cross_border_verification(
    ctx: Context<crate::RecordCrossBorderVerification>,
    task_id: [u8; 32],
    req_index: u8,
    validator: Pubkey,
) -> Result<()> {
    let task = &ctx.accounts.task;
    require!(task.task_id == task_id, TerraError::InvalidTaskRequirement);
    let req = &ctx.accounts.requirement;
    require!(
        req.task_id == task_id && req.req_index == req_index,
        TerraError::InvalidTaskRequirement
    );
    let assignment = &ctx.accounts.assignment;
    require!(
        assignment.status == crate::verification_task::assignment_status::SUBMITTED
            || assignment.status == crate::verification_task::assignment_status::RELEASED,
        TerraError::AssignmentNotSubmitted
    );
    let profile = &ctx.accounts.profile;
    require!(profile.wallet == validator, TerraError::NotAuthorized);

    let home = profile.jurisdiction;
    let required = req.jurisdiction;
    require!(
        home != [0, 0] && required != [0, 0],
        TerraError::UndeclaredJurisdiction
    );
    require!(home != required, TerraError::SameJurisdiction);

    let (c_min, c_max) = order_country_pair(home, required);
    let (expected, _) = Pubkey::find_program_address(
        &[b"cross_border_binding", c_min.as_ref(), c_max.as_ref()],
        &crate::ID,
    );
    require_keys_eq!(
        ctx.accounts.binding.key(),
        expected,
        TerraError::JurisdictionMismatch
    );
    let binding = &ctx.accounts.binding;
    require!(
        binding.status == binding_status::ACTIVE,
        TerraError::BindingRevoked
    );
    require!(
        !binding_is_expired(binding.expires_at, Clock::get()?.unix_timestamp),
        TerraError::BindingExpired
    );

    let now = Clock::get()?.unix_timestamp;
    let v = &mut ctx.accounts.verification;
    v.task_id = task_id;
    v.validator = validator;
    v.req_index = req_index;
    v.country_home = home;
    v.country_required = required;
    v.binding = binding.key();
    v.recorded_at = now;

    emit!(crate::CrossBorderVerificationRecorded {
        verification: v.key(),
        task_id,
        validator,
        country_home: home,
        country_required: required,
        binding: v.binding,
        recorded_at: now,
    });
    Ok(())
}

// ---------------------------------------------------------------------------
// Events
// ---------------------------------------------------------------------------

#[event]
pub struct JurisdictionRegistered {
    pub jurisdiction: Pubkey,
    pub country_code: [u8; 16],
    pub authority: Pubkey,
    pub algorithm_id: u8,
}

#[event]
pub struct JurisdictionUpdated {
    pub jurisdiction: Pubkey,
    pub updated_by: Pubkey,
}

#[event]
pub struct CrossBorderIdentityBound {
    pub binding: Pubkey,
    pub identity_hash: [u8; 32],
    pub jurisdiction: Pubkey,
    pub bound_at: i64,
}

#[event]
pub struct JurisdictionMembershipVerified {
    pub binding: Pubkey,
    pub identity_hash: [u8; 32],
    pub jurisdiction: Pubkey,
    pub validator: Pubkey,
    pub version: u32,
    pub block_time: i64,
}

#[event]
pub struct JurisdictionIdentityRevoked {
    pub binding: Pubkey,
    pub identity_hash: [u8; 32],
    pub jurisdiction: Pubkey,
    pub reason: String,
    pub revoked_by: Pubkey,
    pub block_time: i64,
}

#[event]
pub struct CrossBorderIdentityRebound {
    pub old_binding: Pubkey,
    pub new_binding: Pubkey,
    pub identity_hash: [u8; 32],
    pub jurisdiction: Pubkey,
    pub bound_at: i64,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jurisdiction_status_values_are_contiguous() {
        assert_eq!(jurisdiction_status::ACTIVE, 0);
        assert_eq!(jurisdiction_status::SUSPENDED, 1);
        assert_eq!(jurisdiction_status::WITHDRAWN, 2);
    }

    #[test]
    fn algorithm_id_values() {
        assert_eq!(algorithm_id::GROTH16, 0);
        assert_eq!(algorithm_id::FRI_STARK, 1);
    }

    #[test]
    fn max_proof_len_is_512() {
        assert_eq!(MAX_PROOF_LEN, 512);
    }

    #[test]
    fn max_reason_len_is_128() {
        assert_eq!(MAX_REASON_LEN, 128);
    }

    #[test]
    fn binding_status_values_are_contiguous() {
        assert_eq!(binding_status::ACTIVE, 0);
        assert_eq!(binding_status::SUSPENDED, 1);
        assert_eq!(binding_status::REVOKED, 2);
        assert_eq!(binding_status::MAX, 2);
        assert!(is_valid_binding_status(0));
        assert!(is_valid_binding_status(2));
        assert!(!is_valid_binding_status(3));
    }

    #[test]
    fn country_pair_ordering_is_canonical() {
        assert_eq!(order_country_pair(*b"CM", *b"KE"), (*b"CM", *b"KE"));
        assert_eq!(order_country_pair(*b"KE", *b"CM"), (*b"CM", *b"KE"));
        assert_eq!(order_country_pair(*b"AA", *b"AA"), (*b"AA", *b"AA"));
    }

    #[test]
    fn binding_transitions_respect_revoked_terminal() {
        use binding_status::*;
        assert!(can_transition_binding(ACTIVE, SUSPENDED));
        assert!(can_transition_binding(ACTIVE, REVOKED));
        assert!(can_transition_binding(SUSPENDED, ACTIVE));
        assert!(can_transition_binding(SUSPENDED, REVOKED));
        assert!(can_transition_binding(ACTIVE, ACTIVE)); // idempotent
        assert!(!can_transition_binding(REVOKED, ACTIVE));
        assert!(!can_transition_binding(REVOKED, SUSPENDED));
        assert!(!can_transition_binding(REVOKED, REVOKED));
        assert!(!can_transition_binding(ACTIVE, 9)); // out of range
    }

    #[test]
    fn binding_expiry_zero_means_never() {
        assert!(!binding_is_expired(0, 1_700_000_000));
        assert!(!binding_is_expired(1_800_000_000, 1_700_000_000));
        assert!(binding_is_expired(1_700_000_000, 1_700_000_000)); // boundary
        assert!(binding_is_expired(1_600_000_000, 1_700_000_000));
    }
}
