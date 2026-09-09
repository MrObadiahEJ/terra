use anchor_lang::prelude::*;
use anchor_lang::solana_program::account_info::AccountInfo;

use crate::verification::reputation::{ValidatorReputation, validator_status};
use crate::TerraError;

// ---------------------------------------------------------------------------
// Verification attestation
// ---------------------------------------------------------------------------

/// Result of a validator's attestation on a claim.
pub mod attestation_result {
    pub const CONFIRMED: u8 = 0;
    pub const DISPUTED: u8 = 1;
    pub const UNABLE_TO_VERIFY: u8 = 2;
    pub const MAX: u8 = UNABLE_TO_VERIFY;
}

#[account]
#[derive(InitSpace)]
pub struct VerificationAttestation {
    /// The claim being attested.
    pub claim: Pubkey,
    /// Validator submitting this attestation.
    pub validator: Pubkey,
    /// The observation this attestation is based on.
    pub observation: Pubkey,
    /// Attestation result (confirmed, disputed, unable_to_verify).
    pub result: u8,
    /// Confidence level (0-100).
    pub confidence: u8,
    /// sha-256 of the signed attestation payload.
    pub signature_hash: [u8; 32],
    pub created_at: i64,
    /// Protocol version for audit trail.
    pub protocol_version: u8,
}

// ---------------------------------------------------------------------------
// Helper: look up ValidatorReputation PDA via remaining_accounts
// ---------------------------------------------------------------------------

fn try_load_reputation<'info>(
    remaining_accounts: &[AccountInfo<'info>],
    validator: &Pubkey,
) -> Result<Option<ValidatorReputation>> {
    let (pda, _) = Pubkey::find_program_address(
        &[b"validator_reputation", validator.as_ref()],
        &crate::ID,
    );
    for acc in remaining_accounts {
        if acc.key == &pda {
            let data = acc.try_borrow_data()?;
            let account = ValidatorReputation::try_deserialize(&mut &data[..])?;
            return Ok(Some(account));
        }
    }
    Ok(None)
}

// ---------------------------------------------------------------------------
// Instruction handlers
// ---------------------------------------------------------------------------

/// Submit an attestation for a claim. Only validators who have submitted
/// an observation may attest. One attestation per validator per claim.
///
/// If the validator's ValidatorReputation PDA is provided via
/// remaining_accounts and the validator is JAILED or SLASHED, the
/// transaction is rejected with `ValidatorJailed`.
pub fn submit_attestation(
    ctx: Context<crate::SubmitVerificationAttestation>,
    result: u8,
    confidence: u8,
    signature_hash: [u8; 32],
) -> Result<()> {
    require!(
        result <= attestation_result::MAX,
        TerraError::InvalidAttestationResult
    );
    require!(confidence <= 100, TerraError::InvalidConfidence);

    // Reputation gating: reject jailed/slashed validators.
    if let Some(reputation) =
        try_load_reputation(ctx.remaining_accounts, &ctx.accounts.validator.key())?
    {
        require!(
            reputation.status == validator_status::ACTIVE,
            TerraError::ValidatorJailed
        );
    }

    let claim = &mut ctx.accounts.claim;
    require!(
        claim.status == crate::verification::claim::claim_status::SUBMITTED
            || claim.status == crate::verification::claim::claim_status::UNDER_VERIFICATION,
        TerraError::InvalidClaimStatus
    );

    let now = Clock::get()?.unix_timestamp;
    let attestation = &mut ctx.accounts.attestation;
    attestation.claim = claim.key();
    attestation.validator = ctx.accounts.validator.key();
    attestation.observation = ctx.accounts.observation.key();
    attestation.result = result;
    attestation.confidence = confidence;
    attestation.signature_hash = signature_hash;
    attestation.created_at = now;
    attestation.protocol_version = 1;

    // Only count CONFIRMED attestations toward quorum.
    if result == attestation_result::CONFIRMED {
        claim.attestation_count = claim.attestation_count.saturating_add(1);
    }
    claim.updated_at = now;

    emit!(crate::VerificationAttestationSubmitted {
        claim: claim.key(),
        attestation: attestation.key(),
        validator: attestation.validator,
        result,
        confidence,
        attestation_count: claim.attestation_count,
        required: claim.required_attestations,
    });
    Ok(())
}
