use anchor_lang::prelude::*;

use crate::TerraError;

// ---------------------------------------------------------------------------
// Observer status
// ---------------------------------------------------------------------------

pub mod observer_status {
    pub const ACTIVE: u8 = 0;
    pub const SUSPENDED: u8 = 1;
}

// ---------------------------------------------------------------------------
// Observer account
// ---------------------------------------------------------------------------

/// An on-chain observer (non-validator participant) who can submit
/// observations for claims.
///
/// PDA: `["observer", wallet]`
#[account]
#[derive(InitSpace)]
pub struct Observer {
    /// The observer's wallet pubkey.
    pub wallet: Pubkey,
    /// Optional linked identity (e.g. from terra_identity program).
    pub identity: Pubkey,
    /// Current status (ACTIVE, SUSPENDED).
    pub status: u8,
    /// Total observations submitted.
    pub total_observations: u64,
    /// Observations that were confirmed by quorum.
    pub confirmed_observations: u64,
    pub created_at: i64,
    pub updated_at: i64,
}

// ---------------------------------------------------------------------------
// Instruction handlers
// ---------------------------------------------------------------------------

/// Register a new observer. Anyone may register (observer reputation is
/// tracked by the community, not on-chain).
pub fn register_observer(
    ctx: Context<crate::RegisterObserver>,
    identity: Pubkey,
) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let observer = &mut ctx.accounts.observer;
    observer.wallet = ctx.accounts.wallet.key();
    observer.identity = identity;
    observer.status = observer_status::ACTIVE;
    observer.total_observations = 0;
    observer.confirmed_observations = 0;
    observer.created_at = now;
    observer.updated_at = now;

    emit!(crate::ObserverRegistered {
        observer: observer.key(),
        wallet: observer.wallet,
        identity,
        registered_at: now,
    });
    Ok(())
}

/// Suspend an observer. Only the registry admin may suspend observers.
pub fn suspend_observer(
    ctx: Context<crate::SuspendObserver>,
) -> Result<()> {
    let registry = &ctx.accounts.registry;
    require!(
        ctx.accounts.authority.key() == registry.admin,
        TerraError::NotAuthorized
    );

    let observer = &mut ctx.accounts.observer;
    require!(
        observer.status == observer_status::ACTIVE,
        TerraError::ObserverNotActive
    );

    observer.status = observer_status::SUSPENDED;
    observer.updated_at = Clock::get()?.unix_timestamp;

    emit!(crate::ObserverStatusChanged {
        observer: observer.key(),
        wallet: observer.wallet,
        status: observer_status::SUSPENDED,
        updated_at: observer.updated_at,
    });
    Ok(())
}

/// Reactivate a suspended observer. Only the registry admin may reactivate.
pub fn reactivate_observer(
    ctx: Context<crate::ReactivateObserver>,
) -> Result<()> {
    let registry = &ctx.accounts.registry;
    require!(
        ctx.accounts.authority.key() == registry.admin,
        TerraError::NotAuthorized
    );

    let observer = &mut ctx.accounts.observer;
    require!(
        observer.status == observer_status::SUSPENDED,
        TerraError::ObserverNotSuspended
    );

    observer.status = observer_status::ACTIVE;
    observer.updated_at = Clock::get()?.unix_timestamp;

    emit!(crate::ObserverStatusChanged {
        observer: observer.key(),
        wallet: observer.wallet,
        status: observer_status::ACTIVE,
        updated_at: observer.updated_at,
    });
    Ok(())
}
