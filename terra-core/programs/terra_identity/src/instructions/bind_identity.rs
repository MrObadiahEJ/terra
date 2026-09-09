use anchor_lang::prelude::*;

use crate::errors::IdentityError;
use crate::events::*;

/// Bind a person (identified by a hashed credential) to a wallet.
pub fn bind_identity(
    ctx: Context<crate::BindIdentity>,
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
