use anchor_lang::prelude::*;

use crate::{right_status, Rights, TerraError};

/// 30-day warning window before expiry.
pub const EXPIRING_WARNING_SECS: i64 = 30 * 24 * 3600;
/// Maximum grace period: 1 year.
pub const MAX_GRACE_PERIOD_SECS: i64 = 365 * 24 * 3600;

// ---------------------------------------------------------------------------
// Lazy evaluation helper — called inline by any instruction touching Rights.
// ---------------------------------------------------------------------------

/// Evaluate and update a right's status based on current time.
/// Returns the (possibly changed) status. Emits `RightStatusTransition` if changed.
pub fn update_right_status(rights: &mut Rights, rights_key: Pubkey) -> Result<u8> {
    let now = Clock::get()?.unix_timestamp;
    let old_status = rights.status;

    // Permanent rights (expires_at == 0) are never swept.
    if rights.expires_at == 0 {
        return Ok(rights.status);
    }

    // Already terminal — no-op.
    if rights.status == right_status::REVOKED
        || rights.status == right_status::RENEWED
        || rights.status == right_status::EXPIRED
    {
        return Ok(rights.status);
    }

    if now < rights.expires_at {
        // Not yet expired — check if entering warning window.
        if now > rights.expires_at - EXPIRING_WARNING_SECS {
            rights.status = right_status::EXPIRING;
        } else {
            rights.status = right_status::ACTIVE;
        }
    } else {
        // Past expiry — check grace period.
        if rights.grace_period_secs > 0 && now < rights.expires_at + rights.grace_period_secs {
            rights.status = right_status::GRACE;
        } else {
            rights.status = right_status::EXPIRED;
        }
    }

    if rights.status != old_status {
        emit!(RightStatusTransition {
            parcel: rights.parcel,
            rights: rights_key,
            holder: rights.holder,
            old_status,
            new_status: rights.status,
            expires_at: rights.expires_at,
            block_time: now,
        });
    }

    Ok(rights.status)
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// Renew an expiring or expired right by extending `expires_at`.
/// Requires both holder and granter co-sign.
pub fn renew_right(
    ctx: Context<super::RenewRight>,
    _nonce: u8,
    new_expires_at: i64,
    new_notes: String,
) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    require!(new_expires_at > now, TerraError::InvalidExpiry);
    require!(new_notes.len() <= 128, TerraError::NotesTooLong);

    let rights = &mut ctx.accounts.rights;
    require!(
        rights.status != right_status::REVOKED,
        TerraError::CannotRenewRevokedRight
    );
    require!(
        rights.status != right_status::RENEWED,
        TerraError::CannotRenewAlreadyRenewed
    );
    require!(
        new_expires_at > rights.expires_at,
        TerraError::RenewalMustExtendExpiry
    );

    // Granter must be the original granter or current parcel owner.
    let granter_key = ctx.accounts.granter.key();
    require!(
        granter_key == rights.granter || granter_key == ctx.accounts.parcel.owner,
        TerraError::NotAuthorized
    );

    let old_expires_at = rights.expires_at;
    let old_status = rights.status;
    let rights_key = rights.key();

    rights.expires_at = new_expires_at;
    rights.status = right_status::ACTIVE;
    if !new_notes.is_empty() {
        rights.notes = new_notes;
    }

    emit!(RightRenewed {
        parcel: rights.parcel,
        old_rights: rights_key,
        holder: rights.holder,
        granter: granter_key,
        old_expires_at,
        new_expires_at,
        block_time: now,
    });

    if old_status != right_status::ACTIVE {
        emit!(RightStatusTransition {
            parcel: rights.parcel,
            rights: rights_key,
            holder: rights.holder,
            old_status,
            new_status: right_status::ACTIVE,
            expires_at: new_expires_at,
            block_time: now,
        });
    }

    Ok(())
}

/// Keeper instruction: mark expired rights as EXPIRED or GRACE.
/// Callable by an authorized keeper, the holder, the granter, or the parcel owner.
pub fn sweep_expired_rights(ctx: Context<super::SweepExpiredRights>, _nonce: u8) -> Result<()> {
    let rights = &mut ctx.accounts.rights;

    // Only sweep ACTIVE or EXPIRING rights.
    require!(
        rights.status == right_status::ACTIVE || rights.status == right_status::EXPIRING,
        TerraError::InvalidRightStatus
    );

    let now = Clock::get()?.unix_timestamp;

    // Permanent rights are never swept.
    require!(
        rights.expires_at != 0,
        TerraError::PermanentRightNotSweepable
    );

    // Not yet expired — no-op.
    if now < rights.expires_at {
        return Ok(());
    }

    let old_status = rights.status;
    let rights_key = rights.key();

    if rights.grace_period_secs > 0 && now < rights.expires_at + rights.grace_period_secs {
        rights.status = right_status::GRACE;
    } else {
        rights.status = right_status::EXPIRED;
    }

    emit!(RightStatusTransition {
        parcel: rights.parcel,
        rights: rights_key,
        holder: rights.holder,
        old_status,
        new_status: rights.status,
        expires_at: rights.expires_at,
        block_time: now,
    });

    Ok(())
}

/// Grant a right with an additional time-bound condition.
pub fn grant_conditional_right(
    ctx: Context<super::GrantConditionalRight>,
    nonce: u8,
    rights_kind: u8,
    holder: Pubkey,
    expires_at: i64,
    condition_deadline: i64,
    condition_desc: String,
    grace_period_secs: i64,
    notes: String,
) -> Result<()> {
    let parcel = &mut ctx.accounts.parcel;
    require!(
        parcel.owner == ctx.accounts.owner.key(),
        TerraError::NotOwner
    );
    require!(
        rights_kind <= crate::right_kind::MAX,
        TerraError::InvalidRightKind
    );
    require!(nonce == parcel.rights_count, TerraError::InvalidNonce);
    require!(
        (parcel.rights_count as u16) < u8::MAX as u16,
        TerraError::RightsLimitExceeded
    );
    require!(notes.len() <= 128, TerraError::NotesTooLong);
    require!(condition_desc.len() <= 128, TerraError::NotesTooLong);

    let now = Clock::get()?.unix_timestamp;
    if expires_at != 0 {
        require!(expires_at > now, TerraError::InvalidExpiry);
    }
    require!(condition_deadline > now, TerraError::InvalidExpiry);
    require!(
        expires_at == 0 || condition_deadline < expires_at,
        TerraError::ConditionDeadlineAfterExpiry
    );
    require!(
        grace_period_secs >= 0 && grace_period_secs <= MAX_GRACE_PERIOD_SECS,
        TerraError::InvalidGracePeriod
    );

    let rights = &mut ctx.accounts.rights;
    rights.parcel = parcel.key();
    rights.rights_kind = rights_kind;
    rights.holder = holder;
    rights.granter = ctx.accounts.owner.key();
    rights.created_at = now;
    rights.expires_at = expires_at;
    rights.notes = notes;
    rights.status = right_status::ACTIVE;
    rights.grace_period_secs = grace_period_secs;

    parcel.rights_count += 1;

    emit!(crate::RightGranted {
        parcel: parcel.key(),
        rights_kind,
        holder,
    });
    Ok(())
}

// ---------------------------------------------------------------------------
// Events
// ---------------------------------------------------------------------------

#[event]
pub struct RightStatusTransition {
    pub parcel: Pubkey,
    pub rights: Pubkey,
    pub holder: Pubkey,
    pub old_status: u8,
    pub new_status: u8,
    pub expires_at: i64,
    pub block_time: i64,
}

#[event]
pub struct RightRenewed {
    pub parcel: Pubkey,
    pub old_rights: Pubkey,
    pub holder: Pubkey,
    pub granter: Pubkey,
    pub old_expires_at: i64,
    pub new_expires_at: i64,
    pub block_time: i64,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn right_status_values_are_contiguous() {
        assert_eq!(right_status::ACTIVE, 0);
        assert_eq!(right_status::EXPIRING, 1);
        assert_eq!(right_status::EXPIRED, 2);
        assert_eq!(right_status::GRACE, 3);
        assert_eq!(right_status::RENEWED, 4);
        assert_eq!(right_status::REVOKED, 5);
        assert_eq!(right_status::MAX, right_status::REVOKED);
    }

    #[test]
    fn expiring_warning_is_30_days() {
        assert_eq!(EXPIRING_WARNING_SECS, 30 * 24 * 3600);
    }

    #[test]
    fn max_grace_period_is_1_year() {
        assert_eq!(MAX_GRACE_PERIOD_SECS, 365 * 24 * 3600);
    }
}
