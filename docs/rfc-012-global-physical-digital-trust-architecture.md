# RFC-012: Terra Global Physical-Digital Trust Architecture

**Status:** Accepted (Phases 0–2 complete on `dev`; Phases 3–10 pending)  
**Created:** 2026-09-22  
**Updated:** 2026-09-24  
**Supersedes:** None (architecture contract; refines RFC-003…011)  
**Related:** RFC-004 (escrow), RFC-005 (staking), RFC-007 (disputes), RFC-011 (ZK)

**Entry for a new contributor or agent:** read §8.1 (Handoff) first, then §8 (phase table), §9 (migration map), §10 (PDA sketch). Do not invent source counts — see `terra-core/docs/architecture.md` (verified 2026-09-23).

**Phase 0 (this document + migration map + structural tests)**, **Phase 1 (security hardening)**, and **Phase 2 (generalized validator PDAs)** are complete:

- Unique validator sets enforced on succession, guardianship, attest, rotate, forfeit, dispute, escrow (`DuplicateValidator`).
- Endorsement action binding on `endorse_validator_add` (`WrongEndorsementAction`).
- `remaining_accounts` ownership checks on session/quorum_config/reputation loaders and observation loop.
- Admin/authority constraints on jail, reputation, ZK, dispute, cross-border, and guardian paths (see `terra-core/SECURITY.md`).
- Structural tests: `terra-core/programs/terra_registry/tests/rfc012_structure.rs` (21 tests).

---

## 0. Core Invariant

> **No participant, device, organization, API, oracle, validator, reputation engine, or economic mechanism may unilaterally convert an observation into canonical ownership or legal state.**

There must always be a defined chain:

```
observation → evidence → validation → consensus → state transition
```

---

## 1. Fundamental Principle

Terra is **not** a blockchain land registry. Terra is:

> A decentralized protocol for transforming observations of the physical world into verifiable digital facts, coordinating independent participants to validate those facts, recording rights and claims, and enabling permissionless discovery, exchange and interoperability across jurisdictions.

Land is the first major domain. The architecture is reusable for: buildings, infrastructure, natural resources, agricultural areas, borders/geographic features, physical assets, environmental observations, and other real-world assets.

---

## 2. Decentralization Model

Terra has **no mandatory central operational authority**. Normal operation must not require:

- Terra company approval
- A central validator administrator
- A central database
- A central identity provider
- A central geographic authority
- A central hardware provider
- A central storage provider
- A central oracle

**Decentralized** means no central authority is required for *normal operation* — not that every possible authority can never exist. Governments, courts, organizations, and survey agencies may participate as *trusted data sources or jurisdictional authorities*, but Terra's protocol must not depend on one of them to function.

---

## 3. Three Types of Authority

| Type | Description | Example |
|------|-------------|---------|
| **Protocol authority** | Rules enforced by smart contracts | "A quorum requires 4 independent validators" |
| **Jurisdictional authority** | External organization publishes information | "Country X recognizes document Y" — recorded as external assertion, does not control Terra |
| **Evidence authority** | Non-government provides evidence with provenance/trust level | Surveyor, citizen, drone operator, satellite provider |

---

## 4. Separation of Concerns (Mandatory)

**Do not model:**

```
Person = Validator = Device = Location = Identity
```

**Instead model independently:**

```
PERSON → identities, credentials, validator profile
DEVICE → phone, GNSS, drone, camera, survey instrument
LOCATION → current position, operating region, jurisdiction
OBSERVATION → device, location, timestamp, evidence, provenance
VALIDATION → validator, observations, methodology, conclusion
REGISTRY → parcel, rights, claims, historical state
```

---

## 5. Architectural Vocabulary (Entity Catalog)

These entities define the canonical vocabulary. Not all need accounts on day one.

### Identity & Legal Subjects
- `Identity` — cryptographic identity anchor
- `LegalSubject` — Person | Organization | Cooperative | Government | Trust | Estate | Other
- `Organization` — multi-member entity
- `DeviceIdentity` — device cryptographic key, capabilities, calibration, ownership
- `Credential` — verifiable credential
- `ZKProof` — zero-knowledge proof artifact

### Validator Domain
- `Validator` — mobile, capability-bearing participant (NOT location-bound)
- `ValidatorCapability` — declared/observed/verified/trusted capability set
- `ValidatorPresence` — dynamic geographic presence (timestamped, confidence-scored, expiring)
- `ValidatorAvailability` — ONLINE | AVAILABLE | BUSY | OFFLINE | TEMPORARILY_UNAVAILABLE | SUSPENDED
- `ValidatorReputation` — multi-dimensional trust (identity_confidence, accuracy, reliability, geographic_experience, equipment_confidence, dispute_history, fraud_risk, collusion_risk)
- `ValidatorRelationship` — endorsement, co-validation, shared infrastructure graph edges

### Geographic
- `Jurisdiction` — legal region with its own rules
- `Region` — geographic area
- `CoverageZone` — area needing validation coverage

### Registry
- `Parcel` — land parcel (2D baseline)
- `SpatialAsset` — extends Parcel with 3D, elevation, structures, temporal versions
- `OwnershipRight` / `LandRight` — subject, parcel, right_type, jurisdiction, start, expiry, restrictions, evidence
- `GeometryVersion` — versioned geometry hash
- `ThreeDModel` — off-chain model reference + hash
- `SpatialSnapshot` — point-in-time spatial state

### Observation & Evidence
- `Observation` — device, location, timestamp, evidence, provenance
- `ObservationSource` — GNSS, camera, drone, satellite, human, document, API
- `ObservationProvenance` — SELF_REPORTED | DEVICE_GNSS | MULTI_DEVICE | LOCAL_VALIDATORS | SURVEY_GRADE | SATELLITE_CONFIRMED
- `GeoObservation` — geospatial-specific observation
- `Evidence` — cryptographic anchor for artifacts
- `EvidenceArtifact` — photo, document, video, model, geometry
- `EvidenceManifest` — structured list of artifacts + hashes

### Task Marketplace
- `VerificationTask` — first-class work unit (requester, subject, type, requirements, reward, deadline)
- `TaskRequirement` — required evidence, capabilities, geographic/jurisdiction constraints, min reputation, validator count, independence, confidence target
- `TaskAssignment` — selected validator(s) for a task
- `TaskResult` — outcome + attestations

### Verification Pipeline
- `VerificationSession` — bounded verification context
- `VerificationAttestation` — validator's signed conclusion
- `Quorum` — threshold of independent attestations
- `Dispute` — contested result
- `ReviewCase` — independent review of a dispute/accusation

### Governance & Reputation
- `ReputationEvent` — scored event feeding reputation dimensions
- `FraudReport` — accusation with evidence
- `CapabilityRestriction` — partial capability downgrade (not binary ban)
- `Appeal` — due-process appeal path

### Economics
- `FeePolicy` — protocol fee rules
- `ResourceQuote` — estimated cost of task resources
- `TaskEscrow` — locked funds for a task
- `RewardAllocation` — distribution to participants
- `CoverageIncentive` — demand + coverage-deficit + difficulty + strategic importance subsidy

### Cross-Border & Proofs
- `CrossBorderBinding` — jurisdiction-to-jurisdiction link
- `CrossBorderVerification` — verification spanning jurisdictions

---

## 6. Design Rules

1. **Validators are mobile.** Never store `validator.region = "Yaoundé"` as immutable identity. Use `ValidatorPresence` with timestamp, accuracy, source, confidence, expires_at.
2. **Permissionless ≠ immediately trusted.** New validators start NEW/PROBATIONARY with low confidence; progress through ESTABLISHED → TRUSTED → HIGH_CAPABILITY across multiple dimensions (not one global score).
3. **Selection is multi-factor, not nearest-only.** Eligibility → Capability → Jurisdiction → Geographic relevance → Availability → Reputation → Independence → Conflict filtering → Randomized selection. Distance is one variable.
4. **Task classes:** Physical | Remote | Documentary | Computational | Hybrid. Physical presence is not always required.
5. **Timeout ≠ fraud.** Offline/traveling/power-loss affects availability score, not automatic slashing.
6. **Identity does not require a smartphone.** Assisted ceremonies, hardware wallets, biometric/device-assisted, and no-personal-device paths must be supported. The facilitating validator must NOT become permanent owner of the person's identity.
7. **Subject ≠ Capture Device ≠ Submitter.** A document can be owned by Alice, photographed on Bob's phone, submitted by Charlie — all distinct roles in the observation record.
8. **GNSS is an input, not a central oracle.** Coordinates, timestamps, accuracy, signatures, provenance, consistency checks are managed; absolute truth is never asserted from a single source.
9. **2D and 3D are native.** `SpatialAsset` supports 2D boundary, elevation, 3D geometry, structures, underground features, temporal versions, observation layers.
10. **Do not put 3D models on-chain.** Store geometry_hash, model_hash, metadata, version, storage_reference only.
11. **Reputation ≠ money.** Wealth cannot buy trust status. Registration + reputation + optional staking remain distinct.
12. **Correlation ≠ fraud.** Validator relationship graphs detect suspicious concentration for *review*, not automatic punishment.
13. **Capability loss over binary ban.** Confirmed misconduct removes specific capabilities (high-value, legal-document, high-confidence); low-risk observation may remain. Rehabilitation restores gradually.
14. **Due process:** accusation → evidence → notification → response → independent review → decision → appeal.
15. **Terra does not replace law.** It establishes protocol facts ("according to evidence and consensus, this right exists"). Legal recognition remains with jurisdictions that choose to integrate.

---

## 7. Canonical Protocol Flow

```
1.  Someone creates a TASK.
2.  Task describes: what to establish, required evidence, confidence,
    capabilities, geographic/jurisdiction constraints, reward, deadline.
3.  Terra finds ELIGIBLE participants.
4.  Terra filters: unavailable, conflicts, insufficient reputation,
    insufficient capabilities, excessive correlation.
5.  Terra randomly selects among remaining candidates.
6.  Participants collect OBSERVATIONS.
7.  Observations become EVIDENCE.
8.  Evidence is independently evaluated.
9.  Validators produce ATTESTATIONS.
10. Quorum determines the result.
11. Result becomes a protocol fact/claim.
12. Registry changes only if required conditions satisfied.
13. Requester pays.
14. Rewards distributed.
15. Coverage incentives calculated.
16. All relevant history remains auditable.
17. Disputed results enter independent REVIEW.
```

---

## 8. Implementation Phases (from current `dev`)

| Phase | Goal | Status | Key deliverables |
|-------|------|--------|------------------|
| **0** | Architecture contract | **Done** | This RFC + migration map + structural tests (`rfc012_structure.rs` 21/21) |
| **1** | Secure current core | **Done** | Succession/guardianship duplicate endorsements, unique validator sets, endorsement action binding, remaining_accounts ownership audit, canonical ownership (is_authorized_owner), admin constraints on jail/reputation/ZK/dispute/cross-border/guardian paths; escrow invariants — see SECURITY.md |
| **2** | Generalize validator | **Complete** (2026-09-24) | `ValidatorProfile`/`Presence`/`Availability`/`Capability`/`RelationshipEdge` PDAs + 9 instructions, 5 accounts, 7 events, 9 errors; unit + BPF tests green; IDL 128/49/115/169 |
| **3** | Introduce tasks | Pending | `Task → requirements → routing → assignment → verification → reward` |
| **4** | Introduce observations | Pending | Multi-source: phone, GNSS, drone, satellite, document, human |
| **5** | Evidence/provenance | Pending | Separate subject / capture_device / submitter roles |
| **6** | Dynamic routing | Pending | Capability + geography + availability + reputation + independence + randomness selection |
| **7** | Reputation governance | Pending | Fraud report → review → random independent committee → decision → capability downgrade → appeal → rehabilitation |
| **8** | Economic/resource layer | Pending | Task cost → escrow → rewards → infrastructure → coverage subsidy |
| **9** | Physical infrastructure | Pending | Smartphone, GNSS, drones, survey devices, satellite imagery, 3D scanning |
| **10** | Cross-border + privacy | Pending | Jurisdiction bindings, ZK identity/ownership, selective disclosure |

Phase 2 is complete on `dev` (2026-09-24). Before starting Phase 3 on a clean checkout, confirm the checked-in IDL matches source (`make idl` — as of Phase 2 2026-09-24 it is 128/49/115/169 with A1 session-record accounts + Phase 2 validator-profile PDAs) and clear remaining SECURITY.md mainnet items (RFC-005 reconfirm, ZK audit) — see §8.1.

---

## 8.1. Handoff: Phase 2 complete; how to start Phase 3

**Already done (do not redo):** Phase 0 structural tests (`terra-core/programs/terra_registry/tests/rfc012_structure.rs`, 21 tests); Phase 1 unique-validator sets, endorsement action binding, remaining-accounts owner checks, admin constraints — verified list in `terra-core/SECURITY.md` “Phase 1 additions”.

**Phase 2 delivered (2026-09-24):** `validator_profile.rs` module with `ValidatorProfile`, `ValidatorPresence`, `ValidatorAvailability`, `ValidatorCapability`, `ValidatorRelationshipEdge` accounts; instructions `init/update/set_tier` profile, `set_presence`, `set/suspend_availability`, `declare/admin_verify_capability`, `create_relationship_edge`; 6 lib unit tests + 4 BPF integration tests in `integration.rs` (`phase2_*`). Guards: self-init at tier NEW only, admin-only tier changes, SUSPENDED reserved for admin, VERIFIED/TRUSTED capability admin-only, self-edge rejected.

**Immediate next actions for Phase 3:**
1. Read §9 (Migration Map) and §10 (PDA sketch) — Phase 2 delivered `ValidatorProfile` etc. as PDAs while keeping `ValidatorRegistry` as an index; Phase 3 (tasks) should follow the same pattern.
2. Checked-in IDL was refreshed in A2 (119/44/108/160) and Phase 2 (128/49/115/169); re-run `make idl` after further program edits. Close remaining SECURITY.md items (RFC-005 reconfirm, ZK audit) if touching staking/ZK.
3. When adding accounts/instructions/events/errors: update `rfc012_structure.rs` expectations, root/`terra-core` README source counts, and run `make idl`.
4. Ship each phase with unit tests + BPF integration tests per §11.

**Phase order (next):** 3 (tasks) → 4 (observations) → 5 (evidence provenance) → 6 (routing) → 7 (reputation governance) → 8 (economics) → 9 (infrastructure) → 10 (cross-border/privacy). Do not skip the migration map dual-write rules in §9.

---

## 9. Migration Map from Current `dev`

| Current Entity | Target Entity | Migration Note |
|----------------|---------------|----------------|
| `ValidatorRegistry.validators: Vec<Pubkey>` | `Validator` accounts + registry index | Keep registry as index; move attributes to per-validator PDA |
| `ValidatorReputation` (single score) | `ValidatorReputation` multi-dimensional | Extend struct; keep old score as `legacy_score` for continuity |
| `Observation` (validator-bound) | `Observation` + `ObservationProvenance` + separate subject/capture/submitter | Add role fields; backfill with validator as all roles |
| `Claim` | `VerificationTask` (new) + `Claim` (legacy subject) | Claims become task subjects; new tasks reference claims |
| `Evidence` | `Evidence` + `EvidenceManifest` + `EvidenceArtifact` | Manifest groups artifacts; single evidence becomes one-artifact manifest |
| `Parcel.owner: Pubkey` | `OwnershipRight` via `LegalSubject` | Dual-write during transition; `is_authorized_owner()` already supports both |
| `Attestation` (deprecated) | `VerificationAttestation` | Already bridged via `migrate_attestation_to_claim` |
| `Jurisdiction` | `Jurisdiction` + `CrossBorderBinding` | Extend with binding relationships |
| `EscrowRecord` (parcel-only) | `TaskEscrow` + parcel escrow | Generalize to any task subject |

---

## 10. Account / PDA Sketch (Phase 2+)

```
ValidatorProfile            seeds: ["validator_profile", wallet]
ValidatorPresence           seeds: ["validator_presence", wallet]
ValidatorAvailability       seeds: ["validator_availability", wallet]
ValidatorCapability         seeds: ["validator_capability", wallet, capability_code]
ValidatorRelationshipEdge   seeds: ["validator_edge", from, to, edge_type]

VerificationTask            seeds: ["task", task_id]
TaskRequirement             seeds: ["task_requirement", task_id, req_index]
TaskAssignment              seeds: ["task_assignment", task_id, validator]
TaskEscrow                  seeds: ["task_escrow", task_id]

ObservationV2               seeds: ["observation_v2", task_id, observer, nonce]
EvidenceManifest            seeds: ["evidence_manifest", task_id, nonce]
EvidenceArtifact            seeds: ["evidence_artifact", manifest, artifact_index]

ReviewCase                  seeds: ["review_case", subject, nonce]
FraudReport                 seeds: ["fraud_report", accused, reporter, nonce]
CapabilityRestriction       seeds: ["capability_restriction", wallet, capability_code]
```

---

## 11. Testing Strategy

Every phase must ship with:

1. **Unit tests** for pure logic (selection filters, reputation math, state machines, provenance validation).
2. **Integration tests** (BPF via `solana-program-test`) for instruction happy paths + guard rails.
3. **Structural tests** for this RFC (entity catalog completeness, migration map coverage, invariant presence).

---

## 12. Open Questions (deferred)

- Exact capability taxonomy codes (GNSS, IMAGERY, DOCUMENT, SURVEY, LEGAL, …)
- Presence accuracy thresholds per provenance level
- Randomness source for committee selection (blockhash vs VRF)
- Coverage incentive formula weights
- Max concurrent task limits per validator tier

---

## Appendix A: One-Sentence Architecture

> Build Terra as a permissionless, decentralized physical-digital verification network in which any person can progressively become a validator, validators are mobile rather than location-bound, devices and people are separate identities, observations are distinct from evidence and canonical registry state, verification work is expressed as permissionless tasks, validator selection is dynamically based on capability/geography/availability/reputation/independence followed by randomness, and no single authority is required for ordinary identity, observation, validation, registry, dispute, or settlement operations.
