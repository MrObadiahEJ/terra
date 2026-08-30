use anchor_lang::prelude::*;

declare_id!("GaEDbktvpZ3qiqp4PmFgHwDSa6JsFfVjXFqNb2nTbage");

pub mod parcel_status {
    pub const PENDING: u8 = 0;
    pub const REGISTERED: u8 = 1;
    pub const FOR_SALE: u8 = 2;
    pub const TRANSFERRED: u8 = 3;
}

pub mod right_kind {
    /// Country-agnostic right categories (STDM-inspired).
    pub const OWNERSHIP: u8 = 0;
    pub const USAGE: u8 = 1;
    pub const EASEMENT: u8 = 2;
    pub const SERVITUDE: u8 = 3;
    pub const LIEN: u8 = 4;
    pub const MAX: u8 = LIEN;
}

pub mod infra_flag {
    /// Bitmask of infrastructure available at a parcel. Country-agnostic.
    pub const WASTEWATER: u16 = 1 << 0;
    pub const WATER: u16 = 1 << 1;
    pub const POWER: u16 = 1 << 2;
    pub const GAS: u16 = 1 << 3;
    pub const TELECOM: u16 = 1 << 4;
    pub const ROAD_ACCESS: u16 = 1 << 5;
    pub const BUILDING: u16 = 1 << 6;
    pub const ALL: u16 = (1 << 7) - 1;
}

#[account]
#[derive(InitSpace)]
pub struct Parcel {
    pub id: [u8; 32],
    pub owner: Pubkey,
    #[max_len(64)]
    pub name: String,
    pub geometry_hash: [u8; 32],
    pub status: u8,
    /// Monotonic nonce for the parcel's Rights PDAs. Never decremented.
    pub rights_count: u8,
    pub infrastructure_flags: u16,
    /// sha-256 canonical digest over the off-chain infra/access validation
    /// (parcel id, flags, reachability metrics). Tamper-evidence anchor.
    pub access_hash: [u8; 32],
    pub created_at: i64,
    pub updated_at: i64,
}

/// A right attached to a parcel (ownership, usage, easement, ...).
///
/// PDA: `["rights", parcel, nonce]`. One or more Rights may exist per parcel;
/// `nonce` is allocated from `Parcel::rights_count` at grant time.
#[account]
#[derive(InitSpace)]
pub struct Rights {
    pub parcel: Pubkey,
    pub rights_kind: u8,
    /// Party holding the right.
    pub holder: Pubkey,
    /// Party who granted the right (invariably the parcel owner).
    pub granter: Pubkey,
    pub created_at: i64,
    /// Unix timestamp; 0 means no expiration.
    pub expires_at: i64,
    #[max_len(128)]
    pub notes: String,
}

#[program]
pub mod terra_registry {
    use super::*;

    /// Register a new parcel on-chain. The signer becomes its owner.
    ///
    /// `id` is a caller-provided unique 32-byte identifier (e.g. a SHA-256 of
    /// the parcel geometry). It is also the PDA seed, so it can never change.
    pub fn register_parcel(
        ctx: Context<RegisterParcel>,
        id: [u8; 32],
        name: String,
        geometry_hash: [u8; 32],
    ) -> Result<()> {
        require!(!id.iter().all(|b| *b == 0), TerraError::InvalidId);
        require!(!name.is_empty(), TerraError::EmptyName);
        require!(
            !geometry_hash.iter().all(|b| *b == 0),
            TerraError::EmptyGeometryHash
        );

        let parcel = &mut ctx.accounts.parcel;
        let now = Clock::get()?.unix_timestamp;

        parcel.id = id;
        parcel.owner = ctx.accounts.owner.key();
        parcel.name = name;
        parcel.geometry_hash = geometry_hash;
        parcel.status = parcel_status::REGISTERED;
        parcel.created_at = now;
        parcel.updated_at = now;

        emit!(ParcelRegistered {
            id,
            owner: parcel.owner,
        });
        Ok(())
    }

    /// Transfer ownership of a parcel. Only the current owner can sign.
    pub fn transfer_parcel(ctx: Context<TransferParcel>) -> Result<()> {
        let parcel = &mut ctx.accounts.parcel;
        require!(
            parcel.owner == ctx.accounts.owner.key(),
            TerraError::NotOwner
        );

        let from = parcel.owner;
        let to = ctx.accounts.new_owner.key();
        parcel.owner = to;
        parcel.updated_at = Clock::get()?.unix_timestamp;

        emit!(ParcelTransferred { id: parcel.id, from, to });
        Ok(())
    }

    /// Update a parcel's status (e.g. for-sale). Owner-only.
    pub fn update_status(ctx: Context<UpdateStatus>, status: u8) -> Result<()> {
        require!(
            ctx.accounts.parcel.owner == ctx.accounts.owner.key(),
            TerraError::NotOwner
        );
        require!(
            status <= parcel_status::TRANSFERRED,
            TerraError::InvalidStatus
        );

        let parcel = &mut ctx.accounts.parcel;
        parcel.status = status;
        parcel.updated_at = Clock::get()?.unix_timestamp;
        Ok(())
    }

    /// Grant a right on a parcel to `holder`. Owner-only.
    ///
    /// `nonce` must equal the parcel's current `rights_count`, which is
    /// incremented so every right gets a unique PDA.
    pub fn grant_right(
        ctx: Context<GrantRight>,
        nonce: u8,
        rights_kind: u8,
        holder: Pubkey,
        expires_at: i64,
        notes: String,
    ) -> Result<()> {
        let parcel = &mut ctx.accounts.parcel;
        require!(
            parcel.owner == ctx.accounts.owner.key(),
            TerraError::NotOwner
        );
        require!(rights_kind <= right_kind::MAX, TerraError::InvalidRightKind);
        require!(nonce == parcel.rights_count, TerraError::InvalidNonce);
        require!(
            (parcel.rights_count as u16) < u8::MAX as u16,
            TerraError::RightsLimitExceeded
        );
        require!(notes.len() <= 128, TerraError::NotesTooLong);
        let now = Clock::get()?.unix_timestamp;
        if expires_at != 0 {
            require!(expires_at > now, TerraError::InvalidExpiry);
        }

        let rights = &mut ctx.accounts.rights;
        rights.parcel = parcel.key();
        rights.rights_kind = rights_kind;
        rights.holder = holder;
        rights.granter = ctx.accounts.owner.key();
        rights.created_at = now;
        rights.expires_at = expires_at;
        rights.notes = notes;

        parcel.rights_count += 1;

        emit!(RightGranted {
            parcel: parcel.key(),
            rights_kind,
            holder,
        });
        Ok(())
    }

    /// Revoke a previously granted right. The parcel owner or the original
    /// granter may revoke. The account is closed and its lamports returned.
    pub fn revoke_right(ctx: Context<RevokeRight>, _nonce: u8) -> Result<()> {
        let rights = &ctx.accounts.rights;
        require!(
            ctx.accounts.parcel.owner == ctx.accounts.owner.key()
                || rights.granter == ctx.accounts.owner.key(),
            TerraError::NotAuthorized
        );

        emit!(RightRevoked {
            parcel: rights.parcel,
            rights_kind: rights.rights_kind,
            holder: rights.holder,
        });
        Ok(())
    }

    /// Set the parcel's infrastructure flag bitmask together with the canonical
    /// access digest produced by the off-chain validation engine. Owner-only.
    ///
    /// `access_hash` must be non-zero and match the digests the off-chain
    /// engine derives for these flags on the parcel geometry.
    pub fn update_infrastructure(
        ctx: Context<UpdateInfrastructure>,
        flags: u16,
        access_hash: [u8; 32],
    ) -> Result<()> {
        require!(
            ctx.accounts.parcel.owner == ctx.accounts.owner.key(),
            TerraError::NotOwner
        );
        require!(
            flags & !infra_flag::ALL == 0,
            TerraError::InvalidInfrastructureFlags
        );
        require!(
            !access_hash.iter().all(|b| *b == 0),
            TerraError::EmptyAccessHash
        );

        let parcel = &mut ctx.accounts.parcel;
        parcel.infrastructure_flags = flags;
        parcel.access_hash = access_hash;
        parcel.updated_at = Clock::get()?.unix_timestamp;

        emit!(InfrastructureUpdated {
            parcel: parcel.key(),
            flags,
            access_hash,
        });
        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(id: [u8; 32])]
pub struct RegisterParcel<'info> {
    #[account(
        init,
        payer = owner,
        space = 8 + Parcel::INIT_SPACE,
        seeds = [b"parcel".as_ref(), id.as_ref()],
        bump
    )]
    pub parcel: Account<'info, Parcel>,
    #[account(mut)]
    pub owner: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct TransferParcel<'info> {
    #[account(
        mut,
        seeds = [b"parcel".as_ref(), parcel.id.as_ref()],
        bump
    )]
    pub parcel: Account<'info, Parcel>,
    pub owner: Signer<'info>,
    pub new_owner: SystemAccount<'info>,
}

#[derive(Accounts)]
pub struct UpdateStatus<'info> {
    #[account(
        mut,
        seeds = [b"parcel".as_ref(), parcel.id.as_ref()],
        bump
    )]
    pub parcel: Account<'info, Parcel>,
    pub owner: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(nonce: u8, rights_kind: u8, holder: Pubkey)]
pub struct GrantRight<'info> {
    #[account(
        mut,
        seeds = [b"parcel".as_ref(), parcel.id.as_ref()],
        bump
    )]
    pub parcel: Account<'info, Parcel>,
    #[account(
        init,
        payer = owner,
        space = 8 + Rights::INIT_SPACE,
        seeds = [b"rights".as_ref(), parcel.key().as_ref(), &[nonce]],
        bump
    )]
    pub rights: Account<'info, Rights>,
    #[account(mut)]
    pub owner: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(nonce: u8)]
pub struct RevokeRight<'info> {
    #[account(
        mut,
        seeds = [b"parcel".as_ref(), parcel.id.as_ref()],
        bump
    )]
    pub parcel: Account<'info, Parcel>,
    #[account(
        mut,
        seeds = [b"rights".as_ref(), parcel.key().as_ref(), &[nonce]],
        bump,
        close = owner
    )]
    pub rights: Account<'info, Rights>,
    #[account(mut)]
    pub owner: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateInfrastructure<'info> {
    #[account(
        mut,
        seeds = [b"parcel".as_ref(), parcel.id.as_ref()],
        bump
    )]
    pub parcel: Account<'info, Parcel>,
    pub owner: Signer<'info>,
}

#[event]
pub struct ParcelRegistered {
    pub id: [u8; 32],
    pub owner: Pubkey,
}

#[event]
pub struct ParcelTransferred {
    pub id: [u8; 32],
    pub from: Pubkey,
    pub to: Pubkey,
}

#[event]
pub struct RightGranted {
    pub parcel: Pubkey,
    pub rights_kind: u8,
    pub holder: Pubkey,
}

#[event]
pub struct RightRevoked {
    pub parcel: Pubkey,
    pub rights_kind: u8,
    pub holder: Pubkey,
}

#[event]
pub struct InfrastructureUpdated {
    pub parcel: Pubkey,
    pub flags: u16,
    pub access_hash: [u8; 32],
}

#[error_code]
pub enum TerraError {
    #[msg("Parcel id cannot be all zeros")]
    InvalidId,
    #[msg("Parcel name cannot be empty")]
    EmptyName,
    #[msg("Geometry hash is required")]
    EmptyGeometryHash,
    #[msg("Only the current owner can perform this action")]
    NotOwner,
    #[msg("Invalid parcel status")]
    InvalidStatus,
    #[msg("Invalid right kind")]
    InvalidRightKind,
    #[msg("Nonce does not match the parcel's rights_count")]
    InvalidNonce,
    #[msg("Rights limit reached")]
    RightsLimitExceeded,
    #[msg("Notes exceed the maximum length of 128")]
    NotesTooLong,
    #[msg("Expiry must be in the future")]
    InvalidExpiry,
    #[msg("Not authorized to perform this action")]
    NotAuthorized,
    #[msg("Invalid infrastructure flags")]
    InvalidInfrastructureFlags,
    #[msg("Access hash is required")]
    EmptyAccessHash,
}
