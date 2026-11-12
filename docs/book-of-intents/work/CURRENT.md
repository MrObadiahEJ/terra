# Current Work Ledger

Authoritative record of active execution. Phase derives from evidence here.
Last reconciled: 2026-11-12 (committed `cd93c25`; sprints 001–030 delivered).
Sprint notes for 025–030 are in `sprints/025-*.md` … `sprints/030-*.md`.

## Active intents

- **INT-001/002/003** — maintained (`IMPLEMENTED AND VERIFIED`; regressions
  caught by BPF/vitest gates).
- **INT-004** — `PARTIALLY IMPLEMENTED` (see intent for the exact gap list).
- **INT-005** — `SPECIFIED ONLY` (founder review flag; Q-003 trust commitment
  pending).

## Execution state

| Work item | State | Evidence |
|---|---|---|
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
| Sprint 012 — API security response headers | **done & committed (2026-11-02)** | middleware in `api/src/main.rs`; 2 new tests; `cargo test -p terra-api` **130/130** |
| Sprint 013 — API request body-size limit | **done & committed (2026-11-02)** | layer in `api/src/main.rs`; 2 new tests; `cargo test -p terra-api` **132/132** |
| Sprint 014 — Frontend logic unit tests (LOD + codec helpers) | **done & committed (2026-11-02)** | `src/lib/lod.test.ts` + `src/lib/codec.test.ts`; vitest **52/52**; tsc/eslint/build/bundle-check green |
| Sprint 015 — API request timeout | **done & committed (2026-11-02)** | layer in `api/src/main.rs`; 1 new test; `cargo test -p terra-api` **133/133** |
| Sprint 016/018 — Book index consistency passes | **done & committed (2026-11-02)** | indexes synced (sprints 001–018); doc-only — no gate required |
| Sprint 017 — Frontend logic unit tests (protocol status) | **done & committed (2026-11-02)** | `src/lib/protocolStatus.test.ts`; vitest **56/56**; tsc/eslint/build/bundle-check green |
| Sprint 019 — Frontend/UX polish: demo header source chip | **done & committed (2026-11-02)** | demo header source chip; frontend full gate green |
| Sprint 020 — Frontend/UX polish: welcome offline badge | **done & committed (2026-11-02)** | welcome offline badge; frontend full gate green |
| Sprint 021 — Book consistency catch-up + requirements traceability (REQ-025) | **done & committed (2026-11-02)** | REQUIREMENTS_TRACEABILITY REQ-025 added; indexes extended with sprints 018/019/020; E4 → sprints 001–020; doc-only — no gate required |
| Sprint 022 — Frontend logic unit tests — activity store + API GeoJSON parse | **done & committed (2026-11-02)** | `src/lib/activityStore.test.ts` + `src/lib/api.test.ts`; vitest **65/65**; tsc/eslint/build/bundle-check green |
| Sprint 023 — Research: Q-003 (institutional bootstrap) + Q-004 (residency attestation) | **done & committed (2026-11-02)** | 5 DNS-identity designs + 6 jurisdictions surveyed; findings in RESEARCH_REFERENCES; Q-003/Q-004 → `5 – PROPOSED`; chapters/02 §2.3 + chapters/08 §8.6 updated; doc-only — no gate required |
| Stage 3 — Planned-capability specifications (REQ-020/021/022) | **done & committed (2026-11-02)** | Chapters 2/6/8 → `SPECIFIED ONLY` with ⚠ founder review flags; INT-005 updated; ROADMAP Stage 3 exit met except founder review of the flags; doc-only — no gate required |
| Sprint 024 — Architecture spec: distributed reconstruction & assembly (Ch11), world runtime & synchronized rendering (Ch12), global spatial data delivery & progressive visualization (Ch13) | **done & committed (2026-11-02)** | Chapters 11/12/13 written `SPECIFIED ONLY`; REQ-026/027/028 added to REQUIREMENTS_TRACEABILITY; BOOK_OF_INTENTS/SUMMARY/ROADMAP updated; doc-only — no gate required |
| Sprint 025 — REQ-026/027/028 implementation (RFC-015 component package + manifest + runtime + providers + workers) | **done & committed (2026-11-02/03)** | `geo-engine/src/component_package.rs` (RFC-015 conformance validator) + `canonical3d.rs` (`terra-geom-v2`); migrations 0035–0043; `components.rs`/`assemblies.rs`/`manifests.rs`/`providers.rs`/`runtime.rs`/`workers.rs` routers; frontend `runtime.ts` + `runtime.test.ts` + `WorkerConsolePage.tsx`; API 133/133; geo-engine 33 (+2 ignored); vitest 83/83; tsc clean; build:check PASS |
| Sprint 026 — RFC-016 on-chain security/privacy governance core (REQ-029–040) | **done & committed (2026-11-02/03)** | `src/security.rs` (1070 lines): 11 PDAs + 11 instructions + 13 `TerraError` codes; registered in `lib.rs`; registers updated in IMPLEMENTATION_STATUS + REQUIREMENTS_TRACEABILITY; compiles clean; **no BPF tests yet, not deployed to devnet** |
| Chapter 3 → Ch11/12/13 traceability (acceptance criteria concretized) | **done (2026-11-02)** | Ch3 §3.9 now lists concrete targets: SpatialAsset/GeometryVersion anchor mapping for Ch11; `lod.ts`/`cesiumTerrain.ts` → L0–L4 mapping for Ch13; `terra-geom-v2` integrate-or-deprecate decision recorded in ADR-016; E4 row updated to chapters 01–13 | Doc-only — no gate required |
| Sprint 027 — Atlas 16-point requirement audit (frontend fixes) | **done & committed (2026-11-05, `eef0087`)** | `ThreeAtlasFallback.tsx` restored missing `return <div ref={hostRef}>` host (was returning `void`); `TerraGlobe.tsx` typed `ring.map` destructuring + typed runtime accessor for `moveWest/East/South/North/Up/Down` + WASD; `GlobePage.tsx` `failedRegionsRef` dedupes `sceneError` banner retries + manual "Create manifest" retry clears failure; removed 3 empty files (`ThreeNavOverlay.tsx`, `TerraFlightPad.tsx`, `__write-probe.ts`); tsc clean; vitest 83/83; `vite build` PASS (GlobePage 82.4 kB gz, TerraGlobe 874.2 kB gz) |
| Sprint 028 — Doc reconciliation (all `.md` layer-focused) | **done & committed (2026-11-05, `eef0087`)** | IMPLEMENTATION_STATUS.md audit basis → `244ed7e`/2026-11-05, vitest 83/83, bundle entry 305.1/Cesium 1098.5/total 1864.3 kB gz, geo-engine 33 (+2 ignored); added A37–A40 (RFC-016), B15–B19 (REQ-026/027/028 + workers), D12–D18 (runtime, 3D models, atlas verification, tier loading, front-view/single sidebar, backend-down degradation, worker console); REQUIREMENTS_TRACEABILITY REQ-026/027/028 → `IMPLEMENTED, NOT FULLY VERIFIED`; chapters 11/12/13/14 → `IMPLEMENTED, NOT FULLY VERIFIED` with "why this chapter exists" updated to reflect implemented code; work/CURRENT.md reconciled to 2026-11-05 | Doc-only — no gate required |
| Sprint 029 — Merge `origin/dev` + GitHub Pages publication | **done & deployed (2026-11-12)** | merge `92613c8` (upstream `4eb7c5a`, local `eef0087`, no conflicts); Pages workflow `.github/workflows/deploy-pages.yml` builds `terra-web` with `TERRA_BASE_PATH=/terra/`, writes `404.html` + `.nojekyll`, and deploys from `dev`; base-aware assets (`lib/assets.ts`, `%BASE_URL%` for Cesium, router basename); live <https://mrobadiahej.github.io/terra/> verified (welcome/atlas/demo/stories, keyless tiles, demo data, API-offline states); deploy runs `43263687324`, `43265003528`; tsc clean, vitest pass, build:check PASS (entry 299.8 / initial 305.1 / Cesium 1098.5 / total 1864.2 kB gz); sprints/029 |
| Sprint 030 — Google-Earth-style immersive atlas | **done & deployed (2026-11-12)** | commits `48286af` (immersive chrome + camera-preserving 2D↔3D + dock HUD + swipe sheet), `d2d3d25` (satellite default) and `cd93c25` (fullscreen guard); files `GlobePage.tsx`, `AtlasSearchPill.tsx`, `Sidenav.tsx`, `TerraGlobe.tsx`, `Navbar.tsx`, `index.css`, four locales, README; live <https://mrobadiahej.github.io/terra/> on Pages runs `43289772629` + `43290890798` + `43292573392`; viewport sweep 360×640 … 1920×1080 and 844×390 with no horizontal overflow; tsc clean, vitest **83/83**, build:check PASS (entry 300.2 / initial 305.5 / Cesium 1098.5 / total 1866.7 kB gz); sprints/030 |

## Known blockers

- **Devnet deploy (B6 / Q-001).** Deployer wallet `CCng7wFcSKCh1RadwcFEN36E4tRrPy1D64fwsQDrsy4m`
  holds 0 SOL; faucet rate-limited (~46 attempts). `./deploy.sh devnet` ready.
  Founder action. All on-chain statuses remain local-verified until this runs
  (RISK R-001). **Founder directive 2026-11-02: funding is the last step, not
  the next one — it blocks only Stage 2, never frontend/API/local work.**
- **Founder review pending:** Q-003 (bootstrap council), Q-004 (residency
  attestation recognition), Q-005 (securities research). These gate Stage 4
  implementation but do not block doc-only work.
- **Not blocked:** dev server (`pnpm dev` + terra-api), offline demo stories
  (no wallet/API), API suites, and frontend gates all run with zero SOL.

## Next up (order)

1. **Founder review** of remaining Stage 3/4 flags: Q-003 (bootstrap council),
   Q-004 (residency recognition), Q-005 (securities research). These gate
   REQ-020/021/022 implementation but do not block doc-only work.
   REQ-026/027/028 are implemented (Sprint 025) and only need live-cluster
   verification, which is blocked on funding.
2. **Stage 2 (devnet)** — blocked on Q-001 devnet funding (founder). Once
   funded: `./deploy.sh devnet` + on-chain smoke (register parcel → rights →
   geometry version → 3D model → snapshot → escrow → dispute → slash dry-run),
   then live-cluster verification of REQ-026/027/028 (component submission,
   manifest version transition, multi-provider retrieval, worker heartbeat).
3. **Stage 4 (research-gated)** — after Q-003/Q-004/Q-005 resolve: implement
   institutional identity (REQ-020), residency credential (REQ-021), lien
   encumbrance (REQ-022).
4. **Spatial index decision** (quadtree/S2/H3) and content-addressed retrieval +
   signed manifests — specified in Chapters 12/13, not yet coded.
5. **Frustum/occlusion culling + screen-space error LOD** — specified in
   Chapter 13, not yet coded.

