#![allow(unexpected_cfgs)]

use anchor_lang::prelude::*;

declare_id!("HyjpaHY9yQg1KneLZGjpLDQfP9WV4LUEbNZcb7C4Vo6p");

pub mod constants;
pub mod errors;
pub mod events;
pub mod helpers;
pub mod instructions;
pub mod state;

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

    pub fn revoke_guardianship(ctx: Context<RevokeGuardianship>, new_owner: Pubkey) -> Result<()> {
        instructions::guardianship::revoke_guardianship(ctx, new_owner)
    }

    pub fn execute_revoke_guardianship(ctx: Context<ExecuteRevokeGuardianship>) -> Result<()> {
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

// ---------------------------------------------------------------------------
// SDK metadata (directive §3 / §12): declares the Terra Identity SDK
// identity, capabilities, schema versions, dependencies, security
// assumptions, and upgrade behavior so the SDK is an explicit, versioned
// standard rather than an implicit crate.
// ---------------------------------------------------------------------------
pub mod sdk_meta {
    //! Metadata describing the `terra-identity` SDK standard.

    /// SDK crate name.
    pub fn sdk_name() -> &'static str {
        "terra-identity"
    }
    /// SDK crate version (semver).
    pub fn sdk_version() -> &'static str {
        env!("CARGO_PKG_VERSION")
    }

    /// Capabilities exposed by this SDK module.
    pub fn sdk_capabilities() -> &'static [&'static str] {
        &[
            "identity-binding: bind_identity (identity_hash -> owner)",
            "succession: request/endorse/cancel/claim identity succession",
            "guardianship: request/execute/revoke court-appointed guardianship",
        ]
    }

    /// Protocol schemas implemented by this SDK, with versions.
    pub fn sdk_schema_versions() -> &'static [(&'static str, &'static str)] {
        &[
            ("RFC-006 Cross-Border Identity Bridge", "1.0"),
            ("Identity PDA schema", "1"),
            ("Succession PDA schema", "1"),
        ]
    }

    /// Direct crate dependencies.
    pub fn sdk_dependencies() -> &'static [(&'static str, &'static str)] {
        &[("anchor-lang", "0.30")]
    }

    /// Security assumptions made by this SDK.
    pub fn sdk_security_assumptions() -> &'static [&'static str] {
        &[
            "Identity PDA is derived from [b\"identity\", identity_hash]; owner is the sole authority.",
            "Succession requires quorum of validators before claim can execute.",
            "Guardianship revocation requires court order scope and pending revocation.",
        ]
    }

    /// Upgrade / migration behavior.
    pub fn sdk_upgrade_migration() -> &'static str {
        "Schema versions are additive. Identity PDA seeds are stable; new \
         instruction accounts are appended, never replaced. A change to one \
         SDK module does not force any other module or application to upgrade \
         (directive §4)."
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn sdk_metadata_is_well_formed() {
            assert!(!sdk_name().is_empty());
            assert!(!sdk_version().is_empty());
            assert!(!sdk_capabilities().is_empty());
            assert!(!sdk_schema_versions().is_empty());
            assert!(!sdk_security_assumptions().is_empty());
            assert!(!sdk_upgrade_migration().is_empty());
            for (name, version) in sdk_schema_versions() {
                assert!(!name.is_empty());
                assert!(!version.is_empty());
            }
            assert!(sdk_capabilities()
                .iter()
                .any(|c| c.contains("identity-binding")));
            assert!(sdk_schema_versions()
                .iter()
                .any(|(n, _)| n == "RFC-006 Cross-Border Identity Bridge"));
        }
    }
}
