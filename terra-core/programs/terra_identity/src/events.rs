use anchor_lang::prelude::*;

// ---------------------------------------------------------------------------
// Events
// ---------------------------------------------------------------------------

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
}

#[event]
pub struct CourtGuardianshipRequested {
    pub identity: Pubkey,
    pub successor: Pubkey,
    pub grace_secs: i64,
    pub required: u8,
    pub count: u8,
    pub effective_at: i64,
    pub case_hash: [u8; 32],
    pub scope_notes: String,
}

#[event]
pub struct GuardianshipRevocationRequested {
    pub identity: Pubkey,
    pub previous_guardian: Pubkey,
    pub new_owner: Pubkey,
    pub requested_by: Pubkey,
    pub revoke_after: i64,
    pub block_time: i64,
}

#[event]
pub struct GuardianshipRevoked {
    pub identity: Pubkey,
    pub previous_guardian: Pubkey,
    pub new_owner: Pubkey,
    pub revoked_by: Pubkey,
    pub block_time: i64,
}
