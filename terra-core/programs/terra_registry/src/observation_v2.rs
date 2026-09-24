use anchor_lang::prelude::*;

use crate::verification_task::{task_is_terminal, VerificationTask};
use crate::TerraError;

// ---------------------------------------------------------------------------
// RFC-012 Phase 4 — multi-source observations (ObservationV2)
//
// Goal: multi-source observations (phone, GNSS, drone, satellite, document,
// human) with Subject ≠ Capture Device ≠ Submitter (Design Rule 7).
// PDA seeds per RFC-012 §10: `["observation_v2", task_id, observer, nonce]`.
// Migration map (§9): legacy claim-bound `Observation` remains; new
// `ObservationV2` attaches to `VerificationTask` and records full role
// separation + provenance. EvidenceManifest/EvidenceArtifact = Phase 5.
// ---------------------------------------------------------------------------

/// Max valid source code.
pub const MAX_OBSERVATION_SOURCE: u8 = 7;
/// Max valid provenance code.
pub const MAX_OBSERVATION_PROVENANCE: u8 = 5;

pub mod observation_source {
    /// Phone camera / handheld capture.
    pub const PHONE: u8 = 0;
    /// GNSS receiver fix.
    pub const GNSS: u8 = 1;
    /// Onboard / handheld camera (still image or frame).
    pub const CAMERA: u8 = 2;
    /// Drone / UAV aerial capture.
    pub const DRONE: u8 = 3;
    /// Satellite imagery.
    pub const SATELLITE: u8 = 4;
    /// Human eyewitness / field note.
    pub const HUMAN: u8 = 5;
    /// Document scan / OCR source.
    pub const DOCUMENT: u8 = 6;
    /// External API / oracle feed.
    pub const API: u8 = 7;
    pub const MAX: u8 = API;
}

pub mod observation_provenance {
    /// Submitter self-report; no independent device check.
    pub const SELF_REPORTED: u8 = 0;
    /// Device-signed GNSS fix attached to capture.
    pub const DEVICE_GNSS: u8 = 1;
    /// Two or more independent devices agree.
    pub const MULTI_DEVICE: u8 = 2;
    /// Local validator corroboration.
    pub const LOCAL_VALIDATORS: u8 = 3;
    /// Survey-grade instrument / certified process.
    pub const SURVEY_GRADE: u8 = 4;
    /// Confirmed against satellite imagery.
    pub const SATELLITE_CONFIRMED: u8 = 5;
    pub const MAX: u8 = SATELLITE_CONFIRMED;
}

// ---------------------------------------------------------------------------
// Account (PDA seeds per RFC-012 §10)
// ---------------------------------------------------------------------------

/// Multi-source observation of a task subject.
///
/// PDA: `["observation_v2", task_id, observer, nonce]`
///
/// Roles (Design Rule 7): `subject` ≠ `capture_device` ≠ `observer`
/// (submitter). A document owned by Alice (`subject`), photographed on
/// Bob's phone (`capture_device`), submitted by Charlie (`observer`).
#[account]
#[derive(InitSpace)]
pub struct ObservationV2 {
    /// Task this observation supports.
    pub task_id: [u8; 32],
    /// Submitting wallet (PDA seed). May differ from `capture_device`.
    pub observer: Pubkey,
    /// Unique nonce per (task_id, observer). PDA seed.
    pub nonce: u16,
    /// What is observed — claim, parcel, identity, or other account key.
    pub subject: Pubkey,
    /// Device that captured the raw signal / image (may equal observer).
    pub capture_device: Pubkey,
    /// One of `observation_source`.
    pub source: u8,
    /// One of `observation_provenance`.
    pub provenance: u8,
    /// Lat/lon * 1e7; `[0,0]` = no location (document / remote).
    pub location: [i64; 2],
    /// When the phenomenon was observed (device clock / capture time).
    pub observed_at: i64,
    /// sha256 of findings / structured observation payload.
    pub findings_hash: [u8; 32],
    /// sha256 of evidence pre-image (manifest root; Phase 5 expands).
    pub evidence_hash: [u8; 32],
    /// Confidence 0–100.
    pub confidence: u8,
    /// sha256 of signed observation envelope (device or submitter signature).
    pub signature_hash: [u8; 32],
    pub created_at: i64,
}

// ---------------------------------------------------------------------------
// Pure helpers (unit-testable)
// ---------------------------------------------------------------------------

pub fn is_valid_observation_source(source: u8) -> bool {
    source <= observation_source::MAX
}

pub fn is_valid_observation_provenance(provenance: u8) -> bool {
    provenance <= observation_provenance::MAX
}

pub fn is_valid_observation_confidence(confidence: u8) -> bool {
    confidence <= 100
}

/// Geographic bounds (same as presence): |lat|<=90e7, |lon|<=180e7.
/// `[0,0]` always valid (no location).
pub fn is_valid_observation_location(loc: [i64; 2]) -> bool {
    if loc == [0, 0] {
        return true;
    }
    loc[0].abs() <= 900_000_000 && loc[1].abs() <= 1_800_000_000
}

/// Observations allowed only while task is not terminal (open / assigned /
/// in_progress / completed short of… actually completed is terminal too —
/// same gate as results: not cancelled/expired/completed).
pub fn can_observe(status: u8) -> bool {
    !task_is_terminal(status)
}

// ---------------------------------------------------------------------------
// Instruction handlers
// ---------------------------------------------------------------------------

/// Submit a multi-source observation for a verification task.
/// Permissionless (any wallet may observe); submitter pays rent.
/// Requires task not terminal; validates source / provenance / confidence /
/// location. Subject / capture_device / observer are independent roles.
#[allow(clippy::too_many_arguments)]
pub fn submit_observation_v2(
    ctx: Context<crate::SubmitObservationV2>,
    task_id: [u8; 32],
    nonce: u16,
    subject: Pubkey,
    capture_device: Pubkey,
    source: u8,
    provenance: u8,
    location: [i64; 2],
    observed_at: i64,
    findings_hash: [u8; 32],
    evidence_hash: [u8; 32],
    confidence: u8,
    signature_hash: [u8; 32],
) -> Result<()> {
    require!(
        is_valid_observation_source(source),
        TerraError::InvalidObservationSource
    );
    require!(
        is_valid_observation_provenance(provenance),
        TerraError::InvalidObservationProvenance
    );
    require!(
        is_valid_observation_confidence(confidence),
        TerraError::InvalidConfidence
    );
    require!(
        is_valid_observation_location(location),
        TerraError::InvalidPresenceFix
    );
    require!(task_id != [0u8; 32], TerraError::InvalidTaskRequirement);

    let t: &VerificationTask = &ctx.accounts.task;
    require!(t.task_id == task_id, TerraError::InvalidTaskRequirement);
    require!(can_observe(t.status), TerraError::TaskAlreadyFinalized);

    let now = Clock::get()?.unix_timestamp;
    let observer = ctx.accounts.observer.key();
    let o = &mut ctx.accounts.observation;
    o.task_id = task_id;
    o.observer = observer;
    o.nonce = nonce;
    o.subject = subject;
    o.capture_device = capture_device;
    o.source = source;
    o.provenance = provenance;
    o.location = location;
    o.observed_at = observed_at;
    o.findings_hash = findings_hash;
    o.evidence_hash = evidence_hash;
    o.confidence = confidence;
    o.signature_hash = signature_hash;
    o.created_at = now;

    emit!(crate::ObservationV2Submitted {
        observation: o.key(),
        task_id,
        observer,
        subject,
        capture_device,
        source,
        provenance,
        confidence,
        nonce,
    });
    Ok(())
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn observation_source_constants_are_contiguous() {
        assert_eq!(observation_source::PHONE, 0);
        assert_eq!(observation_source::GNSS, 1);
        assert_eq!(observation_source::CAMERA, 2);
        assert_eq!(observation_source::DRONE, 3);
        assert_eq!(observation_source::SATELLITE, 4);
        assert_eq!(observation_source::HUMAN, 5);
        assert_eq!(observation_source::DOCUMENT, 6);
        assert_eq!(observation_source::API, 7);
        for s in 0..=observation_source::MAX {
            assert!(is_valid_observation_source(s));
        }
        assert!(!is_valid_observation_source(8));
        assert_eq!(MAX_OBSERVATION_SOURCE, observation_source::MAX);
    }

    #[test]
    fn observation_provenance_constants_are_contiguous() {
        assert_eq!(observation_provenance::SELF_REPORTED, 0);
        assert_eq!(observation_provenance::DEVICE_GNSS, 1);
        assert_eq!(observation_provenance::MULTI_DEVICE, 2);
        assert_eq!(observation_provenance::LOCAL_VALIDATORS, 3);
        assert_eq!(observation_provenance::SURVEY_GRADE, 4);
        assert_eq!(observation_provenance::SATELLITE_CONFIRMED, 5);
        for p in 0..=observation_provenance::MAX {
            assert!(is_valid_observation_provenance(p));
        }
        assert!(!is_valid_observation_provenance(6));
        assert_eq!(MAX_OBSERVATION_PROVENANCE, observation_provenance::MAX);
    }

    #[test]
    fn observation_confidence_and_location_bounds() {
        assert!(is_valid_observation_confidence(0));
        assert!(is_valid_observation_confidence(100));
        assert!(!is_valid_observation_confidence(101));

        assert!(is_valid_observation_location([0, 0]));
        assert!(is_valid_observation_location([387_500_000, 121_500_000]));
        assert!(!is_valid_observation_location([900_000_001, 0]));
        assert!(!is_valid_observation_location([0, 1_800_000_001]));
        assert!(!is_valid_observation_location([-900_000_001, 0]));
    }

    #[test]
    fn can_observe_gates_terminal_tasks() {
        use crate::verification_task::task_status;
        assert!(can_observe(task_status::OPEN));
        assert!(can_observe(task_status::ASSIGNED));
        assert!(can_observe(task_status::IN_PROGRESS));
        assert!(!can_observe(task_status::COMPLETED));
        assert!(!can_observe(task_status::CANCELLED));
        assert!(!can_observe(task_status::EXPIRED));
    }
}
