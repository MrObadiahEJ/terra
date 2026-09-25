# Terra Web

React 19 + Vite frontend for Terra — claims, parcels, verification, a Cesium/Leaflet
map view, and the investor progress page.

> Part of Terra — read [`../docs/VISION.md`](../docs/VISION.md) (the north star)
> first, then the [root `../README.md`](../README.md) for repo status.

## Stack

- **React 19** + **TypeScript** + **Vite**
- **CesiumJS** (3D globe) with automatic **2D Leaflet fallback** when WebGL is unavailable (zero new dependencies — `react-leaflet` was already installed; see `components/map/LeafletMap.tsx`)
- **Solana wallet adapter** (`@solana/web3.js`, `@coral-xyz/anchor`)
- **Zustand** state, **React Router** (`BrowserRouter`: `/` → globe, `/progress` → investor demo)
- **pnpm** package manager
- **No CSS framework** — `src/index.css` contains a hand-rolled utility stylesheet (~97 classes) plus app styles; there is no Tailwind and none may be added

## Scripts

```bash
pnpm install --frozen-lockfile

pnpm dev        # Vite dev server
pnpm build      # tsc -b && vite build
pnpm lint       # ESLint
pnpm preview    # Preview production build
pnpm exec tsc --noEmit   # Type check only (CI)
```

If `pnpm` hangs resolving a version on your machine, call the local binaries
directly: `./node_modules/.bin/vite build` (type check via `npx tsc --noEmit`).

## Layout

```
terra-web/src/
├── App.tsx                 # Router shell (BrowserRouter)
├── main.tsx                # Entry
├── pages/                  # Route pages: GlobePage (/), ProgressPage (/progress)
├── components/             # UI + map/globe
│   └── map/                # TerraGlobe (Cesium), LeafletMap (2D fallback)
├── lib/                    # API client (api.ts), progressData.ts (static demo doc)
├── store/                  # Zustand stores
├── idl/                    # terra_registry.json + generated types
├── leaflet.d.ts            # minimal module shim (no @types/leaflet installed)
└── index.css               # hand-rolled utility classes + app styles
```

## Routes

| Route | Page | Purpose |
|-------|------|---------|
| `/` | `GlobePage` → `TerraGlobe` | Cesium 3D globe with parcels/roads/POIs + draw mode; if WebGL fails, shows an amber notice and renders the **2D Leaflet map** instead (same data, fully interactive) |
| `/progress` | `ProgressPage` | Investor/demo document: vision, architecture (3 layers + program IDs), 5 flagship scenarios, live-globe CTA, metrics, roadmap, module catalog, security posture — all content lives in `lib/progressData.ts` |

## IDL

`src/idl/terra_registry.json` is generated from the on-chain program. After
changing `terra-core/programs/terra_registry`, regenerate from the repo:

```bash
cd ../terra-core && make idl
```

**Synced:** checked-in IDL matches source (P0-2 removal path, **2026-09-25**):
**148 instructions / 62 accounts / 135 events / 220 errors**. Re-run `make idl`
(and re-sync `src/idl/` types) before shipping client changes that depend on
new instructions, accounts, events, or error codes.

## Current Status & Next Steps

**Done:** React 19 + Vite shell, typed API client (`src/lib/api.ts`), Cesium globe
with 2D Leaflet fallback on WebGL failure, `/progress` investor page driven by
`lib/progressData.ts`, wallet adapter wiring, hand-rolled utility CSS,
`tsc --noEmit` clean on `dev`.

**Next (for anyone continuing without prior context):**
1. After program edits: `cd ../terra-core && make idl`, then re-sync `src/idl/` types (`terraRegistry.ts` is generated from `terra_registry.json`) — do not hand-edit the JSON.
2. Wire wallet signing to the deployed program IDs (root README / `Anchor.toml`); no live devnet deployment yet.
3. Extend pages/components against new API routes as RFC-012 Phase 9 lands (21 route modules today; list lives in `terra-core/api/src/routes/`).
4. Keep `pnpm exec tsc --noEmit` and `pnpm lint` green (CI runs the typecheck).

Source of truth for protocol behavior: [`../docs/VISION.md`](../docs/VISION.md) →
root `../README.md` → `../terra-core/SECURITY.md` → `docs/rfc-012-…`.

## Environment

Configure Solana cluster / program IDs via wallet adapter and env as needed for
localnet, devnet, or mainnet. Program IDs are listed in the root README and
`terra-core/Anchor.toml`.
