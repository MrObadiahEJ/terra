# Sprint 004 — Request authentication hardening (R-016/R-017)

**Loop:** Research → Understand → Compare → Identify Gaps → Implement → Validate → Record
**Phase derived from evidence:** Test (validated) → Loop (this record)

## Research

- `SignedRequest` (auth.rs) parses `Authorization`/`x-timestamp`/`x-body-hash`
  and enforces the ±60 s replay window in `FromRequestParts`, but verification
  against a key happens only when a handler calls `verify`/`verify_wallet`.
- Phase 04 (`derived.rs`) and reconstruction handlers extracted `SignedRequest`
  then discarded it (`let _ = auth;` / `_auth`), so any caller could complete,
  fail, lease, or invalidate jobs. Recorded as R-016 (H).
- No reusable signing client existed in the repo: the web app signs inline for
  wallet binding, and live smokes hand-rolled Ed25519 in `/tmp` (R-017).

## Plan

- Privileged handlers verify the Ed25519 signature against
  `API_AUTHORITY_PUBKEY` (fail-closed when unset), path-bound to the exact
  route. Wallet binding stays separate (`verify_wallet`) where a caller
  identity is recorded.
- Ship `terra-core/api/examples/sign_request.rs`: computes body hash, timestamp
  and signature, prints the four headers. No new dependencies.
- Validate live with a dev authority key: authority key ⇒ 2xx; wrong key ⇒
  401; no headers ⇒ 401. Re-run the Phase 04 logic smoke under verification.

## Build

- `derived.rs`: `complete_job`, `fail_job`, `invalidate_job` verify against
  `state.api_authority`. `reconstruction.rs`: `create_reconstruction_job`,
  `lease_reconstruction_job`, `job_progress`, `job_complete`,
  `draft_anchor_confirmed` likewise; lease additionally binds `x-wallet` when
  supplied.
- `api/examples/sign_request.rs` + AGENT_GUIDE §6 signing contract.
- Registers: RISK R-016/R-017 annotated applied; CURRENT.md; this sprint.

## Test (evidence)

- API suite: **117/117** (unchanged; no handler contract regressions).
- Client self-check: emitted `x-wallet` equals the requested dev pubkey and
  headers produced a successful signed `curl` create/complete.
- Negative auth: unsigned → **401**, wrong signer → **401**.
- Phase 04 logic smoke re-run signed with the authority key → **16/16**.
- Note: local API without `API_AUTHORITY_PUBKEY` now rejects privileged
  endpoints by design; dev launches set the key explicitly.

## Loop

- R-016/R-017 closed with live evidence. Next: RFC-014 Phase 05 (REQ-018),
  then the verification refresh for non-API cargo baselines.
