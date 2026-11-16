# Sprint A: Surfpool localnet + full lifecycle smoke

**Date:** 2026-11-16 · **Status:** delivered; CI green (run `43855396029`, all
five jobs including Localnet Lifecycle E2E) · **Gate:** 9/9 phases green
against deployed programs on Surfpool v1.7.0; no AVX required.

## Trigger

Sprint A required the whole product loop to run against deployed programs
before any new feature work. The local `solana-test-validator` is unusable on
this machine (no AVX), and the previous TS smoke only covered the RRR ownership
model. The committed registry IDL was stale (165 instructions vs 176 on chain),
so clients could not build RFC-016 instructions at all.

## Delivered

- **IDLs:** `terra-core/idl/terra_registry.json` (176 instructions, sha256
  `8ef100d7…de8e`) and `terra-core/idl/terra_identity.json` (8 instructions,
  sha256 `78f766eb…9865`), checked in. `make idl` and `build.sh` now sync
  generated IDLs into `idl/`; `build.sh` still refreshes the web copy.
- **Runbook:** `terra-core/txtx.yml` + `terra-core/runbooks/deployment/` with
  `instant_surfnet_deployment = true`. This is a Surfnet-only shortcut; the
  real deployment path is exercised on devnet in Sprint D.
- **Smoke:** `terra-core/tests/localnet_e2e.ts` — nine phases:
  1. registry bootstrap;
  2. two validators + profile + availability + reputation;
  3. parcel + ownership right + granted USAGE right;
  4. claim + evidence + two independent observations;
  5. two attestations + quorum verification;
  6. task requirement + fee policy + resource quote + coverage incentive +
     task escrow funding + route + result + reward claim;
  7. spatial asset + geometry version verified 2-of-2 + snapshot;
  8. sale escrow create/deposit/accept + `surfnet_timeTravel` settlement +
     ownership transfer;
  9. second parcel + dispute file + two-validator freeze.
- **Runner:** `scripts/run-localnet-e2e.sh` starts Surfpool offline, creates an
  ephemeral payer/authority keypair when absent, stages IDLs, waits for both
  program accounts, runs the smoke, dumps the log on failure, and cleans up.
- **CI:** `lib-tests` uploads both `.so` artifacts; `localnet-e2e` installs
  pinned Surfpool 1.7.0, downloads the artifacts, and runs the same script.
- **Makefile:** `make test-localnet-e2e`.

## Verification

| Check | Result |
|---|---|
| Local run, normal HOME | **9/9 passing** (11 s) |
| Local run, clean HOME (ephemeral keypair path) | **9/9 passing** (11 s) |
| Escrow settlement | `surfnet_timeTravel` advances past the 3-day settlement window; ownership moves to buyer |
| CI | run `43855396029` — **green**, all five jobs including Localnet Lifecycle E2E |

## Scope note

This covers the Sprint A product loop. Rows of the wider A1–A21 capability
matrix outside that loop (vault shards, staking/slashing, ZK proofs,
subdivision/succession, identity instructions) are still verified only by the
307-test BPF suite and are not yet driven through a client on the local
cluster.

## What is still missing

- Client-side coverage of the remaining A1–A21 rows (folded into Sprint B's
  Playwright loop where they are user-visible).
- Real devnet deploy (Sprint D; Q-001 funding).
