# Current Work Ledger

Authoritative record of active execution. Phase derives from evidence here.
Last reconciled: 2026-11-02 (working tree; sprints 012–022 delivered locally, uncommitted).

## Active intents

- **INT-001/002/003** — maintained (`IMPLEMENTED AND VERIFIED`; regressions
  caught by BPF/vitest gates).
- **INT-004** — `PARTIALLY IMPLEMENTED` (see intent for the exact gap list).
- **INT-005** — `PROPOSED` (research-gated; Q-003).

## Execution state

| Work item | State | Evidence |
| --- | --- | --- |
| RFC-014 Phases 01–03 (story engine, road-graph navigation + version provenance, export refinements) | **done & committed** | commits `c9faf54`, `54ad2bd`, `0f0cc2e` + fixes `b464cdb`, `a2b2afe`, `11be94c`, `ae20af8`; vitest 44/44 (2026-10-31) |
| Canonical geometry digest `terra-geom-v1` | **done & committed** | commit `775e20a`; `docs/canonical-geometry-digest.md` |
| Book of Intents constitution (this directory) | **v0.1.0 delivered & committed** | commit `6be7c91`; sprint 002; CHANGELOG |
| RFC-014 Phase 04 — incremental derived compute (REQ-019) | **done & committed (`ad5a069`)** | migration 0033 + `routes/derived.rs`; API **117/117** at the time (7 derived tests); live smoke **16/16**; reconstruction: 2 drafts → 1 ledger row, equal hashes; sprints/003 |
| R-016 — verify signatures on privileged endpoints | **done (2026-11-02)** | 8 handlers now `auth.verify(…)` fail-closed (derived + reconstruction); live: unsigned 401, wrong key 401, authority key 200; sprints/004 |
| R-017 — shipped request-signing client | **done (2026-11-02)** | `api/examples/sign_request.rs` (deps already present); contract in AGENT_GUIDE §6; drove the live smoke; sprints/004 |
| RFC-014 Phase 05 — external replay/access (REQ-018) | **done & committed (`393c305`)** | `routes/stories.rs` `/replay` + `/replay.geojson` + `/schema/replay`; `terra.story-replay/v1`; API **121/121** at the time; live smoke (hash tracks edits, 404s); chapters/09, sprints/005 |
| Verification refresh (all suites re-executed) | **done (2026-11-02)** | API 121/121; registry lib 153/153; BPF 307/307 (295.9 s); identity 23/23; geo 12 (+1 ignored); vitest 44/44; baselines re-dated; sprints/006 |
| CI Rust-job depth (R-008) | **done (2026-11-02)** | `.github/workflows/ci.yml`: Solana `v3.1.14` + SBF build + 307-test BPF suite; run `41474466105` green (On-chain Program Tests 5m18s) |
| Frontend local verification (no funding) | **done (2026-11-02)** | dev server + local API; `/welcome`, `/stories`, `/demo` (VERIFIED, 12 events / 9938 ms), `/story/demo-road-journey` all render; sprints/007 |
| R-014 — AI provenance on geometry drafts (REQ-024) | **done (2026-11-02)** | migration 0034: `provenance_kind` + model provider/name/version/confidence; fail-closed validation; API **128/128** (7 tests); live smoke **13/13**; DB constraints verified; sprints/008 |
| Book volume 10 (derived compute, auth, AI provenance) | **done (2026-11-02)** | `chapters/10-derived-compute-and-provenance.md`; index/SUMMARY; E4 evidence; sprints/009 |
| R-012 — bundle discipline gate | **done (2026-11-02)** | `terra-web/scripts/check-bundle.mjs` + `pnpm build:check`; CI frontend job now builds; entry **295 kB gz**, initial **301 kB**, Cesium **1098 kB lazy**; negative test exits 1; sprints/010 |
| Sprint 012 — API security response headers (nosniff, frame-deny, referrer-policy, permissions-policy) | **done (2026-11-02)** | middleware in `api/src/main.rs`; 2 new tests; `cargo test -p terra-api` **130/130** (2026-11-02); uncommitted per founder commit directive — no SOL required |
| Sprint 013 — API request body-size limit (`MAX_BODY_BYTES`, default 64 MiB → 413) | **done (2026-11-02)** | layer in `api/src/main.rs`; 2 new tests; `cargo test -p terra-api` **132/132** (2026-11-02); uncommitted per founder commit directive — no SOL required |
| Sprint 014 — Frontend logic unit tests (LOD + codec helpers) | **done (2026-11-02)** | `src/lib/lod.test.ts` + `src/lib/codec.test.ts`; vitest **52/52** (2026-11-02); tsc/eslint/build/bundle-check green | Uncommitted per founder commit directive; no SOL required |
| Sprint 015 — API request timeout (`REQUEST_TIMEOUT_SECS`, default 60 s → 408) | **done (2026-11-02)** | layer in `api/src/main.rs`; 1 new test; `cargo test -p terra-api` **133/133** (2026-11-02); uncommitted per founder commit directive — no SOL required |
| Sprint 016 — Book index consistency pass | **done (2026-11-02)** | indexes synced (sprints 001–015); doc-only — no gate required | Uncommitted per founder commit directive — no SOL required |
| Sprint 017 — Frontend logic unit tests (protocol status) | **done (2026-11-02)** | `src/lib/protocolStatus.test.ts`; vitest **56/56** (2026-11-02); tsc/eslint/build/bundle-check green | Uncommitted per founder commit directive; no SOL required |
| Sprint 018 — Book index consistency pass (part 2) | **done (2026-11-02)** | indexes extended with sprints 016/017; doc-only — no gate required | Uncommitted per founder commit directive — no SOL required |
| Sprint 019 — Frontend/UX polish: demo header source chip | **done (2026-11-02)** | demo header source chip; frontend full gate green | Uncommitted per founder commit directive — no SOL required |
| Sprint 020 — Frontend/UX polish: welcome offline badge | **done (2026-11-02)** | welcome offline badge; frontend full gate green | Uncommitted per founder commit directive — no SOL required |
| Sprint 021 — Book consistency catch-up + requirements traceability (REQ-025) | **done (2026-11-02)** | REQUIREMENTS_TRACEABILITY REQ-025 added; indexes extended with sprints 018/019/020; E4 → sprints 001–020; doc-only — no gate required | Uncommitted per founder commit directive — no SOL required |
| Sprint 022 — Frontend logic unit tests — activity store + API GeoJSON parse | **done (2026-11-02)** | `src/lib/activityStore.test.ts` + `src/lib/api.test.ts`; vitest **65/65**; tsc/eslint/build/bundle-check green | Uncommitted per founder commit directive — no SOL required |
| Sprint 023 — Research: Q-003 (institutional bootstrap) + Q-004 (residency attestation) | **done (2026-11-02)** | 5 DNS-identity designs + 6 jurisdictions surveyed; findings in RESEARCH_REFERENCES; Q-003/Q-004 → `5 – PROPOSED`; chapters/02 §2.3 + chapters/08 §8.6 updated | Doc-only — no gate required |
| Stage 3 — Planned-capability specifications (REQ-020/021/022) | **done (2026-11-02)** | Chapters 2/6/8 → `SPECIFIED ONLY` with ⚠ founder review flags; INT-005 updated; ROADMAP Stage 3 exit met except founder review of the flags | Doc-only — no gate required |

## Known blockers

- **Devnet deploy (B6 / Q-001).** Deployer wallet `CCng7wFcSKCh1RadwcFEN36E4tRrPy1D64fwsQDrsy4m`
  holds 0 SOL; faucet rate-limited (~46 attempts). `./deploy.sh devnet` ready.
  Founder action. All on-chain statuses remain local-verified until this runs
  (RISK R-001). **Founder directive 2026-11-02: funding is the last step, not
  the next one — it blocks only Stage 2, never frontend/API/local work.**
- **Not blocked:** dev server (`pnpm dev` + terra-api), offline demo stories
  (no wallet/API), API suites, and frontend gates all run with zero SOL.

## Next up (order)

1. **Between phases** — all Stage 1 non-chain deliverables are complete (sprints 003–022). CI green (API 133/133, BPF 307/307, vitest 65/65, frontend typecheck + budget). No funding required. Registers current. Waiting on founder decision for Q-001 devnet funding (Stage 2) or research progress on Q-003/Q-005.

2. **Remaining non-chain work** — frontend/UX polish and small API hardening are captured as backlog items; larger requirements (REQ-020…REQ-023) stay research-gated (Q-003/Q-005). R-012 bundle gate is in place — future frontend work must pass `pnpm build:check` (sprints/010). See sprint 011 for wrap-up.

3. **Last**: fund the devnet deployer wallet (`CCng7wFcSKCh1RadwcFEN36E4tRrPy1D64fwsQDrsy4m`) — founder action, Q-001 — then `./deploy.sh devnet` + Stage 2 on-chain smoke (A1/B6 cluster-verified).
