# Sprint 042 — P0 Security & Trust-Chain Fixes (REQ-026/026a/027/028/029–040)

**Date:** 2026-11-13 · **Status:** done, committed (`7dbe27a`→this commit)
**Gate:** `cargo check -p terra-registry -p terra-api` clean; `cargo clippy -p terra-registry -p terra-api -- -D warnings` clean; `cargo fmt --check` clean; frontend `tsc` clean; vitest **120/120**; eslint no new errors on changed files; `pnpm build:check` PASS.

## Trigger

Evidence-based audit of `dev @ 722a7ab` against the Book of Intents and `TERRA_PROTOCOL_DIRECTIVE.md` exposed critical trust-chain gaps: the system trusted *declarations* at points where the protocol requires *evidence*. The audit found systemic bugs in the RFC-016 on-chain security core (REQ-029–040) and the RFC-015 component pipeline (REQ-026). These are P0 because they make core protocol guarantees unenforceable.

## P0 Fixes — Backend API (`terra-core/api/`)

### components.rs — `submit_component` trust-state self-declaration (REQ-026, B1)
- **Bug:** `validation_state` was bound from `pkg.validation_state` — submitters could self-declare `independently_valid`, bypassing validator attestation.
- **Fix:** Force `validation_state = 'pending'` server-side; submitters cannot self-declare trust. Validator-gated transition remains future work.
- **File:** `api/src/routes/components.rs` (line ~152)

### components.rs — Artifact upload body-hash replay (REQ-026a, C6)
- **Bug:** `SignedRequest` signs `{method}\n{path}\n{body_hash}\n{timestamp}`, but no handler recomputed SHA-256 of the actual body. A captured signature could be replayed with a swapped body within the ±60s window.
- **Fix:** `upload_component_artifact` now computes `SHA-256(body)` and rejects if it doesn't match `auth.body_hash_hex`.
- **File:** `api/src/routes/components.rs` (+ `sha2` import)

### components.rs — Data-plane artifact retrieval (REQ-026a, C4)
- **Pre-existing gap:** `GET /components/{id}/artifact` route parses `Path<Uuid>` but the frontend only knows the TEXT `component_id`. The data-plane fallback in `fetchComponentAsset` would 400 on every call.
- **Status:** Known gap documented; requires exposing the package UUID through scene nodes (future work).

## P0 Fixes — On-chain Security Core (`terra-core/programs/terra_registry/src/security.rs`, REQ-029–040)

### A1 — PDA seed circular reference (CreateReviewCase, FinalizeDecision)
- **Bug:** `seeds = [b"security_request", request.key().as_ref()]` — derives PDA from the account's own key, which is circular and can never match. The `create_review_case` instruction failed on every legitimate request, making the entire distributed-review lifecycle dead at step one.
- **Fix:** Changed to `seeds = [b"security_request", requester.key().as_ref(), nonce.to_le_bytes().as_ref()]`, matching the `SubmitSecurityRequest` PDA derivation. Added `#[instruction(nonce: u16, requester: Pubkey)]` to `FinalizeDecision` (was missing nonce entirely).
- **File:** `programs/terra_registry/src/security.rs` + `lib.rs` dispatcher

### A3 — `SecurityRequest.subject = Pubkey::default()` forever
- **Bug:** `submit_security_request` sets `sr.subject = Pubkey::default()` with comment "filled by specific request types" — but no instruction ever fills it.
- **Status:** Documented as known gap; filling it requires adding a `subject` parameter to `submit_security_request` (future work).

### A4 — Reviewer assignment lacks randomization (REQ-033)
- **Bug:** `assign_reviewer` checks only `reputation_score >= 5000`; `authority: Signer` is anyone — the requester can hand-pick 5 colluding validators.
- **Status:** Documented as known gap; VRF/slot-seed randomization is future work.

### A5 — `finalize_appeal` permissionless (REQ-040)
- **Bug:** `authority: Signer` (anyone), no check that `appeal.decision == decision.key()`. Anyone could overturn any privacy decision.
- **Fix:** Added `ValidatorReputation` account constraint (reputation-gated, matching `assign_reviewer` pattern); added `require!(appeal.decision == decision.key())`.
- **File:** `programs/terra_registry/src/security.rs` + `lib.rs` (`AppealDecisionMismatch` error)

### A6 — `emergency_protect` censorship gadget (REQ-040)
- **Bug:** Any signer, any subject, 7-day protection renewable forever, **no lift instruction exists**.
- **Fix:** Added `emergency_lift` instruction with `close = requester` — closes the protection PDA and returns rent. Requires the original requester, checks expiry, sets `lifted = true`.
- **File:** `programs/terra_registry/src/security.rs` (+ `EmergencyProtectionLifted` event)

### A2 — Security tests never run in CI
- **Bug:** `tests/security.rs` (620-line ProgramTest suite) existed but CI only ran `--test integration` (307 suite) and `--lib`. The security tests never executed.
- **Fix:** Added `cargo test -p terra-registry --test security` to `.github/workflows/ci.yml` `lib-tests` job.

## Verification

| Check | Result |
| --- | --- |
| `cargo check -p terra-registry -p terra-api` | clean |
| `cargo clippy -p terra-registry -p terra-api -- -D warnings` | clean |
| `cargo fmt --check` | clean |
| Frontend `tsc -b --noEmit` | clean |
| Frontend `vitest run` | **120/120** |
| Frontend `eslint` (changed files) | no new errors |
| `pnpm build:check` | PASS (entry 300.2 / initial 305.5 / Cesium 1098.5 / total 1868.7 kB gz) |

## Known gaps (not fixed in this sprint)

- `SecurityRequest.subject` remains `Pubkey::default()` — no instruction fills it
- `geometry_digest` never verified against artifact bytes (`validate_with_geometry` not called)
- `POST /components` has no auth at all (anonymous submission)
- Worker `ON CONFLICT (worker_id) DO UPDATE SET wallet_address` — identity takeover
- Runtime uses unsigned mutable manifests instead of signed-manifest path
- Client never verifies fetched bytes against signed manifest content address

## Book updates

`IMPLEMENTATION_STATUS.md` (A37 row: +1 instruction, fixes documented, CI test added; B15 row: validation_state fix, body-hash fix), `REQUIREMENTS_TRACEABILITY.md` (REQ-029–040 rows: CI test added, fixes documented, known gaps noted), `CHANGELOG.md`, this note.
