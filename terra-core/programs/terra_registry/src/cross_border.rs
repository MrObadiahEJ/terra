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
    binding.identity_hash = ctx.accounts.identity.identity_hash;
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
}
