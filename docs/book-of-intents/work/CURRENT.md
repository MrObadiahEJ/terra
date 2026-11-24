# Current Work Ledger

Authoritative record of active execution. Phase derives from evidence here.
Last reconciled: 2026-11-24. Previous ledger archived at
`work/archive/CURRENT-2026-11-16.md` (sprints 001–062).

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
| **D** | Devnet hosting: real deploy, hosted API + managed PostGIS + private RPC, env-driven clients, Pages on hosted API, neutral globe focus | **in progress / partially delivered** | Render API `https://terra-gscx.onrender.com` healthy; Pages `https://mrobadiahej.github.io/terra/`; devnet programs deployed (registry `GaEDbk...bage`, identity `HyjpaH...Vo6p`); Supabase `qgtstweelyankeqftepx` migrated 0001–0056; browser-wallet E2E still pending; `sprints/059-sprint-d-devnet-hosting.md` |
| **E** | Routing/economics hardening: VRF-verified routing (server-side `GET /geo/route` with `from_node_id`/`to_node_id` topology), anti-spam staging bond, premium-category routing, reliability derived from challenge results | **partially delivered** | `geo-engine/src/reach.rs` `route_between` Dijkstra; `api/migrations/0049_provider_content_address.sql`; `storage_providers.stake_bond` + `wallet_address` (migration 0053); `GET /providers/replication-audit` enforces N-of-M ≥2 provider classes; commits `8cd7aa9`, `0f0a052`; note `sprints/060-sprint-e-routing-economics.md` |
| **F** | Evidence durability: IPFS backend option (`IPFS_API_URL`), quarantine + TTL columns on `evidence_uploads`, `StorageBackend` enum (`Local`/`Ipfs`), honest labeling until paid | **delivered** | `api/src/storage.rs` (IPFS backend, quarantine/TTL); migration `0053_evidence_durability_and_provider_economy.sql`; commit `0f0a052`; note `sprints/061-sprint-f-evidence-durability.md` |
| **G** | Ratified vision features: institutional credentials, residency attestation, lien/encumbrance, one real reconstruction job hashed on-chain | planned — **Q-003/Q-004 ratification; Q-005 hard gate** | `sprints/062-sprint-g-ratified-vision.md` |

## Delivered this cycle (Sprints B, C, E-partially, F)

- **Sprint B** — Real write path: deleted `tx_prep.rs` placeholder builders and route registration; every write now uses client-side IDL `program.methods` with real PDAs, discriminators and signers; fixed `identityPda`/`successionPda` to derive on `terra_identity` program ID; generated and checked in IDLs (`idl/terra_registry.json` 176 instructions, `idl/terra_identity.json` 8 instructions); wired `/console` and parcel panels to the real loop; commit `76d85df`.
- **Sprint C** — Chain indexer: `api/src/indexer.rs` polls `terra_registry` program accounts, decodes Anchor `Parcel` and `VerificationAttestation` accounts by discriminator, upserts mirror rows with `chain_verified=true`, `chain_slot`, `chain_signature`; `GET /mirror/audit` compares DB to chain across 7 tables including `verification_attestations_onchain`; migration `0051_chain_verified_columns.sql` adds `chain_verified`/`chain_slot`/`chain_signature` to all mirror tables; commit `ac754f6`.
- **Sprint E (partial)** — Routing/economics hardening: server-side `GET /geo/route` with `route_between` Dijkstra using `from_node_id`/`to_node_id` node-id topology (migration 0049 adds `content_address` to `provider_replicas` fixing the `prove_replica` column bug); `prove_replica` **fails closed** (503) without `VRF_ORACLE_PUBKEY` — verifies Ed25519 oracle signature over `challenge + content_address`; legacy SHA-256 fallback removed; oracle independence enforced (`oracle ≠ winner/candidate/requester`); `storage_providers.stake_bond` + `wallet_address` (migration 0053) for anti-spam; `GET /providers/replication-audit` enforces N-of-M ≥2 provider classes; `reliability_score` derived from `last_proof_at` challenge results (not self-declared); commits `8cd7aa9`, `0f0a052`.
- **Sprint F** — Evidence durability: `StorageBackend` enum (`Local`/`Ipfs`); IPFS pinning via `IPFS_API_URL`/`IPFS_GATEWAY_URL` env vars; quarantine + TTL columns on `evidence_uploads` (`quarantined`, `quarantine_reason`, `ttl_seconds`, `expires_at`); honest labeling in `storage.rs` until paid IPFS/S3 backend exists; migration `0053_evidence_durability_and_provider_economy.sql`; commit `0f0a052`.
- **ZK (out-of-sprint)** — Full Groth16 prover→verifier E2E: `zk_membership_prover_verifier_e2e` registers a zone set, pins VK hash, generates an ownership root, arkworks produces a real Groth16 proof over `terra-zk-membership::MembershipCircuit`, and on-chain `verify_ownership_proof` accepts via `alt_bn128` pairing; tampered proofs rejected. `IMPLEMENTATION_STATUS.md` A14 → `1 – IMPLEMENTED AND VERIFIED`; BPF suite now **308/308**; `ARCHITECTURAL_DECISIONS.md` ADR-025.
- **Edition 2024 migration** (`1d495d5`): all 5 workspace crates migrated from Rust edition 2021 to 2024 — `terra-core/Cargo.toml` (workspace.package), `programs/terra_registry/Cargo.toml`, `programs/terra_identity/Cargo.toml`, `api/Cargo.toml`, `geo-engine/Cargo.toml`, `zk-membership/Cargo.toml`. `cargo check --workspace` clean; all tests green (registry lib 164/164, API 166/166, identity 23/23, geo 45/45+1, zk-membership 6/6).
- **MVP demo release** (`15af9fc`): terra-web v0.1.0 — `/demo` scenario player, `/mvp` Intent 5 flow, `/verify`, `/atlas`, `/lab` all functional; `tsc` clean, vitest 138/138, build:check PASS (entry 303 KB gz, Cesium 1098 KB gz, total 1908 KB gz). Built for GitHub Pages at `TERRA_BASE_PATH=/terra/`.

## Deep-audit report

- **`DEEP_AUDIT_REPORT.md`** (updated): security architecture, dependency analysis, BPF adversarial test results (308/308 passing, 0 failed — CI run `41474466105` green), authority/pause key map, known gaps, recommendations.

## Known blockers

- **Q-001 devnet funding / Sprint D hosting** — deployer `AK3LmJmQijTJbsqj1PGt1hfXtdHirhNZybhPrYeRPvs3`
  is funded (~0.09 SOL on devnet at last check) and programs are deployed
  (`terra_registry` `GaEDbktvpZ3qiqp4PmFgHwDSa6JsFfVjXFqNb2nTbage`,
  `terra_identity` `HyjpaHY9yQg1KneLZGjpLDQfP9WV4LUEbNZcb7C4Vo6p`). Hosted API
  is live on Render (`https://terra-gscx.onrender.com`, `/health` OK, DB up),
  frontend on GitHub Pages (`https://mrobadiahej.github.io/terra/`), Supabase
  project `qgtstweelyankeqftepx` migrated through 0056. Remaining Sprint D
  work: full founder signoff on “stranger on devnet” exit criteria and
  browser-wallet E2E validation. Keep faucet balance topped up for demo txs.
- **Supabase project** — `qgtstweelyankeqftepx`; enable `postgis` + `pgcrypto`
  extensions, copy pooled `DATABASE_URL` + `?sslmode=require` into Render.
- **Helius devnet RPC** — use `https://devnet.helius-rpc.com/?api-key=<KEY>&rebate-address=AK3LmJmQijTJbsqj1PGt1hfXtdHirhNZybhPrYeRPvs3` (generate your own key; revoke any previously published key).
- **Render free web service** — root `terra-core`, build `cargo build --release -p terra-api`, start `./target/release/terra-api`, health `/health`.
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
2. Sprint D (founder-gated: Q-001 devnet funding): real devnet deploy on
   `AK3LmJmQijTJbsqj1PGt1hfXtdHirhNZybhPrYeRPvs3`, hosted API + managed
   PostGIS (Supabase `qgtstweelyankeqftepx`) + private RPC (Helius devnet),
   env-driven `TERRA_RPC_URL`/`API_BASE`, Pages on hosted API, neutral globe
   focus. Render settings: root `terra-core`, build `cargo build --release -p
   terra-api`, start `./target/release/terra-api`, health `/health`.
3. Sprint E remainder (founder-gated partially): protocol-side candidate discovery (not requester-supplied), premium-category guarantee (≥1 requested-category validator + random reputable fill), worker re-registration regression test.
4. Continue Sprint F remainder if Q-001 unblocks: ≥2 replicas for “verified” evidence status, scheduled PoP challenges.
5. Docs/book-of-intents fully synced to code reality (edition 2024, test baselines, MVP demo).
