# Sprint 026 — RFC-016 On-Chain Security/Privacy Governance Core

**Date:** 2026-11-02/03 · **Status:** done & committed · **Gate:** on-chain program
compiles; **no BPF tests yet, not deployed to devnet** (B6 blocks both)

**Branch:** `dev` · **Commits:** `748f669`, `c05af79`, `8e296aa`

## What

Implement the on-chain security/privacy governance core per RFC-016
(REQ-029…REQ-040). The spec was written first (commit `748f669`); the code
followed immediately after.

### PDAs (11)

| PDA | Purpose |
|---|---|
| `SecurityRequest` | Authenticated security request lifecycle |
| `PrivacyRequest` | Privacy request: authentication → scope → classification → emergency check → distributed review → quorum → appeal → finalized |
| `JurisdictionalRestrictionRequest` | Scoped jurisdictional restriction (authenticated, scoped, expiry/review, challengeable, not auto-global) |
| `PrivacyState` | Versioned privacy state: `PUBLIC` / `RESTRICTED` / `PRIVATE` / `REDACTED` / `TEMPORARILY_PROTECTED` / `UNDER_REVIEW` / `WITHDRAWN_FROM_PUBLIC_ACCESS` / `TOMBSTONED` / `RESTORED` |
| `SecurityReviewCase` | Distributed validator review case |
| `SecurityReviewAssignment` | Per-reviewer assignment with reputation gating |
| `SecurityDecision` | Finalized decision |
| `SecurityAppeal` | Appeal registration + review |
| `SecurityAuditEvent` | Immutable audit trail |
| `EmergencyProtection` | Time-boxed emergency protection (`expires_at`) |

### Instructions (11)

`submit_security_request`, `submit_privacy_request`,
`submit_jurisdictional_restriction`, `create_review_case`, `assign_reviewer`,
`cast_review_vote`, `finalize_decision`, `appeal_decision`, `finalize_appeal`,
`record_audit_event`, `emergency_protect`.

### Other

- 13 new `TerraError` codes for security/privacy failure modes.
- Security module registered in `lib.rs`.
- `IMPLEMENTATION_STATUS.md` A37–A40 + `REQUIREMENTS_TRACEABILITY.md`
  REQ-029–REQ-040 updated to `IMPLEMENTED, NOT FULLY VERIFIED`.

## Verification

- `cargo build -p terra-registry` compiles clean (on-chain program).
- `cargo test -p terra-api` **133/133** (unaffected — no application changes).
- `cargo test --lib` (geo-engine) **33 (+2 ignored)**.

## Status after this sprint

REQ-029–REQ-040 → `IMPLEMENTED, NOT FULLY VERIFIED`. The on-chain core is
written and compiles, but:

- **No BPF tests exist yet** — `tests/security.rs` is created but empty.
- **Not deployed to devnet** — B6 devnet funding (Q-001) blocks both.

RFC-016 Phase 2 (on-chain core) is complete. Phases 3 (off-chain privacy
enforcement), 4 (incentives + independent participation), and 5 (censorship +
recovery testing) are **not started** — A35/A36 remain `SPECIFIED ONLY`.

## Still open after this sprint

- BPF test suite for `src/security.rs` (all 11 instructions, all state
  transitions, appeal path, emergency expiry).
- Devnet deployment + on-chain smoke.
- Off-chain privacy enforcement middleware (A35/A36).
- Security task rewards + reviewer compensation design.
- Censorship/recovery test scenarios (RFC-016 Phase 5).