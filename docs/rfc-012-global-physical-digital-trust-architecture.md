# RFC-012: Terra Global Physical-Digital Trust Architecture

**Status:** Accepted (Phases 0–8 complete on `dev`; Phases 9–10 pending)  
**Created:** 2026-09-22  
**Updated:** 2026-09-24  
**Supersedes:** None (architecture contract; refines RFC-003…011)  
**Related:** RFC-004 (escrow), RFC-005 (staking), RFC-007 (disputes), RFC-011 (ZK)

**Entry for a new contributor or agent:** read §8.1 (Handoff) first, then §8 (phase table), §9 (migration map), §10 (PDA sketch). Do not invent source counts — see `terra-core/docs/architecture.md` (verified 2026-09-23).

**Phase 0 (this document + migration map + structural tests)**, **Phase 1 (security hardening)**, and **Phase 2 (generalized validator PDAs)** are complete:

- Unique validator sets enforced on succession, guardianship, forfeit, dispute, escrow (`DuplicateValidator`).
- Endorsement action binding on `endorse_validator_add` (`WrongEndorsementAction`).
- `remaining_accounts` ownership checks on session/quorum_config/reputation loaders and observation loop.
- Admin/authority constraints on reputation, ZK, dispute, cross-border, and guardian paths (see `terra-core/SECURITY.md`).
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
| **1** | Secure current core | **Done** | Succession/guardianship duplicate endorsements, unique validator sets, endorsement action binding, remaining_accounts ownership audit, canonical ownership (is_authorized_holder; RRR migration replaced `is_authorized_owner`), admin constraints on jail/reputation/ZK/dispute/cross-border/guardian paths; escrow invariants — see SECURITY.md |
| **2** | Generalize validator | **Complete** (2026-09-24) | `ValidatorProfile`/`Presence`/`Availability`/`Capability`/`RelationshipEdge` PDAs + 9 instructions, 5 accounts, 7 events, 9 errors; unit + BPF tests green; IDL 128/49/115/169 |
| **3** | Introduce tasks | **Complete** (2026-09-24) | `VerificationTask`/`TaskRequirement`/`TaskAssignment` PDAs + 6 instructions (create/add/assign/claim/submit/cancel), 3 accounts, 5 events, 13 errors; unit + BPF tests green; IDL 134/52/120/182 |
| **4** | Introduce observations | **Complete** (2026-09-24) | `ObservationV2` PDA (seeds `["observation_v2", task_id, observer, nonce]`) + `submit_observation_v2`; multi-source PHONE/GNSS/CAMERA/DRONE/SATELLITE/HUMAN/DOCUMENT/API; provenance SELF_REPORTED…SATELLITE_CONFIRMED; subject/capture_device/observer role separation; 1 account, 1 instruction, 1 event, 2 errors; 4 unit + 2 BPF tests; IDL 135/53/121/184 |
| **5** | Evidence/provenance | **Complete** (2026-09-24) | `EvidenceManifest`/`EvidenceArtifact` PDAs (`["evidence_manifest", task_id, submitter, nonce]`, `["evidence_artifact", manifest, artifact_index]`) + `submit_evidence_manifest`/`add_evidence_artifact`; artifact kinds PHOTO…OTHER; optional ObservationV2 link; append-only index + cap 32; 2 accounts, 2 instructions, 2 events, 3 errors (6184–6186); 5 unit + 2 BPF tests; IDL 137/55/123/187 |
| **6** | Dynamic routing | **Complete** (2026-09-24) | `routing.rs` multi-factor eligibility (Eligibility→Capability→Jurisdiction→Geo→Availability→Reputation→independence) + instruction `route_task` creating Phase 3 `TaskAssignment` for the random winner; 0 new PDAs, 1 instruction, 1 event (`TaskRouted`), 4 errors (6187–6190); 7 unit + 2 BPF tests; IDL 138/55/124/191 |
| **7** | Reputation governance | **Complete** (2026-09-24) | `fraud_governance.rs` with `FraudReport` (`["fraud_report", accused, reporter, nonce]`), `ReviewCase` (`["review_case", report]`), `CapabilityRestriction` (`["capability_restriction", wallet, capability_code]`), `Appeal` (`["appeal", restriction, appellant, nonce]`); 9 instructions (submit/open/cast/finalize fraud + file/open/cast/finalize appeal + rehabilitate); random well-reputed committee (COMMITTEE_SIZE=5, MIN reputation 5000 bps); upheld fraud demotes to DECLARED/PROBATIONARY (no jail — Design Rule 13); route_task capability gate (stride `3 + need_geo + 2*need_cap`); 15 errors (6191–6205); 15 unit + 6 BPF tests (`phase7_*`); IDL 147/59/133/206 |
| **8** | Economic/resource layer | **Complete** (2026-09-24) | `task_economics.rs`: quote → task escrow → reward + coverage subsidy → refund; `FeePolicy` (`["fee_policy"]`), `CoverageIncentive` (`["coverage_incentive"]`), `ResourceQuote` (`["resource_quote", task_id]`), `TaskEscrow` (`["task_escrow", task_id]`, vault `["task_escrow_vault", escrow]`), `RewardAllocation` (`["reward_allocation", task_id, validator]`), fee sink `task_treasury` (`["task_treasury"]`); 6 instructions (set policies / quote / fund / claim / refund); 6 events; 14 errors (6206–6219, reuses 6175/6176/6174); 11 unit + 3 BPF tests (`phase8_*`); IDL 153/64/139/220 |
| **9** | Physical infrastructure | Pending | Smartphone, GNSS, drones, survey devices, satellite imagery, 3D scanning |
| **10** | Cross-border + privacy | Pending | Jurisdiction bindings, ZK identity/ownership, selective disclosure |

Phase 8 is complete on `dev` (2026-09-24). Before starting Phase 9 on a clean checkout, confirm the checked-in IDL matches source (`make idl` — as of Phase 8 2026-09-24 it is 153/64/139/220 with Phase 2 validator-profile PDAs + Phase 3 task PDAs + Phase 4 ObservationV2 + Phase 5 EvidenceManifest/EvidenceArtifact + Phase 6 route_task + Phase 7 fraud/review/restriction/appeal PDAs + Phase 8 fee/coverage/quote/escrow/allocation PDAs) and clear remaining SECURITY.md mainnet items (RFC-005 reconfirm, ZK audit) — see §8.1.

---

## 8.1. Handoff: Phase 8 complete; how to start Phase 9

**Already done (do not redo):** Phase 0 structural tests (`terra-core/programs/terra_registry/tests/rfc012_structure.rs`, 21 tests); Phase 1 unique-validator sets, endorsement action binding, remaining-accounts owner checks, admin constraints — verified list in `terra-core/SECURITY.md` “Phase 1 additions”.

**Phase 2 delivered (2026-09-24):** `validator_profile.rs` module with `ValidatorProfile`, `ValidatorPresence`, `ValidatorAvailability`, `ValidatorCapability`, `ValidatorRelationshipEdge` accounts; instructions `init/update/set_tier` profile, `set_presence`, `set/suspend_availability`, `declare/admin_verify_capability`, `create_relationship_edge`; 6 lib unit tests + 4 BPF integration tests in `integration.rs` (`phase2_*`). Guards: self-init at tier NEW only, admin-only tier changes, SUSPENDED reserved for admin, VERIFIED/TRUSTED capability admin-only, self-edge rejected.

**Phase 3 delivered (2026-09-24):** `verification_task.rs` module with `VerificationTask` (seeds `["task", task_id]`), `TaskRequirement` (seeds `["task_requirement", task_id, req_index]`), `TaskAssignment` (seeds `["task_assignment", task_id, validator]`); 6 instructions `create_verification_task`, `add_task_requirement`, `assign_task_validator`, `claim_task`, `submit_task_result`, `cancel_task`; 5 events; 13 errors appended after `SelfRelationshipEdge`; 6 lib unit tests + 2 BPF integration tests (`phase3_*`). Task classes PHYSICAL/REMOTE/DOCUMENTARY/COMPUTATIONAL/HYBRID; statuses OPEN/ASSIGNED/IN_PROGRESS/COMPLETED/CANCELLED/EXPIRED; outcomes PASS/FAIL/INCONCLUSIVE. `reward_lamports` is recorded on the task (escrow/distribution deferred to Phase 8). Full multi-factor routing remains Phase 6 — Phase 3 ships explicit requester assignment + first-come claim.

**Phase 4 delivered (2026-09-24):** `observation_v2.rs` module with `ObservationV2` account (PDA seeds `["observation_v2", task_id, observer, nonce]`); instruction `submit_observation_v2` (permissionless; observer pays rent); `observation_source` PHONE/GNSS/CAMERA/DRONE/SATELLITE/HUMAN/DOCUMENT/API; `observation_provenance` SELF_REPORTED/DEVICE_GNSS/MULTI_DEVICE/LOCAL_VALIDATORS/SURVEY_GRADE/SATELLITE_CONFIRMED; subject / capture_device / observer independent roles (Design Rule 7); event `ObservationV2Submitted`; errors `InvalidObservationSource` (6182), `InvalidObservationProvenance` (6183); 4 lib unit tests + 2 BPF integration tests (`phase4_*`). Legacy claim-bound `Observation` remains (migration map §9 — dual path until Phase 5 evidence layers).

**Phase 5 delivered (2026-09-24):** `evidence_manifest.rs` module with `EvidenceManifest` (PDA seeds `["evidence_manifest", task_id, submitter, nonce]`) and `EvidenceArtifact` (PDA seeds `["evidence_artifact", manifest, artifact_index]`); instructions `submit_evidence_manifest` (permissionless; optional ObservationV2 link; pre-declared `root_hash`) and `add_evidence_artifact` (append-only index == `artifact_count`, cap `MAX_MANIFEST_ARTIFACTS=32`); artifact kinds PHOTO/DOCUMENT/VIDEO/MODEL/GEOMETRY/OTHER; per-artifact `source` + `provenance` reuse Phase 4 observation enums (richer provenance); events `EvidenceManifestSubmitted` / `EvidenceArtifactAdded`; errors `InvalidEvidenceArtifactKind` (6184), `EvidenceIndexMismatch` (6185), `EvidenceManifestFull` (6186); reused `EmptyContentHash` (6014), `EmptyStorageReference` (6134), `TaskAlreadyFinalized` (6174); 5 lib unit tests + 2 BPF integration tests (`phase5_*`). Legacy claim-bound `Evidence` remains (migration map §9 — single legacy evidence = one-artifact manifest during dual-write).

**Phase 6 delivered (2026-09-24):** `routing.rs` multi-factor dynamic routing per Design Rule 3 (Eligibility → Capability → Jurisdiction → Geo → Availability → Reputation → independence → randomized pick among eligible candidates). Instruction `route_task(task_id, req_index, candidates, chosen, competitor_count)` is requester-signed; no new PDAs — creates Phase 3 `TaskAssignment` for the random winner. Candidate eligibility PDAs are passed in `remaining_accounts` with fixed stride `3 + need_geo + need_cap` (profile, availability, reputation, [presence if radius_m>0], [capability if code≠CAPABILITY_ANY]). Entropy: `hashv(task_id ‖ task_pda ‖ slot)` + domain-separated draw (`draw_bps`); admission gate always passes when `competitor_count==1`. Jurisdiction filter is a soft-pass stub until Phase 10 cross-border bindings. Event `TaskRouted`; errors `ValidatorNotEligible` (6187), `NotRouteWinner` (6188), `TooManyRouteCandidates` (6189), `RouteAccountMismatch` (6190); 7 lib unit tests + 2 BPF integration tests (`phase6_*`); IDL 138/55/124/191.

**Phase 7 delivered (2026-09-24):** `fraud_governance.rs` implements reputation governance with **no jail** (user vision + Design Rules 13/14): progressive admin → `PEER_CONSENSUS` self-regulation, fraud confirmed by a random committee of well-reputed validators, punishment is a **capability demotion** (DECLARED level + PROBATIONARY tier — "just like a new validator"), never a binary ban. Accounts per §10: `FraudReport` (seeds `["fraud_report", accused, reporter, nonce]`), `ReviewCase` (`["review_case", report]`, reuse `["review_case", appeal]` for appeal reviews), `CapabilityRestriction` (`["capability_restriction", wallet, capability_code]`), `Appeal` (`["appeal", restriction, appellant, nonce]`). 9 instructions: `submit_fraud_report` (permissionless, self-report banned), `open_fraud_review` (permissionless open; committee = 5 distinct validators picked from `remaining_accounts` (profile, reputation) pairs with reputation ≥ 5000 bps, excluding the accused, via `hashv(report ‖ slot ‖ "terra_committee")`), `cast_fraud_vote` (committee-only, one vote each), `finalize_fraud_review` (all voted → majority; upheld creates ACTIVE `CapabilityRestriction` + clamps capability to DECLARED + tier to PROBATIONARY), `file_fraud_appeal` (after 24 h appeal window, restricted wallet only), `open_appeal_review` / `cast_appeal_vote` / `finalize_appeal_review` (fresh random committee excluding appellant; majority grant → LIFTED), `rehabilitate_restriction` (self-lift after 30-day clean window, no admin). `route_task` gained a Phase 7 gate: stride is now `3 + need_geo + 2*need_cap` — when a specific capability code is required, an optional `CapabilityRestriction` slot follows the capability slot; an ACTIVE restriction on (candidate, code) makes the candidate ineligible (`blocks_capability`). Events: `FraudReportSubmitted`, `FraudReviewOpened`, `FraudVoteCast`, `FraudReviewFinalized`, `CapabilityRestrictionApplied`, `FraudAppealFiled`, `AppealReviewOpened`, `FraudAppealDecided`, `CapabilityRestrictionLifted`. Errors `InvalidFraudReason` (6191) … `CapabilityRestricted` (6205); 15 lib unit tests + 6 BPF tests (`phase7_*`); IDL 147/59/133/206. Legacy `jail_validator`/`unjail_validator` (RFC-005) were removed in the RFC-012 legacy sweep (2026-09-25) — slashing and reputation auto-jail remain the only jailing paths.

**Phase 8 delivered (2026-09-24):** `task_economics.rs` implements §5 Economics / §7 steps 13–15: quote → task escrow → reward + coverage subsidy → refund. Accounts per §10: `FeePolicy` and `CoverageIncentive` (registry-admin-set config PDAs, seeds `["fee_policy"]` / `["coverage_incentive"]`, no signer — set via `registry.admin == admin` constraint), `ResourceQuote` (seeds `["resource_quote", task_id]`), `TaskEscrow` (seeds `["task_escrow", task_id]`, status 0=fresh/1=FUNDED/2=RELEASED/3=REFUNDED — the 0 marker keeps `init-if-needed` re-calls hitting custom errors instead of silent re-inits), `RewardAllocation` (seeds `["reward_allocation", task_id, validator]`, one per validator, `init-if-needed`); escrow vault PDA `["task_escrow_vault", escrow]` holds funds rent-exempt, fee sink is `task_treasury` PDA `["task_treasury"]`. 6 instructions: `set_fee_policy(fee_bps)` / `set_coverage_incentive(demand, deficit, difficulty, strategic, max_subsidy_bps)` (registry admin only, bounds-checked); `quote_task_resources` (requester; cost model = reward_lamports + resource estimate, stores `ResourceQuote`); `fund_task_escrow` (requester; requires `amount >= rent_min × required_validators`; `amount` moves to the vault, fee = `fee_bps × amount` moves to the treasury, fee waived when the treasury would be left sub-rent-exempt so dust fees can never freeze the treasury); `claim_task_reward` (assignee only; guards order: task COMPLETED 6213 → allocation fresh 6216 → assignment SUBMITTED 6214 → escrow FUNDED 6215; pays base from the vault + subsidy from the treasury capped at `treasury − rent_min`; the claim also creates the `RewardAllocation`); `refund_task_escrow` (requester; FAILED/EXPIRED/TIMEOUT → immediate full refund; COMPLETED after the 7-day claim window → unclaimed shares split among assignees by `refund_verdict`, otherwise 6217). Subsidy formula: `bps = demand + deficit + difficulty × min(required_validators, 4) + strategic (if class ∈ {PHYSICAL, DOCUMENTARY, HYBRID})`, clamped to `max_subsidy_bps`. Constants: `MAX_FEE_BPS=1000`, `MAX_COVERAGE_FACTOR_BPS=5000`, `MAX_SUBSIDY_BPS=5000`, `DIFFICULTY_FACTOR_CAP=4`, `CLAIM_WINDOW_SECS=7d`. Events: `FeePolicySet`, `CoverageIncentiveSet`, `TaskResourcesQuoted`, `TaskEscrowFunded`, `TaskRewardClaimed`, `TaskEscrowRefunded`. Errors `InvalidFeeBps` (6206) … `NothingToRefund` (6219), reusing `InvalidEscrowAmount` (6435), `NotTaskRequester` (6175), `NotTaskAssignee` (6176), `TaskAlreadyFinalized` (6174); 11 lib unit tests + 3 BPF tests (`phase8_*`: policies/quote/fund guards, claim-with-subsidy, refund paths); IDL 153/64/139/220. Rent rules: all transfers must leave accounts rent-exempt (Solana rejects accounts ending 0 < lamports < rent_min) — fund/claim enforce this on-chain, tests seed the treasury to exercise the fee-charged path and leave it empty for the waiver path.

**Immediate next actions for Phase 9:**
1. Read §9 (Migration Map) and §10 (PDA sketch) — Phase 8 delivered the economic layer (quote → escrow → rewards → coverage incentives → refunds) over Phase 3 task PDAs; Phase 9 is physical infrastructure (smartphone, GNSS, drones, survey devices, satellite imagery, 3D scanning) and should reuse Phase 4 observation sources/provenance.
2. Checked-in IDL was refreshed in A2 (119/44/108/160), Phase 2 (128/49/115/169), Phase 3 (134/52/120/182), Phase 4 (135/53/121/184), Phase 5 (137/55/123/187), Phase 6 (138/55/124/191), Phase 7 (147/59/133/206), and Phase 8 (153/64/139/220); re-run `make idl` after further program edits. Close remaining SECURITY.md items (RFC-005 reconfirm, ZK audit) if touching staking/ZK.
3. When adding accounts/instructions/events/errors: update `rfc012_structure.rs` expectations, root/`terra-core` README source counts, and run `make idl`.
4. Ship each phase with unit tests + BPF integration tests per §11.

**Phase order (next):** 9 (infrastructure) → 10 (cross-border/privacy). Do not skip the migration map dual-write rules in §9.

---

## 9. Migration Map from Current `dev`

| Current Entity | Target Entity | Migration Note |
|----------------|---------------|----------------|
| `ValidatorRegistry.validators: Vec<Pubkey>` | `Validator` accounts + registry index | Keep registry as index; move attributes to per-validator PDA |
| `ValidatorReputation` (single score) | `ValidatorReputation` multi-dimensional | Extend struct; keep old score as `legacy_score` for continuity |
| `Observation` (validator-bound) | `Observation` + `ObservationProvenance` + separate subject/capture/submitter | Add role fields; backfill with validator as all roles |
| `Claim` | `VerificationTask` (new) + `Claim` (legacy subject) | Claims become task subjects; new tasks reference claims |
| `Evidence` | `Evidence` + `EvidenceManifest` + `EvidenceArtifact` | Manifest groups artifacts; single evidence becomes one-artifact manifest |
| `Parcel.owner: Pubkey` | `Rights` PDA `["ownership", parcel]` (`rights_kind == OWNERSHIP`, field `holder`) | **Done on `dev`** — field hard-removed; `is_authorized_holder()` resolves wallet **or** Identity-PDA holders; no dual-write remains (bridge closed) |
| `Attestation` (deprecated) | `VerificationAttestation` | **Bridge closed (2026-09-25):** `Attestation` account + `attest` / `rotate_validators` / `register_document` / `migrate_attestation_to_claim` / `migrate_attestations` hard-removed; claim pipeline only |
| `Jurisdiction` | `Jurisdiction` + `CrossBorderBinding` | Extend with binding relationships |
| `EscrowRecord` (parcel-only) | `TaskEscrow` + parcel escrow | Generalize to any task subject |

---

## 10. Account / PDA Sketch (Phase 2+)

```
Rights (ownership)          seeds: ["ownership", parcel]  (canonical parcel holder; field `holder`)
Rights (sub-rights)         seeds: ["rights", parcel, nonce]

ValidatorProfile            seeds: ["validator_profile", wallet]
ValidatorPresence           seeds: ["validator_presence", wallet]
ValidatorAvailability       seeds: ["validator_availability", wallet]
ValidatorCapability         seeds: ["validator_capability", wallet, capability_code]
ValidatorRelationshipEdge   seeds: ["validator_edge", from, to, edge_type]

VerificationTask            seeds: ["task", task_id]
TaskRequirement             seeds: ["task_requirement", task_id, req_index]
TaskAssignment              seeds: ["task_assignment", task_id, validator]
TaskEscrow                  seeds: ["task_escrow", task_id]
TaskEscrow vault            seeds: ["task_escrow_vault", escrow]
FeePolicy                   seeds: ["fee_policy"]  (registry admin)
CoverageIncentive           seeds: ["coverage_incentive"]  (registry admin)
ResourceQuote               seeds: ["resource_quote", task_id]
RewardAllocation            seeds: ["reward_allocation", task_id, validator]
task_treasury               seeds: ["task_treasury"]  (fee sink)

ObservationV2               seeds: ["observation_v2", task_id, observer, nonce]
EvidenceManifest            seeds: ["evidence_manifest", task_id, nonce]
EvidenceArtifact            seeds: ["evidence_artifact", manifest, artifact_index]

ReviewCase                  seeds: ["review_case", report]  (also ["review_case", appeal])
FraudReport                 seeds: ["fraud_report", accused, reporter, nonce]
CapabilityRestriction       seeds: ["capability_restriction", wallet, capability_code]
Appeal                      seeds: ["appeal", restriction, appellant, nonce]
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
