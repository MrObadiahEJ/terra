# Deployment runbook — devnet + free hosting

This is the operational companion to Sprint D (`059-sprint-d-devnet-hosting.md`)
and to `DEEP_AUDIT_REPORT.md`. It lists every credential-gated step so the
founder can complete them in order.

The code side is done: `deploy.sh` verifies deploy keypairs against
`declare_id!`, and `preflight-devnet.sh` reports the rent estimate, balance and
shortfall. Hosting is configured entirely through environment variables, never
committed secrets.

> **Q-005 gate:** the settlement rail pays for verified storage service only
> (`settlement_kind='service_fee'`). Do not add token, fractional-interest or
> profit-share fields to any deployment. The payout **asset** is operator data
> in `settlement_assets`, not a fixed constant: the default is native SOL, and
> USDC/EURC are opt-in rows. Adding another rail is an `INSERT`, not a
> redeploy.

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
| `VRF_ORACLE_PUBKEY` | yes for storage PoP | Base58 storage-PoP oracle key; absent = fail-closed (no challenges, `prove_replica` 503) |
| `TERRA_ZK_MODE` | optional | `disabled` (default), `development`, or `production`; dev proving is loopback-only |
| `TERRA_ZK_VERIFYING_KEY_PATH` | with ZK | Path to the Groth16 verifying key loaded at startup |
| `TERRA_ZK_PROVING_KEY_PATH` | development only | Path to the dev proving key; never deploy this |
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

### Zero-knowledge membership proofs

The on-chain verifier is always enabled; it accepts only a `proof_data` frame
whose 576-byte VK hash equals the value pinned per ownership root via
`update_verification_key_hash`.

- **Development / devnet demo:** run `make zk-dev-setup` in `terra-core/`. It
  writes `.zk/membership.{pk,vk}` and `.zk/env.sh` (gitignored), then prints the
  VK hash. `source .zk/env.sh`, restart the API, and pin the same hash on the
  zone root (`update_verification_key_hash`). `prepare`/`generate` are
  loopback-only; production mode never loads a proving key.
- **Production:** the dev setup is a single-party setup, so it is forgeable.
  Do not use it with real value. Production requires an arkworks Groth16 MPC
  ceremony over `MembershipCircuit` (or a constraint-equivalent circom port
  with an R1CS equivalence test), a published transcript, an external circuit
  audit, and the resulting VK hash pinned on-chain. `scripts/zk-ceremony.sh`
  refuses to run until the circom-port precondition exists. See Q-009.
- Verify key identity with `make zk-info` (prints the frame and its SHA-256).
- The agave `alt_bn128` pairing syscall does not perform G2 subgroup checks;
  treat this as a protocol-level risk until the toolchain enforces it.

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

The rail is asset-agnostic. `settlement_assets` lists enabled payout assets and
their unit counts; exactly one row is the default.

| Asset | Kind | Decimals | Starting rate | Issuer |
|---|---|---|---|---|
| `SOL` | native | 9 (lamports) | 666,667 / GB | none |
| `USDC` | stablecoin | 6 | 100,000 / GB | Circle |
| `EURC` | stablecoin | 6 | 100,000 / GB | Circle |

`SOL` is the default because it has no issuer and no country behind it. USDC and
EURC are available for providers that prefer a stable unit of account; the
issuer can freeze addresses, so they are a convenience rail, not a
censorship-resistance claim. Add any other asset with:

```sql
INSERT INTO settlement_assets
    (currency, display_name, decimals, kind, default_rate_micro_units_per_gb)
VALUES ('XYZ', 'Example asset', 6, 'other', 100000);
```

Retiring a rail is a data change too, but do it in order: reassign its
providers first, then disable it. Otherwise those providers have an effective
currency that can no longer be closed.

```sql
UPDATE storage_providers SET payout_currency = NULL WHERE payout_currency = 'XYZ';
UPDATE settlement_assets SET is_enabled = false WHERE currency = 'XYZ';
```

Keep exactly one enabled row with `is_default = true`; if none is enabled, epoch
closes that omit `currency` fail closed with a 400.

1. List assets: `GET /api/v1/providers/settlement/assets`.
2. Query epochs: `GET /api/v1/providers/settlement/epochs`.
3. Close an epoch (signed by `API_AUTHORITY_PUBKEY`):

   ```http
   POST /api/v1/providers/settlement/epochs
   {"currency": "SOL", "rate_micro_units_per_gb": 666667, "min_payout_micro_units": 0}
   ```

   Omit `currency` to use the default asset. Omit the rate to use the asset's
   `default_rate_micro_units_per_gb`. The response includes each provider leaf
   and the Merkle root.

   An epoch only includes providers whose **effective** currency matches:
   their `payout_currency`, or the registry default when it is unset. Closing a
   non-default asset therefore never pays a provider who did not opt in. To pay
   everyone in SOL and a subset in USDC, close twice: once with `"currency":
   "SOL"` and once with `"currency": "USDC"`.

4. A provider chooses its rail at
   `POST /api/v1/providers/{id}/payout-currency` (signed by its registered
   wallet), or at registration with `payout_currency`. `null` resets to the
   default.
5. A provider fetches its proof at
   `GET /api/v1/providers/settlement/epochs/{epoch_id}/leaves/{provider_id}/proof`
   and claims at `POST /api/v1/providers/settlement/claims` with its registered
   wallet. Claims settle exactly once and the epoch flips to `settled` when all
   leaves are claimed.

Run epoch close on whatever cadence the payout process uses (manual or an
external scheduler). Payout execution remains an operational transfer on the
chosen rail; the API records the reference, it does not move funds. Because the
claim is a signed, Merkle-provable obligation, a frozen address does not erase
the debt: it can be honoured over another rail without changing the proof.

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
