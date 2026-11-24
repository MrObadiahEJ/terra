# Devnet / Hosted Smoke Test

Local checklist for the Terra devnet demo environment. Last verified **2026-11-24**.

## Quick run

```bash
./scripts/devnet-smoke-test.sh
```

Expected: all automated checks pass (`FAIL=0`). Requires `curl`; on-chain checks also require `solana` CLI.

## Environment under test

| Item | Value |
|---|---|
| Frontend (Pages) | https://mrobadiahej.github.io/terra/ |
| API (Render) | https://terra-gscx.onrender.com |
| API health | https://terra-gscx.onrender.com/health |
| Supabase project | `qgtstweelyankeqftepx` |
| Devnet RPC | Helius devnet (configured in Pages secret `VITE_TERRA_RPC_URL`; CLI smoke uses `https://api.devnet.solana.com` by default) |
| Dev wallet / deploy authority | `AK3LmJmQijTJbsqj1PGt1hfXtdHirhNZybhPrYeRPvs3` |
| Registry program | `GaEDbktvpZ3qiqp4PmFgHwDSa6JsFfVjXFqNb2nTbage` |
| Identity program | `HyjpaHY9yQg1KneLZGjpLDQfP9WV4LUEbNZcb7C4Vo6p` |

## Automated smoke checks

| Check | Command / URL | Expected |
|---|---|---|
| API health | `curl -sS https://terra-gscx.onrender.com/health` | `{"status":"ok","database":"up",...}` |
| Mirror audit | `curl -sS https://terra-gscx.onrender.com/api/v1/mirror/audit` | HTTP 200, JSON audit payload |
| Parcels | `curl -sS https://terra-gscx.onrender.com/api/v1/parcels` | HTTP 200, JSON array |
| Stories | `curl -sS https://terra-gscx.onrender.com/api/v1/stories` | HTTP 200, JSON array |
| Providers | `curl -sS https://terra-gscx.onrender.com/api/v1/providers` | HTTP 200, JSON array |
| Settlement assets | `curl -sS https://terra-gscx.onrender.com/api/v1/providers/settlement/assets` | HTTP 200, includes SOL default asset |
| Pages | `curl -I https://mrobadiahej.github.io/terra/` | HTTP 200 |
| Bundle config | Inspect Pages JS bundle | Contains API base, devnet RPC, registry program ID |
| Wallet balance | `solana balance AK3LmJmQijTJbsqj1PGt1hfXtdHirhNZybhPrYeRPvs3 --url devnet` | > 0 SOL (top up before heavy demo) |
| Registry program | `solana program show GaEDbktvpZ3qiqp4PmFgHwDSa6JsFfVjXFqNb2nTbage --url devnet` | Program exists, executable |
| Identity program | `solana program show HyjpaHY9yQg1KneLZGjpLDQfP9WV4LUEbNZcb7C4Vo6p --url devnet` | Program exists, executable |

## Latest automated run (2026-11-24)

Result: **14/14 PASS, 0 FAIL, 0 SKIP** via `./scripts/devnet-smoke-test.sh`.

- API `/health`: OK, DB up
- Mirror audit/parcels/stories/providers/settlement assets: 200
- Pages: 200
- Frontend bundle contains configured API base, devnet RPC, registry program ID
- Wallet: ~0.090390397 SOL
- Registry/identity programs: deployed on devnet

**Caveat:** bundle grep found `localhost`/`localhost:8080` strings in JS assets; manual inspection suggests at least some are benign cluster-detection/library literals, but verify no runtime fallback hits localhost before demo.

## Manual browser-wallet MVP checklist (required)

These cannot be fully verified from CLI. Run on https://mrobadiahej.github.io/terra/ with Phantom set to **Devnet**.

- [ ] Page loads without console config errors; no `localhost` in network requests.
- [ ] Connect Phantom (Devnet) wallet.
- [ ] Register identity/profile if required by flow.
- [ ] Register a parcel / claim flow completes with wallet signature.
- [ ] Upload evidence; evidence appears in UI/API as expected.
- [ ] Pay for verification / fund required escrow or task flow.
- [ ] Validator routing/quorum status surfaces (no "SIMULATION" chip on real MVP path).
- [ ] Geometry/proof export available after verification.
- [ ] Open a dispute on a parcel/flow and confirm UI/status update.
- [ ] Check Explorer links for wallet-signed devnet transactions.

## Additional online tests to run before demo

- [ ] **CORS**: confirm Pages origin can call API from browser (no CORS error in console).
- [ ] **Render cold start**: hit `/health` after idle period; confirm recovery < ~60s or UptimeRobot keeps warm.
- [ ] **Evidence durability**: upload evidence, restart/redeploy Render (if safe), confirm evidence still resolves if S3 configured.
- [ ] **DB migrations/schema**: `mirror/audit` has no drift/unanchored unexpected rows after test actions.
- [ ] **Write/read round trip**: create a small record through UI/API and confirm it appears in list endpoints.
- [ ] **Wallet balance guard**: ensure demo wallet has enough SOL for all planned txs; top up via faucet if needed.
- [ ] **Program ID/config drift**: Pages bundle program IDs match deployed devnet programs.
- [ ] **Mobile/responsive sanity**: open Pages on phone-sized viewport; key CTA/connect flows usable.
- [ ] **Error states**: disconnect wallet / reject signature / bad input; UI shows actionable errors.

## Known constraints

- Full E2E requires a real browser wallet; CLI smoke only proves services/programs are reachable.
- Dev wallet balance is low for extended demos; keep a faucet top-up plan.
- Secrets exposed in chat must be rotated after demo (Helius key, any DB password shared in chat). Do not commit `.env` files.