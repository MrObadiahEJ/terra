# REQUIREMENTS_TRACEABILITY — Vision → Requirement → Code → Test

Chain enforced for every requirement:

**VISION → INTENT → REQUIREMENT → DESIGN/RFC → SOURCE → TEST → (SECURITY REVIEW) → DEPLOYMENT**

Priority: P0 = protocol-critical · P1 = core value · P2 = planned capability.
Status uses the 7-level vocabulary ([BOOK_OF_INTENTS.md](BOOK_OF_INTENTS.md)).

## Overview matrix

| ID | Requirement | Priority | Status | Design | Source | Test |
| --- | --- | --- | --- | --- | --- | --- |
| REQ-001 | No state without the full evidence chain | P0 | 1 | RFC-012 | `observation_v2.rs`, `evidence_manifest.rs`, `verification/*`, `quorum.rs` | BPF stage3 evidence/quorum suites; vitest activity-store + GeoJSON-parse suites (sprint 022) |
| REQ-002 | No unilateral ownership conversion | P0 | 1 | RFC-012 | `transfer_parcel` owner guards; rights-as-PDA (migration 0025) | BPF transfer/rights suites |
| REQ-003 | Country-agnostic operation | P0 | 1 | RFC-012 | `world_registry.rs` (249 ISO codes; genesis ≥3 countries) | BPF genesis suites |
| REQ-004 | Verifiable spatial history (append-only versions) | P0 | 1 | RFC-013 | `spatial_asset.rs` `append_geometry_version`, `verify_geometry_version` | `stage3_spatial_asset_lifecycle`, `stage3_geometry_version_verification` |
| REQ-005 | Canonical geometry commitment (representation, not reality) | P0 | 1 | RFC-013 §9 | `terra-geom-v1` digest (commit `775e20a`), `docs/canonical-geometry-digest.md` | geo digest tests |
| REQ-006 | Anti-fraud validator selection (random, independent, capable) | P0 | 1 | RFC-012 Ph.6 | `routing.rs` `route_task` (nine-factor, ≤8, seeded randomness) | BPF routing suites |
| REQ-007 | Proportional slashing with due process | P0 | 1 | RFC-005 | `staking.rs` tiers (10%/100%), 24-h review, `dispute_slashing` | BPF staking suites |
| REQ-008 | Dispute due process (freeze, notice windows, adjudication, appeal) | P0 | 1 | RFC-007 | `dispute.rs` lifecycle; migration 0009 | BPF dispute + CDP settlement 33/33 |
| REQ-009 | Escrow safety (state machine, windows, mutual cancel) | P1 | 1 | RFC-004 | `escrow.rs` 8 instructions | BPF escrow suites + CDP smoke |
| REQ-010 | Cross-jurisdiction identity with expiry/revocation | P1 | 1 | RFC-006 | `cross_border.rs`; migrations 0012/0020 | BPF cross-border suites |
| REQ-011 | Recoverability (guardianship, succession, emergency injection) | P1 | 1 | RFC-010 | `recovery.rs`; identity `guardianship.rs`/`succession.rs` | BPF recovery/identity suites |
| REQ-012 | Selective disclosure for ownership claims | P1 | 2 | RFC-011 | `zk/groth16.rs`, VK pinning (0027) | on-chain verify path exists; ceremony tooling + live validation missing |
| REQ-013 | GIS interoperability (PostGIS mirror as read model) | P1 | 3 | RFC-012; migration 0017 | `api/routes/*` (25 routers), migrations 0004/0017/0030 | API suite; mirror hash-anchoring of every row not implemented |
| REQ-014 | Replayable spatial narratives (deterministic stories) | P1 | 1 | RFC-014 Ph.01 | `story/timeline.ts`, `routes/stories.rs`, migration 0032 | vitest 44/44; API story tests |
| REQ-015 | Journey navigation with honest method labels | P1 | 1 | RFC-014 Ph.02 | `story/roads.ts` (client-side); method `road-graph`/`great-circle` | vitest roads suite |
| REQ-016 | Version provenance in stories (history narratives) | P1 | 1 | RFC-014 Ph.02 | story version stops ↔ `/spatial-accounts` | vitest + CDP smokes |
| REQ-017 | Client-side video export with receipts | P2 | 1 | RFC-014 Ph.03 | `story/export.ts`; export receipts table (0032) | vitest export-adjacent suites; manual CDP |
| REQ-018 | External machine-readable replay (robotics/API consumers) | P2 | 1 | RFC-014 Ph.05 | `routes/stories.rs` `/replay`, `/replay.geojson`, `/schema/replay`; `terra.story-replay/v1` | API 121/121; live smoke (hash tracks edits; 404s) |
| REQ-019 | Incremental derived compute (dedup jobs) | P2 | 1 | RFC-014 Ph.04 | `routes/derived.rs`; migration 0033 (UNIQUE processing_key); recursive-CTE invalidation | API 117/117 (7 derived tests); live smoke 16/16; 2 identical computations → 1 ledger row |
| REQ-020 | Institutional identity with scoped authority | P1 | 4 – SPECIFIED ONLY | directive §8 | — (spec in `chapters/08-institutional-identity.md`, `intents/INT-005-institutional-identity.md`) | — |
| REQ-021 | Residency/proof-of-location credential | P2 | 4 – SPECIFIED ONLY | directive §8 | — (spec in `chapters/02-proof-of-location-residency.md`; building blocks REQ-012/A15) | — |
| REQ-022 | Lien encumbrance lifecycle | P1 | 4 – SPECIFIED ONLY | directive §3E | — (spec in `chapters/06-lien-encumbrance.md`; building blocks REQ-002, REQ-008) | — |
| REQ-023 | Fractional participation with escrowed guarantees | P2 | 6 – RESEARCH REQUIRED | directive §10 | building block REQ-009 | legal research gate (Q-005) |
| REQ-024 | AI outputs distinguishable from verified results | P1 | 1 | directive §11 | migration 0034: drafts record `provenance_kind` + model provider/name/version/confidence; API validation fail-closed | API 128/128 (7 provenance tests); live smoke 13/13; DB constraints enforced |
| REQ-025 | API runtime defense-in-depth (headers + body limit + timeout) | P2 | 1 | — (operational hardening, sprints 012/013/015) | `api/src/main.rs` (`security_headers`, `RequestBodyLimitLayer`, `TimeoutLayer`), `api/src/config.rs` (`MAX_BODY_BYTES`, `REQUEST_TIMEOUT_SECS`) | API 133/133 (`tests::ok_response_carries_security_headers`, `tests::within_limit_is_accepted`, `tests::oversized_content_length_rejected_with_413`, `tests::slow_request_times_out_with_408`) |
| REQ-026 | Distributed spatial component package & assembly | P1 | 2 – IMPLEMENTED, NOT FULLY VERIFIED | Chapter 11; RFC-015 | `geo-engine/src/component_package.rs` (conformance validator + `terra-geom-v2` digest); `api/migrations/0035_component_packages.sql`, `0041_component_version.sql`; `api/src/routes/components.rs` (`POST /components`, `GET /components/{id}`, `PUT/GET /components/{id}/artifact`, conformance-validation middleware); `api/src/routes/assemblies.rs` (`POST /assemblies`, `GET /assemblies/{id}`, conflict detection); `api/src/storage.rs` (`store_content_addressed`); `api/migrations/0045_component_artifacts.sql`; `terra-web/src/lib/runtime.ts` (`fetchComponentArtifact` data-plane fallback + digest re-verification) | API **147/147** (2026-11-13, incl. 2 DB-backed artifact roundtrip + 2 storage unit tests); frontend vitest **86/86**; no live-cluster component submission yet (B6 blocked) |
| REQ-027 | World Representation Manifest & central coordination | P1 | 2 – IMPLEMENTED, NOT FULLY VERIFIED | Chapter 12 | `api/migrations/0036_world_manifests.sql` + `0044_signed_manifests.sql`; `api/src/routes/manifests.rs` (`POST /manifests`, `GET /manifests/{id}`, `PUT /manifests/{id}/objects`, `POST /manifests/verify`, `POST /manifests/signed`, `GET /manifests/signed`, `GET /manifests/signed/{address}`); `geo-engine/src/spatial_index.rs` (quadtree discovery cells, ADR-020); `geo-engine/src/manifest.rs` (content-addressed Ed25519-signed manifest payloads); `api/src/routes/spatial.rs` (`GET /spatial/cell`, `GET /spatial/cover`); `terra-web/src/lib/runtime.ts` (manifest resolver + scene graph manager) | API full suite **147/147** (2026-11-13); `terra-geo` 48/48 (+2 ignored); signed envelopes persist, list by region/exact cell, and retrieve by digest (ADR-021); mutable CRUD rows and external component storage remain; scene requests do not use cells; no live-cluster manifest version transition yet (B6 blocked) |
| REQ-028 | Progressive delivery & runtime discovery | P1 | 2 – IMPLEMENTED, NOT FULLY VERIFIED | Chapter 13 | `api/migrations/0037_storage_providers.sql`, `0039_runtime_scene_graphs.sql`, `0040_worker_registration.sql`, `0042_derived_job_leases.sql`, `0043_scene_node_validation_state.sql`; `api/src/routes/providers.rs` (provider discovery + fallback); `api/src/routes/runtime.rs` (`/runtime/scene`, `/runtime/assets`, `/runtime/lod`); `api/src/routes/spatial.rs` (`GET /spatial/cell`, `GET /spatial/cover`); `api/src/routes/workers.rs` (worker registration + assignment); `terra-web/src/lib/runtime.ts` (LOD 0–4, provider fallback, asset retrieval, `fetchComponentArtifact` data-plane fallback, `getQuadCell`/`getQuadCover` quadtree discovery client); `terra-web/src/lib/viewport.ts` (screen-space-error LOD + frustum/occlusion culling) | API **147/147**; frontend runtime **115/115**; no live-cluster multi-provider retrieval yet (B6 blocked) |
| REQ-029 | No single point of control — no mandatory central operator for protocol operation | P0 | 2 – IMPLEMENTED, NOT FULLY VERIFIED | RFC-016 §1.1 | `src/security.rs` (SecurityRequest PDAs + instructions); `docs/rfc-016-protocol-security-decentralization-privacy.md` | `security.rs` compiles; no BPF tests yet |
| REQ-030 | Protocol/application separation — applications/marketplaces acquire no protocol-level authority | P0 | 2 – IMPLEMENTED, NOT FULLY VERIFIED | RFC-016 §1.2 | `src/security.rs` (on-chain enforces process; off-chain implements enforcement) | `security.rs` compiles; no BPF tests yet |
| REQ-031 | No universal shutdown mechanism — emergency controls narrowly scoped, time-limited, auditable | P0 | 2 – IMPLEMENTED, NOT FULLY VERIFIED | RFC-016 §1.3 | `src/security.rs` (EmergencyProtection PDA + `emergency_protect` instruction with expires_at) | `security.rs` compiles; no BPF tests yet |
| REQ-032 | Measurable decentralization — 12 categories with concentration-risk indicators | P0 | 2 – IMPLEMENTED, NOT FULLY VERIFIED | RFC-016 §2 | `src/security.rs` (random reviewer assignment, reputation-gated); `validator_registry.rs` (ValidatorRelationshipEdge) | `security.rs` compiles; no BPF tests yet |
| REQ-033 | Distributed validator review for privacy/security — local+global, verifiable randomization, anti-collusion | P0 | 2 – IMPLEMENTED, NOT FULLY VERIFIED | RFC-016 §3.2 | `src/security.rs` (SecurityReviewCase, SecurityReviewAssignment PDAs + instructions); reuses `validator_profile.rs` reputation gating | `security.rs` compiles; no BPF tests yet |
| REQ-034 | Privacy request lifecycle — authentication → scope → classification → emergency → distributed review → quorum → appeal → finalized | P0 | 2 – IMPLEMENTED, NOT FULLY VERIFIED | RFC-016 §4 | `src/security.rs` (PrivacyRequest PDA + full lifecycle instructions) | `security.rs` compiles; no BPF tests yet |
| REQ-035 | Privacy state versioning + remedies — `PUBLIC`…`RESTORED`; restrict/redact/encrypt/revoke/tombstone/correct/reject | P0 | 2 – IMPLEMENTED, NOT FULLY VERIFIED | RFC-016 §4.7 | `src/security.rs` (PrivacyState PDA + remedy_type enum) | `security.rs` compiles; no BPF tests yet |
| REQ-036 | Scoped jurisdictional restrictions — authenticated, scoped, expiry/review, challengeable, not auto-global | P0 | 2 – IMPLEMENTED, NOT FULLY VERIFIED | RFC-016 §5 | `src/security.rs` (JurisdictionalRestrictionRequest PDA + `submit_jurisdictional_restriction` instruction) | `security.rs` compiles; no BPF tests yet |
| REQ-037 | Security incentives & slashing with due process — rewards not outcome-dependent; slashing requires provable violation | P0 | 2 – IMPLEMENTED, NOT FULLY VERIFIED | RFC-016 §6 | Reuses `task_economics.rs` escrow/reward + `staking.rs` slashing infrastructure | `staking.rs`/`task_economics.rs` compile; BPF staking suites exist |
| REQ-038 | Storage independence & censorship resilience — multi-provider, replication, fallback, recovery | P1 | 2 – IMPLEMENTED, NOT FULLY VERIFIED | RFC-016 §7 | `src/routes/providers.rs` (multi-provider storage, fallback, replication) | API 133/133; provider tests |
| REQ-039 | On-chain enforces decision process; off-chain implements enforcement; AI must not decide privacy disputes | P0 | 2 – IMPLEMENTED, NOT FULLY VERIFIED | RFC-016 §8 | `src/security.rs` (on-chain decision finalization); `src/routes/reconstruction.rs` (AI provenance fields, fail-closed validation) | API 128/128 (7 provenance tests); `security.rs` compiles |
| REQ-040 | Abuse prevention — proportionality, public-interest evidence preservation, appeals/restoration, privacy-not-censorship | P0 | 2 – IMPLEMENTED, NOT FULLY VERIFIED | RFC-016 §9 | `src/security.rs` (appeal mechanism, proportional remedy selection) | `security.rs` compiles; no BPF tests yet |

## Orphan / drift checks (per directive §14)

- **Implemented without a requirement:** settlement center UI (covered under
  REQ-008/009 as evidence surfaces — no orphan). `zk-membership` crate: verify
  its consumers before release (flagged Q-009).
- **Requirements with no implementation:** REQ-020…REQ-022 are `SPECIFIED ONLY`
  (chapters + intents + acceptance criteria written; ⚠ founder review flags
  pending on Q-003/Q-004). REQ-023 is `RESEARCH REQUIRED` (Q-005 hard gate).
  REQ-026…REQ-028 are `IMPLEMENTED, NOT FULLY VERIFIED` (routes + migrations +
  geo-engine validator + frontend runtime client all written and pass local
  suites; not yet exercised against a live cluster — B6 devnet funding blocks
  that). REQ-029…REQ-040 are `IMPLEMENTED, NOT FULLY VERIFIED`
  (on-chain PDAs + instructions in `src/security.rs`; no BPF tests yet, not
  deployed to devnet). All intentional; tracked in ROADMAP Stages 3–4.5.
- **Vision elements at risk of drift:** AI provenance (REQ-024) is now
  recorded on drafts (R-014 resolved); future model outputs must use those
  fields, never free-text `method` alone.

## Acceptance-criteria rule

Any status transition to `IMPLEMENTED AND VERIFIED` requires: (a) source path,
(b) a named passing test, (c) CHANGELOG entry. Agents update this matrix in the
same change that moves the code.
