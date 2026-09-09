use anchor_lang::prelude::*;

use crate::TerraError;

// ---------------------------------------------------------------------------
// Audit action constants
// ---------------------------------------------------------------------------

pub mod audit_action {
    pub const CLAIM_CREATED: u8 = 0;
    pub const EVIDENCE_ADDED: u8 = 1;
    pub const OBSERVATION_SUBMITTED: u8 = 2;
    pub const ATTESTATION_SUBMITTED: u8 = 3;
    pub const CLAIM_VERIFIED: u8 = 4;
    pub const CLAIM_REJECTED: u8 = 5;
    pub const SESSION_OPENED: u8 = 6;
    pub const SESSION_CLOSED: u8 = 7;
    pub const CHALLENGE_FILED: u8 = 8;
    pub const CHALLENGE_RESOLVED: u8 = 9;
    pub const GUARDIAN_CLAIM: u8 = 10;
    pub const CROSS_BORDER_LINKED: u8 = 11;
    pub const OBSERVER_REGISTERED: u8 = 12;
    pub const VOTE_CAST: u8 = 13;
    pub const QUORUM_REACHED: u8 = 14;
    pub const MAX: u8 = QUORUM_REACHED;
}

// ---------------------------------------------------------------------------
// AuditEntry account
// ---------------------------------------------------------------------------

/// Append-only audit log of all verification state transitions.
///
/// PDA: `["audit_entry", entity, sequence_bytes]`
#[account]
#[derive(InitSpace)]
pub struct AuditEntry {
    /// The entity this audit entry is about (claim, session, challenge, etc.).
    pub entity: Pubkey,
    /// Monotonic sequence number for this entity.
    pub sequence: u32,
    /// Type of transition (see audit_action constants).
    pub action: u8,
    /// Previous status value.
    pub from_status: u8,
    /// New status value.
    pub to_status: u8,
    /// Who triggered the transition.
    pub actor: Pubkey,
    /// Optional metadata hash (sha-256 of additional context).
    pub metadata_hash: [u8; 32],
    pub timestamp: i64,
}

// ---------------------------------------------------------------------------
// Instruction handlers
// ---------------------------------------------------------------------------

/// Append an audit entry. The entity PDA must be derivable. Uses `init`
/// with PDA seeds `["audit_entry", entity, sequence_bytes]`. The sequence
/// is passed as an arg and must match what's expected (starts at 0, increments).
pub fn record_audit_entry(
    ctx: Context<crate::RecordAuditEntry>,
    sequence: u32,
    action: u8,
    from_status: u8,
    to_status: u8,
    metadata_hash: [u8; 32],
) -> Result<()> {
    require!(action <= audit_action::MAX, TerraError::InvalidAuditAction);

    let now = Clock::get()?.unix_timestamp;
    let entry = &mut ctx.accounts.audit_entry;
    entry.entity = ctx.accounts.entity.key();
    entry.sequence = sequence;
    entry.action = action;
    entry.from_status = from_status;
    entry.to_status = to_status;
    entry.actor = ctx.accounts.actor.key();
    entry.metadata_hash = metadata_hash;
    entry.timestamp = now;

    emit!(crate::AuditEntryRecorded {
        audit_entry: entry.key(),
        entity: entry.entity,
        sequence: entry.sequence,
        action,
        from_status,
        to_status,
        actor: entry.actor,
        timestamp: now,
    });
    Ok(())
}
