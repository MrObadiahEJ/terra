# Sprint 010 — Bundle discipline gate (R-012)

**Loop:** Understand → Measure → Guard → Validate → Record
**Phase derived from evidence:** Test (build gate executed) → Loop (record)

## Why

R-012 recorded that the Cesium widgets chunk (~4.1 MB raw) dominates the
frontend payload, mitigated only by a convention: every page is a `lazy()`
route and map widgets mount through `TerraGlobe`/`StoryPlayer`. Conventions
erode silently — one static `import * as Cesium from 'cesium'` in a shared
component would put the whole engine in the initial graph and nothing would
fail. The CI frontend job also ran `tsc` + `vitest` but never `vite build`,
so even a broken production build could merge.

## Measurement (before)

`pnpm build` on `3ee842f`:

| Chunk | Raw | gzip |
| --- | --- | --- |
| `index-*.js` (entry) | 913.4 KB | 288.4 KB (KiB) / 295 kB |
| `constants-*.js` + `geo-*.js` (modulepreload) | 12.7 KB | 5.2 KB |
| **initial graph** | 926.1 KB | **~301 kB** |
| `widgets-*.js` (Cesium, lazy) | 4015.8 KB | **~1098 kB** |
| all emitted JS | — | ~1678 kB |

The initial chunk contains no Cesium marker (`CesiumJS`/`CesiumWidget`), and
`dist/index.html` preloads only the entry, `constants` and `geo`.

## Guard

`terra-web/scripts/check-bundle.mjs` runs after `vite build` and:

1. reads the initial graph from `dist/index.html` (entry script +
   `modulepreload` links), not from a guessed filename;
2. fails if any initial chunk contains a Cesium marker;
3. fails if the entry chunk or the initial graph exceeds its gzip ceiling;
4. fails if the content-identified Cesium chunk exceeds its ceiling;
5. fails if total emitted JS exceeds its ceiling.

Current ceilings (decimal kB gzip, matching Vite's report):

| Budget | Ceiling | Current |
| --- | --- | --- |
| entry chunk | 340 kB | 295.3 kB |
| initial graph | 360 kB | 300.6 kB |
| Cesium chunk | 1200 kB | 1098.3 kB |
| total JS | 1900 kB | 1678.1 kB |

Raising a ceiling is a deliberate decision: update the constant **and** record
why in the Book (R-012). The script prints a per-chunk table on every run so
growth is visible even when it stays under budget.

Wired as `pnpm build:check` and added to the CI frontend job after vitest, so
CI now also proves the production build works.

## Test (evidence)

- `pnpm build:check` → build green; `PASS bundle discipline holds`;
  entry 295.3 kB / initial 300.6 kB / Cesium 1098.3 kB / total 1678.1 kB.
- Negative test: a copy of the script with the entry/initial ceilings lowered
  to 100 kB exited **1** and named both violated budgets — the failure path
  is live, not decorative.
- Cesium is still a lazy chunk: not referenced by `dist/index.html`, so the
  welcome/stories routes never download it.

## Loop

- R-012 → mitigated (not resolved: dependency growth can still raise the
  floor, and budgets must be reviewed on Cesium upgrades per R-007).
- D6 added to IMPLEMENTATION_STATUS; E1 now includes build + budget gate;
  baseline table records the measured sizes.
- Not changed: no dependency upgrades, no rewrite of the wallet stack. The
  entry chunk (~295 kB gz) carries React, router, wallet adapter and web3;
  shrinking it would mean deferring the wallet provider, a product decision
  not needed to close R-012.
- No funding required; this is pure frontend/CI work.
