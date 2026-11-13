# Sprint 039 — Frustum/Occlusion Culling + Screen-Space-Error LOD

**Date:** 2026-11-13 · **Status:** done, uncommitted (`viewport.ts` + `viewport.test.ts`)
**Gate:** `tsc -b --noEmit` clean; vitest **115/115** (was 91/115); eslint no new
errors (`viewport.ts`/`viewport.test.ts` clean; `runtime.ts` carries 8
pre-existing `no-explicit-any` errors, unchanged); `pnpm build:check` PASS
(entry 300.2 / initial 305.5 / Cesium 1098.5 / total 1866.9 kB gz).

## Trigger

Chapter 13 §13.6 specifies view-dependent loading, frustum culling, occlusion
culling, distance-based LOD, and screen-space error — but nothing was coded
(`work/CURRENT.md` item 5). Distance-based LOD (`lodFromHeight`,
`TIER_SETTINGS.buildingCap`, `selectBuildingsForTier`) existed; screen-space
error and culling did not. This sprint implements them as a pure, dependency-free
module so every function is unit-testable without Cesium or a renderer (weak-machine
friendly — frontend only, no Rust builds).

## Implementation

**Frontend — `terra-web/src/lib/viewport.ts` (new, 283 lines)**

- `Camera` / `Viewport` / `Bounds` / `CullResult` types.
- `screenSpaceError(radiusM, camera, viewport)` — projected pixel size of a
  feature's bounding radius at camera height / FOV.
- `lodFromScreenSpaceError(sse, {minLod, maxLod, sseThreshold, lodStepSse})` —
  clamps LOD to [minLod, maxLod], stepping up as SSE grows.
- `metersPerDegreeLat` / `metersPerDegreeLon` / `haversineDeg` /
  `bearingDeg` / `boundsCenterAndRadius` — WGS84-approximate earth geometry.
- `outsideHorizontalFov` / `outsideVerticalFov` — frustum cull tests.
- `isOccluded` — behind-camera / backface heuristic (no depth buffer in a pure
  module; refined client-side by Cesium).
- `cullComponent(componentId, bounds, camera, viewport, opts)` — combined
  cull + LOD: occlusion → frustum → range priority, returns `{visible, reason,
  screenSpaceError, lod, distanceM}`.
- `selectVisible(components, camera, viewport, opts)` — culls + LOD-selects a
  list, sorted near-first.
- `defaultOptions()` — sane defaults (minLod 0, maxLod 4, SSE threshold 40 px,
  step 20 px, max range 50 km).

**Frontend — `terra-web/src/lib/viewport.test.ts` (new, 304 lines)**: 24 tests —
earth geometry, heading normalisation/delta, bearing, screen-space error
(grows with radius, shrinks with height), LOD clamping, frustum/occlusion/range
cull reasons, near-first sorting, default options.

## Verification

| Check | Result |
| --- | --- |
| `tsc -b --noEmit` | clean |
| `vitest run` | **115/115** (was 91/115; +24 viewport tests) |
| `eslint` (changed files) | no new errors (8 pre-existing `no-explicit-any` in `runtime.ts`, unchanged) |
| `pnpm build:check` | PASS (entry 300.2 / initial 305.5 / Cesium 1098.5 / total 1866.9 kB gz) |

## What is still missing

- `viewport.ts` is not yet wired into `GlobePage` scene requests: camera bounds
  are not converted into quadtree cover cells, and `cullComponent`/`selectVisible`
  are not applied to the fetched runtime nodes before rendering.
- The component package UUID is not exposed through scene nodes, so
  `fetchComponentArtifact` remains a manual fallback.
- Spatial tiling / 3D Tiles pipeline; live-cluster verification of every item
  above (B6 devnet funding blocks it).

## Book updates

`IMPLEMENTATION_STATUS.md` (header deltas, D12 row, vitest 115/115 baselines),
`REQUIREMENTS_TRACEABILITY.md` (REQ-028 row), `ROADMAP.md` (Stage 4 item),
`work/CURRENT.md` (item 5), `CHANGELOG.md`, this note.