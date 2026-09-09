use anchor_lang::prelude::*;

use crate::constants::*;
use crate::errors::IdentityError;
use crate::events::*;
use crate::helpers::*;

/// Request a wallet passation (succession, recovery, or deliberate transfer).
pub fn request_succession(
    ctx: Context<crate::RequestSuccession>,
    successor: Pubkey,
    kind: u8,
    grace_secs: i64,
    required_validations: u8,
    validators: [Pubkey; MAX_VALIDATORS],
) -> Result<()> {
    require!(successor != Pubkey::default(), IdentityError::EmptySuccessor);
    require!(
        kind <= succession_kind::MAX,
        IdentityError::InvalidSuccessionKind
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
        (required_validations as usize) <= count as usize,
        IdentityError::InvalidThreshold
    );
    require!(
        required_validations >= MIN_SUCCESSION_VALIDATIONS,
        IdentityError::InvalidThreshold
    );

    if is_guardianship_kind(kind) {
        require!(
            validate_guardianship_threshold(required_validations, count as usize),
            IdentityError::GuardianshipThresholdTooLow
        );
    }

    let grace = if is_guardianship_kind(kind) {
        if grace_secs == 0 {
            DEFAULT_GUARDIANSHIP_GRACE_SECS
        } else {
            require!(
                grace_secs >= MIN_GUARDIANSHIP_GRACE_SECS,
                IdentityError::GuardianshipGraceTooShort
            );
            grace_secs.min(MAX_SUCCESSION_GRACE_SECS)
        }
    } else if grace_secs == 0 {
        DEFAULT_SUCCESSION_GRACE_SECS
    } else {
        grace_secs.clamp(MIN_SUCCESSION_GRACE_SECS, MAX_SUCCESSION_GRACE_SECS)
    };

    let now = Clock::get()?.unix_timestamp;
    let succession = &mut ctx.accounts.succession;
    succession.identity = identity.key();
    succession.successor = successor;
    succession.kind = kind;
    succession.requested_at = now;
    succession.grace_secs = grace;
    succession.effective_at = now.saturating_add(grace);
    succession.required = required_validations;
    succession.validations_count = 0;
    succession.validators = validators;

    emit!(SuccessionRequested {
        identity: identity.key(),
        successor,
        kind,
        grace_secs: grace,
        required: required_validations,
        count,
        effective_at: succession.effective_at,
    });
    Ok(())
}

/// Record one validator's endorsement of a pending succession.
pub fn endorse_succession(ctx: Context<crate::EndorseSuccession>) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let succession = &mut ctx.accounts.succession;
    require!(
        now < succession.effective_at,
        IdentityError::SuccessionAlreadyEffective
    );
    require!(
        (succession.validations_count as usize) < (succession.required as usize),
        IdentityError::ValidationLimitReached
    );

    let validator = ctx.accounts.validator.key();
    require!(
        succession.validators.contains(&validator),
        IdentityError::NotValidator
    );
    require!(
        validator != ctx.accounts.identity.owner,
        IdentityError::ValidatorOwnsAsset
    );

    succession.validations_count += 1;

    emit!(SuccessionEndorsed {
        identity: succession.identity,
        successor: succession.successor,
        validator,
        validations_count: succession.validations_count,
        required: succession.required,
    });
    Ok(())
}

/// Cancel an in-flight succession.
pub fn cancel_succession(ctx: Context<crate::CancelSuccession>) -> Result<()> {
    let identity = &ctx.accounts.identity;
    let signer = ctx.accounts.signer.key();
    require!(
        signer == identity.owner || signer == identity.recovery,
        IdentityError::NotAuthorized
    );
    require!(
        ctx.accounts.succession.effective_at > Clock::get()?.unix_timestamp,
        IdentityError::SuccessionAlreadyEffective
    );

    emit!(SuccessionCancelled {
        identity: identity.key(),
        successor: ctx.accounts.succession.successor,
        kind: ctx.accounts.succession.kind,
    });
    Ok(())
}

/// Claim a passation once both the grace period has elapsed AND the required
/// number of validators have endorsed it.
///
/// Optionally accepts parcel accounts as remaining_accounts. Each parcel
/// whose `owner` matches the current identity owner is transferred to the
/// successor. This keeps succession proportional — only explicitly provided
/// parcels are updated, not the entire registry.
pub fn claim_succession(ctx: Context<crate::ClaimSuccession>) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let succession = &ctx.accounts.succession;
    require!(
        succession.successor == ctx.accounts.signer.key(),
        IdentityError::NotSuccessor
    );
    require!(
        now >= succession.effective_at,
        IdentityError::SuccessionNotYetEffective
    );
    require!(
        succession.validations_count >= succession.required,
        IdentityError::InsufficientValidations
    );

    let identity = &mut ctx.accounts.identity;
    require!(
        succession.identity == identity.key(),
        IdentityError::IdentityMismatch
    );

    let previous = identity.owner;
    let successor = succession.successor;

    // Transfer ownership of explicitly provided parcels to the successor.
    // Each parcel must be a mutable UncheckedAccount whose data starts with
    // the 8-byte Anchor discriminator, followed by borsh-encoded Parcel fields.
    for account_info in ctx.remaining_accounts.iter() {
        let data = account_info.try_borrow_data()?;
        require!(data.len() >= 8 + 32, IdentityError::ParcelDataTooShort);
        // Skip 8-byte Anchor discriminator, then read the Parcel fields.
        // Parcel layout: id(32) + owner(32) + name(String) + ...
        let slice = &data[8..];
        let parcel: crate::state::Parcel =
            anchor_lang::AnchorDeserialize::try_from_slice(slice)
                .map_err(|_| error!(IdentityError::ParcelDeserializeFailed))?;
        require!(
            parcel.owner == previous,
            IdentityError::ParcelOwnerMismatch
        );
        drop(data);
        // Write the new owner into the parcel account data at the owner offset
        // (8 discriminator + 32 id = offset 40).
        let mut data = account_info.try_borrow_mut_data()?;
        let owner_bytes = successor.to_bytes();
        data[40..72].copy_from_slice(&owner_bytes);
    }

    identity.owner = successor;
    identity.recovery = Pubkey::default();
    identity.updated_at = now;

    emit!(SuccessionClaimed {
        identity: identity.key(),
        from: previous,
        to: successor,
        kind: succession.kind,
    });
    Ok(())
}
