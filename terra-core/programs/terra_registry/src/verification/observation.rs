use anchor_lang::prelude::*;

use crate::TerraError;

// ---------------------------------------------------------------------------
// Observation account
// ---------------------------------------------------------------------------

#[account]
#[derive(InitSpace)]
pub struct Observation {
    /// The claim this observation is about.
    pub claim: Pubkey,
    /// Validator who made this observation.
    pub validator: Pubkey,
    /// When the observation was made.
    pub observed_at: i64,
    /// Location of observation (encoded as latitude/longitude i64 pair, or 0,0).
    pub location: [i64; 2],
    /// Method used for observation (e.g. 0=visual, 1=GPS, 2=survey, 3=document review).
    pub method: u8,
    /// Free-form findings hash (sha-256 of detailed findings document).
    pub findings_hash: [u8; 32],
    /// Confidence level (0-100).
    pub confidence: u8,
    /// sha-256 of the signed observation payload (for signature verification).
    pub signature_hash: [u8; 32],
    pub created_at: i64,
}

// ---------------------------------------------------------------------------
// Instruction handlers
// ---------------------------------------------------------------------------

/// Submit an observation for a claim. Any signer may observe (validators
/// are expected to call this, but the protocol does not enforce role
/// at this layer — reputation and quorum handle trust).
pub fn submit_observation(
    ctx: Context<crate::SubmitObservation>,
    location: [i64; 2],
    method: u8,
    findings_hash: [u8; 32],
    confidence: u8,
    signature_hash: [u8; 32],
) -> Result<()> {
    require!(confidence <= 100, TerraError::InvalidConfidence);

    let claim = &mut ctx.accounts.claim;
    require!(
        claim.status == crate::verification::claim::claim_status::SUBMITTED
            || claim.status == crate::verification::claim::claim_status::UNDER_VERIFICATION,
        TerraError::InvalidClaimStatus
    );

    let now = Clock::get()?.unix_timestamp;
    let observation = &mut ctx.accounts.observation;
    observation.claim = claim.key();
    observation.validator = ctx.accounts.validator.key();
    observation.observed_at = now;
    observation.location = location;
    observation.method = method;
    observation.findings_hash = findings_hash;
    observation.confidence = confidence;
    observation.signature_hash = signature_hash;
    observation.created_at = now;

    // Transition claim to UNDER_VERIFICATION on first observation.
    if claim.status == crate::verification::claim::claim_status::SUBMITTED {
        claim.status = super::claim::claim_status::UNDER_VERIFICATION;
    }
    claim.observation_count = claim.observation_count.saturating_add(1);
    claim.updated_at = now;

    emit!(crate::ObservationSubmitted {
        claim: claim.key(),
        observation: observation.key(),
        validator: observation.validator,
        method,
        confidence,
    });
    Ok(())
}
