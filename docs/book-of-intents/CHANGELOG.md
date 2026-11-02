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

## 2026-11-02 — RFC-014 Phase 05 (external read-only replay, REQ-018)

- `routes/stories.rs`: `GET /stories/{id}/replay` (versioned replay document,
  `terra.story-replay/v1`, `read_only: true`, `definition_hash`),
  `GET /stories/{id}/replay.geojson` (`application/geo+json`: stops as Point,
  routes as LineString), `GET /stories/schema/replay` (JSON Schema 2020-12
  pinned by `$id`). All read-only and unauthenticated — presentation data,
  never proof.
- Stability policy: v1 additive-only; breaking change mints v2 at a new path.
- Evidence: API suite **121/121** (4 new replay tests); live smoke — replay
  fields verified, GeoJSON content-type, schema pinned, editing the story
  changed `definition_hash`, missing ids → 404.
- Book: Chapter 09 (+ SUMMARY/BOOK index), REQ-018/REQ-019 → status 1,
  IMPLEMENTATION_STATUS B2 + API baseline 121, ROADMAP Stage 1 item 3 met,
  work/CURRENT.md, sprints/005.

## 2026-11-02 — Verification refresh (Stage 1 item 4)

- Re-executed every suite against current sources: API **121/121**, registry
  lib **153/153**, registry BPF integration **307/307** (295.9 s, after
  `anchor build --skip-lint` refreshed stale `.so` artifacts), identity
  **23/23**, geo **12 passed + 1 ignored** (suite grew from the 6 baseline),
  frontend vitest **44/44**. Baseline table re-dated.
- R-008 confirmed: the CI job "On-chain Program Tests" runs lib/rfc012/geo
  but not the 307-test BPF integration suite (it needs built `.so` artifacts
  first). E1 → `PARTIALLY IMPLEMENTED`; this is the remaining Stage 1 exit
  item.
- Book: ROADMAP item 4 partially met, IMPLEMENTATION_STATUS baselines + E1,
  RISK R-008 annotation, work/CURRENT.md, sprints/006.

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

## 2026-11-02 — CI green: SBF build + 307-test BPF suite wired (R-008 resolved, Stage 1 exit)

- `.github/workflows/ci.yml`: the "On-chain Program Tests" job now installs
  Solana `v3.1.14` (pinned to the `solana-program-test` 3.1.x line in
  Cargo.lock), builds both program artifacts with `cargo build-sbf`, and runs
  the 307-test registry BPF integration suite single-threaded.
- First attempt used release `stable`; its SBF output was rejected by the
  pinned runtime ("Program is not deployed / Unsupported program id"),
  failing all tests. Pin `v3.1.14` and bump it together with
  `solana-program-test`.
- Evidence: CI run `41474466105` green — Frontend 32s, API Build Check 39s,
  API Tests 3m6s, On-chain Program Tests 5m18s including BPF integration.
- R-008 resolved; E1 → `IMPLEMENTED AND VERIFIED`; ROADMAP Stage 1 exit met.
  Stage 2 (devnet truth) is now blocked only on founder funding (Q-001).

## 2026-11-02 — Frontend local verification; funding deferred to last (founder directive)

- Verified the product surface locally with **zero SOL**: `/welcome`,
  `/stories` (3 offline demos + 1 saved story), `/demo` (verification
  scenario reaches `VERIFIED`, 12 events / 9938 ms, 3-reviewer replay),
  `/story/demo-road-journey` (Cesium route + 31 s timeline). Headless Chrome
  screenshots; no console errors or failed requests.
- **Founder directive:** devnet funding (Q-001) is the last step, not the
  next. ROADMAP Stage 2 carries a sequencing note; CURRENT.md next-up is
  reordered to local product work first, funding last. Q-001 itself is
  unchanged (founder decision).
- Evidence: sprints/007.

## 2026-11-02 — AI provenance on geometry drafts (REQ-024, R-014 resolved)

- Migration 0034 adds `provenance_kind` (`deterministic` / `ai-inference` /
  `ai-assisted`) and model provider/name/version/confidence to
  `geometry_drafts`; existing rows default to `deterministic`.
- `POST /reconstruction/jobs/{id}/complete` validates provenance fail-closed:
  model fields require an AI kind, AI kinds require model name + version,
  confidence must be finite in `[0,1]`. DB constraints mirror the invariants.
- Evidence: API **128/128** (7 new tests); live smoke **13/13** (AI metadata
  persisted + returned by `GET …/draft`; four ambiguous payloads → 400;
  deterministic default clean); psql constraint checks reject bypasses.
- Book: REQ-024 → status 1, R-014 resolved, IMPLEMENTATION_STATUS B3/B10 +
  API baseline 128, ROADMAP post-exit note, sprint 008. No SOL required.

## 2026-11-02 — Book volume 10 (derived compute, request auth, AI provenance)

- New chapter documenting the three boundaries kept separate this session:
  cache dedup (same key ⇒ same output, not truth), Ed25519 request
  authenticity (authority, not identity), and AI provenance (description,
  not verification). Includes failure-mode table and acceptance evidence.
- Index/SUMMARY updated; E4 evidence now lists chapters 01–10; sprint 009.
  Documentation only — no behavior change.
