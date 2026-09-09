use anchor_lang::prelude::*;
use anchor_lang::solana_program::account_info::AccountInfo;

use crate::verification::session::{VerificationSession, session_status};
use crate::TerraError;

// ---------------------------------------------------------------------------
// Challenge status
// ---------------------------------------------------------------------------

pub mod challenge_status {
    pub const FILED: u8 = 0;
    pub const UNDER_REVIEW: u8 = 1;
    pub const UPHELD: u8 = 2;
    pub const OVERTURNED: u8 = 3;
    pub const MAX: u8 = OVERTURNED;
}

/// Challenge review window (14 days).
pub const CHALLENGE_REVIEW_SECS: i64 = 14 * 24 * 3600;

// ---------------------------------------------------------------------------
// Helper: look up VerificationSession PDA via remaining_accounts
// ---------------------------------------------------------------------------

fn try_load_session<'info>(
    remaining_accounts: &[AccountInfo<'info>],
    claim: &Pubkey,
) -> Result<Option<(Pubkey, VerificationSession)>> {
    for acc in remaining_accounts {
        let data = acc.try_borrow_data()?;
        // Skip accounts that are too small to be a valid Anchor account.
        if data.len() < 8 {
            continue;
        }
        // Attempt to deserialize as a VerificationSession — check discriminators.
        let account = VerificationSession::try_deserialize(&mut &data[..])?;
        if account.claim == *claim {
            return Ok(Some((acc.key(), account)));
        }
    }
    Ok(None)
}

// ---------------------------------------------------------------------------
// Challenge account
// ---------------------------------------------------------------------------

/// An on-chain challenge to a verified claim. Anyone may challenge a
/// verified claim by providing evidence. Validators review and vote.
///
/// PDA: `["challenge", claim, challenger]`
#[account]
#[derive(InitSpace)]
pub struct Challenge {
    /// The claim being challenged.
    pub claim: Pubkey,
    /// Who filed the challenge.
    pub challenger: Pubkey,
    /// SHA-256 of the off-chain challenge document / evidence.
    pub challenge_hash: [u8; 32],
    /// Current challenge status.
    pub status: u8,
    /// Number of validators who voted to uphold.
    pub uphold_votes: u8,
    /// Number of validators who voted to overturn.
    pub overturn_votes: u8,
    /// Required votes to resolve.
    pub required_votes: u8,
    /// Validators who voted (deduplication record).
    #[max_len(8)]
    pub voters: Vec<Pubkey>,
    pub created_at: i64,
    pub review_deadline: i64,
    pub resolved_at: i64,
}

// ---------------------------------------------------------------------------
// Instruction handlers
// ---------------------------------------------------------------------------

/// File a challenge against a verified claim.
///
/// If an active VerificationSession for this claim is provided via
/// remaining_accounts, it is transitioned to CHALLENGED status.
pub fn file_challenge(
    ctx: Context<crate::FileChallenge>,
    challenge_hash: [u8; 32],
    required_votes: u8,
) -> Result<()> {
    require!(
        !challenge_hash.iter().all(|b| *b == 0),
        TerraError::EmptyStatementHash
    );
    require!(required_votes >= 1, TerraError::InvalidThreshold);

    let now = Clock::get()?.unix_timestamp;
    let challenge = &mut ctx.accounts.challenge;
    challenge.claim = ctx.accounts.claim_account.key();
    challenge.challenger = ctx.accounts.challenger.key();
    challenge.challenge_hash = challenge_hash;
    challenge.status = challenge_status::FILED;
    challenge.uphold_votes = 0;
    challenge.overturn_votes = 0;
    challenge.required_votes = required_votes;
    challenge.voters = Vec::new();
    challenge.created_at = now;
    challenge.review_deadline = now.saturating_add(CHALLENGE_REVIEW_SECS);
    challenge.resolved_at = 0;

    let claim = &mut ctx.accounts.claim_account;
    require!(
        claim.status == crate::verification::claim::claim_status::VERIFIED,
        TerraError::InvalidClaimStatus
    );
    claim.status = crate::verification::claim::claim_status::CHALLENGED;
    claim.updated_at = now;

    // Close/transition any active VerificationSession for this claim.
    if let Ok(Some((session_key, mut session_account))) =
        try_load_session(ctx.remaining_accounts, &claim.key())
    {
        if !VerificationSession::is_terminal(session_account.status) {
            session_account.status = session_status::CHALLENGED;
            session_account.closed_at = now;
            session_account.updated_at = now;

            emit!(crate::VerificationSessionClosed {
                session: session_key,
                claim: session_account.claim,
                status: session_status::CHALLENGED,
                closed_at: now,
            });
        }
    }

    emit!(crate::ChallengeFiled {
        challenge: challenge.key(),
        claim: challenge.claim,
        challenger: challenge.challenger,
        challenge_hash,
        review_deadline: challenge.review_deadline,
    });
    Ok(())
}

/// Vote on a challenge. Only registered validators may vote.
///
/// When the challenge resolves (UPHELD or OVERTURNED), any active
/// VerificationSession for this claim is transitioned accordingly.
pub fn vote_challenge(
    ctx: Context<crate::VoteChallenge>,
    vote_uphold: bool,
) -> Result<()> {
    let challenge = &mut ctx.accounts.challenge;
    require!(
        challenge.status == challenge_status::FILED
            || challenge.status == challenge_status::UNDER_REVIEW,
        TerraError::InvalidClaimStatus
    );

    let now = Clock::get()?.unix_timestamp;
    require!(
        now <= challenge.review_deadline,
        TerraError::SettlementNotYetEffective
    );

    let validator = ctx.accounts.validator.key();
    let registry = &ctx.accounts.registry;
    require!(
        registry.validators.contains(&validator),
        TerraError::NotValidator
    );
    require!(
        !challenge.voters.contains(&validator),
        TerraError::AlreadyEndorsedRotation
    );

    challenge.voters.push(validator);

    if vote_uphold {
        challenge.uphold_votes = challenge.uphold_votes.saturating_add(1);
    } else {
        challenge.overturn_votes = challenge.overturn_votes.saturating_add(1);
    }

    if challenge.status == challenge_status::FILED {
        challenge.status = challenge_status::UNDER_REVIEW;
    }

    let claim_key = challenge.claim;
    let mut resolved = false;
    let mut final_outcome: u8 = 0;

    if challenge.uphold_votes >= challenge.required_votes {
        challenge.status = challenge_status::UPHELD;
        challenge.resolved_at = now;
        resolved = true;
        final_outcome = challenge_status::UPHELD;

        let claim = &mut ctx.accounts.claim_account;
        claim.status = crate::verification::claim::claim_status::REJECTED;
        claim.updated_at = now;

        emit!(crate::ChallengeResolved {
            challenge: challenge.key(),
            claim: challenge.claim,
            outcome: challenge_status::UPHELD,
            resolved_at: now,
        });
    } else if challenge.overturn_votes >= challenge.required_votes {
        challenge.status = challenge_status::OVERTURNED;
        challenge.resolved_at = now;
        resolved = true;
        final_outcome = challenge_status::OVERTURNED;

        let claim = &mut ctx.accounts.claim_account;
        claim.status = crate::verification::claim::claim_status::VERIFIED;
        claim.updated_at = now;

        emit!(crate::ChallengeResolved {
            challenge: challenge.key(),
            claim: challenge.claim,
            outcome: challenge_status::OVERTURNED,
            resolved_at: now,
        });
    }

    // On resolution, transition any active VerificationSession.
    if resolved {
        if let Ok(Some((session_key, mut session_account))) =
            try_load_session(ctx.remaining_accounts, &claim_key)
        {
            if !VerificationSession::is_terminal(session_account.status) {
                // UPHELD → CHALLENGED (session was for a verified claim that
                // is now rejected). OVERTURNED → QUORUM_REACHED (claim stays
                // verified).
                let session_final = if final_outcome == challenge_status::UPHELD {
                    session_status::CHALLENGED
                } else {
                    session_status::QUORUM_REACHED
                };
                session_account.status = session_final;
                session_account.closed_at = now;
                session_account.updated_at = now;

                emit!(crate::VerificationSessionClosed {
                    session: session_key,
                    claim: session_account.claim,
                    status: session_final,
                    closed_at: now,
                });
            }
        }
    }

    emit!(crate::ChallengeVoteRecorded {
        challenge: challenge.key(),
        validator,
        vote_uphold,
        uphold_votes: challenge.uphold_votes,
        overturn_votes: challenge.overturn_votes,
    });
    Ok(())
}
