#![allow(unexpected_cfgs)]

use anchor_lang::prelude::*;

declare_id!("68urV9nGcRcoWT1QjzZfXuCnTS9921x2se1SybKJr1U4");

// ---------------------------------------------------------------------------
// Succession kind constants
// ---------------------------------------------------------------------------

pub mod succession_kind {
    pub const SUCCESSOR: u8 = 0;
    pub const RECOVERY: u8 = 1;
    pub const TRANSFER: u8 = 2;
    pub const GUARDIANSHIP: u8 = 3;
    pub const COURT_APPOINTED_GUARDIAN: u8 = 4;
    pub const MAX: u8 = COURT_APPOINTED_GUARDIAN;
}

// ---------------------------------------------------------------------------
// Grace period constants
// ---------------------------------------------------------------------------

pub const MIN_SUCCESSION_GRACE_SECS: i64 = 7 * 24 * 3600;
pub const MAX_SUCCESSION_GRACE_SECS: i64 = 180 * 24 * 3600;
pub const DEFAULT_SUCCESSION_GRACE_SECS: i64 = 30 * 24 * 3600;
pub const MIN_SUCCESSION_VALIDATIONS: u8 = 1;

// ---------------------------------------------------------------------------
// Guardianship constants (RFC-010)
// ---------------------------------------------------------------------------

pub const MIN_GUARDIANSHIP_GRACE_SECS: i64 = 90 * 24 * 3600;
pub const DEFAULT_GUARDIANSHIP_GRACE_SECS: i64 = 180 * 24 * 3600;
pub const MIN_GUARDIANSHIP_VALIDATIONS: u8 = 3;
pub const GUARDIANSHIP_REVOKE_TIMELOCK_SECS: i64 = 48 * 3600;
pub const MAX_VALIDATORS: usize = 8;

/// Maximum scope-notes length for court guardianship.
pub const MAX_SCOPE_NOTES_LEN: usize = 128;

// ---------------------------------------------------------------------------
// Account types
// ---------------------------------------------------------------------------

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
    pub parcel: Pubkey,
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
pub struct CourtGuardianshipRequested {
    pub identity: Pubkey,
    pub successor: Pubkey,
    pub grace_secs: i64,
    pub required: u8,
    pub count: u8,
    pub effective_at: i64,
    pub case_hash: [u8; 32],
    pub scope_notes: String,
}

#[event]
pub struct GuardianshipRevocationRequested {
    pub identity: Pubkey,
    pub previous_guardian: Pubkey,
    pub new_owner: Pubkey,
    pub requested_by: Pubkey,
    pub revoke_after: i64,
    pub block_time: i64,
}

#[event]
pub struct GuardianshipRevoked {
    pub identity: Pubkey,
    pub previous_guardian: Pubkey,
    pub new_owner: Pubkey,
    pub revoked_by: Pubkey,
    pub block_time: i64,
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[error_code]
pub enum IdentityError {
    #[msg("Identity hash is required")]
    EmptyIdentityHash,
    #[msg("Recovery wallet is required")]
    EmptyRecovery,
    #[msg("Successor wallet is required")]
    EmptySuccessor,
    #[msg("Invalid succession kind")]
    InvalidSuccessionKind,
    #[msg("Successor must differ from the current owner")]
    SuccessorIsOwner,
    #[msg("Not authorized to perform this action")]
    NotAuthorized,
    #[msg("Signing wallet is not a declared validator for this succession")]
    NotValidator,
    #[msg("A validator cannot be the owner of the asset being validated")]
    ValidatorOwnsAsset,
    #[msg("Succession has already become effective")]
    SuccessionAlreadyEffective,
    #[msg("No more validators may endorse this succession (limit reached)")]
    ValidationLimitReached,
    #[msg("Only the named successor may claim this succession")]
    NotSuccessor,
    #[msg("Succession is not yet effective")]
    SuccessionNotYetEffective,
    #[msg("Succession requires validator endorsements before it can be claimed")]
    InsufficientValidations,
    #[msg("Attestation does not belong to this parcel")]
    AttestationMismatch,
    #[msg("Identity owner does not match the parcel owner")]
    IdentityMismatch,
    #[msg("Only the current owner can perform this action")]
    NotOwner,
    #[msg("Required threshold exceeds the number of validators")]
    InvalidThreshold,
    #[msg("Court case hash is required")]
    EmptyCaseHash,
    #[msg("Notes exceed the maximum length of 128")]
    NotesTooLong,
    #[msg("Guardianship grace period is below the 90-day minimum")]
    GuardianshipGraceTooShort,
    #[msg("Guardianship requires at least 3 validator endorsements")]
    GuardianshipThresholdTooLow,
    #[msg("No proposal found")]
    NoProposalFound,
    #[msg("A revocation request is already pending for this identity")]
    GuardianshipAlreadyActive,
    #[msg("Settlement not yet effective")]
    SettlementNotYetEffective,
}

// ---------------------------------------------------------------------------
// Helper functions
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

pub fn validate_guardianship_threshold(required_validations: u8, declared: usize) -> bool {
    (required_validations as usize) >= MIN_GUARDIANSHIP_VALIDATIONS as usize
        && (required_validations as usize) <= declared
}

// ---------------------------------------------------------------------------
// Program
// ---------------------------------------------------------------------------

#[program]
pub mod terra_identity_program {
    use super::*;

    /// Bind a person (identified by a hashed credential) to a wallet.
    pub fn bind_identity(
        ctx: Context<BindIdentity>,
        identity_hash: [u8; 32],
        recovery: Pubkey,
    ) -> Result<()> {
        require!(
            !identity_hash.iter().all(|b| *b == 0),
            IdentityError::EmptyIdentityHash
        );
        require!(recovery != Pubkey::default(), IdentityError::EmptyRecovery);

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

    /// Request a wallet passation (succession, recovery, or deliberate transfer).
    pub fn request_succession(
        ctx: Context<RequestSuccession>,
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
    pub fn endorse_succession(ctx: Context<EndorseSuccession>) -> Result<()> {
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
    pub fn cancel_succession(ctx: Context<CancelSuccession>) -> Result<()> {
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
    /// number of validators have endorsed it. The successor becomes the identity's
    /// new owner.
    pub fn claim_succession(ctx: Context<ClaimSuccession>) -> Result<()> {
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

    /// Request a court-appointed guardianship with an explicit case_hash binding.
    pub fn request_court_guardianship(
        ctx: Context<RequestCourtGuardianship>,
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
    /// Sets a 48-hour timelock. The revoker must be the recovery wallet.
    pub fn revoke_guardianship(
        ctx: Context<RevokeGuardianship>,
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
    pub fn execute_revoke_guardianship(ctx: Context<ExecuteRevokeGuardianship>) -> Result<()> {
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
}

// ---------------------------------------------------------------------------
// Account contexts
// ---------------------------------------------------------------------------

#[derive(Accounts)]
#[instruction(identity_hash: [u8; 32])]
pub struct BindIdentity<'info> {
    #[account(
        init,
        payer = owner,
        space = 8 + Identity::INIT_SPACE,
        seeds = [b"identity".as_ref(), identity_hash.as_ref()],
        bump
    )]
    pub identity: Account<'info, Identity>,
    #[account(mut)]
    pub owner: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(successor: Pubkey, kind: u8, grace_secs: i64, required_validations: u8, validators: [Pubkey; MAX_VALIDATORS])]
pub struct RequestSuccession<'info> {
    #[account(
        mut,
        seeds = [b"identity".as_ref(), identity.identity_hash.as_ref()],
        bump
    )]
    pub identity: Account<'info, Identity>,
    #[account(
        init,
        payer = signer,
        space = 8 + Succession::INIT_SPACE,
        seeds = [b"succession".as_ref(), identity.key().as_ref(), successor.as_ref()],
        bump
    )]
    pub succession: Account<'info, Succession>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct EndorseSuccession<'info> {
    #[account(
        mut,
        seeds = [b"identity".as_ref(), identity.identity_hash.as_ref()],
        bump
    )]
    pub identity: Account<'info, Identity>,
    #[account(
        mut,
        seeds = [b"succession".as_ref(), succession.identity.as_ref(), succession.successor.as_ref()],
        bump
    )]
    pub succession: Account<'info, Succession>,
    pub validator: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CancelSuccession<'info> {
    #[account(
        mut,
        seeds = [b"identity".as_ref(), identity.identity_hash.as_ref()],
        bump
    )]
    pub identity: Account<'info, Identity>,
    #[account(
        mut,
        seeds = [b"succession".as_ref(), succession.identity.as_ref(), succession.successor.as_ref()],
        bump,
        close = signer
    )]
    pub succession: Account<'info, Succession>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ClaimSuccession<'info> {
    #[account(
        mut,
        seeds = [b"identity".as_ref(), identity.identity_hash.as_ref()],
        bump
    )]
    pub identity: Account<'info, Identity>,
    #[account(
        mut,
        seeds = [b"succession".as_ref(), succession.identity.as_ref(), succession.successor.as_ref()],
        bump,
        close = signer
    )]
    pub succession: Account<'info, Succession>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(successor: Pubkey, grace_secs: i64, required_validations: u8, validators: [Pubkey; MAX_VALIDATORS], case_hash: [u8; 32], scope_notes: String)]
pub struct RequestCourtGuardianship<'info> {
    #[account(
        mut,
        seeds = [b"identity".as_ref(), identity.identity_hash.as_ref()],
        bump
    )]
    pub identity: Account<'info, Identity>,
    #[account(
        init,
        payer = signer,
        space = 8 + Succession::INIT_SPACE,
        seeds = [b"succession".as_ref(), identity.key().as_ref(), successor.as_ref()],
        bump
    )]
    pub succession: Account<'info, Succession>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(new_owner: Pubkey)]
pub struct RevokeGuardianship<'info> {
    #[account(
        mut,
        seeds = [b"identity".as_ref(), identity.identity_hash.as_ref()],
        bump
    )]
    pub identity: Account<'info, Identity>,
    pub revoker: Signer<'info>,
    /// CHECK: the target new owner — validated in handler.
    pub new_owner: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct ExecuteRevokeGuardianship<'info> {
    #[account(
        mut,
        seeds = [b"identity".as_ref(), identity.identity_hash.as_ref()],
        bump
    )]
    pub identity: Account<'info, Identity>,
    /// CHECK: validated as the pending new_owner in handler.
    pub new_owner: Signer<'info>,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn succession_kind_reserved_variants_are_contiguous() {
        assert_eq!(succession_kind::SUCCESSOR, 0);
        assert_eq!(succession_kind::RECOVERY, 1);
        assert_eq!(succession_kind::TRANSFER, 2);
        assert_eq!(succession_kind::GUARDIANSHIP, 3);
        assert_eq!(succession_kind::COURT_APPOINTED_GUARDIAN, 4);
        assert_eq!(
            succession_kind::MAX,
            succession_kind::COURT_APPOINTED_GUARDIAN
        );
    }

    #[test]
    fn is_guardianship_kind_works() {
        assert!(!is_guardianship_kind(succession_kind::SUCCESSOR));
        assert!(!is_guardianship_kind(succession_kind::RECOVERY));
        assert!(!is_guardianship_kind(succession_kind::TRANSFER));
        assert!(is_guardianship_kind(succession_kind::GUARDIANSHIP));
        assert!(is_guardianship_kind(succession_kind::COURT_APPOINTED_GUARDIAN));
    }

    #[test]
    fn normalize_guardianship_grace_defaults_to_180d() {
        assert_eq!(normalize_guardianship_grace(0), DEFAULT_GUARDIANSHIP_GRACE_SECS);
    }

    #[test]
    fn normalize_guardianship_grace_clamps() {
        assert_eq!(
            normalize_guardianship_grace(120 * 24 * 3600),
            120 * 24 * 3600
        );
        assert_eq!(
            normalize_guardianship_grace(50 * 24 * 3600),
            MIN_GUARDIANSHIP_GRACE_SECS
        );
        assert_eq!(
            normalize_guardianship_grace(200 * 24 * 3600),
            MAX_SUCCESSION_GRACE_SECS
        );
    }

    #[test]
    fn validate_guardianship_threshold_works() {
        assert!(validate_guardianship_threshold(3, 3));
        assert!(validate_guardianship_threshold(3, 5));
        assert!(!validate_guardianship_threshold(2, 3));
        assert!(!validate_guardianship_threshold(4, 3));
    }
}
