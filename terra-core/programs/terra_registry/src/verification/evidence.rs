use anchor_lang::prelude::*;

use crate::TerraError;

// ---------------------------------------------------------------------------
// Evidence types
// ---------------------------------------------------------------------------

pub mod evidence_type {
    pub const PERSON_SUBMISSION: u8 = 0;
    pub const PHOTO: u8 = 1;
    pub const VIDEO: u8 = 2;
    pub const DOCUMENT: u8 = 3;
    pub const SURVEY: u8 = 4;
    pub const GPS_OBSERVATION: u8 = 5;
    pub const PHYSICAL_OBSERVATION: u8 = 6;
    pub const WITNESS_STATEMENT: u8 = 7;
    pub const TRANSACTION_RECORD: u8 = 8;
    pub const PROPERTY_RECORD: u8 = 9;
    pub const MAP: u8 = 10;
    pub const SATELLITE_OBSERVATION: u8 = 11;
    pub const SIGNATURE: u8 = 12;
    pub const MAX: u8 = SIGNATURE;
}

// ---------------------------------------------------------------------------
// Evidence account
// ---------------------------------------------------------------------------

#[account]
#[derive(InitSpace)]
pub struct Evidence {
    /// The claim this evidence supports.
    pub claim: Pubkey,
    /// Monotonic nonce within the claim (0, 1, 2, ...).
    pub nonce: u8,
    /// Participant who submitted this evidence.
    pub submitted_by: Pubkey,
    /// Type of evidence (see evidence_type constants).
    pub evidence_type: u8,
    /// sha-256 hash of the off-chain evidence content.
    pub content_hash: [u8; 32],
    /// Storage reference (IPFS CID, URL, or content address).
    #[max_len(128)]
    pub storage_reference: String,
    pub created_at: i64,
    /// When the evidence was originally observed (off-chain), if applicable.
    pub observed_at: i64,
}

// ---------------------------------------------------------------------------
// Instruction handlers
// ---------------------------------------------------------------------------

/// Attach evidence to a claim. Only the claim submitter may add evidence.
pub fn add_evidence(
    ctx: Context<crate::AddEvidence>,
    evidence_type: u8,
    content_hash: [u8; 32],
    storage_reference: String,
    observed_at: i64,
) -> Result<()> {
    require!(
        evidence_type <= evidence_type::MAX,
        TerraError::InvalidEvidenceType
    );
    require!(
        !content_hash.iter().all(|b| *b == 0),
        TerraError::EmptyContentHash
    );
    require!(
        !storage_reference.is_empty(),
        TerraError::EmptyStorageReference
    );
    require!(
        storage_reference.len() <= 128,
        TerraError::NotesTooLong
    );

    let claim = &ctx.accounts.claim;
    require!(
        ctx.accounts.submitter.key() == claim.submitted_by,
        TerraError::NotClaimSubmitter
    );
    require!(
        claim.status == crate::verification::claim::claim_status::SUBMITTED
            || claim.status == crate::verification::claim::claim_status::UNDER_VERIFICATION,
        TerraError::InvalidClaimStatus
    );

    let now = Clock::get()?.unix_timestamp;
    let evidence = &mut ctx.accounts.evidence;
    evidence.claim = claim.key();
    evidence.nonce = claim.evidence_count;
    evidence.submitted_by = ctx.accounts.submitter.key();
    evidence.evidence_type = evidence_type;
    evidence.content_hash = content_hash;
    evidence.storage_reference = storage_reference;
    evidence.created_at = now;
    evidence.observed_at = observed_at;

    // Bump claim evidence counter.
    let claim = &mut ctx.accounts.claim;
    claim.evidence_count = claim.evidence_count.saturating_add(1);
    claim.updated_at = now;

    emit!(crate::EvidenceAdded {
        claim: claim.key(),
        evidence: evidence.key(),
        nonce: evidence.nonce,
        evidence_type,
        submitted_by: evidence.submitted_by,
        content_hash,
    });
    Ok(())
}
