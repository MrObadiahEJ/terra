# Sprint 001 — Story Engine vertical slice

**Loop:** Research → Plan → Build → Test
**Phase derived from evidence below:** Test (green) → ready to Loop

## Research

Terra could display parcels and 3D reconstructions but had no way to *experience*
a place over time. The gap: a deterministic, replayable, shareable spatial
narrative ("journey", "history", "discovery") that any client reproduces
identically, plus a browser-side video export. No Three.js — extend Cesium.

## Plan

1. Story schema (Rust + TS mirror): 2–20 stops, ≤300 s, monotonic keyframes.
2. Deterministic camera timeline: smoothstep easing, shortest-arc heading, wall-clock rAF playback.
3. Own minimal Cesium player (do not reuse the heavy TerraGlobe), WebGL-honest fallback.
4. Deep links (`?time=` beats `?step=`), share-copy, offline demo stories.
5. Client-side `MediaRecorder` export with aspect presets + export receipts.

## Build

- Backend: migration `0032_stories.sql` (stories + story_exports); routes CRUD +
  export receipts + validation (9 unit tests); registered via `.merge()`.
- Frontend: `terra-web/src/story/{types,timeline,url,api,demo,export}.ts`,
  `components/story/StoryPlayer.tsx`, `pages/{StoryPage,StoriesPage}.tsx`,
  App routes, locale keys ×4 (en/fr/ru/zh), CSS.
- Fixed en-route: Cesium `ScreenSpaceCameraController` has no `.enabled`
  (used `enableRotate/Translate/Zoom/Tilt`); `react-hooks/set-state-in-effect`
  (deferred via `queueMicrotask`); `no-useless-assignment` on a catch-block reset.

## Test (evidence)

| Gate | Command | Result |
| --- | --- | --- |
| Types | `pnpm exec tsc --noEmit -p tsconfig.app.json` | clean |
| Lint | `pnpm exec eslint .` | clean |
| Build | `pnpm build` | success (StoryPage 14 kB lazy chunk) |
| API | `CARGO_INCREMENTAL=0 cargo test -p terra-api` | 108 passing |

## Loop

Story Engine slice is `realized` and mergeable. Carry forward: road-graph routing
(`GET /geo/route`), journey builder, history-story provenance → next sprint.
