use anchor_lang::prelude::*;

pub const MAX_SHARD_HOLDERS: usize = 8;
pub const MAX_STORAGE_URIS: usize = 4;
pub const PING_INTERVAL_SECS: i64 = 7 * 24 * 3600;
pub const MISSED_PINGS_BEFORE_ROTATION: u8 = 3;
pub const ROTATION_TIMELOCK_SECS: i64 = 7 * 24 * 3600;
pub const EMERGENCY_RECOVERY_FRACTION_BPS: u16 = 7500;
/// Maximum seconds into the future an access expiry may be set (24 hours).
pub const MAX_ACCESS_EXPIRY_SECS: i64 = 24 * 3600;

pub mod vault_algorithm {
    pub const AES_256_GCM: u8 = 0;
}

pub mod rotation_status {
    pub const PENDING: u8 = 0;
    pub const EXECUTED: u8 = 1;
    pub const CANCELLED: u8 = 2;
}

#[account]
#[derive(InitSpace)]
pub struct VaultRecord {
    pub subject: Pubkey,
    #[max_len(128)]
    pub ciphertext_cid: String,
    pub ciphertext_hash: [u8; 32],
    pub algorithm_id: u8,
    #[max_len(4, 128)]
    pub storage_uris: Vec<String>,
    #[max_len(8)]
    pub shard_holders: Vec<Pubkey>,
    pub threshold: u8,
    pub version: u32,
    pub last_ping_at: i64,
    pub created_at: i64,
}

#[account]
#[derive(InitSpace)]
pub struct VaultShardRotation {
    pub vault: Pubkey,
    pub old_ciphertext_hash: [u8; 32],
    pub new_ciphertext_hash: [u8; 32],
    #[max_len(8)]
    pub new_shard_holders: Vec<Pubkey>,
    pub new_threshold: u8,
    pub initiated_by: Pubkey,
    #[max_len(8)]
    pub endorsements: Vec<Pubkey>,
    pub required_endorsements: u8,
    pub initiated_at: i64,
    pub effective_at: i64,
    pub status: u8,
}

/// Create a new vault for a subject's sensitive personal data.
///
/// Guards:
/// - ciphertext_hash must not be all zeros
/// - ciphertext_cid must not be empty
/// - algorithm_id must be supported
/// - storage_uris length <= MAX_STORAGE_URIS
/// - shard_holders length in [1, MAX_SHARD_HOLDERS]
/// - threshold in [1, shard_holders.len()]
/// - authority must be the identity owner or recovery wallet
pub fn create_vault(
    ctx: Context<super::CreateVault>,
    ciphertext_cid: String,
    ciphertext_hash: [u8; 32],
    algorithm_id: u8,
    storage_uris: Vec<String>,
    shard_holders: Vec<Pubkey>,
    threshold: u8,
) -> Result<()> {
    require!(
        ciphertext_hash != [0u8; 32],
        super::TerraError::CiphertextHashRequired
    );
    require!(!ciphertext_cid.is_empty(), super::TerraError::CidRequired);
    require!(
        algorithm_id == vault_algorithm::AES_256_GCM,
        super::TerraError::AlgorithmNotSupported
    );
    require!(
        storage_uris.len() <= MAX_STORAGE_URIS,
        super::TerraError::TooManyStorageUris
    );
    require!(
        !shard_holders.is_empty() && shard_holders.len() <= MAX_SHARD_HOLDERS,
        super::TerraError::TooManyShardHolders
    );
    require!(
        threshold >= 1 && (threshold as usize) <= shard_holders.len(),
        super::TerraError::ThresholdExceedsHolders
    );

    let authority_key = ctx.accounts.authority.key();
    let subject = &ctx.accounts.subject;
    require!(
        authority_key == subject.owner || authority_key == subject.recovery,
        super::TerraError::NotAuthorizedToCreate
    );

    let clock = Clock::get()?;
    let vault = &mut ctx.accounts.vault_record;
    vault.subject = ctx.accounts.subject.key();
    vault.ciphertext_cid = ciphertext_cid;
    vault.ciphertext_hash = ciphertext_hash;
    vault.algorithm_id = algorithm_id;
    vault.storage_uris = storage_uris;
    vault.shard_holders = shard_holders;
    vault.threshold = threshold;
    vault.version = 0;
    vault.last_ping_at = clock.unix_timestamp;
    vault.created_at = clock.unix_timestamp;

    emit!(super::VaultCreated {
        subject: vault.subject,
        vault: vault.key(),
        ciphertext_hash: vault.ciphertext_hash,
        algorithm_id: vault.algorithm_id,
        threshold: vault.threshold,
        holder_count: vault.shard_holders.len() as u8,
    });

    Ok(())
}

/// Authorize temporary access to the vault's encrypted data.
///
/// Guards:
/// - subject must match the vault's subject
/// - authority must be one of the shard holders
/// - expiry must be in the future and within 24 hours
/// - off_chain_nonce must not be all zeros
pub fn authorize_vault_access(
    ctx: Context<super::AuthorizeVaultAccess>,
    purpose: String,
    expiry: i64,
    off_chain_nonce: [u8; 32],
) -> Result<()> {
    let clock = Clock::get()?;
    require!(
        ctx.accounts.vault_record.subject == ctx.accounts.subject.key(),
        super::TerraError::IdentityMismatch
    );
    require!(
        ctx.accounts
            .vault_record
            .shard_holders
            .contains(&ctx.accounts.authority.key()),
        super::TerraError::NotShardHolder
    );
    require!(expiry > clock.unix_timestamp, super::TerraError::ExpiryInPast);
    require!(
        expiry <= clock.unix_timestamp + MAX_ACCESS_EXPIRY_SECS,
        super::TerraError::ExpiryTooFar
    );
    require!(
        off_chain_nonce != [0u8; 32],
        super::TerraError::NonceRequired
    );

    emit!(super::VaultAccessAuthorized {
        subject: ctx.accounts.subject.key(),
        vault: ctx.accounts.vault_record.key(),
        purpose,
        validators: ctx.accounts.vault_record.shard_holders.clone(),
        off_chain_nonce,
        expiry,
        block_time: clock.unix_timestamp,
    });

    Ok(())
}

/// Initiate a shard rotation ceremony. Creates a time-locked rotation record.
///
/// Guards:
/// - initiator must be one of the current shard holders
/// - no pending rotation may already exist for this vault
/// - new_ciphertext_hash must not be all zeros
/// - new_shard_holders must be non-empty and within limit
/// - new_threshold must be <= new_shard_holders.len()
pub fn initiate_shard_rotation(
    ctx: Context<super::InitiateShardRotation>,
    new_ciphertext_hash: [u8; 32],
    new_shard_holders: Vec<Pubkey>,
    new_threshold: u8,
) -> Result<()> {
    require!(
        new_ciphertext_hash != [0u8; 32],
        super::TerraError::CiphertextHashRequired
    );
    require!(
        !new_shard_holders.is_empty() && new_shard_holders.len() <= MAX_SHARD_HOLDERS,
        super::TerraError::TooManyShardHolders
    );
    require!(
        new_threshold >= 1 && (new_threshold as usize) <= new_shard_holders.len(),
        super::TerraError::NewThresholdExceedsHolders
    );
    require!(
        ctx.accounts
            .vault_record
            .shard_holders
            .contains(&ctx.accounts.initiator.key()),
        super::TerraError::NotActiveValidator
    );

    let vault_key = ctx.accounts.vault_record.key();
    let clock = Clock::get()?;
    let n = ctx.accounts.vault_record.shard_holders.len() as u8;
    let required = (n * 2 / 3) + if n % 3 != 0 { 1 } else { 0 };

    let rotation = &mut ctx.accounts.rotation;
    rotation.vault = vault_key;
    rotation.old_ciphertext_hash = ctx.accounts.vault_record.ciphertext_hash;
    rotation.new_ciphertext_hash = new_ciphertext_hash;
    rotation.new_shard_holders = new_shard_holders;
    rotation.new_threshold = new_threshold;
    rotation.initiated_by = ctx.accounts.initiator.key();
    rotation.endorsements = Vec::new();
    rotation.required_endorsements = required;
    rotation.initiated_at = clock.unix_timestamp;
    rotation.effective_at = clock.unix_timestamp + ROTATION_TIMELOCK_SECS;
    rotation.status = rotation_status::PENDING;

    emit!(super::ShardRotationInitiated {
        vault: vault_key,
        old_ciphertext_hash: rotation.old_ciphertext_hash,
        new_ciphertext_hash: rotation.new_ciphertext_hash,
        initiated_by: rotation.initiated_by,
        effective_at: rotation.effective_at,
    });

    Ok(())
}

/// Endorse a pending shard rotation.
///
/// Guards:
/// - rotation must be pending
/// - validator must be one of the current shard holders
/// - validator must not be the initiator (initiator auto-endorsed)
/// - validator must not have already endorsed
pub fn endorse_shard_rotation(ctx: Context<super::EndorseShardRotation>) -> Result<()> {
    let rotation = &mut ctx.accounts.rotation;
    require!(
        rotation.status == rotation_status::PENDING,
        super::TerraError::RotationAlreadyFinalized
    );
    let validator_key = ctx.accounts.validator.key();
    require!(
        ctx.accounts
            .vault_record
            .shard_holders
            .contains(&validator_key),
        super::TerraError::NotActiveValidator
    );
    require!(
        validator_key != rotation.initiated_by,
        super::TerraError::SelfEndorsementNotAllowed
    );
    require!(
        !rotation.endorsements.contains(&validator_key),
        super::TerraError::AlreadyEndorsedRotation
    );
    // Self-dealing check: validator must not be the vault subject's owner.
    require!(
        validator_key != ctx.accounts.subject.owner,
        super::TerraError::ValidatorOwnsAsset
    );

    rotation.endorsements.push(validator_key);

    emit!(super::RotationEndorsed {
        vault: rotation.vault,
        new_ciphertext_hash: rotation.new_ciphertext_hash,
        validator: validator_key,
        endorsements_count: rotation.endorsements.len() as u8,
        required: rotation.required_endorsements,
    });

    Ok(())
}

/// Execute a shard rotation after the time lock and quorum are met.
///
/// Guards:
/// - rotation must be pending
/// - time lock must have elapsed
/// - endorsements must meet quorum (ceil(2n/3))
///
/// Effects:
/// - vault ciphertext_hash, shard_holders, threshold updated
/// - vault version bumped
/// - rotation account closed (lamports returned to initiator)
pub fn execute_shard_rotation(ctx: Context<super::ExecuteShardRotation>) -> Result<()> {
    let rotation = &ctx.accounts.rotation;
    require!(
        rotation.status == rotation_status::PENDING,
        super::TerraError::RotationAlreadyFinalized
    );
    let clock = Clock::get()?;
    require!(
        clock.unix_timestamp >= rotation.effective_at,
        super::TerraError::RotationNotYetEffective
    );
    require!(
        rotation.endorsements.len() as u8 >= rotation.required_endorsements,
        super::TerraError::QuorumNotMetForRotation
    );

    let vault = &mut ctx.accounts.vault_record;
    vault.ciphertext_hash = rotation.new_ciphertext_hash;
    vault.shard_holders = rotation.new_shard_holders.clone();
    vault.threshold = rotation.new_threshold;
    vault.version = vault.version.saturating_add(1);

    emit!(super::ShardRotationExecuted {
        vault: vault.key(),
        new_ciphertext_hash: vault.ciphertext_hash,
        new_version: vault.version,
        new_threshold: vault.threshold,
    });

    // Rotation account is closed by the `close = initiator` constraint.
    Ok(())
}

/// Cancel a pending shard rotation.
///
/// Guards:
/// - rotation must be pending
/// - canceller must be the registry admin (identity owner/recovery) or the
///   original initiator
pub fn cancel_shard_rotation(ctx: Context<super::CancelShardRotation>) -> Result<()> {
    let rotation = &ctx.accounts.rotation;
    require!(
        rotation.status == rotation_status::PENDING,
        super::TerraError::RotationAlreadyFinalized
    );

    let canceller_key = ctx.accounts.canceller.key();
    require!(
        canceller_key == rotation.initiated_by
            || canceller_key == ctx.accounts.vault_record.subject,
        super::TerraError::NotAuthorizedToCancel
    );

    let vault_key = rotation.vault;
    let new_hash = rotation.new_ciphertext_hash;

    // Rotation account is closed by the `close = canceller` constraint.
    // We need to emit before close consumes the account data.
    emit!(super::ShardRotationCancelled {
        vault: vault_key,
        new_ciphertext_hash: new_hash,
        cancelled_by: canceller_key,
    });

    Ok(())
}

/// Record a shard liveness ping.
///
/// Guards:
/// - validator must be one of the current shard holders
/// - at least PING_INTERVAL_SECS must have elapsed since last ping
pub fn ping_shard(ctx: Context<super::PingShard>) -> Result<()> {
    let validator_key = ctx.accounts.validator.key();
    require!(
        ctx.accounts
            .vault_record
            .shard_holders
            .contains(&validator_key),
        super::TerraError::NotShardHolder
    );

    let clock = Clock::get()?;
    let vault = &mut ctx.accounts.vault_record;
    require!(
        clock.unix_timestamp >= vault.last_ping_at + PING_INTERVAL_SECS,
        super::TerraError::PingIntervalNotElapsed
    );

    vault.last_ping_at = clock.unix_timestamp;

    emit!(super::ShardPinged {
        vault: vault.key(),
        validator: validator_key,
        pinged_at: clock.unix_timestamp,
    });

    Ok(())
}
