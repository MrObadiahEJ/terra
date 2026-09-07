#![allow(unexpected_cfgs)]

use anchor_lang::prelude::*;

declare_id!("68urV9nGcRcoWT1QjzZfXuCnTS9921x2se1SybKJr1U4");

pub const MAX_VALIDATORS: usize = 8;

pub mod succession_kind {
    pub const SUCCESSOR: u8 = 0;
    pub const RECOVERY: u8 = 1;
    pub const TRANSFER: u8 = 2;
    pub const GUARDIANSHIP: u8 = 3;
    pub const COURT_APPOINTED_GUARDIAN: u8 = 4;
    pub const MAX: u8 = COURT_APPOINTED_GUARDIAN;
}

pub const MIN_SUCCESSION_GRACE_SECS: i64 = 7 * 24 * 3600;
pub const MAX_SUCCESSION_GRACE_SECS: i64 = 180 * 24 * 3600;
pub const DEFAULT_SUCCESSION_GRACE_SECS: i64 = 30 * 24 * 3600;
pub const MIN_SUCCESSION_VALIDATIONS: u8 = 1;

pub const MIN_GUARDIANSHIP_GRACE_SECS: i64 = 90 * 24 * 3600;
pub const DEFAULT_GUARDIANSHIP_GRACE_SECS: i64 = 180 * 24 * 3600;
pub const MIN_GUARDIANSHIP_VALIDATIONS: usize = 3;
pub const GUARDIANSHIP_REVOKE_TIMELOCK_SECS: i64 = 48 * 3600;

/// Binds a person (via a hashed identity credential) to a wallet.
/// PDA: `["identity", identity_hash]`.
#[account]
#[derive(InitSpace)]
pub struct Identity {
    pub identity_hash: [u8; 32],
    pub owner: Pubkey,
    pub recovery: Pubkey,
    pub parcel_count: u16,
    pub created_at: i64,
    pub updated_at: i64,
    pub pending_revocation: bool,
    pub pending_new_owner: Pubkey,
    pub revoke_after: i64,
}

/// An in-flight passation of wallet control.
/// PDA: `["succession", identity, successor]`.
#[account]
#[derive(InitSpace)]
pub struct Succession {
    pub identity: Pubkey,
    pub successor: Pubkey,
    pub kind: u8,
    pub requested_at: i64,
    pub effective_at: i64,
    pub grace_secs: i64,
    pub required: u8,
    pub validations_count: u8,
    pub validators: [Pubkey; MAX_VALIDATORS],
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

pub fn is_guardianship_kind(kind: u8) -> bool {
    kind == succession_kind::GUARDIANSHIP || kind == succession_kind::COURT_APPOINTED_GUARDIAN
}

pub fn normalize_guardianship_grace(grace_secs: i64) -> i64 {
    if grace_secs == 0 {
        DEFAULT_GUARDIANSHIP_GRACE_SECS
    } else {
        grace_secs.clamp(MIN_GUARDIANSHIP_GRACE_SECS, MAX_SUCCESSION_GRACE_SECS)
    }
}

pub fn validate_guardianship_threshold(required: u8, count: usize) -> Result<()> {
    require!(
        required as usize >= MIN_GUARDIANSHIP_VALIDATIONS,
        TerraIdentityError::GuardianshipThresholdTooLow
    );
    require!(
        (required as usize) <= count,
        TerraIdentityError::InvalidThreshold
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// Instruction handlers
// ---------------------------------------------------------------------------

/// Bind a person to a wallet. This is the CPI entry point called by
/// terra_registry (or directly for initial binding).
pub fn bind_identity(
    ctx: Context<BindIdentity>,
    identity_hash: [u8; 32],
    recovery: Pubkey,
) -> Result<()> {
    require!(
        !identity_hash.iter().all(|b| *b == 0),
        TerraIdentityError::EmptyIdentityHash
    );
    require!(recovery != Pubkey::default(), TerraIdentityError::EmptyRecovery);

    let now = Clock::get()?.unix_timestamp;
    let identity = &mut ctx.accounts.identity;
    identity.identity_hash = identity_hash;
    identity.owner = ctx.accounts.owner.key();
    identity.recovery = recovery;
    identity.parcel_count = 0;
    identity.created_at = now;
    identity.updated_at = now;

    emit!(IdentityBound {
        identity: identity.key(),
        identity_hash,
        owner: identity.owner,
        recovery,
    });
    Ok(())
}

/// Attach a parcel to an identity (increments parcel_count).
pub fn attach_parcel(ctx: Context<AttachParcel>) -> Result<()> {
    let identity = &mut ctx.accounts.identity;
    require!(
        identity.owner == ctx.accounts.owner.key(),
        TerraIdentityError::IdentityMismatch
    );

    identity.parcel_count = identity.parcel_count.saturating_add(1);
    identity.updated_at = Clock::get()?.unix_timestamp;
    emit!(ParcelAttached {
        identity: identity.key(),
        owner: identity.owner,
    });
    Ok(())
}

/// Request a wallet passation. Creates a Succession account.
pub fn request_succession(
    ctx: Context<RequestSuccession>,
    successor: Pubkey,
    kind: u8,
    grace_secs: i64,
    required_validations: u8,
    validators: [Pubkey; MAX_VALIDATORS],
) -> Result<()> {
    require!(successor != Pubkey::default(), TerraIdentityError::EmptySuccessor);
    require!(
        kind <= succession_kind::MAX,
        TerraIdentityError::InvalidSuccessionKind
    );

    let identity = &ctx.accounts.identity;
    let signer = ctx.accounts.signer.key();
    require!(
        signer == identity.owner || signer == identity.recovery,
        TerraIdentityError::NotAuthorized
    );
    require!(successor != identity.owner, TerraIdentityError::SuccessorIsOwner);

    let mut count: u8 = 0;
    for &v in validators.iter() {
        if v == Pubkey::default() {
            continue;
        }
        require!(v != identity.owner, TerraIdentityError::ValidatorOwnsAsset);
        count += 1;
    }
    require!(count > 0, TerraIdentityError::NoValidators);
    require!(
        (required_validations as usize) <= count as usize,
        TerraIdentityError::InvalidThreshold
    );
    require!(
        required_validations >= MIN_SUCCESSION_VALIDATIONS,
        TerraIdentityError::InvalidThreshold
    );

    if is_guardianship_kind(kind) {
        validate_guardianship_threshold(required_validations, count as usize)?;
    }

    let grace = if is_guardianship_kind(kind) {
        normalize_guardianship_grace(grace_secs)
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
pub fn endorse_succession(ctx: Context<EndorseSuccession>) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let succession = &mut ctx.accounts.succession;
    require!(
        now < succession.effective_at,
        TerraIdentityError::SuccessionAlreadyEffective
    );
    require!(
        (succession.validations_count as usize) < (succession.required as usize),
        TerraIdentityError::ValidationLimitReached
    );

    let validator = ctx.accounts.validator.key();
    require!(
        succession.validators.contains(&validator),
        TerraIdentityError::NotValidator
    );
    require!(
        validator != ctx.accounts.identity.owner,
        TerraIdentityError::ValidatorOwnsAsset
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
pub fn cancel_succession(ctx: Context<CancelSuccession>) -> Result<()> {
    let identity = &ctx.accounts.identity;
    let signer = ctx.accounts.signer.key();
    require!(
        signer == identity.owner || signer == identity.recovery,
        TerraIdentityError::NotAuthorized
    );
    require!(
        ctx.accounts.succession.effective_at > Clock::get()?.unix_timestamp,
        TerraIdentityError::SuccessionAlreadyEffective
    );

    emit!(SuccessionCancelled {
        identity: identity.key(),
        successor: ctx.accounts.succession.successor,
        kind: ctx.accounts.succession.kind,
    });
    Ok(())
}

/// Claim a passation once both grace period elapsed AND validators endorsed.
/// Updates Identity ownership. Parcels are repointed by terra_registry via
/// a separate instruction.
pub fn claim_succession(ctx: Context<ClaimSuccession>) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let succession = &ctx.accounts.succession;
    require!(
        succession.successor == ctx.accounts.signer.key(),
        TerraIdentityError::NotSuccessor
    );
    require!(
        now >= succession.effective_at,
        TerraIdentityError::SuccessionNotYetEffective
    );
    require!(
        succession.validations_count >= succession.required,
        TerraIdentityError::InsufficientValidations
    );

    let identity = &mut ctx.accounts.identity;
    require!(
        succession.identity == identity.key(),
        TerraIdentityError::IdentityMismatch
    );

    let previous = identity.owner;
    let successor = succession.successor;
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

/// Execute guardianship revocation after timelock expires.
pub fn execute_revoke_guardianship(ctx: Context<ExecuteRevokeGuardianship>) -> Result<()> {
    let identity = &mut ctx.accounts.identity;
    require!(
        identity.pending_revocation,
        TerraIdentityError::NoPendingRevocation
    );
    let now = Clock::get()?.unix_timestamp;
    require!(
        now >= identity.revoke_after,
        TerraIdentityError::RevocationTimelockNotExpired
    );
    require!(
        identity.pending_new_owner != Pubkey::default(),
        TerraIdentityError::NoPendingRevocation
    );

    let previous = identity.owner;
    identity.owner = identity.pending_new_owner;
    identity.recovery = Pubkey::default();
    identity.pending_revocation = false;
    identity.pending_new_owner = Pubkey::default();
    identity.revoke_after = 0;
    identity.updated_at = now;

    emit!(GuardianshipRevoked {
        identity: identity.key(),
        from: previous,
        to: identity.owner,
    });
    Ok(())
}

// ---------------------------------------------------------------------------
// Account contexts
// ---------------------------------------------------------------------------

#[derive(Accounts)]
#[instruction(identity_hash: [u8; 32], recovery: Pubkey)]
pub struct BindIdentity<'info> {
    #[account(
        init,
        payer = owner,
        space = 8 + Identity::INIT_SPACE,
        seeds = [b"identity", identity_hash.as_ref()],
        bump,
    )]
    pub identity: Account<'info, Identity>,
    #[account(mut)]
    pub owner: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct AttachParcel<'info> {
    #[account(mut)]
    pub identity: Account<'info, Identity>,
    pub owner: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(successor: Pubkey, kind: u8, grace_secs: i64, required_validations: u8, validators: [Pubkey; MAX_VALIDATORS])]
pub struct RequestSuccession<'info> {
    #[account(
        init,
        payer = signer,
        space = 8 + Succession::INIT_SPACE,
        seeds = [b"succession", identity.key().as_ref(), successor.as_ref()],
        bump,
    )]
    pub succession: Account<'info, Succession>,
    #[account(mut)]
    pub identity: Account<'info, Identity>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct EndorseSuccession<'info> {
    #[account(mut)]
    pub succession: Account<'info, Succession>,
    pub identity: Account<'info, Identity>,
    pub validator: Signer<'info>,
}

#[derive(Accounts)]
pub struct CancelSuccession<'info> {
    #[account(
        mut,
        close = signer,
    )]
    pub succession: Account<'info, Succession>,
    pub identity: Account<'info, Identity>,
    #[account(mut)]
    pub signer: Signer<'info>,
}

#[derive(Accounts)]
pub struct ClaimSuccession<'info> {
    #[account(
        mut,
        close = signer,
    )]
    pub succession: Account<'info, Succession>,
    #[account(mut)]
    pub identity: Account<'info, Identity>,
    #[account(mut)]
    pub signer: Signer<'info>,
}

#[derive(Accounts)]
pub struct ExecuteRevokeGuardianship<'info> {
    #[account(mut)]
    pub identity: Account<'info, Identity>,
    pub executor: Signer<'info>,
}

// ---------------------------------------------------------------------------
// Events
// ---------------------------------------------------------------------------

#[event]
pub struct IdentityBound {
    pub identity: Pubkey,
    pub identity_hash: [u8; 32],
    pub owner: Pubkey,
    pub recovery: Pubkey,
}

#[event]
pub struct ParcelAttached {
    pub identity: Pubkey,
    pub owner: Pubkey,
}

#[event]
pub struct SuccessionRequested {
    pub identity: Pubkey,
    pub successor: Pubkey,
    pub kind: u8,
    pub grace_secs: i64,
    pub required: u8,
    pub count: u8,
    pub effective_at: i64,
}

#[event]
pub struct SuccessionEndorsed {
    pub identity: Pubkey,
    pub successor: Pubkey,
    pub validator: Pubkey,
    pub validations_count: u8,
    pub required: u8,
}

#[event]
pub struct SuccessionCancelled {
    pub identity: Pubkey,
    pub successor: Pubkey,
    pub kind: u8,
}

#[event]
pub struct SuccessionClaimed {
    pub identity: Pubkey,
    pub from: Pubkey,
    pub to: Pubkey,
    pub kind: u8,
}

#[event]
pub struct GuardianshipRevoked {
    pub identity: Pubkey,
    pub from: Pubkey,
    pub to: Pubkey,
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[error_code]
pub enum TerraIdentityError {
    #[msg("Identity hash is all zeros")]
    EmptyIdentityHash,
    #[msg("Recovery wallet cannot be the zero address")]
    EmptyRecovery,
    #[msg("Successor wallet cannot be the zero address")]
    EmptySuccessor,
    #[msg("Invalid succession kind")]
    InvalidSuccessionKind,
    #[msg("Not authorized to perform this action")]
    NotAuthorized,
    #[msg("Successor cannot be the current owner")]
    SuccessorIsOwner,
    #[msg("Validator is the identity owner (self-dealing)")]
    ValidatorOwnsAsset,
    #[msg("No validators declared")]
    NoValidators,
    #[msg("Invalid threshold")]
    InvalidThreshold,
    #[msg("Identity key mismatch")]
    IdentityMismatch,
    #[msg("Successor is not the signer")]
    NotSuccessor,
    #[msg("Succession is not yet effective (grace period)")]
    SuccessionNotYetEffective,
    #[msg("Succession is already effective (cannot endorse/cancel)")]
    SuccessionAlreadyEffective,
    #[msg("Validation limit reached")]
    ValidationLimitReached,
    #[msg("Not enough validator endorsements")]
    InsufficientValidations,
    #[msg("Not a declared validator for this succession")]
    NotValidator,
    #[msg("Guardianship grace period is below the 90-day minimum")]
    GuardianshipGraceTooShort,
    #[msg("Guardianship requires at least 3 validator endorsements")]
    GuardianshipThresholdTooLow,
    #[msg("No pending revocation on this identity")]
    NoPendingRevocation,
    #[msg("Revocation timelock has not yet expired")]
    RevocationTimelockNotExpired,
}

// ---------------------------------------------------------------------------
// Program entry point
// ---------------------------------------------------------------------------

#[program]
pub mod terra_identity {
    use super::*;

    pub fn bind_identity(
        ctx: Context<BindIdentity>,
        identity_hash: [u8; 32],
        recovery: Pubkey,
    ) -> Result<()> {
        super::bind_identity(ctx, identity_hash, recovery)
    }

    pub fn attach_parcel(ctx: Context<AttachParcel>) -> Result<()> {
        super::attach_parcel(ctx)
    }

    pub fn request_succession(
        ctx: Context<RequestSuccession>,
        successor: Pubkey,
        kind: u8,
        grace_secs: i64,
        required_validations: u8,
        validators: [Pubkey; MAX_VALIDATORS],
    ) -> Result<()> {
        super::request_succession(ctx, successor, kind, grace_secs, required_validations, validators)
    }

    pub fn endorse_succession(ctx: Context<EndorseSuccession>) -> Result<()> {
        super::endorse_succession(ctx)
    }

    pub fn cancel_succession(ctx: Context<CancelSuccession>) -> Result<()> {
        super::cancel_succession(ctx)
    }

    pub fn claim_succession(ctx: Context<ClaimSuccession>) -> Result<()> {
        super::claim_succession(ctx)
    }

    pub fn execute_revoke_guardianship(ctx: Context<ExecuteRevokeGuardianship>) -> Result<()> {
        super::execute_revoke_guardianship(ctx)
    }
}
