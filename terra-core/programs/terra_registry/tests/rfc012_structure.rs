//! Structural tests for RFC-012: Terra Global Physical-Digital Trust Architecture.
//!
//! These tests validate that the RFC document contains required sections,
//! entity vocabulary, migration coverage, and the core invariant.
//!
//! Run: `cargo test -p terra-registry --test rfc012_structure`
//! (or as part of the workspace test suite)

use std::fs;

/// Locate RFC-012 relative to CARGO_MANIFEST_DIR (cargo) or CWD (standalone).
fn rfc012_path() -> std::path::PathBuf {
    // Prefer cargo-provided manifest dir; fall back to CWD for standalone rustc --test.
    let base = std::env::var("CARGO_MANIFEST_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::env::current_dir().expect("cwd"));
    // terra-core/programs/terra_registry -> repo root docs/
    // Also handle running from repo root directly.
    let candidates = [
        base.join("../../..")
            .join("docs/rfc-012-global-physical-digital-trust-architecture.md"),
        base.join("docs/rfc-012-global-physical-digital-trust-architecture.md"),
        base.join("../docs/rfc-012-global-physical-digital-trust-architecture.md"),
    ];
    for c in &candidates {
        if c.exists() {
            return c.clone();
        }
    }
    candidates[0].clone()
}

fn load_rfc012() -> String {
    let path = rfc012_path();
    assert!(
        path.exists(),
        "RFC-012 not found at {} — docs must live in repo docs/",
        path.display()
    );
    fs::read_to_string(&path).expect("failed to read RFC-012")
}

// ---------------------------------------------------------------------------
// Section presence
// ---------------------------------------------------------------------------

#[test]
fn rfc012_exists_and_is_nonempty() {
    let content = load_rfc012();
    assert!(
        content.len() > 2000,
        "RFC-012 too short: {} bytes",
        content.len()
    );
}

#[test]
fn rfc012_contains_core_invariant() {
    let content = load_rfc012();
    assert!(
        content.contains("No participant, device, organization, API, oracle, validator"),
        "missing core invariant sentence"
    );
    assert!(
        content.contains("observation → evidence → validation → consensus → state transition"),
        "missing observation→evidence→validation→consensus→state chain"
    );
}

#[test]
fn rfc012_contains_required_sections() {
    let content = load_rfc012();
    let required = [
        "## 0. Core Invariant",
        "## 1. Fundamental Principle",
        "## 2. Decentralization Model",
        "## 3. Three Types of Authority",
        "## 4. Separation of Concerns",
        "## 5. Architectural Vocabulary",
        "## 6. Design Rules",
        "## 7. Canonical Protocol Flow",
        "## 8. Implementation Phases",
        "## 9. Migration Map",
        "## 10. Account / PDA Sketch",
        "## 11. Testing Strategy",
        "## 12. Open Questions",
        "## Appendix A",
    ];
    for section in required {
        assert!(
            content.contains(section),
            "missing required section: {}",
            section
        );
    }
}

// ---------------------------------------------------------------------------
// Entity vocabulary completeness
// ---------------------------------------------------------------------------

#[test]
fn rfc012_entity_catalog_contains_identity_domain() {
    let content = load_rfc012();
    for entity in [
        "Identity",
        "LegalSubject",
        "Organization",
        "DeviceIdentity",
        "Credential",
        "ZKProof",
    ] {
        assert!(content.contains(entity), "missing entity: {}", entity);
    }
}

#[test]
fn rfc012_entity_catalog_contains_validator_domain() {
    let content = load_rfc012();
    for entity in [
        "Validator",
        "ValidatorCapability",
        "ValidatorPresence",
        "ValidatorAvailability",
        "ValidatorReputation",
        "ValidatorRelationship",
    ] {
        assert!(
            content.contains(entity),
            "missing validator entity: {}",
            entity
        );
    }
}

#[test]
fn rfc012_entity_catalog_contains_task_domain() {
    let content = load_rfc012();
    for entity in [
        "VerificationTask",
        "TaskRequirement",
        "TaskAssignment",
        "TaskResult",
        "TaskEscrow",
    ] {
        assert!(content.contains(entity), "missing task entity: {}", entity);
    }
}

#[test]
fn rfc012_entity_catalog_contains_observation_evidence_domain() {
    let content = load_rfc012();
    for entity in [
        "Observation",
        "ObservationSource",
        "ObservationProvenance",
        "GeoObservation",
        "Evidence",
        "EvidenceArtifact",
        "EvidenceManifest",
    ] {
        assert!(
            content.contains(entity),
            "missing observation/evidence entity: {}",
            entity
        );
    }
}

#[test]
fn rfc012_entity_catalog_contains_registry_domain() {
    let content = load_rfc012();
    for entity in [
        "Parcel",
        "SpatialAsset",
        "OwnershipRight",
        "LandRight",
        "GeometryVersion",
    ] {
        assert!(
            content.contains(entity),
            "missing registry entity: {}",
            entity
        );
    }
}

// ---------------------------------------------------------------------------
// Design rules coverage (key vision points)
// ---------------------------------------------------------------------------

#[test]
fn rfc012_design_rules_cover_mobile_validators() {
    let content = load_rfc012();
    assert!(
        content.contains("Validators are mobile"),
        "missing mobile validator rule"
    );
    assert!(
        content.contains("ValidatorPresence"),
        "presence must be referenced in design rules"
    );
}

#[test]
fn rfc012_design_rules_cover_permissionless_not_trusted() {
    let content = load_rfc012();
    assert!(
        content.contains("Permissionless ≠ immediately trusted"),
        "missing permissionless≠trusted rule"
    );
}

#[test]
fn rfc012_design_rules_cover_multifactor_selection() {
    let content = load_rfc012();
    assert!(
        content.contains("Selection is multi-factor"),
        "missing multi-factor selection rule"
    );
    assert!(
        content.contains("Randomized selection"),
        "selection must end in randomization"
    );
}

#[test]
fn rfc012_design_rules_cover_timeout_not_fraud() {
    let content = load_rfc012();
    assert!(
        content.contains("Timeout ≠ fraud"),
        "missing timeout≠fraud rule"
    );
}

#[test]
fn rfc012_design_rules_cover_subject_device_submitter_separation() {
    let content = load_rfc012();
    assert!(
        content.contains("Subject ≠ Capture Device ≠ Submitter"),
        "missing subject/device/submitter separation"
    );
}

#[test]
fn rfc012_design_rules_cover_gnss_as_input_not_oracle() {
    let content = load_rfc012();
    assert!(
        content.contains("GNSS is an input, not a central oracle"),
        "missing GNSS-as-input rule"
    );
}

#[test]
fn rfc012_design_rules_cover_capability_loss_over_ban() {
    let content = load_rfc012();
    assert!(
        content.contains("Capability loss over binary ban"),
        "missing capability-loss rule"
    );
}

#[test]
fn rfc012_design_rules_cover_due_process() {
    let content = load_rfc012();
    assert!(
        content.contains("accusation → evidence → notification → response → independent review → decision → appeal"),
        "missing due-process chain"
    );
}

// ---------------------------------------------------------------------------
// Migration map coverage
// ---------------------------------------------------------------------------

#[test]
fn rfc012_migration_map_covers_current_core_entities() {
    let content = load_rfc012();
    for entity in [
        "ValidatorRegistry",
        "ValidatorReputation",
        "Observation",
        "Claim",
        "Evidence",
        "Parcel.owner",
        "Attestation",
        "Jurisdiction",
        "EscrowRecord",
    ] {
        assert!(
            content.contains(entity),
            "migration map missing current entity: {}",
            entity
        );
    }
}

#[test]
fn rfc012_migration_map_references_dual_write_or_bridge() {
    let content = load_rfc012();
    assert!(
        content.contains("Dual-write")
            || content.contains("dual-write")
            || content.contains("bridge"),
        "migration should mention dual-write or bridge strategy"
    );
}

// ---------------------------------------------------------------------------
// Protocol flow completeness
// ---------------------------------------------------------------------------

#[test]
fn rfc012_protocol_flow_has_17_steps() {
    let content = load_rfc012();
    for step in 1..=17u32 {
        let marker = format!("{}.", step);
        // Steps appear as "1. " … "17. " in the flow block
        assert!(
            content.contains(&format!("\n{}. ", step)) || content.contains(&format!(" {}. ", step)),
            "protocol flow missing step {}: {}",
            step,
            marker
        );
    }
}

// ---------------------------------------------------------------------------
// PDA sketch presence for Phase 2+ entities
// ---------------------------------------------------------------------------

#[test]
fn rfc012_pda_sketch_covers_phase2_entities() {
    let content = load_rfc012();
    for seed in [
        "validator_profile",
        "validator_presence",
        "validator_availability",
        "validator_capability",
        "task",
        "task_requirement",
        "task_assignment",
        "task_escrow",
        "observation_v2",
        "evidence_manifest",
    ] {
        assert!(content.contains(seed), "PDA sketch missing seed: {}", seed);
    }
}

// ---------------------------------------------------------------------------
// Phase roadmap
// ---------------------------------------------------------------------------

#[test]
fn rfc012_phases_cover_0_through_10() {
    let content = load_rfc012();
    for phase in [
        "**0**", "**1**", "**2**", "**3**", "**4**", "**5**", "**6**", "**7**", "**8**", "**9**",
        "**10**",
    ] {
        assert!(content.contains(phase), "missing phase {}", phase);
    }
}
