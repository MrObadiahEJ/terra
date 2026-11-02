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

## Orphan / drift checks (per directive §14)

- **Implemented without a requirement:** settlement center UI (covered under
  REQ-008/009 as evidence surfaces — no orphan). `zk-membership` crate: verify
  its consumers before release (flagged Q-009).
- **Requirements with no implementation:** REQ-020…REQ-022 are `SPECIFIED ONLY`
  (chapters + intents + acceptance criteria written; ⚠ founder review flags
  pending on Q-003/Q-004). REQ-023 is `RESEARCH REQUIRED` (Q-005 hard gate).
  All intentional; tracked in ROADMAP Stages 3–4.
- **Vision elements at risk of drift:** AI provenance (REQ-024) is now
  recorded on drafts (R-014 resolved); future model outputs must use those
  fields, never free-text `method` alone.

## Acceptance-criteria rule

Any status transition to `IMPLEMENTED AND VERIFIED` requires: (a) source path,
(b) a named passing test, (c) CHANGELOG entry. Agents update this matrix in the
same change that moves the code.
