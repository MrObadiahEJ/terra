# Terra Web

React 19 + Vite frontend for Terra — claims, parcels, verification, and a Cesium/Leaflet map view.

## Stack

- **React 19** + **TypeScript** + **Vite**
- **CesiumJS** (globe) and **Leaflet** (map)
- **Solana wallet adapter** (`@solana/web3.js`, `@coral-xyz/anchor`)
- **Zustand** state, **React Router**
- **pnpm** package manager

## Scripts

```bash
pnpm install --frozen-lockfile

pnpm dev        # Vite dev server
pnpm build      # tsc -b && vite build
pnpm lint       # ESLint
pnpm preview    # Preview production build
pnpm exec tsc --noEmit   # Type check only (CI)
```

## Layout

```
terra-web/src/
├── App.tsx           # Router shell
├── main.tsx          # Entry
├── pages/            # Route pages
├── components/       # UI + map/globe
├── lib/              # API client (api.ts), helpers
├── store/            # Zustand stores
├── idl/              # terra_registry.json + generated types
└── index.css
```

## IDL

`src/idl/terra_registry.json` is generated from the on-chain program. After
changing `terra-core/programs/terra_registry`, regenerate from the repo:

```bash
cd ../terra-core && make idl
```

**Known lag:** checked-in IDL currently reflects an older program (≈74
instructions / 23 accounts / 58 events / 120 errors) while source is **118 /
47 / 108 / 160**. Always run `make idl` before shipping client changes that
depend on new instructions, accounts, events, or error codes.

## Current Status & Next Steps

**Done:** React 19 + Vite shell, typed API client (`src/lib/api.ts`), Cesium
globe + Leaflet map, wallet adapter wiring, `tsc --noEmit` clean on `dev`.

**Next (for anyone continuing without prior context):**
1. Run `cd ../terra-core && make idl`, then re-sync `src/idl/` types — do not hand-edit the JSON.
2. Wire wallet signing to the deployed program IDs (root README / `Anchor.toml`); no live devnet deployment yet.
3. Extend pages/components against new API routes as RFC-012 Phase 2+ lands (23 routes today; list lives in `terra-core/api/src/routes/`).
4. Keep `pnpm exec tsc --noEmit` and `pnpm lint` green (CI runs the typecheck).

Source of truth for protocol behavior: root `README.md` → `terra-core/SECURITY.md` → `docs/rfc-012-…`.

## Environment

Configure Solana cluster / program IDs via wallet adapter and env as needed for
localnet, devnet, or mainnet. Program IDs are listed in the root README and
`terra-core/Anchor.toml`.
