use anchor_lang::prelude::*;

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[error_code]
pub enum IdentityError {
    #[msg("Identity hash is required")]
    EmptyIdentityHash,
    #[msg("Recovery wallet is required")]
    EmptyRecovery,
    #[msg("Successor wallet is required")]
    EmptySuccessor,
    #[msg("Invalid succession kind")]
    InvalidSuccessionKind,
    #[msg("Successor must differ from the current owner")]
    SuccessorIsOwner,
    #[msg("Not authorized to perform this action")]
    NotAuthorized,
    #[msg("Signing wallet is not a declared validator for this succession")]
    NotValidator,
    #[msg("A validator cannot be the owner of the asset being validated")]
    ValidatorOwnsAsset,
    #[msg("Succession has already become effective")]
    SuccessionAlreadyEffective,
    #[msg("No more validators may endorse this succession (limit reached)")]
    ValidationLimitReached,
    #[msg("Only the named successor may claim this succession")]
    NotSuccessor,
    #[msg("Succession is not yet effective")]
    SuccessionNotYetEffective,
    #[msg("Succession requires validator endorsements before it can be claimed")]
    InsufficientValidations,
    #[msg("Attestation does not belong to this parcel")]
    AttestationMismatch,
    #[msg("Identity owner does not match the parcel owner")]
    IdentityMismatch,
    #[msg("Only the current owner can perform this action")]
    NotOwner,
    #[msg("Required threshold exceeds the number of validators")]
    InvalidThreshold,
    #[msg("Court case hash is required")]
    EmptyCaseHash,
    #[msg("Notes exceed the maximum length of 128")]
    NotesTooLong,
    #[msg("Guardianship grace period is below the 90-day minimum")]
    GuardianshipGraceTooShort,
    #[msg("Guardianship requires at least 3 validator endorsements")]
    GuardianshipThresholdTooLow,
    #[msg("No proposal found")]
    NoProposalFound,
    #[msg("A revocation request is already pending for this identity")]
    GuardianshipAlreadyActive,
    #[msg("Settlement not yet effective")]
    SettlementNotYetEffective,
    #[msg("Parcel account data is too short for deserialization")]
    ParcelDataTooShort,
    #[msg("Failed to deserialize parcel account data")]
    ParcelDeserializeFailed,
    #[msg("Parcel owner does not match the identity owner")]
    ParcelOwnerMismatch,
}
