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
| **B** | Real write path: delete `tx_prep.rs`, IDL/client `program.methods`, fix identity PDAs, sync IDL/types, wire full UI loop | **delivered** | `tx_prep.rs` deleted; `program.methods` write path; identity PDA fix (`identityPda` → `terra_identity`); IDL sync (`idl/terra_registry.json` 176 instructions, `idl/terra_identity.json` 8); commit `76d85df`; note `sprints/057-sprint-b-real-write-path.md` |
| **C** | Chain indexer: `api/src/indexer.rs`, RPC in `AppState`, `chain_verified` mirror upserts, DB-vs-chain audit | **delivered** | `api/src/indexer.rs`; migration `0051_chain_verified_columns.sql`; `GET /mirror/audit` covers 7 tables + `verification_attestations_onchain`; commit `ac754f6`; note `sprints/058-sprint-c-chain-indexer.md` |
| **D** | Devnet hosting: real deploy, hosted API + managed PostGIS + private RPC, env-driven clients, Pages on hosted API, neutral globe focus | planned — **Q-001** | `sprints/059-sprint-d-devnet-hosting.md` |
| **E** | Routing/economics hardening: VRF-verified routing (server-side `GET /geo/route` with `from_node_id`/`to_node_id` topology), anti-spam staging bond, premium-category routing, reliability derived from challenge results | **partially delivered** | `geo-engine/src/reach.rs` `route_between` Dijkstra; `api/migrations/0049_provider_content_address.sql`; `storage_providers.stake_bond` + `wallet_address` (migration 0053); `GET /providers/replication-audit` enforces N-of-M ≥2 provider classes; commits `8cd7aa9`, `0f0a052`; note `sprints/060-sprint-e-routing-economics.md` |
| **F** | Evidence durability: IPFS backend option (`IPFS_API_URL`), quarantine + TTL columns on `evidence_uploads`, `StorageBackend` enum (`Local`/`Ipfs`), honest labeling until paid | **delivered** | `api/src/storage.rs` (IPFS backend, quarantine/TTL); migration `0053_evidence_durability_and_provider_economy.sql`; commit `0f0a052`; note `sprints/061-sprint-f-evidence-durability.md` |
| **G** | Ratified vision features: institutional credentials, residency attestation, lien/encumbrance, one real reconstruction job hashed on-chain | planned — **Q-003/Q-004 ratification; Q-005 hard gate** | `sprints/062-sprint-g-ratified-vision.md` |

## Delivered this cycle (Sprints B, C, E-partially, F)

- **Sprint B** — Real write path: deleted `tx_prep.rs` placeholder builders and route registration; every write now uses client-side IDL `program.methods` with real PDAs, discriminators and signers; fixed `identityPda`/`successionPda` to derive on `terra_identity` program ID; generated and checked in IDLs (`idl/terra_registry.json` 176 instructions, `idl/terra_identity.json` 8 instructions); wired `/console` and parcel panels to the real loop; commit `76d85df`.
- **Sprint C** — Chain indexer: `api/src/indexer.rs` polls `terra_registry` program accounts, decodes Anchor `Parcel` and `VerificationAttestation` accounts by discriminator, upserts mirror rows with `chain_verified=true`, `chain_slot`, `chain_signature`; `GET /mirror/audit` compares DB to chain across 7 tables including `verification_attestations_onchain`; migration `0051_chain_verified_columns.sql` adds `chain_verified`/`chain_slot`/`chain_signature` to all mirror tables; commit `ac754f6`.
- **Sprint E (partial)** — Routing/economics hardening: server-side `GET /geo/route` with `route_between` Dijkstra using `from_node_id`/`to_node_id` node-id topology (migration 0049 adds `content_address` to `provider_replicas` fixing the `prove_replica` column bug); `prove_replica` verifies `SHA-256(challenge + content_address)` via Ed25519 precompile; oracle independence enforced (`oracle ≠ winner/candidate/requester`); `storage_providers.stake_bond` + `wallet_address` (migration 0053) for anti-spam; `GET /providers/replication-audit` enforces N-of-M ≥2 provider classes; `reliability_score` derived from `last_proof_at` challenge results (not self-declared); commits `8cd7aa9`, `0f0a052`.
- **Sprint F** — Evidence durability: `StorageBackend` enum (`Local`/`Ipfs`); IPFS pinning via `IPFS_API_URL`/`IPFS_GATEWAY_URL` env vars; quarantine + TTL columns on `evidence_uploads` (`quarantined`, `quarantine_reason`, `ttl_seconds`, `expires_at`); honest labeling in `storage.rs` until paid IPFS/S3 backend exists; migration `0053_evidence_durability_and_provider_economy.sql`; commit `0f0a052`.

## Deep-audit report

- **`DEEP_AUDIT_REPORT.md`** (new): security architecture, dependency analysis, BPF adversarial test results (307/307 passing, 0 failed — CI run `41474466105` green), authority/pause key map, known gaps, recommendations.

## Known blockers

- **Q-001 devnet funding** — deployer `CCng7wFc…Drsy4m` holds 0 SOL. Blocks
  Sprint D's real devnet deploy and every “stranger on devnet” exit criterion.
  Sprint A (local), B, C, E (partial), and F are zero-SOL and not blocked.
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

1. Confirm CI is green including the `localnet-e2e` job; if red, fix before any new work.
2. Sprint D (founder-gated: Q-001 devnet funding): real devnet deploy, hosted API + managed PostGIS + private RPC, env-driven `TERRA_RPC_URL`/`API_BASE`, Pages on hosted API, neutral globe focus.
3. Sprint E remainder (founder-gated partially): protocol-side candidate discovery (not requester-supplied), premium-category guarantee (≥1 requested-category validator + random reputable fill), worker re-registration regression test.
4. Continue Sprint F remainder if Q-001 unblocks: ≥2 replicas for “verified” evidence status, scheduled PoP challenges.
