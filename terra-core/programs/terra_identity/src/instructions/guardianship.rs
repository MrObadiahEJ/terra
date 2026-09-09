use anchor_lang::prelude::*;

use crate::constants::*;
use crate::errors::IdentityError;
use crate::events::*;
use crate::helpers::*;

/// Request a court-appointed guardianship with an explicit case_hash binding.
pub fn request_court_guardianship(
    ctx: Context<crate::RequestCourtGuardianship>,
    successor: Pubkey,
    grace_secs: i64,
    required_validations: u8,
    validators: [Pubkey; MAX_VALIDATORS],
    case_hash: [u8; 32],
    scope_notes: String,
) -> Result<()> {
    require!(successor != Pubkey::default(), IdentityError::EmptySuccessor);
    require!(
        !case_hash.iter().all(|b| *b == 0),
        IdentityError::EmptyCaseHash
    );
    require!(
        scope_notes.len() <= MAX_SCOPE_NOTES_LEN,
        IdentityError::NotesTooLong
    );

    let identity = &ctx.accounts.identity;
    let signer = ctx.accounts.signer.key();
    require!(
        signer == identity.owner || signer == identity.recovery,
        IdentityError::NotAuthorized
    );
    require!(successor != identity.owner, IdentityError::SuccessorIsOwner);

    let mut count: u8 = 0;
    for &v in validators.iter() {
        if v == Pubkey::default() {
            continue;
        }
        require!(v != identity.owner, IdentityError::ValidatorOwnsAsset);
        count += 1;
    }
    require!(count > 0, IdentityError::NotValidator);
    require!(
        validate_guardianship_threshold(required_validations, count as usize),
        IdentityError::GuardianshipThresholdTooLow
    );

    let grace = if grace_secs == 0 {
        DEFAULT_GUARDIANSHIP_GRACE_SECS
    } else {
        require!(
            grace_secs >= MIN_GUARDIANSHIP_GRACE_SECS,
            IdentityError::GuardianshipGraceTooShort
        );
        grace_secs.min(MAX_SUCCESSION_GRACE_SECS)
    };

    let now = Clock::get()?.unix_timestamp;
    let succession = &mut ctx.accounts.succession;
    succession.identity = identity.key();
    succession.successor = successor;
    succession.kind = succession_kind::COURT_APPOINTED_GUARDIAN;
    succession.requested_at = now;
    succession.grace_secs = grace;
    succession.effective_at = now.saturating_add(grace);
    succession.required = required_validations;
    succession.validations_count = 0;
    succession.validators = validators;

    emit!(CourtGuardianshipRequested {
        identity: identity.key(),
        successor,
        grace_secs: grace,
        required: required_validations,
        count,
        effective_at: succession.effective_at,
        case_hash,
        scope_notes,
    });
    Ok(())
}

/// Request revocation of an active guardianship (RFC-010 §6.5).
pub fn revoke_guardianship(
    ctx: Context<crate::RevokeGuardianship>,
    new_owner: Pubkey,
) -> Result<()> {
    require!(new_owner != Pubkey::default(), IdentityError::EmptySuccessor);

    let revoker = ctx.accounts.revoker.key();
    let identity = &ctx.accounts.identity;
    let previous = identity.owner;
    require!(new_owner != previous, IdentityError::SuccessorIsOwner);

    let is_recovery = revoker == identity.recovery;
    require!(is_recovery, IdentityError::NotAuthorized);

    let now = Clock::get()?.unix_timestamp;
    let identity = &mut ctx.accounts.identity;
    require!(
        !identity.pending_revocation,
        IdentityError::GuardianshipAlreadyActive
    );
    identity.pending_revocation = true;
    identity.pending_new_owner = new_owner;
    identity.revoke_after = now.saturating_add(GUARDIANSHIP_REVOKE_TIMELOCK_SECS);
    identity.updated_at = now;

    emit!(GuardianshipRevocationRequested {
        identity: identity.key(),
        previous_guardian: previous,
        new_owner,
        requested_by: revoker,
        revoke_after: identity.revoke_after,
        block_time: now,
    });
    Ok(())
}

/// Execute a previously requested guardianship revocation after the timelock.
pub fn execute_revoke_guardianship(ctx: Context<crate::ExecuteRevokeGuardianship>) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let identity = &ctx.accounts.identity;
    require!(identity.pending_revocation, IdentityError::NoProposalFound);
    require!(
        now >= identity.revoke_after,
        IdentityError::SettlementNotYetEffective
    );

    let new_owner = ctx.accounts.new_owner.key();
    require!(new_owner != Pubkey::default(), IdentityError::EmptySuccessor);
    require!(new_owner != identity.owner, IdentityError::SuccessorIsOwner);
    require!(
        new_owner == identity.pending_new_owner,
        IdentityError::NotAuthorized
    );

    let identity_key = identity.key();
    let previous_guardian = identity.owner;

    let identity = &mut ctx.accounts.identity;
    identity.owner = new_owner;
    identity.recovery = Pubkey::default();
    identity.pending_revocation = false;
    identity.pending_new_owner = Pubkey::default();
    identity.revoke_after = 0;
    identity.updated_at = now;

    emit!(GuardianshipRevoked {
        identity: identity_key,
        previous_guardian,
        new_owner,
        revoked_by: new_owner,
        block_time: now,
    });
    Ok(())
}
