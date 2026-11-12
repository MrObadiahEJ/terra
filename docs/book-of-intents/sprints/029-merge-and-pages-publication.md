# Sprint 029 — Merge with origin/dev + GitHub Pages Frontend Publication

**Date:** 2026-11-12 · **Status:** done & committed (`92613c8`, `4b18f30`, `3ffab13`)
**Gate:** frontend — `tsc -b` clean, vitest pass, `build:check` PASS; deployment verified live

## Trigger

`dev` had diverged from `origin/dev` (1 ahead / 1 behind) and the frontend had
no public host. The founder needs a working, browser-only demo for local and
international investors before any backend deployment. Scope: frontend only.

## Merge

- `git merge origin/dev` → `92613c8`. Upstream carried `4eb7c5a fix(atlas):
  keep 3D buildings on the low graphics tier`; local carried the sprint 025–028
  Book commits (`eef0087`). No conflicts; the uncommitted `terra-registry`
  work stayed untouched in the working tree and was not part of the merge.
- `dev` pushed to origin after the merge.

## Publication

- `terra-web/vite.config.ts` — `base` comes from `TERRA_BASE_PATH` (deploy sets
  `/terra/`; dev/preview stay `/`).
- `terra-web/index.html` — `%BASE_URL%` drives `CESIUM_BASE_URL` and the favicon
  so Cesium workers/assets resolve under the project-site prefix.
- `terra-web/src/App.tsx` — `BrowserRouter` basename derived from
  `import.meta.env.BASE_URL`.
- `terra-web/scripts/postbuild.mjs` — copies `dist/index.html` to `404.html`
  (SPA deep links) and writes `.nojekyll`; wired into `pnpm build`.
- `terra-web/scripts/check-bundle.mjs` — accepts base-prefixed `/assets/…`
  references so the R-012 gate keeps working for the Pages build.
- `terra-web/src/lib/assets.ts` — `assetUrl()` via `import.meta.env.BASE_URL`;
  used for `continents.geojson` and the GLB house models in both Cesium and the
  Three.js fallback. Lab playground links to `/transactions` now use router
  `Link` (basename-aware).
- `.github/workflows/deploy-pages.yml` — builds `terra-web` on pushes to `dev`
  that touch it (plus `workflow_dispatch`), then uploads and deploys the Pages
  artifact. Pages was enabled with `build_type=workflow`; the `github-pages`
  environment branch policy now allows `main` **and** `dev`.

**Live:** <https://mrobadiahej.github.io/terra/> — deploy runs `43263687324`
(initial) and `43265003528` (asset-path fix), both green.

## Verification (live site, headless Chrome)

- `/terra/` and `/terra/welcome` render; API-offline chips shown, no blank page.
- `/terra/atlas?parcel=Soa%20Sample%20Lot%2006` loads the Cesium globe with
  keyless Esri satellite imagery (and OSM streets by default), renders the
  demo parcels, selects the parcel, and shows the single sidebar + navigation
  HUD. Deep links work through `404.html`.
- `/terra/demo` runs the deterministic local scenario preview; `/terra/stories`
  lists bundled demo stories and reports the API as unreachable.
- Third-party tile requests succeed; the only 404s are the intentionally absent
  `/api/v1/*` endpoints. `continents.geojson`, `models/*.glb` and
  `cesium/Workers/*` all return 200 under `/terra/`.

## Evidence

- `tsc -b` clean; vitest pass (2.6 s); `pnpm build:check` PASS with entry
  **299.8 kB gz**, initial **305.1 kB gz**, lazy Cesium **1098.5 kB gz**,
  total **1864.2 kB gz**.
- `terra-web/README.md` rewritten to the actual 14-route / 13-page / 33-router /
  IDL 165-71-152-247 state, including the Pages deployment contract.

## Known limits

- The off-chain API is **not** deployed with the site; all pages must keep
  their offline/demo behaviour. `lib/constants.ts` still uses `/api/v1`.
- The Three.js fallback still lacks the Cesium nav HUD/flight-pad/keyboard
  controls (focus and picking work).
- The Rust CI jobs on `dev` were already failing before this sprint
  (uncommitted `terra_registry` work and pre-existing gate failures); the Pages
  workflow only runs frontend steps and does not depend on them.

## Book updates

`work/CURRENT.md` reconciled to 2026-11-12 (sprints 027/028 marked committed,
sprint 029 added) and `CHANGELOG.md` appended. Historical sprint notes were left
untouched.
