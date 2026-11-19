# Deployment runbook — devnet + free hosting

This is the operational companion to Sprint D (`059-sprint-d-devnet-hosting.md`)
and to `DEEP_AUDIT_REPORT.md`. It lists every credential-gated step so the
founder can complete them in order.

The code side is done: `deploy.sh` verifies deploy keypairs against
`declare_id!`, and `preflight-devnet.sh` reports the rent estimate, balance and
shortfall. Hosting is configured entirely through environment variables, never
committed secrets.

> **Q-005 gate:** the settlement rail is USDC service-fee only
> (`currency='USDC'`, `settlement_kind='service_fee'`). Do not add token,
> fractional-interest or profit-share fields to any deployment.

---

## 1. Devnet program deployment (Q-001)

### 1.1 Fund the deployer

1. Create/choose the deployer keypair (`DEPLOY_KEYPAIR`, default
   `~/.config/solana/id.json`).
2. Back it up somewhere durable. Losing it loses the program upgrade authority.
3. Fund it from https://faucet.solana.com. The registry program is the large
   artifact (~3.4 MB); the preflight prints the exact shortfall:

   ```bash
   cd terra-core
   ./preflight-devnet.sh
   ```

   Approximate devnet cost at the time of writing: **~15.7 SOL** for registry
   rent plus identity rent and transaction fees. Do not deploy before the
   preflight reports `Preflight PASSED`.

### 1.2 Back up program keypairs

`target/deploy/*-keypair.json` determines the program address. The repository
already carries the expected addresses:
`GaEDbktvpZ3qiqp4PmFgHwDSa6JsFfVjXFqNb2nTbage` (registry) and
`HyjpaHY9yQg1KneLZGjpLDQfP9WV4LUEbNZcb7C4Vo6p` (identity). Copy both keypairs
to `terra-core/deploy-backup/` (and an offline secret store) before deploying.
`deploy.sh` fails fast if a keypair does not match `declare_id!`.

### 1.3 Deploy

```bash
cd terra-core
./deploy.sh devnet
```

Order matters: identity first, then registry (registry CPIs into identity).
Record the transaction signatures and `solana program show <PROGRAM_ID> --url
devnet` output in the sprint note.

### 1.4 Verify

```bash
solana program show GaEDbktvpZ3qiqp4PmFgHwDSa6JsFfVjXFqNb2nTbage --url devnet
solana program show HyjpaHY9yQg1KneLZGjpLDQfP9WV4LUEbNZcb7C4Vo6p --url devnet
```

Publish the Explorer links for both programs and one wallet-signed transaction
from the public site (Sprint D acceptance).

---

## 2. Managed PostGIS (Supabase or Neon)

1. Create a free project on Supabase (Postgres + PostGIS) or Neon.
2. Enable extensions:

   ```sql
   CREATE EXTENSION IF NOT EXISTS postgis;
   CREATE EXTENSION IF NOT EXISTS pgcrypto;
   ```

3. Copy the pooled connection string into `DATABASE_URL` and append
   `?sslmode=require` if the provider does not already include it.
4. Apply migrations from a trusted machine (or as Render's pre-deploy step):

   ```bash
   cd terra-core
   DATABASE_URL='postgres://...' sqlx migrate run --source api/migrations
   ```

Migrations are plain SQL (no compile-time `query!` macros), so no
`SQLX_OFFLINE` cache is needed.

---

## 3. Render API (free web service)

Create a Web Service from the repository:

| Setting | Value |
|---|---|
| Root directory | `terra-core` |
| Build command | `cargo build --release -p terra-api` |
| Start command | `./target/release/terra-api` |
| Health check path | `/health` |

Render injects `PORT`; the API binds `HOST=0.0.0.0` and reads `PORT`.

### Environment variables

Set these in the Render dashboard (see
`terra-core/api/.env.example` for the full annotated list):

| Variable | Required | Notes |
|---|---|---|
| `DATABASE_URL` | yes | Managed PostGIS URL |
| `TERRA_RPC_URL` | yes | Helius/QuickNode devnet endpoint (public RPC is rate-limited) |
| `API_AUTHORITY_PUBKEY` | yes for privileged routes | Base58 Ed25519 key; absent = fail-closed |
| `CORS_ALLOWED_ORIGINS` | yes | `https://<user>.github.io` (comma-separated, no trailing slash) |
| `VRF_ORACLE_PUBKEY` | recommended | Base58 storage-PoP oracle key; absent = legacy SHA-256 proof |
| `S3_BUCKET` | recommended | Private bucket |
| `S3_ENDPOINT` | recommended | e.g. `https://s3.filebase.com` |
| `S3_ACCESS_KEY` | recommended | Bucket credential |
| `S3_SECRET_KEY` | recommended | Bucket credential |
| `S3_REGION` | optional | Default `us-east-1` |
| `RUST_LOG` | optional | `info` |

### Evidence durability

Render's disk is ephemeral. Without `S3_*`, uploaded evidence disappears on
restart and is not shared across replicas. Set all four S3 values before
publishing the demo; the backend is selected at startup and logs
`evidence storage: s3`.

### Maintenance loop

The API owns TTL purge and scheduled PoP challenges (Render has no cron). A
background task runs every 15 minutes and is safe to run on multiple replicas:
purging is idempotent and a pending PoP challenge is never replaced.

### UptimeRobot

Create an HTTP(s) monitor for:

```
https://<service>.onrender.com/health
```

`/health` touches the database (`SELECT true`), so it satisfies the
DB-touching ping requirement and also keeps the free dyno warm.

---

## 4. Frontend (GitHub Pages or any static host)

Build with the API URL baked in:

```bash
cd terra-web
VITE_API_BASE='https://<service>.onrender.com/api/v1' \
VITE_TERRA_RPC_URL='https://api.devnet.solana.com' \
pnpm install --frozen-lockfile
pnpm build:check
```

Deploy `terra-web/dist` to Pages (or Cloudflare Pages/Vercel). Ensure:

- `CORS_ALLOWED_ORIGINS` on the API includes the exact Pages origin.
- The Pages URL is HTTPS (wallet adapters require a secure context).
- No `localhost` appears in the built bundle — grep `dist/` before publishing.

---

## 5. Settlement rail operations

1. Query usage: `GET /api/v1/providers/settlement/epochs`.
2. Close an epoch (signed by `API_AUTHORITY_PUBKEY`):

   ```http
   POST /api/v1/providers/settlement/epochs
   {"rate_micro_usdc_per_gb": 100000, "min_payout_micro_usdc": 0}
   ```

   Default rate is 100,000 micro-USDC/GB ($0.10/GB). The response includes each
   provider leaf and the Merkle root.
3. A provider fetches its proof at
   `GET /api/v1/providers/settlement/epochs/{epoch_id}/leaves/{provider_id}/proof`
   and claims at `POST /api/v1/providers/settlement/claims` with its registered
   wallet. Claims settle exactly once and the epoch flips to `settled` when all
   leaves are claimed.

Run epoch close on whatever cadence the payout process uses (manual or an
external scheduler). Payout execution remains an operational USDC transfer; the
API records the reference, it does not move funds.

---

## 6. Go-live checklist

- [ ] `./preflight-devnet.sh` reports `Preflight PASSED`
- [ ] Both programs deployed; Explorer links recorded
- [ ] Migrations applied to managed PostGIS
- [ ] Render `/health` returns `{"status":"ok","database":"up"}`
- [ ] `CORS_ALLOWED_ORIGINS` matches the Pages origin
- [ ] S3 bucket configured; evidence survives a Render restart
- [ ] UptimeRobot monitor green
- [ ] Pages bundle contains no `localhost`
- [ ] Wallet-signed devnet transaction visible from the public site
- [ ] CI green on `dev`; then merge `dev` → `main` with branch protection
