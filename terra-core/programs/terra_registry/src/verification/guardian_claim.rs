use anchor_lang::prelude::*;

use crate::TerraError;

// ---------------------------------------------------------------------------
// Guardian type
// ---------------------------------------------------------------------------

pub mod guardian_type {
    pub const COURT: u8 = 0;
    pub const COUNCIL: u8 = 1;
    pub const EMERGENCY: u8 = 2;
    pub const MAX: u8 = EMERGENCY;
}

// ---------------------------------------------------------------------------
// Guardian claim status
// ---------------------------------------------------------------------------

pub mod guardian_claim_status {
    pub const PENDING: u8 = 0;
    pub const ACTIVE: u8 = 1;
    pub const RESOLVED: u8 = 2;
    pub const DISPUTED: u8 = 3;
    pub const MAX: u8 = DISPUTED;
}

// ---------------------------------------------------------------------------
// GuardianClaim account
// ---------------------------------------------------------------------------

/// Bridge between the guardian/court system and the verification pipeline.
/// When a guardianship event occurs (e.g. court-ordered guardianship via
/// `terra_identity`), a GuardianClaim can be created to link that event
/// into the verification pipeline.
///
/// PDA: `["guardian_claim", claim]`
#[account]
#[derive(InitSpace)]
pub struct GuardianClaim {
    /// The associated Claim account.
    pub claim: Pubkey,
    /// The identity under guardianship.
    pub identity: Pubkey,
    /// Who triggered the guardianship (court/guardian wallet).
    pub triggered_by: Pubkey,
    /// SHA-256 of the court case document.
    pub case_hash: [u8; 32],
    /// Guardian type (0=court, 1=council, 2=emergency).
    pub guardian_type: u8,
    /// Current status (0=pending, 1=active, 2=resolved, 3=disputed).
    pub status: u8,
    pub created_at: i64,
    pub resolved_at: i64,
}

// ---------------------------------------------------------------------------
// Instruction handlers
// ---------------------------------------------------------------------------

/// Create a GuardianClaim linking a guardianship event to the verification
/// pipeline. The caller must be a registered validator. The claim account
/// must already exist (created via `create_claim`).
pub fn create_guardian_claim(
    ctx: Context<crate::CreateGuardianClaim>,
    case_hash: [u8; 32],
    guardian_type: u8,
) -> Result<()> {
    require!(
        !case_hash.iter().all(|b| *b == 0),
        TerraError::EmptyCaseHash
    );
    require!(
        guardian_type <= guardian_type::MAX,
        TerraError::InvalidGuardianType
    );

    let registry = &ctx.accounts.registry;
    let caller = ctx.accounts.caller.key();
    require!(
        registry.validators.contains(&caller),
        TerraError::NotValidator
    );

    let now = Clock::get()?.unix_timestamp;
    let guardian_claim = &mut ctx.accounts.guardian_claim;
    guardian_claim.claim = ctx.accounts.claim.key();
    guardian_claim.identity = ctx.accounts.identity.key();
    guardian_claim.triggered_by = caller;
    guardian_claim.case_hash = case_hash;
    guardian_claim.guardian_type = guardian_type;
    guardian_claim.status = guardian_claim_status::PENDING;
    guardian_claim.created_at = now;
    guardian_claim.resolved_at = 0;

    emit!(crate::GuardianClaimCreated {
        guardian_claim: guardian_claim.key(),
        claim: guardian_claim.claim,
        identity: guardian_claim.identity,
        triggered_by: caller,
        case_hash,
        guardian_type,
        created_at: now,
    });
    Ok(())
}

/// Resolve a guardian claim. Only callable by the original triggerer or
/// the registry admin. Transitions status to RESOLVED.
pub fn resolve_guardian_claim(
    ctx: Context<crate::ResolveGuardianClaim>,
) -> Result<()> {
    let guardian_claim = &mut ctx.accounts.guardian_claim;
    require!(
        guardian_claim.status == guardian_claim_status::PENDING
            || guardian_claim.status == guardian_claim_status::ACTIVE,
        TerraError::InvalidGuardianClaimStatus
    );

    let now = Clock::get()?.unix_timestamp;
    guardian_claim.status = guardian_claim_status::RESOLVED;
    guardian_claim.resolved_at = now;

    emit!(crate::GuardianClaimResolved {
        guardian_claim: guardian_claim.key(),
        claim: guardian_claim.claim,
        identity: guardian_claim.identity,
        resolved_at: now,
    });
    Ok(())
}

/// Dispute a guardian claim. Only callable by the identity's owner or
/// recovery wallet. Transitions status to DISPUTED.
pub fn dispute_guardian_claim(
    ctx: Context<crate::DisputeGuardianClaim>,
) -> Result<()> {
    let guardian_claim = &mut ctx.accounts.guardian_claim;
    require!(
        guardian_claim.status == guardian_claim_status::PENDING
            || guardian_claim.status == guardian_claim_status::ACTIVE,
        TerraError::InvalidGuardianClaimStatus
    );

    let now = Clock::get()?.unix_timestamp;
    guardian_claim.status = guardian_claim_status::DISPUTED;

    emit!(crate::GuardianClaimDisputed {
        guardian_claim: guardian_claim.key(),
        claim: guardian_claim.claim,
        identity: guardian_claim.identity,
        disputed_at: now,
    });
    Ok(())
}
