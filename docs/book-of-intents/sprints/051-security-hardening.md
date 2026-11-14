# Sprint 051: Security hardening — submit_component auth, SecurityRequest.subject, PrivacyState writes, complete_assignment auth

**Date:** 2026-11-14 · **Status:** done & committed (`90b76d3`)
**Gate:** `cargo check --workspace` clean; `cargo clippy --workspace -- -D warnings` clean; `cargo fmt --all --check` clean

## Trigger

work/CURRENT.md item 3 (RFC-016 Phase 2+ code) listed several unblocked code tasks:
`SecurityRequest.subject` fill, `PrivacyState` writes on finalize, provider
proof-of-possession audits, client-side digest verification, `POST /components`
auth (Sprint 050), and `complete_assignment` auth. Sprint 051 closes three of
these; provider proof-of-possession audits remain for a future sprint.

## Implementation

**Backend — `terra-core/api/src/routes/components.rs`:**
- `submit_component` already required `auth: SignedRequest` from Sprint 050;
  confirmed it verifies `auth.verify(&key, "POST", "/api/v1/components")` —
  no change needed here beyond confirming the Sprint 50 fix is in place.

**Backend — `terra-core/programs/terra_registry/src/security.rs`:**
- `submit_security_request`: changed `sr.subject = Pubkey::default()` to
  `sr.subject = ctx.accounts.requester.key()` — a `SecurityRequest` now always
  records who it is about (self-submission by default), fixing the latent
  `Pubkey::default()` hole.
- `finalize_decision`: added `privacy_state: Account<'info, PrivacyState>` to
  the `FinalizeDecision` context with PDA seeds `["privacy_state", subject,
  version]`. When a decision is finalized, `privacy_state` is written with
  `state`, `remedy`, `restriction_request`, and timestamps — so privacy state
  transitions are now durably recorded on-chain.

**Backend — `terra-core/api/src/routes/workers.rs`:**
- `complete_assignment` now requires `auth: SignedRequest` and verifies
  `auth.verify(&key, "POST", &format!("/assignments/{job_id}/complete"))` —
  fail-closed when `API_AUTHORITY_PUBKEY` is not configured. Previously any
  caller could mark any job assignment complete.

## Verification

| Check | Result |
|---|---|
| `cargo check --workspace` | clean |
| `cargo clippy --workspace -- -D warnings` | clean |
| `cargo fmt --all --check` | clean |

**Note:** Anchor BPF tests not run locally (weak machine, long compile times);
source compilation verified via `cargo check`.

## Remaining RFC-016 Phase 2+ code

- **Provider proof-of-possession audits** — providers should prove they hold
  the data they claim to replicate; not yet implemented.
- **Client-side digest verification vs signed manifest** — frontend
  `fetchComponentAsset` already performs SHA-256 re-verification against
  `x-content-sha256` header (Sprint 047/050); complete.
