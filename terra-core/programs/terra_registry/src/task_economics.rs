use anchor_lang::prelude::*;

use crate::verification_task;
use crate::TerraError;

// ---------------------------------------------------------------------------
// RFC-012 Phase 8 — economic / resource layer
//
// Flow (RFC-012 §7 steps 13–15):
//   quote (resource cost) → fund escrow (reward + resources + protocol fee)
//   → task completes → claim reward (+ coverage subsidy from treasury)
//   → refund leftovers (cancel / expiry / claim-window close).
//
// Entities (RFC-012 §5 Economics):
//   FeePolicy           protocol fee rules (admin-set; bootstrap authority)
//   CoverageIncentive   demand + coverage-deficit + difficulty + strategic
//                       importance subsidy params (admin-set in Phase 8)
//   ResourceQuote       estimated resource cost of a task (requester)
//   TaskEscrow          locked funds for a task   (PDA ["task_escrow", task_id])
//   RewardAllocation    per-validator distribution record
//
// Migration map (§9): parcel-only `EscrowRecord` stays for parcel sales;
// `TaskEscrow` generalizes escrow to any verification-task subject.
//
// Authority: fee/coverage params are registry-admin-set during bootstrap;
// path to PEER_CONSENSUS (RFC-012 §2) is a later governance step.
// ---------------------------------------------------------------------------

/// Max protocol fee in basis points (10%).
pub const MAX_FEE_BPS: u16 = 1_000;
/// Max per-factor coverage contribution in basis points.
pub const MAX_COVERAGE_FACTOR_BPS: u16 = 5_000;
/// Max total coverage subsidy in basis points (50% of a validator share).
pub const MAX_SUBSIDY_BPS: u16 = 5_000;
/// Difficulty factor cap: required_validators is 1..=8 but subsidy difficulty
/// contribution scales only up to 4× difficulty_bps.
pub const DIFFICULTY_FACTOR_CAP: u8 = 4;
/// After task completion, unclaimed rewards become refundable to the
/// requester this many seconds later.
pub const CLAIM_WINDOW_SECS: i64 = 7 * 24 * 3600;

pub mod task_escrow_status {
    /// 0 = fresh / uninitialized (init_if_needed memory is zeroed).
    pub const FUNDED: u8 = 1;
    pub const RELEASED: u8 = 2;
    pub const REFUNDED: u8 = 3;
    pub const MAX: u8 = REFUNDED;
}

// ---------------------------------------------------------------------------
// Accounts (PDA seeds per RFC-012 §10)
// ---------------------------------------------------------------------------

/// Global protocol fee rules.
///
/// PDA: `["fee_policy"]`
#[account]
#[derive(InitSpace)]
pub struct FeePolicy {
    /// Wallet allowed to update (registry admin during bootstrap).
    pub authority: Pubkey,
    /// Protocol fee in bps, charged on top of every task escrow funding.
    pub fee_bps: u16,
    pub updated_at: i64,
}

/// Global coverage-incentive parameters.
///
/// PDA: `["coverage_incentive"]`
#[account]
#[derive(InitSpace)]
pub struct CoverageIncentive {
    pub authority: Pubkey,
    /// Baseline demand contribution (bps).
    pub demand_bps: u16,
    /// Coverage-deficit contribution (bps).
    pub deficit_bps: u16,
    /// Base difficulty contribution; scaled ×required_validators (cap 4).
    pub difficulty_bps: u16,
    /// Strategic-importance contribution for strategic task classes.
    pub strategic_bps: u16,
    /// Hard cap applied after summing all contributions.
    pub max_subsidy_bps: u16,
    pub updated_at: i64,
}

/// Requester's estimated resource cost for a task (travel, materials, …).
///
/// PDA: `["resource_quote", task_id]`
#[account]
#[derive(InitSpace)]
pub struct ResourceQuote {
    pub task_id: [u8; 32],
    /// Task account this quote belongs to.
    pub task: Pubkey,
    /// Requester wallet that quoted.
    pub quoter: Pubkey,
    /// Estimated resource cost in lamports (funded alongside the reward).
    pub resource_cost: u64,
    pub created_at: i64,
}

/// Locked funds for a verification task.
///
/// PDA: `["task_escrow", task_id]` — vault PDA: `["task_escrow_vault", escrow]`
#[account]
#[derive(InitSpace)]
pub struct TaskEscrow {
    pub task_id: [u8; 32],
    pub task: Pubkey,
    pub requester: Pubkey,
    /// Vault PDA holding the funded lamports.
    pub vault: Pubkey,
    /// Total funded for distribution (reward + resource cost).
    pub amount: u64,
    /// Protocol fee paid to the treasury at funding time.
    pub fee_paid: u64,
    /// Base rewards paid out so far.
    pub released: u64,
    /// How many validator shares have been claimed.
    pub released_count: u8,
    /// One of `task_escrow_status`.
    pub status: u8,
    pub created_at: i64,
    pub funded_at: i64,
    pub updated_at: i64,
}

/// Distribution record for one validator's claim on a task escrow.
///
/// PDA: `["reward_allocation", task_id, validator]`
#[account]
#[derive(InitSpace)]
pub struct RewardAllocation {
    pub task_id: [u8; 32],
    pub task: Pubkey,
    pub validator: Pubkey,
    /// Base share paid from the escrow vault.
    pub base_paid: u64,
    /// Coverage subsidy paid from the treasury.
    pub subsidy_paid: u64,
    pub total_paid: u64,
    pub created_at: i64,
}

// ---------------------------------------------------------------------------
// Pure helpers (unit-tested)
// ---------------------------------------------------------------------------

pub fn is_valid_fee_bps(fee_bps: u16) -> bool {
    fee_bps <= MAX_FEE_BPS
}

pub fn is_valid_coverage_params(
    demand_bps: u16,
    deficit_bps: u16,
    difficulty_bps: u16,
    strategic_bps: u16,
    max_subsidy_bps: u16,
) -> bool {
    demand_bps <= MAX_COVERAGE_FACTOR_BPS
        && deficit_bps <= MAX_COVERAGE_FACTOR_BPS
        && difficulty_bps <= MAX_COVERAGE_FACTOR_BPS
        && strategic_bps <= MAX_COVERAGE_FACTOR_BPS
        && max_subsidy_bps <= MAX_SUBSIDY_BPS
}

/// Protocol fee for `amount` at `fee_bps` (u128 math, floored).
pub fn fee_lamports(amount: u64, fee_bps: u16) -> u64 {
    (((amount as u128) * (fee_bps as u128)) / 10_000u128) as u64
}

/// Strategic-importance task classes (field / document / mixed work).
pub fn is_strategic_class(task_class: u8) -> bool {
    matches!(
        task_class,
        verification_task::task_class::PHYSICAL
            | verification_task::task_class::DOCUMENTARY
            | verification_task::task_class::HYBRID
    )
}

/// Total coverage subsidy in bps for a task, clamped to `max_subsidy_bps`.
///
///   bps = demand + deficit + difficulty × min(required_validators, 4)
///         + (strategic if strategic class)
///   subsidy = min(bps, max_subsidy_bps)
pub fn subsidy_bps_for(policy: &CoverageIncentive, task_class: u8, required_validators: u8) -> u16 {
    let mut bps = policy.demand_bps as u32 + policy.deficit_bps as u32;
    let factor = if required_validators > DIFFICULTY_FACTOR_CAP {
        DIFFICULTY_FACTOR_CAP
    } else if required_validators == 0 {
        1
    } else {
        required_validators
    };
    bps += policy.difficulty_bps as u32 * factor as u32;
    if is_strategic_class(task_class) {
        bps += policy.strategic_bps as u32;
    }
    let cap = policy.max_subsidy_bps as u32;
    if bps > cap {
        cap as u16
    } else {
        bps as u16
    }
}

/// Coverage subsidy in lamports for a base share.
pub fn subsidy_lamports(base_share: u64, subsidy_bps: u16) -> u64 {
    (((base_share as u128) * (subsidy_bps as u128)) / 10_000u128) as u64
}

/// Even division that hands the remainder to the last claimer:
/// `next_share(amount - released, required - released_count)`.
pub fn next_share(amount_remaining: u64, shares_remaining: u8) -> u64 {
    if shares_remaining == 0 {
        0
    } else {
        amount_remaining / shares_remaining as u64
    }
}

/// Whether `refund_task_escrow` may release the remaining vault balance.
#[derive(Debug, PartialEq, Eq)]
pub enum RefundVerdict {
    /// Refund allowed (cancelled / expired / window closed / all claimed).
    Allow,
    /// Task completed but the claim window is still open and shares remain.
    WindowOpen,
    /// Task is still active — cancel it first.
    NotTerminal,
}

pub fn refund_verdict(
    task_status: u8,
    now: i64,
    completed_at: i64,
    released_count: u8,
    required_validators: u8,
) -> RefundVerdict {
    if task_status == verification_task::task_status::CANCELLED
        || task_status == verification_task::task_status::EXPIRED
    {
        return RefundVerdict::Allow;
    }
    if task_status == verification_task::task_status::COMPLETED {
        if released_count >= required_validators {
            return RefundVerdict::Allow;
        }
        if now > completed_at + CLAIM_WINDOW_SECS {
            return RefundVerdict::Allow;
        }
        return RefundVerdict::WindowOpen;
    }
    RefundVerdict::NotTerminal
}

/// Manually deserialize a config PDA (missing/unowned → caller's custom error).
fn deser_policy<T: AccountDeserialize>(info: &AccountInfo, err: TerraError) -> Result<T> {
    if *info.owner != crate::ID || info.data_is_empty() {
        return Err(error!(err));
    }
    let mut data: &[u8] = &info.data.borrow();
    T::try_deserialize(&mut data).map_err(|_| error!(err))
}

// ---------------------------------------------------------------------------
// Instruction handlers
// ---------------------------------------------------------------------------

/// Set (or update) the global protocol fee. Registry admin only.
pub fn set_fee_policy(ctx: Context<crate::SetFeePolicy>, fee_bps: u16) -> Result<()> {
    require!(is_valid_fee_bps(fee_bps), TerraError::InvalidFeeBps);
    let now = Clock::get()?.unix_timestamp;
    let p = &mut ctx.accounts.fee_policy;
    p.authority = ctx.accounts.admin.key();
    p.fee_bps = fee_bps;
    p.updated_at = now;
    emit!(crate::FeePolicySet {
        authority: p.authority,
        fee_bps,
    });
    Ok(())
}

/// Set (or update) global coverage-incentive parameters. Registry admin only.
#[allow(clippy::too_many_arguments)]
pub fn set_coverage_incentive(
    ctx: Context<crate::SetCoverageIncentive>,
    demand_bps: u16,
    deficit_bps: u16,
    difficulty_bps: u16,
    strategic_bps: u16,
    max_subsidy_bps: u16,
) -> Result<()> {
    require!(
        is_valid_coverage_params(
            demand_bps,
            deficit_bps,
            difficulty_bps,
            strategic_bps,
            max_subsidy_bps
        ),
        TerraError::InvalidCoverageParams
    );
    let now = Clock::get()?.unix_timestamp;
    let p = &mut ctx.accounts.coverage_incentive;
    p.authority = ctx.accounts.admin.key();
    p.demand_bps = demand_bps;
    p.deficit_bps = deficit_bps;
    p.difficulty_bps = difficulty_bps;
    p.strategic_bps = strategic_bps;
    p.max_subsidy_bps = max_subsidy_bps;
    p.updated_at = now;
    emit!(crate::CoverageIncentiveSet {
        authority: p.authority,
        demand_bps,
        deficit_bps,
        difficulty_bps,
        strategic_bps,
        max_subsidy_bps,
    });
    Ok(())
}

/// Requester records the estimated resource cost for their task (once).
pub fn quote_task_resources(
    ctx: Context<crate::QuoteTaskResources>,
    resource_cost: u64,
) -> Result<()> {
    require!(
        ctx.accounts.quote.created_at == 0,
        TerraError::QuoteAlreadyExists
    );
    require!(
        !verification_task::task_is_terminal(ctx.accounts.task.status),
        TerraError::TaskAlreadyFinalized
    );
    let now = Clock::get()?.unix_timestamp;
    let task_key = ctx.accounts.task.key();
    let q = &mut ctx.accounts.quote;
    q.task_id = ctx.accounts.task.task_id;
    q.task = task_key;
    q.quoter = ctx.accounts.quoter.key();
    q.resource_cost = resource_cost;
    q.created_at = now;
    emit!(crate::TaskResourcesQuoted {
        task: task_key,
        task_id: q.task_id,
        quoter: q.quoter,
        resource_cost,
    });
    Ok(())
}

/// Requester funds the task escrow: reward + resource cost into the vault,
/// protocol fee (FeePolicy.bps of the total) into the treasury.
pub fn fund_task_escrow(ctx: Context<crate::FundTaskEscrow>) -> Result<()> {
    require!(
        !verification_task::task_is_terminal(ctx.accounts.task.status),
        TerraError::TaskAlreadyFinalized
    );
    require!(
        ctx.accounts.task_escrow.created_at == 0,
        TerraError::EscrowAlreadyFunded
    );
    let quote: ResourceQuote = deser_policy(
        &ctx.accounts.quote.to_account_info(),
        TerraError::ResourceQuoteMissing,
    )?;
    require_keys_eq!(
        quote.task,
        ctx.accounts.task.key(),
        TerraError::ResourceQuoteMissing
    );
    let fee_policy: FeePolicy = deser_policy(
        &ctx.accounts.fee_policy.to_account_info(),
        TerraError::FeePolicyNotInitialized,
    )?;

    let amount = ctx
        .accounts
        .task
        .reward_lamports
        .checked_add(quote.resource_cost)
        .ok_or(error!(TerraError::InvalidEscrowAmount))?;
    // The vault must stay rent-exempt through every partial claim, so the
    // pool has to cover one rent-minimum per required validator.
    let rent_min = Rent::get()?.minimum_balance(0);
    let required = ctx.accounts.task.required_validators;
    require!(
        amount >= rent_min.saturating_mul(required as u64),
        TerraError::InvalidEscrowAmount
    );
    let mut fee = fee_lamports(amount, fee_policy.fee_bps);
    // Waive sub-rent-exempt dust fees when the treasury is still empty —
    // a 0-lamports PDA cannot be created with a dust balance.
    if fee > 0 && ctx.accounts.treasury.to_account_info().lamports() == 0 && fee < rent_min {
        fee = 0;
    }

    let now = Clock::get()?.unix_timestamp;
    let requester = ctx.accounts.requester.key();
    let task_key = ctx.accounts.task.key();
    let task_id = ctx.accounts.task.task_id;
    let vault_key = ctx.accounts.vault.key();
    let treasury_key = ctx.accounts.treasury.key();

    // Requester → vault (distribution pool).
    let vault_ix =
        anchor_lang::solana_program::system_instruction::transfer(&requester, &vault_key, amount);
    anchor_lang::solana_program::program::invoke(
        &vault_ix,
        &[
            ctx.accounts.requester.to_account_info(),
            ctx.accounts.vault.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
        ],
    )?;

    // Requester → treasury (protocol fee).
    if fee > 0 {
        let fee_ix = anchor_lang::solana_program::system_instruction::transfer(
            &requester,
            &treasury_key,
            fee,
        );
        anchor_lang::solana_program::program::invoke(
            &fee_ix,
            &[
                ctx.accounts.requester.to_account_info(),
                ctx.accounts.treasury.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;
    }

    let escrow = &mut ctx.accounts.task_escrow;
    escrow.task_id = task_id;
    escrow.task = task_key;
    escrow.requester = requester;
    escrow.vault = vault_key;
    escrow.amount = amount;
    escrow.fee_paid = fee;
    escrow.released = 0;
    escrow.released_count = 0;
    escrow.status = task_escrow_status::FUNDED;
    escrow.created_at = now;
    escrow.funded_at = now;
    escrow.updated_at = now;

    emit!(crate::TaskEscrowFunded {
        escrow: escrow.key(),
        task: task_key,
        task_id,
        requester,
        amount,
        fee_paid: fee,
    });
    Ok(())
}

/// Assigned validator claims their share of a COMPLETED task's escrow,
/// plus a coverage subsidy from the treasury (capped at treasury balance).
pub fn claim_task_reward(ctx: Context<crate::ClaimTaskReward>) -> Result<()> {
    // Order matters: double-claim must report RewardAlreadyClaimed even
    // though the first claim also flipped assignment/escrow status.
    require!(
        ctx.accounts.task.status == verification_task::task_status::COMPLETED,
        TerraError::TaskNotCompleted
    );
    require!(
        ctx.accounts.reward_allocation.created_at == 0,
        TerraError::RewardAlreadyClaimed
    );
    require!(
        ctx.accounts.assignment.status == verification_task::assignment_status::SUBMITTED,
        TerraError::AssignmentNotSubmitted
    );
    require!(
        ctx.accounts.task_escrow.status == task_escrow_status::FUNDED,
        TerraError::EscrowNotFunded
    );
    let coverage: CoverageIncentive = deser_policy(
        &ctx.accounts.coverage_incentive.to_account_info(),
        TerraError::CoverageIncentiveNotInitialized,
    )?;

    let escrow = &ctx.accounts.task_escrow;
    let required = ctx.accounts.task.required_validators;
    let shares_remaining = required.saturating_sub(escrow.released_count);
    require!(shares_remaining > 0, TerraError::TaskAlreadyFinalized);

    let amount_remaining = escrow.amount.saturating_sub(escrow.released);
    let base_share = next_share(amount_remaining, shares_remaining);

    let bps = subsidy_bps_for(&coverage, ctx.accounts.task.task_class, required);
    let treasury_lamports = ctx.accounts.treasury.to_account_info().lamports();
    let rent_min = Rent::get()?.minimum_balance(0);
    let mut subsidy = subsidy_lamports(base_share, bps);
    // Keep the treasury rent-exempt (it may never dip into its own rent).
    let allowed = treasury_lamports.saturating_sub(rent_min);
    if subsidy > allowed {
        subsidy = allowed;
    }

    let now = Clock::get()?.unix_timestamp;
    let task_key = ctx.accounts.task.key();
    let task_id = ctx.accounts.task.task_id;
    let validator = ctx.accounts.validator.key();
    let escrow_key = ctx.accounts.task_escrow.key();

    // Vault → validator (base share), signed by the vault PDA.
    if base_share > 0 {
        let vault_info = ctx.accounts.vault.to_account_info();
        let available = vault_info.lamports();
        require!(available >= base_share, TerraError::EscrowNotFunded);
        let (vault_pda, bump) =
            Pubkey::find_program_address(&[b"task_escrow_vault", escrow_key.as_ref()], &crate::ID);
        let bump_seed = [bump];
        let signer_seeds: &[&[u8]] = &[b"task_escrow_vault", escrow_key.as_ref(), &bump_seed];
        let transfer_ix = anchor_lang::solana_program::system_instruction::transfer(
            &vault_pda, &validator, base_share,
        );
        anchor_lang::solana_program::program::invoke_signed(
            &transfer_ix,
            &[
                vault_info,
                ctx.accounts.validator.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
            &[signer_seeds],
        )?;
    }

    // Treasury → validator (coverage subsidy), signed by the treasury PDA.
    if subsidy > 0 {
        let treasury_info = ctx.accounts.treasury.to_account_info();
        let (treasury_pda, bump) = Pubkey::find_program_address(&[b"task_treasury"], &crate::ID);
        let bump_seed = [bump];
        let signer_seeds: &[&[u8]] = &[b"task_treasury", &bump_seed];
        let transfer_ix = anchor_lang::solana_program::system_instruction::transfer(
            &treasury_pda,
            &validator,
            subsidy,
        );
        anchor_lang::solana_program::program::invoke_signed(
            &transfer_ix,
            &[
                treasury_info,
                ctx.accounts.validator.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
            &[signer_seeds],
        )?;
    }

    // Record the distribution (idempotency guard: created_at != 0).
    let alloc = &mut ctx.accounts.reward_allocation;
    alloc.task_id = task_id;
    alloc.task = task_key;
    alloc.validator = validator;
    alloc.base_paid = base_share;
    alloc.subsidy_paid = subsidy;
    alloc.total_paid = base_share.saturating_add(subsidy);
    alloc.created_at = now;

    let escrow = &mut ctx.accounts.task_escrow;
    escrow.released = escrow.released.saturating_add(base_share);
    escrow.released_count = escrow.released_count.saturating_add(1);
    escrow.updated_at = now;
    if escrow.released_count >= required {
        escrow.status = task_escrow_status::RELEASED;
    }

    // Mark the assignment as rewarded (assignment_status::RELEASED).
    let assignment = &mut ctx.accounts.assignment;
    assignment.status = verification_task::assignment_status::RELEASED;
    assignment.updated_at = now;

    emit!(crate::TaskRewardClaimed {
        allocation: alloc.key(),
        task: task_key,
        task_id,
        validator,
        base_paid: base_share,
        subsidy_paid: subsidy,
    });
    Ok(())
}

/// Requester reclaims the remaining vault balance when the task was
/// cancelled/expired, the claim window closed, or all shares were claimed.
pub fn refund_task_escrow(ctx: Context<crate::RefundTaskEscrow>) -> Result<()> {
    require!(
        ctx.accounts.task_escrow.status == task_escrow_status::FUNDED,
        TerraError::EscrowNotFunded
    );
    let now = Clock::get()?.unix_timestamp;
    let verdict = refund_verdict(
        ctx.accounts.task.status,
        now,
        ctx.accounts.task.completed_at,
        ctx.accounts.task_escrow.released_count,
        ctx.accounts.task.required_validators,
    );
    match verdict {
        RefundVerdict::Allow => {}
        RefundVerdict::WindowOpen => return err!(TerraError::ClaimWindowNotClosed),
        RefundVerdict::NotTerminal => return err!(TerraError::EscrowNotRefundable),
    }

    let vault_info = ctx.accounts.vault.to_account_info();
    let balance = vault_info.lamports();
    require!(balance > 0, TerraError::NothingToRefund);

    let requester = ctx.accounts.requester.key();
    let escrow_key = ctx.accounts.task_escrow.key();
    let (vault_pda, bump) =
        Pubkey::find_program_address(&[b"task_escrow_vault", escrow_key.as_ref()], &crate::ID);
    let bump_seed = [bump];
    let signer_seeds: &[&[u8]] = &[b"task_escrow_vault", escrow_key.as_ref(), &bump_seed];
    let transfer_ix =
        anchor_lang::solana_program::system_instruction::transfer(&vault_pda, &requester, balance);
    anchor_lang::solana_program::program::invoke_signed(
        &transfer_ix,
        &[
            vault_info,
            ctx.accounts.requester.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
        ],
        &[signer_seeds],
    )?;

    let task_key = ctx.accounts.task.key();
    let task_id = ctx.accounts.task.task_id;
    let escrow = &mut ctx.accounts.task_escrow;
    escrow.status = task_escrow_status::REFUNDED;
    escrow.updated_at = now;

    emit!(crate::TaskEscrowRefunded {
        escrow: escrow.key(),
        task: task_key,
        task_id,
        requester,
        amount: balance,
    });
    Ok(())
}

// ---------------------------------------------------------------------------
// Unit tests (pure logic)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn policy(
        demand: u16,
        deficit: u16,
        difficulty: u16,
        strategic: u16,
        max: u16,
    ) -> CoverageIncentive {
        CoverageIncentive {
            authority: Pubkey::default(),
            demand_bps: demand,
            deficit_bps: deficit,
            difficulty_bps: difficulty,
            strategic_bps: strategic,
            max_subsidy_bps: max,
            updated_at: 0,
        }
    }

    #[test]
    fn fee_lamports_basic() {
        assert_eq!(fee_lamports(1_000_000, 100), 10_000);
        assert_eq!(fee_lamports(1_000_000, 0), 0);
        assert_eq!(fee_lamports(999, 1), 0); // floors
        assert_eq!(fee_lamports(0, 500), 0);
    }

    #[test]
    fn fee_lamports_no_overflow() {
        assert_eq!(fee_lamports(u64::MAX, 1_000), {
            let expected = ((u64::MAX as u128) * 1_000) / 10_000;
            expected as u64
        });
    }

    #[test]
    fn fee_bps_validation() {
        assert!(is_valid_fee_bps(0));
        assert!(is_valid_fee_bps(MAX_FEE_BPS));
        assert!(!is_valid_fee_bps(MAX_FEE_BPS + 1));
    }

    #[test]
    fn coverage_params_validation() {
        assert!(is_valid_coverage_params(
            MAX_COVERAGE_FACTOR_BPS,
            MAX_COVERAGE_FACTOR_BPS,
            MAX_COVERAGE_FACTOR_BPS,
            MAX_COVERAGE_FACTOR_BPS,
            MAX_SUBSIDY_BPS
        ));
        assert!(is_valid_coverage_params(0, 0, 0, 0, 0));
        assert!(!is_valid_coverage_params(
            MAX_COVERAGE_FACTOR_BPS + 1,
            0,
            0,
            0,
            0
        ));
        assert!(!is_valid_coverage_params(0, 0, 0, 0, MAX_SUBSIDY_BPS + 1));
    }

    #[test]
    fn subsidy_bps_factors_and_clamp() {
        let p = policy(100, 50, 25, 400, 500);
        // REMOTE (not strategic), 1 validator: 100 + 50 + 25×1 = 175.
        assert_eq!(
            subsidy_bps_for(&p, verification_task::task_class::REMOTE, 1),
            175
        );
        // PHYSICAL (strategic): +400 → 575, clamped to 500.
        assert_eq!(
            subsidy_bps_for(&p, verification_task::task_class::PHYSICAL, 1),
            500
        );
        // Difficulty scales ×required_validators (cap 4): 100+50+25×4=250.
        assert_eq!(
            subsidy_bps_for(&p, verification_task::task_class::COMPUTATIONAL, 8),
            250
        );
        // Zero policy → 0.
        let z = policy(0, 0, 0, 0, 0);
        assert_eq!(
            subsidy_bps_for(&z, verification_task::task_class::HYBRID, 4),
            0
        );
    }

    #[test]
    fn strategic_classes() {
        assert!(is_strategic_class(verification_task::task_class::PHYSICAL));
        assert!(is_strategic_class(
            verification_task::task_class::DOCUMENTARY
        ));
        assert!(is_strategic_class(verification_task::task_class::HYBRID));
        assert!(!is_strategic_class(verification_task::task_class::REMOTE));
        assert!(!is_strategic_class(
            verification_task::task_class::COMPUTATIONAL
        ));
        assert!(!is_strategic_class(99));
    }

    #[test]
    fn subsidy_lamports_calc() {
        assert_eq!(subsidy_lamports(1_000_000, 2_500), 250_000);
        assert_eq!(subsidy_lamports(1_000_000, 0), 0);
        assert_eq!(subsidy_lamports(999, 5_000), 499);
    }

    #[test]
    fn next_share_even_split() {
        assert_eq!(next_share(9, 3), 3);
        assert_eq!(next_share(0, 3), 0);
        assert_eq!(next_share(5, 0), 0);
    }

    #[test]
    fn next_share_uneven_gives_remainder_to_last() {
        let total = 10u64;
        let required = 3u8;
        let mut released = 0u64;
        let mut claims = Vec::new();
        for i in 0..required {
            let shares = required - i as u8;
            let share = next_share(total - released, shares);
            released += share;
            claims.push(share);
        }
        assert_eq!(claims, vec![3, 3, 4]);
        assert_eq!(released, total);
    }

    #[test]
    fn refund_verdict_matrix() {
        use verification_task::task_status;
        // Cancelled / expired → allowed regardless of claims.
        assert_eq!(
            refund_verdict(task_status::CANCELLED, 100, 0, 0, 1),
            RefundVerdict::Allow
        );
        assert_eq!(
            refund_verdict(task_status::EXPIRED, 100, 0, 0, 1),
            RefundVerdict::Allow
        );
        // Active tasks → not terminal.
        for s in [
            task_status::OPEN,
            task_status::ASSIGNED,
            task_status::IN_PROGRESS,
        ] {
            assert_eq!(refund_verdict(s, 100, 0, 0, 1), RefundVerdict::NotTerminal);
        }
        // Completed, in window, shares outstanding → window open.
        assert_eq!(
            refund_verdict(task_status::COMPLETED, 100, 50, 0, 1),
            RefundVerdict::WindowOpen
        );
        // Completed, claim window passed → allowed.
        assert_eq!(
            refund_verdict(task_status::COMPLETED, 50 + CLAIM_WINDOW_SECS + 1, 50, 0, 1),
            RefundVerdict::Allow
        );
        // Completed, all shares claimed → dust refund allowed.
        assert_eq!(
            refund_verdict(task_status::COMPLETED, 100, 50, 1, 1),
            RefundVerdict::Allow
        );
    }

    #[test]
    fn escrow_status_values_distinct_from_zero() {
        // 0 marks a fresh init_if_needed account; funded must be non-zero.
        assert_ne!(task_escrow_status::FUNDED, 0);
        assert_ne!(task_escrow_status::RELEASED, task_escrow_status::FUNDED);
        assert_ne!(task_escrow_status::REFUNDED, task_escrow_status::FUNDED);
        assert!(task_escrow_status::REFUNDED <= task_escrow_status::MAX);
    }
}
