# Sprint 050: Fix submit_component auth (REQ-026/B15)

**Date:** 2026-11-14 · **Status:** done & committed (`da4d219`)
**Gate:** `cargo check --workspace` clean; `cargo clippy --workspace -- -D warnings` clean; `cargo fmt --all --check` clean

## Trigger

work/CURRENT.md item 3 (RFC-016 Phase 2+ code) lists `POST /components` auth as an unblocked work item. The `submit_component` handler had no authentication — any caller could submit a component package without signing. RFC-015 requires that component submissions be authenticated and validated before insert.

## Implementation

**Backend — `terra-core/api/src/routes/components.rs`:**
- Changed `submit_component` signature from `State(state), Json(input)` to `State(state), auth: SignedRequest, Json(input)`
- Added `auth.verify(&key, "POST", "/api/v1/components")` — verifies the Ed25519 signature against `API_AUTHORITY_PUBKEY` before insert. Fail-closed when authority is not configured.
- Added `signed_submit_request()` test helper that creates a properly signed `SignedRequest` (SHA-256 of JSON body + Ed25519 signature over `POST\n/api/v1/components\n{body_hash}\n{timestamp}`)
- Updated all 3 `submit_component` test call sites to pass `signed_submit_request(&pkg)` and use `ApiAuthority(Some(key))`

**Book of Intents:**
- `IMPLEMENTATION_STATUS.md` B15 row: notes `validation_state` forced to `'pending'`, body-hash verification, and `submit_component` now requires signed auth

## Verification

| Check | Result |
|---|---|
| `cargo check --workspace` | clean |
| `cargo clippy --workspace -- -D warnings` | clean |
| `cargo fmt --all --check` | clean |

**Note:** Anchor BPF tests not run locally (weak machine, long compile times); source compilation verified via `cargo check`.

## Remaining RFC-016 Phase 2+ code

- `SecurityRequest.subject` still `Pubkey::default()` (no instruction fills it)
- `PrivacyState` writes on finalize
- Provider proof-of-possession audits
- Client-side digest verification vs signed manifest (frontend: `fetchComponentAsset` already wired)
- `complete_assignment` auth

These remaining items are backend (Rust) work requiring heavier builds; the frontend data-plane artifact path is already wired and verified.
