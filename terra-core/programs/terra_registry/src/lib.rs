use anchor_lang::prelude::*;
use anchor_lang::Discriminator;

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

/// Maximum number of validators that can approve a single attestation.
/// Keeps the account space bounded and predictable.
pub const MAX_VALIDATORS: usize = 8;

/// An on-chain attestation that binds a set of off-chain documents/data to a
/// parcel and records *who* (which wallets) must validate a transaction.
///
/// PDA: `["attestation", parcel, specifier]`. The heavy payload — actual
/// documents and per-validator Ed25519 signatures — lives off-chain, but it is
/// anchored here by `content_hash`, and each validator's public key is recorded
/// so that any signature can be independently verified against this list.
#[account]
#[derive(InitSpace)]
pub struct Attestation {
    pub parcel: Pubkey,
    /// 32-byte specifier (e.g. sha256 over the artifact/signing-session id).
    pub specifier: [u8; 32],
    /// sha-256 over the off-chain payload (documents, deed, survey, ...).
    pub content_hash: [u8; 32],
    /// Required threshold of validator signatures to consider this validated.
    pub required: u8,
    /// Number of validator keys currently registered (<= MAX_VALIDATORS).
    pub count: u8,
    /// Monotonic rotation counter. Each rotate_validators bumps it so a
    /// reconstituted validator set is provably newer than the previous one.
    pub version: u8,
    pub created_at: i64,
    pub updated_at: i64,
    pub validators: [Pubkey; MAX_VALIDATORS],
}

/// Grace period (seconds) before a queued succession can be claimed. Gives the
/// original owner (or a contested party) a window to cancel the passation.
pub const SUCCESSION_GRACE_PERIOD_SECS: i64 = 7 * 24 * 3600; // 7 days

/// Maximum number of successions a wallet may have in flight.
pub const MAX_PENDING_SUCCESSIONS: usize = 8;

pub mod succession_kind {
    /// Wallet passation to an heir/beneficiary (estate / inheritance).
    pub const SUCCESSOR: u8 = 0;
    /// Passation because the active key was lost/stolen (recovery).
    pub const RECOVERY: u8 = 1;
    /// Passation of a parcel's control (sale / deliberate transfer).
    pub const TRANSFER: u8 = 2;
    pub const MAX: u8 = TRANSFER;
}

/// Binds a person (via a hashed identity credential) to a wallet the person
/// actually holds, plus a recovery wallet. This is the resolvable on-chain link
/// behind "who owns this." A provisioned wallet is exported to the person; the
/// program only ever sees the public keys.
///
/// PDA: `["identity", identity_hash]`.
#[account]
#[derive(InitSpace)]
pub struct Identity {
    /// 32-byte hash over the person's identity credential (e.g. national ID),
    /// so the credential itself never lives on-chain.
    pub identity_hash: [u8; 32],
    /// The active wallet acting on behalf of this identity.
    pub owner: Pubkey,
    /// A separate wallet the person also controls (backup / recovery). Used to
    /// request a recovery passation if the main key is lost.
    pub recovery: Pubkey,
    /// Number of parcels currently owned by this identity.
    pub parcel_count: u16,
    pub created_at: i64,
    pub updated_at: i64,
}

/// An in-flight passation of wallet control, time-boxed by a grace period so
/// the original owner can cancel or object before it settles.
///
/// PDA: `["succession", identity, successor]`.
#[account]
#[derive(InitSpace)]
pub struct Succession {
    /// The Identity whose control is being passed.
    pub identity: Pubkey,
    /// The wallet that will take over after the grace period.
    pub successor: Pubkey,
    /// succession_kind.
    pub kind: u8,
    pub requested_at: i64,
    /// requested_at + SUCCESSION_GRACE_PERIOD_SECS. Claim is only allowed after.
    pub effective_at: i64,
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

    /// Register an attestation that binds heavy off-chain data to this parcel
    /// and records the set of validator wallets required to validate it.
    ///
    /// `validators` holds the public keys of the (possibly several) parties
    /// who must sign off on the transaction; `required` is how many signatures
    /// are needed. The signer must be the parcel owner or a registered
    /// registrar. Per-validator Ed25519 signatures live off-chain but are
    /// verified against this on-chain identity set and `content_hash`.
    pub fn attest(
        ctx: Context<Attest>,
        specifier: [u8; 32],
        content_hash: [u8; 32],
        required: u8,
        validators: [Pubkey; MAX_VALIDATORS],
    ) -> Result<()> {
        let parcel = &ctx.accounts.parcel;
        // Only the parcel owner (or program authority) may create attestations.
        require!(
            parcel.owner == ctx.accounts.authority.key(),
            TerraError::NotOwner
        );
        require!(
            !specifier.iter().all(|b| *b == 0),
            TerraError::EmptySpecifier
        );
        require!(
            !content_hash.iter().all(|b| *b == 0),
            TerraError::EmptyContentHash
        );

        let mut count: u8 = 0;
        for &v in validators.iter() {
            if v == Pubkey::default() {
                continue;
            }
            count += 1;
        }
        require!(count > 0, TerraError::NoValidators);
        require!(
            (required as usize) <= count as usize,
            TerraError::InvalidThreshold
        );

        let now = Clock::get()?.unix_timestamp;
        let attestation = &mut ctx.accounts.attestation;
        attestation.parcel = parcel.key();
        attestation.specifier = specifier;
        attestation.content_hash = content_hash;
        attestation.required = required;
        attestation.count = count;
        attestation.created_at = now;
        attestation.validators = validators;

        emit!(Attested {
            parcel: parcel.key(),
            specifier,
            content_hash,
            required,
            count,
        });
        Ok(())
    }

    /// Bind a person (identified by a hashed credential) to a wallet the person
    /// holds. `recovery` is a second wallet the person controls, for recovering
    /// the identity if the main key is lost. The signer becomes `owner`.
    ///
    /// This is the root of the resolvable "who owns this" link: every on-chain
    /// actor is ultimately a wallet, and this account binds that wallet to a
    /// human without ever publishing the credential itself.
    pub fn bind_identity(
        ctx: Context<BindIdentity>,
        identity_hash: [u8; 32],
        recovery: Pubkey,
    ) -> Result<()> {
        require!(
            !identity_hash.iter().all(|b| *b == 0),
            TerraError::EmptyIdentityHash
        );
        require!(recovery != Pubkey::default(), TerraError::EmptyRecovery);

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

    /// Attach a parcel to an identity (the person behind its owner wallet).
    /// Only the parcel's owner may do this, and only for an identity whose
    /// owner wallet matches.
    pub fn attach_parcel(
        ctx: Context<AttachParcel>,
    ) -> Result<()> {
        let parcel = &ctx.accounts.parcel;
        require!(
            parcel.owner == ctx.accounts.owner.key(),
            TerraError::NotOwner
        );
        let identity = &mut ctx.accounts.identity;
        require!(
            identity.owner == ctx.accounts.owner.key(),
            TerraError::IdentityMismatch
        );

        identity.parcel_count = identity.parcel_count.saturating_add(1);
        identity.updated_at = Clock::get()?.unix_timestamp;
        emit!(ParcelAttached {
            identity: identity.key(),
            parcel: parcel.key(),
            owner: identity.owner,
        });
        Ok(())
    }

    /// Request a wallet passation (succession, recovery, or deliberate control
    /// transfer). A Succession account is created and becomes effective only
    /// after the grace period — within which the original owner can cancel.
    ///
    /// Authorized by the current `owner` for kind TRANSFER, or by the `owner`
    /// OR the `recovery` wallet for kind RECOVERY/SUCCESSOR.
    pub fn request_succession(
        ctx: Context<RequestSuccession>,
        successor: Pubkey,
        kind: u8,
    ) -> Result<()> {
        require!(successor != Pubkey::default(), TerraError::EmptySuccessor);
        require!(kind <= succession_kind::MAX, TerraError::InvalidSuccessionKind);

        let identity = &ctx.accounts.identity;
        let signer = ctx.accounts.signer.key();
        require!(
            signer == identity.owner || signer == identity.recovery,
            TerraError::NotAuthorized
        );
        require!(successor != identity.owner, TerraError::SuccessorIsOwner);

        let now = Clock::get()?.unix_timestamp;
        let succession = &mut ctx.accounts.succession;
        succession.identity = identity.key();
        succession.successor = successor;
        succession.kind = kind;
        succession.requested_at = now;
        succession.effective_at = now.saturating_add(SUCCESSION_GRACE_PERIOD_SECS);

        emit!(SuccessionRequested {
            identity: identity.key(),
            successor,
            kind,
            effective_at: succession.effective_at,
        });
        Ok(())
    }

    /// Cancel an in-flight succession. Only the current `owner` (or `recovery`
    /// for a recovery passation) may cancel, and only before it is effective.
    pub fn cancel_succession(
        ctx: Context<CancelSuccession>,
    ) -> Result<()> {
        let identity = &ctx.accounts.identity;
        let signer = ctx.accounts.signer.key();
        require!(
            signer == identity.owner || signer == identity.recovery,
            TerraError::NotAuthorized
        );
        require!(
            ctx.accounts.succession.effective_at > Clock::get()?.unix_timestamp,
            TerraError::SuccessionAlreadyEffective
        );

        emit!(SuccessionCancelled {
            identity: identity.key(),
            successor: ctx.accounts.succession.successor,
            kind: ctx.accounts.succession.kind,
        });
        Ok(())
    }

    /// Claim a passation once the grace period has elapsed. The `successor`
    /// becomes the identity's new owner. Any parcels the identity owned that
    /// are supplied via `remaining_accounts` are re-pointed to the successor.
    pub fn claim_succession(
        ctx: Context<ClaimSuccession>,
    ) -> Result<()> {
        let now = Clock::get()?.unix_timestamp;
        let succession = &ctx.accounts.succession;
        require!(
            succession.successor == ctx.accounts.signer.key(),
            TerraError::NotSuccessor
        );
        require!(
            now >= succession.effective_at,
            TerraError::SuccessionNotYetEffective
        );

        let identity = &mut ctx.accounts.identity;
        require!(
            succession.identity == identity.key(),
            TerraError::IdentityMismatch
        );

        let previous = identity.owner;
        let successor = succession.successor;
        identity.owner = successor;
        identity.recovery = Pubkey::default();
        identity.updated_at = now;

        // Re-point every supplied parcel owned by this identity to the
        // successor's wallet. The Parcel `owner` field sits at a fixed borsh
        // offset (8-byte discriminator + 32-byte id = 40..72), so we patch it
        // directly rather than re-serializing the whole account.
        let mut successions_applied: u16 = 0;
        for account in ctx.remaining_accounts.iter() {
            if account.owner == ctx.program_id {
                let mut data = account.try_borrow_mut_data()?;
                let pdisc: &[u8] = <Parcel as anchor_lang::Discriminator>::DISCRIMINATOR;
                if data.len() < 72 || &data[0..8] != pdisc {
                    continue;
                }
                let mut current_owner = [0u8; 32];
                current_owner.copy_from_slice(&data[40..72]);
                let current_owner = Pubkey::from(current_owner);
                if current_owner == previous {
                    data[40..72].copy_from_slice(&successor.to_bytes());
                    successions_applied += 1;
                }
            }
        }
        identity.parcel_count = identity.parcel_count.saturating_sub(successions_applied);

        emit!(SuccessionClaimed {
            identity: identity.key(),
            from: previous,
            to: successor,
            kind: succession.kind,
            parcels_repointed: successions_applied as u8,
        });
        Ok(())
    }

    /// Replace the validator set on an attestation (the fix for dead/leaving
    /// validators). Only the parcel owner may rotate. Bumps `version` so a
    /// reconstituted set is provably newer, and resets `required`/`count`.
    pub fn rotate_validators(
        ctx: Context<RotateValidators>,
        new_required: u8,
        new_validators: [Pubkey; MAX_VALIDATORS],
    ) -> Result<()> {
        let parcel = &ctx.accounts.parcel;
        require!(
            parcel.owner == ctx.accounts.authority.key(),
            TerraError::NotOwner
        );
        require!(
            ctx.accounts.attestation.parcel == parcel.key(),
            TerraError::AttestationMismatch
        );

        let mut count: u8 = 0;
        for &v in new_validators.iter() {
            if v == Pubkey::default() {
                continue;
            }
            count += 1;
        }
        require!(count > 0, TerraError::NoValidators);
        require!(
            (new_required as usize) <= count as usize,
            TerraError::InvalidThreshold
        );

        let now = Clock::get()?.unix_timestamp;
        let attestation = &mut ctx.accounts.attestation;
        attestation.validators = new_validators;
        attestation.required = new_required;
        attestation.count = count;
        attestation.version = attestation.version.saturating_add(1);
        attestation.updated_at = now;

        emit!(ValidatorsRotated {
            parcel: parcel.key(),
            specifier: attestation.specifier,
            version: attestation.version,
            required: new_required,
            count,
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

#[derive(Accounts)]
#[instruction(specifier: [u8; 32])]
pub struct Attest<'info> {
    #[account(
        seeds = [b"parcel".as_ref(), parcel.id.as_ref()],
        bump
    )]
    pub parcel: Account<'info, Parcel>,
    #[account(
        init,
        payer = authority,
        space = 8 + Attestation::INIT_SPACE,
        seeds = [b"attestation".as_ref(), parcel.key().as_ref(), specifier.as_ref()],
        bump
    )]
    pub attestation: Account<'info, Attestation>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

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
pub struct AttachParcel<'info> {
    #[account(
        mut,
        seeds = [b"parcel".as_ref(), parcel.id.as_ref()],
        bump
    )]
    pub parcel: Account<'info, Parcel>,
    #[account(
        mut,
        seeds = [b"identity".as_ref(), identity.identity_hash.as_ref()],
        bump
    )]
    pub identity: Account<'info, Identity>,
    pub owner: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(successor: Pubkey, kind: u8)]
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
pub struct RotateValidators<'info> {
    #[account(
        mut,
        seeds = [b"parcel".as_ref(), parcel.id.as_ref()],
        bump
    )]
    pub parcel: Account<'info, Parcel>,
    #[account(
        mut,
        seeds = [b"attestation".as_ref(), parcel.key().as_ref(), attestation.specifier.as_ref()],
        bump
    )]
    pub attestation: Account<'info, Attestation>,
    pub authority: Signer<'info>,
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

#[event]
pub struct Attested {
    pub parcel: Pubkey,
    pub specifier: [u8; 32],
    pub content_hash: [u8; 32],
    pub required: u8,
    pub count: u8,
}

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
    pub effective_at: i64,
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
    pub parcels_repointed: u8,
}

#[event]
pub struct ValidatorsRotated {
    pub parcel: Pubkey,
    pub specifier: [u8; 32],
    pub version: u8,
    pub required: u8,
    pub count: u8,
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
    #[msg("Attestation specifier is required")]
    EmptySpecifier,
    #[msg("Content hash is required")]
    EmptyContentHash,
    #[msg("Attestation requires at least one validator")]
    NoValidators,
    #[msg("Required threshold exceeds the number of validators")]
    InvalidThreshold,
    #[msg("Identity hash is required")]
    EmptyIdentityHash,
    #[msg("Recovery wallet is required")]
    EmptyRecovery,
    #[msg("Identity owner does not match the parcel owner")]
    IdentityMismatch,
    #[msg("Successor wallet is required")]
    EmptySuccessor,
    #[msg("Invalid succession kind")]
    InvalidSuccessionKind,
    #[msg("Successor must differ from the current owner")]
    SuccessorIsOwner,
    #[msg("Succession has already become effective")]
    SuccessionAlreadyEffective,
    #[msg("Only the named successor may claim this succession")]
    NotSuccessor,
    #[msg("Succession is not yet effective")]
    SuccessionNotYetEffective,
    #[msg("Attestation does not belong to this parcel")]
    AttestationMismatch,
}
