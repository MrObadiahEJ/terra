# Current Work Ledger

Authoritative record of active execution. Phase derives from evidence here.
Last reconciled: 2026-11-14 (committed `90b76d3`; sprints 001–051 delivered:
037 data-plane component artifact storage, 038 quadtree discovery client,
039 frustum/occlusion culling + screen-space-error LOD, 040 viewport/globe
wiring, 041 Terra File protocol architecture directive, 042 P0 security &
trust-chain fixes, 043 worker identity takeover fix, 044 open-questions →
ratifiable decisions, 045 Terra Atlas frontend, 046 protocol architecture
directives RFC-014 economics + RFC-017 secure evidence intake, 047 UUID/
component_id artifact-retrieval fix, 048 SDK standard shift — terra-geo
SDK v0.1.0 declared + RFC-015/protocol alignment, 049 SDK standard shift —
terra-identity + terra-registry SDKs declared + RFC protocol alignment,
050 submit_component auth fix, 051 security hardening — SecurityRequest.subject,
PrivacyState writes, complete_assignment auth).
Sprint notes for 025–048 are in `sprints/025-*.md` … `sprints/048-*.md`.
Sprint notes for 025–046 are in `sprints/025-*.md` … `sprints/046-*.md`.

## Active intents

- **INT-001/002/003** — maintained (`IMPLEMENTED AND VERIFIED`; regressions
  caught by BPF/vitest gates).
- **INT-004** — `PARTIALLY IMPLEMENTED` (see intent for the exact gap list).
- **INT-005** — `SPECIFIED ONLY` (Q-003 now a decision proposed: sunset bootstrap
  council with vLEI/DNSSEC evidence bundles; founder ratification pending; does
  not block doc-only work).

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
| Sprint 031 — RFC-016 registry dispatch repair | **done & committed (2026-11-12)** | `41fe53d` re-exports the eleven generated `__client_accounts_*` modules at the crate root with `#[doc(hidden)] pub(crate)`; `3a6c0a1` rustfmt for `security.rs` + its tests; `cargo check -p terra-registry` passes (33.6 s) and `cargo fmt --check` clean; A37 remains `2 – IMPLEMENTED, NOT FULLY VERIFIED` (no BPF tests, not deployed); sprints/031 |
| Sprint 032 — Spatial index decision + signed content-addressed manifests | **done & committed (2026-11-13)** | `db22de0` adds `geo-engine/src/spatial_index.rs` (EPSG:4326 quadtree, `q/level/x/y` IDs, covering with 65,536-cell cap) and `manifest.rs` (`terra.manifest/v1` payload, SHA-256 content addresses, Ed25519 domain-separated signatures, object/component references, bounds/LOD/validation invariants); ADR-020 records quadtree-now/S2-H3-later/octree-BVH-for-3D; `40b6633` rustfmt for `component_package.rs`; `cargo test -p terra-geo --lib` **48 passed, 2 ignored** (15 new), `cargo check -p terra-geo` and `cargo fmt --check` pass; API integration still outstanding; sprints/032 |
| Sprint 033 — API discovery cells + signed manifest verification | **done & committed (2026-11-13, `496ec49`)** | `GET /spatial/cell` + `GET /spatial/cover` expose the quadtree without DB access; `POST /manifests/verify` validates `terra.manifest/v1` envelopes (payload invariants, publisher key, Ed25519 signature, content address) without persisting; `cargo check -p terra-api` passes (5m08s); `cargo test -p terra-api endpoint` **6 passed** (133 filtered); rustfmt clean; signed persistence, content-addressed retrieval, signing-key custody and scene-path cell use remain; sprints/033 |
| Sprint 034 — Restore CI formatting and clippy gates | **done & committed (2026-11-13, `9ab419d`, `f9f0a0d`)** | `cargo fmt --all` for five API files; clippy cleanup (deprecated `TimeoutLayer`, unused runtime items/import, needless borrows, `i32` cast, `component_package.rs` lints); workspace `cargo clippy -- -D warnings` and `cargo fmt --all -- --check` clean; `cargo test -p terra-api` **139 passed**, `cargo test -p terra-geo --lib` **48 passed, 2 ignored**; CI run `43460062612` fully green (format, clippy, unit/RFC-012/geo, SBF, BPF, API, frontend); no behavior change; sprints/034 |
| Sprint 035 — Signed manifest persistence + content-addressed retrieval | **done & committed (2026-11-13, `795c60d`, `ff7adbe`)** | migration 0044 `signed_manifests` keyed by `(content_address, publisher)`; `POST /manifests/signed` verifies and stores pre-signed envelopes idempotently; `GET /manifests/signed/{address}` re-verifies and returns them; ADR-021 records that the API is not a signer; `cargo test -p terra-api` **142 passed** (3 new, including the env-gated DB roundtrip), clippy/fmt clean; data-plane component bytes and Terra publisher key custody remain; sprints/035 |
| Sprint 036 — Signed manifest discovery by region/cell | **done & committed (2026-11-13, `638912a`)** | `GET /manifests/signed?region_id=&cell=&limit=` lists stored envelopes (metadata only) with exact `q/level/x/y` cell validation, default limit 20, clamp 1–100 and deterministic ordering; the DB roundtrip also asserts listing; `cargo test -p terra-api` **143 passed**, clippy/fmt clean; pagination, parent/prefix discovery and data-plane component storage remain; sprints/036 |
| Sprint 042 — P0 security & trust-chain fixes (REQ-026/026a/027/028/029–040) | **done & committed** | Fixed PDA seed circular reference in `create_review_case`/`finalize_decision` (review pipeline was unexecutable); `finalize_appeal` now requires committee reputation + `appeal.decision == decision.key()` binding; added `emergency_lift` instruction (was a censorship gadget with no lift path); `validation_state` forced to `'pending'` server-side on component submission; `upload_component_artifact` verifies `x-body-hash` against actual body SHA-256; added `tests/security.rs` to CI. All Rust gates clean (`cargo check`, `clippy -- -D warnings`, `fmt --check`); frontend gates clean. | `sprints/042-p0-security-trust-chain-fixes.md` |
| Worker identity takeover fix (`workers.rs` ON CONFLICT wallet overwrite) | **done & committed** | `register_worker` no longer overwrites `wallet_address` on `ON CONFLICT (worker_id)` — a worker's wallet is set once at creation and cannot be hijacked by re-registering under a different wallet. Closes REQ-038 identity-takeover vector. | `cargo check -p terra-api` clean; `cargo clippy -- -D warnings` clean; `cargo fmt --check` clean |
| Sprint 044 — Open questions → ratifiable decisions (Q-001–Q-006, Q-008–Q-018) | **done & committed** | `OPEN_QUESTIONS.md` rewritten: each blocker now a *decision proposed* with default recommendation, external grounding, and exit criterion. Q-001 devnet funding reframed as 3 tracks ($0 local validator/Surfpool, faucet rotation, shrink rent). Q-003 sunset bootstrap council w/ vLEI/DNSSEC evidence. Q-004 selective-disclosure VC + EUDI path. Q-005 keeps hard gate (protocol ≠ issuer). Q-006 7-point mainnet checklist. Q-008–Q-018 agent-research answers with code-vs-design split. | `sprints/044-open-questions-decisions.md` |
| Sprint 045 — Terra Atlas frontend (investor-facing planetary observation surface) | **done & committed** | `AtlasPage` + glass window system (Asset Inspector 3DGS splat, Wallet/Rights simulated panel, Layers, Provenance, InvestorTour); lazy-loaded per route (Cesium out of initial graph); design tokens (violet accent, gradient mesh, glass panels); demo splat `demo-house.splat` (2.5 kB) labeled synthetic; i18n keys en/fr/ru/zh; verified: tsc clean, eslint clean, vitest 120/120, build:check PASS (entry 308.8 kB gz ≤ 360, Cesium lazy 1098.5 kB ≤ 1200, total 1882.8 kB gz ≤ 1900). Published at https://mrobadiahej.github.io/terra/atlas | `sprints/045-terra-atlas-frontend.md` |

## Known blockers

- **Devnet deploy (B6 / Q-001).** Deployer wallet `CCng7wFcSKCh1RadwcFEN36E4tRrPy1D64fwsQDrsy4m`
  holds 0 SOL; faucet rate-limited (~46 attempts). `./deploy.sh devnet` ready.
  Founder action. All on-chain statuses remain local-verified until this runs
  (RISK R-001). **Founder directive 2026-11-02: funding is the last step, not
  the next one — it blocks only Stage 2, never frontend/API/local work.**
- **Founder review pending:** Q-003 (bootstrap council), Q-004 (residency
  attestation recognition), Q-005 (securities research), Q-019 (reward-governance
  council), Q-020 (Assurance Profile per task class), Q-021 (vendor-neutral
  attestation for global access). These gate Stage 4 implementation but do not
  block doc-only work.
- **Not blocked:** dev server (`pnpm dev` + terra-api), offline demo stories
  (no wallet/API), API suites, and frontend gates all run with zero SOL.

## Next up (order)

1. **Stage 2 (devnet)** — Q-001 devnet funding is now a *decision proposed*, not a
   code blocker: default decision is three tracks in parallel — (A) run the full
   A1–A21 smoke suite on a local validator/Surfpool today ($0), (B) faucet
   rotation (~17.1 devnet SOL via QuickNode/Helius/Chainstack/Alchemy; scripted
   cron claimer), (C) shrink rent (close stale programs, deploy registry first).
   Exit criterion: A1–A21 green on localnet/Surfpool AND registry deployed to
   devnet with wallet signing verified. Founder approves the exit definition.
   Once satisfied: `./deploy.sh devnet` + on-chain smoke (register parcel →
   rights → geometry version → 3D model → snapshot → escrow → dispute → slash
   dry-run), then live-cluster verification of REQ-026/027/028.

2. **Stage 4 (research-gated)** — Q-003/Q-004/Q-005 are now *decisions proposed*,
   not indefinite research: (Q-003) sunset bootstrap council with vLEI/DNSSEC
   evidence bundles; (Q-004) selective-disclosure VC profile + two-proof evidence
   bundle + EUDI/QTSP pathway; (Q-005) securities memo (US, EU, LU, SG, AE) +
   counsel sign-off, protocol ≠ issuer (compliance as adapter). After founder
   ratification: implement institutional identity (REQ-020), residency credential
   (REQ-021), lien encumbrance (REQ-022). Q-005 keeps REQ-023 hard-gated; no code
   before written legal research.

3. **RFC-016 Phase 2+ code** — remaining item: **provider proof-of-possession
   audits** (unblocked agent work). Completed this sprint:
   `SecurityRequest.subject` set to requester key on `submit_security_request`
   (was `Pubkey::default()`); `PrivacyState` written on `finalize_decision`
   (new `privacy_state` PDA in `FinalizeDecision` context); `submit_component`
   requires signed auth (Sprint 050); `complete_assignment` requires signed
   auth (Sprint 051).

4. **Data-plane component artifact storage** — **done (Sprint 037)**: component
   bytes stored under their SHA-256 content address (`migration 0045` +
   `PUT/GET /components/{id}/artifact` + `store_content_addressed`), served with
   client-side digest re-verification, and wired as a frontend fallback in
   `fetchComponentAsset`. **Quadtree discovery cells derived from camera bounds
   in GlobePage scene requests (Sprint 038):** `cameraViewBounds` + `getQuadCover`
   are called from `GlobePage`'s camera-moved effect; the resulting cover cells
   are passed to `TerraGlobe`, which coarsely culls runtime nodes whose
   `spatial_extent` does not intersect a cover cell. Remaining: expose the
   package UUID through scene nodes so `fetchComponentArtifact` is automatic.

5. **Frustum/occlusion culling + screen-space error LOD** — **done (Sprint 039),
   wired into GlobePage (Sprint 040):** `screenSpaceError`,
   `lodFromScreenSpaceError`, frustum cull (`outsideHorizontalFov`/
   `outsideVerticalFov`), occlusion/backface cull (`isOccluded`), and combined
   `cullComponent`/`selectVisible` are applied to `runtimeNodes` in `GlobePage`
   (culledNodes state, recomputed on camera move), so only visible components are
   fetched/rendered. Distance-based LOD (`lodFromHeight`,
   `TIER_SETTINGS.buildingCap`, `selectBuildingsForTier`) already existed;
   screen-space error and culling are implemented, unit-tested
   (viewport.test.ts, 29 tests), and wired into rendering. Awaiting live-cluster
   verification only.

6. **Protocol architecture directives (docs only, Sprint 046):**
   `docs/rfc-014-protocol-economics-contributor-rewards.md` (treasury outflow-only
   economics, Q-005 securities hard gate, staging bonds) and
   `docs/rfc-017-secure-evidence-and-accountable-validation.md` (secure evidence
   intake, rendition + forensic marking, sealed attribution, Assurance Profiles
   AP-0..AP-3), plus amendments to RFC-016 (§11 retention/erasure), RFC-010
   (§15 compromise fast-path), RFC-007 (§9.5 compromise freeze trigger),
   RFC-012 (§6 rule 16 Assurance Profile routing), and Terra File directive §7
   (sensitive layers never default-on; identity documents excluded from exports
   unless explicitly authorized). Book registers updated: REQ-041..052, ADR-022/023,
   R-027..R-032, Q-019/020/021. Verified against code: `MAX_BODY_BYTES` default
   64 MiB / `REQUEST_TIMEOUT_SECS` 60 s; `StorageBackend::Local` only, no
   quarantine/isolation, no retention/TTL/purge path (R-027 documents the gap).
   No code changed — docs only.

7. **SDK standard shift (code + docs, Sprint 048):**
   `terra-core/geo-engine/src/lib.rs` adds an `sdk_meta` module declaring the
   Terra Geo SDK (`terra-geo` v0.1.0) identity per directive §3 / §12:
   capabilities (geometry digest terra-geom-v1/v2, RFC-015 component-package
   conformance, signed manifests, quadtree discovery), schema versions
   (RFC-015, terra-geom-v1/v2, terra.manifest/v1), direct crate dependencies,
   security assumptions, and upgrade/migration behavior. Unit test
   `sdk_meta::tests::sdk_metadata_is_well_formed` verifies the declaration.
   RFC-015 §5 names `terra-geo` SDK v0.1.0 as the canonical RFC-015 conformance
   validator implementation (adds §5.1 SDK standard declaration); chapter 11
   §11.3 references the SDK as the canonical implementation. IMPLEMENTATION_STATUS
   section C adds row C6 (SDK standard declaration, status 1). Verified:
   cargo check --workspace clean; clippy --workspace -- -D warnings clean;
   fmt --all --check clean; cargo test -p terra-geo --lib 49 passed.

