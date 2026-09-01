use anchor_lang::prelude::*;

pub const MAX_SHARD_HOLDERS: usize = 8;
pub const MAX_STORAGE_URIS: usize = 4;
pub const PING_INTERVAL_SECS: i64 = 7 * 24 * 3600;
pub const MISSED_PINGS_BEFORE_ROTATION: u8 = 3;
pub const ROTATION_TIMELOCK_SECS: i64 = 7 * 24 * 3600;
pub const EMERGENCY_RECOVERY_FRACTION_BPS: u16 = 7500;

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

pub fn create_vault(
    _ctx: Context<super::CreateVault>,
    _ciphertext_cid: String,
    _ciphertext_hash: [u8; 32],
    _algorithm_id: u8,
    _storage_uris: Vec<String>,
    _shard_holders: Vec<Pubkey>,
    _threshold: u8,
) -> Result<()> {
    Ok(())
}

pub fn authorize_vault_access(
    _ctx: Context<super::AuthorizeVaultAccess>,
    _purpose: String,
    _expiry: i64,
    _off_chain_nonce: [u8; 32],
) -> Result<()> {
    Ok(())
}

pub fn initiate_shard_rotation(
    _ctx: Context<super::InitiateShardRotation>,
    _new_ciphertext_hash: [u8; 32],
    _new_shard_holders: Vec<Pubkey>,
    _new_threshold: u8,
) -> Result<()> {
    Ok(())
}

pub fn endorse_shard_rotation(_ctx: Context<super::EndorseShardRotation>) -> Result<()> {
    Ok(())
}

pub fn execute_shard_rotation(_ctx: Context<super::ExecuteShardRotation>) -> Result<()> {
    Ok(())
}

pub fn cancel_shard_rotation(_ctx: Context<super::CancelShardRotation>) -> Result<()> {
    Ok(())
}

pub fn ping_shard(_ctx: Context<super::PingShard>) -> Result<()> {
    Ok(())
}
