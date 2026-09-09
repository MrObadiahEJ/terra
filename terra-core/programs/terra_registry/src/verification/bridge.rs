use anchor_lang::prelude::*;

use crate::TerraError;

/// Bridge instruction to migrate a legacy `Attestation` into a `Claim` in the
/// verification pipeline. The old Attestation account is left intact (not closed)
/// for backward compatibility. The new Claim gets status SUBMITTED and can
/// proceed through the normal verification flow.
///
/// Mapping:
/// - `Attestation.parcel` → `Claim.parcel`
/// - `Attestation.content_hash` → `Claim.statement_hash`
/// - `Attestation.required` → `Claim.required_attestations`
///
/// The caller must derive `claim_id` as `SHA-256(attestation.specifier)` and
/// pass it as an instruction argument so Anchor can derive the PDA.
pub fn migrate_attestation_to_claim(
    ctx: Context<crate::MigrateAttestationToClaim>,
    claim_id: [u8; 32],
    claim_type: u8,
) -> Result<()> {
    let attestation = &ctx.accounts.attestation;
    require!(
        attestation.parcel == ctx.accounts.parcel.key(),
        TerraError::AttestationMismatch
    );
    require!(
        claim_type <= crate::verification::claim::claim_type::MAX,
        TerraError::InvalidClaimType
    );
    require!(
        !claim_id.iter().all(|b| *b == 0),
        TerraError::EmptyClaimId
    );

    let now = Clock::get()?.unix_timestamp;
    let claim = &mut ctx.accounts.claim;
    claim.claim_id = claim_id;
    claim.parcel = attestation.parcel;
    claim.claim_type = claim_type;
    claim.submitted_by = ctx.accounts.authority.key();
    claim.status = crate::verification::claim::claim_status::SUBMITTED;
    claim.statement_hash = attestation.content_hash;
    claim.version = 1;
    claim.evidence_count = 0;
    claim.observation_count = 0;
    claim.attestation_count = 0;
    claim.required_attestations = attestation.required;
    claim.created_at = now;
    claim.updated_at = now;

    emit!(crate::ClaimCreated {
        claim: claim.key(),
        parcel: claim.parcel,
        claim_id,
        claim_type,
        submitted_by: claim.submitted_by,
        statement_hash: claim.statement_hash,
    });
    Ok(())
}
