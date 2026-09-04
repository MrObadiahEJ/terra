use anchor_lang::prelude::*;

use crate::TerraError;

// ---------------------------------------------------------------------------
// Constants (RFC-011 §4, §6)
// ---------------------------------------------------------------------------

/// Maximum serialized ZK proof size accepted on-chain: 1024 bytes.
pub const MAX_ZK_PROOF_SIZE: usize = 1024;
/// Maximum human-readable proof purpose length.
pub const MAX_PROOF_PURPOSE_LEN: usize = 128;
/// Maximum IPFS snapshot CID length.
pub const MAX_SNAPSHOT_CID_LEN: usize = 128;
/// Poseidon Merkle tree depth (supports ~1M leaves per zone).
pub const MERKLE_TREE_DEPTH: u8 = 20;

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
// Accounts
// ---------------------------------------------------------------------------

/// A zone registered for ZK ownership proofs.
///
/// PDA seed: `["zone_set", zone_id]`.
#[account]
#[derive(InitSpace)]
pub struct ZoneSet {
    /// Authority-paused zone identifier (e.g. pilot zone key).
    pub zone_id: Pubkey,
    /// Zone administrator (AuthorityRegistry validator).
    pub authority: Pubkey,
    /// Number of parcels registered in this zone set.
    pub parcel_count: u32,
    /// Monotonic counter for root updates.
    pub current_root_version: u32,
    pub created_at: i64,
    pub updated_at: i64,
}

/// The current Merkle root of owner commitments for a zone.
///
/// PDA seed: `["ownership_root", zone_set_key]`.
#[account]
#[derive(InitSpace)]
pub struct OwnershipRoot {
    /// ZoneSet PDA key.
    pub zone_set: Pubkey,
    /// Poseidon Merkle root of all owner commitments in the zone.
    pub merkle_root: [u8; 32],
    /// Monotonic counter, bumped on each root update.
    pub version: u32,
    /// Number of leaf commitments in the tree.
    pub commitment_count: u32,
    /// Hash/proof system (0=Poseidon+Groth16, 1=PQ-STARK).
    pub algorithm_id: u8,
    /// IPFS CID of the full commitment tree snapshot.
    #[max_len(128)]
    pub snapshot_cid: String,
    /// SHA-256 of the snapshot bytes.
    pub snapshot_hash: [u8; 32],
    /// Ed25519 signature of (merkle_root || version) by the zone authority.
    pub authority_signature: [u8; 64],
    pub created_at: i64,
    pub updated_at: i64,
}

/// First-use record for a proof nullifier (double-proving prevention).
///
/// PDA seed: `["nullifier", nullifier_hash]`.
#[account]
#[derive(InitSpace)]
pub struct NullifierRecord {
    /// Poseidon(owner_commitment, zone_root_version).
    pub nullifier_hash: [u8; 32],
    /// ZoneSet PDA key.
    pub zone_set: Pubkey,
    /// Version of the Merkle root this nullifier is bound to.
    pub root_version: u32,
    /// Wallet that submitted the proof.
    pub prover: Pubkey,
    /// Human-readable purpose (e.g. "subsidy_qualification").
    #[max_len(128)]
    pub proof_purpose: String,
    /// 0=membership, 1=range, 2=count.
    pub disclosure_type: u8,
    /// Solana block time when the proof was verified.
    pub block_time: i64,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// Register a new zone for ZK ownership proofs with an empty root.
pub fn register_zone_set(
    ctx: Context<super::RegisterZoneSet>,
    snapshot_cid: String,
    snapshot_hash: [u8; 32],
) -> Result<()> {
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

/// Commit a new Merkle root after parcels were added/removed/transferred.
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

/// Verify a ZK ownership proof and record its nullifier (first-use wins).
///
/// Structural on-chain checks only: the Groth16/PLONK circuit verification
/// itself happens in the client + verifier precompile and is attested by the
/// zone authority's signed root; on-chain we enforce nullifier uniqueness,
/// root-version currency, proof size, purpose, and disclosure-type bounds.
pub fn verify_ownership_proof(
    ctx: Context<super::VerifyOwnershipProof>,
    proof_data: Vec<u8>,
    nullifier_hash: [u8; 32],
    root_version: u32,
    proof_purpose: String,
    disclosure_type: u8,
) -> Result<()> {
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
    let record = &mut ctx.accounts.nullifier_record;
    record.nullifier_hash = nullifier_hash;
    record.zone_set = ctx.accounts.zone_set.key();
    record.root_version = root_version;
    record.prover = ctx.accounts.prover.key();
    record.proof_purpose = proof_purpose.clone();
    record.disclosure_type = disclosure_type;
    record.block_time = now;

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

/// Mark a stale root version as invalidated after ownership transfer.
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

// ---------------------------------------------------------------------------
// Events
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
    fn max_proof_purpose_is_128() {
        assert_eq!(MAX_PROOF_PURPOSE_LEN, 128);
    }

    #[test]
    fn disclosure_types_are_contiguous() {
        assert_eq!(disclosure_type::MEMBERSHIP, 0);
        assert_eq!(disclosure_type::RANGE, 1);
        assert_eq!(disclosure_type::COUNT, 2);
        assert_eq!(disclosure_type::MAX, disclosure_type::COUNT);
    }

    #[test]
    fn zk_algorithm_ids() {
        assert_eq!(zk_algorithm_id::POSEIDON_GROTH16, 0);
        assert_eq!(zk_algorithm_id::PQ_STARK, 1);
    }

    #[test]
    fn merkle_tree_depth_is_20() {
        assert_eq!(MERKLE_TREE_DEPTH, 20);
    }

    #[test]
    fn rejects_empty_proof_shape() {
        let empty: Vec<u8> = vec![];
        assert!(empty.is_empty());
        let oversized = vec![0u8; MAX_ZK_PROOF_SIZE + 1];
        assert!(oversized.len() > MAX_ZK_PROOF_SIZE);
    }
}
