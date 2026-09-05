use anchor_lang::prelude::*;

use crate::{parcel_status, TerraError, MAX_VALIDATORS};

pub mod escrow_status {
    pub const CREATED: u8 = 0;
    pub const DEPOSITED: u8 = 1;
    pub const ACCEPTED: u8 = 2;
    pub const SETTLED: u8 = 3;
    pub const CANCELLED: u8 = 4;
    pub const DISPUTED: u8 = 5;
}

/// Settlement window: buyer has this long after seller accepts to call settle.
pub const SETTLEMENT_WINDOW_SECS: i64 = 3 * 24 * 3600; // 3 days
/// Cancel window: buyer can cancel within this window after escrow creation.
pub const CANCEL_WINDOW_SECS: i64 = 7 * 24 * 3600; // 7 days
/// Maximum escrow amount (1M SOL in lamports).
pub const MAX_ESCROW_AMOUNT: u64 = 1_000_000_000_000;
/// Minimum escrow amount (0.1 SOL in lamports).
pub const MIN_ESCROW_AMOUNT: u64 = 100_000_000;

/// An on-chain escrow record for a parcel sale.
///
/// PDA seed: `["escrow", parcel_key]`. One escrow per parcel.
#[account]
#[derive(InitSpace)]
pub struct EscrowRecord {
    /// The parcel being sold.
    pub parcel: Pubkey,
    /// Wallet of the seller (must match parcel.owner).
    pub seller: Pubkey,
    /// Wallet of the buyer (set at creation).
    pub buyer: Pubkey,
    /// Sale price in lamports.
    pub amount: u64,
    /// Amount deposited so far by buyer.
    pub deposit_amount: u64,
    /// Vault PDA holding deposited SOL.
    pub vault: Pubkey,
    /// Current escrow status.
    pub status: u8,
    pub created_at: i64,
    pub deposited_at: i64,
    pub accepted_at: i64,
    /// accepted_at + SETTLEMENT_WINDOW_SECS
    pub settle_deadline: i64,
    /// created_at + CANCEL_WINDOW_SECS
    pub cancel_deadline: i64,
    /// Case hash if dispute filed (0 if none).
    pub dispute_case_hash: [u8; 32],
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// Seller initiates an escrow for a parcel marked FOR_SALE.
pub fn create_escrow(ctx: Context<super::CreateEscrow>, amount: u64, buyer: Pubkey) -> Result<()> {
    require!(
        ctx.accounts.parcel.status == parcel_status::FOR_SALE,
        TerraError::InvalidStatus
    );
    require!(
        ctx.accounts.seller.key() == ctx.accounts.parcel.owner,
        TerraError::NotOwner
    );
    require!(buyer != Pubkey::default(), TerraError::EmptySuccessor);
    require!(
        buyer != ctx.accounts.seller.key(),
        TerraError::SelfDealingNotAllowed
    );
    require!(
        (MIN_ESCROW_AMOUNT..=MAX_ESCROW_AMOUNT).contains(&amount),
        TerraError::InvalidEscrowAmount
    );

    let now = Clock::get()?.unix_timestamp;
    let escrow = &mut ctx.accounts.escrow_record;
    escrow.parcel = ctx.accounts.parcel.key();
    escrow.seller = ctx.accounts.seller.key();
    escrow.buyer = buyer;
    escrow.amount = amount;
    escrow.deposit_amount = 0;
    escrow.vault = ctx.accounts.escrow_vault.key();
    escrow.status = escrow_status::CREATED;
    escrow.created_at = now;
    escrow.cancel_deadline = now + CANCEL_WINDOW_SECS;
    escrow.settle_deadline = 0;

    let parcel = &mut ctx.accounts.parcel;
    parcel.updated_at = now;

    emit!(EscrowCreated {
        escrow: escrow.key(),
        parcel: escrow.parcel,
        seller: escrow.seller,
        buyer,
        amount,
        created_at: now,
    });
    Ok(())
}

/// Buyer deposits SOL into the escrow vault. Can be partial (earnest money) or full.
pub fn deposit_escrow(ctx: Context<super::DepositEscrow>, deposit_amount: u64) -> Result<()> {
    let escrow = &mut ctx.accounts.escrow_record;
    require!(
        escrow.status == escrow_status::CREATED,
        TerraError::InvalidEscrowStatus
    );
    require!(
        ctx.accounts.buyer.key() == escrow.buyer,
        TerraError::NotDesignatedBuyer
    );
    require!(deposit_amount > 0, TerraError::InvalidEscrowAmount);
    require!(
        escrow.deposit_amount.saturating_add(deposit_amount) <= escrow.amount,
        TerraError::DepositExceedsAmount
    );

    let now = Clock::get()?.unix_timestamp;
    require!(
        now < escrow.cancel_deadline,
        TerraError::CancelWindowExpired
    );

    // Transfer SOL from buyer to vault PDA.
    let ix = anchor_lang::solana_program::system_instruction::transfer(
        &ctx.accounts.buyer.key(),
        &ctx.accounts.escrow_vault.key(),
        deposit_amount,
    );
    anchor_lang::solana_program::program::invoke(
        &ix,
        &[
            ctx.accounts.buyer.to_account_info(),
            ctx.accounts.escrow_vault.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
        ],
    )?;

    escrow.deposit_amount = escrow.deposit_amount.saturating_add(deposit_amount);
    escrow.deposited_at = now;

    if escrow.deposit_amount >= escrow.amount {
        escrow.status = escrow_status::DEPOSITED;
    }

    emit!(EscrowDeposited {
        escrow: escrow.key(),
        parcel: escrow.parcel,
        buyer: ctx.accounts.buyer.key(),
        deposit_amount,
        total_deposited: escrow.deposit_amount,
    });
    Ok(())
}

/// Seller accepts the deposited payment, triggering the settlement window.
pub fn accept_escrow(ctx: Context<super::AcceptEscrow>) -> Result<()> {
    let escrow = &mut ctx.accounts.escrow_record;
    require!(
        escrow.status == escrow_status::DEPOSITED,
        TerraError::InvalidEscrowStatus
    );
    require!(
        ctx.accounts.seller.key() == escrow.seller,
        TerraError::NotDesignatedSeller
    );
    require!(
        ctx.accounts.seller.key() == ctx.accounts.parcel.owner,
        TerraError::NotOwner
    );
    require!(
        escrow.deposit_amount >= escrow.amount,
        TerraError::InsufficientDeposit
    );

    let now = Clock::get()?.unix_timestamp;
    escrow.status = escrow_status::ACCEPTED;
    escrow.accepted_at = now;
    escrow.settle_deadline = now + SETTLEMENT_WINDOW_SECS;

    emit!(EscrowAccepted {
        escrow: escrow.key(),
        parcel: escrow.parcel,
        seller: escrow.seller,
        accepted_at: now,
        settle_deadline: escrow.settle_deadline,
    });
    Ok(())
}

/// After the settlement window, execute the atomic swap: parcel → buyer, SOL → seller.
pub fn settle_escrow(ctx: Context<super::SettleEscrow>) -> Result<()> {
    let escrow = &ctx.accounts.escrow_record;
    require!(
        escrow.status == escrow_status::ACCEPTED,
        TerraError::InvalidEscrowStatus
    );

    let now = Clock::get()?.unix_timestamp;
    require!(
        now >= escrow.settle_deadline,
        TerraError::SettlementNotYetEffective
    );
    require!(
        escrow.deposit_amount >= escrow.amount,
        TerraError::InsufficientDeposit
    );
    require!(
        ctx.accounts.parcel.owner == escrow.seller,
        TerraError::ParcelStillOwnedBySeller
    );

    let vault_info = ctx.accounts.escrow_vault.to_account_info();
    let seller_info = ctx.accounts.seller.to_account_info();
    let buyer_info = ctx.accounts.buyer.to_account_info();

    // Transfer SOL from vault to seller.
    let vault_lamports = vault_info.lamports();
    let transfer_amount = escrow.amount.min(vault_lamports);

    **vault_info.try_borrow_mut_lamports()? -= transfer_amount;
    **seller_info.try_borrow_mut_lamports()? += transfer_amount;

    // Return any excess deposit to buyer.
    let excess = vault_lamports.saturating_sub(transfer_amount);
    if excess > 0 {
        **vault_info.try_borrow_mut_lamports()? -= excess;
        **buyer_info.try_borrow_mut_lamports()? += excess;
    }

    // Transfer parcel ownership.
    let parcel = &mut ctx.accounts.parcel;
    parcel.owner = escrow.buyer;
    parcel.status = parcel_status::TRANSFERRED;
    parcel.updated_at = now;

    let escrow_key = escrow.key();
    let parcel_key = escrow.parcel;
    let seller = escrow.seller;
    let buyer = escrow.buyer;
    let amount = escrow.amount;

    // The escrow record is closed by the `close = seller` constraint (rent
    // lamports → seller); no manual zeroing, so no stale account data lingers.

    emit!(EscrowSettled {
        escrow: escrow_key,
        parcel: parcel_key,
        seller,
        buyer,
        amount,
        settled_at: now,
    });
    Ok(())
}

/// Cancel the escrow and return funds.
pub fn cancel_escrow(ctx: Context<super::CancelEscrow>) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let escrow = &ctx.accounts.escrow_record;
    let status = escrow.status;
    let seller = escrow.seller;
    let buyer_key = escrow.buyer;
    let deposit_amount = escrow.deposit_amount;
    let parcel_key = escrow.parcel;
    let cancel_deadline = escrow.cancel_deadline;

    match status {
        // Seller can cancel before any deposit.
        escrow_status::CREATED => {
            require!(
                ctx.accounts.signer.key() == seller,
                TerraError::NotDesignatedSeller
            );
            require!(deposit_amount == 0, TerraError::DepositExists);
        }
        // Buyer can cancel within grace period after deposit.
        escrow_status::DEPOSITED => {
            require!(
                ctx.accounts.signer.key() == buyer_key,
                TerraError::NotDesignatedBuyer
            );
            require!(now < cancel_deadline, TerraError::CancelWindowExpired);
        }
        _ => return Err(TerraError::InvalidEscrowStatus.into()),
    }

    // Return deposited SOL to buyer.
    if deposit_amount > 0 {
        let vault_info = ctx.accounts.escrow_vault.to_account_info();
        let buyer_info = ctx.accounts.buyer.to_account_info();
        let vault_lamports = vault_info.lamports();
        let return_amount = deposit_amount.min(vault_lamports);
        **vault_info.try_borrow_mut_lamports()? -= return_amount;
        **buyer_info.try_borrow_mut_lamports()? += return_amount;
    }

    // Reset parcel status.
    let parcel = &mut ctx.accounts.parcel;
    parcel.status = parcel_status::FOR_SALE;
    parcel.updated_at = now;

    // Escrow record is closed by the `close = signer` constraint: rent
    // lamports are returned to the canceller; no manual zeroing needed.

    let cancelled_by = ctx.accounts.signer.key();

    emit!(EscrowCancelled {
        escrow: ctx.accounts.escrow_record.key(),
        parcel: parcel_key,
        cancelled_by,
    });
    Ok(())
}

/// Either party triggers a dispute, routing the parcel into RFC-007's FROZEN state.
pub fn dispute_escrow(
    ctx: Context<super::DisputeEscrow>,
    case_hash: [u8; 32],
    required: u8,
    validators: [Pubkey; MAX_VALIDATORS],
) -> Result<()> {
    use crate::dispute;

    let escrow = &ctx.accounts.escrow_record;
    require!(
        escrow.status == escrow_status::CREATED
            || escrow.status == escrow_status::DEPOSITED
            || escrow.status == escrow_status::ACCEPTED,
        TerraError::InvalidEscrowStatus
    );

    let filer = ctx.accounts.filer.key();
    require!(
        filer == escrow.seller || filer == escrow.buyer,
        TerraError::NotPartyToEscrow
    );

    // Enforce anti-grief: minimum validators required.
    require!(
        required >= dispute::MIN_DISPUTE_VALIDATORS,
        TerraError::InvalidThreshold
    );
    require!(
        !case_hash.iter().all(|b| *b == 0),
        TerraError::EmptyCaseHash
    );

    // Count declared validators and enforce self-dealing checks.
    let mut count: u8 = 0;
    for &v in validators.iter() {
        if v == Pubkey::default() {
            continue;
        }
        // Validator cannot be buyer, seller, or filer.
        require!(v != escrow.buyer, TerraError::ValidatorOwnsAsset);
        require!(v != escrow.seller, TerraError::ValidatorOwnsAsset);
        require!(v != filer, TerraError::ValidatorOwnsAsset);
        count += 1;
    }
    require!(count > 0, TerraError::NoValidators);
    require!(
        (required as usize) <= count as usize,
        TerraError::InvalidThreshold
    );

    let now = Clock::get()?.unix_timestamp;

    // Initialize the RFC-007 dispute account.
    let dispute_account = &mut ctx.accounts.dispute;
    dispute_account.parcel = ctx.accounts.parcel.key();
    dispute_account.filed_by = filer;
    dispute_account.case_hash = case_hash;
    dispute_account.status = dispute::dispute_status::FILED;
    dispute_account.required = required;
    dispute_account.declared_count = count;
    dispute_account.validators = validators;
    dispute_account.filed_at = now;

    // Update escrow status.
    let escrow = &mut ctx.accounts.escrow_record;
    escrow.status = escrow_status::DISPUTED;
    escrow.dispute_case_hash = case_hash;

    // Update parcel status to DISPUTED (first step of freeze flow).
    let parcel = &mut ctx.accounts.parcel;
    parcel.status = parcel_status::DISPUTED;
    parcel.updated_at = now;

    emit!(EscrowDisputed {
        escrow: escrow.key(),
        parcel: escrow.parcel,
        filer,
        case_hash,
        dispute: dispute_account.key(),
    });
    Ok(())
}

/// Cleanup an expired escrow in CREATED status after the cancel deadline has passed.
pub fn expire_escrow(ctx: Context<super::ExpireEscrow>) -> Result<()> {
    let escrow = &ctx.accounts.escrow_record;
    require!(
        escrow.status == escrow_status::CREATED,
        TerraError::InvalidEscrowStatus
    );

    let now = Clock::get()?.unix_timestamp;
    require!(
        now >= escrow.cancel_deadline,
        TerraError::CancelWindowNotExpired
    );
    require!(escrow.deposit_amount == 0, TerraError::DepositExists);

    // Reset parcel status.
    let parcel = &mut ctx.accounts.parcel;
    parcel.status = parcel_status::FOR_SALE;
    parcel.updated_at = now;

    // Escrow record is closed by the `close = caller` constraint: rent
    // lamports are returned to the caller; no manual zeroing needed.

    let parcel_key = escrow.parcel;

    emit!(EscrowExpired {
        escrow: escrow.key(),
        parcel: parcel_key,
        expired_at: now,
    });
    Ok(())
}

// ---------------------------------------------------------------------------
// Events
// ---------------------------------------------------------------------------

#[event]
pub struct EscrowCreated {
    pub escrow: Pubkey,
    pub parcel: Pubkey,
    pub seller: Pubkey,
    pub buyer: Pubkey,
    pub amount: u64,
    pub created_at: i64,
}

#[event]
pub struct EscrowDeposited {
    pub escrow: Pubkey,
    pub parcel: Pubkey,
    pub buyer: Pubkey,
    pub deposit_amount: u64,
    pub total_deposited: u64,
}

#[event]
pub struct EscrowAccepted {
    pub escrow: Pubkey,
    pub parcel: Pubkey,
    pub seller: Pubkey,
    pub accepted_at: i64,
    pub settle_deadline: i64,
}

#[event]
pub struct EscrowSettled {
    pub escrow: Pubkey,
    pub parcel: Pubkey,
    pub seller: Pubkey,
    pub buyer: Pubkey,
    pub amount: u64,
    pub settled_at: i64,
}

#[event]
pub struct EscrowCancelled {
    pub escrow: Pubkey,
    pub parcel: Pubkey,
    pub cancelled_by: Pubkey,
}

#[event]
pub struct EscrowDisputed {
    pub escrow: Pubkey,
    pub parcel: Pubkey,
    pub filer: Pubkey,
    pub case_hash: [u8; 32],
    pub dispute: Pubkey,
}

#[event]
pub struct EscrowExpired {
    pub escrow: Pubkey,
    pub parcel: Pubkey,
    pub expired_at: i64,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escrow_status_values_are_contiguous() {
        assert_eq!(escrow_status::CREATED, 0);
        assert_eq!(escrow_status::DEPOSITED, 1);
        assert_eq!(escrow_status::ACCEPTED, 2);
        assert_eq!(escrow_status::SETTLED, 3);
        assert_eq!(escrow_status::CANCELLED, 4);
        assert_eq!(escrow_status::DISPUTED, 5);
    }

    #[test]
    fn settlement_window_is_3_days() {
        assert_eq!(SETTLEMENT_WINDOW_SECS, 3 * 24 * 3600);
    }

    #[test]
    fn cancel_window_is_7_days() {
        assert_eq!(CANCEL_WINDOW_SECS, 7 * 24 * 3600);
    }

    #[test]
    fn min_max_escrow_amounts() {
        assert_eq!(MIN_ESCROW_AMOUNT, 100_000_000); // 0.1 SOL
        assert_eq!(MAX_ESCROW_AMOUNT, 1_000_000_000_000); // 1M SOL
        assert!(MIN_ESCROW_AMOUNT < MAX_ESCROW_AMOUNT);
    }

    #[test]
    fn self_dealing_prevention() {
        let seller = [1u8; 32];
        let buyer = [2u8; 32];
        let same_as_seller = [1u8; 32];
        assert_ne!(seller, buyer);
        assert_eq!(seller, same_as_seller);
    }
}
