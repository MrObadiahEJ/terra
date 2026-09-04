use anchor_lang::prelude::*;

use crate::{parcel_status, TerraError, MAX_VALIDATORS};

pub mod dispute_status {
    pub const FILED: u8 = 0;
    pub const FROZEN: u8 = 1;
    pub const ADJUDICATED: u8 = 2;
    pub const EXECUTED: u8 = 3;
    pub const CANCELLED: u8 = 4;
}

/// Minimum validator co-signatures required to FILE a dispute (anti-grief).
pub const MIN_DISPUTE_VALIDATORS: u8 = 2;

/// Disputes auto-expire after this window if not adjudicated.
pub const DISPUTE_EXPIRY_SECS: i64 = 90 * 24 * 3600; // 90 days

/// Outcome of an adjudicated dispute.
pub mod dispute_outcome {
    pub const OWNER_WINS: u8 = 0;
    pub const OWNER_LOSES: u8 = 1;
}

/// An on-chain dispute record bound to a parcel.
///
/// PDA: `["dispute", parcel, case_hash]`. The `case_hash` is a SHA-256 over
/// the off-chain court document or complaint, anchoring evidence immutably.
#[account]
#[derive(InitSpace)]
pub struct Dispute {
    /// The parcel under dispute.
    pub parcel: Pubkey,
    /// The wallet that filed the dispute.
    pub filed_by: Pubkey,
    /// SHA-256 of the off-chain court document / complaint.
    pub case_hash: [u8; 32],
    /// Current dispute status (FILED → FROZEN → ADJUDICATED → EXECUTED).
    pub status: u8,
    /// Required validator co-signatures to advance this dispute.
    pub required: u8,
    /// Number of validators declared at filing time (NOT actual signatures collected).
    pub declared_count: u8,
    /// Declared validator set for this dispute.
    pub validators: [Pubkey; MAX_VALIDATORS],
    pub filed_at: i64,
    pub frozen_at: i64,
    pub adjudicated_at: i64,
    /// Outcome of adjudication (OWNER_WINS or OWNER_LOSES). Set at adjudication.
    pub outcome: u8,
    /// New owner if outcome is OWNER_LOSES. Set at adjudication.
    pub new_owner: Pubkey,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

pub fn file_dispute(
    ctx: Context<super::FileDispute>,
    case_hash: [u8; 32],
    required: u8,
    validators: [Pubkey; MAX_VALIDATORS],
) -> Result<()> {
    require!(
        !case_hash.iter().all(|b| *b == 0),
        TerraError::EmptyCaseHash
    );
    require!(
        required >= MIN_DISPUTE_VALIDATORS,
        TerraError::InvalidThreshold
    );

    // Count declared validators and enforce self-dealing checks.
    let mut count: u8 = 0;
    for &v in validators.iter() {
        if v == Pubkey::default() {
            continue;
        }
        // The filer cannot be their own validator (self-dealing).
        require!(
            v != ctx.accounts.filer.key(),
            TerraError::ValidatorOwnsAsset
        );
        // The parcel owner cannot be a dispute validator (self-dealing).
        require!(
            v != ctx.accounts.parcel.owner,
            TerraError::ValidatorOwnsAsset
        );
        count += 1;
    }
    require!(count > 0, TerraError::NoValidators);
    require!(
        (required as usize) <= count as usize,
        TerraError::InvalidThreshold
    );

    // Parcel must be in REGISTERED status to be disputed.
    require!(
        ctx.accounts.parcel.status == parcel_status::REGISTERED,
        TerraError::InvalidStatus
    );

    let now = Clock::get()?.unix_timestamp;
    let dispute = &mut ctx.accounts.dispute;
    dispute.parcel = ctx.accounts.parcel.key();
    dispute.filed_by = ctx.accounts.filer.key();
    dispute.case_hash = case_hash;
    dispute.status = dispute_status::FILED;
    dispute.required = required;
    dispute.declared_count = count;
    dispute.validators = validators;
    dispute.filed_at = now;

    // Update parcel status to DISPUTED.
    let parcel = &mut ctx.accounts.parcel;
    parcel.status = parcel_status::DISPUTED;
    parcel.updated_at = now;

    emit!(DisputeFiled {
        dispute: dispute.key(),
        parcel: parcel.key(),
        filed_by: dispute.filed_by,
        case_hash,
        required,
        declared_count: count,
    });
    Ok(())
}

pub fn freeze_parcel(ctx: Context<super::FreezeParcel>) -> Result<()> {
    let dispute = &ctx.accounts.dispute;
    require!(
        dispute.status == dispute_status::FILED,
        TerraError::InvalidDisputeStatus
    );

    // Check expiry.
    let now = Clock::get()?.unix_timestamp;
    require!(
        now < dispute.filed_at + DISPUTE_EXPIRY_SECS,
        TerraError::DisputeExpired
    );

    // The parcel must currently be DISPUTED.
    require!(
        ctx.accounts.parcel.status == parcel_status::DISPUTED,
        TerraError::InvalidStatus
    );

    // Count actual signer validators via remaining_accounts.
    let mut present: u8 = 0;
    for signer in ctx.remaining_accounts.iter() {
        if signer.is_signer && dispute.validators.contains(&signer.key()) {
            present += 1;
        }
    }
    require!(
        present >= dispute.required,
        TerraError::InsufficientValidatorSigners
    );

    let dispute = &mut ctx.accounts.dispute;
    dispute.status = dispute_status::FROZEN;
    dispute.frozen_at = now;

    let parcel = &mut ctx.accounts.parcel;
    parcel.status = parcel_status::FROZEN;
    parcel.updated_at = now;

    emit!(ParcelFrozen {
        dispute: dispute.key(),
        parcel: parcel.key(),
        frozen_at: now,
    });
    Ok(())
}

pub fn adjudicate_dispute(
    ctx: Context<super::AdjudicateDispute>,
    outcome: u8,
    new_owner: Pubkey,
) -> Result<()> {
    require!(
        outcome <= dispute_outcome::OWNER_LOSES,
        TerraError::InvalidDisputeOutcome
    );

    let dispute = &ctx.accounts.dispute;
    require!(
        dispute.status == dispute_status::FROZEN,
        TerraError::InvalidDisputeStatus
    );

    // Check expiry.
    let now = Clock::get()?.unix_timestamp;
    require!(
        now < dispute.filed_at + DISPUTE_EXPIRY_SECS,
        TerraError::DisputeExpired
    );

    // Count actual signer validators via remaining_accounts.
    let mut present: u8 = 0;
    for signer in ctx.remaining_accounts.iter() {
        if signer.is_signer && dispute.validators.contains(&signer.key()) {
            present += 1;
        }
    }
    require!(
        present >= dispute.required,
        TerraError::InsufficientValidatorSigners
    );

    // If owner loses, new_owner must be provided.
    if outcome == dispute_outcome::OWNER_LOSES {
        require!(new_owner != Pubkey::default(), TerraError::EmptyNewOwner);
    }

    let dispute = &mut ctx.accounts.dispute;
    dispute.status = dispute_status::ADJUDICATED;
    dispute.adjudicated_at = now;
    dispute.outcome = outcome;
    dispute.new_owner = new_owner;

    let parcel = &mut ctx.accounts.parcel;
    parcel.status = parcel_status::ADJUDICATED;
    parcel.updated_at = now;

    emit!(DisputeAdjudicated {
        dispute: dispute.key(),
        parcel: parcel.key(),
        outcome,
        new_owner,
        adjudicated_at: now,
    });
    Ok(())
}

pub fn execute_judgment(ctx: Context<super::ExecuteJudgment>) -> Result<()> {
    let dispute = &ctx.accounts.dispute;
    require!(
        dispute.status == dispute_status::ADJUDICATED,
        TerraError::InvalidDisputeStatus
    );

    let now = Clock::get()?.unix_timestamp;
    let parcel = &mut ctx.accounts.parcel;

    if dispute.outcome == dispute_outcome::OWNER_WINS {
        // Owner wins — unfreeze, return to ACTIVE.
        parcel.status = parcel_status::REGISTERED;
    } else {
        // Owner loses — forfeit to new_owner.
        require!(
            dispute.new_owner != Pubkey::default(),
            TerraError::EmptyNewOwner
        );
        parcel.owner = dispute.new_owner;
        parcel.status = parcel_status::FORFEITED;
    }
    parcel.updated_at = now;

    let dispute = &mut ctx.accounts.dispute;
    dispute.status = dispute_status::EXECUTED;

    emit!(JudgmentExecuted {
        dispute: dispute.key(),
        parcel: parcel.key(),
        outcome: dispute.outcome,
        new_owner: dispute.new_owner,
        executed_at: now,
    });
    Ok(())
}

pub fn cancel_dispute(ctx: Context<super::CancelDispute>) -> Result<()> {
    let dispute = &ctx.accounts.dispute;
    require!(
        dispute.status == dispute_status::FILED,
        TerraError::InvalidDisputeStatus
    );
    // Only the filer or parcel owner can cancel before adjudication.
    let signer = ctx.accounts.signer.key();
    require!(
        signer == dispute.filed_by || signer == ctx.accounts.parcel.owner,
        TerraError::NotAuthorized
    );

    let now = Clock::get()?.unix_timestamp;
    let parcel = &mut ctx.accounts.parcel;
    // Only unfreeze if the parcel is in DISPUTED status (not already frozen).
    if parcel.status == parcel_status::DISPUTED {
        parcel.status = parcel_status::REGISTERED;
    }
    parcel.updated_at = now;

    let dispute = &mut ctx.accounts.dispute;
    dispute.status = dispute_status::CANCELLED;

    emit!(DisputeCancelled {
        dispute: dispute.key(),
        parcel: parcel.key(),
        cancelled_by: signer,
    });
    Ok(())
}

// ---------------------------------------------------------------------------
// Events
// ---------------------------------------------------------------------------

#[event]
pub struct DisputeFiled {
    pub dispute: Pubkey,
    pub parcel: Pubkey,
    pub filed_by: Pubkey,
    pub case_hash: [u8; 32],
    pub required: u8,
    pub declared_count: u8,
}

#[event]
pub struct ParcelFrozen {
    pub dispute: Pubkey,
    pub parcel: Pubkey,
    pub frozen_at: i64,
}

#[event]
pub struct DisputeAdjudicated {
    pub dispute: Pubkey,
    pub parcel: Pubkey,
    pub outcome: u8,
    pub new_owner: Pubkey,
    pub adjudicated_at: i64,
}

#[event]
pub struct JudgmentExecuted {
    pub dispute: Pubkey,
    pub parcel: Pubkey,
    pub outcome: u8,
    pub new_owner: Pubkey,
    pub executed_at: i64,
}

#[event]
pub struct DisputeCancelled {
    pub dispute: Pubkey,
    pub parcel: Pubkey,
    pub cancelled_by: Pubkey,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dispute_status_values_are_contiguous() {
        assert_eq!(dispute_status::FILED, 0);
        assert_eq!(dispute_status::FROZEN, 1);
        assert_eq!(dispute_status::ADJUDICATED, 2);
        assert_eq!(dispute_status::EXECUTED, 3);
        assert_eq!(dispute_status::CANCELLED, 4);
    }

    #[test]
    fn dispute_outcome_values_are_correct() {
        assert_eq!(dispute_outcome::OWNER_WINS, 0);
        assert_eq!(dispute_outcome::OWNER_LOSES, 1);
    }

    #[test]
    fn min_dispute_validators_prevents_unilateral_filing() {
        assert!(
            MIN_DISPUTE_VALIDATORS >= 2,
            "Minimum dispute validators must be at least 2 to prevent griefing"
        );
    }

    #[test]
    fn dispute_expiry_is_90_days() {
        let expected_secs = 90 * 24 * 3600;
        assert_eq!(DISPUTE_EXPIRY_SECS, expected_secs);
    }
}
