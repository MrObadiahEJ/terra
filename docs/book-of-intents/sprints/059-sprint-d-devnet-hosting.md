# Sprint D: Devnet deployment and hosting

**Date:** planned · **Status:** planned — **Q-001 funding**
**Exit gate:** a stranger with Phantom completes the Sprint B loop on devnet
through a public URL.

## Trigger

Everything so far is local. The deployer wallet holds 0 SOL and the frontend
still points at local endpoints. Nothing proves the product outside this
machine.

## Scope

1. **Funding / rent.** Rotate faucet sources or shrink rent (deploy registry
   first, close stale programs). Record the actual cost in the sprint note.
2. **Deploy.** `./deploy.sh devnet` both programs; verify program IDs, upgrade
   authorities and a real RPC deployment (not the Surfnet cheatcode).
3. **Host.** API + managed PostGIS + private RPC. Secrets stay in the host
   environment, never in the repo.
4. **Env-driven clients.** `TERRA_RPC_URL` and `API_BASE` drive the web build
   and the E2E harness; no hardcoded localhost in production paths.
5. **Pages.** Point the deployed frontend at the hosted API; verify CORS,
   HTTPS and wallet network selection.
6. **Globe focus.** Neutral initial camera (no demo-region bias) and honest
   empty/offline states.
7. **Merge.** Merge `dev` → `main` only after all gates are green.

## Acceptance

- Stranger with Phantom connects, registers a parcel, uploads evidence, pays
  for verification, sees the routed task, sees quorum-verified geometry on the
  globe via the indexer, exports proof and opens a dispute.
- No localhost URLs and no simulated chips on that path.
- Deploy cost and faucet evidence recorded in the note.

## Blocked by

**Q-001** (devnet funding). Sprint B and C are prerequisites for the UI/indexer
parts of the exit criterion.
