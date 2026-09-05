use anchor_lang::prelude::*;

use crate::TerraError;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// 7-day unbonding period (seconds).
pub const UNBONDING_PERIOD_SECS: i64 = 7 * 24 * 3600;
/// 30-day maximum unbonding period.
pub const MAX_UNBONDING_PERIOD_SECS: i64 = 30 * 24 * 3600;
/// 7-day appeal window (seconds).
pub const APPEAL_WINDOW_SECS: i64 = 7 * 24 * 3600;
/// 7-day liveness threshold.
pub const LIVENESS_THRESHOLD_SECS: i64 = 7 * 24 * 3600;
/// 10% slash for first offense (basis points).
pub const FIRST_OFFENSE_SLASH_BPS: u16 = 1000;
/// 100% slash for repeat offense (basis points).
pub const REPEAT_OFFENSE_SLASH_BPS: u16 = 10000;
/// Minimum stake: 1 SOL in lamports.
pub const MIN_STAKE_LAMPORTS: u64 = 1_000_000_000;
/// Maximum validators per pool.
pub const MAX_VALIDATORS_PER_POOL: usize = 8;
/// Reporter bond: 1% of potential slash (basis points).
pub const REPORTER_BOND_BPS: u16 = 100;
/// Minimum interval between reward distributions (1 day).
pub const REWARD_DISTRIBUTION_INTERVAL_SECS: i64 = 86400;
/// 24-hour review period before slash can execute.
pub const REVIEW_PERIOD_SECS: i64 = 24 * 3600;
/// Maximum appeal reason length.
pub const MAX_APPEAL_REASON_LEN: usize = 256;

// ---------------------------------------------------------------------------
// Slashing report status
// ---------------------------------------------------------------------------

pub mod report_status {
    pub const PENDING: u8 = 0;
    pub const VERIFIED: u8 = 1;
    pub const SLASHED: u8 = 2;
    pub const APPEALED: u8 = 3;
    pub const REJECTED: u8 = 4;
    pub const DISMISSED: u8 = 5;
}

// ---------------------------------------------------------------------------
// Offense types
// ---------------------------------------------------------------------------

pub mod offense_type {
    pub const EQUIVOCATION: u8 = 0;
    pub const LIVENESS: u8 = 1;
    pub const COLLUSION: u8 = 2;
}

// ---------------------------------------------------------------------------
// Accounts
// ---------------------------------------------------------------------------

/// One stake pool per region, derived from the AuthorityRegistry key.
///
/// PDA seed: `["stake_pool", region_registry_key]`.
#[account]
#[derive(InitSpace)]
pub struct StakePool {
    /// AuthorityRegistry key for this region.
    pub region_registry: Pubkey,
    /// Total SOL staked across all validators (lamports).
    pub total_staked: u64,
    /// Annual reward rate in basis points (e.g., 500 = 5%).
    pub reward_rate_bps: u16,
    /// Total rewards accrued but not yet distributed (lamports).
    pub accumulated_rewards: u64,
    /// Timestamp of last reward distribution.
    pub last_reward_distribution: i64,
    /// Total slashing events executed.
    pub slash_count: u32,
    pub created_at: i64,
    pub updated_at: i64,
}

/// One record per validator per region.
///
/// PDA seed: `["validator_stake", stake_pool_key, validator_key]`.
#[account]
#[derive(InitSpace)]
pub struct ValidatorStake {
    /// StakePool key.
    pub stake_pool: Pubkey,
    /// Validator's Ed25519 pubkey.
    pub validator: Pubkey,
    /// Current stake in lamports.
    pub staked_amount: u64,
    /// Amount in unbonding period.
    pub unbonding_amount: u64,
    /// When unbonding began (0 if not unbonding).
    pub unbonding_starts_at: i64,
    /// Rewards accumulated but not yet claimed.
    pub rewards_accrued: u64,
    /// Number of past slashing events (for graduated severity).
    pub slash_history: u8,
    /// Recent offense flags: [equivocation, liveness, collusion, unused].
    pub offenses: [u8; 4],
    pub created_at: i64,
    pub updated_at: i64,
}

/// One record per slashing report.
///
/// PDA seed: `["slashing_report", stake_pool_key, reporter_key, evidence_hash]`.
#[account]
#[derive(InitSpace)]
pub struct SlashingReport {
    /// StakePool key.
    pub stake_pool: Pubkey,
    /// Wallet that filed the report.
    pub reporter: Pubkey,
    /// SHA-256 of the evidence payload.
    pub evidence_hash: [u8; 32],
    /// Validator accused of misbehavior.
    pub offender: Pubkey,
    /// 0=equivocation, 1=liveness, 2=collusion.
    pub offense_type: u8,
    /// Bounded details (e.g., two conflicting parcel hashes).
    pub offense_details: [u8; 64],
    /// SOL bonded by reporter (for false-report penalty).
    pub reporter_bond: u64,
    /// 0=Pending, 1=Verified, 2=Slashed, 3=Appealed, 4=Rejected, 5=Dismissed.
    pub status: u8,
    /// When the report was filed.
    pub filed_at: i64,
    /// filed_at + APPEAL_WINDOW_SECS.
    pub appeal_deadline: i64,
    /// When the report was resolved (0 if pending).
    pub resolved_at: i64,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// Initialize a stake pool for a region.
pub fn create_stake_pool(ctx: Context<super::CreateStakePool>, reward_rate_bps: u16) -> Result<()> {
    require!(
        reward_rate_bps > 0 && reward_rate_bps <= 2000,
        TerraError::InvalidStatus
    );

    let registry = &ctx.accounts.region_registry;
    require!(
        registry.validators.iter().any(|v| *v != Pubkey::default()),
        TerraError::NoValidators
    );

    let now = Clock::get()?.unix_timestamp;
    let pool = &mut ctx.accounts.stake_pool;
    pool.region_registry = ctx.accounts.region_registry.key();
    pool.total_staked = 0;
    pool.reward_rate_bps = reward_rate_bps;
    pool.accumulated_rewards = 0;
    pool.last_reward_distribution = now;
    pool.slash_count = 0;
    pool.created_at = now;
    pool.updated_at = now;

    emit!(StakePoolCreated {
        stake_pool: pool.key(),
        region_registry: pool.region_registry,
        reward_rate_bps,
    });
    Ok(())
}

/// A validator deposits SOL as a bond. Top-ups are allowed while no
/// unbonding is in progress; a fresh record is initialized on first deposit.
pub fn deposit_stake(ctx: Context<super::DepositStake>, amount: u64) -> Result<()> {
    require!(amount >= MIN_STAKE_LAMPORTS, TerraError::InsufficientStake);

    // Verify validator is in the registry.
    let registry = &ctx.accounts.region_registry;
    let validator_key = ctx.accounts.validator.key();
    require!(
        registry.validators.contains(&validator_key),
        TerraError::NotValidator
    );

    let stake = &mut ctx.accounts.validator_stake;
    require!(stake.unbonding_amount == 0, TerraError::UnbondingInProgress);

    // Transfer SOL from validator to stake pool.
    let ix = anchor_lang::solana_program::system_instruction::transfer(
        &ctx.accounts.validator.key(),
        &ctx.accounts.stake_pool.key(),
        amount,
    );
    anchor_lang::solana_program::program::invoke(
        &ix,
        &[
            ctx.accounts.validator.to_account_info(),
            ctx.accounts.stake_pool.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
        ],
    )?;

    let now = Clock::get()?.unix_timestamp;
    if stake.created_at == 0 {
        stake.stake_pool = ctx.accounts.stake_pool.key();
        stake.validator = validator_key;
        stake.created_at = now;
    }
    stake.staked_amount = stake.staked_amount.saturating_add(amount);
    stake.updated_at = now;

    let pool = &mut ctx.accounts.stake_pool;
    pool.total_staked = pool.total_staked.saturating_add(amount);
    pool.updated_at = now;

    emit!(StakeDeposited {
        stake_pool: pool.key(),
        validator: validator_key,
        amount,
    });
    Ok(())
}

/// Validator begins the unbonding process.
pub fn initiate_unbonding(ctx: Context<super::InitiateUnbonding>) -> Result<()> {
    let stake = &mut ctx.accounts.validator_stake;
    require!(stake.unbonding_amount == 0, TerraError::UnbondingInProgress);
    require!(stake.staked_amount > 0, TerraError::InsufficientStake);

    let now = Clock::get()?.unix_timestamp;
    stake.unbonding_amount = stake.staked_amount;
    stake.staked_amount = 0;
    stake.unbonding_starts_at = now;
    stake.updated_at = now;

    emit!(UnbondingInitiated {
        stake_pool: stake.stake_pool,
        validator: stake.validator,
        amount: stake.unbonding_amount,
    });
    Ok(())
}

/// Validator withdraws stake after the unbonding period has elapsed.
/// Single-use: the unbonding record is zeroed so one unbonding can only be
/// withdrawn once.
pub fn withdraw_stake(ctx: Context<super::WithdrawStake>) -> Result<()> {
    let (amount, validator_key, pool_key);
    {
        let stake = &ctx.accounts.validator_stake;
        require!(stake.unbonding_amount > 0, TerraError::InsufficientStake);
        require!(stake.staked_amount == 0, TerraError::UnbondingInProgress);

        let now = Clock::get()?.unix_timestamp;
        require!(
            now >= stake.unbonding_starts_at + UNBONDING_PERIOD_SECS,
            TerraError::UnbondingNotComplete
        );
        amount = stake.unbonding_amount;
        validator_key = stake.validator;
        pool_key = stake.stake_pool;
    }

    // Transfer SOL from stake pool to validator.
    **ctx
        .accounts
        .stake_pool
        .to_account_info()
        .try_borrow_mut_lamports()? -= amount;
    **ctx
        .accounts
        .validator
        .to_account_info()
        .try_borrow_mut_lamports()? += amount;

    {
        let stake = &mut ctx.accounts.validator_stake;
        stake.unbonding_amount = 0;
        stake.unbonding_starts_at = 0;
        stake.updated_at = Clock::get()?.unix_timestamp;
    }

    let pool = &mut ctx.accounts.stake_pool;
    pool.total_staked = pool.total_staked.saturating_sub(amount);
    pool.updated_at = Clock::get()?.unix_timestamp;

    emit!(StakeWithdrawn {
        stake_pool: pool_key,
        validator: validator_key,
        amount,
    });
    Ok(())
}

/// Report a validator for signing two conflicting attestations.
pub fn report_equivocation(
    ctx: Context<super::ReportEquivocation>,
    evidence_hash: [u8; 32],
    offense_details: [u8; 64],
) -> Result<()> {
    require!(
        !evidence_hash.iter().all(|b| *b == 0),
        TerraError::EmptyGeometryHash
    );

    let offender_stake = &ctx.accounts.offender_stake;
    require!(
        offender_stake.staked_amount > 0,
        TerraError::InsufficientStake
    );

    let reporter_key = ctx.accounts.reporter.key();
    require!(
        reporter_key != offender_stake.validator,
        TerraError::SelfReportNotAllowed
    );

    // Compute required bond.
    let potential_slash = offender_stake.staked_amount;
    let required_bond = potential_slash
        .checked_mul(REPORTER_BOND_BPS as u64)
        .ok_or(TerraError::RightsLimitExceeded)?
        .checked_div(10000)
        .ok_or(TerraError::RightsLimitExceeded)?;

    // Collect the reporter bond up front: the bond lives in the report PDA
    // (which the reporter funds here) and is returned on slash/dismiss.
    if required_bond > 0 {
        let ix = anchor_lang::solana_program::system_instruction::transfer(
            &reporter_key,
            &ctx.accounts.slashing_report.key(),
            required_bond,
        );
        anchor_lang::solana_program::program::invoke(
            &ix,
            &[
                ctx.accounts.reporter.to_account_info(),
                ctx.accounts.slashing_report.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;
    }

    let now = Clock::get()?.unix_timestamp;

    let report = &mut ctx.accounts.slashing_report;
    report.stake_pool = ctx.accounts.stake_pool.key();
    report.reporter = reporter_key;
    report.evidence_hash = evidence_hash;
    report.offender = offender_stake.validator;
    report.offense_type = offense_type::EQUIVOCATION;
    report.offense_details = offense_details;
    report.reporter_bond = required_bond;
    report.status = report_status::PENDING;
    report.filed_at = now;
    report.appeal_deadline = now + APPEAL_WINDOW_SECS;
    report.resolved_at = 0;

    emit!(EquivocationReported {
        stake_pool: report.stake_pool,
        reporter: reporter_key,
        offender: report.offender,
        evidence_hash,
    });
    Ok(())
}

/// Report a validator for liveness or collusion offenses (or equivocation
/// with explicit typing). Mirrors `report_equivocation` but takes the
/// offense type as an argument and flags it on the offender's record.
pub fn report_validator_offense(
    ctx: Context<super::ReportOffense>,
    offense_kind: u8,
    evidence_hash: [u8; 32],
    offense_details: [u8; 64],
) -> Result<()> {
    require!(
        offense_kind <= offense_type::COLLUSION,
        TerraError::InvalidOffenseType
    );
    require!(
        !evidence_hash.iter().all(|b| *b == 0),
        TerraError::EmptyGeometryHash
    );

    let reporter_key = ctx.accounts.reporter.key();
    let offender_key;
    let required_bond;
    {
        let offender_stake = &ctx.accounts.offender_stake;
        require!(
            offender_stake.staked_amount > 0,
            TerraError::InsufficientStake
        );
        require!(
            reporter_key != offender_stake.validator,
            TerraError::SelfReportNotAllowed
        );
        offender_key = offender_stake.validator;
        required_bond = offender_stake
            .staked_amount
            .checked_mul(REPORTER_BOND_BPS as u64)
            .ok_or(TerraError::RightsLimitExceeded)?
            .checked_div(10000)
            .ok_or(TerraError::RightsLimitExceeded)?;
    }

    if required_bond > 0 {
        let ix = anchor_lang::solana_program::system_instruction::transfer(
            &reporter_key,
            &ctx.accounts.slashing_report.key(),
            required_bond,
        );
        anchor_lang::solana_program::program::invoke(
            &ix,
            &[
                ctx.accounts.reporter.to_account_info(),
                ctx.accounts.slashing_report.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;
    }

    let now = Clock::get()?.unix_timestamp;

    {
        let offender_stake = &mut ctx.accounts.offender_stake;
        offender_stake.offenses[offense_kind as usize] =
            offender_stake.offenses[offense_kind as usize].saturating_add(1);
        offender_stake.updated_at = now;
    }

    let report = &mut ctx.accounts.slashing_report;
    report.stake_pool = ctx.accounts.stake_pool.key();
    report.reporter = reporter_key;
    report.evidence_hash = evidence_hash;
    report.offender = offender_key;
    report.offense_type = offense_kind;
    report.offense_details = offense_details;
    report.reporter_bond = required_bond;
    report.status = report_status::PENDING;
    report.filed_at = now;
    report.appeal_deadline = now + APPEAL_WINDOW_SECS;
    report.resolved_at = 0;

    emit!(EquivocationReported {
        stake_pool: report.stake_pool,
        reporter: reporter_key,
        offender: report.offender,
        evidence_hash,
    });
    Ok(())
}

/// Execute slashing after evidence review.
pub fn verify_and_slash(ctx: Context<super::VerifyAndSlash>) -> Result<()> {
    let reporter_bond;
    let reporter_key;
    let stake_pool_key;
    {
        let report = &ctx.accounts.slashing_report;
        require!(
            report.status == report_status::PENDING,
            TerraError::InvalidDisputeStatus
        );

        let now = Clock::get()?.unix_timestamp;
        require!(
            now >= report.filed_at + REVIEW_PERIOD_SECS,
            TerraError::SettlementNotYetEffective
        );

        reporter_bond = report.reporter_bond;
        reporter_key = report.reporter;
        stake_pool_key = report.stake_pool;
    }

    let now = Clock::get()?.unix_timestamp;

    let stake = &mut ctx.accounts.offender_stake;
    let pool = &mut ctx.accounts.stake_pool;

    // Determine slash percentage.
    let slash_bps = if stake.slash_history == 0 {
        FIRST_OFFENSE_SLASH_BPS
    } else {
        REPEAT_OFFENSE_SLASH_BPS
    };

    let slash_amount = stake
        .staked_amount
        .checked_mul(slash_bps as u64)
        .ok_or(TerraError::RightsLimitExceeded)?
        .checked_div(10000)
        .ok_or(TerraError::RightsLimitExceeded)?;

    let offender_key = stake.validator;

    stake.staked_amount = stake.staked_amount.saturating_sub(slash_amount);
    pool.total_staked = pool.total_staked.saturating_sub(slash_amount);
    stake.slash_history = stake.slash_history.saturating_add(1);
    pool.slash_count = pool.slash_count.saturating_add(1);

    // Move slashed lamports out of the pool into the treasury so bookkeeping
    // and real balances cannot diverge ("zombie lamports").
    if slash_amount > 0 {
        **ctx
            .accounts
            .stake_pool
            .to_account_info()
            .try_borrow_mut_lamports()? -= slash_amount;
        **ctx
            .accounts
            .treasury
            .to_account_info()
            .try_borrow_mut_lamports()? += slash_amount;
    }

    // Update report status.
    {
        let report = &mut ctx.accounts.slashing_report;
        report.status = report_status::SLASHED;
        report.resolved_at = now;
    }

    // Return bond to reporter.
    if reporter_bond > 0 {
        **ctx
            .accounts
            .slashing_report
            .to_account_info()
            .try_borrow_mut_lamports()? -= reporter_bond;
        **ctx
            .accounts
            .reporter
            .to_account_info()
            .try_borrow_mut_lamports()? += reporter_bond;
    }

    emit!(ValidatorSlashed {
        stake_pool: stake_pool_key,
        offender: offender_key,
        slash_amount,
        slash_bps,
        reporter: reporter_key,
    });
    Ok(())
}

/// Validator claims accumulated rewards.
pub fn claim_rewards(ctx: Context<super::ClaimRewards>) -> Result<()> {
    let stake = &ctx.accounts.validator_stake;
    require!(stake.rewards_accrued > 0, TerraError::InsufficientStake);
    require!(stake.staked_amount > 0, TerraError::InsufficientStake);

    let amount = stake.rewards_accrued;

    // Transfer rewards from pool to validator.
    **ctx
        .accounts
        .stake_pool
        .to_account_info()
        .try_borrow_mut_lamports()? -= amount;
    **ctx
        .accounts
        .validator
        .to_account_info()
        .try_borrow_mut_lamports()? += amount;

    let stake = &mut ctx.accounts.validator_stake;
    stake.rewards_accrued = 0;
    stake.updated_at = Clock::get()?.unix_timestamp;

    let pool = &mut ctx.accounts.stake_pool;
    pool.accumulated_rewards = pool.accumulated_rewards.saturating_sub(amount);
    pool.updated_at = Clock::get()?.unix_timestamp;

    emit!(RewardsClaimed {
        stake_pool: pool.key(),
        validator: stake.validator,
        amount,
    });
    Ok(())
}

/// Distribute accumulated rewards proportionally to all staked validators.
pub fn distribute_rewards(ctx: Context<super::DistributeRewards>) -> Result<()> {
    let pool = &ctx.accounts.stake_pool;
    require!(pool.total_staked > 0, TerraError::InsufficientStake);

    let now = Clock::get()?.unix_timestamp;
    require!(
        now >= pool.last_reward_distribution + REWARD_DISTRIBUTION_INTERVAL_SECS,
        TerraError::SettlementNotYetEffective
    );

    let time_delta = now - pool.last_reward_distribution;
    let annual_reward = (pool.total_staked as u128)
        .checked_mul(pool.reward_rate_bps as u128)
        .ok_or(TerraError::RightsLimitExceeded)?
        .checked_div(10000)
        .ok_or(TerraError::RightsLimitExceeded)?;
    let period_reward = annual_reward
        .checked_mul(time_delta as u128)
        .ok_or(TerraError::RightsLimitExceeded)?
        .checked_div((365 * 24 * 3600) as u128)
        .ok_or(TerraError::RightsLimitExceeded)? as u64;

    // Rewards are backed by real SOL: the treasury (admin-funded) pays the
    // period reward into the pool. Without this, claims would be paid out of
    // other validators' deposits.
    if period_reward > 0 {
        **ctx
            .accounts
            .treasury
            .to_account_info()
            .try_borrow_mut_lamports()? -= period_reward;
        **ctx
            .accounts
            .stake_pool
            .to_account_info()
            .try_borrow_mut_lamports()? += period_reward;
    }

    let pool = &mut ctx.accounts.stake_pool;
    pool.accumulated_rewards = pool.accumulated_rewards.saturating_add(period_reward);
    pool.last_reward_distribution = now;
    pool.updated_at = now;

    emit!(RewardsDistributed {
        stake_pool: pool.key(),
        period_reward,
        total_staked: pool.total_staked,
    });
    Ok(())
}

/// Validator disputes a slashing report during the appeal window.
pub fn dispute_slashing(ctx: Context<super::DisputeSlashing>, appeal_reason: String) -> Result<()> {
    require!(
        appeal_reason.len() <= MAX_APPEAL_REASON_LEN,
        TerraError::NotesTooLong
    );

    let report = &mut ctx.accounts.slashing_report;
    require!(
        report.status == report_status::PENDING,
        TerraError::InvalidDisputeStatus
    );

    let now = Clock::get()?.unix_timestamp;
    require!(
        now < report.appeal_deadline,
        TerraError::CancelWindowExpired
    );
    require!(
        report.offender == ctx.accounts.offender.key(),
        TerraError::NotDesignatedBuyer
    );

    report.status = report_status::APPEALED;
    report.resolved_at = 0;

    emit!(SlashingAppealed {
        stake_pool: report.stake_pool,
        offender: report.offender,
        appeal_reason,
    });
    Ok(())
}

/// Admin dismisses a slashing report (evidence insufficient or false).
pub fn dismiss_report(ctx: Context<super::DismissReport>) -> Result<()> {
    let reporter_bond;
    let reporter_key;
    let stake_pool_key;
    let offender_key;
    {
        let report = &ctx.accounts.slashing_report;
        require!(
            report.status == report_status::PENDING || report.status == report_status::APPEALED,
            TerraError::InvalidDisputeStatus
        );
        reporter_bond = report.reporter_bond;
        reporter_key = report.reporter;
        stake_pool_key = report.stake_pool;
        offender_key = report.offender;
    }

    let now = Clock::get()?.unix_timestamp;
    {
        let report = &mut ctx.accounts.slashing_report;
        report.status = report_status::DISMISSED;
        report.resolved_at = now;
    }

    // Return bond to reporter.
    if reporter_bond > 0 {
        **ctx
            .accounts
            .slashing_report
            .to_account_info()
            .try_borrow_mut_lamports()? -= reporter_bond;
        **ctx
            .accounts
            .reporter
            .to_account_info()
            .try_borrow_mut_lamports()? += reporter_bond;
    }

    emit!(ReportDismissed {
        stake_pool: stake_pool_key,
        reporter: reporter_key,
        offender: offender_key,
    });
    Ok(())
}

// ---------------------------------------------------------------------------
// Events
// ---------------------------------------------------------------------------

#[event]
pub struct StakePoolCreated {
    pub stake_pool: Pubkey,
    pub region_registry: Pubkey,
    pub reward_rate_bps: u16,
}

#[event]
pub struct StakeDeposited {
    pub stake_pool: Pubkey,
    pub validator: Pubkey,
    pub amount: u64,
}

#[event]
pub struct UnbondingInitiated {
    pub stake_pool: Pubkey,
    pub validator: Pubkey,
    pub amount: u64,
}

#[event]
pub struct StakeWithdrawn {
    pub stake_pool: Pubkey,
    pub validator: Pubkey,
    pub amount: u64,
}

#[event]
pub struct EquivocationReported {
    pub stake_pool: Pubkey,
    pub reporter: Pubkey,
    pub offender: Pubkey,
    pub evidence_hash: [u8; 32],
}

#[event]
pub struct ValidatorSlashed {
    pub stake_pool: Pubkey,
    pub offender: Pubkey,
    pub slash_amount: u64,
    pub slash_bps: u16,
    pub reporter: Pubkey,
}

#[event]
pub struct RewardsClaimed {
    pub stake_pool: Pubkey,
    pub validator: Pubkey,
    pub amount: u64,
}

#[event]
pub struct RewardsDistributed {
    pub stake_pool: Pubkey,
    pub period_reward: u64,
    pub total_staked: u64,
}

#[event]
pub struct SlashingAppealed {
    pub stake_pool: Pubkey,
    pub offender: Pubkey,
    pub appeal_reason: String,
}

#[event]
pub struct ReportDismissed {
    pub stake_pool: Pubkey,
    pub reporter: Pubkey,
    pub offender: Pubkey,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unbonding_period_is_7_days() {
        assert_eq!(UNBONDING_PERIOD_SECS, 7 * 24 * 3600);
    }

    #[test]
    fn appeal_window_is_7_days() {
        assert_eq!(APPEAL_WINDOW_SECS, 7 * 24 * 3600);
    }

    #[test]
    fn first_offense_slash_is_10_percent() {
        assert_eq!(FIRST_OFFENSE_SLASH_BPS, 1000);
    }

    #[test]
    fn repeat_offense_slash_is_100_percent() {
        assert_eq!(REPEAT_OFFENSE_SLASH_BPS, 10000);
    }

    #[test]
    fn min_stake_is_1_sol() {
        assert_eq!(MIN_STAKE_LAMPORTS, 1_000_000_000);
    }

    #[test]
    fn reporter_bond_is_1_percent() {
        assert_eq!(REPORTER_BOND_BPS, 100);
    }

    #[test]
    fn report_status_values_are_contiguous() {
        assert_eq!(report_status::PENDING, 0);
        assert_eq!(report_status::VERIFIED, 1);
        assert_eq!(report_status::SLASHED, 2);
        assert_eq!(report_status::APPEALED, 3);
        assert_eq!(report_status::REJECTED, 4);
        assert_eq!(report_status::DISMISSED, 5);
    }

    #[test]
    fn offense_type_values_are_contiguous() {
        assert_eq!(offense_type::EQUIVOCATION, 0);
        assert_eq!(offense_type::LIVENESS, 1);
        assert_eq!(offense_type::COLLUSION, 2);
    }
}
