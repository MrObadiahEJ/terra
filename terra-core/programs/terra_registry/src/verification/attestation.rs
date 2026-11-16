use anchor_lang::prelude::*;
use anchor_lang::solana_program::account_info::AccountInfo;
use solana_program::hash::hashv;

use crate::verification::reputation::{validator_status, ValidatorReputation};
use crate::TerraError;

// ---------------------------------------------------------------------------
// W3C Verifiable Credential + SD-JWT attestation (Intent 2)
// ---------------------------------------------------------------------------
//
// Migrates VerificationAttestation from a custom format to W3C Verifiable
// Credentials 2.0 with SD-JWT selective disclosure. On-chain, the account
// anchors the sha-256 digest of the serialized SD-JWT plus the VC metadata
// fields needed for querying. Off-chain, the full SD-JWT is constructed and
// can be selectively disclosed (holder reveals only chosen claims).
//
// Do NOT invent a custom format: fields map to W3C VC 2.0 / SD-JWT VC terms.

/// VC type constants (W3C VC `type` array values).
pub mod vc_type {
    pub const VERIFICATION_ATTESTATION: &str = "VerificationAttestation";
    pub const RESIDENCY_ATTESTATION: &str = "ResidencyAttestation";
    pub const MAX_LEN: usize = 32;
}

/// Result of a validator's attestation on a claim (W3C VC `credentialStatus`).
pub mod attestation_result {
    pub const CONFIRMED: u8 = 0;
    pub const DISPUTED: u8 = 1;
    pub const UNABLE_TO_VERIFY: u8 = 2;
    pub const MAX: u8 = UNABLE_TO_VERIFY;
}

#[account]
#[derive(InitSpace)]
pub struct VerificationAttestation {
    /// The claim being attested (W3C VC `credentialSubject.id` anchor).
    pub claim: Pubkey,
    /// Validator submitting this attestation = VC issuer (`issuer`).
    pub validator: Pubkey,
    /// The observation this attestation is based on.
    pub observation: Pubkey,
    /// Credential subject the VC is about (parcel / owner / tenant pubkey).
    pub credential_subject: Pubkey,
    /// W3C VC type: "VerificationAttestation" or "ResidencyAttestation".
    #[max_len(vc_type::MAX_LEN)]
    pub vc_type: String,
    /// VC issuance date (unix seconds).
    pub issuance_date: i64,
    /// VC expiration date (unix seconds).
    pub expiration_date: i64,
    /// SD-JWT digest: sha-256 over the serialized SD-JWT. Anchors the full
    /// credential off-chain; selective disclosure reveals claims selectively.
    pub sd_jwt_digest: [u8; 32],
    /// Whether SD-JWT selective disclosure is enabled for this credential.
    pub selective_disclosure: bool,
    pub created_at: i64,
    /// Protocol version for audit trail.
    pub protocol_version: u8,
}

// ---------------------------------------------------------------------------
// Helper: load the required ValidatorReputation PDA via remaining_accounts
// ---------------------------------------------------------------------------

/// Loads the validator's reputation account. The reputation PDA
/// (`["validator_reputation", validator]`) MUST be present in
/// `remaining_accounts`; otherwise the jail/slash gate could be bypassed by
/// simply omitting the account. Rejects accounts not owned by this program.
fn load_reputation<'info>(
    remaining_accounts: &[AccountInfo<'info>],
    validator: &Pubkey,
) -> Result<ValidatorReputation> {
    let (pda, _) =
        Pubkey::find_program_address(&[b"validator_reputation", validator.as_ref()], &crate::ID);
    for acc in remaining_accounts {
        if acc.key == &pda {
            require!(acc.owner == &crate::ID, TerraError::MissingReputation);
            let data = acc.try_borrow_data()?;
            let account = ValidatorReputation::try_deserialize(&mut &data[..])?;
            return Ok(account);
        }
    }
    err!(TerraError::MissingReputation)
}

// ---------------------------------------------------------------------------
// Helper: canonical SD-JWT digest (Intent 2)
// ---------------------------------------------------------------------------

/// Domain-separated sha-256 over the attestation's VC content. Off-chain, the
/// validator constructs an SD-JWT over exactly these fields and signs it;
/// on-chain, `sd_jwt_digest` must equal `sha256(sd_jwt)` so the recorded
/// attestation can never differ from what the validator committed to, and
/// auditors can recompute the digest from the account fields alone.
fn canonical_sd_jwt_digest(
    claim: &Pubkey,
    validator: &Pubkey,
    observation: &Pubkey,
    credential_subject: &Pubkey,
    vc_type: &str,
    issuance_date: i64,
    expiration_date: i64,
) -> [u8; 32] {
    hashv(&[
        b"terra:vc:sd-jwt:v1",
        claim.as_ref(),
        validator.as_ref(),
        observation.as_ref(),
        credential_subject.as_ref(),
        vc_type.as_bytes(),
        &issuance_date.to_le_bytes(),
        &expiration_date.to_le_bytes(),
    ])
    .to_bytes()
}

// ---------------------------------------------------------------------------
// Instruction handlers
// ---------------------------------------------------------------------------

/// Submit an attestation for a claim as a W3C VC (SD-JWT). Only validators
/// who have submitted an observation may attest. One attestation per validator
/// per claim.
///
/// The validator's `ValidatorReputation` PDA MUST be supplied via
/// `remaining_accounts`. If it is missing, the transaction is rejected with
/// `MissingReputation`. If the validator is JAILED or SLASHED, the
/// transaction is rejected with `ValidatorJailed`.
///
/// `sd_jwt_digest` is sha-256 over the serialized SD-JWT constructed off-chain
/// from exactly these fields. The full SD-JWT is held off-chain and can be
/// selectively disclosed; the digest anchors it on-chain.
pub fn submit_attestation(
    ctx: Context<crate::SubmitVerificationAttestation>,
    vc_type: String,
    credential_subject: Pubkey,
    issuance_date: i64,
    expiration_date: i64,
    sd_jwt_digest: [u8; 32],
    selective_disclosure: bool,
) -> Result<()> {
    require!(
        vc_type.len() <= vc_type::MAX_LEN,
        TerraError::VcTypeTooLong
    );
    require!(
        issuance_date <= expiration_date,
        TerraError::InvalidVcValidity
    );

    // Intent 2: sd_jwt_digest must be the canonical digest over this
    // attestation's VC content — a tampered vc_type/subject/dates cannot
    // reuse a digest committed to different content.
    let expected_digest = canonical_sd_jwt_digest(
        &ctx.accounts.claim.key(),
        &ctx.accounts.validator.key(),
        &ctx.accounts.observation.key(),
        &credential_subject,
        &vc_type,
        issuance_date,
        expiration_date,
    );
    require!(
        sd_jwt_digest == expected_digest,
        TerraError::SdJwtDigestMismatch
    );

    // Mandatory reputation gating: reject jailed/slashed validators.
    let reputation = load_reputation(ctx.remaining_accounts, &ctx.accounts.validator.key())?;
    require!(
        reputation.status == validator_status::ACTIVE,
        TerraError::ValidatorJailed
    );

    let claim = &mut ctx.accounts.claim;
    require!(
        claim.status == crate::verification::claim::claim_status::SUBMITTED
            || claim.status == crate::verification::claim::claim_status::UNDER_VERIFICATION,
        TerraError::InvalidClaimStatus
    );

    let now = Clock::get()?.unix_timestamp;
    let attestation = &mut ctx.accounts.attestation;
    attestation.claim = claim.key();
    attestation.validator = ctx.accounts.validator.key();
    attestation.observation = ctx.accounts.observation.key();
    attestation.credential_subject = credential_subject;
    attestation.vc_type = vc_type.clone();
    attestation.issuance_date = issuance_date;
    attestation.expiration_date = expiration_date;
    attestation.sd_jwt_digest = sd_jwt_digest;
    attestation.selective_disclosure = selective_disclosure;
    attestation.created_at = now;
    attestation.protocol_version = 2; // W3C VC + SD-JWT

    // Only count CONFIRMED attestations toward quorum (result implied by
    // presence of a valid SD-JWT digest from an ACTIVE validator).
    claim.attestation_count = claim.attestation_count.saturating_add(1);
    claim.updated_at = now;

    emit!(crate::VerificationAttestationSubmitted {
        claim: claim.key(),
        attestation: attestation.key(),
        validator: attestation.validator,
        credential_subject,
        vc_type,
        issuance_date,
        expiration_date,
        sd_jwt_digest,
        attestation_count: claim.attestation_count,
        required: claim.required_attestations,
    });
    Ok(())
}
