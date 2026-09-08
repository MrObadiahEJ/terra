use anchor_lang::prelude::*;

use crate::TerraError;

/// Validators not seen within this window are considered inactive for quorum
/// calculations. 24 hours balances liveness sensitivity with clock skew.
pub const ACTIVITY_WINDOW_SECS: i64 = 24 * 3600;

/// Minimum time between emergency injection request and execution. 48 hours
/// gives existing validators time to react or restore quorum organically.
pub const EMERGENCY_INJECTION_TIMELOCK_SECS: i64 = 48 * 3600;

// ---------------------------------------------------------------------------
// Accounts
// ---------------------------------------------------------------------------

#[account]
#[derive(InitSpace)]
pub struct ValidatorActivityTracker {
    /// The AuthorityRegistry this tracker belongs to.
    pub registry: Pubkey,
    /// The validator whose activity is tracked.
    pub validator: Pubkey,
    /// Unix timestamp of last recorded activity (heartbeat or signed tx).
    pub last_active: i64,
    /// Soft flag — set to false by admin to forcibly exclude a validator from
    /// quorum without removing them from the registry.
    pub is_active: bool,
}

#[account]
#[derive(InitSpace)]
pub struct EmergencyInjection {
    /// The AuthorityRegistry this injection targets.
    pub registry: Pubkey,
    /// Admin who queued the injection.
    pub requested_by: Pubkey,
    /// Validator public key to inject.
    pub candidate: Pubkey,
    /// Unix timestamp after which the injection can be executed.
    pub execute_after: i64,
    /// True once executed.
    pub executed: bool,
}

// ---------------------------------------------------------------------------
// Instruction handlers
// ---------------------------------------------------------------------------

/// Validator heartbeat — updates last_active timestamp. Can be called by any
/// registered validator to prove liveness.
pub fn heartbeat(ctx: &mut Context<super::Heartbeat>) -> Result<()> {
    let tracker = &mut ctx.accounts.activity_tracker;
    let registry = &ctx.accounts.registry;

    require!(
        tracker.registry == registry.key(),
        TerraError::NotValidator
    );
    require!(
        registry.validators.contains(&ctx.accounts.validator.key()),
        TerraError::NotValidator
    );

    tracker.last_active = Clock::get()?.unix_timestamp;
    tracker.is_active = true;
    Ok(())
}

/// Admin sets a validator's active flag.
pub fn set_validator_active(
    ctx: &mut Context<super::SetValidatorActive>,
    validator: Pubkey,
    is_active: bool,
) -> Result<()> {
    let registry = &ctx.accounts.registry;
    require!(
        registry.admin == ctx.accounts.admin.key(),
        TerraError::NotAuthorized
    );

    let tracker = &mut ctx.accounts.activity_tracker;
    if tracker.validator == Pubkey::default() {
        tracker.registry = registry.key();
        tracker.validator = validator;
        tracker.last_active = Clock::get()?.unix_timestamp;
    } else {
        require!(tracker.validator == validator, TerraError::NotValidator);
        require!(tracker.registry == registry.key(), TerraError::NotValidator);
    }
    tracker.is_active = is_active;

    emit!(super::ValidatorActiveSet {
        registry: registry.key(),
        validator,
        is_active,
        set_by: ctx.accounts.admin.key(),
    });
    Ok(())
}

/// Compute how many validators are currently active. Emits event for
/// off-chain clients to decide whether to trigger emergency injection.
pub fn check_quorum_reachable(ctx: &Context<super::CheckQuorumReachable>) -> Result<()> {
    let registry = &ctx.accounts.registry;

    let active_count = registry.validators.len() as u8;
    let quorum_reachable = active_count >= crate::authority_registry::CONSENSUS_FLIP_THRESHOLD
        && registry.required_endorsements > 0;

    emit!(super::QuorumReachabilityChecked {
        registry: registry.key(),
        total_validators: registry.validators.len() as u8,
        active_count,
        required_endorsements: registry.required_endorsements,
        quorum_reachable,
    });
    Ok(())
}

/// Queue an emergency validator injection. Admin-only. Executes after
/// EMERGENCY_INJECTION_TIMELOCK_SECS.
pub fn queue_emergency_injection(
    ctx: &mut Context<super::QueueEmergencyInjection>,
    candidate: Pubkey,
) -> Result<()> {
    let registry = &ctx.accounts.registry;
    require!(
        registry.admin == ctx.accounts.admin.key(),
        TerraError::NotAuthorized
    );

    let now = Clock::get()?.unix_timestamp;
    let injection = &mut ctx.accounts.emergency_injection;
    injection.registry = registry.key();
    injection.requested_by = ctx.accounts.admin.key();
    injection.candidate = candidate;
    injection.execute_after = now + EMERGENCY_INJECTION_TIMELOCK_SECS;
    injection.executed = false;

    emit!(super::EmergencyInjectionQueued {
        registry: registry.key(),
        candidate,
        requested_by: ctx.accounts.admin.key(),
        execute_after: injection.execute_after,
    });
    Ok(())
}

/// Execute a queued emergency injection after the timelock has elapsed.
pub fn execute_emergency_injection(ctx: &mut Context<super::ExecuteEmergencyInjection>) -> Result<()> {
    let injection = &mut ctx.accounts.emergency_injection;
    require!(!injection.executed, TerraError::AlreadyEndorsedRotation);
    require!(
        Clock::get()?.unix_timestamp >= injection.execute_after,
        TerraError::EmergencyTimelockNotElapsed
    );

    let registry = &mut ctx.accounts.registry;
    require!(
        !registry.validators.contains(&injection.candidate),
        TerraError::AlreadyEndorsedRotation
    );

    registry.validators.push(injection.candidate);
    registry.version = registry.version.saturating_add(1);
    registry.updated_at = Clock::get()?.unix_timestamp;

    let n = registry.validators.len() as u8;
    let mode = crate::authority_registry::effective_mode(n);
    if mode == crate::authority_registry::registry_mode::PEER_CONSENSUS {
        registry.required_endorsements = crate::authority_registry::consensus_required(n);
    } else {
        registry.required_endorsements = 0;
    }

    let tracker = &mut ctx.accounts.new_validator_activity;
    tracker.registry = registry.key();
    tracker.validator = injection.candidate;
    tracker.last_active = Clock::get()?.unix_timestamp;
    tracker.is_active = true;

    injection.executed = true;

    emit!(super::EmergencyInjectionExecuted {
        registry: registry.key(),
        candidate: injection.candidate,
        executed_by: ctx.accounts.admin.key(),
        new_validator_count: registry.validators.len() as u8,
    });
    Ok(())
}
