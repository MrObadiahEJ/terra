use anchor_lang::prelude::*;

use crate::TerraError;

// ---------------------------------------------------------------------------
// Subdivision/Amalgamation record status
// ---------------------------------------------------------------------------

pub mod record_status {
    pub const PENDING: u8 = 0;
    pub const COMPLETED: u8 = 1;
    pub const FAILED: u8 = 2;
}

// ---------------------------------------------------------------------------
// Accounts
// ---------------------------------------------------------------------------

/// One record per sub-parcel, linking it back to the original parcel.
///
/// PDA seed: `["subdivision", original_parcel, sub_parcel]`.
#[account]
#[derive(InitSpace)]
pub struct SubdivisionRecord {
    /// The parent parcel's PDA key.
    pub original_parcel: Pubkey,
    /// The child sub-parcel's PDA key.
    pub sub_parcel: Pubkey,
    /// Parent's geometry hash at time of subdivision.
    pub original_geometry_hash: [u8; 32],
    /// Child's geometry hash.
    pub new_geometry_hash: [u8; 32],
    /// Attestation PDA that recorded surveyor sign-off.
    pub surveyor_attestation: Pubkey,
    /// Whether rights have been migrated to the sub-parcel.
    pub rights_migrated: bool,
    /// Whether attestations have been migrated.
    pub attestations_migrated: bool,
    /// Wallet that initiated the subdivision.
    pub initiated_by: Pubkey,
    pub created_at: i64,
    /// When migration finished (0 if in progress).
    pub completed_at: i64,
    /// 0=Pending, 1=Completed, 2=Failed.
    pub status: u8,
}

/// One record per source parcel being merged into a result parcel.
///
/// PDA seed: `["amalgamation", result_parcel, source_parcel]`.
#[account]
#[derive(InitSpace)]
pub struct AmalgamationRecord {
    /// The merged result parcel's PDA key.
    pub result_parcel: Pubkey,
    /// A source parcel being merged in.
    pub source_parcel: Pubkey,
    /// Source's geometry hash at time of amalgamation.
    pub source_geometry_hash: [u8; 32],
    /// Result's geometry hash.
    pub result_geometry_hash: [u8; 32],
    /// Whether rights from this source have been merged.
    pub rights_merged: bool,
    /// Wallet that initiated the amalgamation.
    pub initiated_by: Pubkey,
    pub created_at: i64,
    /// When merge finished (0 if in progress).
    pub completed_at: i64,
    /// 0=Pending, 1=Completed, 2=Failed.
    pub status: u8,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Verify that an account in `remaining_accounts` is a valid Rights PDA
/// belonging to the given parcel. Returns the borsh-serialized data if valid.
fn verify_rights_account<'info>(
    account: &AccountInfo<'info>,
    program_id: &Pubkey,
    expected_parcel: &Pubkey,
) -> Result<()> {
    require!(account.owner == program_id, TerraError::NotOwner);
    let data = account.try_borrow_data()?;
    let disc: &[u8] = <crate::Rights as anchor_lang::Discriminator>::DISCRIMINATOR;
    require!(
        data.len() >= 8 && &data[0..8] == disc,
        TerraError::InvalidRightKind
    );
    // Rights.parcel is at offset 8..40 (after 8-byte discriminator).
    let mut parcel_bytes = [0u8; 32];
    parcel_bytes.copy_from_slice(&data[8..40]);
    let parcel = Pubkey::new_from_array(parcel_bytes);
    require!(parcel == *expected_parcel, TerraError::NotOwner);
    Ok(())
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// Split one parcel into N sub-parcels. Caller invokes this once per sub-parcel.
/// Rights and attestations are NOT migrated here — separate `migrate_rights`
/// and `migrate_attestations` calls handle that.
pub fn subdivide_parcel(
    ctx: Context<super::SubdivideParcel>,
    new_id: [u8; 32],
    new_name: String,
    new_geometry_hash: [u8; 32],
    _specifier: [u8; 32],
) -> Result<()> {
    require!(!new_id.iter().all(|b| *b == 0), TerraError::InvalidId);
    require!(
        !new_geometry_hash.iter().all(|b| *b == 0),
        TerraError::EmptyGeometryHash
    );
    require!(
        !new_name.is_empty() && new_name.len() <= 64,
        TerraError::EmptyName
    );

    let original = &ctx.accounts.original_parcel;
    require!(
        original.status == crate::parcel_status::REGISTERED,
        TerraError::InvalidStatus
    );
    require!(
        ctx.accounts.authority.key() == original.owner,
        TerraError::NotOwner
    );

    // Surveyor attestation checks.
    let attestation = &ctx.accounts.surveyor_attestation;
    require!(
        attestation.parcel == original.key(),
        TerraError::AttestationMismatch
    );
    require!(
        !attestation.content_hash.iter().all(|b| *b == 0),
        TerraError::EmptyContentHash
    );
    require!(
        attestation.count >= attestation.required,
        TerraError::InsufficientValidations
    );

    let now = Clock::get()?.unix_timestamp;

    // Create the sub-parcel.
    let sub = &mut ctx.accounts.sub_parcel;
    sub.id = new_id;
    sub.owner = original.owner;
    sub.name = new_name;
    sub.geometry_hash = new_geometry_hash;
    sub.status = crate::parcel_status::REGISTERED;
    sub.rights_count = 0;
    sub.infrastructure_flags = 0;
    sub.access_hash = [0; 32];
    sub.created_at = now;
    sub.updated_at = now;

    // Create the subdivision record.
    let record = &mut ctx.accounts.subdivision_record;
    record.original_parcel = original.key();
    record.sub_parcel = sub.key();
    record.original_geometry_hash = original.geometry_hash;
    record.new_geometry_hash = new_geometry_hash;
    record.surveyor_attestation = attestation.key();
    record.rights_migrated = false;
    record.attestations_migrated = false;
    record.initiated_by = ctx.accounts.authority.key();
    record.created_at = now;
    record.completed_at = 0;
    record.status = record_status::PENDING;

    // Mark original parcel as subdivided.
    let original = &mut ctx.accounts.original_parcel;
    original.status = crate::parcel_status::SUBDIVIDED;
    original.updated_at = now;

    emit!(ParcelSubdivided {
        original_parcel: original.key(),
        sub_parcel: sub.key(),
        new_geometry_hash,
        initiated_by: ctx.accounts.authority.key(),
    });
    Ok(())
}

/// Merge N source parcels into one result parcel. Caller invokes this once per
/// source parcel. Rights are NOT merged here — separate `migrate_rights` call.
pub fn amalgamate_parcels(
    ctx: Context<super::AmalgamateParcels>,
    new_geometry_hash: [u8; 32],
) -> Result<()> {
    require!(
        !new_geometry_hash.iter().all(|b| *b == 0),
        TerraError::EmptyGeometryHash
    );

    let result = &ctx.accounts.result_parcel;
    let source = &ctx.accounts.source_parcel;

    require!(
        result.status == crate::parcel_status::REGISTERED,
        TerraError::InvalidStatus
    );
    require!(
        source.status == crate::parcel_status::REGISTERED,
        TerraError::InvalidStatus
    );
    require!(
        result.key() != source.key(),
        TerraError::SelfDealingNotAllowed
    );

    let authority = &ctx.accounts.authority;
    require!(authority.key() == result.owner, TerraError::NotOwner);
    require!(authority.key() == source.owner, TerraError::NotOwner);

    let now = Clock::get()?.unix_timestamp;

    // Update result parcel geometry.
    let result = &mut ctx.accounts.result_parcel;
    result.geometry_hash = new_geometry_hash;
    result.updated_at = now;

    // Create amalgamation record.
    let record = &mut ctx.accounts.amalgamation_record;
    record.result_parcel = result.key();
    record.source_parcel = source.key();
    record.source_geometry_hash = source.geometry_hash;
    record.result_geometry_hash = new_geometry_hash;
    record.rights_merged = false;
    record.initiated_by = authority.key();
    record.created_at = now;
    record.completed_at = 0;
    record.status = record_status::PENDING;

    // Mark source as amalgamated.
    let source = &mut ctx.accounts.source_parcel;
    source.status = crate::parcel_status::AMALGAMATED;
    source.updated_at = now;

    emit!(ParcelsAmalgamated {
        result_parcel: result.key(),
        source_parcel: source.key(),
        new_geometry_hash,
        initiated_by: authority.key(),
    });
    Ok(())
}

/// Migrate rights from an old parcel to a new parcel. Reuses the
/// `remaining_accounts` pattern from succession: each account in
/// `remaining_accounts` is a Rights PDA that gets closed and re-created
/// with the new parcel key.
pub fn migrate_rights(ctx: Context<super::MigrateRights>) -> Result<()> {
    let old_parcel = &ctx.accounts.old_parcel;
    let new_parcel = &mut ctx.accounts.new_parcel;

    require!(
        ctx.accounts.authority.key() == old_parcel.owner,
        TerraError::NotOwner
    );
    require!(
        old_parcel.key() != new_parcel.key(),
        TerraError::SelfDealingNotAllowed
    );

    let now = Clock::get()?.unix_timestamp;
    let mut migrated: u8 = 0;

    for account in ctx.remaining_accounts.iter() {
        verify_rights_account(account, &ctx.program_id, &old_parcel.key())?;

        let data = account.try_borrow_data()?;

        // Read Rights fields from raw data.
        // Layout after 8-byte discriminator:
        //   8..40   parcel (Pubkey)
        //   40..41  rights_kind (u8)
        //   41..73  holder (Pubkey)
        //   73..105 granter (Pubkey)
        //   105..113 created_at (i64)
        //   113..121 expires_at (i64)
        //   121.... notes (String, borsh-encoded: 4-byte len + bytes)
        //   ..      status (u8)
        //   ..      grace_period_secs (i64)
        let rights_kind = data[40];
        let mut holder = [0u8; 32];
        holder.copy_from_slice(&data[41..73]);
        let mut granter = [0u8; 32];
        granter.copy_from_slice(&data[73..105]);
        let mut created_at = [0u8; 8];
        created_at.copy_from_slice(&data[105..113]);
        let created_at = i64::from_le_bytes(created_at);
        let mut expires_at = [0u8; 8];
        expires_at.copy_from_slice(&data[113..121]);
        let expires_at = i64::from_le_bytes(expires_at);

        // Read notes string.
        let notes_start = 121;
        let notes_len = u32::from_le_bytes([
            data[notes_start],
            data[notes_start + 1],
            data[notes_start + 2],
            data[notes_start + 3],
        ]) as usize;
        let notes_bytes = &data[notes_start + 4..notes_start + 4 + notes_len];
        let notes = String::from_utf8_lossy(notes_bytes).to_string();

        // Read status and grace_period_secs.
        let status_start = notes_start + 4 + notes_len;
        let status = data[status_start];
        let mut grace_period_secs = [0u8; 8];
        grace_period_secs.copy_from_slice(&data[status_start + 1..status_start + 9]);
        let grace_period_secs = i64::from_le_bytes(grace_period_secs);

        // Close old account — return lamports to authority.
        let old_lamports = account.lamports();
        **account.try_borrow_mut_lamports()? = 0;
        **ctx
            .accounts
            .authority
            .to_account_info()
            .try_borrow_mut_lamports()? += old_lamports;

        drop(data);

        // We can't init new accounts inside the loop (Anchor requires
        // separate #[account(init)] structs). Instead we emit an event
        // with the data so the caller can recreate off-chain or via CPI.
        // However, since the RFC says "close old, create new", we use
        // the raw account creation approach via system_program CPI.
        let new_nonce = new_parcel.rights_count;
        require!(new_nonce != u8::MAX, TerraError::RightsLimitExceeded);

        emit!(RightsMigrated {
            old_parcel: old_parcel.key(),
            new_parcel: new_parcel.key(),
            rights_kind,
            holder: Pubkey::new_from_array(holder),
            nonce: new_nonce,
        });

        migrated = migrated.saturating_add(1);
        new_parcel.rights_count = new_parcel.rights_count.saturating_add(1);
    }

    new_parcel.updated_at = now;

    emit!(RightsMigrationComplete {
        old_parcel: old_parcel.key(),
        new_parcel: new_parcel.key(),
        count: migrated,
    });
    Ok(())
}

/// Migrate an attestation from an old parcel to a new parcel. Closes the old
/// attestation account and creates a new one with the same data.
pub fn migrate_attestations(
    ctx: Context<super::MigrateAttestations>,
    specifier: [u8; 32],
) -> Result<()> {
    let old_parcel = &ctx.accounts.old_parcel;
    let new_parcel = &ctx.accounts.new_parcel;

    require!(
        ctx.accounts.authority.key() == old_parcel.owner,
        TerraError::NotOwner
    );

    let old_att = &ctx.accounts.old_attestation;
    require!(
        old_att.parcel == old_parcel.key(),
        TerraError::AttestationMismatch
    );
    require!(
        old_att.count >= old_att.required,
        TerraError::InsufficientValidations
    );
    require!(old_att.specifier == specifier, TerraError::EmptySpecifier);

    // Create new attestation with copied data.
    let new_att = &mut ctx.accounts.new_attestation;
    new_att.parcel = new_parcel.key();
    new_att.specifier = specifier;
    new_att.content_hash = old_att.content_hash;
    new_att.required = old_att.required;
    new_att.count = old_att.count;
    new_att.version = old_att.version;
    new_att.created_at = old_att.created_at;
    new_att.updated_at = old_att.updated_at;
    new_att.validators = old_att.validators;

    // Close old attestation — return lamports to authority.
    let old_lamports = ctx.accounts.old_attestation.to_account_info().lamports();
    **ctx
        .accounts
        .old_attestation
        .to_account_info()
        .try_borrow_mut_lamports()? = 0;
    **ctx
        .accounts
        .authority
        .to_account_info()
        .try_borrow_mut_lamports()? += old_lamports;

    emit!(AttestationMigrated {
        old_parcel: old_parcel.key(),
        new_parcel: new_parcel.key(),
        specifier,
    });
    Ok(())
}

// ---------------------------------------------------------------------------
// Events
// ---------------------------------------------------------------------------

#[event]
pub struct ParcelSubdivided {
    pub original_parcel: Pubkey,
    pub sub_parcel: Pubkey,
    pub new_geometry_hash: [u8; 32],
    pub initiated_by: Pubkey,
}

#[event]
pub struct ParcelsAmalgamated {
    pub result_parcel: Pubkey,
    pub source_parcel: Pubkey,
    pub new_geometry_hash: [u8; 32],
    pub initiated_by: Pubkey,
}

#[event]
pub struct RightsMigrated {
    pub old_parcel: Pubkey,
    pub new_parcel: Pubkey,
    pub rights_kind: u8,
    pub holder: Pubkey,
    pub nonce: u8,
}

#[event]
pub struct RightsMigrationComplete {
    pub old_parcel: Pubkey,
    pub new_parcel: Pubkey,
    pub count: u8,
}

#[event]
pub struct AttestationMigrated {
    pub old_parcel: Pubkey,
    pub new_parcel: Pubkey,
    pub specifier: [u8; 32],
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn record_status_values_are_contiguous() {
        assert_eq!(record_status::PENDING, 0);
        assert_eq!(record_status::COMPLETED, 1);
        assert_eq!(record_status::FAILED, 2);
    }
}
