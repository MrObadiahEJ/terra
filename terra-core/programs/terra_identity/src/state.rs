use anchor_lang::prelude::*;

use super::constants::MAX_VALIDATORS;

// ---------------------------------------------------------------------------
// Account types
// ---------------------------------------------------------------------------

#[account]
#[derive(InitSpace)]
pub struct Identity {
    pub identity_hash: [u8; 32],
    pub owner: Pubkey,
    pub recovery: Pubkey,
    pub parcel_count: u16,
    pub created_at: i64,
    pub updated_at: i64,
    pub pending_revocation: bool,
    pub pending_new_owner: Pubkey,
    pub revoke_after: i64,
}

#[account]
#[derive(InitSpace)]
pub struct Succession {
    pub identity: Pubkey,
    pub successor: Pubkey,
    pub kind: u8,
    pub requested_at: i64,
    pub effective_at: i64,
    pub grace_secs: i64,
    pub required: u8,
    pub validations_count: u8,
    pub validators: [Pubkey; MAX_VALIDATORS],
    /// Tracks which validators have endorsed. Each validator can endorse at most once.
    pub endorsers: [Pubkey; MAX_VALIDATORS],
    pub endorsers_count: u8,
}
