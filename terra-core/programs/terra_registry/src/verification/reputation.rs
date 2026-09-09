use anchor_lang::prelude::*;

use crate::TerraError;

// ---------------------------------------------------------------------------
// Validator reputation status
// ---------------------------------------------------------------------------

pub mod validator_status {
    pub const ACTIVE: u8 = 0;
    pub const JAILED: u8 = 1;
    pub const SLASHED: u8 = 2;
    pub const MAX: u8 = SLASHED;
}

/// Maximum reputation score (basis points, 10000 = perfect).
pub const MAX_REPUTATION: u16 = 10_000;

/// Jail duration for misconduct (7 days).
pub const JAIL_DURATION_SECS: i64 = 7 * 24 * 3600;

// ---------------------------------------------------------------------------
// ValidatorReputation account
// ---------------------------------------------------------------------------

/// Tracks an individual validator's reputation and performance metrics.
///
/// PDA: `["validator_reputation", validator]`
#[account]
#[derive(InitSpace)]
pub struct ValidatorReputation {
    /// The validator's pubkey.
    pub validator: Pubkey,
    /// Current status (ACTIVE, JAILED, SLASHED).
    pub status: u8,
    /// Total attestations submitted.
    pub total_attestations: u64,
    /// Attestations that were confirmed by quorum.
    pub confirmed_attestations: u64,
    /// Attestations that were disputed/challenged.
    pub disputed_attestations: u64,
    /// Reputation score in basis points (0 = slashed, 10000 = perfect).
    pub reputation_score: u16,
    /// Timestamp when the validator was jailed (0 if not jailed).
    pub jailed_until: i64,
    /// Total number of successful challenges against this validator.
    pub challenges_received: u8,
    /// Total number of challenges raised by this validator.
    pub challenges_raised: u8,
    pub created_at: i64,
    pub updated_at: i64,
}

impl ValidatorReputation {
    /// Compute reputation score based on performance.
    /// Formula: confirmed / total * MAX_REPUTATION, with penalties for disputes.
    pub fn compute_reputation(&self) -> u16 {
        if self.total_attestations == 0 {
            return MAX_REPUTATION; // new validators start at perfect
        }
        let base = (self.confirmed_attestations as u16)
            .saturating_mul(MAX_REPUTATION)
            .checked_div(self.total_attestations as u16)
            .unwrap_or(0);

        // Penalty: each dispute costs 500 bps (5%), floor at 0.
        let penalty = (self.disputed_attestations as u16).saturating_mul(500);
        base.saturating_sub(penalty)
    }
}

// ---------------------------------------------------------------------------
// Instruction handlers
// ---------------------------------------------------------------------------

/// Initialize reputation tracking for a validator. Called automatically
/// when a validator is first added to the registry, or can be called
/// explicitly.
pub fn initialize_validator_reputation(
    ctx: Context<crate::InitializeValidatorReputation>,
) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let rep = &mut ctx.accounts.reputation;
    rep.validator = ctx.accounts.validator.key();
    rep.status = validator_status::ACTIVE;
    rep.total_attestations = 0;
    rep.confirmed_attestations = 0;
    rep.disputed_attestations = 0;
    rep.reputation_score = MAX_REPUTATION;
    rep.jailed_until = 0;
    rep.challenges_received = 0;
    rep.challenges_raised = 0;
    rep.created_at = now;
    rep.updated_at = now;

    emit!(crate::ValidatorReputationInitialized {
        validator: rep.validator,
        initialized_at: now,
    });
    Ok(())
}

/// Record an attestation outcome for a validator. Called after a claim
/// is verified or rejected to update the validator's reputation.
pub fn record_attestation_outcome(
    ctx: Context<crate::RecordAttestationOutcome>,
    confirmed: bool,
) -> Result<()> {
    let rep = &mut ctx.accounts.reputation;
    require!(
        rep.status == validator_status::ACTIVE,
        TerraError::ValidatorNotActive
    );

    rep.total_attestations = rep.total_attestations.saturating_add(1);
    if confirmed {
        rep.confirmed_attestations = rep.confirmed_attestations.saturating_add(1);
    } else {
        rep.disputed_attestations = rep.disputed_attestations.saturating_add(1);
    }
    rep.reputation_score = rep.compute_reputation();
    rep.updated_at = Clock::get()?.unix_timestamp;

    // Auto-jail if reputation drops below 2000 bps (20%).
    if rep.reputation_score < 2000 {
        rep.status = validator_status::JAILED;
        rep.jailed_until = Clock::get()?.unix_timestamp.saturating_add(JAIL_DURATION_SECS);

        emit!(crate::ValidatorJailed {
            validator: rep.validator,
            reputation_score: rep.reputation_score,
            jailed_until: rep.jailed_until,
        });
    }

    Ok(())
}

/// Jail a validator for misconduct. Only callable through dispute resolution
/// or by the registry admin.
pub fn jail_validator(ctx: Context<crate::JailValidator>, duration_secs: i64) -> Result<()> {
    let rep = &mut ctx.accounts.reputation;
    require!(
        rep.status == validator_status::ACTIVE,
        TerraError::ValidatorNotActive
    );

    let now = Clock::get()?.unix_timestamp;
    rep.status = validator_status::JAILED;
    rep.jailed_until = now.saturating_add(duration_secs);
    rep.updated_at = now;

    emit!(crate::ValidatorJailed {
        validator: rep.validator,
        reputation_score: rep.reputation_score,
        jailed_until: rep.jailed_until,
    });
    Ok(())
}

/// Unjail a validator after the jail period has elapsed.
pub fn unjail_validator(ctx: Context<crate::UnjailValidator>) -> Result<()> {
    let rep = &mut ctx.accounts.reputation;
    require!(
        rep.status == validator_status::JAILED,
        TerraError::InvalidClaimStatus
    );

    let now = Clock::get()?.unix_timestamp;
    require!(now >= rep.jailed_until, TerraError::SettlementNotYetEffective);

    rep.status = validator_status::ACTIVE;
    rep.jailed_until = 0;
    rep.updated_at = now;

    emit!(crate::ValidatorUnjailed {
        validator: rep.validator,
        unjailed_at: now,
    });
    Ok(())
}

/// Slash a validator's reputation. Called through dispute resolution.
pub fn slash_validator(ctx: Context<crate::SlashValidator>, reputation_penalty: u16) -> Result<()> {
    let rep = &mut ctx.accounts.reputation;
    require!(
        rep.status == validator_status::ACTIVE,
        TerraError::ValidatorNotActive
    );

    let new_score = rep.reputation_score.saturating_sub(reputation_penalty);
    rep.reputation_score = new_score;
    rep.updated_at = Clock::get()?.unix_timestamp;

    if new_score == 0 {
        rep.status = validator_status::SLASHED;

        emit!(crate::ValidatorSlashed {
            validator: rep.validator,
            reputation_score: 0,
            slashed_at: rep.updated_at,
        });
    } else if new_score < 2000 {
        rep.status = validator_status::JAILED;
        rep.jailed_until = Clock::get()?.unix_timestamp.saturating_add(JAIL_DURATION_SECS);

        emit!(crate::ValidatorJailed {
            validator: rep.validator,
            reputation_score: new_score,
            jailed_until: rep.jailed_until,
        });
    }

    Ok(())
}
