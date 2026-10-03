use anchor_lang::prelude::*;

use crate::TerraError;

pub mod groth16;

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
// Signature verification (P0-ZK)
//
// RFC-011 §6.2/§8.3 require an Ed25519 authority signature over every
// committed Merkle root. Until the Poseidon/Groth16 circuit is wired (RFC-011
// §6.3 "verified by the circuit" — deferred to audit), proof presentations
// are verified as Ed25519 statements instead of being blindly hashed:
//
// * root attestation message  = merkle_root || version (RFC-011 §6.2, LE)
// * ownership proof statement  = DOMAIN || zone_set || merkle_root ||
//   root_version || nullifier || prover || purpose || disclosure,
//   signed by the PROVER (binds the presentation to the prover's key)
// * credential proof statement = DOMAIN || credential_hash || nullifier ||
//   region_registry || prover || purpose || disclosure, signed by the PROVER
//
// Statement encodings are length-delimited and domain-tagged so no two
// statement kinds can collide. Third parties can re-verify proof_data
// offline from transaction bytes.
//
// Signatures are verified by the Solana runtime's native Ed25519 precompile
// (agave-precompiles, ~0 program CU). In-program curve math with
// ed25519-dalek exceeds the 1.4M CU per-transaction ceiling on SBF, so each
// instruction takes an `instructions` sysvar account, locates the matching
// precompile instruction in the current transaction, and requires it to
// resolve entirely within that instruction (u16::MAX / self index) so the
// runtime checked exactly (signer, statement, signature). Transaction
// atomicity guarantees the precompile runs — an invalid signature aborts the
// whole transaction, including any state writes. Anything unmatched is
// rejected with TerraError::InvalidProofData.
// ---------------------------------------------------------------------------

const OWNERSHIP_PROOF_DOMAIN: &[u8] = b"TERRA_ZK_OWNERSHIP_PROOF_V1";
const CREDENTIAL_PROOF_DOMAIN: &[u8] = b"TERRA_ZK_CREDENTIAL_PROOF_V1";

/// RFC-011 §6.2: `sign(new_merkle_root || zone_set.current_root_version)`.
/// Version is little-endian (Solana convention); the signer must know the
/// version *after* the increment this instruction performs.
pub fn root_attestation_message(merkle_root: &[u8; 32], version: u32) -> [u8; 36] {
    let mut msg = [0u8; 36];
    msg[..32].copy_from_slice(merkle_root);
    msg[32..].copy_from_slice(&version.to_le_bytes());
    msg
}

fn push_str(buf: &mut Vec<u8>, s: &str) {
    // Length-delimited: purpose is variable-length and must not shift the
    // fields after it.
    buf.push(s.len() as u8);
    buf.extend_from_slice(s.as_bytes());
}

fn push_fixed_ownership_purpose(buf: &mut Vec<u8>, purpose: &str) {
    buf.push(purpose.len() as u8);
    buf.extend_from_slice(purpose.as_bytes());
    buf.resize(buf.len() + MAX_PROOF_PURPOSE_LEN - purpose.len(), 0);
}

/// Canonical ownership-proof statement (see module docs). Returns the exact
/// bytes the prover must sign and the program verifies.
pub fn ownership_proof_statement(
    zone_set: &Pubkey,
    merkle_root: &[u8; 32],
    root_version: u32,
    nullifier_hash: &[u8; 32],
    prover: &Pubkey,
    proof_purpose: &str,
    disclosure_type: u8,
) -> Vec<u8> {
    let mut m = Vec::with_capacity(
        OWNERSHIP_PROOF_DOMAIN.len() + 32 + 32 + 4 + 32 + 32 + 1 + proof_purpose.len() + 1,
    );
    m.extend_from_slice(OWNERSHIP_PROOF_DOMAIN);
    m.extend_from_slice(zone_set.as_ref());
    m.extend_from_slice(merkle_root);
    m.extend_from_slice(&root_version.to_le_bytes());
    m.extend_from_slice(nullifier_hash);
    m.extend_from_slice(prover.as_ref());
    push_fixed_ownership_purpose(&mut m, proof_purpose);
    m.push(disclosure_type);
    m
}

/// Canonical credential-proof statement (see module docs).
pub fn credential_proof_statement(
    credential_hash: &[u8; 32],
    nullifier_hash: &[u8; 32],
    region_registry: &Pubkey,
    prover: &Pubkey,
    purpose: &str,
    disclosure_type: u8,
) -> Vec<u8> {
    let mut m = Vec::with_capacity(
        CREDENTIAL_PROOF_DOMAIN.len() + 32 + 32 + 32 + 32 + 1 + purpose.len() + 1,
    );
    m.extend_from_slice(CREDENTIAL_PROOF_DOMAIN);
    m.extend_from_slice(credential_hash);
    m.extend_from_slice(nullifier_hash);
    m.extend_from_slice(region_registry.as_ref());
    m.extend_from_slice(prover.as_ref());
    push_str(&mut m, purpose);
    m.push(disclosure_type);
    m
}

/// Parse an Ed25519 precompile instruction that references only its own data
/// (`u16::MAX` or its own instruction index — the same convention the runtime
/// verifier in `agave-precompiles` uses for `u16::MAX`). Returns
/// `(pubkey, signature, message)` slices from the instruction data.
fn parse_ed25519_self_attestation(
    data: &[u8],
    self_index: usize,
) -> Option<(&[u8; 32], &[u8; 64], &[u8])> {
    // [0]=num_signatures, [1]=padding, [2..16]=Ed25519SignatureOffsets,
    // pubkey@16, signature@48, message@112 (see solana-ed25519-program).
    if data.len() < 16 + 32 + 64 || data[0] != 1 {
        return None;
    }
    let o = &data[2..16];
    let so = u16::from_le_bytes(o[0..2].try_into().ok()?) as usize;
    let si = u16::from_le_bytes(o[2..4].try_into().ok()?);
    let po = u16::from_le_bytes(o[4..6].try_into().ok()?) as usize;
    let pi = u16::from_le_bytes(o[6..8].try_into().ok()?);
    let mo = u16::from_le_bytes(o[8..10].try_into().ok()?) as usize;
    let ms = u16::from_le_bytes(o[10..12].try_into().ok()?) as usize;
    let mi = u16::from_le_bytes(o[12..14].try_into().ok()?);
    let refers_self = |idx: u16| idx == u16::MAX || idx as usize == self_index;
    if !refers_self(si) || !refers_self(pi) || !refers_self(mi) {
        return None;
    }
    let pk = <&[u8; 32]>::try_from(data.get(po..po + 32)?).ok()?;
    let sig = <&[u8; 64]>::try_from(data.get(so..so + 64)?).ok()?;
    Some((pk, sig, data.get(mo..mo + ms)?))
}

/// Locate the runtime Ed25519 precompile instruction attesting exactly
/// (`expected_signer`, `expected_message`, `expected_signature`) in the
/// current transaction and accept it. The runtime performs the actual
/// signature verification when the precompile executes; atomicity ties that
/// success to this instruction's state writes. `instructions` must be the
/// Instructions sysvar.
#[allow(deprecated)] // solana-program 3.0 re-exports of the instructions sysvar
pub fn verify_precompiled_ed25519(
    instructions: &AccountInfo,
    expected_signer: &Pubkey,
    expected_message: &[u8],
    expected_signature: &[u8],
) -> Result<()> {
    use solana_program::sysvar::instructions::{
        load_instruction_at_checked, ID as INSTRUCTIONS_SYSVAR_ID,
    };
    require_keys_eq!(
        *instructions.key,
        INSTRUCTIONS_SYSVAR_ID,
        TerraError::InvalidProofData
    );
    // A transaction fits at most ~60 instructions; the loader errors out of
    // range (after the address check above, errors are end-of-list).
    const MAX_IX_SCAN: usize = 64;
    for i in 0..MAX_IX_SCAN {
        let ix = match load_instruction_at_checked(i, instructions) {
            Ok(ix) => ix,
            Err(_) => break,
        };
        if ix.program_id != solana_program::ed25519_program::ID {
            continue;
        }
        if let Some((pk, sig, msg)) = parse_ed25519_self_attestation(&ix.data, i) {
            if pk == expected_signer.as_ref()
                && msg == expected_message
                && sig.as_slice() == expected_signature
            {
                return Ok(());
            }
        }
    }
    err!(TerraError::InvalidProofData)
}

// ---------------------------------------------------------------------------
// Legacy Handlers (ZoneSet)
// ---------------------------------------------------------------------------

pub fn register_zone_set(
    ctx: Context<super::RegisterZoneSet>,
    snapshot_cid: String,
    snapshot_hash: [u8; 32],
) -> Result<()> {
    crate::validator_registry::require_not_paused(&ctx.accounts.registry)?;
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
    authority_signature: [u8; 64],
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

    // RFC-011 §6.2: the committed root must carry the zone authority's
    // Ed25519 signature over `new_merkle_root || version` (§8.3 — prevents
    // accepting a root that was never authorized off-chain).
    let new_version = zone_set.current_root_version.saturating_add(1);
    let attestation = root_attestation_message(&new_merkle_root, new_version);
    verify_precompiled_ed25519(
        &ctx.accounts.instructions,
        &zone_set.authority,
        &attestation,
        &authority_signature,
    )?;

    let now = Clock::get()?.unix_timestamp;
    zone_set.current_root_version = new_version;
    zone_set.parcel_count = commitment_count;
    zone_set.updated_at = now;

    let root = &mut ctx.accounts.ownership_root;
    root.merkle_root = new_merkle_root;
    root.version = new_version;
    root.commitment_count = commitment_count;
    root.snapshot_cid = new_snapshot_cid.clone();
    root.snapshot_hash = new_snapshot_hash;
    root.authority_signature = authority_signature;
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

    // P0-ZK-01: proof_data is a framed v1 payload (RFC-011 §6.3) — the
    // prover's Ed25519 signature over the canonical statement (zone, root,
    // version, nullifier, prover, purpose, disclosure), the hash-pinned
    // Groth16 verification key, and the Groth16 proof. Framing is parsed
    // only after the guards above so existing error semantics
    // (ProofTooLarge / RootVersionMismatch / InvalidProofPurpose /
    // InvalidDisclosureType / EmptyZoneSet) are preserved; every
    // proof-layer failure below reports InvalidProofData (6087).
    let frame = groth16::parse_ownership_frame(&proof_data)?;
    require!(
        root.algorithm_id == zk_algorithm_id::POSEIDON_GROTH16,
        TerraError::InvalidProofData
    );
    require!(
        !root.verification_key_hash.iter().all(|b| *b == 0),
        TerraError::InvalidProofData
    );
    require!(
        solana_program::hash::hash(frame.verification_key).to_bytes() == root.verification_key_hash,
        TerraError::InvalidProofData
    );
    let statement = ownership_proof_statement(
        &ctx.accounts.zone_set.key(),
        &root.merkle_root,
        root.version,
        &nullifier_hash,
        &ctx.accounts.prover.key(),
        &proof_purpose,
        disclosure_type,
    );
    verify_precompiled_ed25519(
        &ctx.accounts.instructions,
        &ctx.accounts.prover.key(),
        &statement,
        frame.signature,
    )?;
    // Groth16 pairing equation over the canonical 248-bit SHA-256 statement prefix.
    // (statement-bound: zone, root, version, nullifier, prover, purpose and
    // disclosure are all covered by the circuit's public input).
    require!(
        groth16::verify_groth16(
            frame.verification_key,
            frame.proof,
            &groth16::public_input(&statement),
        ),
        TerraError::InvalidProofData
    );

    let now = Clock::get()?.unix_timestamp;
    let proof_hash =
        solana_program::hash::hash(&[statement.as_slice(), proof_data.as_slice()].concat())
            .to_bytes();

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
    require!(!request.finalized, TerraError::AlreadyEndorsedRotation);

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
        required: crate::validator_registry::consensus_required(registry.validators.len() as u8,),
    });
    Ok(())
}

pub fn finalize_credential(ctx: &mut Context<super::FinalizeCredential>) -> Result<()> {
    let request = &mut ctx.accounts.credential_request;
    require!(!request.finalized, TerraError::AlreadyEndorsedRotation);

    let registry = &ctx.accounts.registry;
    let required = crate::validator_registry::consensus_required(registry.validators.len() as u8);
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
    require!(!credential.consumed, TerraError::AlreadyEndorsedRotation);

    let registry = &ctx.accounts.registry;
    let required = crate::validator_registry::consensus_required(registry.validators.len() as u8);
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
        nullifier.nullifier_hash == [0u8; 32]
            || nullifier.nullifier_hash != credential.nullifier_hash,
        TerraError::AlreadyEndorsedRotation
    );

    // P0-ZK: proof_data must be the prover's Ed25519 signature over the
    // canonical credential statement (hash, nullifier, registry, prover,
    // purpose, disclosure) — verified via the runtime Ed25519 precompile.
    let statement = credential_proof_statement(
        &credential.credential_hash,
        &credential.nullifier_hash,
        &credential.region_registry,
        &credential.prover,
        &credential.purpose,
        credential.disclosure_type,
    );
    verify_precompiled_ed25519(
        &ctx.accounts.instructions,
        &credential.prover,
        &statement,
        &proof_data,
    )?;

    let now = Clock::get()?.unix_timestamp;

    nullifier.nullifier_hash = credential.nullifier_hash;
    nullifier.credential = credential.key();
    nullifier.consumed_at = now;

    credential.consumed = true;
    credential.version = credential.version.saturating_add(1);

    let proof_hash =
        solana_program::hash::hash(&[statement.as_slice(), proof_data.as_slice()].concat())
            .to_bytes();

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

    #[test]
    fn root_attestation_message_layout() {
        let msg = root_attestation_message(&[7u8; 32], 0x01020304);
        assert_eq!(&msg[..32], &[7u8; 32]);
        assert_eq!(&msg[32..], &[4, 3, 2, 1]);
    }

    #[test]
    fn statements_are_domain_separated_and_length_delimited() {
        let zone = Pubkey::new_unique();
        let prover = Pubkey::new_unique();
        let reg = Pubkey::new_unique();
        let own = ownership_proof_statement(&zone, &[1u8; 32], 3, &[2u8; 32], &prover, "a", 0);
        let cred = credential_proof_statement(&[1u8; 32], &[2u8; 32], &reg, &prover, "a", 0);
        assert_ne!(
            &own[..OWNERSHIP_PROOF_DOMAIN.len()],
            &cred[..CREDENTIAL_PROOF_DOMAIN.len()]
        );

        // purpose shifting must not move trailing fields ambiguously:
        // "ab"+[0] vs "a"+[0,?] cannot collide because purpose is length-prefixed.
        let a = ownership_proof_statement(&zone, &[1u8; 32], 3, &[2u8; 32], &prover, "a", 1);
        let ab = ownership_proof_statement(&zone, &[1u8; 32], 3, &[2u8; 32], &prover, "ab", 1);
        assert_ne!(a, ab);
        // Same inputs → same bytes (deterministic).
        assert_eq!(
            a,
            ownership_proof_statement(&zone, &[1u8; 32], 3, &[2u8; 32], &prover, "a", 1)
        );
    }

    #[test]
    fn ownership_statement_matches_groth16_circuit_encoding() {
        let zone = Pubkey::new_from_array([1u8; 32]);
        let root = [2u8; 32];
        let nullifier = [3u8; 32];
        let presenter = Pubkey::new_from_array([4u8; 32]);
        let purpose = "membership";
        let zone_bytes = zone.to_bytes();
        let presenter_bytes = presenter.to_bytes();
        let onchain = ownership_proof_statement(
            &zone,
            &root,
            7,
            &nullifier,
            &presenter,
            purpose,
            disclosure_type::MEMBERSHIP,
        );
        let circuit = terra_zk_membership::canonical_ownership_statement(
            &zone_bytes,
            &root,
            7,
            &nullifier,
            &presenter_bytes,
            purpose,
            disclosure_type::MEMBERSHIP,
        )
        .expect("valid circuit statement");
        assert_eq!(onchain, circuit);
    }

    #[test]
    fn ed25519_self_attestation_layout() {
        // Exact layout produced by solana-ed25519-program's
        // new_ed25519_instruction_with_signature (u16::MAX = "this ix").
        let mut data = vec![1u8, 0];
        data.extend_from_slice(&48u16.to_le_bytes()); // signature offset
        data.extend_from_slice(&u16::MAX.to_le_bytes()); // signature ix index
        data.extend_from_slice(&16u16.to_le_bytes()); // pubkey offset
        data.extend_from_slice(&u16::MAX.to_le_bytes()); // pubkey ix index
        data.extend_from_slice(&112u16.to_le_bytes()); // message offset
        data.extend_from_slice(&7u16.to_le_bytes()); // message size
        data.extend_from_slice(&u16::MAX.to_le_bytes()); // message ix index
        data.extend_from_slice(&[3u8; 32]); // pubkey
        data.extend_from_slice(&[4u8; 64]); // signature
        data.extend_from_slice(b"hello-!"); // message (7 bytes)

        let (pk, sig, msg) =
            parse_ed25519_self_attestation(&data, 0).expect("self-referencing layout parses");
        assert_eq!(pk, &[3u8; 32]);
        assert_eq!(sig, &[4u8; 64]);
        assert_eq!(msg, b"hello-!");

        // Explicit self index (instead of u16::MAX) also accepted.
        // Offset table bytes: [2..4]=sig_off, [4..6]=sig_ix, [6..8]=pk_off,
        // [8..10]=pk_ix, [10..12]=msg_off, [12..14]=msg_size, [14..16]=msg_ix.
        let mut data2 = data.clone();
        data2[8..10].copy_from_slice(&1u16.to_le_bytes()); // pubkey ix index = 1
        assert!(parse_ed25519_self_attestation(&data2, 1).is_some());

        // Cross-instruction references must be rejected: the runtime could
        // then verify bytes we did not inspect.
        let mut data3 = data.clone();
        data3[8..10].copy_from_slice(&0u16.to_le_bytes()); // pubkey ix index = 0 (self=1)
        assert!(parse_ed25519_self_attestation(&data3, 1).is_none());

        // Truncated / malformed data rejected.
        assert!(parse_ed25519_self_attestation(&data[..20], 0).is_none());
        let mut data4 = data.clone();
        data4[0] = 2; // multi-signature layout not parsed
        assert!(parse_ed25519_self_attestation(&data4, 0).is_none());
    }
}
