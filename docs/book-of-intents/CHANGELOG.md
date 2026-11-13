# CHANGELOG — Book of Intents

Format: date · version · what changed · evidence/why. Append-only.

## 2026-11-13 — Sprint 035: signed manifest persistence + content-addressed retrieval

- **Migration 0044** (`795c60d`): `signed_manifests` stores verified envelopes
  keyed by `(content_address, publisher)`, with region/cell/version indexes; the
  migration is additive and leaves mutable `world_manifests` untouched.
- **`POST /manifests/signed`** (`795c60d`): verifies a pre-signed
  `terra.manifest/v1` envelope and stores it; returns `201` on first store and
  `200` on an idempotent repeat.
- **`GET /manifests/signed/{address}`** (`795c60d`): fetches by `sha256:…`,
  re-verifies payload and signature, and rejects a digest mismatch as a server
  error.
- **ADR-021:** the API is not a publisher signer; signing stays external, and
  Terra-operated key custody (KMS/HSM/offline) remains to be chosen.
- **Verified:** `cargo test -p terra-api` **142 passed** (3 new, including an
  env-gated DB roundtrip that runs in CI), `cargo test -p terra-geo --lib`
  **48 passed, 2 ignored**, workspace clippy and fmt clean. Data-plane component
  storage remains.
- Evidence: `sprints/035-signed-manifest-persistence.md`;
  `ARCHITECTURAL_DECISIONS.md` ADR-021; Chapter 12 §12.8/§12.13; Chapter 13
  §13.8; Chapter 14; REQ-027; `IMPLEMENTATION_STATUS.md` B16/B20/C5;
  `work/CURRENT.md`.

## 2026-11-13 — Sprint 034: restore CI formatting and clippy gates

- **`9ab419d`:** `cargo fmt --all` across five API files; formatting was the
  only failure in CI `43454803577`.
- **`f9f0a0d`:** clear the workspace clippy backlog without behavior changes:
  deprecated `TimeoutLayer::new` → `with_status_code(REQUEST_TIMEOUT, …)`,
  remove unused runtime items/import, drop needless borrows and an `i32` cast,
  and simplify `component_package.rs` lints.
- **Verified:** `cargo fmt --all -- --check` and `cargo clippy -- -D warnings`
  clean; `cargo test -p terra-api` **139 passed**; `cargo test -p terra-geo
  --lib` **48 passed, 2 ignored**. CI `43460062612` is fully green, including
  SBF builds and BPF integration; no behavior changed.
- Evidence: `sprints/034-ci-format-and-clippy-gates.md`; `work/CURRENT.md`.

## 2026-11-13 — Sprint 033: API quadtree discovery + signed manifest verification

- **`GET /spatial/cell?lon=&lat=&level=`** (`496ec49`): returns the stable
  quadtree cell and bounds; defaults to level 12, rejects invalid coordinates
  or levels with `400`. No database access.
- **`GET /spatial/cover?min_lon=&min_lat=&max_lon=&max_lat=&level=`**
  (`496ec49`): returns ordered intersecting cells, defaults to level 10, and
  enforces the library's 65,536-cell cap.
- **`POST /manifests/verify`** (`496ec49`): validates a `terra.manifest/v1`
  envelope (payload invariants, publisher key, Ed25519 signature, canonical
  content address) and returns the address/region/version/object count. It
  persists nothing and needs no signing key.
- **Verified:** `cargo check -p terra-api` passes (5 m 08 s);
  `cargo test -p terra-api endpoint` **6 passed** (133 filtered; 4 cell/cover +
  2 manifest tests); `cargo test -p terra-geo --lib` still 48/48 (+2 ignored);
  rustfmt clean on changed files. Full API suite and BPF/SBF not run.
- **Still missing:** signing-key custody, signed persistence, content-addressed
  retrieval/storage, scene-path cell use, and live-cluster verification (B6).
  C4/C5 stay `3 – PARTIALLY IMPLEMENTED`; B16 stays `2`.
- Evidence: `sprints/033-api-discovery-and-manifest-verification.md`; Chapter
  12 §12.8/§12.13; Chapter 13 §13.3/§13.8; Chapter 14; REQ-027;
  `IMPLEMENTATION_STATUS.md` B16/B20/C4/C5; `work/CURRENT.md`; `CHANGELOG.md`.

## 2026-11-13 — Sprint 032: spatial index decision + signed content-addressed manifests

- **ADR-020:** control-plane 2D discovery uses a deterministic **quadtree over
  EPSG:4326** with stable `q/{level}/{x}/{y}` IDs (level 0 = world, max level
  30, 65,536-cell covering cap). S2/H3 is deferred to a future
  visualization-tile adapter; octree/BVH remains the 3D-volume structure.
- **`geo-engine/src/spatial_index.rs`** (`db22de0`): validated `GeoBounds`,
  `QuadCell` point/parent/children/bounds, conservative `covering_cells`, and
  edge clamping so boundary objects are never missed.
- **`geo-engine/src/manifest.rs`** (`db22de0`): `terra.manifest/v1` payloads
  addressed by SHA-256 and signed with Ed25519 over
  `terra.manifest.signature.v1\0 || payload_digest`. Objects bind component
  versions, content digests, storage references, bounds, LODs 0–4, and
  validation state. Validation rejects tampering, duplicates, out-of-range
  LODs, and objects outside the region cell.
- **Verified:** `cargo check -p terra-geo` passes; `cargo test -p terra-geo
  --lib` **48 passed, 2 ignored** (15 new: 7 spatial-index, 8 manifest);
  `cargo fmt -p terra-geo -- --check` clean. `component_package.rs` rustfmt is
  a separate formatting-only commit (`40b6633`).
- **Not done:** API still stores mutable unsigned rows and does not serve signed
  envelopes, content addresses, or quadtree lookups; no live-cluster
  verification (B6). C4/C5 stay `3 – PARTIALLY IMPLEMENTED`.
- Evidence: `sprints/032-spatial-index-signed-manifests.md`;
  `ARCHITECTURAL_DECISIONS.md` ADR-020; Chapter 12 §12.4/§12.8/§12.13; Chapter
  13 §13.3/§13.8; REQ-027; `IMPLEMENTATION_STATUS.md` C4/C5/B16; `work/CURRENT.md`.

## 2026-11-12 — Sprint 031: RFC-016 registry dispatch repair

- **Fixed the E0432 left by `9bb31f4`:** Anchor's `#[program]` macro resolves
  `Context<…>` against the crate root, so the eleven generated
  `__client_accounts_*` modules for the RFC-016 security contexts are now
  re-exported there with `#[doc(hidden)] pub(crate) use security::{…}`
  (`41fe53d`). Plain `pub use` fails with E0365 because the generated modules
  are only crate-public.
- **Formatted the RFC-016 files** that had never been rustfmt'd
  (`src/security.rs`, `tests/security.rs`, `3a6c0a1`); formatting only.
- **Verified:** `cargo check -p terra-registry` passes in 33.6 s;
  `cargo fmt -p terra-registry -- --check` clean. No BPF/SBF suite was run.
- A37 stays `2 – IMPLEMENTED, NOT FULLY VERIFIED`: dispatch resolves, but the
  security instructions still have no BPF tests and are not deployed (B6).
- Evidence: `sprints/031-rfc-016-dispatch-repair.md`; `IMPLEMENTATION_STATUS.md`
  A37; `work/CURRENT.md`.

## 2026-11-12 — Sprint 030: Google-Earth-style immersive atlas

- **Immersive atlas route** (`48286af`, with `d2d3d25` + `cd93c25` follow-ups): `body.atlas-immersive` floats the
  global header and a `100dvh` map; added the glass search pill, left tool rail
  (Layers, Stories, Register, Measure, Share, Fullscreen), basemap quick-switch
  and coordinate status bar. Cesium/Leaflet/Three remain the only renderers.
- **Responsive land panel:** floating 400px panel (440px ≥1800px) on desktop;
  `76dvh` bottom sheet at ≤860px with scrim, Escape/close, and a swipe-down
  handle that dismisses past 90px. Mobile opens with the sheet closed.
- **Search:** bundled coordinates/gazetteer resolve instantly; an explicit
  online Photon row appears when nothing local matches. Fresh visits default to
  satellite imagery (`d2d3d25`); saved Streets/Terrain choices still win.
- **2D↔3D camera preservation:** the Cesium morph captures cartographic camera
  position and restores it on `morphComplete`; repeated toggles stayed at
  `3.8480°, 11.5021° · 400,000 m` in verification.
- **Navigation HUD:** WASDQE + X/Y/Z pads fold behind one dock button on the
  atlas (`collapsibleHud`); DemoPage keeps the always-on HUD.
- **Responsive fixes:** icon-only compact menu, mark-only header ≤560px,
  compact language selector, landscape control row at ≤860px × ≤560px; no
  horizontal overflow at 360×640 … 1920×1080 or 844×390.
- **Verified:** tsc clean; vitest **83/83**; `build:check` PASS (entry 300.2 /
  initial 305.5 / Cesium 1098.5 / total 1866.7 kB gz). Live Pages runs
  `43289772629` + `43290890798` + `43292573392` green; desktop/phone atlas, satellite/streets,
  search, 2D/3D, bottom sheet, `/demo` and `/stories` checked in headless
  Chrome. Only direct-document 404s (Pages SPA fallback) and `/api/v1/*` fail.
- Evidence: `sprints/030-immersive-atlas-ux.md`; `IMPLEMENTATION_STATUS.md`
  D19 + frontend baselines; `work/CURRENT.md`; `terra-web/README.md`.

## 2026-11-12 — Sprint 029: merge `origin/dev` + GitHub Pages publication

- **Merged** local `dev` with `origin/dev` (`92613c8`); upstream low-tier
  building fix `4eb7c5a` plus local Book commits `eef0087`; no conflicts.
- **Published the frontend** at <https://mrobadiahej.github.io/terra/> from
  `dev`: `base=/terra/` (env-driven), `%BASE_URL%` Cesium base, router
  basename, postbuild `404.html` + `.nojekyll`, new
  `.github/workflows/deploy-pages.yml`; Pages enabled with `build_type=workflow`
  and the `github-pages` branch policy extended to `dev`.
- **Fixed base-path assets:** `lib/assets.ts` for `continents.geojson` and GLB
  models (Cesium + Three fallback), router `Link` for playground
  `/transactions` links; bundle gate accepts prefixed asset URLs.
- **Verified live** in headless Chrome: welcome/atlas/demo/stories render with
  keyless Esri/OSM tiles and demo parcels, deep links resolve through
  `404.html`; only `/api/v1/*` returns 404 (API intentionally not deployed).
- `terra-web/README.md` rewritten to the actual route/page/router/IDL state;
  `work/CURRENT.md` reconciled; sprints 027/028 marked committed; evidence
  `sprints/029-merge-and-pages-publication.md`.

## 2026-11-05 — Sprint 025/026: REQ-026/027/028 + RFC-016 implementation; Sprint 027/028: atlas fixes + doc reconciliation

- **REQ-026 implemented** (`IMPLEMENTED, NOT FULLY VERIFIED`): `geo-engine/src/component_package.rs`
  (RFC-015 conformance validator + `terra-geom-v2` 3D digest), migrations 0035
  (`component_packages`) + 0041 (`component_version`), `api/src/routes/components.rs`
  (`POST /components`, `GET /components/{id}`, `GET /components`), `api/src/routes/assemblies.rs`
  (`POST /assemblies`, `GET /assemblies/{id}` with objects + conflicts, list, add objects,
  add conflicts, `PUT /assemblies/{id}/status`), migration 0038.
- **REQ-027 implemented** (`IMPLEMENTED, NOT FULLY VERIFIED`): migration 0036
  (`world_manifests` + `manifest_objects`), `api/src/routes/manifests.rs` (create, get with
  objects, list by region, upsert objects), frontend `terra-web/src/lib/runtime.ts` manifest
  resolver + scene graph manager.
- **REQ-028 implemented** (`IMPLEMENTED, NOT FULLY VERIFIED`): migrations 0037
  (`storage_providers` + `provider_replicas` + `provider_capabilities`), 0039
  (`runtime_scene_graphs` + `scene_nodes` + `asset_cache`), 0040 (`worker_registration`),
  0042 (`derived_job_leases`), 0043 (`scene_node_validation_state`); `api/src/routes/providers.rs`
  (provider discovery + fallback + replication), `api/src/routes/runtime.rs` (`/runtime/scene`,
  `/runtime/scene/{lod}`, `/runtime/assets` with fallback, `/runtime/lod`, `/runtime/cache`),
  `api/src/routes/workers.rs` (worker registration + heartbeat staleness + job assignment),
  frontend `runtime.ts` (LOD 0–4, provider fallback, asset retrieval) + `runtime.test.ts`,
  `WorkerConsolePage.tsx`.
- **RFC-016 on-chain security core implemented** (`IMPLEMENTED, NOT FULLY VERIFIED`):
  `src/security.rs` (1070 lines) — 11 PDAs + 11 instructions + 13 `TerraError` codes,
  registered in `lib.rs`; REQ-029…REQ-040 → `IMPLEMENTED, NOT FULLY VERIFIED`.
- **Sprint 027 — Atlas 16-point requirement audit (frontend fixes):** `ThreeAtlasFallback.tsx`
  restored missing `return <div ref={hostRef}>` host (was returning `void`); `TerraGlobe.tsx`
  typed `ring.map` destructuring + typed runtime accessor for `moveWest/East/South/North/Up/Down`
  + WASD keyboard handler; `GlobePage.tsx` `failedRegionsRef` dedupes `sceneError` banner
  retries + manual "Create manifest" retry clears the failure; removed 3 empty files
  (`ThreeNavOverlay.tsx`, `TerraFlightPad.tsx`, `__write-probe.ts`).
- **Sprint 028 — Doc reconciliation (all `.md` layer-focused):** IMPLEMENTATION_STATUS.md
  audit basis → `244ed7e`/2026-11-05, vitest **83/83**, bundle entry **305.1 kB gz** /
  Cesium **1098.5 kB gz** / total **1864.3 kB gz**, geo-engine **33 (+2 ignored)**; added
  A37–A40 (RFC-016), B15–B19 (REQ-026/027/028 + workers), D12–D18 (runtime client, 3D
  model rendering, atlas verification, progressive tier loading, front-view/single sidebar,
  backend-down graceful degradation, worker console); REQUIREMENTS_TRACEABILITY
  REQ-026/027/028 → `IMPLEMENTED, NOT FULLY VERIFIED`; chapters 11/12/13/14 +
  `rfc-015` + BOOK_OF_INTENTS/SUMMARY → `IMPLEMENTED, NOT FULLY VERIFIED` with
  "why this chapter exists" updated to reflect implemented code; work/CURRENT.md
  reconciled to 2026-11-05; ADR-016 resolved (`terra-geom-v2` integrated); R-012/R-021
  re-verified; ROADMAP stages relabeled; new sprint notes `sprints/025-*.md` …
  `sprints/028-*.md` per the Book's one-`.md`-per-sprint convention.
- Verification this date: API 133/133, geo-engine 33 (+2 ignored), registry lib 153/153,
  vitest 83/83, `tsc -b` clean, `pnpm build:check` PASS, eslint matches pre-existing
  baseline (3 errors + 3 warnings on `GlobePage.tsx`/`TerraGlobe.tsx`, unchanged).
- All REQ-026/027/028 + RFC-016 code is **local-verified only** — B6 devnet funding
  (Q-001, deployer wallet 0 SOL) blocks live-cluster verification.

## 2026-11-02 — RFC-016 Protocol Security, Decentralization, Privacy & Censorship Resilience

- New `docs/rfc-016-protocol-security-decentralization-privacy.md` — full security
  directive: no-single-point-of-control principles; measurable decentralization
  model (12 categories); on-chain security registry (SecurityRequest,
  PrivacyRequest, JurisdictionalRestrictionRequest, SecurityReviewCase,
  PrivacyState, SecurityDecision, SecurityAppeal, SecurityAuditEvent); distributed
  validator review (local+global, verifiable randomization, anti-collusion);
  privacy request lifecycle (authentication → scope → classification → emergency
  check → distributed review → quorum → appeal → finalized); privacy remedies
  and state versioning; scoped jurisdictional restrictions; security incentives
  (rewards, collateral, slashing with due process, reporting, anti-collusion);
  storage independence and censorship resilience; on-chain vs off-chain
  responsibility split; abuse prevention (privacy-as-censorship, proportionality,
  public-interest evidence preservation); audit rule (inspect actual code before
  designing); implementation phases 0–5; acceptance criteria (RFC-016 §11);
  18 non-negotiable design rules.
- Code-grounded gap analysis vs existing mechanisms:
  fraud_governance.rs (random committee, no jail, appeals — fraud-specific only),
  routing.rs (multi-factor + random winner), staking.rs (slashing/appeal/collateral),
  validator_registry.rs (admin bootstrap + `paused` = central points — R-022),
  evidence_manifest.rs (evidence-specific), cross_border.rs (treaty-binding,
  not per-data restriction), recovery.rs (validator injection, not privacy
  protection).
- Registers: IMPLEMENTATION_STATUS (A27/A28/A29 security+privacy specs; B14 API
  privacy enforcement; E5 threat model + gap analysis); RISK_REGISTER (R-022
  centralized governance, R-023 privacy-as-censorship, R-024 jurisdictional
  overreach, R-025 storage unavailability/censorship, R-026 reviewer collusion);
  OPEN_QUESTIONS (Q-013..Q-018); ARCHITECTURAL_DECISIONS (ADR-017 on-chain vs
  off-chain, ADR-018 privacy-state semantics, ADR-019 jurisdictional boundaries);
  ROADMAP Stage 4.5 + verification gates; BOOK_OF_INTENTS index + navigation;
  SUMMARY; AGENT_GUIDE permanent requirements.
- Documentation only — no application code changed; no gate required.

## 2026-11-02 — Chapter 14 Implementation Preparation Plan (REQ-020…REQ-028 code-area mapping)

- New `chapters/14-implementation-preparation.md` — dependency-ordered code-area
  mapping for every missing requirement: foundational changes (component package
  migration + router, manifest migration + router, provider migration + router),
  dependent changes (assembly resolver, scene graph, LOD manager, asset
  retrieval), founder-gated changes (institutional identity, residency, liens),
  and a code-area inventory of what exists to build on (SpatialAsset,
  ThreeDModel, Reconstruction API, Derived compute, Task economics, Routing,
  Geo-engine, Frontend LOD/terrain, API routers, Migrations).
- Dependency order: founder decisions (Q-003/Q-004) → component package → 3D
  digest (terra-geom-v2 integrate-or-deprecate) → manifest → providers →
  assembly → runtime → frontend streaming.
- Explicit "what must NOT be built yet" list (marketplace, speculative token,
  mainnet, terra-geom-v2 in production, fractional investment, institutional
  identity/residency/liens implementation).
- Documentation only — no application code changed; no gate required.

## 2026-11-02 — RFC-015 Spatial Component Package + remaining Part XIV diagrams + Part XVI principles

- New `rfc-015-spatial-component-package.md` — Spatial Component Package schema
  (component ID, physical entity ID, relationships, geometry artifact, format,
  digest algorithm version, CRS, elevation reference, transform, bounding volume,
  resolution/LOD, evidence refs, engine identity/version, provenance,
  accuracy/uncertainty, validation state, version lineage) + 8 conformance rules
  (REQ-026, `SPECIFIED ONLY`).
- Added Diagram 3 — central coordination & decentralized storage (Chapter 12 §12.2.1).
- Added Diagram 6 — component update lifecycle (Chapter 12 §12.9.1).
- Added Diagram 7 — financial lifecycle (Chapter 5 §5.6.1).
- Completed Diagram 8 — dependency roadmap (ROADMAP.md).
- Added Part XVI — 15 non-negotiable design principles to `BOOK_OF_INTENTS.md`.
- All 8 Part XIV technical diagrams now present across Chapters 11, 12, 13, 5 and ROADMAP.
- Documentation only — no application code changed; no gate required.

## 2026-11-02 — Executive summary + Chapter 13 lazy-loading diagram + Chapter 3 traceability (sprint 024 follow-up)

- Added `Executive summary` appendix to `BOOK_OF_INTENTS.md` answering the eight
  master-instruction questions directly: what Terra has, what is missing, what
  is incorrectly described/implemented, how independently stored objects become
  one synchronized world representation, how runtime discovery/lazy loading/
  progressive visualization work, how engines and workers communicate, how the
  financial system pays for work and encourages coverage, what to build first,
  what to not build yet.
- Added `13.2.1 Diagram — global lazy loading levels` (mermaid flowchart,
  L0–L4) to `chapters/13-global-spatial-data-delivery-and-progressive-visualization.md`.
- Concretized `chapters/03-3d-spatial-intelligence.md` §3.9 acceptance criteria:
  SpatialAsset/GeometryVersion anchor mapping for Chapter 11;
  `lod.ts`/`cesiumTerrain.ts` → L0–L4 mapping for Chapter 13; `terra-geom-v2`
  integrate-or-deprecate decision recorded in ADR-016.
- Updated `IMPLEMENTATION_STATUS.md` E4 row to chapters 01–13.
- Documentation only — no application code changed; no gate required.

## 2026-11-02 — Sprint 012: API security response headers

- New `security_headers` middleware in `terra-core/api/src/main.rs`, applied to
  the production router via `axum::middleware::from_fn` (outermost layer, after
  CORS + Trace).
- Headers: `X-Content-Type-Options: nosniff`, `X-Frame-Options: DENY`,
  `Referrer-Policy: no-referrer`, `Permissions-Policy: camera=(), microphone=(),
  geolocation=()`. JSON-only API surface (plus multipart ingest) → no CSP needed.
- Rationale: defense-in-depth on the unauthenticated mirror (CORS locked; R-016
  fail-closed privileged auth) so browsers cannot MIME-sniff, frame, or leak
  referrer context from API responses.
- Evidence: `CARGO_INCREMENTAL=0 cargo test -p terra-api` → **130/130** (was
  **128**; +2 new tests in `main.rs`:
  `tests::ok_response_carries_security_headers`, `tests::error_response_also_carries_security_headers`).
- No SOL required.

## 2026-11-02 — Sprint 013: API request body-size limit

- New `MAX_BODY_BYTES` env config (bytes; default **64 MiB**) in
  `terra-core/api/src/config.rs`, consumed by `RequestBodyLimitLayer`
  (`tower-http` `limit` feature) applied to the production router in
  `terra-core/api/src/main.rs`.
- Oversized bodies → **413 Payload Too Large** (Content-Length pre-check).
- Rationale: defense-in-depth on the unauthenticated mirror; caps request
  flooding into handlers/storage before body reads. Default covers multipart
  OSM ingest (PBF) and photo evidence uploads.
- Evidence: `CARGO_INCREMENTAL=0 cargo test -p terra-api` → **132/132** (was
  **130**; +2 new tests: `tests::oversized_content_length_rejected_with_413`,
  `tests::within_limit_is_accepted`).
- No SOL required.

## 2026-11-02 — Sprint 014: Frontend logic unit tests (LOD + codec)

- New frontend test suites: `terra-web/src/lib/lod.test.ts` (LOD level derivation
  from height + leaflet zoom, clamping, level table) and `terra-web/src/lib/codec.test.ts`
  (byte→hex, hex→bytes, UTF-8 encode).
- Evidence: `pnpm exec vitest run` → **52/52** (was **44/44**; +8 new tests);
  tsc/eslint/vite build/bundle-check all green.
- No SOL required.

## 2026-11-02 — Sprint 015: API request timeout

- New `REQUEST_TIMEOUT_SECS` env config (default **60 s`) in
  `terra-core/api/src/config.rs`, consumed by tower-http `TimeoutLayer` applied
  to the production router (`api/src/main.rs`); default response **408 Request
  Timeout**.
- Rationale: defense-in-depth on the unauthenticated mirror — caps handler
  execution time per request before compute/storage work runs unbounded.
- Evidence: `CARGO_INCREMENTAL=0 cargo test -p terra-api` → **133/133** (was
  **132**; +1 new test: `tests::slow_request_times_out_with_408`).
- No SOL required.

## 2026-11-02 — Sprint 022: Frontend logic unit tests — activity store + API GeoJSON parse

- New frontend test suites: `terra-web/src/lib/activityStore.test.ts` (dedup by
  run-scoped id, newest-first ordering, 200-item cap, source preservation) and
  `terra-web/src/lib/api.test.ts` (`parseGeoJSON` null/valid/invalid handling).
- Evidence: `pnpm exec vitest run` → **65/65** (was **56/56**; +9 new tests);
  tsc/eslint/vite build/bundle-check all green.
- No SOL required.

## 2026-11-02 — Sprint 021: Book consistency catch-up + requirements traceability (REQ-025)

- REQUIREMENTS_TRACEABILITY: added REQ-025 row — API runtime defense-in-depth
  (security headers + body-size limit + request timeout); status 1
  (IMPLEMENTED AND VERIFIED) with source paths, named tests, suite evidence
  (API 133/133). Added in the same change set as the hardening sprints, per the
  acceptance-criteria rule.
- BOOK_OF_INTENTS.md / SUMMARY.md: work & sprints indexes extended with
  sprints 018, 019, 020 (previously stale at sprint 017); SUMMARY REQ range
  refreshed to REQ-001…REQ-025.
- IMPLEMENTATION_STATUS.md: E4 evidence annotation updated (sprints 001–020).
- Doc-only change; no gate required.

## 2026-11-02 — Sprint 020: Frontend/UX polish — welcome offline badge

- `WelcomePage.tsx`: render an offline badge (`t('apiOffline')`) in the hero when
  the activity feed state is `offline`.
- Evidence: frontend full gate green (tsc · eslint · vite build · vitest **56/56** ·
  bundle PASS).

## 2026-11-02 — Sprint 019: Frontend/UX polish — demo header source chip

- DemoPage header: added catalogue-source chip (`scenarioChecking` / `apiCatalogue` /
  `builtinCatalogue`) alongside the existing demo badge and engine-status link.
- Evidence: frontend full gate green (tsc · eslint · vite build · vitest **56/56** ·
  bundle PASS).

## 2026-11-02 — Sprint 018: Book index consistency pass (part 2)

- Central index (`BOOK_OF_INTENTS.md`) and `SUMMARY.md` work & sprints lists extended
  with sprints 016 and 017; E4 evidence annotation updated (sprints 001–017).
- Doc-only change; no gate required.

## 2026-11-02 — Sprint 017: Frontend logic unit tests (protocol status)

- New frontend test suite: `terra-web/src/lib/protocolStatus.test.ts` (status
  levels/tracks, meta tables, `statusCounts` aggregation incl. custom modules).
- Evidence: `pnpm exec vitest run` → **56/56** (was **52/52**; +4 new tests);
  tsc/eslint/vite build/bundle-check green.
- No SOL required.

## 2026-11-02 — Sprint 016: Book index consistency pass

- Central index (`BOOK_OF_INTENTS.md`) and `SUMMARY.md` "Work & sprints" sections
  updated to list sprints 001–015 (previously stale at 001–002).
- Doc-only change; no behavior change, no SOL required.

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

## 2026-11-02 — Research: institutional bootstrap (Q-003) and residency attestation (Q-004)

- Consulted external sources and recorded them in RESEARCH_REFERENCES with
  dates, claims, and fact-vs-interpretation separation (sprint 023).
- **Q-003:** five DNS-anchored identity designs studied (Open-Attestation
  DNS-TXT ADR; IETF Groundmark; Froglet attestation service; HAVID; MetaMail
  DON). Trust-minimising designs converge on: DNS as discovery only + external
  attestation service that re-verifies live + N-of-M consensus + reputation
  gating. A pure DNS-TXT root is weaker than a council and must not be the sole
  gate for `register_institutional_identity`.
- **Q-004:** six jurisdictions surveyed (Portugal, Queensland AU, DC US, New
  York US, Monaco, Luxembourg). Every one requires two independent proofs of
  address from distinct sources and a recency window; **none** issues a
  selective-disclosure credential. A dual-signed residency attestation is a
  novel instrument whose recognition cannot be assumed.
- Both questions moved `6 → 5 – PROPOSED` (research done, founder decision
  pending). REQ-020 and REQ-021 remain `PROPOSED` with no source or test — no
  code written, per the research gates.
- Registers updated: RESEARCH_REFERENCES (2 entries), OPEN_QUESTIONS (Q-003,
  Q-004), chapters/02 §2.3, chapters/08 §8.6, sprints/023.

## 2026-11-02 — Stage 3: planned-capability specifications promoted to `SPECIFIED ONLY`

- Chapters 2 (REQ-021), 6 (REQ-022), and 8 (REQ-020) moved
  `PROPOSED → SPECIFIED ONLY`, each carrying a ⚠ founder review flag; INT-005
  updated likewise. REQUIREMENTS_TRACEABILITY rows for REQ-020/021/022 moved to
  `4 – SPECIFIED ONLY` with the spec paths cited (no source, no test — no code
  written, per the research gates).
- Chapter 5 (REQ-023) stays `PROPOSED` — Q-005 is a hard securities-law gate and
  no specification can substitute for written legal research.
- ROADMAP Stage 3 exit met except founder review of the three flags.
- Doc-only; no gate required.

## 2026-11-02 — Request authentication hardening (R-016/R-017)

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
- Registers updated: RISK_REGISTER (R-016, R-017), work/CURRENT.md, sprints/004.

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

## 2026-11-02 — Bundle discipline gate (R-012)

- `terra-web/scripts/check-bundle.mjs` enforces route-level lazy loading after
  `vite build`: no Cesium in the initial graph, and gzip ceilings for the
  entry chunk (340 kB), initial graph (360 kB), Cesium chunk (1200 kB) and
  total JS (1900 kB).
- Measured: entry **295 kB gz**, initial **301 kB**, Cesium **1098 kB** (lazy),
  total **1678 kB**. A lowered-budget copy exits 1, so the failure path works.
- CI frontend job now runs `pnpm build:check` (it previously type-checked and
  unit-tested but never built the app). R-012 mitigated; D6 added; sprint 010.
  No SOL required.

## 2026-11-02 — Architecture spec sprint 024: distributed reconstruction, world runtime, progressive delivery

- New chapter `chapters/11-distributed-spatial-reconstruction-and-assembly.md` —
  spatial component package (RFC-015 planned), multi-engine reconstruction
  workflow, component registration/alignment/conflict handling, immutable
  versioning and assembly (REQ-026, `SPECIFIED ONLY`).
- New chapter `chapters/12-world-runtime-and-synchronized-rendering.md` —
  control plane vs data plane, World Representation Manifest, spatial index,
  scene graph, synchronized rendering runtime, provider discovery/fallback,
  synchronization/consistency/failure behavior (REQ-027, `SPECIFIED ONLY`).
- New chapter `chapters/13-global-spatial-data-delivery-and-progressive-visualization.md` —
  hierarchical lazy loading levels, spatial request resolution, provider
  discovery, progressive rendering, LOD/streaming formats, device-adaptive
  rendering (REQ-028, `SPECIFIED ONLY`).
- Deep update `chapters/03-3d-spatial-intelligence.md` — spatial object identity
  and relationships, coordinate frames, geometry versioning, AI provenance,
  acceptance criteria.
- Update `chapters/05-fractional-investment.md` — reconstruction financing vs
  fractional ownership; no automatic property rights from compute/funding.
- Update `chapters/10-derived-compute-and-provenance.md` — connection to
  distributed reconstruction, resource metering, evidence lineage.
- Registers: BOOK_OF_INTENTS index, SUMMARY, REQUIREMENTS_TRACEABILITY (REQ-026
  to REQ-028), ROADMAP Stage 3/4, ARCHITECTURAL_DECISIONS (ADR-014/015/016),
  RISK_REGISTER (R-018 to R-021), OPEN_QUESTIONS (founder decisions), CHANGELOG.
- New sprint record `sprints/024-architecture-spec.md`.
- Documentation only — no application code changed; no gate required.
