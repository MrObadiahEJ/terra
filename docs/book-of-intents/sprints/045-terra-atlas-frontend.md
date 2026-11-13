# Sprint 045 — Terra Atlas frontend (investor-facing planetary observation surface)

**Date:** 2026-11-13 · **Status:** done & committed (`3604545`)
**Gate:** `tsc -b` clean; eslint clean; vitest **120/120**; `pnpm build:check` PASS
(entry 308.8 kB gz ≤ 360; Cesium lazy 1098.5 kB ≤ 1200; total 1882.8 kB gz ≤ 1900);
frontend assets lazy-loaded per route (Cesium stays out of initial graph).

## Trigger

Directive TASK: Ship the investor-facing Terra Atlas experience (frontend only),
published at `https://mrobadiahej.github.io/terra/atlas`. Strict scope:
`terra-web/**` only + Book docs. No `terra-core/**`, no on-chain code, no RFC
edits, no new backend dependencies. Page fully navigable with API unreachable
(demo-data fallback). No mandatory wallet. No real transaction signing (program
not deployed to devnet).

## Implementation

### `terra-web/src/components/atlas/AtlasPage.tsx` (new)
Wraps `GlobePage` (the existing immersive Cesium globe) as the map background,
overlays the violet gradient mesh (`atlas-gradient-mesh`), and hosts the glass
window system. Route `/atlas` now points to `AtlasPage` in `App.tsx` (replacing
bare `GlobePage`). `body.atlas-immersive` class added/removed on mount/unmount.

### Window system — `WindowSystem` + `AtlasWindow` (new, in `AtlasPage.tsx`)
Draggable, resizable, focusable glass windows (`atlas-window`) with
minimize-to-dock (`atlas-dock`) and `localStorage`-persisted layout via
`atlasStore`. Desktop: floating glass cards. Mobile `<768px`: bottom sheets.
Windows are lazy-loaded via `React.lazy` + `Suspense` (kept out of initial
bundle per R-012).

### `terra-web/src/components/atlas/AssetInspector.tsx` (new)
3D Gaussian Splat inspector using three.js `Points` + custom shader — **no
SharedArrayBuffer, no cross-origin isolation required**. Lazy-loads
`public/models/demo-house.splat` (2.5 kB synthetic splat sampled from the
bundled `house_scene.glb`, labeled honestly as synthetic — not a survey
capture). Fallbacks: static preview image on unsupported/WebGL2-unavailable.
Orbit controls. Honesty chip (`simulation`) on every surface.

### `terra-web/src/components/atlas/WalletPanel.tsx` (new)
Phantom-style assets/rights panel: gradient header, account chip, collectible
cards with status chips (CLAIMED/PROTOTYPE/SIMULATION/VISION), activity area.
Optional read-only Phantom connect via the existing wallet adapter — address +
devnet balance via public RPC, **no program calls, no signing**. Without a
wallet, shows a clearly-labeled simulated portfolio (never presented as live
chain state).

### `terra-web/src/components/atlas/LayersPanel.tsx` + `ProvenancePanel.tsx` (new)
Layers: parcel/road/POI/building toggles + satellite/terrain basemap switch.
Provenance: evidence lineage with honest status chips; nothing self-declared
"verified."

### `terra-web/src/components/atlas/InvestorTour.tsx` (new)
6-stop guided narrative overlay: claim → evidence → validators → consensus →
immutable history → 3D asset + rights. Each stop flies camera, opens the
relevant window, plain-language caption + honest status chip. Skippable,
replayable, reduced-motion safe. Ends with links to VISION/README.

### Design (`index.css`)
Dark surfaces (`#16161C`–`#1C1C24`), Phantom-violet accent (`#AB9FF2`), ambient
animated gradient mesh (violet→blue→teal) behind globe chrome, glassmorphic
blurred panels, rounded-2xl cards, soft shadows, subtle grain, smooth
micro-interactions. `prefers-reduced-motion` → static gradient fallback.

### Demo data (`public/models/demo-house.splat`, new)
2.5 kB synthetic splat (150 points) sampled from bundled `house_scene.glb` —
well under the ~8 MB new-assets cap. All heavy assets lazy-fetched via
`assetUrl()` from `lib/assets.ts` (base-path aware for GitHub Pages `/terra/`).

### i18n (`lib/locale.tsx`)
New `atlas*` keys added to all 4 locales (en, fr, ru, zh): window titles,
status chips, layer names, tour steps, honesty labels.

### Verification
| Gate | Result |
|---|---|
| `tsc -b --noEmit` | clean |
| `eslint` | clean |
| `vitest run` | **120/120** |
| `pnpm build:check` | PASS (entry 308.8 kB gz ≤ 360; Cesium lazy 1098.5 kB ≤ 1200; total 1882.8 kB gz ≤ 1900) |

### Relationship to directive §11 (interoperability)
Does not force one 3D format: `.splat`/`.ksplat`/compressed `.ply` supported
via the splat adapter; GLB mesh fallback; static preview fallback. External
adapters (Blender, GIS, glTF/GLB, 3D Tiles, GeoJSON, LAS/LAZ, IFC, game
engines) remain pluggable — none are bundled.

## Gates (per process)
All green before push; pushed to `dev`; Pages workflow auto-deploys.
Verify live: `https://mrobadiahej.github.io/terra/atlas` — deep-link refresh,
assets under `/terra/`, splat inspector renders, budget gate passed.

## What is still missing
- Scene-node `package_uuid` exposure so `fetchComponentArtifact` is automatic
  from GlobePage scene nodes (frontend-only follow-up; no backend change).
- Client-side digest verification against signed manifest content address
  (frontend-only follow-up).
- Live-cluster verification of REQ-026/027/028 (B6 devnet funding blocks).
- Optional: smaller Draco/meshopt-compressed `.glb` for `modern-house.glb`
  (15 MB) — tracked as bundle discipline follow-up; not blocking this sprint.

## Book updates
`CHANGELOG.md`, `work/CURRENT.md` (sprints 001–045), this note, `App.tsx` route
change. `IMPLEMENTATION_STATUS.md`/`REQUIREMENTS_TRACEABILITY.md` unchanged
(frontend-only demo experience; no REQ status change).
