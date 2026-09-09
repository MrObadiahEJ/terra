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
}

// ---------------------------------------------------------------------------
// Cross-program mirror types (for reading accounts owned by other programs)
// ---------------------------------------------------------------------------

/// Read-only mirror of `terra_registry::Parcel` for cross-program
/// deserialization. Fields and layout must match the canonical definition
/// in terra_registry exactly (borsh order).
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct Parcel {
    pub id: [u8; 32],
    pub owner: Pubkey,
    pub name: String,
    pub geometry_hash: [u8; 32],
    pub status: u8,
    pub rights_count: u8,
    pub infrastructure_flags: u16,
    pub access_hash: [u8; 32],
    pub created_at: i64,
    pub updated_at: i64,
}
