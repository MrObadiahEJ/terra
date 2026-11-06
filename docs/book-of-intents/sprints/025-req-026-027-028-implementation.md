# Sprint 025 — REQ-026/027/028 Implementation

**Date:** 2026-11-02/03 · **Status:** done & committed · **Gate:** doc-only, no
test gate required (code carries its own suites)

**Branch:** `dev` · **Commits:** `fa5e939`, `edf4a99`, `3e9f405`, `0d2d711`,
`71d19e8`, `0c33ff5`, `5c55a37`, `9b19de9`, `9109159`, `d58c913`, `c936c9b`,
`c31d10c`, `ba6a4c2`, `b60f192`, `bc450c9`, `244ed7e`

## What

Implement the distributed spatial infrastructure specified in Chapters 11/12/13
(REQ-026/027/028). The architecture was written first (Sprint 024); this sprint
turns it into code.

### REQ-026 — Distributed spatial component package & assembly

- `terra-core/geo-engine/src/component_package.rs` — RFC-015 conformance
  validator (`validate`, `ComponentPackage`), rejects packages that misdeclare
  CRS, elevation reference, provenance, or version lineage before insert.
- `terra-core/geo-engine/src/canonical3d.rs` — `terra-geom-v2` 3D digest
  (elevation ref + vertical units + precision). Integrates the previously
  experimental `terra-3d-spatial-pipeline` branch per ADR-016.
- `terra-core/api/migrations/0035_component_packages.sql` — `component_packages`
  table.
- `terra-core/api/migrations/0041_component_version.sql` — `component_version`
  table, anchoring component versions to `SpatialAsset`/`GeometryVersion`.
- `terra-core/api/src/routes/components.rs` — `POST /components` (submit +
  conformance-validate), `GET /components/{id}`, `GET /components` (list).
- `terra-core/api/src/routes/assemblies.rs` — `POST /assemblies`,
  `GET /assemblies/{id}` (with objects + conflicts), `GET /assemblies` (list),
  `POST /assemblies/{id}/objects`, `POST /assemblies/{id}/conflicts`,
  `PUT /assemblies/{id}/status`.
- `terra-core/api/migrations/0038_assemblies.sql` — `assemblies`,
  `assembly_objects`, `assembly_conflicts` (duplicate geometry, boundary
  mismatch, intersection, terrain inconsistency, elevation datum).

### REQ-027 — World Representation Manifest & central coordination

- `terra-core/api/migrations/0036_world_manifests.sql` — `world_manifests`
  (region, version, parent/neighbor manifests) + `manifest_objects`
  (object_id, component version, assembly version, storage reference, LODs,
  validation state, access rules).
- `terra-core/api/src/routes/manifests.rs` — `POST /manifests`,
  `GET /manifests/{id}` (with objects), `GET /manifests` (list by region),
  `PUT /manifests/{id}/objects` (upsert).
- `terra-web/src/lib/runtime.ts` — frontend manifest resolver + scene graph
  manager + LOD 0–4 + provider fallback + asset retrieval (505 lines).
- `terra-web/src/lib/runtime.test.ts` — 322 lines of client tests.

### REQ-028 — Progressive delivery & runtime discovery

- `terra-core/api/migrations/0037_storage_providers.sql` — `storage_providers`,
  `provider_replicas`, `provider_capabilities`.
- `terra-core/api/migrations/0039_runtime_scene_graphs.sql` — `scene_graphs`,
  `scene_nodes`, `asset_cache`.
- `terra-core/api/migrations/0040_worker_registration.sql` — worker
  registration with capabilities.
- `terra-core/api/migrations/0042_derived_job_leases.sql` — derived-job
  lease/claim with expired-lease recovery.
- `terra-core/api/migrations/0043_scene_node_validation_state.sql` — scene
  node validation state (`missing` / `preliminary` / `independently_valid`).
- `terra-core/api/src/routes/providers.rs` — `GET /providers`,
  `GET /providers/{id}/replicas`; multi-provider storage, fallback, replication.
- `terra-core/api/src/routes/runtime.rs` — `GET /runtime/scene/{manifest_id}`,
  `GET /runtime/scene/{manifest_id}/{lod}`,
  `GET /runtime/assets/{component_id}` (provider fallback),
  `GET /runtime/lod` (LOD 0–4), `POST /runtime/cache`.
- `terra-core/api/src/routes/workers.rs` — worker registration + heartbeat
  staleness detection + job assignment (Stage 5).
- `terra-web/src/pages/WorkerConsolePage.tsx` — worker registration UI + signed
  derived-job claim controls.
- `terra-web/src/lib/lod.ts` — `lodFromHeight`, `TIER_SETTINGS.buildingCap`,
  `selectBuildingsForTier`, `distanceDisplayCondition` (progressive tier loading).
- `terra-web/src/lib/perfTier.ts` — device-adaptive rendering policy.

## Verification

- `cargo test -p terra-api` **133/133**
- `cargo test --lib` (geo-engine) **33 (+2 ignored)**
- `pnpm exec vitest run` **83/83** (incl. new `runtime.test.ts`)
- `pnpm exec tsc -b` clean
- `pnpm build:check` PASS (entry 305.1 kB gz / Cesium 1098.5 kB gz / total
  1864.3 kB gz — all within budget)
- On-chain program compiles clean

## Status after this sprint

REQ-026/027/028 → `IMPLEMENTED, NOT FULLY VERIFIED`. All code is
local-verified. **Live-cluster verification is blocked on B6 devnet funding**
(Q-001, deployer wallet 0 SOL) — not a code blocker.

## Still open after this sprint

- Spatial index service (quadtree/S2/H3 decision pending) — specified, not coded.
- Content-addressed retrieval + signed manifests — specified, not coded.
- Frustum culling, occlusion culling, screen-space error LOD — specified, not coded.
- Spatial tiling / 3D Tiles pipeline — specified, not coded.
- Reconstruction-task economic model — founder decision pending.
- Route reconstruction jobs through component packages (instead of bare drafts).