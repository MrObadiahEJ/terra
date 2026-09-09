use anchor_lang::prelude::*;
use anchor_lang::solana_program::account_info::AccountInfo;

use crate::verification::reputation::{ValidatorReputation, validator_status, MAX_REPUTATION};
use crate::TerraError;

// ---------------------------------------------------------------------------
// Quorum vote choice
// ---------------------------------------------------------------------------

pub mod quorum_vote_choice {
    pub const CONFIRM: u8 = 0;
    pub const DISPUTE: u8 = 1;
    pub const ABSTAIN: u8 = 2;
    pub const MAX: u8 = ABSTAIN;
}

// ---------------------------------------------------------------------------
// QuorumVote account
// ---------------------------------------------------------------------------

/// A weighted vote cast by a validator on a claim.
///
/// PDA: `["quorum_vote", claim, voter]`
#[account]
#[derive(InitSpace)]
pub struct QuorumVote {
    /// The claim being voted on.
    pub claim: Pubkey,
    /// The validator casting the vote.
    pub voter: Pubkey,
    /// Vote choice (0=confirm, 1=dispute, 2=abstain).
    pub vote: u8,
    /// Reputation-based weight at time of vote (min(reputation_score, 10000)).
    pub weight: u16,
    /// SOL staked at time of vote (if applicable, 0 otherwise).
    pub stake_amount: u64,
    pub created_at: i64,
}

// ---------------------------------------------------------------------------
// QuorumTally account
// ---------------------------------------------------------------------------

/// Aggregated quorum tally for a claim.
///
/// PDA: `["quorum_tally", claim]`
#[account]
#[derive(InitSpace)]
pub struct QuorumTally {
    /// The claim this tally is for.
    pub claim: Pubkey,
    /// Total weight of all votes.
    pub total_weight: u16,
    /// Weight of confirm votes.
    pub confirm_weight: u16,
    /// Weight of dispute votes.
    pub dispute_weight: u16,
    /// Weight of abstain votes.
    pub abstain_weight: u16,
    /// Total number of votes cast.
    pub total_votes: u8,
    /// Required weight to pass quorum.
    pub quorum_threshold: u16,
    /// Whether quorum has been resolved.
    pub resolved: bool,
    pub created_at: i64,
    pub updated_at: i64,
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

/// Validator casts a weighted vote on a claim. Weight = min(reputation_score,
/// 10000). Looks up ValidatorReputation via remaining_accounts. Updates tally.
/// If confirm_weight >= quorum_threshold, marks tally as resolved.
///
/// One vote per validator per claim (enforced by PDA uniqueness).
pub fn cast_quorum_vote(
    ctx: Context<crate::CastQuorumVote>,
    vote_choice: u8,
) -> Result<()> {
    require!(
        vote_choice <= quorum_vote_choice::MAX,
        TerraError::InvalidQuorumVoteChoice
    );

    // Reputation gating: reject jailed/slashed validators.
    let voter_key = ctx.accounts.voter.key();
    if let Some(reputation) =
        try_load_reputation(ctx.remaining_accounts, &voter_key)?
    {
        require!(
            reputation.status == validator_status::ACTIVE,
            TerraError::ValidatorJailed
        );
    }

    let now = Clock::get()?.unix_timestamp;

    // Determine weight from ValidatorReputation or default to MAX_REPUTATION.
    let weight = if let Some(reputation) =
        try_load_reputation(ctx.remaining_accounts, &voter_key)?
    {
        reputation.reputation_score.min(MAX_REPUTATION)
    } else {
        MAX_REPUTATION
    };

    let vote = &mut ctx.accounts.quorum_vote;
    vote.claim = ctx.accounts.claim.key();
    vote.voter = voter_key;
    vote.vote = vote_choice;
    vote.weight = weight;
    vote.stake_amount = 0; // caller can extend with staking lookup if needed
    vote.created_at = now;

    let tally = &mut ctx.accounts.tally;
    require!(!tally.resolved, TerraError::QuorumAlreadyResolved);

    // Initialize tally on first vote if not yet initialized.
    if tally.total_votes == 0 {
        tally.claim = ctx.accounts.claim.key();
        tally.quorum_threshold = ctx.accounts.claim.required_attestations as u16 * MAX_REPUTATION / 100;
        if tally.quorum_threshold == 0 {
            tally.quorum_threshold = 1;
        }
        tally.created_at = now;
    }

    tally.total_weight = tally.total_weight.saturating_add(weight);
    tally.total_votes = tally.total_votes.saturating_add(1);

    match vote_choice {
        quorum_vote_choice::CONFIRM => {
            tally.confirm_weight = tally.confirm_weight.saturating_add(weight);
        }
        quorum_vote_choice::DISPUTE => {
            tally.dispute_weight = tally.dispute_weight.saturating_add(weight);
        }
        quorum_vote_choice::ABSTAIN => {
            tally.abstain_weight = tally.abstain_weight.saturating_add(weight);
        }
        _ => {}
    }

    // Auto-resolve when confirm weight meets quorum.
    if tally.confirm_weight >= tally.quorum_threshold && !tally.resolved {
        tally.resolved = true;

        emit!(crate::QuorumReached {
            claim: tally.claim,
            confirm_weight: tally.confirm_weight,
            quorum_threshold: tally.quorum_threshold,
            total_votes: tally.total_votes,
            resolved_at: now,
        });
    }

    tally.updated_at = now;

    emit!(crate::QuorumVoteCast {
        claim: vote.claim,
        voter: vote.voter,
        vote: vote.vote,
        weight: vote.weight,
        confirm_weight: tally.confirm_weight,
        dispute_weight: tally.dispute_weight,
        total_votes: tally.total_votes,
    });
    Ok(())
}

/// Finalize the quorum. If resolved (confirm_weight >= quorum_threshold),
/// update the Claim status to VERIFIED. If dispute_weight exceeds confirm_weight,
/// update the Claim status to REJECTED.
pub fn finalize_quorum(
    ctx: Context<crate::FinalizeQuorum>,
) -> Result<()> {
    let tally = &mut ctx.accounts.tally;
    require!(
        tally.resolved || tally.dispute_weight > tally.confirm_weight,
        TerraError::QuorumNotReached
    );

    let now = Clock::get()?.unix_timestamp;
    let claim = &mut ctx.accounts.claim;

    require!(
        claim.status == crate::verification::claim::claim_status::SUBMITTED
            || claim.status == crate::verification::claim::claim_status::UNDER_VERIFICATION,
        TerraError::InvalidClaimStatus
    );

    if tally.confirm_weight >= tally.quorum_threshold {
        claim.status = crate::verification::claim::claim_status::VERIFIED;
        claim.version = claim.version.saturating_add(1);
        claim.updated_at = now;

        emit!(crate::ClaimVerified {
            claim: claim.key(),
            parcel: claim.parcel,
            claim_type: claim.claim_type,
            attestation_count: claim.attestation_count,
            verified_at: now,
        });
    } else if tally.dispute_weight > tally.confirm_weight {
        claim.status = crate::verification::claim::claim_status::REJECTED;
        claim.version = claim.version.saturating_add(1);
        claim.updated_at = now;

        emit!(crate::ClaimRejected {
            claim: claim.key(),
            reason: 1, // quorum dispute
        });
    }

    Ok(())
}
