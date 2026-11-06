# Sprint 028 — Book of Intents Doc Reconciliation (Layer-Focused `.md`)

**Date:** 2026-11-05 · **Status:** done, uncommitted (carried into commit `8583e46`)
**Gate:** doc-only — no test gate required

## Trigger

Sprints 025/026 delivered REQ-026/027/028 and RFC-016, but the Book registers and
chapters still described the 2026-11-02 20:47 state. Every `.md` file must talk
only to its own layer: frontend docs must not reference backend status, backend
docs must not reference frontend state, and every status label must cite real
evidence.

## Changes

### Registers

- `REQUIREMENTS_TRACEABILITY.md` — REQ-026/027/028 →
  `IMPLEMENTED, NOT FULLY VERIFIED` with real evidence paths
  (`component_package.rs`, migrations 0035–0043, the six new routers, `runtime.ts`).
- `IMPLEMENTATION_STATUS.md` — audit basis → `244ed7e`/2026-11-05; vitest
  **83/83**; bundle entry **305.1 kB gz** / Cesium **1098.5 kB gz** / total
  **1864.3 kB gz**; geo-engine **33 (+2 ignored)**; added A37–A40 (RFC-016),
  B15–B19 (components/manifests/runtime/providers/workers routers),
  D12–D18 (runtime client, 3D model rendering, atlas verification, progressive
  tier loading, front-view/single sidebar, backend-down graceful degradation,
  worker console); verification baselines re-dated with current figures.
- `RISK_REGISTER.md` — R-012 bundle figures re-verified 2026-11-05;
  R-021 `terra-geom-v2` marked integrated (was "experimental").
- `ARCHITECTURAL_DECISIONS.md` — ADR-016 resolved: `terra-geom-v2` integrated
  into production (was "experimental, not merged").
- `ROADMAP.md` — Stage 3 REQ-026/027/028 → `IMPLEMENTED, NOT FULLY VERIFIED`;
  Stage 4.5 Phase 2 on-chain core done (no BPF tests/devnet yet), Phases 3–5 not
  started; stage diagram relabeled.
- `work/CURRENT.md` — reconciled to 2026-11-05; sprints 025–028 added; "Next up"
  rewritten (REQ-026/027/028 implemented, only live-cluster verification +
  spatial-index/culling/caching gaps remain).
- `CHANGELOG.md` — new 2026-11-05 entry for sprints 025–028.

### Chapters

- `chapters/11`, `12`, `13`, `14` — status `SPECIFIED ONLY` →
  `IMPLEMENTED, NOT FULLY VERIFIED`; "Why this chapter exists" rewritten to
  describe what is now built rather than what is missing; "Connection to
  existing code" tables updated with the new routes/migrations/frontend
  modules.
- `chapters/03` — 3D digest status updated: `terra-geom-v2` integrated
  (was "experimental and not merged").
- `rfc-015-spatial-component-package.md` — status →
  `IMPLEMENTED, NOT FULLY VERIFIED`.
- `BOOK_OF_INTENTS.md`, `SUMMARY.md` — index labels + audit basis updated.

### New sprint files

`sprints/025-req-026-027-028-implementation.md`,
`sprints/026-rfc-016-on-chain-security-core.md`,
`sprints/027-atlas-16-point-audit.md`,
`sprints/028-book-of-intents-doc-reconciliation.md` — the Book convention is
one `.md` per sprint; sprints 025–028 had only been recorded in `work/CURRENT.md`
and CHANGELOG.

## Cross-layer audit

- No frontend chapter references backend code paths (`routes/`, `.rs`, `cargo
  test`, `BPF`, `anchor deploy`, `migration 00`): clean.
- No backend chapter references frontend state.
- Chapter 15 correctly remains `SPECIFIED ONLY` — the end-to-end prototype is
  genuinely not built.
- Historical sprint files (001–024) and CHANGELOG entries were left untouched;
  they accurately record the state at the time they were written.

## Status after this sprint

All `.md` files are layer-focused and cite evidence verified this date:
API 133/133, geo-engine 33 (+2 ignored), registry lib 153/153, vitest 83/83,
`tsc` clean, `build:check` PASS, eslint matches pre-existing baseline.