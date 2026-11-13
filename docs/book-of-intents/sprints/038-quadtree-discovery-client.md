# Sprint 038 — Wire Quadtree Discovery Cells into the Frontend Runtime Client

**Date:** 2026-11-13 · **Status:** done, uncommitted (`runtime.ts` + `runtime.test.ts`)
**Gate:** `tsc -b --noEmit` clean; vitest **91/91** (was 86/86); eslint no new
errors (runtime.ts carries 8 pre-existing `no-explicit-any` errors, unchanged);
`pnpm build:check` PASS (entry 300.2 / initial 305.5 / Cesium 1098.5 / total
1866.9 kB gz).

## Trigger

Chapter 13 §13.3 tracks the gap: "The runtime client does not call these yet"
(`GET /spatial/cell` + `GET /spatial/cover` exist in `api/src/routes/spatial.rs`
since Sprint 032/033, but the frontend never queries them). `work/CURRENT.md`
item 4 named "derive quadtree cells from camera bounds in scene requests" as the
next local step after data-plane artifact storage (Sprint 037). This sprint
closes the discovery-client half: add `getQuadCell`/`getQuadCover` so the
runtime client can resolve coverage cells for scene requests.

## Implementation

**Frontend — `terra-web/src/lib/runtime.ts`**

- `QuadCellRow` / `QuadCoverRow` types mirroring the API `CellResponse` /
  `CoverResponse` schemas (`id`, `level`, `x`, `y`, `bounds`).
- `getQuadCell(params)` — `GET /spatial/cell?lon=&lat=&level=`, returns the
  quadtree cell containing a point.
- `getQuadCover(params)` — `GET /spatial/cover?min_lon=&min_lat=&max_lon=&max_lat=&level=`,
  returns the cells intersecting a bounds rectangle, south-to-north /
  west-to-east order.
- Both use the existing `request` helper (retries, timeout, `API_BASE`) and are
  stateless — no database dependency on the client side, matching the API's
  control-plane partition lookups.

**Frontend — `terra-web/src/lib/runtime.test.ts`**: 5 new tests —
`getQuadCell` default level + explicit level + 404 handling; `getQuadCover`
bounds request + 500 handling. All fetch is mocked; no network.

## Verification

| Check | Result |
| --- | --- |
| `tsc -b --noEmit` | clean |
| `vitest run` | **91/91** (was 86/86; +5 quadtree discovery tests) |
| `eslint src/lib/runtime.ts src/lib/runtime.test.ts` | no new errors (8 pre-existing `no-explicit-any` in runtime.ts, unchanged) |
| `pnpm build:check` | PASS (entry 300.2 / initial 305.5 / Cesium 1098.5 / total 1866.9 kB gz) |

## What is still missing

- `getQuadCell`/`getQuadCover` are not yet called from `GlobePage` scene
  requests; camera bounds are not converted into cover cells.
- The component package UUID is not exposed through scene nodes, so
  `fetchComponentArtifact` remains a manual fallback rather than automatic from
  `fetchComponentAsset` scene-node fetches.
- Frustum/occlusion culling and screen-space-error LOD (Chapter 13 §13.6) are
  specified but not coded; distance-based LOD (`lodFromHeight`,
  `TIER_SETTINGS.buildingCap`, `selectBuildingsForTier`) exists.
- Live-cluster verification of every item above (B6 devnet funding blocks it).

## Book updates

`IMPLEMENTATION_STATUS.md` (header deltas, vitest 91/91, D12 row), baselines
table (vitest 91/91), `REQUIREMENTS_TRACEABILITY.md` (REQ-028 row),
`work/CURRENT.md` (items 4–5), `CHANGELOG.md`, this note.