# Sprint 027 — Atlas 16-Point Requirement Audit (Frontend Fixes)

**Date:** 2026-11-05 · **Status:** done, uncommitted (carried into commit `8583e46`)
**Gate:** `pnpm exec tsc -b` clean · `pnpm exec vitest run` 83/83 · `pnpm build:check`
PASS · eslint matches pre-existing baseline

## Trigger

A 16-point audit of the `/atlas` implementation against the Book of Intents
requirements. Found the frontend in a degraded state: a component returning
`void`, runtime type errors on the Cesium `Camera`, and a `sceneError` banner
that fired on every camera tick when the backend was down.

## Fixes

### `ThreeAtlasFallback.tsx` (new file)
- Removed a corrupted trailing `// PROBE pppp…` line that broke the file.
- Restored the missing `return <div ref={hostRef} className="three-atlas-fallback" />`
  so the component renders a Three.js host instead of returning `void`
  (TypeScript: "not a valid JSX element type").

### `TerraGlobe.tsx`
- Added `[lon]: [number, number]` tuple types to `ring.map` destructuring.
- Replaced direct `camera.moveWest/East/South/North/Up/Down` calls (which the
  Cesium `Camera` type omits at the type level) with a typed runtime accessor,
  used by both the axis-move buttons and the WASD keyboard handler.

### `GlobePage.tsx`
- Added `failedRegionsRef` to the manifest effect so a region retries its manifest
  fetch only once until the region actually changes — stops the `sceneError`
  banner storm on every camera tick when the backend is down.
- Manual "Create manifest" button clears the failure for retry.

### Housekeeping
- Removed 3 empty leftover files with no dangling references:
  `ThreeNavOverlay.tsx`, `TerraFlightPad.tsx`, `__write-probe.ts`.

## Verification

- `tsc -b` clean.
- vitest **83/83**.
- `vite build` PASS — GlobePage **82.4 kB gz**, TerraGlobe **874.2 kB gz**.
- eslint: 3 errors + 3 warnings on `GlobePage.tsx`/`TerraGlobe.tsx`, identical
  to the pre-existing baseline (no regression).

## Status after this sprint

The 16 atlas requirements are satisfied to the level the codebase allows.
Manual visual verification only — no automated visual regression suite exists.
Backend-down graceful degradation (req 13) is now stable via bundled demo data
(`DEMO_PARCELS`, `DEMO_ROADS`, `DEMO_POIS`, `DEMO_BUILDINGS`, `DEMO_GEO_STATS`,
`DEMO_FUSION_STATS`); dev server serves `/atlas` (200), `continents.geojson`
(138 kB), and all 3 GLBs.