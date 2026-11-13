# Implementation Preparation Plan

**Status:** `IMPLEMENTED, NOT FULLY VERIFIED` — the code-area mapping below was
written first (Sprint 024, commit `1ebb6f5`); the foundational items for
REQ-026/027/028 were implemented immediately after (2026-11-02/03, commits
`fa5e939`…`244ed7e`). Items still `SPECIFIED ONLY` are called out per-section.

**See:** REQUIREMENTS_TRACEABILITY.md (REQ-020…REQ-028), Chapter 11, Chapter 12,
Chapter 13, RFC-015.

---

## 1. REQ-026 — Distributed spatial component package & assembly

**Status:** `IMPLEMENTED, NOT FULLY VERIFIED`. The component package schema,
conformance test, component registry, and assembly system all exist and pass
local suites; not yet exercised against a live cluster (B6 devnet funding).

### Implemented (2026-11-02)

| Code area | Change | Evidence |
|---|---|---|
| `docs/rfc-015-spatial-component-package.md` | Schema already specified | — |
| `terra-core/geo-engine/src/component_package.rs` | RFC-015 conformance validator (`validate`, `ComponentPackage`) + `terra-geom-v2` 3D digest | geo-engine 33 (+2 ignored) |
| `terra-core/api/migrations/0035_component_packages.sql` | `component_packages` table | — |
| `terra-core/api/migrations/0041_component_version.sql` | `component_version` table (anchors to SpatialAsset/GeometryVersion) | — |
| `terra-core/api/src/routes/components.rs` | `POST /components` (submit + conformance-validate), `GET /components/{id}`, `GET /components` (list) | API 133/133 |
| `terra-core/api/src/routes/assemblies.rs` | `POST /assemblies`, `GET /assemblies/{id}` (objects + conflicts), `GET /assemblies` (list), `POST /assemblies/{id}/objects`, `POST /assemblies/{id}/conflicts`, `PUT /assemblies/{id}/status` | API 133/133 |

### Still to do

- Route reconstruction jobs through component packages (`complete_job` writes a
  component package instead of a bare draft) — depends on components router.
- Reuse `processing_key` dedup for identical component packages.
- Frontend component-package client (submit, retrieve, validate digest).
- Live-cluster component submission + anchoring (B6).

### Acceptance evidence (when implemented on a live cluster)

- Two different workers submit valid component packages with different internal engines
- Packages pass the conformance validator
- Component versions anchor to `SpatialAsset`/`GeometryVersion` on-chain

---

## 2. REQ-027 — World Representation Manifest & central coordination

**Status:** `IMPLEMENTED, NOT FULLY VERIFIED`. The manifest router, migrations,
and frontend runtime client all exist and pass local suites; not yet exercised
against a live cluster (B6).

### Implemented (2026-11-02)

| Code area | Change | Evidence |
|---|---|---|
| `terra-core/api/migrations/0036_world_manifests.sql` | `world_manifests` (region, version, parent/neighbor manifests) + `manifest_objects` (object_id, component version, assembly version, storage reference, LODs, validation state, access rules) | — |
| `terra-core/api/src/routes/manifests.rs` | `POST /manifests`, `GET /manifests/{id}` (with objects), `GET /manifests` (list by region), `PUT /manifests/{id}/objects` (upsert), `POST /manifests/verify` (signed envelope) | 6 targeted API tests, 2026-11-13 |
| `terra-core/api/src/routes/spatial.rs` | `GET /spatial/cell` + `GET /spatial/cover` over the quadtree | 4 targeted API tests, 2026-11-13 |
| `terra-core/geo-engine/src/spatial_index.rs` | Quadtree discovery cells (`q/level/x/y`), point/parent/children, conservative covering (ADR-020) | geo 48/48 (+2 ignored), 2026-11-13 |
| `terra-core/geo-engine/src/manifest.rs` | `terra.manifest/v1` content-addressed payloads, Ed25519 signatures, object/component references and validation | geo 48/48 (+2 ignored), 2026-11-13 |
| `terra-web/src/lib/runtime.ts` | Manifest resolver + scene graph manager + LOD 0–4 + provider fallback | vitest 83/83 |

### Still to do

- Persist signed `terra.manifest/v1` envelopes and expose content-addressed
  retrieval; the stateless verification endpoint exists, but the database still
  stores unsigned mutable rows.
- Use the quadtree discovery endpoints in the scene/partition path.
- Cache invalidation propagation on manifest version change.
- Live-cluster manifest version transition (B6).

### Acceptance evidence

- Client discovers a building's components via manifest without downloading all geometry
- Manifest version transition keeps old clients on consistent assembly

---

## 3. REQ-028 — Progressive delivery & runtime discovery

**Status:** `IMPLEMENTED, NOT FULLY VERIFIED`. Provider discovery, fallback,
runtime scene graphs, worker registration, and the frontend runtime client all
exist and pass local suites; not yet exercised against a live cluster (B6).

### Implemented (2026-11-02/03)

| Code area | Change | Evidence |
|---|---|---|
| `terra-web/src/lib/lod.ts` | `LOD_LEVELS` + `lodFromHeight` + `TIER_SETTINGS.buildingCap` + `selectBuildingsForTier` + `distanceDisplayCondition` | vitest 83/83 |
| `terra-web/src/lib/cesiumTerrain.ts` | Single terrain provider + retry (unchanged; provider discovery now in backend) | — |
| `terra-core/api/migrations/0037_storage_providers.sql` | `storage_providers` + `provider_replicas` + `provider_capabilities` | — |
| `terra-core/api/migrations/0039_runtime_scene_graphs.sql` | `scene_graphs` + `scene_nodes` + `asset_cache` | — |
| `terra-core/api/migrations/0040_worker_registration.sql` | Worker registration with capabilities | — |
| `terra-core/api/migrations/0042_derived_job_leases.sql` | Derived-job lease/claim with expired-lease recovery | — |
| `terra-core/api/migrations/0043_scene_node_validation_state.sql` | Scene node validation state (`missing`/`preliminary`/`independently_valid`) | — |
| `terra-core/api/src/routes/providers.rs` | `GET /providers`, `GET /providers/{id}/replicas`; multi-provider storage, fallback, replication | API 133/133 |
| `terra-core/api/src/routes/runtime.rs` | `GET /runtime/scene/{manifest_id}`, `GET /runtime/scene/{manifest_id}/{lod}`, `GET /runtime/assets/{component_id}` (provider fallback), `GET /runtime/lod`, `POST /runtime/cache` | API 133/133 |
| `terra-core/api/src/routes/workers.rs` | Worker registration + heartbeat staleness detection + job assignment | API 133/133 |
| `terra-web/src/lib/runtime.ts` + `runtime.test.ts` | Asset retrieval (range requests, cache prioritization, cancellation/retry), provider selection, LOD 0–4 | vitest 83/83 |
| `terra-web/src/pages/WorkerConsolePage.tsx` | Worker registration UI + signed derived-job claim controls | tsc clean, build OK |

### Still to do

- Persist signed manifests and serve their content addresses (the library and
  stateless `POST /manifests/verify` exist; no storage/signing endpoint yet).
- Frustum culling, occlusion culling, screen-space error LOD.
- Use `GET /spatial/cell` + `/spatial/cover` in scene requests and the runtime
  client.
- Live-cluster multi-provider retrieval + provider failover (B6).

### Acceptance evidence

- Runtime retrieves components from separate storage providers
- Runtime does not download unrelated geographic data
- Viewer displays low-detail data before high-detail finishes loading
- Storage-provider failure does not blank out an entire region

---

## 4. REQ-020 / REQ-021 / REQ-022 — Institutional identity, residency credential, lien encumbrance

**Status:** `SPECIFIED ONLY` with founder review flags (Q-003, Q-004). Q-005 hard-gates REQ-023.

### Foundational changes (blocked on founder decisions)

| Code area | Change | Blocker |
|---|---|---|
| `terra-core/programs/terra_registry/src/` | `register_institutional_identity` instruction + `InstitutionalValidator` PDA | Q-003 (bootstrap council — founder) |
| `terra-core/programs/terra_registry/src/` | Residency credential instruction + dual-signed attestation | Q-004 (jurisdiction recognition — founder) |
| `terra-core/programs/terra_registry/src/` | Lien encumbrance lifecycle (builds on RRR Rights PDA A3) | Q-003 (depends on Ch8 institutional identity) |
| `docs/book-of-intents/chapters/08-*.md` | Institutional identity spec already written; needs founder authorization | Q-003 |

### Acceptance evidence
- None until Q-003/Q-004 resolve; no code before founder authorization

---

## 5. Implementation order (dependency-ordered)

Per ROADMAP Stage 3–4 and Chapter 47:

1. **Founder decisions first:** Q-003 (bootstrap council), Q-004 (residency
   recognition) — unblock REQ-020/021/022 specs.
2. **Component package:** RFC-015 conformance validator + `component_packages` +
   `component_version` migrations + `components` router — **done** (2026-11-02).
3. **3D digest:** Integrate-or-deprecate `terra-geom-v2` (ADR-016 decision) —
   **done** (`geo-engine/src/canonical3d.rs`).
4. **Manifest:** `world_manifests` migration + `manifests` router + frontend
   runtime client — **done** (2026-11-02).
5. **Providers:** `storage_providers` migration + `providers` router + provider
   discovery — **done** (2026-11-02).
6. **Assembly:** assembly resolver + conflict detection + incremental assembly —
   **done** (`assemblies.rs`, 2026-11-02).
7. **Runtime:** scene graph + LOD manager + asset retrieval + progressive
   streaming — **done** (`runtime.rs` + `runtime.ts`, 2026-11-02/03).
8. **Frontend:** integrate `LOD_LEVELS`/`cesiumTerrain.ts` with progressive
   tier loading + provider fallback — **done** (`lod.ts`, `perfTier.ts`,
   `runtime.ts`, `GlobePage.tsx`, 2026-11-03).
9. **Workers:** worker registration + heartbeat + derived-job lease/claim —
   **done** (`workers.rs` + `WorkerConsolePage.tsx`, 2026-11-03).

### No-funding work available now

All of the above is implemented and passes local suites. The remaining non-funding
items are:

- Spatial index design decision — **done** (ADR-020: quadtree for control-plane
  discovery; S2/H3 deferred to visualization tiles); `GET /spatial/cell` +
  `/spatial/cover` are live, scene-path wiring remains.
- Manifest architecture founder decision — documented, pending.
- Content-addressed retrieval + signed manifests — **library + stateless
  verification done** (Sprint 032/033: `geo-engine/src/manifest.rs`,
  `POST /manifests/verify`); signed persistence/content-addressed storage
  remains.
- Frustum/occlusion culling + screen-space error LOD — specified, not coded.
- Reconstruction-task economic model — founder decision pending.

---

## 6. Code-area inventory (what exists to build on)

| Area | Existing | Status | Can build on? |
|---|---|---|---|
| SpatialAsset / GeometryVersion | `spatial_asset.rs` (on-chain, append-only, quorum verify) | IMPLEMENTED AND VERIFIED | Yes — component version anchors via migration 0041 |
| ThreeDModel | `three_d_model.rs` (register + verify, off-chain URI) | IMPLEMENTED AND VERIFIED | Yes — model artifact reference pattern |
| Component Package | `geo-engine/src/component_package.rs` (RFC-015 validator) + `components.rs` router + migrations 0035/0041 | IMPLEMENTED, NOT FULLY VERIFIED | Yes — submission + conformance + retrieval |
| World Manifest | `manifests.rs` router + migration 0036 + `runtime.ts` manifest resolver | IMPLEMENTED, NOT FULLY VERIFIED | Yes — manifest CRUD + object upsert |
| Storage Providers | `providers.rs` router + migration 0037 | IMPLEMENTED, NOT FULLY VERIFIED | Yes — discovery + fallback + replicas |
| Assembly | `assemblies.rs` router + migration 0038 | IMPLEMENTED, NOT FULLY VERIFIED | Yes — create + objects + conflicts + status |
| Runtime Scene Graph | `runtime.rs` router + migrations 0039/0043 + `runtime.ts` client | IMPLEMENTED, NOT FULLY VERIFIED | Yes — scene resolution + LOD + asset retrieval |
| Workers | `workers.rs` router + migrations 0040/0042 + `WorkerConsolePage.tsx` | IMPLEMENTED, NOT FULLY VERIFIED | Yes — registration + assignment + lease/claim |
| Reconstruction API | `routes/reconstruction.rs` (jobs, lease, complete, draft, anchor) | PARTIALLY IMPLEMENTED | Yes — component submission baseline; needs component-package routing |
| Derived compute | `routes/derived.rs` (processing_key dedup, recursive-CTE invalidation) | IMPLEMENTED AND VERIFIED | Yes — dedup pattern for component packages; now with lease/claim (0042) |
| Task economics | `task_economics.rs` (FeePolicy, CoverageIncentive, TaskEscrow, RewardAllocation) | IMPLEMENTED AND VERIFIED | Partial — verification tasks only; needs reconstruction task types |
| Routing | `routing.rs` (nine-factor, seeded randomness) | IMPLEMENTED AND VERIFIED | Partial — needs reconstruction-task routing |
| Geo-engine | `canonical.rs` (terra-geom-v1, 2D), `canonical3d.rs` (terra-geom-v2, 3D), `reach.rs`, `graph.rs`, `osm.rs`, `component_package.rs` | IMPLEMENTED / PARTIAL | Yes — 2D + 3D digest baseline; component package validator |
| Frontend | `lod.ts` (LOD_LEVELS, lodFromHeight, TIER_SETTINGS, selectBuildingsForTier), `cesiumTerrain.ts`, `runtime.ts` (manifest resolver, scene graph, LOD 0–4, provider fallback), story engine | IMPLEMENTED AND VERIFIED | Partial — LOD defined + progressive tier loading implemented; no spatial tiling/3D Tiles/frustum culling |
| API routers | 37 routers in `routes/mod.rs` | IMPLEMENTED AND VERIFIED | Yes — add new routers as needed |
| Migrations | 0001–0043 chain | IMPLEMENTED AND VERIFIED | Yes — add component/manifest/provider/runtime/worker migrations |

---

## 7. What must NOT be built yet

| Item | Reason |
|---|---|
| Reconstruction marketplace (offers/bids/settlement for reconstruction tasks) | `task_economics` applies to verification tasks only; new task types needed first |
| Speculative Terra token | Explicitly not a prerequisite (directive §41) |
| Mainnet deployment | Founder decision only (Q-006), after devnet soak + adversarial slashing simulation + external security review |
| `terra-geom-v2` in production | **Resolved 2026-11-02** — integrated (not deprecated): `geo-engine/src/canonical3d.rs` is the production 3D digest path, used by the RFC-015 conformance validator in `component_package.rs`; ADR-016 integrate-or-deprecate decision recorded |
| Fractional investment (REQ-023) | Hard-gated on Q-005 securities research |
| Institutional identity / residency / liens implementation | Founder review flags pending (Q-003/Q-004) |
