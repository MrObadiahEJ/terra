# Sprint 030 — Google-Earth-Style Immersive Atlas

**Date:** 2026-11-12 · **Status:** done, committed (`48286af`, `d2d3d25`) and deployed
(`43289772629`, `43290890798`)
**Gate:** frontend — `tsc -b` clean, vitest **83/83**, `build:check` PASS;
desktop/tablet/phone layout sweep and live Pages verification

## Trigger

The founder asked for the Terra frontend to feel like
[earth.google.com](https://earth.google.com): a map-first, full-viewport
experience with floating search and panels, while keeping Terra's existing
features and its Cesium/Leaflet/Three rendering stack. The work had to be
verified on phones and large screens, committed, and published to
<https://mrobadiahej.github.io/terra/>.

## Scope and design

Only the atlas *chrome* changed. Cesium remains the 3D renderer, Leaflet the 2D
renderer, and Three.js the WebGL-free fallback; no rendering engine, tile
contract, or atlas data path was replaced. The atlas route adds
`body.atlas-immersive`, which hides document scroll and floats a `100dvh` map
under the global header.

- **Floating glass chrome** — global header, search pill, left tool rail,
  basemap quick-switch, circular zoom/2D-3D/locate controls and a coordinate
  status bar (lat, lon, altitude, zoom detail). All use the existing light/dark
  CSS variables.
- **Search** — `AtlasSearchPill` resolves coordinates and the bundled
  gazetteer instantly, and shows an explicit online-search row (Photon, keyless)
  when no local result matches. The panel's search remains available too.
- **Land panel** — floats at 400px on desktop (440px ≥1800px) and becomes a
  `76dvh` bottom sheet on ≤860px, with a drag handle, scrim, Escape and close
  button. Swiping the handle down >90px dismisses it; shorter drags spring back.
- **Feature integration** — the rail exposes Layers, Stories, Register,
  Measure, Share and Fullscreen; Stories also joins the global nav. The panel
  keeps Register / Browse / Plan / Verification / Media / Scene.
- **2D ↔ 3D** — the Cesium morph now captures the camera in cartographic space
  and restores it on `morphComplete`. Before this sprint a switch fell back to
  the whole planet; repeated toggles now keep the selected parcel in view.
- **Navigation HUD** — the WASDQE pad and X/Y/Z axis pad fold behind one dock
  button in the atlas (`collapsibleHud`); DemoPage keeps the always-visible HUD.
- **Responsive chrome** — icon-only compact menu, mark-only header ≤560px,
  compact language selector, landscape control row on short phones, and no
  horizontal overflow from 360px to 1920px.
- **Sharing/fullscreen** — share links are base-path aware and fall back to the
  current camera (`?c=lon,lat,height`); fullscreen request rejections are
  ignored safely.
- **Default basemap** — fresh visits open on satellite imagery, matching Google
  Earth; a saved Streets/Terrain choice still wins.

## Verification

### Layout sweep (headless Chrome + SwiftShader)

| Viewport | Result |
| --- | --- |
| 1920×1080, 1600×1000, 1280×800, 1024×768, 900×800 | No horizontal overflow; search, rail, panel, controls and dock stay disjoint (e.g. 1600: search 76–596, panel 1186–1586, controls 1033–1170, dock 14–54) |
| 768×1024 | Header wordmark + compact menu; land panel closed by default; map controls clear the header |
| 390×844, 360×640 | Mark-only header, icon menu, search pill, map controls; panel closed by default; dock above the Cesium credit |
| 844×390 landscape | Controls switch to a bottom row (467–834, y304–344), clear of the header/search |

The mobile bottom sheet computed `8–382 × 216–836` at 390×844; swipe-down
closed it, a 30px drag left it open. The HUD toggle showed/hid the axis and
flight pads; `DemoPage` still rendered the always-on HUD with no toggle.

### Interaction checks

- Search `Yaound` → local row “Yaoundé, Cameroon”; status became
  `3.8480°, 11.5021° · 400,000 m · Zoom detail 2`.
- Two full 2D→3D→2D→3D cycles kept `3.8480°, 11.5021° · 400,000 m` each time.
- Satellite, Streets and Terrain basemaps all render with keyless tiles.
- Rail Stories link and the global Stories entry open `/stories`; `/demo` still
  runs its local scenario preview.

### Gates

- `pnpm exec tsc -b` — clean.
- `pnpm exec vitest run` — **83/83** across 10 files (2.5 s).
- `TERRA_BASE_PATH=/terra/ pnpm build:check` — **PASS**: entry **300.2 kB gz**,
  initial **305.5 kB gz**, Cesium **1098.5 kB gz** lazy, total **1866.7 kB gz**
  (budgets 360 / 1200 / 1900).
- `pnpm lint` — the repository's pre-existing 17 errors remain (any-typed
  runtime payloads and React-compiler rules in `TerraGlobe`, `GlobePage`,
  `runtime.ts`, `WorkerConsolePage`); this sprint added none. Lint is not part
  of the CI or Pages gates.

### Live publication

Pages runs `43289772629` (immersive atlas) and `43290890798` (satellite
default) are green. Headless Chrome against the live site confirmed:

- `/atlas?parcel=Soa%20Sample%20Lot%2006` boots with the Cesium canvas, the
  floating header/search/rail/panel/dock, and **Satellite** active by default.
- Desktop and phone screenshots show keyless Esri satellite tiles, demo parcel
  outlines and the selected parcel, with no horizontal overflow.
- Switching to Streets loads OSM tiles; searching and 2D/3D toggles behave as
  above; the bottom sheet opens at 390×844.
- `/demo` and `/stories` render; the only failing requests are the direct
  document load (`/atlas`, `/welcome`, … return HTTP 404 from GitHub Pages even
  though `404.html` renders the app) and the intentionally absent `/api/v1/*`.

## Known limits

- GitHub Pages cannot return HTTP 200 for SPA deep links; `404.html` serves the
  app shell, so clients still receive a 404 status on the first document.
- The API is not deployed with the site. Pages remain offline/demo-first, and
  the online search fallback needs external network access to Photon.
- The 2D/3D restore preserves location and returns north-up; a 3D tilt is not
  carried through 2D.
- Software WebGL in headless Chrome runs at ~1–2 fps, so tile pop-in can take
  seconds in verification screenshots; GPU browsers are substantially faster.
- Fullscreen depends on browser permission and user-gesture rules.

## Book updates

`work/CURRENT.md` reconciled, `IMPLEMENTATION_STATUS.md` D19/E4 and the
frontend baselines updated, `CHANGELOG.md` appended, and the sprint indexes
(`SUMMARY.md`, `BOOK_OF_INTENTS.md`) extended with this note.
