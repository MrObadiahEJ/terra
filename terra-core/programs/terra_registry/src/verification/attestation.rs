use anchor_lang::prelude::*;
use anchor_lang::solana_program::account_info::AccountInfo;
use solana_program::hash::hashv;

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
// Helper: load the required ValidatorReputation PDA via remaining_accounts
// ---------------------------------------------------------------------------

/// Loads the validator's reputation account. The reputation PDA
/// (`["validator_reputation", validator]`) MUST be present in
/// `remaining_accounts`; otherwise the jail/slash gate could be bypassed by
/// simply omitting the account. Rejects accounts not owned by this program.
fn load_reputation<'info>(
    remaining_accounts: &[AccountInfo<'info>],
    validator: &Pubkey,
) -> Result<ValidatorReputation> {
    let (pda, _) = Pubkey::find_program_address(
        &[b"validator_reputation", validator.as_ref()],
        &crate::ID,
    );
    for acc in remaining_accounts {
        if acc.key == &pda {
            require!(
                acc.owner == &crate::ID,
                TerraError::MissingReputation
            );
            let data = acc.try_borrow_data()?;
            let account = ValidatorReputation::try_deserialize(&mut &data[..])?;
            return Ok(account);
        }
    }
    err!(TerraError::MissingReputation)
}

// ---------------------------------------------------------------------------
// Helper: canonical attestation digest (P0-6)
// ---------------------------------------------------------------------------

/// Domain-separated sha-256 over the attestation's content. Off-chain, the
/// validator signs this exact preimage; on-chain, the stored
/// `signature_hash` must equal it so the recorded attestation can never
/// differ from what the validator committed to (and so auditors can
/// recompute the digest from the account fields alone).
fn canonical_attestation_digest(
    claim: &Pubkey,
    validator: &Pubkey,
    observation: &Pubkey,
    result: u8,
    confidence: u8,
) -> [u8; 32] {
    hashv(&[
        b"terra:attestation:v1",
        claim.as_ref(),
        validator.as_ref(),
        observation.as_ref(),
        &[result],
        &[confidence],
    ])
    .to_bytes()
}

// ---------------------------------------------------------------------------
// Instruction handlers
// ---------------------------------------------------------------------------

/// Submit an attestation for a claim. Only validators who have submitted
/// an observation may attest. One attestation per validator per claim.
///
/// The validator's `ValidatorReputation` PDA MUST be supplied via
/// `remaining_accounts`. If it is missing, the transaction is rejected with
/// `MissingReputation`. If the validator is JAILED or SLASHED, the
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

    // P0-6: signature_hash must be the canonical digest over this
    // attestation's content — a tampered result/confidence/accounts cannot
    // reuse a digest signed over different content.
    let expected_digest = canonical_attestation_digest(
        &ctx.accounts.claim.key(),
        &ctx.accounts.validator.key(),
        &ctx.accounts.observation.key(),
        result,
        confidence,
    );
    require!(
        signature_hash == expected_digest,
        TerraError::AttestationDigestMismatch
    );

    // Mandatory reputation gating: reject jailed/slashed validators.
    let reputation = load_reputation(ctx.remaining_accounts, &ctx.accounts.validator.key())?;
    require!(
        reputation.status == validator_status::ACTIVE,
        TerraError::ValidatorJailed
    );

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
