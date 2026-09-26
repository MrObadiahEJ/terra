use anchor_lang::prelude::*;

use crate::observation_v2;
use crate::TerraError;

// ---------------------------------------------------------------------------
// RFC-012 Phase 9 — physical infrastructure (device identities)
//
// Entity (RFC-012 §5): DeviceIdentity — device cryptographic key,
// capabilities, calibration, ownership.
//
// Design rules honored here:
//   - Design Rule 7 (Subject ≠ Capture Device ≠ Submitter): the device is its
//     own account, distinct from the observing wallet. Observations
//     (Phase 4 `ObservationV2.capture_device`) may point at a registered
//     device key; evidence artifacts (Phase 5) reuse the same source codes.
//   - Reuses Phase 4 `observation_source` codes — a device registers under
//     the capture class it produces (PHONE/GNSS/CAMERA/DRONE/…).
//   - Registration is permissionless (owner pays rent, device starts
//     UNVERIFIED); a validator with a `ValidatorProfile` flips `verified`
//     — same claim → verify pattern as RFC-006 jurisdiction bindings.
//   - REVOKED is terminal; SUSPEND/RESUME are owner-controlled.
//
// Phase 9 ships the on-chain identity registry only. Device-hardware
// attestation (Android Key Attestation / secure-enclave certificates bound to
// `device_key`) and observation-envelope signature verification are off-chain
// protocols over these accounts; on-chain checks the keys, status, and
// validator verification.
// ---------------------------------------------------------------------------

/// Maximum metadata reference length (model/firmware URI or IPFS CID).
pub const MAX_DEVICE_METADATA_LEN: usize = 128;

/// Device lifecycle status.
pub mod device_status {
    /// Registered and usable (default).
    pub const ACTIVE: u8 = 0;
    /// Temporarily disabled by the owner (resumable).
    pub const SUSPENDED: u8 = 1;
    /// Permanently retired — terminal state.
    pub const REVOKED: u8 = 2;
    pub const MAX: u8 = REVOKED;
}

/// Declared capture capabilities (informational bitfield, RFC-012 §5).
pub mod device_capability {
    pub const GNSS: u64 = 1 << 0;
    pub const PHOTO: u64 = 1 << 1;
    pub const VIDEO: u64 = 1 << 2;
    pub const LIDAR: u64 = 1 << 3;
    pub const IMU: u64 = 1 << 4;
    pub const RTK: u64 = 1 << 5;
    pub const SCANNER_3D: u64 = 1 << 6;
    pub const THERMAL: u64 = 1 << 7;
}

// ---------------------------------------------------------------------------
// Accounts (PDA seeds per RFC-012 §10)
// ---------------------------------------------------------------------------

/// A registered capture device (phone, GNSS receiver, drone, survey
/// instrument, 3D scanner, …) owned by a wallet.
///
/// PDA: `["device_identity", owner, device_nonce]`
#[account]
#[derive(InitSpace)]
pub struct DeviceIdentity {
    /// Wallet that registered (and controls) the device.
    pub owner: Pubkey,
    /// PDA seed companion — one device per (owner, nonce).
    pub device_nonce: u16,
    /// The device's Ed25519 public key (signs observation envelopes).
    pub device_key: Pubkey,
    /// Phase 4 `observation_source` code the device captures as.
    pub source: u8,
    /// Declared capability bitfield (`device_capability::*`).
    pub capabilities: u64,
    /// Model/firmware metadata reference (IPFS CID or URL).
    #[max_len(MAX_DEVICE_METADATA_LEN)]
    pub metadata_ref: String,
    /// SHA-256 of the latest calibration certificate (all-zero = never).
    pub calibration_hash: [u8; 32],
    /// When the calibration certificate was issued (0 = never).
    pub calibrated_at: i64,
    /// `device_status` — ACTIVE / SUSPENDED / REVOKED.
    pub status: u8,
    /// A validator has verified this registration (claim ≠ fact until then).
    pub verified: bool,
    /// Validator wallet that last verified.
    pub verified_by: Pubkey,
    /// Monotonic counter, bumped on each verification.
    pub verify_version: u32,
    pub registered_at: i64,
    pub updated_at: i64,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

pub fn is_valid_status(status: u8) -> bool {
    status <= device_status::MAX
}

/// Status transitions: REVOKED is terminal; any other pair is allowed
/// (including no-ops, which only bump `updated_at`).
pub fn can_transition(from: u8, to: u8) -> bool {
    is_valid_status(to) && (from != device_status::REVOKED || to == device_status::REVOKED)
}

pub fn is_valid_metadata(metadata_ref: &str) -> bool {
    metadata_ref.len() <= MAX_DEVICE_METADATA_LEN
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// Register a capture device (permissionless; owner pays rent).
pub fn register_device(
    ctx: Context<crate::RegisterDevice>,
    device_nonce: u16,
    device_key: Pubkey,
    source: u8,
    capabilities: u64,
    metadata_ref: String,
) -> Result<()> {
    require!(
        device_key != Pubkey::default(),
        TerraError::InvalidDeviceKey
    );
    require!(
        observation_v2::is_valid_observation_source(source),
        TerraError::InvalidObservationSource
    );
    require!(
        is_valid_metadata(&metadata_ref),
        TerraError::DeviceMetadataTooLong
    );

    let now = Clock::get()?.unix_timestamp;
    let device = &mut ctx.accounts.device;
    device.owner = ctx.accounts.owner.key();
    device.device_nonce = device_nonce;
    device.device_key = device_key;
    device.source = source;
    device.capabilities = capabilities;
    device.metadata_ref = metadata_ref;
    device.calibration_hash = [0u8; 32];
    device.calibrated_at = 0;
    device.status = device_status::ACTIVE;
    device.verified = false;
    device.verified_by = Pubkey::default();
    device.verify_version = 0;
    device.registered_at = now;
    device.updated_at = now;

    emit!(crate::DeviceRegistered {
        device: device.key(),
        owner: device.owner,
        device_key,
        source,
        registered_at: now,
    });
    Ok(())
}

/// Update declared capabilities and metadata reference (owner only).
pub fn update_device(
    ctx: Context<crate::UpdateDevice>,
    capabilities: u64,
    metadata_ref: String,
) -> Result<()> {
    require!(
        is_valid_metadata(&metadata_ref),
        TerraError::DeviceMetadataTooLong
    );
    let device = &mut ctx.accounts.device;
    require!(
        device.status != device_status::REVOKED,
        TerraError::DeviceRevoked
    );

    device.capabilities = capabilities;
    device.metadata_ref = metadata_ref;
    device.updated_at = Clock::get()?.unix_timestamp;

    emit!(crate::DeviceUpdated {
        device: device.key(),
        owner: device.owner,
        capabilities: device.capabilities,
        updated_at: device.updated_at,
    });
    Ok(())
}

/// Rotate the device's signing key (owner only — e.g. re-provisioning).
pub fn rotate_device_key(
    ctx: Context<crate::RotateDeviceKey>,
    new_device_key: Pubkey,
) -> Result<()> {
    require!(
        new_device_key != Pubkey::default(),
        TerraError::InvalidDeviceKey
    );
    let device = &mut ctx.accounts.device;
    require!(
        device.status != device_status::REVOKED,
        TerraError::DeviceRevoked
    );

    let old_device_key = device.device_key;
    device.device_key = new_device_key;
    device.updated_at = Clock::get()?.unix_timestamp;

    emit!(crate::DeviceKeyRotated {
        device: device.key(),
        owner: device.owner,
        old_device_key,
        new_device_key,
        rotated_at: device.updated_at,
    });
    Ok(())
}

/// Suspend / resume / revoke the device (owner only; REVOKED is terminal).
pub fn set_device_status(ctx: Context<crate::SetDeviceStatus>, status: u8) -> Result<()> {
    require!(is_valid_status(status), TerraError::InvalidDeviceStatus);
    let device = &mut ctx.accounts.device;
    require!(
        can_transition(device.status, status),
        TerraError::DeviceRevoked
    );

    let old_status = device.status;
    device.status = status;
    device.updated_at = Clock::get()?.unix_timestamp;

    emit!(crate::DeviceStatusChanged {
        device: device.key(),
        owner: device.owner,
        old_status,
        new_status: status,
        changed_at: device.updated_at,
    });
    Ok(())
}

/// Record a calibration certificate (owner only).
pub fn set_device_calibration(
    ctx: Context<crate::SetDeviceCalibration>,
    calibration_hash: [u8; 32],
    calibrated_at: i64,
) -> Result<()> {
    require!(calibration_hash != [0u8; 32], TerraError::EmptyContentHash);
    let device = &mut ctx.accounts.device;
    require!(
        device.status != device_status::REVOKED,
        TerraError::DeviceRevoked
    );

    device.calibration_hash = calibration_hash;
    device.calibrated_at = calibrated_at;
    device.updated_at = Clock::get()?.unix_timestamp;

    emit!(crate::DeviceCalibrationSet {
        device: device.key(),
        owner: device.owner,
        calibration_hash,
        calibrated_at,
    });
    Ok(())
}

/// A registered validator verifies the device registration (claim → fact).
pub fn verify_device(ctx: Context<crate::VerifyDevice>) -> Result<()> {
    let device = &mut ctx.accounts.device;
    require!(
        device.status != device_status::REVOKED,
        TerraError::DeviceRevoked
    );

    device.verified = true;
    device.verified_by = ctx.accounts.validator.key();
    device.verify_version = device.verify_version.saturating_add(1);
    device.updated_at = Clock::get()?.unix_timestamp;

    emit!(crate::DeviceVerified {
        device: device.key(),
        verified_by: device.verified_by,
        verify_version: device.verify_version,
        verified_at: device.updated_at,
    });
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests (pure helpers — handlers are exercised in BPF `phase9_*` tests)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_values_distinct_from_zero_marker() {
        // ACTIVE is the zero default (init memory) — the other states must not be.
        assert_eq!(device_status::ACTIVE, 0);
        assert_ne!(device_status::SUSPENDED, 0);
        assert_ne!(device_status::REVOKED, 0);
    }

    #[test]
    fn status_validation_matrix() {
        assert!(is_valid_status(device_status::ACTIVE));
        assert!(is_valid_status(device_status::SUSPENDED));
        assert!(is_valid_status(device_status::REVOKED));
        assert!(!is_valid_status(device_status::REVOKED + 1));
        assert!(!is_valid_status(u8::MAX));
    }

    #[test]
    fn transition_matrix_revoked_is_terminal() {
        // Normal lifecycle.
        assert!(can_transition(
            device_status::ACTIVE,
            device_status::SUSPENDED
        ));
        assert!(can_transition(
            device_status::SUSPENDED,
            device_status::ACTIVE
        ));
        assert!(can_transition(
            device_status::ACTIVE,
            device_status::REVOKED
        ));
        assert!(can_transition(
            device_status::SUSPENDED,
            device_status::REVOKED
        ));
        // Terminal: nothing leaves REVOKED (self-transition only).
        assert!(can_transition(
            device_status::REVOKED,
            device_status::REVOKED
        ));
        assert!(!can_transition(
            device_status::REVOKED,
            device_status::ACTIVE
        ));
        assert!(!can_transition(
            device_status::REVOKED,
            device_status::SUSPENDED
        ));
        // Invalid target values always fail.
        assert!(!can_transition(
            device_status::ACTIVE,
            device_status::MAX + 1
        ));
    }

    #[test]
    fn metadata_length_boundary() {
        assert!(is_valid_metadata(""));
        assert!(is_valid_metadata(&"a".repeat(MAX_DEVICE_METADATA_LEN)));
        assert!(!is_valid_metadata(&"a".repeat(MAX_DEVICE_METADATA_LEN + 1)));
        // Length is bytes, not chars — multibyte input can only overflow sooner.
        let multibyte = "é".repeat(MAX_DEVICE_METADATA_LEN); // 2 bytes each
        assert!(!is_valid_metadata(&multibyte));
    }

    #[test]
    fn source_reuses_phase4_validation() {
        use crate::observation_v2::observation_source;
        for src in [
            observation_source::PHONE,
            observation_source::GNSS,
            observation_source::CAMERA,
            observation_source::DRONE,
            observation_source::SATELLITE,
            observation_source::HUMAN,
            observation_source::DOCUMENT,
            observation_source::API,
        ] {
            assert!(
                observation_v2::is_valid_observation_source(src),
                "src {src}"
            );
        }
        assert!(!observation_v2::is_valid_observation_source(
            observation_source::MAX + 1
        ));
    }

    #[test]
    fn capability_bits_are_distinct() {
        let bits = [
            device_capability::GNSS,
            device_capability::PHOTO,
            device_capability::VIDEO,
            device_capability::LIDAR,
            device_capability::IMU,
            device_capability::RTK,
            device_capability::SCANNER_3D,
            device_capability::THERMAL,
        ];
        for (i, a) in bits.iter().enumerate() {
            assert_ne!(*a, 0, "bit {i} must be non-zero");
            for b in bits.iter().skip(i + 1) {
                assert_ne!(*a, *b, "bits must be distinct");
            }
        }
    }
}
