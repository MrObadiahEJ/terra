# Sprint 003 — RFC-014 Phase 04: incremental derived compute (REQ-019)

**Loop:** Research → Understand → Compare → Identify Gaps → Implement → Validate → Record
**Phase derived from evidence:** Test (validated) → Loop (this record)

## Research

- Prior phases (01–03) landed the story engine; Phase 04 is the backend
  half: compute expensive deterministic results once and reuse them, instead
  of recomputing identical inputs (REQ-019, ROADMAP Stage 1 item 2).
- Studied existing patterns: migration style (`0031_reconstruction_worker`),
  `SignedRequest` auth (extractor parses; handlers verify), reconstruction
  `job_complete` flow (draft → anchor prep), unit-only API test suite.

## Plan

- `derived_jobs` keyed by `processing_key = SHA-256(kind ‖ input_hash)` with a
  **UNIQUE constraint** so dedup is atomic in the database, not in application
  logic. `ON CONFLICT … DO UPDATE … RETURNING` in one statement; the fresh-vs-
  reused signal is `row.id == fresh_id` (race-free insert-vs-reuse).
- Reuse semantics: existing `done` → untouched (`reused: true`); existing
  `pending`/`running`/`stale`/`failed` → reset `pending` (recompute), keeping
  row identity so history and dependency edges survive.
- `derived_job_dependencies` adjacency edges + **recursive CTE** invalidation:
  marking a job `stale` propagates to all transitive dependents in one
  statement. Stale rows are retained (auditable), never silently trusted.
- First draft contained invalid `SELECT … WHERE id = (DML … RETURNING)` SQL —
  rewritten to pure `UPDATE … RETURNING` forms before testing (known Postgres
  rule; caught at rewrite, not in review).

## Build

- `terra-core/api/migrations/0033_derived_jobs.sql` — tables, UNIQUE key,
  status CHECK, no-self-loop CHECK, indexes.
- `terra-core/api/src/routes/derived.rs` — create/get/complete/fail/invalidate
  + `record_reconstruction_draft` helper; registered `.nest("/derived", …)`.
- `routes/reconstruction.rs::job_complete` records a `reconstruction-draft`
  ledger row keyed on `(parcel_id, capture points)` after the draft insert —
  retries collapse onto one computation while geometry drafts stay per-job.

## Test (evidence)

- `CARGO_INCREMENTAL=0 cargo test -p terra-api` → **117 passed, 0 failed**
  (2026-10-31), incl. 7 new `derived` unit tests (key determinism/kind/input
  separation, hash format, validation).
- Migration applied live at API startup ( `_sqlx_migrations` v33).
- Live smoke `/api/v1/derived/jobs` (Ed25519-signed requests) → **16/16**:
  create→dedup (same id, `created=false`), complete→idempotent re-complete,
  reuse of `done` row (`reused=true`, result kept), dependent edge →
  invalidate marks both stale (count 2), re-invalidate count 0, stale→409,
  re-create stale→reset pending, failed→409 on complete, 404s, 400s.
  **Re-verified 2026-10-31 20:38 on a cleared table** (14/16 on first run was
  stale rows from an earlier smoke, not a code defect).
- Reconstruction wiring live: two completions with identical capture points →
  2 geometry drafts, **1** ledger row; both draft `geometry_hash`s equal the
  ledger `result_hash` (`e69c81671c0ea35e…` on re-verify 2026-10-31 20:46) —
  **same key ⇒ same output** verified. (Hash value is input-dependent, so it
  differs from the earlier `77f2b688…` run; that is expected, not drift.)
- **Re-verified after the R-016 auth fix (2026-10-31 20:46):** API restarted
  with `API_AUTHORITY_PUBKEY` set; unsigned `complete` → **401** (was 200,
  unauthenticated); signed smoke → **16/16**; draft ledger → 2 drafts / 1 row /
  1 hash. Auth fix does not change the Phase 04 behavior, only its trust
  boundary.

## Loop

- Phase 04 exit criteria met (new tests + UNIQUE dedup migration + suites
  green). Book registers updated: IMPLEMENTATION_STATUS B8 + baseline row,
  ROADMAP Stage 1 item 2, work/CURRENT.md, CHANGELOG.
- Open for next loop: Phase 05 (external read-only replay, REQ-018); cargo
  suite re-runs for non-API baselines; devnet deploy still founder-blocked
  (Q-001).

## Gaps found in this loop (2026-10-31 re-verification)

1. **No request-signing client exists in the repo.** The live smoke scripts
   live in `/tmp/opencode/` (ephemeral) and hand-roll Ed25519 signing. Every
   privileged endpoint is gated by `auth::SignedRequest`, which requires
   `Authorization: Signature <b64>`, `x-timestamp`, `x-body-hash` — but no
   shipped client produces those headers. Recorded as **R-016**.
2. **`complete_job` / `fail_job` / `invalidate_job` discard the signature.**
   `let _ = auth;` means the Ed25519 proof is parsed and replay-checked
   (±60 s) but never *verified against a key*. A caller who can forge the
   headers (or reach the API from behind a gateway that strips auth) can
   complete, fail, or invalidate any job. Same pattern already present in
   `reconstruction.rs` (`_auth: SignedRequest` on 5 endpoints incl.
   `job_complete`, `job_progress`, `lease_reconstruction_job`,
   `draft_anchor_confirmed`). Recorded as **R-016** — fix is one line per
   handler (`auth.verify(&key, method, path)?`).
