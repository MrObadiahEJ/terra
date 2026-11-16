# Current Work Ledger

Authoritative record of active execution. Phase derives from evidence here.
Last reconciled: 2026-11-16. Previous ledger archived at
`work/archive/CURRENT-2026-11-15.md` (sprints 001–055).

## Objective

Turn Terra from a spec-heavy, local-only repository into a devnet product.
MVP done means a stranger can: connect Phantom, register a parcel, upload
evidence, pay for verification, be routed to validators, see quorum-verified
geometry on the globe through an indexer, export proof, and open a dispute —
with no “SIMULATION” chips on any step of that path.

## Execution roadmap (Sprints A–G)

| Sprint | Scope | State | Evidence |
|---|---|---|---|
| **A** | Repeatable Surfpool localnet + full lifecycle smoke (parcel → rights → claim → evidence → observation → attestation → quorum → task → fund → route → reward → geometry → snapshot → escrow → dispute) | **delivered; CI green** | `terra-core/tests/localnet_e2e.ts` 9/9 local; `scripts/run-localnet-e2e.sh`; CI run `43855396029` green (all five jobs incl. Localnet Lifecycle E2E); commit `fd89d89`; note `sprints/056-sprint-a-localnet-lifecycle.md` |
| **B** | Real write path: delete `tx_prep.rs`, IDL/client `program.methods`, fix identity PDAs, sync IDL/types, wire full UI loop | **next** | plan in `sprints/057-sprint-b-real-write-path.md` |
| **C** | Chain indexer: `api/src/indexer.rs`, RPC in `AppState`, `chain_verified` mirror upserts, DB-vs-chain audit | planned | `sprints/058-sprint-c-chain-indexer.md` |
| **D** | Devnet hosting: real deploy, hosted API + managed PostGIS + private RPC, env-driven clients, Pages on hosted API, neutral globe focus | planned — **Q-001** | `sprints/059-sprint-d-devnet-hosting.md` |
| **E** | Routing/economics hardening: staging bond, reputation-weighted throughput, protocol candidate discovery, premium category, VRF ADR, worker re-registration regression | planned | `sprints/060-sprint-e-routing-economics.md` |
| **F** | Evidence durability: S3 + pinning, quarantine intake, retention/TTL/purge, PoP challenges, ≥2 replicas for “verified” | planned | `sprints/061-sprint-f-evidence-durability.md` |
| **G** | Ratified vision features: institutional credentials, residency attestation, lien/encumbrance, one real reconstruction job hashed on-chain | planned — **Q-003/Q-004 ratification; Q-005 hard gate** | `sprints/062-sprint-g-ratified-vision.md` |

## Delivered this cycle (Sprint A)

- **Committed generated IDLs** in `terra-core/idl/`:
  - `terra_registry.json` — 176 instructions (11 RFC-016 instructions added
    since the stale 165-instruction copy), program `GaEDbkt…Tbage`,
    sha256 `8ef100d7…de8e`.
  - `terra_identity.json` — 8 instructions, program `HyjpaHY…Vo6p`,
    sha256 `78f766eb…9865`.
  - `make idl` / `build.sh` now sync generated IDLs into the tracked `idl/`
    directory; the web copy is refreshed by `build.sh`.
- **Pinned Surfpool runbook** (`terra-core/txtx.yml`,
  `terra-core/runbooks/deployment/`) with `instant_surfnet_deployment = true`.
  This is a Surfnet-only deployment shortcut; the real RPC deploy path is
  exercised on devnet in Sprint D.
- **`terra-core/tests/localnet_e2e.ts`** — nine passing phases against deployed
  programs, built entirely from the IDL and client-side `program.methods`:
  1. registry bootstrap;
  2. two validators with profile, availability, reputation;
  3. parcel + canonical ownership right + granted USAGE right;
  4. claim + evidence + two observations;
  5. two attestations + quorum verification;
  6. task requirement + fee policy + quote + coverage + escrow funding +
     route + result + reward claim;
  7. spatial asset + geometry version verified 2-of-2 + snapshot;
  8. sale escrow create/deposit/accept, `surfnet_timeTravel` settlement,
     ownership moves to buyer;
  9. second parcel: dispute file + two-validator freeze.
- **`terra-core/scripts/run-localnet-e2e.sh`** — starts Surfpool offline,
  creates an ephemeral payer/authority keypair when `~/.config/solana/id.json`
  is absent, stages IDLs, waits for both program accounts, runs the smoke,
  dumps the Surfpool log on failure, and tears down.
- **CI** — `lib-tests` uploads the SBF artifacts; new `localnet-e2e` job
  installs pinned Surfpool 1.7.0, downloads the artifacts, and runs the same
  script. No AVX needed.
- **Book** — REQ-053…REQ-059 added, sprint notes 056–062 written, this ledger
  rewritten.

### Sprint A scope note (honest coverage)

The required product loop above is exercised end-to-end on a deployed local
cluster. The wider A1–A21 capability matrix is not one test: rows outside the
product loop (vault shards, staking/slashing, ZK ownership proofs, subdivision/
succession, identity program instructions) remain covered by the 307-test BPF
suite and are not yet driven through a client on the local cluster. They stay
listed as `VERIFIED` only for their BPF coverage, not as live-cluster evidence.

## Known blockers

- **Q-001 devnet funding** — deployer `CCng7wFc…Drsy4m` holds 0 SOL. Blocks
  Sprint D's real devnet deploy and every “stranger on devnet” exit criterion.
  Sprint A (local), B, C, and most of E/F are zero-SOL and not blocked.
- **Q-003 / Q-004 founder ratification** — blocks Sprint G implementation
  (institutional identity, residency attestation).
- **Q-005 securities hard gate** — no token, fee-yield, or profit-sharing work;
  no code before a written legal memo and counsel sign-off.
- **No AVX on this machine** — local `solana-test-validator` and SBF builds are
  unavailable; Surfpool covers local execution, CI remains authoritative for
  SBF builds.
- **Disk headroom** — keep `target/debug` pruned (`make prune`); CI artifacts
  are the source of `.so` files for local E2E.

## Next up

1. Confirm CI `43855396029` is green, including the `localnet-e2e` job; if red,
   fix before any new work.
2. Sprint B, in order:
   - delete `api/src/routes/tx_prep.rs` and its route wiring;
   - move every write to client-side IDL `program.methods` with real PDAs,
     discriminators and signers;
   - fix `terra-web/src/lib/program.ts` `identityPda` / `successionPda` to the
     `terra_identity` program ID;
   - generate and check in `target/types` + refresh `terra-web/src/idl`;
   - wire the `/console` and parcel panels to the real loop;
   - add a Playwright E2E that completes the loop on localnet with zero
     simulated steps.
3. Then Sprint C (indexer + audit), keeping each step behind a green gate.
