use anchor_lang::prelude::*;

use crate::TerraError;

// ---------------------------------------------------------------------------
// Claim types
// ---------------------------------------------------------------------------

pub mod claim_type {
    pub const PARCEL_EXISTS: u8 = 0;
    pub const BOUNDARY: u8 = 1;
    pub const OWNERSHIP: u8 = 2;
    pub const USAGE: u8 = 3;
    pub const LEASE: u8 = 4;
    pub const EASEMENT: u8 = 5;
    pub const LIEN: u8 = 6;
    pub const MORTGAGE: u8 = 7;
    pub const OCCUPANCY: u8 = 8;
    pub const TRANSACTION: u8 = 9;
    pub const SUBDIVISION: u8 = 10;
    pub const AMALGAMATION: u8 = 11;
    pub const PROPERTY_ATTRIBUTE: u8 = 12;
    pub const INFRASTRUCTURE: u8 = 13;
    pub const MAX: u8 = INFRASTRUCTURE;
}

// ---------------------------------------------------------------------------
// Claim status
// ---------------------------------------------------------------------------

pub mod claim_status {
    pub const SUBMITTED: u8 = 0;
    pub const UNDER_VERIFICATION: u8 = 1;
    pub const VERIFIED: u8 = 2;
    pub const REJECTED: u8 = 3;
    pub const CHALLENGED: u8 = 4;
    pub const MAX: u8 = CHALLENGED;
}

// ---------------------------------------------------------------------------
// Claim account
// ---------------------------------------------------------------------------

#[account]
#[derive(InitSpace)]
pub struct Claim {
    /// Unique claim identifier (caller-provided, e.g. SHA-256 of the statement).
    pub claim_id: [u8; 32],
    /// The parcel this claim is about.
    pub parcel: Pubkey,
    /// Type of claim (see claim_type constants).
    pub claim_type: u8,
    /// Participant who submitted the claim.
    pub submitted_by: Pubkey,
    /// Current status (see claim_status constants).
    pub status: u8,
    /// sha-256 hash of the off-chain claim statement / document.
    pub statement_hash: [u8; 32],
    /// Monotonic version counter. Incremented on status transitions.
    pub version: u8,
    /// Number of evidence items attached to this claim.
    pub evidence_count: u8,
    /// Number of observations submitted for this claim.
    pub observation_count: u8,
    /// Number of attestations submitted for this claim.
    pub attestation_count: u8,
    /// Quorum threshold — number of attestations needed to verify.
    pub required_attestations: u8,
    pub created_at: i64,
    pub updated_at: i64,
}

// ---------------------------------------------------------------------------
// Instruction handlers
// ---------------------------------------------------------------------------

/// Create a new claim about a parcel. Anyone may submit a claim.
///
/// `required_attestations` is resolved from QuorumConfig (looked up via
/// remaining_accounts) or falls back to 2.
pub fn create_claim(
    ctx: Context<crate::CreateClaim>,
    claim_id: [u8; 32],
    claim_type: u8,
    statement_hash: [u8; 32],
    parcel_type: u8,
    region: [u8; 2],
) -> Result<()> {
    require!(
        !claim_id.iter().all(|b| *b == 0),
        TerraError::EmptyClaimId
    );
    require!(claim_type <= claim_type::MAX, TerraError::InvalidClaimType);
    require!(
        !statement_hash.iter().all(|b| *b == 0),
        TerraError::EmptyStatementHash
    );

    // Resolve quorum from QuorumConfig or fall back to defaults.
    let required_attestations = if let Some(config) =
        super::session::try_load_quorum_config(ctx.remaining_accounts, parcel_type, region)?
    {
        config.required_attestations
    } else {
        2
    };

    require!(
        required_attestations >= 1,
        TerraError::InvalidRequiredAttestations
    );

    let now = Clock::get()?.unix_timestamp;
    let claim = &mut ctx.accounts.claim;
    claim.claim_id = claim_id;
    claim.parcel = ctx.accounts.parcel.key();
    claim.claim_type = claim_type;
    claim.submitted_by = ctx.accounts.submitter.key();
    claim.status = claim_status::SUBMITTED;
    claim.statement_hash = statement_hash;
    claim.version = 1;
    claim.evidence_count = 0;
    claim.observation_count = 0;
    claim.attestation_count = 0;
    claim.required_attestations = required_attestations;
    claim.created_at = now;
    claim.updated_at = now;

    emit!(crate::ClaimCreated {
        claim: claim.key(),
        parcel: claim.parcel,
        claim_id,
        claim_type,
        submitted_by: claim.submitted_by,
        statement_hash,
    });
    Ok(())
}

/// Verify a claim once enough confirmatory attestations have been submitted.
/// Anyone may call this — it is a read-then-write quorum check.
pub fn verify_claim(ctx: Context<crate::VerifyClaim>) -> Result<()> {
    let claim = &mut ctx.accounts.claim;
    require!(
        claim.status == crate::verification::claim::claim_status::SUBMITTED
            || claim.status == crate::verification::claim::claim_status::UNDER_VERIFICATION,
        TerraError::InvalidClaimStatus
    );
    require!(
        claim.attestation_count >= claim.required_attestations,
        TerraError::QuorumNotReached
    );

    let now = Clock::get()?.unix_timestamp;
    claim.status = claim_status::VERIFIED;
    claim.version = claim.version.saturating_add(1);
    claim.updated_at = now;

    emit!(crate::ClaimVerified {
        claim: claim.key(),
        parcel: claim.parcel,
        claim_type: claim.claim_type,
        attestation_count: claim.attestation_count,
        verified_at: now,
    });
    Ok(())
}
