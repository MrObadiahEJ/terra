# Terra Web

React 19 + Vite frontend for Terra — claims, parcels, verification, a Cesium/Leaflet
map view, and the demo transactions explorer.

> Part of Terra — read [`../docs/VISION.md`](../docs/VISION.md) (the north star)
> first, then the [root `../README.md`](../README.md) for repo status.

## Stack

- **React 19** + **TypeScript** + **Vite**
- **CesiumJS** (3D globe) with automatic **2D Leaflet fallback** when WebGL is unavailable (zero new dependencies — `react-leaflet` was already installed; see `components/map/LeafletMap.tsx`)
- **Solana wallet adapter** (`@solana/web3.js`, `@coral-xyz/anchor`)
- **Zustand** state, **React Router** (`BrowserRouter`: `/` → globe, `/lab` → interactive experiments, `/transactions` → demo chain explorer)
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
├── pages/                  # Route pages: GlobePage (/), LabPage (/lab), TransactionsPage (/transactions)
├── components/             # UI + map/globe
│   └── map/                # TerraGlobe (Cesium), LeafletMap (2D fallback)
├── lib/                    # API client (api.ts), txStore (demo chain feed), labStore (Lab experiments)
├── store/                  # Zustand stores
├── idl/                    # terra_registry.json + generated types
├── leaflet.d.ts            # minimal module shim (no @types/leaflet installed)
└── index.css               # hand-rolled utility classes + app styles
```

## Routes

| Route | Page | Purpose |
|-------|------|---------|
| `/` | `GlobePage` → `TerraGlobe` | Cesium 3D globe with parcels/roads/POIs + draw mode; if WebGL fails, shows an amber notice and renders the **2D Leaflet map** instead (same data, fully interactive) |
| `/lab` | `LabPage` | Three hands-on experiments (`components/lab/`): **Geometry Vault** (init/append/verify with real guard errors + `sha256` digests + borsh byte inspector + **WebGL-free SVG isometric 3D land skeleton**), **Verification pipeline** (claim → evidence → observation → attestations → quorum → challenge, authentic event log), **Cross-border gate** (jurisdiction/binding routing verdicts). Deep-links: `?tab=vault\|pipeline\|crossborder` |
| `/transactions` | `TransactionsPage` | Demo chain explorer: simulated tx feed (signatures, accounts, Anchor-style program logs, exact `TerraError` codes), broadcast composer with failure injection, wallet-signed txs marked `LIVE` + devnet status check + Solana Explorer link, **Land versions tab** (every anchored geometry version through time: anchored → verified states, per-version 3D skeleton), realtime pills for validator API + devnet RPC |

## IDL

`src/idl/terra_registry.json` is generated from the on-chain program. After
changing `terra-core/programs/terra_registry`, regenerate from the repo:

```bash
cd ../terra-core && make idl
```

**Synced:** checked-in IDL matches source (RFC-012 Phase 10, **2026-09-28**):
**161 instructions / 67 accounts / 148 events / 237 errors**. Re-run `make idl`
(and re-sync `src/idl/` types) before shipping client changes that depend on
new instructions, accounts, events, or error codes.

## Current Status & Next Steps

**Done:** React 19 + Vite shell, typed API client (`src/lib/api.ts`), Cesium globe
with 2D Leaflet fallback on WebGL failure, `/lab` interactive experiments (incl. a
WebGL-free SVG isometric 3D land skeleton), `/transactions` demo chain explorer
(simulated feed + land-version timeline + devnet-checked wallet txs), wallet
adapter wiring, hand-rolled utility CSS, `tsc --noEmit` clean on `dev`.
(The old `/progress` investor page was removed 2026-09-30 — its content lives in the root README.)

**Next (for anyone continuing without prior context):**
1. After program edits: `cd ../terra-core && make idl`, then re-sync `src/idl/` types (`terraRegistry.ts` is generated from `terra_registry.json`) — do not hand-edit the JSON.
2. Wire wallet signing to the deployed program IDs (root README / `Anchor.toml`); no live devnet deployment yet.
3. Extend pages/components against new API routes (21 route modules today; list lives in `terra-core/api/src/routes/`); RFC-012 Phases 0–10 are on `dev` — next is devnet deployment + wallet wiring.
4. Keep `pnpm exec tsc --noEmit` and `pnpm lint` green (CI runs the typecheck).

Source of truth for protocol behavior: [`../docs/VISION.md`](../docs/VISION.md) →
root `../README.md` → `../terra-core/SECURITY.md` → `docs/rfc-012-…`.

## Environment

Configure Solana cluster / program IDs via wallet adapter and env as needed for
localnet, devnet, or mainnet. Program IDs are listed in the root README and
`terra-core/Anchor.toml`.
