# Sprint 040 — Wire Quadtree Discovery + Culling/SSE LOD into GlobePage

**Date:** 2026-11-13 · **Status:** done, committed `3dc2c6d`→Sprint 040 work
**Gate:** `tsc -b --noEmit` clean; vitest **120/120** (was 115/115; +5
`destinationPoint`/`cameraViewBounds` tests); eslint no new errors
(TerraGlobe/GlobePage carry only pre-existing `no-explicit-any`/`refs`
warnings); `pnpm build:check` PASS (entry 300.2 / initial 305.5 / Cesium
1098.5 / total 1868.7 kB gz).

## Trigger

Sprints 038/039 delivered the quadtree discovery client (`getQuadCell`/
`getQuadCover`) and the screen-space-error + frustum/occlusion culling module
(`viewport.ts`), but neither was wired into rendering — `work/CURRENT.md`
items 4/5 listed "derive cover cells from camera bounds in GlobePage scene
requests" and "culling/SSE awaiting GlobePage wiring" as the remaining local
work. This sprint closes that gap: GlobePage now derives cover cells from the
camera frustum and culls scene-graph nodes before they reach `TerraGlobe`,
which additionally skips nodes whose `spatial_extent` does not intersect a
cover cell.

## Implementation

**Frontend — `terra-web/src/lib/viewport.ts`** (added to Sprint 039 module)

- `destinationPoint(lat, lon, bearing, distanceM)` — great-circle destination
  point (WGS84), longitude wrapped to [-180, 180].
- `cameraViewBounds(camera, viewport)` — projects the camera frustum (heading
  as view centre, FOV as left/right extent, pitch-derived near/far distances)
  to a ground bounding box, used to derive quadtree cover cells from camera
  bounds (Chapter 13 §13.3).

**Frontend — `terra-web/src/pages/GlobePage.tsx`**

- `viewportRef` (viewport dims) + `nodeBounds(node)` (bounding box of a
  scene node's `spatial_extent` polygon).
- `refreshCoverCellsForCam(cam)` — `cameraViewBounds` → `getQuadCover`;
  invoked whenever the scene graph is resolved (new region or LOD change).
- `culledNodes` state, computed in an effect on `[sceneGraph, camMoveTick]`
  via `selectVisible(camera, viewport)` — frustum + occlusion + range culling
  with SSE-based LOD; only visible nodes are passed to `TerraGlobe`.
- `coverCells` state passed to `TerraGlobe` as a new prop.

**Frontend — `terra-web/src/components/map/TerraGlobe.tsx`**

- New `coverCells?: QuadCoverRow | null` prop.
- Runtime-node render loop skips any node whose `spatial_extent` bounding box
  does not intersect at least one cover cell (coarse quadtree pre-cull before
  the finer `selectVisible` cull in GlobePage).

**Frontend — `terra-web/src/lib/viewport.test.ts`**: 5 new tests —
`destinationPoint` (north/east/round-trip) and `cameraViewBounds` (valid box,
grows with height, extends in view direction). Total viewport tests: 29.

## Verification

| Check | Result |
| --- | --- |
| `tsc -b --noEmit` | clean |
| `vitest run` | **120/120** (was 115/115; +5 viewport tests) |
| `eslint` (changed files) | no new errors (pre-existing `no-explicit-any`/`refs` warnings in TerraGlobe/GlobePage unchanged) |
| `pnpm build:check` | PASS (entry 300.2 / initial 305.5 / Cesium 1098.5 / total 1868.7 kB gz) |

## What is still missing

- The component package UUID is not exposed through scene nodes, so
  `fetchComponentArtifact` remains a manual fallback rather than automatic
  from `fetchComponentAsset` scene-node fetches.
- Spatial tiling / 3D Tiles pipeline; screen-space-error LOD currently picks a
  LOD number but does not yet drive mesh simplification.
- Live-cluster verification of every item above (B6 devnet funding blocks it).

## Book updates

`IMPLEMENTATION_STATUS.md` (header deltas, D12 row, vitest 120/120 baselines),
`REQUIREMENTS_TRACEABILITY.md` (REQ-028 row), `ROADMAP.md` (Stage 4 item),
`work/CURRENT.md` (items 4–5), `CHANGELOG.md`, this note.