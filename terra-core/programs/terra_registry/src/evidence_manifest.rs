use anchor_lang::prelude::*;

use crate::observation_v2::{is_valid_observation_provenance, is_valid_observation_source};
use crate::verification_task::{task_is_terminal, VerificationTask};
use crate::TerraError;

// ---------------------------------------------------------------------------
// RFC-012 Phase 5 — evidence provenance (EvidenceManifest / EvidenceArtifact)
//
// Goal: structured list of artifacts + hashes attached to a task; richer
// provenance flows (source + provenance on each artifact, optional link to
// an ObservationV2). PDA seeds per RFC-012 §10:
//   EvidenceManifest  ["evidence_manifest", task_id, nonce]
//   EvidenceArtifact  ["evidence_artifact", manifest, artifact_index]
// Migration map (§9): legacy claim-bound `Evidence` remains; new path is
// task-bound manifest + artifacts. Single legacy evidence = one-artifact
// manifest during dual-write.
// ---------------------------------------------------------------------------

/// Max artifacts per manifest (append-only; caps rent abuse).
pub const MAX_MANIFEST_ARTIFACTS: u16 = 32;
/// Max storage_reference length (same as legacy Evidence).
pub const MAX_STORAGE_REFERENCE_LEN: usize = 128;
/// Max valid artifact kind.
pub const MAX_ARTIFACT_KIND: u8 = 5;

/// Artifact kinds (RFC: photo, document, video, model, geometry, + other).
pub mod artifact_kind {
    pub const PHOTO: u8 = 0;
    pub const DOCUMENT: u8 = 1;
    pub const VIDEO: u8 = 2;
    pub const MODEL: u8 = 3;
    pub const GEOMETRY: u8 = 4;
    pub const OTHER: u8 = 5;
    pub const MAX: u8 = OTHER;
}

// ---------------------------------------------------------------------------
// Accounts (PDA seeds per RFC-012 §10)
// ---------------------------------------------------------------------------

/// Structured list of evidence artifacts for a verification task.
///
/// PDA: `["evidence_manifest", task_id, nonce]`
#[account]
#[derive(InitSpace)]
pub struct EvidenceManifest {
    /// Task this manifest supports.
    pub task_id: [u8; 32],
    /// Wallet that opened the manifest (PDA seed companion). Pays rent.
    pub submitter: Pubkey,
    /// Unique nonce per (task_id, submitter). PDA seed.
    pub nonce: u16,
    /// Optional ObservationV2 this manifest expands (`Pubkey::default()` = none).
    pub observation: Pubkey,
    /// Number of artifacts appended so far (0..=MAX_MANIFEST_ARTIFACTS).
    pub artifact_count: u16,
    /// Pre-declared or rolling manifest root hash (sha256 of artifact set).
    /// May be zero at open; clients recompute/verify off-chain.
    pub root_hash: [u8; 32],
    pub created_at: i64,
}

/// One artifact inside a manifest (photo / document / video / model / geometry).
///
/// PDA: `["evidence_artifact", manifest, artifact_index]`
#[account]
#[derive(InitSpace)]
pub struct EvidenceArtifact {
    /// Parent manifest (PDA seed).
    pub manifest: Pubkey,
    /// Append-only index within the manifest (PDA seed).
    pub artifact_index: u16,
    /// One of `artifact_kind`.
    pub kind: u8,
    /// Capture source — one of `observation_source` (reused for provenance richness).
    pub source: u8,
    /// Provenance — one of `observation_provenance`.
    pub provenance: u8,
    /// sha256 of the artifact bytes / content envelope.
    pub content_hash: [u8; 32],
    /// IPFS CID, URL, or content address (≤128 bytes).
    #[max_len(MAX_STORAGE_REFERENCE_LEN)]
    pub storage_reference: String,
    pub created_at: i64,
}

// ---------------------------------------------------------------------------
// Pure helpers (unit-testable)
// ---------------------------------------------------------------------------

pub fn is_valid_artifact_kind(kind: u8) -> bool {
    kind <= artifact_kind::MAX
}

pub fn is_valid_storage_reference(s: &str) -> bool {
    !s.is_empty() && s.len() <= MAX_STORAGE_REFERENCE_LEN
}

pub fn is_nonzero_hash(h: &[u8; 32]) -> bool {
    !h.iter().all(|b| *b == 0)
}

/// Evidence may be attached only while the task is not terminal.
pub fn can_attach_evidence(status: u8) -> bool {
    !task_is_terminal(status)
}

/// Next slot must equal current count (append-only order).
pub fn is_expected_artifact_index(current_count: u16, artifact_index: u16) -> bool {
    artifact_index == current_count
}

/// Manifest still has room for another artifact.
pub fn manifest_has_room(current_count: u16) -> bool {
    current_count < MAX_MANIFEST_ARTIFACTS
}

// ---------------------------------------------------------------------------
// Instruction handlers
// ---------------------------------------------------------------------------

/// Open an evidence manifest for a verification task.
/// Permissionless (any wallet may open); submitter pays rent.
/// Optional `observation` links an ObservationV2 (Phase 4) this expands.
#[allow(clippy::too_many_arguments)]
pub fn submit_evidence_manifest(
    ctx: Context<crate::SubmitEvidenceManifest>,
    task_id: [u8; 32],
    nonce: u16,
    observation: Pubkey,
    root_hash: [u8; 32],
) -> Result<()> {
    require!(task_id != [0u8; 32], TerraError::InvalidTaskRequirement);

    let t: &VerificationTask = &ctx.accounts.task;
    require!(t.task_id == task_id, TerraError::InvalidTaskRequirement);
    require!(
        can_attach_evidence(t.status),
        TerraError::TaskAlreadyFinalized
    );

    let now = Clock::get()?.unix_timestamp;
    let submitter = ctx.accounts.submitter.key();
    let m = &mut ctx.accounts.manifest;
    m.task_id = task_id;
    m.submitter = submitter;
    m.nonce = nonce;
    m.observation = observation;
    m.artifact_count = 0;
    m.root_hash = root_hash;
    m.created_at = now;

    emit!(crate::EvidenceManifestSubmitted {
        manifest: m.key(),
        task_id,
        submitter,
        observation,
        nonce,
    });
    Ok(())
}

/// Append one artifact to an existing manifest.
/// Permissionless; `artifact_index` must equal `manifest.artifact_count`
/// (append-only). Validates kind / source / provenance / content / storage.
#[allow(clippy::too_many_arguments)]
pub fn add_evidence_artifact(
    ctx: Context<crate::AddEvidenceArtifact>,
    artifact_index: u16,
    kind: u8,
    source: u8,
    provenance: u8,
    content_hash: [u8; 32],
    storage_reference: String,
) -> Result<()> {
    require!(
        is_valid_artifact_kind(kind),
        TerraError::InvalidEvidenceArtifactKind
    );
    require!(
        is_valid_observation_source(source),
        TerraError::InvalidObservationSource
    );
    require!(
        is_valid_observation_provenance(provenance),
        TerraError::InvalidObservationProvenance
    );
    require!(is_nonzero_hash(&content_hash), TerraError::EmptyContentHash);
    require!(
        is_valid_storage_reference(&storage_reference),
        TerraError::EmptyStorageReference
    );

    // Terminal-task gate via the parent task account (seeds-checked).
    let t: &VerificationTask = &ctx.accounts.task;
    require!(
        t.task_id == ctx.accounts.manifest.task_id,
        TerraError::InvalidTaskRequirement
    );
    require!(
        can_attach_evidence(t.status),
        TerraError::TaskAlreadyFinalized
    );

    let m = &mut ctx.accounts.manifest;
    require!(
        is_expected_artifact_index(m.artifact_count, artifact_index),
        TerraError::EvidenceIndexMismatch
    );
    require!(
        manifest_has_room(m.artifact_count),
        TerraError::EvidenceManifestFull
    );
    m.artifact_count = m.artifact_count.saturating_add(1);

    let now = Clock::get()?.unix_timestamp;
    let manifest_key = m.key();
    let a = &mut ctx.accounts.artifact;
    a.manifest = manifest_key;
    a.artifact_index = artifact_index;
    a.kind = kind;
    a.source = source;
    a.provenance = provenance;
    a.content_hash = content_hash;
    a.storage_reference = storage_reference;
    a.created_at = now;

    emit!(crate::EvidenceArtifactAdded {
        artifact: a.key(),
        manifest: manifest_key,
        task_id: m.task_id,
        artifact_index,
        kind,
        source,
        provenance,
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
    fn artifact_kind_constants_are_contiguous() {
        assert_eq!(artifact_kind::PHOTO, 0);
        assert_eq!(artifact_kind::DOCUMENT, 1);
        assert_eq!(artifact_kind::VIDEO, 2);
        assert_eq!(artifact_kind::MODEL, 3);
        assert_eq!(artifact_kind::GEOMETRY, 4);
        assert_eq!(artifact_kind::OTHER, 5);
        for k in 0..=artifact_kind::MAX {
            assert!(is_valid_artifact_kind(k));
        }
        assert!(!is_valid_artifact_kind(6));
        assert_eq!(MAX_ARTIFACT_KIND, artifact_kind::MAX);
    }

    #[test]
    fn storage_reference_bounds() {
        assert!(is_valid_storage_reference("ipfs://cid"));
        assert!(!is_valid_storage_reference(""));
        assert!(!is_valid_storage_reference(&"x".repeat(129)));
        assert!(is_valid_storage_reference(&"x".repeat(128)));
    }

    #[test]
    fn content_hash_nonzero_check() {
        assert!(is_nonzero_hash(&[1u8; 32]));
        assert!(!is_nonzero_hash(&[0u8; 32]));
    }

    #[test]
    fn append_only_index_and_cap() {
        assert!(is_expected_artifact_index(0, 0));
        assert!(is_expected_artifact_index(1, 1));
        assert!(!is_expected_artifact_index(1, 0));
        assert!(!is_expected_artifact_index(1, 2));

        assert!(manifest_has_room(0));
        assert!(manifest_has_room(MAX_MANIFEST_ARTIFACTS - 1));
        assert!(!manifest_has_room(MAX_MANIFEST_ARTIFACTS));
    }

    #[test]
    fn can_attach_evidence_gates_terminal_tasks() {
        use crate::verification_task::task_status;
        assert!(can_attach_evidence(task_status::OPEN));
        assert!(can_attach_evidence(task_status::ASSIGNED));
        assert!(can_attach_evidence(task_status::IN_PROGRESS));
        assert!(!can_attach_evidence(task_status::COMPLETED));
        assert!(!can_attach_evidence(task_status::CANCELLED));
        assert!(!can_attach_evidence(task_status::EXPIRED));
    }
}
