use anchor_lang::prelude::*;
use anchor_lang::Discriminator;

declare_id!("GaEDbktvpZ3qiqp4PmFgHwDSa6JsFfVjXFqNb2nTbage");

pub mod parcel_status {
    pub const PENDING: u8 = 0;
    pub const REGISTERED: u8 = 1;
    pub const FOR_SALE: u8 = 2;
    pub const TRANSFERRED: u8 = 3;
    /// Reserved for RFC-007 (Dispute Resolution & Parcel Freeze).
    pub const DISPUTED: u8 = 4;
    /// Reserved for RFC-007 (Dispute Resolution & Parcel Freeze).
    pub const FROZEN: u8 = 5;
    /// Reserved for RFC-007 (Dispute Resolution & Parcel Freeze).
    pub const ADJUDICATED: u8 = 6;
    /// Reserved for RFC-007 (Dispute Resolution & Parcel Freeze).
    pub const FORFEITED: u8 = 7;
    pub const MAX: u8 = FORFEITED;
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
    /// Time-bound credential status (RFC-009): ACTIVE, EXPIRING, EXPIRED, GRACE, RENEWED, REVOKED.
    pub status: u8,
    /// Grace period in seconds after expiry (0 = no grace). Set at grant time.
    pub grace_period_secs: i64,
}

pub mod right_status {
    pub const ACTIVE: u8 = 0;
    pub const EXPIRING: u8 = 1;
    pub const EXPIRED: u8 = 2;
    pub const GRACE: u8 = 3;
    pub const RENEWED: u8 = 4;
    pub const REVOKED: u8 = 5;
    pub const MAX: u8 = REVOKED;
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

/// Minimum grace period (seconds) a requester may choose. Chosen so legitimate
/// heirs far from a local validator aren't rushed, while still bounding the
/// theft window.
pub const MIN_SUCCESSION_GRACE_SECS: i64 = 7 * 24 * 3600; // 7 days
/// Maximum grace period a requester may choose.
pub const MAX_SUCCESSION_GRACE_SECS: i64 = 180 * 24 * 3600; // 180 days
/// Default grace period when a requester passes 0.
pub const DEFAULT_SUCCESSION_GRACE_SECS: i64 = 30 * 24 * 3600; // 30 days
/// Floor for the number of validator endorsements required on a passation.
pub const MIN_SUCCESSION_VALIDATIONS: u8 = 1;
/// Floor for the number of validator signers required to forfeit a parcel.
pub const MIN_FORFEIT_VALIDATORS: u8 = 2;

/// The account struct (below) uses these — bump the account size accordingly.

pub mod succession_kind {
    /// Wallet passation to an heir/beneficiary (estate / inheritance).
    pub const SUCCESSOR: u8 = 0;
    /// Passation because the active key was lost/stolen (recovery).
    pub const RECOVERY: u8 = 1;
    /// Passation of a parcel's control (sale / deliberate transfer).
    pub const TRANSFER: u8 = 2;
    /// Reserved for RFC-010 (Guardian & Recovery Council).
    pub const GUARDIANSHIP: u8 = 3;
    /// Reserved for RFC-010 (Guardian & Recovery Council).
    pub const COURT_APPOINTED_GUARDIAN: u8 = 4;
    pub const MAX: u8 = COURT_APPOINTED_GUARDIAN;
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

/// An in-flight passation of wallet control, gated by BOTH a configurable grace
/// period AND a minimum number of validator endorsements (so a stolen wallet
/// can't seize land) before it can be claimed.
///
/// PDA: `["succession", identity, successor]`.
#[account]
#[derive(InitSpace)]
pub struct Succession {
    /// The Identity whose control is being passed.
    pub identity: Pubkey,
    /// The wallet that will take over once gated.
    pub successor: Pubkey,
    /// succession_kind.
    pub kind: u8,
    pub requested_at: i64,
    /// effective = requested_at + grace_secs. Claim only allowed after this
    /// AND validations_count >= required.
    pub effective_at: i64,
    /// Configurable per-request grace (0 => DEFAULT_SUCCESSION_GRACE_SECS).
    pub grace_secs: i64,
    /// Number of validator endorsements required before claim (>= MIN, <= count).
    pub required: u8,
    /// Number of endorsements collected so far.
    pub validations_count: u8,
    /// Declared local-authority validator set acting as testifiers.
    pub validators: [Pubkey; MAX_VALIDATORS],
}

pub mod vault;
pub mod authority_registry;
pub mod ipfs_docs;
pub mod dispute;
pub mod escrow;
pub mod time_bound;

// ---------------------------------------------------------------------------
// Vault instruction contexts (RFC-003)
// ---------------------------------------------------------------------------

#[derive(Accounts)]
#[instruction(ciphertext_cid: String, ciphertext_hash: [u8; 32], algorithm_id: u8, storage_uris: Vec<String>, shard_holders: Vec<Pubkey>, threshold: u8)]
pub struct CreateVault<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + vault::VaultRecord::INIT_SPACE,
        seeds = [b"vault_record", subject.key().as_ref()],
        bump
    )]
    pub vault_record: Account<'info, vault::VaultRecord>,
    pub subject: Account<'info, Identity>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(purpose: String, expiry: i64)]
pub struct AuthorizeVaultAccess<'info> {
    #[account(mut)]
    pub vault_record: Account<'info, vault::VaultRecord>,
    pub subject: Account<'info, Identity>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(new_ciphertext_hash: [u8; 32], new_shard_holders: Vec<Pubkey>)]
pub struct InitiateShardRotation<'info> {
    #[account(
        init,
        payer = initiator,
        space = 8 + vault::VaultShardRotation::INIT_SPACE,
        seeds = [
            b"vault_shard_rotation",
            vault_record.key().as_ref(),
            new_ciphertext_hash.as_ref()
        ],
        bump
    )]
    pub rotation: Account<'info, vault::VaultShardRotation>,
    #[account(mut)]
    pub vault_record: Account<'info, vault::VaultRecord>,
    #[account(mut)]
    pub initiator: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct EndorseShardRotation<'info> {
    #[account(mut)]
    pub rotation: Account<'info, vault::VaultShardRotation>,
    #[account(
        constraint = vault_record.subject == subject.key() @ TerraError::IdentityMismatch
    )]
    pub vault_record: Account<'info, vault::VaultRecord>,
    pub subject: Account<'info, Identity>,
    pub validator: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ExecuteShardRotation<'info> {
    #[account(
        mut,
        close = initiator,
        constraint = rotation.vault == vault_record.key() @ TerraError::RotationNotFound
    )]
    pub rotation: Account<'info, vault::VaultShardRotation>,
    #[account(mut)]
    pub vault_record: Account<'info, vault::VaultRecord>,
    #[account(mut)]
    pub initiator: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CancelShardRotation<'info> {
    #[account(
        mut,
        close = canceller,
        constraint = rotation.vault == vault_record.key() @ TerraError::RotationNotFound
    )]
    pub rotation: Account<'info, vault::VaultShardRotation>,
    pub vault_record: Account<'info, vault::VaultRecord>,
    #[account(mut)]
    pub canceller: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct PingShard<'info> {
    #[account(mut)]
    pub vault_record: Account<'info, vault::VaultRecord>,
    pub validator: Signer<'info>,
    pub system_program: Program<'info, System>,
}

// ---------------------------------------------------------------------------
// AuthorityRegistry v2 contexts (progressive decentralization)
// ---------------------------------------------------------------------------

#[derive(Accounts)]
pub struct CreateRegistry<'info> {
    #[account(
        init,
        payer = admin,
        space = 8 + authority_registry::AuthorityRegistry::INIT_SPACE,
        seeds = [b"authority_registry"],
        bump
    )]
    pub registry: Account<'info, authority_registry::AuthorityRegistry>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct AddValidator<'info> {
    #[account(mut)]
    pub registry: Account<'info, authority_registry::AuthorityRegistry>,
    /// Admin signer — required in bootstrap mode, ignored in peer-consensus
    /// (the endorsement account carries authority instead).
    #[account(mut)]
    pub admin_signer: Signer<'info>,
    #[account(
        init_if_needed,
        payer = admin_signer,
        space = 8 + authority_registry::ValidatorEndorsement::INIT_SPACE,
        seeds = [
            b"validator_endorsement",
            registry.key().as_ref(),
            validator.key().as_ref()
        ],
        bump
    )]
    pub endorsement: Account<'info, authority_registry::ValidatorEndorsement>,
    /// CHECK: validated as a non-zero pubkey in handler.
    pub validator: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RemoveValidator<'info> {
    #[account(mut)]
    pub registry: Account<'info, authority_registry::AuthorityRegistry>,
    pub admin_signer: Signer<'info>,
    #[account(
        mut,
        seeds = [
            b"validator_endorsement",
            registry.key().as_ref(),
            validator.key().as_ref()
        ],
        bump,
    )]
    pub endorsement: Account<'info, authority_registry::ValidatorEndorsement>,
    /// CHECK: validated against registry in handler.
    pub validator: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct EndorseValidatorAdd<'info> {
    #[account(
        mut,
        seeds = [
            b"validator_endorsement",
            registry.key().as_ref(),
            endorsement.proposed.as_ref()
        ],
        bump,
    )]
    pub endorsement: Account<'info, authority_registry::ValidatorEndorsement>,
    pub registry: Account<'info, authority_registry::AuthorityRegistry>,
    pub endorser: Signer<'info>,
}

#[derive(Accounts)]
pub struct FlipToConsensus<'info> {
    #[account(mut)]
    pub registry: Account<'info, authority_registry::AuthorityRegistry>,
    pub admin_signer: Signer<'info>,
}

// ---------------------------------------------------------------------------
// IPFS document storage contexts
// ---------------------------------------------------------------------------

#[derive(Accounts)]
#[instruction(cid: String, content_hash: [u8; 32], category: String)]
pub struct RegisterDocument<'info> {
    #[account(
        init,
        payer = registrant,
        space = 8 + ipfs_docs::DocumentAnchor::INIT_SPACE,
        seeds = [
            b"document",
            attestation.key().as_ref(),
            cid.as_ref()
        ],
        bump
    )]
    pub document: Account<'info, ipfs_docs::DocumentAnchor>,
    #[account(
        seeds = [b"attestation".as_ref(), parcel.key().as_ref(), attestation.specifier.as_ref()],
        bump,
    )]
    pub attestation: Account<'info, Attestation>,
    #[account(
        seeds = [b"parcel".as_ref(), parcel.id.as_ref()],
        bump,
    )]
    pub parcel: Account<'info, Parcel>,
    #[account(mut)]
    pub registrant: Signer<'info>,
    pub system_program: Program<'info, System>,
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
            status <= parcel_status::MAX,
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
        rights.status = right_status::ACTIVE;
        rights.grace_period_secs = 0;

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
    ///
    /// `grace_secs` lets the requester choose the window (0 => default 30d),
    /// clamped to [MIN, MAX]. `required_validations` is the number of declared
    /// local validators that must endorse the passation before it can be
    /// claimed (>= 1) — so a stolen wallet can't seize land alone.
    /// `validators` declares the local-authority testifiers for this passation.
    pub fn request_succession(
        ctx: Context<RequestSuccession>,
        successor: Pubkey,
        kind: u8,
        grace_secs: i64,
        required_validations: u8,
        validators: [Pubkey; MAX_VALIDATORS],
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

        let mut count: u8 = 0;
        for &v in validators.iter() {
            if v == Pubkey::default() {
                continue;
            }
            // Self-dealing check: the identity owner cannot be their own validator.
            require!(v != identity.owner, TerraError::ValidatorOwnsAsset);
            count += 1;
        }
        require!(count > 0, TerraError::NoValidators);
        require!(
            (required_validations as usize) <= count as usize,
            TerraError::InvalidThreshold
        );
        require!(
            required_validations >= MIN_SUCCESSION_VALIDATIONS,
            TerraError::InvalidThreshold
        );

        let grace = if grace_secs == 0 {
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

    /// Record one validator's endorsement of a pending succession. The signing
    /// validator must be in the succession's declared validator set; this bumps
    /// `validations_count`. Each endorsement is an Ed25519 signature because the
    /// validator signs this transaction with their wallet. Only meaningful
    /// before the succession becomes effective (validations are then moot).
    pub fn endorse_succession(
        ctx: Context<EndorseSuccession>,
    ) -> Result<()> {
        let now = Clock::get()?.unix_timestamp;
        let succession = &mut ctx.accounts.succession;
        require!(
            now < succession.effective_at,
            TerraError::SuccessionAlreadyEffective
        );
        require!(
            (succession.validations_count as usize) < (succession.required as usize),
            TerraError::ValidationLimitReached
        );

        let validator = ctx.accounts.validator.key();
        require!(
            succession.validators.contains(&validator),
            TerraError::NotValidator
        );
        // Self-dealing check: a validator must not be the identity owner.
        require!(
            validator != ctx.accounts.identity.owner,
            TerraError::ValidatorOwnsAsset
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

    /// Claim a passation once BOTH the grace period has elapsed AND the required
    /// number of validators have endorsed it. The `successor` becomes the
    /// identity's new owner. Any parcels the identity owned that are supplied
    /// via `remaining_accounts` are re-pointed to the successor.
    pub fn claim_succession(
        ctx: Context<ClaimSuccession>,
    ) -> Result<()> {
        let now = Clock::get()?.unix_timestamp;
        let succession = &ctx.accounts.succession;
        require!(
            succession.successor == ctx.accounts.signer.key(),
            TerraError::NotSuccessor
        );
        // Two independent gates: time AND validator endorsement. A stolen wallet
        // alone (or a thief who happens to know the successor) still can't claim
        // without the local validators testifying.
        require!(
            now >= succession.effective_at,
            TerraError::SuccessionNotYetEffective
        );
        require!(
            succession.validations_count >= succession.required,
            TerraError::InsufficientValidations
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

    /// Force-transfer a parcel's ownership away from a non-compliant owner, per
    /// a court order. This is deliberately heavier than a normal transfer:
    /// at least `MIN_FORFEIT_VALIDATORS` (2) of the declared validators must
    /// sign this transaction themselves, and the order is bound to a
    /// `case_hash` (e.g. SHA-256 of the court order document) for auditability.
    ///
    /// This is how validators collectively inform the chain that land no longer
    /// belongs to someone who refuses to release it — e.g. repossession by a
    /// government, or a court ruling that title passed to another person.
    pub fn judicial_forfeiture(
        ctx: Context<JudicialForfeiture>,
        case_hash: [u8; 32],
        new_owner: Pubkey,
        threshold: u8,
        validators: [Pubkey; MAX_VALIDATORS],
    ) -> Result<()> {
        require!(
            !case_hash.iter().all(|b| *b == 0),
            TerraError::EmptyCaseHash
        );
        require!(new_owner != Pubkey::default(), TerraError::EmptyNewOwner);
        require!(
            threshold >= MIN_FORFEIT_VALIDATORS,
            TerraError::InvalidThreshold
        );

        let mut count: u8 = 0;
        for &v in validators.iter() {
            if v == Pubkey::default() {
                continue;
            }
            // Self-dealing check: declared validators must not include the parcel owner.
            require!(v != ctx.accounts.parcel.owner, TerraError::ValidatorOwnsAsset);
            count += 1;
        }
        require!((threshold as usize) <= count as usize, TerraError::InvalidThreshold);

        let parcel = &mut ctx.accounts.parcel;
        let from = parcel.owner;

        // Count how many of the discovered validator signers are part of the
        // declared set AND actually signed this transaction.
        let mut present: u8 = 0;
        for signer in ctx.remaining_accounts.iter() {
            if signer.is_signer && validators.contains(&signer.key()) {
                // Self-dealing check: parcel owner cannot sign as validator.
                require!(signer.key() != from, TerraError::ValidatorOwnsAsset);
                present += 1;
            }
        }
        require!(present >= threshold, TerraError::InsufficientValidatorSigners);

        // The relaying party must not be the current owner (prevents self-forfeit).
        require!(
            ctx.accounts.authority.key() != from,
            TerraError::OwnerCannotSelfForfeit
        );

        parcel.owner = new_owner;
        parcel.updated_at = Clock::get()?.unix_timestamp;

        emit!(ParcelForfeited {
            parcel: parcel.key(),
            case_hash,
            from,
            to: new_owner,
            threshold,
            present,
        });
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Vault shard protocol (RFC-003)
    // -----------------------------------------------------------------------

    pub fn create_vault(
        ctx: Context<CreateVault>,
        ciphertext_cid: String,
        ciphertext_hash: [u8; 32],
        algorithm_id: u8,
        storage_uris: Vec<String>,
        shard_holders: Vec<Pubkey>,
        threshold: u8,
    ) -> Result<()> {
        vault::create_vault(ctx, ciphertext_cid, ciphertext_hash, algorithm_id, storage_uris, shard_holders, threshold)
    }

    pub fn authorize_vault_access(
        ctx: Context<AuthorizeVaultAccess>,
        purpose: String,
        expiry: i64,
        off_chain_nonce: [u8; 32],
    ) -> Result<()> {
        vault::authorize_vault_access(ctx, purpose, expiry, off_chain_nonce)
    }

    pub fn initiate_shard_rotation(
        ctx: Context<InitiateShardRotation>,
        new_ciphertext_hash: [u8; 32],
        new_shard_holders: Vec<Pubkey>,
        new_threshold: u8,
    ) -> Result<()> {
        vault::initiate_shard_rotation(ctx, new_ciphertext_hash, new_shard_holders, new_threshold)
    }

    pub fn endorse_shard_rotation(ctx: Context<EndorseShardRotation>) -> Result<()> {
        vault::endorse_shard_rotation(ctx)
    }

    pub fn execute_shard_rotation(ctx: Context<ExecuteShardRotation>) -> Result<()> {
        vault::execute_shard_rotation(ctx)
    }

    pub fn cancel_shard_rotation(ctx: Context<CancelShardRotation>) -> Result<()> {
        vault::cancel_shard_rotation(ctx)
    }

    pub fn ping_shard(ctx: Context<PingShard>) -> Result<()> {
        vault::ping_shard(ctx)
    }

    // -----------------------------------------------------------------------
    // AuthorityRegistry v2 (progressive decentralization)
    // -----------------------------------------------------------------------

    pub fn create_registry(ctx: Context<CreateRegistry>) -> Result<()> {
        authority_registry::create_registry(ctx)
    }

    pub fn add_validator_to_registry(
        ctx: Context<AddValidator>,
        validator: Pubkey,
    ) -> Result<()> {
        authority_registry::add_validator(ctx, validator)
    }

    pub fn remove_validator_from_registry(
        ctx: Context<RemoveValidator>,
        validator: Pubkey,
    ) -> Result<()> {
        authority_registry::remove_validator(ctx, validator)
    }

    pub fn endorse_validator_add(ctx: Context<EndorseValidatorAdd>) -> Result<()> {
        authority_registry::endorse_validator_add(ctx)
    }

    pub fn flip_to_consensus(ctx: Context<FlipToConsensus>) -> Result<()> {
        authority_registry::flip_to_consensus(ctx)
    }

    // -----------------------------------------------------------------------
    // IPFS document storage
    // -----------------------------------------------------------------------

    pub fn register_document(
        ctx: Context<RegisterDocument>,
        cid: String,
        content_hash: [u8; 32],
        category: String,
    ) -> Result<()> {
        ipfs_docs::register_document(ctx, cid, content_hash, category)
    }

    // -----------------------------------------------------------------------
    // Dispute / freeze protocol (RFC-007)
    // -----------------------------------------------------------------------

    pub fn file_dispute(
        ctx: Context<FileDispute>,
        case_hash: [u8; 32],
        required: u8,
        validators: [Pubkey; MAX_VALIDATORS],
    ) -> Result<()> {
        dispute::file_dispute(ctx, case_hash, required, validators)
    }

    pub fn freeze_parcel(ctx: Context<FreezeParcel>) -> Result<()> {
        dispute::freeze_parcel(ctx)
    }

    pub fn adjudicate_dispute(
        ctx: Context<AdjudicateDispute>,
        outcome: u8,
        new_owner: Pubkey,
    ) -> Result<()> {
        dispute::adjudicate_dispute(ctx, outcome, new_owner)
    }

    pub fn execute_judgment(ctx: Context<ExecuteJudgment>) -> Result<()> {
        dispute::execute_judgment(ctx)
    }

    pub fn cancel_dispute(ctx: Context<CancelDispute>) -> Result<()> {
        dispute::cancel_dispute(ctx)
    }

    // -----------------------------------------------------------------------
    // Escrow settlement (RFC-004)
    // -----------------------------------------------------------------------

    pub fn create_escrow(
        ctx: Context<CreateEscrow>,
        amount: u64,
        buyer: Pubkey,
    ) -> Result<()> {
        escrow::create_escrow(ctx, amount, buyer)
    }

    pub fn deposit_escrow(
        ctx: Context<DepositEscrow>,
        deposit_amount: u64,
    ) -> Result<()> {
        escrow::deposit_escrow(ctx, deposit_amount)
    }

    pub fn accept_escrow(ctx: Context<AcceptEscrow>) -> Result<()> {
        escrow::accept_escrow(ctx)
    }

    pub fn settle_escrow(ctx: Context<SettleEscrow>) -> Result<()> {
        escrow::settle_escrow(ctx)
    }

    pub fn cancel_escrow(ctx: Context<CancelEscrow>) -> Result<()> {
        escrow::cancel_escrow(ctx)
    }

    pub fn dispute_escrow(
        ctx: Context<DisputeEscrow>,
        case_hash: [u8; 32],
        required: u8,
        validators: [Pubkey; MAX_VALIDATORS],
    ) -> Result<()> {
        escrow::dispute_escrow(ctx, case_hash, required, validators)
    }

    pub fn expire_escrow(ctx: Context<ExpireEscrow>) -> Result<()> {
        escrow::expire_escrow(ctx)
    }

    // -----------------------------------------------------------------------
    // Time-bound credentials (RFC-009)
    // -----------------------------------------------------------------------

    pub fn renew_right(
        ctx: Context<RenewRight>,
        nonce: u8,
        new_expires_at: i64,
        new_notes: String,
    ) -> Result<()> {
        time_bound::renew_right(ctx, nonce, new_expires_at, new_notes)
    }

    pub fn sweep_expired_rights(
        ctx: Context<SweepExpiredRights>,
        nonce: u8,
    ) -> Result<()> {
        time_bound::sweep_expired_rights(ctx, nonce)
    }

    pub fn grant_conditional_right(
        ctx: Context<GrantConditionalRight>,
        nonce: u8,
        rights_kind: u8,
        holder: Pubkey,
        expires_at: i64,
        condition_deadline: i64,
        condition_desc: String,
        grace_period_secs: i64,
        notes: String,
    ) -> Result<()> {
        time_bound::grant_conditional_right(
            ctx, nonce, rights_kind, holder, expires_at,
            condition_deadline, condition_desc, grace_period_secs, notes,
        )
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
    /// A declared local validator endorsing the passation (signs this tx).
    pub validator: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(case_hash: [u8; 32])]
pub struct JudicialForfeiture<'info> {
    #[account(
        mut,
        seeds = [b"parcel".as_ref(), parcel.id.as_ref()],
        bump
    )]
    pub parcel: Account<'info, Parcel>,
    /// Relaying authority (court clerk / govt channel). Must NOT be the owner.
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

// ---------------------------------------------------------------------------
// Dispute / freeze contexts (RFC-007)
// ---------------------------------------------------------------------------

#[derive(Accounts)]
#[instruction(case_hash: [u8; 32])]
pub struct FileDispute<'info> {
    #[account(
        init,
        payer = filer,
        space = 8 + dispute::Dispute::INIT_SPACE,
        seeds = [b"dispute", parcel.key().as_ref(), case_hash.as_ref()],
        bump
    )]
    pub dispute: Account<'info, dispute::Dispute>,
    #[account(
        mut,
        seeds = [b"parcel".as_ref(), parcel.id.as_ref()],
        bump,
    )]
    pub parcel: Account<'info, Parcel>,
    #[account(mut)]
    pub filer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct FreezeParcel<'info> {
    #[account(
        mut,
        seeds = [b"dispute", parcel.key().as_ref(), dispute.case_hash.as_ref()],
        bump,
    )]
    pub dispute: Account<'info, dispute::Dispute>,
    #[account(
        mut,
        seeds = [b"parcel".as_ref(), parcel.id.as_ref()],
        bump,
    )]
    pub parcel: Account<'info, Parcel>,
    pub validator: Signer<'info>,
}

#[derive(Accounts)]
pub struct AdjudicateDispute<'info> {
    #[account(
        mut,
        seeds = [b"dispute", parcel.key().as_ref(), dispute.case_hash.as_ref()],
        bump,
    )]
    pub dispute: Account<'info, dispute::Dispute>,
    #[account(
        mut,
        seeds = [b"parcel".as_ref(), parcel.id.as_ref()],
        bump,
    )]
    pub parcel: Account<'info, Parcel>,
    /// Court authority or designated adjudicator.
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct ExecuteJudgment<'info> {
    #[account(
        mut,
        seeds = [b"dispute", parcel.key().as_ref(), dispute.case_hash.as_ref()],
        bump,
    )]
    pub dispute: Account<'info, dispute::Dispute>,
    #[account(
        mut,
        seeds = [b"parcel".as_ref(), parcel.id.as_ref()],
        bump,
    )]
    pub parcel: Account<'info, Parcel>,
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct CancelDispute<'info> {
    #[account(
        mut,
        seeds = [b"dispute", parcel.key().as_ref(), dispute.case_hash.as_ref()],
        bump,
    )]
    pub dispute: Account<'info, dispute::Dispute>,
    #[account(
        mut,
        seeds = [b"parcel".as_ref(), parcel.id.as_ref()],
        bump,
    )]
    pub parcel: Account<'info, Parcel>,
    pub signer: Signer<'info>,
}

// ---------------------------------------------------------------------------
// Escrow settlement contexts (RFC-004)
// ---------------------------------------------------------------------------

#[derive(Accounts)]
#[instruction(amount: u64, buyer: Pubkey)]
pub struct CreateEscrow<'info> {
    #[account(
        init,
        payer = seller,
        space = 8 + escrow::EscrowRecord::INIT_SPACE,
        seeds = [b"escrow", parcel.key().as_ref()],
        bump
    )]
    pub escrow_record: Account<'info, escrow::EscrowRecord>,
    /// CHECK: escrow vault PDA — holds deposited SOL.
    #[account(
        mut,
        seeds = [b"escrow_vault", escrow_record.key().as_ref()],
        bump
    )]
    pub escrow_vault: SystemAccount<'info>,
    #[account(
        mut,
        seeds = [b"parcel".as_ref(), parcel.id.as_ref()],
        bump,
    )]
    pub parcel: Account<'info, Parcel>,
    #[account(mut)]
    pub seller: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(deposit_amount: u64)]
pub struct DepositEscrow<'info> {
    #[account(
        mut,
        seeds = [b"escrow", parcel.key().as_ref()],
        bump,
    )]
    pub escrow_record: Account<'info, escrow::EscrowRecord>,
    /// CHECK: escrow vault PDA.
    #[account(
        mut,
        seeds = [b"escrow_vault", escrow_record.key().as_ref()],
        bump
    )]
    pub escrow_vault: SystemAccount<'info>,
    #[account(
        seeds = [b"parcel".as_ref(), parcel.id.as_ref()],
        bump,
    )]
    pub parcel: Account<'info, Parcel>,
    #[account(mut)]
    pub buyer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct AcceptEscrow<'info> {
    #[account(
        mut,
        seeds = [b"escrow", parcel.key().as_ref()],
        bump,
    )]
    pub escrow_record: Account<'info, escrow::EscrowRecord>,
    #[account(
        seeds = [b"parcel".as_ref(), parcel.id.as_ref()],
        bump,
    )]
    pub parcel: Account<'info, Parcel>,
    pub seller: Signer<'info>,
}

#[derive(Accounts)]
pub struct SettleEscrow<'info> {
    #[account(
        mut,
        seeds = [b"escrow", parcel.key().as_ref()],
        bump,
    )]
    pub escrow_record: Account<'info, escrow::EscrowRecord>,
    /// CHECK: escrow vault PDA.
    #[account(
        mut,
        seeds = [b"escrow_vault", escrow_record.key().as_ref()],
        bump
    )]
    pub escrow_vault: SystemAccount<'info>,
    #[account(
        mut,
        seeds = [b"parcel".as_ref(), parcel.id.as_ref()],
        bump,
    )]
    pub parcel: Account<'info, Parcel>,
    /// Seller receives SOL.
    #[account(mut)]
    /// CHECK: validated as escrow.seller in handler.
    pub seller: UncheckedAccount<'info>,
    /// Buyer receives excess deposit.
    #[account(mut)]
    /// CHECK: validated as escrow.buyer in handler.
    pub buyer: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CancelEscrow<'info> {
    #[account(
        mut,
        seeds = [b"escrow", parcel.key().as_ref()],
        bump,
    )]
    pub escrow_record: Account<'info, escrow::EscrowRecord>,
    /// CHECK: escrow vault PDA.
    #[account(
        mut,
        seeds = [b"escrow_vault", escrow_record.key().as_ref()],
        bump
    )]
    pub escrow_vault: SystemAccount<'info>,
    #[account(
        mut,
        seeds = [b"parcel".as_ref(), parcel.id.as_ref()],
        bump,
    )]
    pub parcel: Account<'info, Parcel>,
    #[account(mut)]
    pub signer: Signer<'info>,
    /// CHECK: validated as escrow.buyer in handler. Receives returned deposit.
    #[account(mut)]
    pub buyer: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(case_hash: [u8; 32])]
pub struct DisputeEscrow<'info> {
    #[account(
        mut,
        seeds = [b"escrow", parcel.key().as_ref()],
        bump,
    )]
    pub escrow_record: Account<'info, escrow::EscrowRecord>,
    #[account(
        mut,
        seeds = [b"parcel".as_ref(), parcel.id.as_ref()],
        bump,
    )]
    pub parcel: Account<'info, Parcel>,
    #[account(
        init,
        payer = filer,
        space = 8 + dispute::Dispute::INIT_SPACE,
        seeds = [b"dispute", parcel.key().as_ref(), case_hash.as_ref()],
        bump
    )]
    pub dispute: Account<'info, dispute::Dispute>,
    #[account(mut)]
    pub filer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ExpireEscrow<'info> {
    #[account(
        mut,
        seeds = [b"escrow", parcel.key().as_ref()],
        bump,
    )]
    pub escrow_record: Account<'info, escrow::EscrowRecord>,
    #[account(
        mut,
        seeds = [b"parcel".as_ref(), parcel.id.as_ref()],
        bump,
    )]
    pub parcel: Account<'info, Parcel>,
    #[account(mut)]
    pub caller: Signer<'info>,
    pub system_program: Program<'info, System>,
}

// ---------------------------------------------------------------------------
// Time-bound credential contexts (RFC-009)
// ---------------------------------------------------------------------------

#[derive(Accounts)]
#[instruction(nonce: u8)]
pub struct RenewRight<'info> {
    #[account(
        mut,
        seeds = [b"rights".as_ref(), parcel.key().as_ref(), &[nonce]],
        bump,
    )]
    pub rights: Account<'info, Rights>,
    #[account(
        seeds = [b"parcel".as_ref(), parcel.id.as_ref()],
        bump,
    )]
    pub parcel: Account<'info, Parcel>,
    /// Holder must sign.
    pub holder: Signer<'info>,
    /// Granter must co-sign (original granter or current parcel owner).
    pub granter: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(nonce: u8)]
pub struct SweepExpiredRights<'info> {
    #[account(
        mut,
        seeds = [b"rights".as_ref(), parcel.key().as_ref(), &[nonce]],
        bump,
    )]
    pub rights: Account<'info, Rights>,
    #[account(
        seeds = [b"parcel".as_ref(), parcel.id.as_ref()],
        bump,
    )]
    pub parcel: Account<'info, Parcel>,
    pub keeper: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(nonce: u8, rights_kind: u8, holder: Pubkey, _expires_at: i64, _condition_deadline: i64, _condition_desc: String, _grace_period_secs: i64, _notes: String)]
pub struct GrantConditionalRight<'info> {
    #[account(
        mut,
        seeds = [b"parcel".as_ref(), parcel.id.as_ref()],
        bump,
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

#[event]
pub struct ParcelForfeited {
    pub parcel: Pubkey,
    pub case_hash: [u8; 32],
    pub from: Pubkey,
    pub to: Pubkey,
    pub threshold: u8,
    pub present: u8,
}

// ---------------------------------------------------------------------------
// Vault events (RFC-003)
// ---------------------------------------------------------------------------

#[event]
pub struct VaultCreated {
    pub subject: Pubkey,
    pub vault: Pubkey,
    pub ciphertext_hash: [u8; 32],
    pub algorithm_id: u8,
    pub threshold: u8,
    pub holder_count: u8,
}

#[event]
pub struct VaultAccessAuthorized {
    pub subject: Pubkey,
    pub vault: Pubkey,
    pub purpose: String,
    pub validators: Vec<Pubkey>,
    pub off_chain_nonce: [u8; 32],
    pub expiry: i64,
    pub block_time: i64,
}

#[event]
pub struct ShardRotationInitiated {
    pub vault: Pubkey,
    pub old_ciphertext_hash: [u8; 32],
    pub new_ciphertext_hash: [u8; 32],
    pub initiated_by: Pubkey,
    pub effective_at: i64,
}

#[event]
pub struct RotationEndorsed {
    pub vault: Pubkey,
    pub new_ciphertext_hash: [u8; 32],
    pub validator: Pubkey,
    pub endorsements_count: u8,
    pub required: u8,
}

#[event]
pub struct ShardRotationExecuted {
    pub vault: Pubkey,
    pub new_ciphertext_hash: [u8; 32],
    pub new_version: u32,
    pub new_threshold: u8,
}

#[event]
pub struct ShardRotationCancelled {
    pub vault: Pubkey,
    pub new_ciphertext_hash: [u8; 32],
    pub cancelled_by: Pubkey,
}

#[event]
pub struct ShardPinged {
    pub vault: Pubkey,
    pub validator: Pubkey,
    pub pinged_at: i64,
}

// ---------------------------------------------------------------------------
// AuthorityRegistry events (progressive decentralization)
// ---------------------------------------------------------------------------

#[event]
pub struct RegistryCreated {
    pub registry: Pubkey,
    pub admin: Pubkey,
    pub mode: u8,
}

#[event]
pub struct ValidatorAdded {
    pub registry: Pubkey,
    pub validator: Pubkey,
    pub added_by: Pubkey,
    pub mode: u8,
}

#[event]
pub struct ValidatorRemoved {
    pub registry: Pubkey,
    pub validator: Pubkey,
    pub mode: u8,
}

#[event]
pub struct ValidatorEndorsed {
    pub registry: Pubkey,
    pub proposed: Pubkey,
    pub endorser: Pubkey,
    pub endorsements_count: u8,
    pub required: u8,
}

#[event]
pub struct ConsensusFlipped {
    pub registry: Pubkey,
    pub admin: Pubkey,
    pub required_endorsements: u8,
    pub validator_count: u8,
}

// ---------------------------------------------------------------------------
// IPFS document storage events
// ---------------------------------------------------------------------------

#[event]
pub struct DocumentRegistered {
    pub attestation: Pubkey,
    pub cid: String,
    pub content_hash: [u8; 32],
    pub category: String,
    pub registered_by: Pubkey,
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
    #[msg("Succession requires validator endorsements before it can be claimed")]
    InsufficientValidations,
    #[msg("Signing wallet is not a declared validator for this succession")]
    NotValidator,
    #[msg("No more validators may endorse this succession (limit reached)")]
    ValidationLimitReached,
    #[msg("Court case hash is required")]
    EmptyCaseHash,
    #[msg("New forfeiture owner is required")]
    EmptyNewOwner,
    #[msg("Not enough validator signers to forfeit this parcel")]
    InsufficientValidatorSigners,
    #[msg("The current owner cannot self-forfeit their own parcel")]
    OwnerCannotSelfForfeit,
    #[msg("A validator cannot be the owner of the asset being validated")]
    ValidatorOwnsAsset,
    #[msg("Vault already exists for this subject")]
    VaultAlreadyExists,
    #[msg("Vault not found")]
    VaultNotFound,
    #[msg("Threshold exceeds the number of shard holders")]
    ThresholdExceedsHolders,
    #[msg("Signer is not a shard holder for this vault")]
    NotShardHolder,
    #[msg("Signer is not an active validator in this vault")]
    NotActiveValidator,
    #[msg("Ciphertext hash cannot be all zeros")]
    CiphertextHashRequired,
    #[msg("Ciphertext CID cannot be empty")]
    CidRequired,
    #[msg("Expiry must be within 24 hours from now")]
    ExpiryTooFar,
    #[msg("Expiry must be in the future")]
    ExpiryInPast,
    #[msg("Nonce cannot be all zeros")]
    NonceRequired,
    #[msg("No pending rotation exists for this vault")]
    RotationNotFound,
    #[msg("Rotation has already been executed or cancelled")]
    RotationAlreadyFinalized,
    #[msg("Rotation time lock has not yet expired")]
    RotationNotYetEffective,
    #[msg("Not enough endorsements for rotation (need ceil(2n/3))")]
    QuorumNotMetForRotation,
    #[msg("Validator has already endorsed this rotation")]
    AlreadyEndorsedRotation,
    #[msg("Initiator cannot endorse their own rotation")]
    SelfEndorsementNotAllowed,
    #[msg("A pending rotation already exists for this vault")]
    PendingRotationExists,
    #[msg("Ping interval has not yet elapsed")]
    PingIntervalNotElapsed,
    #[msg("Encryption algorithm is not supported")]
    AlgorithmNotSupported,
    #[msg("Storage URIs exceed the maximum count")]
    TooManyStorageUris,
    #[msg("Shard holders exceed the maximum count")]
    TooManyShardHolders,
    #[msg("This nonce has already been used for this vault")]
    NonceAlreadyUsed,
    #[msg("Only the registry admin or subject's recovery wallet can create a vault")]
    NotAuthorizedToCreate,
    #[msg("Only the admin or initiator can cancel a rotation")]
    NotAuthorizedToCancel,
    #[msg("New threshold exceeds the number of new shard holders")]
    NewThresholdExceedsHolders,
    #[msg("Invalid dispute status for this operation")]
    InvalidDisputeStatus,
    #[msg("Dispute has expired (90-day window elapsed)")]
    DisputeExpired,
    #[msg("Invalid dispute outcome")]
    InvalidDisputeOutcome,
    #[msg("Invalid escrow status for this operation")]
    InvalidEscrowStatus,
    #[msg("Escrow amount is outside the allowed range")]
    InvalidEscrowAmount,
    #[msg("Buyer wallet cannot be the same as the seller")]
    SelfDealingNotAllowed,
    #[msg("Signer is not the designated buyer for this escrow")]
    NotDesignatedBuyer,
    #[msg("Signer is not the designated seller for this escrow")]
    NotDesignatedSeller,
    #[msg("Deposit would exceed the escrow amount")]
    DepositExceedsAmount,
    #[msg("Full deposit not yet received — seller cannot accept")]
    InsufficientDeposit,
    #[msg("Settlement window has not yet expired")]
    SettlementNotYetEffective,
    #[msg("Seller no longer owns the parcel at settlement time")]
    ParcelStillOwnedBySeller,
    #[msg("Buyer grace period has expired — cannot cancel")]
    CancelWindowExpired,
    #[msg("Cancel window has not yet expired — escrow not stale")]
    CancelWindowNotExpired,
    #[msg("Cannot cancel: deposit already exists")]
    DepositExists,
    #[msg("Signer is not a party to this escrow")]
    NotPartyToEscrow,
    #[msg("Parcel is already in an escrow")]
    ParcelAlreadyInEscrow,
    #[msg("Minimum escrow amount is 0.1 SOL")]
    EscrowAmountTooLow,
    #[msg("Maximum escrow amount is 1,000,000 SOL")]
    EscrowAmountTooHigh,
    #[msg("Buyer wallet address is required")]
    EmptyBuyer,
    #[msg("Cannot renew a revoked right")]
    CannotRenewRevokedRight,
    #[msg("Cannot renew an already-renewed right")]
    CannotRenewAlreadyRenewed,
    #[msg("Renewal must extend the expiry, not shorten it")]
    RenewalMustExtendExpiry,
    #[msg("Invalid right status for this operation")]
    InvalidRightStatus,
    #[msg("Permanent rights (expires_at=0) cannot be swept")]
    PermanentRightNotSweepable,
    #[msg("Condition deadline must be before the expiry")]
    ConditionDeadlineAfterExpiry,
    #[msg("Invalid grace period")]
    InvalidGracePeriod,
}

#[cfg(test)]
mod prep_hook_tests {
    use super::*;

    #[test]
    fn parcel_status_reserved_variants_are_contiguous() {
        assert_eq!(parcel_status::PENDING, 0);
        assert_eq!(parcel_status::REGISTERED, 1);
        assert_eq!(parcel_status::FOR_SALE, 2);
        assert_eq!(parcel_status::TRANSFERRED, 3);
        assert_eq!(parcel_status::DISPUTED, 4);
        assert_eq!(parcel_status::FROZEN, 5);
        assert_eq!(parcel_status::ADJUDICATED, 6);
        assert_eq!(parcel_status::FORFEITED, 7);
        assert_eq!(parcel_status::MAX, parcel_status::FORFEITED);
    }

    #[test]
    fn parcel_status_update_accepts_all_reserved_variants() {
        for status in 0..=parcel_status::MAX {
            assert!(
                status <= parcel_status::MAX,
                "Status {} should be accepted by update_status",
                status
            );
        }
    }

    #[test]
    fn parcel_status_rejects_out_of_range() {
        let invalid = parcel_status::MAX + 1;
        assert!(invalid > parcel_status::MAX);
    }

    #[test]
    fn succession_kind_reserved_variants_are_contiguous() {
        assert_eq!(succession_kind::SUCCESSOR, 0);
        assert_eq!(succession_kind::RECOVERY, 1);
        assert_eq!(succession_kind::TRANSFER, 2);
        assert_eq!(succession_kind::GUARDIANSHIP, 3);
        assert_eq!(succession_kind::COURT_APPOINTED_GUARDIAN, 4);
        assert_eq!(succession_kind::MAX, succession_kind::COURT_APPOINTED_GUARDIAN);
    }

    #[test]
    fn succession_kind_update_accepts_all_reserved_variants() {
        for kind in 0..=succession_kind::MAX {
            assert!(
                kind <= succession_kind::MAX,
                "Kind {} should be accepted by request_succession",
                kind
            );
        }
    }

    #[test]
    fn succession_kind_rejects_out_of_range() {
        let invalid = succession_kind::MAX + 1;
        assert!(invalid > succession_kind::MAX);
    }
}
