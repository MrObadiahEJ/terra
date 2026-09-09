#![allow(unexpected_cfgs)]

use anchor_lang::prelude::*;

declare_id!("68urV9nGcRcoWT1QjzZfXuCnTS9921x2se1SybKJr1U4");

pub mod constants;
pub mod state;
pub mod errors;
pub mod events;
pub mod helpers;
pub mod instructions;

pub use constants::*;

// ---------------------------------------------------------------------------
// Program
// ---------------------------------------------------------------------------

#[program]
pub mod terra_identity_program {
    use super::*;

    pub fn bind_identity(
        ctx: Context<BindIdentity>,
        identity_hash: [u8; 32],
        recovery: Pubkey,
    ) -> Result<()> {
        instructions::bind_identity::bind_identity(ctx, identity_hash, recovery)
    }

    pub fn request_succession(
        ctx: Context<RequestSuccession>,
        successor: Pubkey,
        kind: u8,
        grace_secs: i64,
        required_validations: u8,
        validators: [Pubkey; MAX_VALIDATORS],
    ) -> Result<()> {
        instructions::succession::request_succession(
            ctx,
            successor,
            kind,
            grace_secs,
            required_validations,
            validators,
        )
    }

    pub fn endorse_succession(ctx: Context<EndorseSuccession>) -> Result<()> {
        instructions::succession::endorse_succession(ctx)
    }

    pub fn cancel_succession(ctx: Context<CancelSuccession>) -> Result<()> {
        instructions::succession::cancel_succession(ctx)
    }

    pub fn claim_succession(ctx: Context<ClaimSuccession>) -> Result<()> {
        instructions::succession::claim_succession(ctx)
    }

    pub fn request_court_guardianship(
        ctx: Context<RequestCourtGuardianship>,
        successor: Pubkey,
        grace_secs: i64,
        required_validations: u8,
        validators: [Pubkey; MAX_VALIDATORS],
        case_hash: [u8; 32],
        scope_notes: String,
    ) -> Result<()> {
        instructions::guardianship::request_court_guardianship(
            ctx,
            successor,
            grace_secs,
            required_validations,
            validators,
            case_hash,
            scope_notes,
        )
    }

    pub fn revoke_guardianship(
        ctx: Context<RevokeGuardianship>,
        new_owner: Pubkey,
    ) -> Result<()> {
        instructions::guardianship::revoke_guardianship(ctx, new_owner)
    }

    pub fn execute_revoke_guardianship(
        ctx: Context<ExecuteRevokeGuardianship>,
    ) -> Result<()> {
        instructions::guardianship::execute_revoke_guardianship(ctx)
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
        space = 8 + state::Identity::INIT_SPACE,
        seeds = [b"identity".as_ref(), identity_hash.as_ref()],
        bump
    )]
    pub identity: Account<'info, state::Identity>,
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
    pub identity: Account<'info, state::Identity>,
    #[account(
        init,
        payer = signer,
        space = 8 + state::Succession::INIT_SPACE,
        seeds = [b"succession".as_ref(), identity.key().as_ref(), successor.as_ref()],
        bump
    )]
    pub succession: Account<'info, state::Succession>,
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
    pub identity: Account<'info, state::Identity>,
    #[account(
        mut,
        seeds = [b"succession".as_ref(), succession.identity.as_ref(), succession.successor.as_ref()],
        bump
    )]
    pub succession: Account<'info, state::Succession>,
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
    pub identity: Account<'info, state::Identity>,
    #[account(
        mut,
        seeds = [b"succession".as_ref(), succession.identity.as_ref(), succession.successor.as_ref()],
        bump,
        close = signer
    )]
    pub succession: Account<'info, state::Succession>,
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
    pub identity: Account<'info, state::Identity>,
    #[account(
        mut,
        seeds = [b"succession".as_ref(), succession.identity.as_ref(), succession.successor.as_ref()],
        bump,
        close = signer
    )]
    pub succession: Account<'info, state::Succession>,
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
    pub identity: Account<'info, state::Identity>,
    #[account(
        init,
        payer = signer,
        space = 8 + state::Succession::INIT_SPACE,
        seeds = [b"succession".as_ref(), identity.key().as_ref(), successor.as_ref()],
        bump
    )]
    pub succession: Account<'info, state::Succession>,
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
    pub identity: Account<'info, state::Identity>,
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
    pub identity: Account<'info, state::Identity>,
    /// CHECK: validated as the pending new_owner in handler.
    pub new_owner: Signer<'info>,
}
