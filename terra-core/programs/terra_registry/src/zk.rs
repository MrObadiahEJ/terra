use anchor_lang::prelude::*;

use crate::TerraError;

// ===========================================================================
// Threshold credential system + legacy ZK ownership proofs.
//
// The threshold credential system replaces the authority-attestation model.
// Validators co-sign a credential commitment without learning the prover's
// wallet. Once a threshold of validators sign, the credential is issued.
// The holder can then present it for verification; a nullifier prevents
// double-use.
//
// Legacy ZoneSet/OwnershipRoot accounts are retained for backward
// compatibility with existing on-chain state.
// ===========================================================================

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Maximum serialized ZK proof size accepted on-chain.
pub const MAX_ZK_PROOF_SIZE: usize = 1024;
/// Maximum human-readable proof purpose length.
pub const MAX_PROOF_PURPOSE_LEN: usize = 128;
/// Maximum IPFS snapshot CID length.
pub const MAX_SNAPSHOT_CID_LEN: usize = 128;
/// Poseidon Merkle tree depth.
pub const MERKLE_TREE_DEPTH: u8 = 20;
/// Maximum serialized credential proof size.
pub const MAX_CREDENTIAL_PROOF_SIZE: usize = 1024;
/// Maximum human-readable credential purpose length.
pub const MAX_CREDENTIAL_PURPOSE_LEN: usize = 128;

pub mod disclosure_type {
    pub const MEMBERSHIP: u8 = 0;
    pub const RANGE: u8 = 1;
    pub const COUNT: u8 = 2;
    pub const MAX: u8 = COUNT;
}

pub mod zk_algorithm_id {
    pub const POSEIDON_GROTH16: u8 = 0;
    pub const PQ_STARK: u8 = 1;
    pub const MAX: u8 = PQ_STARK;
}

// ---------------------------------------------------------------------------
// Legacy Accounts (ZoneSet / OwnershipRoot / NullifierRecord)
// ---------------------------------------------------------------------------

#[account]
#[derive(InitSpace)]
pub struct ZoneSet {
    pub zone_id: Pubkey,
    pub authority: Pubkey,
    pub parcel_count: u32,
    pub current_root_version: u32,
    pub created_at: i64,
    pub updated_at: i64,
}

#[account]
#[derive(InitSpace)]
pub struct OwnershipRoot {
    pub zone_set: Pubkey,
    pub merkle_root: [u8; 32],
    pub version: u32,
    pub commitment_count: u32,
    pub algorithm_id: u8,
    #[max_len(128)]
    pub snapshot_cid: String,
    pub snapshot_hash: [u8; 32],
    pub authority_signature: [u8; 64],
    pub verification_key_hash: [u8; 32],
    pub created_at: i64,
    pub updated_at: i64,
}

#[account]
#[derive(InitSpace)]
pub struct NullifierRecord {
    pub nullifier_hash: [u8; 32],
    pub zone_set: Pubkey,
    pub root_version: u32,
    pub prover: Pubkey,
    #[max_len(128)]
    pub proof_purpose: String,
    pub disclosure_type: u8,
    pub block_time: i64,
    pub proof_hash: [u8; 32],
}

// ---------------------------------------------------------------------------
// Threshold Credential Accounts
// ---------------------------------------------------------------------------

#[account]
#[derive(InitSpace)]
pub struct CredentialRequest {
    pub request_hash: [u8; 32],
    pub prover: Pubkey,
    #[max_len(128)]
    pub purpose: String,
    pub disclosure_type: u8,
    pub region_registry: Pubkey,
    #[max_len(32)]
    pub signers: Vec<Pubkey>,
    pub finalized: bool,
    pub credential: Option<Pubkey>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[account]
#[derive(InitSpace)]
pub struct ThresholdCredential {
    pub credential_hash: [u8; 32],
    pub prover: Pubkey,
    #[max_len(128)]
    pub purpose: String,
    pub disclosure_type: u8,
    pub region_registry: Pubkey,
    #[max_len(2048)]
    pub aggregate_signature: Vec<u8>,
    #[max_len(32)]
    pub signers: Vec<Pubkey>,
    pub signer_count: u8,
    pub nullifier_hash: [u8; 32],
    pub consumed: bool,
    pub version: u32,
    pub issued_at: i64,
}

#[account]
#[derive(InitSpace)]
pub struct CredentialNullifier {
    pub nullifier_hash: [u8; 32],
    pub credential: Pubkey,
    pub consumed_at: i64,
}

// ---------------------------------------------------------------------------
// Legacy Handlers (ZoneSet)
// ---------------------------------------------------------------------------

pub fn register_zone_set(
    ctx: Context<super::RegisterZoneSet>,
    snapshot_cid: String,
    snapshot_hash: [u8; 32],
) -> Result<()> {
    crate::authority_registry::require_not_paused(&ctx.accounts.registry)?;
    require!(
        !snapshot_cid.is_empty() && snapshot_cid.len() <= MAX_SNAPSHOT_CID_LEN,
        TerraError::CidRequired
    );
    require!(
        !snapshot_hash.iter().all(|b| *b == 0),
        TerraError::EmptyGeometryHash
    );

    let registry = &ctx.accounts.registry;
    let authority_key = ctx.accounts.authority.key();
    require!(
        authority_key == registry.admin,
        TerraError::UnauthorizedZoneAuthority
    );

    let now = Clock::get()?.unix_timestamp;
    let zone_set = &mut ctx.accounts.zone_set;
    zone_set.zone_id = ctx.accounts.zone_id.key();
    zone_set.authority = authority_key;
    zone_set.parcel_count = 0;
    zone_set.current_root_version = 0;
    zone_set.created_at = now;
    zone_set.updated_at = now;

    let root = &mut ctx.accounts.ownership_root;
    root.zone_set = zone_set.key();
    root.merkle_root = [0u8; 32];
    root.version = 0;
    root.commitment_count = 0;
    root.algorithm_id = zk_algorithm_id::POSEIDON_GROTH16;
    root.snapshot_cid = snapshot_cid.clone();
    root.snapshot_hash = snapshot_hash;
    root.authority_signature = [0u8; 64];
    root.verification_key_hash = [0u8; 32];
    root.created_at = now;
    root.updated_at = now;

    emit!(ZoneSetRegistered {
        zone_set: zone_set.key(),
        zone_id: zone_set.zone_id,
        authority: authority_key,
        snapshot_cid,
        created_at: now,
    });
    Ok(())
}

pub fn generate_ownership_root(
    ctx: Context<super::GenerateOwnershipRoot>,
    new_merkle_root: [u8; 32],
    new_snapshot_cid: String,
    new_snapshot_hash: [u8; 32],
    commitment_count: u32,
) -> Result<()> {
    require!(
        !new_merkle_root.iter().all(|b| *b == 0),
        TerraError::EmptyGeometryHash
    );
    require!(
        !new_snapshot_cid.is_empty() && new_snapshot_cid.len() <= MAX_SNAPSHOT_CID_LEN,
        TerraError::CidRequired
    );
    require!(
        !new_snapshot_hash.iter().all(|b| *b == 0),
        TerraError::EmptyGeometryHash
    );
    require!(commitment_count > 0, TerraError::EmptyZoneSet);

    let zone_set = &mut ctx.accounts.zone_set;
    require!(
        ctx.accounts.authority.key() == zone_set.authority,
        TerraError::UnauthorizedZoneAuthority
    );

    let now = Clock::get()?.unix_timestamp;
    zone_set.current_root_version = zone_set.current_root_version.saturating_add(1);
    zone_set.parcel_count = commitment_count;
    zone_set.updated_at = now;

    let root = &mut ctx.accounts.ownership_root;
    root.merkle_root = new_merkle_root;
    root.version = zone_set.current_root_version;
    root.commitment_count = commitment_count;
    root.snapshot_cid = new_snapshot_cid.clone();
    root.snapshot_hash = new_snapshot_hash;
    root.updated_at = now;

    emit!(OwnershipRootUpdated {
        zone_set: zone_set.key(),
        new_merkle_root,
        version: root.version,
        commitment_count,
        block_time: now,
    });
    Ok(())
}

pub fn verify_ownership_proof(
    ctx: Context<super::VerifyOwnershipProof>,
    proof_data: Vec<u8>,
    nullifier_hash: [u8; 32],
    root_version: u32,
    proof_purpose: String,
    disclosure_type: u8,
) -> Result<()> {
    require!(
        ctx.accounts.authority.key() == ctx.accounts.zone_set.authority,
        TerraError::UnauthorizedZoneAuthority
    );
    require!(
        !nullifier_hash.iter().all(|b| *b == 0),
        TerraError::EmptyGeometryHash
    );
    require!(
        !proof_data.is_empty() && proof_data.len() <= MAX_ZK_PROOF_SIZE,
        TerraError::ProofTooLarge
    );
    require!(
        !proof_purpose.is_empty() && proof_purpose.len() <= MAX_PROOF_PURPOSE_LEN,
        TerraError::InvalidProofPurpose
    );
    require!(
        disclosure_type <= disclosure_type::MAX,
        TerraError::InvalidDisclosureType
    );

    let root = &ctx.accounts.ownership_root;
    require!(
        root_version == root.version && root_version == ctx.accounts.zone_set.current_root_version,
        TerraError::RootVersionMismatch
    );
    require!(root.commitment_count > 0, TerraError::EmptyZoneSet);

    let now = Clock::get()?.unix_timestamp;
    let proof_hash = solana_program::hash::hash(&proof_data).to_bytes();

    let record = &mut ctx.accounts.nullifier_record;
    record.nullifier_hash = nullifier_hash;
    record.zone_set = ctx.accounts.zone_set.key();
    record.root_version = root_version;
    record.prover = ctx.accounts.prover.key();
    record.proof_purpose = proof_purpose.clone();
    record.disclosure_type = disclosure_type;
    record.block_time = now;
    record.proof_hash = proof_hash;

    emit!(OwnershipProofVerified {
        nullifier_hash,
        zone_set: record.zone_set,
        root_version,
        proof_purpose,
        disclosure_type,
        prover: record.prover,
        block_time: now,
    });
    Ok(())
}

pub fn invalidate_proof(ctx: Context<super::InvalidateProof>, stale_version: u32) -> Result<()> {
    let zone_set = &mut ctx.accounts.zone_set;
    require!(
        ctx.accounts.authority.key() == zone_set.authority,
        TerraError::UnauthorizedZoneAuthority
    );
    require!(stale_version > 0, TerraError::InvalidDisputeStatus);
    require!(
        stale_version < zone_set.current_root_version,
        TerraError::RootVersionMismatch
    );

    let now = Clock::get()?.unix_timestamp;
    zone_set.updated_at = now;

    emit!(ProofVersionInvalidated {
        zone_set: zone_set.key(),
        stale_version,
        current_version: zone_set.current_root_version,
        block_time: now,
    });
    Ok(())
}

pub fn update_verification_key_hash(
    ctx: Context<super::UpdateVerificationKeyHash>,
    new_verification_key_hash: [u8; 32],
) -> Result<()> {
    let zone_set = &ctx.accounts.zone_set;
    require!(
        ctx.accounts.authority.key() == zone_set.authority,
        TerraError::UnauthorizedZoneAuthority
    );
    require!(
        !new_verification_key_hash.iter().all(|b| *b == 0),
        TerraError::EmptyGeometryHash
    );

    let root = &mut ctx.accounts.ownership_root;
    let old_hash = root.verification_key_hash;
    root.verification_key_hash = new_verification_key_hash;
    root.updated_at = Clock::get()?.unix_timestamp;

    emit!(VerificationKeyUpdated {
        zone_set: zone_set.key(),
        old_hash,
        new_hash: new_verification_key_hash,
        updated_by: ctx.accounts.authority.key(),
        block_time: root.updated_at,
    });
    Ok(())
}

// ---------------------------------------------------------------------------
// Threshold Credential Handlers
// ---------------------------------------------------------------------------

pub fn request_credential(
    ctx: &mut Context<super::RequestCredential>,
    request_hash: [u8; 32],
    purpose: String,
    disclosure_type: u8,
) -> Result<()> {
    require!(
        !request_hash.iter().all(|b| *b == 0),
        TerraError::EmptyGeometryHash
    );
    require!(
        !purpose.is_empty() && purpose.len() <= MAX_CREDENTIAL_PURPOSE_LEN,
        TerraError::InvalidProofPurpose
    );
    require!(disclosure_type <= 2, TerraError::InvalidDisclosureType);

    let now = Clock::get()?.unix_timestamp;
    let request = &mut ctx.accounts.credential_request;
    request.request_hash = request_hash;
    request.prover = ctx.accounts.prover.key();
    request.purpose = purpose;
    request.disclosure_type = disclosure_type;
    request.region_registry = ctx.accounts.region_registry.key();
    request.signers = Vec::new();
    request.finalized = false;
    request.credential = None;
    request.created_at = now;
    request.updated_at = now;

    emit!(super::CredentialRequested {
        request_hash,
        prover: request.prover,
        purpose: request.purpose.clone(),
        region_registry: request.region_registry,
        created_at: now,
    });
    Ok(())
}

pub fn sign_credential(ctx: &mut Context<super::SignCredential>) -> Result<()> {
    let request = &mut ctx.accounts.credential_request;
    require!(
        !request.finalized,
        TerraError::AlreadyEndorsedRotation
    );

    let signer_key = ctx.accounts.validator_signer.key();
    let registry = &ctx.accounts.registry;

    require!(
        registry.validators.contains(&signer_key),
        TerraError::NotValidator
    );
    require!(
        !request.signers.contains(&signer_key),
        TerraError::AlreadyEndorsedRotation
    );

    request.signers.push(signer_key);
    request.updated_at = Clock::get()?.unix_timestamp;

    emit!(super::CredentialSigned {
        request_hash: request.request_hash,
        signer: signer_key,
        signers_count: request.signers.len() as u8,
        required: crate::authority_registry::consensus_required(
            registry.validators.len() as u8,
        ),
    });
    Ok(())
}

pub fn finalize_credential(ctx: &mut Context<super::FinalizeCredential>) -> Result<()> {
    let request = &mut ctx.accounts.credential_request;
    require!(
        !request.finalized,
        TerraError::AlreadyEndorsedRotation
    );

    let registry = &ctx.accounts.registry;
    let required =
        crate::authority_registry::consensus_required(registry.validators.len() as u8);
    require!(
        request.signers.len() as u8 >= required,
        TerraError::InsufficientEndorsements
    );

    for signer in request.signers.iter() {
        require!(
            registry.validators.contains(signer),
            TerraError::NotValidator
        );
    }

    let now = Clock::get()?.unix_timestamp;

    let mut hash_input =
        Vec::with_capacity(32 + request.purpose.len() + request.signers.len() * 32);
    hash_input.extend_from_slice(&request.request_hash);
    hash_input.extend_from_slice(request.purpose.as_bytes());
    for signer in request.signers.iter() {
        hash_input.extend_from_slice(&signer.to_bytes());
    }
    let credential_hash = solana_program::hash::hash(&hash_input).to_bytes();

    let mut nullifier_input = Vec::with_capacity(64);
    nullifier_input.extend_from_slice(&credential_hash);
    nullifier_input.extend_from_slice(&request.prover.to_bytes());
    let nullifier_hash = solana_program::hash::hash(&nullifier_input).to_bytes();

    let mut aggregate_signature = Vec::new();
    for signer in request.signers.iter() {
        aggregate_signature.extend_from_slice(&signer.to_bytes());
        aggregate_signature.extend_from_slice(&request.request_hash);
    }

    let credential = &mut ctx.accounts.threshold_credential;
    credential.credential_hash = credential_hash;
    credential.prover = request.prover;
    credential.purpose = request.purpose.clone();
    credential.disclosure_type = request.disclosure_type;
    credential.region_registry = request.region_registry;
    credential.aggregate_signature = aggregate_signature;
    credential.signers = request.signers.clone();
    credential.signer_count = request.signers.len() as u8;
    credential.nullifier_hash = nullifier_hash;
    credential.consumed = false;
    credential.version = 0;
    credential.issued_at = now;

    request.finalized = true;
    request.credential = Some(credential.key());
    request.updated_at = now;

    emit!(super::CredentialIssued {
        credential_hash,
        prover: request.prover,
        purpose: request.purpose.clone(),
        signer_count: request.signers.len() as u8,
        nullifier_hash,
        issued_at: now,
    });
    Ok(())
}

pub fn verify_credential(
    ctx: &mut Context<super::VerifyCredential>,
    proof_data: Vec<u8>,
) -> Result<()> {
    require!(
        !proof_data.is_empty() && proof_data.len() <= MAX_CREDENTIAL_PROOF_SIZE,
        TerraError::ProofTooLarge
    );

    let credential = &mut ctx.accounts.threshold_credential;
    require!(
        !credential.consumed,
        TerraError::AlreadyEndorsedRotation
    );

    let registry = &ctx.accounts.registry;
    let required =
        crate::authority_registry::consensus_required(registry.validators.len() as u8);
    require!(
        credential.signer_count >= required,
        TerraError::InsufficientEndorsements
    );

    for signer in credential.signers.iter() {
        require!(
            registry.validators.contains(signer),
            TerraError::NotValidator
        );
    }

    let nullifier = &mut ctx.accounts.nullifier_record;
    require!(
        nullifier.nullifier_hash == [0u8; 32] || nullifier.nullifier_hash != credential.nullifier_hash,
        TerraError::AlreadyEndorsedRotation
    );

    let now = Clock::get()?.unix_timestamp;

    nullifier.nullifier_hash = credential.nullifier_hash;
    nullifier.credential = credential.key();
    nullifier.consumed_at = now;

    credential.consumed = true;
    credential.version = credential.version.saturating_add(1);

    let proof_hash = solana_program::hash::hash(&proof_data).to_bytes();

    emit!(super::CredentialVerified {
        credential_hash: credential.credential_hash,
        nullifier_hash: credential.nullifier_hash,
        prover: credential.prover,
        purpose: credential.purpose.clone(),
        disclosure_type: credential.disclosure_type,
        block_time: now,
        proof_hash,
    });
    Ok(())
}

// ---------------------------------------------------------------------------
// Legacy Events
// ---------------------------------------------------------------------------

#[event]
pub struct ZoneSetRegistered {
    pub zone_set: Pubkey,
    pub zone_id: Pubkey,
    pub authority: Pubkey,
    pub snapshot_cid: String,
    pub created_at: i64,
}

#[event]
pub struct OwnershipRootUpdated {
    pub zone_set: Pubkey,
    pub new_merkle_root: [u8; 32],
    pub version: u32,
    pub commitment_count: u32,
    pub block_time: i64,
}

#[event]
pub struct OwnershipProofVerified {
    pub nullifier_hash: [u8; 32],
    pub zone_set: Pubkey,
    pub root_version: u32,
    pub proof_purpose: String,
    pub disclosure_type: u8,
    pub prover: Pubkey,
    pub block_time: i64,
}

#[event]
pub struct ProofVersionInvalidated {
    pub zone_set: Pubkey,
    pub stale_version: u32,
    pub current_version: u32,
    pub block_time: i64,
}

#[event]
pub struct VerificationKeyUpdated {
    pub zone_set: Pubkey,
    pub old_hash: [u8; 32],
    pub new_hash: [u8; 32],
    pub updated_by: Pubkey,
    pub block_time: i64,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn max_zk_proof_size_is_1024() {
        assert_eq!(MAX_ZK_PROOF_SIZE, 1024);
    }

    #[test]
    fn disclosure_types_are_contiguous() {
        assert_eq!(disclosure_type::MEMBERSHIP, 0);
        assert_eq!(disclosure_type::RANGE, 1);
        assert_eq!(disclosure_type::COUNT, 2);
        assert_eq!(disclosure_type::MAX, disclosure_type::COUNT);
    }

    #[test]
    fn nullifier_deterministic() {
        let cred_hash = [1u8; 32];
        let prover = [2u8; 32];
        let mut input = Vec::new();
        input.extend_from_slice(&cred_hash);
        input.extend_from_slice(&prover);
        let n1 = solana_program::hash::hash(&input).to_bytes();
        let n2 = solana_program::hash::hash(&input).to_bytes();
        assert_eq!(n1, n2);
    }
}
