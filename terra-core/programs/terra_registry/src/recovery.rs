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
    /// The ValidatorRegistry this tracker belongs to.
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
    /// The ValidatorRegistry this injection targets.
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

    require!(tracker.registry == registry.key(), TerraError::NotValidator);
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

    // In peer consensus the admin may reactivate a validator but must not
    // deactivate one unilaterally: `is_active = false` is exactly the silent
    // quorum-exclusion vector. Deactivation is expressed by liveness (no
    // heartbeat inside ACTIVITY_WINDOW_SECS) or by an endorsed removal.
    if crate::validator_registry::effective_mode(registry.validators.len() as u8)
        == crate::validator_registry::registry_mode::PEER_CONSENSUS
    {
        require!(is_active, TerraError::PeerConsensusRequired);
    }

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

/// Compute how many validators are currently active and emit an event for
/// off-chain clients to decide whether to trigger emergency injection.
///
/// Liveness is derived from the `ValidatorActivityTracker` PDAs passed in
/// `remaining_accounts` (max one per validator): a validator counts only when
/// its tracker is enabled and its `last_active` falls inside
/// [`ACTIVITY_WINDOW_SECS`]. Passing no trackers therefore reports zero active
/// validators — conservative, never falsely reassuring.
pub fn check_quorum_reachable(ctx: &Context<super::CheckQuorumReachable>) -> Result<()> {
    let registry = &ctx.accounts.registry;
    let now = Clock::get()?.unix_timestamp;

    let mut active_count: u8 = 0;
    let mut seen: Vec<Pubkey> = Vec::new();
    for info in ctx.remaining_accounts.iter() {
        let tracker = match Account::<crate::recovery::ValidatorActivityTracker>::try_from(info) {
            Ok(tracker) => tracker,
            Err(_) => continue,
        };
        if tracker.registry != registry.key() {
            continue;
        }
        if !registry.validators.contains(&tracker.validator) {
            continue;
        }
        // One tracker per validator; ignore duplicates so the count cannot be
        // inflated by replaying the same account.
        if seen.contains(&tracker.validator) {
            continue;
        }
        seen.push(tracker.validator);

        let fresh = now.saturating_sub(tracker.last_active) <= ACTIVITY_WINDOW_SECS;
        if tracker.is_active && fresh {
            active_count = active_count.saturating_add(1);
        }
    }

    let quorum_reachable =
        registry.required_endorsements > 0 && active_count >= registry.required_endorsements;

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
pub fn execute_emergency_injection(
    ctx: &mut Context<super::ExecuteEmergencyInjection>,
) -> Result<()> {
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
    let mode = crate::validator_registry::effective_mode(n);
    if mode == crate::validator_registry::registry_mode::PEER_CONSENSUS {
        registry.required_endorsements = crate::validator_registry::consensus_required(n);
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
