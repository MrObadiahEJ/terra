# Terra Web

React 19 + Vite frontend for Terra — the public atlas, claims and verification
surfaces, the scenario player, protocol consoles, and spatial stories.

> Part of Terra — read [`../docs/VISION.md`](../docs/VISION.md) (the north star)
> first, then the [root `../README.md`](../README.md) for repo status.

**Live demo:** <https://mrobadiahej.github.io/terra/> — published from `dev`
by `.github/workflows/deploy-pages.yml`. The off-chain API is not deployed with
the site; every page degrades to its demo/empty state when `/api/v1` is
unreachable (see [Deployment](#deployment-github-pages)).

## Stack

- **React 19** + **TypeScript** + **Vite**
- **CesiumJS** 3D atlas with a keyless **2D Leaflet** fallback and a
  WebGL-free **Three.js** fallback (`components/map/`)
- **Solana wallet adapter** (`@solana/web3.js`, `@coral-xyz/anchor`)
- **Zustand** state, **React Router** (`BrowserRouter`, basename from
  `import.meta.env.BASE_URL`)
- **pnpm** package manager
- **No CSS framework** — `src/index.css` contains a hand-rolled utility stylesheet
  plus app styles; there is no Tailwind and none may be added

## Scripts

```bash
pnpm install --frozen-lockfile

pnpm dev        # Vite dev server (serves at /)
pnpm build      # tsc -b && vite build && postbuild (404.html + .nojekyll)
pnpm build:check # build + bundle-budget gate (R-012)
pnpm lint       # ESLint
pnpm test       # vitest run
pnpm preview    # Preview production build
```

If `pnpm` hangs resolving a version on your machine, call the local binaries
directly: `./node_modules/.bin/vite build` (type check via `npx tsc --noEmit`).

## Layout

```
terra-web/src/
├── App.tsx                 # Router shell (BrowserRouter, 14 routes)
├── main.tsx                # Entry
├── pages/                  # 13 route pages (see Routes)
├── components/
│   ├── layout/             # Navbar, OneSidebar
│   ├── map/                # TerraGlobe (Cesium), LeafletMap (2D), ThreeAtlasFallback, LodLadder
│   ├── story/              # Story player / export
│   └── lab/                # Protocol playground modules
├── lib/                    # API clients (api, runtime, evidence, spatial, story),
│                           # atlas layers, LOD/tier logic, locale, wallet
├── store/                  # Zustand stores
├── idl/                    # terra_registry.json + generated types
└── index.css               # hand-rolled utility classes + app styles
```

## Routes

| Route | Page | Purpose |
|-------|------|---------|
| `/` | → `/welcome` | Redirect |
| `/welcome` | `WelcomePage` | Landing page with atlas preview and offline/API status |
| `/atlas` | `GlobePage` → `TerraGlobe` | Primary 3D atlas: keyless OSM / Esri World Imagery / OpenTopoMap basemaps, 2D+3D views, layer loading by LOD, parcel/building selection, navigation HUD (WASDQE + X/Y/Z), single sidebar. Deep links: `?parcel=…`, `?c=lon,lat,height`. Falls back to Leaflet (2D) or Three.js when WebGL/Cesium fails |
| `/lab` | `LabPage` | Interactive experiments (`components/lab/`); deep-link `?tab=vault\|pipeline\|crossborder` |
| `/lab/land` | `LandDetailPage` | Land record detail / viewer |
| `/demo` | `DemoPage` | Seeded scenario player (land registration, verification, cross-border, dispute, validator routing, spatial update, fraud review, recovery); deterministic browser preview when the scenario API is offline. Deep links: `?scenario=…&seed=…` |
| `/network` | `NetworkPage` | Validator/worker network view |
| `/portfolio` | `PortfolioPage` | Wallet holdings and land assets |
| `/status` | `StatusPage` | Protocol + API + devnet status |
| `/transactions` | `TransactionsPage` | Demo chain explorer: simulated tx feed, failure injection, wallet-signed txs, land-version timeline |
| `/console` | `ProtocolConsolePage` | Protocol operation console |
| `/worker` | `WorkerConsolePage` | Worker registration, job assignment and derived-job management |
| `/stories` | `StoriesPage` | Spatial story catalogue (bundled offline demos + saved stories) |
| `/story/:storyId` | `StoryPage` | Story playback, geometry history, replay/export |

Unknown paths and `/terra/<route>` deep links resolve through `404.html`
(SPA fallback) and redirect to `/welcome` when unmatched.

## Atlas data and basemaps

- Basemaps are **keyless**: Streets → OpenStreetMap, Satellite → Esri World
  Imagery, Terrain → OpenTopoMap. Cesium Ion stays disabled
  (`Cesium.Ion.defaultAccessToken = ''`); an optional token can be set per
  browser in localStorage for world terrain.
- Layers load progressively by camera height (`lib/lod.ts`, `lib/perfTier.ts`):
  roads/parcels/POIs/buildings appear at their own LODs, and GLB houses stream
  in only inside 4 km on medium+ tiers (low keeps a small building budget).
- The atlas never requires the backend: bundled demo parcels/buildings keep it
  navigable, and API failures surface as status chips rather than blank canvases.

## IDL

`src/idl/terra_registry.json` is generated from the on-chain program. After
changing `terra-core/programs/terra_registry`, regenerate from the repo:

```bash
cd ../terra-core && make idl
```

**Synced:** checked-in IDL matches source (2026-11-05):
**165 instructions / 71 accounts / 152 events / 247 errors**. Re-run `make idl`
(and re-sync `src/idl/` types) before shipping client changes that depend on
new instructions, accounts, events, or error codes.

## Deployment (GitHub Pages)

The site is a project page served under `/terra/`:

- `vite.config.ts` reads `TERRA_BASE_PATH` (the deploy workflow sets
  `/terra/`); local `pnpm dev` / `pnpm preview` stay at `/`.
- `index.html` derives `CESIUM_BASE_URL` from `%BASE_URL%`, and runtime assets
  go through `lib/assets.ts` (`import.meta.env.BASE_URL`).
- `scripts/postbuild.mjs` copies `index.html` to `404.html` (SPA deep links) and
  writes `.nojekyll`.
- `.github/workflows/deploy-pages.yml` builds `terra-web` on every push to
  `dev` that touches it, then uploads and deploys the artifact. The
  `github-pages` environment allows both `main` and `dev`.

The published bundle has no off-chain API. Keep demo/empty-state behaviour
working for every page: the atlas uses bundled data and keyless tiles, the
scenario player runs its deterministic local preview, and stories fall back to
bundled demos.

## Current Status & Next Steps

**Done:** router shell and 13 pages, typed API clients, keyless Cesium atlas
with Leaflet/Three fallbacks, LOD/tier streaming, 3D model rendering, scenario
player, protocol console, worker console, spatial stories, wallet adapter
wiring, hand-rolled utility CSS, GitHub Pages deployment.

**Next (for anyone continuing without prior context):**

1. After program edits: `cd ../terra-core && make idl`, then re-sync `src/idl/`
   types (`terraRegistry.ts` is generated from `terra_registry.json`) — do not
   hand-edit the JSON.
2. Deploy the off-chain API and point `API_BASE` (`lib/constants.ts`) at it;
   until then the Pages build runs backend-less.
3. The Three.js fallback still needs the nav HUD/flight-pad/keyboard controls
   that Cesium has (focus and pick work already).
4. Keep `pnpm exec tsc -b`, `pnpm test` and `pnpm build:check` green (CI runs
   the typecheck, tests and bundle budget).

Source of truth for protocol behavior: [`../docs/VISION.md`](../docs/VISION.md) →
root `../README.md` → `../terra-core/SECURITY.md` → `docs/rfc-012-…`.

## Environment

Configure Solana cluster / program IDs via wallet adapter and env as needed for
localnet, devnet, or mainnet. Program IDs are listed in the root README and
`terra-core/Anchor.toml`.
