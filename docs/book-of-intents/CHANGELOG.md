# CHANGELOG — Book of Intents

Format: date · version · what changed · evidence/why. Append-only.

## 2026-10-31 — v0.1.0 (constitution established)

- Repository audit at commit `775e20a` (dev): 100+ registry instructions
  inventoried, identity program, ~25 API routers, migrations 0001–0032,
  geo-engine (`reach.rs` Dijkstra, `terra-geom-v1` digest), frontend story
  module incl. client-side road routing (`story/roads.ts`).
- Executed: vitest **44/44** (timeline/roads/demo/url suites). Cargo suites
  recorded as historical baselines with re-verify flags (BPF count
  re-confirmed in source: 307).
- Created the constitutional registers: TERRA_VISION, IMPLEMENTATION_STATUS
  (7-status vocabulary), REQUIREMENTS_TRACEABILITY (REQ-001…REQ-024),
  ARCHITECTURAL_DECISIONS (ADR-001…ADR-013), OPEN_QUESTIONS (Q-001…Q-012),
  RISK_REGISTER (R-001…R-015), RESEARCH_REFERENCES, ROADMAP (Stages 0–5),
  AGENT_GUIDE, CHANGELOG; root entry point `BOOK_OF_INTENTS.md`.
## 2026-10-31 — v0.1.0 (constitution established)

- Wrote Chapters 02–08 as evidence-grounded specifications (replacing planned
  dead links): Ch3/Ch4/Ch7 cite real code (`spatial_*`, `routing.rs`,
  `reach.rs`, `story/roads.ts`); Ch2/Ch5/Ch6/Ch8 are honest `PROPOSED` specs
  with building blocks + acceptance criteria + research gates. Link check: 0
  broken.
  `IMPLEMENTED AND VERIFIED` client-side; server `/geo/route` does not exist →
  Q-007; geo road-graph topology server-side is `PARTIALLY IMPLEMENTED`).
- Marked institutional identity, residency credentials, liens, fractional
  investment, transaction lineage as `PROPOSED`/`RESEARCH REQUIRED` per
  directive §13 — no code exists; building blocks identified.

## 2026-10-31 — RFC-014 Phase 04 (incremental derived compute, REQ-019)

- Migration `0033_derived_jobs.sql`: `derived_jobs` (UNIQUE `processing_key`
  = SHA-256(kind ‖ input_hash), status pending|running|done|failed|stale) +
  `derived_job_dependencies` adjacency edges. New router `routes/derived.rs`
  at `/api/v1/derived/jobs`: atomic create-or-reuse (`INSERT … ON CONFLICT …
  DO UPDATE … RETURNING`, fresh-vs-existing via returned id), idempotent
  complete, status-guarded fail, recursive-CTE invalidation (job + transitive
  dependents → `stale`; stale rows retained for audit).
- Wired into reconstruction: `job_complete` records a `reconstruction-draft`
  ledger row keyed on `(parcel_id, capture points)` — identical computations
  collapse onto one row while geometry drafts stay per-job.
- Executed evidence: API suite **117/117** (7 new derived tests); live smoke
  **16/16** signed endpoint checks; reconstruction flow twice with identical
  points → 2 drafts, **1** ledger row, both draft hashes = ledger
  `result_hash` (`77f2b688…`) — same key ⇒ same output.
- Registers updated: IMPLEMENTATION_STATUS (B8, B1/B2, API baseline 117),
  ROADMAP Stage 1 item 2 (exit met), work/CURRENT.md, sprints/003.

## 2026-11-02 — Request authentication hardening (R-016/R-017)

- Privileged handlers now verify Ed25519 signatures against
  `API_AUTHORITY_PUBKEY` and fail closed when it is unset: derived
  `complete`/`fail`/`invalidate`; reconstruction `create`, `lease`,
  `progress`, `complete`, `anchor` (lease also binds `x-wallet` when sent).
  Previously these parsed `SignedRequest` and discarded it (`let _ = auth;`).
- Shipped `terra-core/api/examples/sign_request.rs` (no new deps): emits
  `Authorization`/`x-timestamp`/`x-body-hash`/`x-wallet` for the documented
  payload `METHOD\nPATH\nBODY_SHA256\nTIMESTAMP`; contract added to
  AGENT_GUIDE §6.
- Evidence: API **117/117**; unsigned → 401, wrong key → 401, authority key →
  200; Phase 04 smoke re-run **16/16** under verification (sprints/004).

## 2026-10-31 — Security fix: verify Ed25519 signatures on privileged endpoints

- **R-016 (H).** `complete_job`, `fail_job`, `invalidate_job` (`derived.rs`)
  and `create_reconstruction_job`, `lease_reconstruction_job`, `job_progress`,
  `job_complete`, `draft_anchor_confirmed` (`reconstruction.rs`) discarded the
  `SignedRequest` with `let _ = auth;` — they parsed and replay-checked the
  headers but never verified the signature against a key. Any caller that could
  reach the API (e.g. behind a gateway that strips auth) could complete, fail, or
  invalidate arbitrary jobs.
- **Fix:** one-line `auth.verify(&key, method, path)?` per handler, mirroring
  the existing pattern in `parcels.rs:228-232`; fail-closed when
  `API_AUTHORITY_PUBKEY` is unset. `lease_reconstruction_job` additionally
  verifies the claimed worker wallet when one is supplied.
- **Verification:** API restarted with a configured authority key; unsigned
  `complete` → **401** (was 200); signed smoke → **16/16**; draft ledger →
  2 drafts / 1 row / 1 hash. `cargo test -p terra-api` → **117/117**. Behavior
  unchanged except the trust boundary.
- Registers updated: RISK_REGISTER (R-016, R-017), work/CURRENT.md.
