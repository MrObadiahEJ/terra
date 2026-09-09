use anchor_lang::prelude::*;

use crate::TerraError;

// ---------------------------------------------------------------------------
// Cross-border verification status
// ---------------------------------------------------------------------------

pub mod cross_border_verification_status {
    pub const PENDING: u8 = 0;
    pub const VERIFIED: u8 = 1;
    pub const REVOKED: u8 = 2;
    pub const MAX: u8 = REVOKED;
}

// ---------------------------------------------------------------------------
// CrossBorderVerification account
// ---------------------------------------------------------------------------

/// Bridges cross-border identity bindings into the verification pipeline.
///
/// PDA: `["cross_border_verification", binding]`
#[account]
#[derive(InitSpace)]
pub struct CrossBorderVerification {
    /// The JurisdictionBinding account being verified.
    pub binding: Pubkey,
    /// Associated Claim (if any).
    pub claim: Pubkey,
    /// Associated VerificationSession (if any).
    pub session: Pubkey,
    /// The Jurisdiction account.
    pub jurisdiction: Pubkey,
    /// Current status (0=pending, 1=verified, 2=revoked).
    pub status: u8,
    /// Who verified this binding.
    pub verified_by: Pubkey,
    pub created_at: i64,
    pub verified_at: i64,
}

// ---------------------------------------------------------------------------
// Instruction handlers
// ---------------------------------------------------------------------------

/// Link a JurisdictionBinding to a VerificationSession. Creates a
/// CrossBorderVerification account. Only callable if the binding exists
/// and is not revoked.
pub fn link_cross_border_to_session(
    ctx: Context<crate::LinkCrossBorderToSession>,
) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let cross_border = &mut ctx.accounts.cross_border_verification;
    cross_border.binding = ctx.accounts.binding.key();
    cross_border.claim = ctx.accounts.claim.key();
    cross_border.session = ctx.accounts.session.key();
    cross_border.jurisdiction = ctx.accounts.jurisdiction.key();
    cross_border.status = cross_border_verification_status::PENDING;
    cross_border.verified_by = ctx.accounts.caller.key();
    cross_border.created_at = now;
    cross_border.verified_at = 0;

    emit!(crate::CrossBorderLinked {
        cross_border_verification: cross_border.key(),
        binding: cross_border.binding,
        claim: cross_border.claim,
        session: cross_border.session,
        jurisdiction: cross_border.jurisdiction,
        linked_by: cross_border.verified_by,
        created_at: now,
    });
    Ok(())
}

/// Mark a cross-border verification as verified. Only callable by the
/// original linker or the registry admin.
pub fn verify_cross_border(
    ctx: Context<crate::VerifyCrossBorder>,
) -> Result<()> {
    let cross_border = &mut ctx.accounts.cross_border_verification;
    require!(
        cross_border.status == cross_border_verification_status::PENDING,
        TerraError::InvalidCrossBorderStatus
    );

    let now = Clock::get()?.unix_timestamp;
    cross_border.status = cross_border_verification_status::VERIFIED;
    cross_border.verified_at = now;

    emit!(crate::CrossBorderVerified {
        cross_border_verification: cross_border.key(),
        binding: cross_border.binding,
        claim: cross_border.claim,
        verified_by: cross_border.verified_by,
        verified_at: now,
    });
    Ok(())
}

/// Revoke a cross-border verification. Only callable by the registry admin.
pub fn revoke_cross_border(
    ctx: Context<crate::RevokeCrossBorder>,
) -> Result<()> {
    let cross_border = &mut ctx.accounts.cross_border_verification;
    require!(
        cross_border.status == cross_border_verification_status::PENDING
            || cross_border.status == cross_border_verification_status::VERIFIED,
        TerraError::InvalidCrossBorderStatus
    );

    let now = Clock::get()?.unix_timestamp;
    cross_border.status = cross_border_verification_status::REVOKED;

    emit!(crate::CrossBorderRevoked {
        cross_border_verification: cross_border.key(),
        binding: cross_border.binding,
        claim: cross_border.claim,
        revoked_at: now,
    });
    Ok(())
}
