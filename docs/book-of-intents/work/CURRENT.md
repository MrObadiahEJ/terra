# Current Work Ledger

Authoritative record of active execution. Phase derives from evidence here.
Last reconciled: 2026-11-27. Previous ledger archived at
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
| **C** | Chain indexer: `api/src/indexer.rs`, RPC in `AppState`, `chain_verified` mirror upserts, DB-vs-chain audit | **delivered; Q-033/Q-035 follow-ups complete locally** | `api/src/indexer.rs`; migrations `0051`/`0057`–`0061` applied; `GET /mirror/audit` covers typed mirrors + `chain_accounts` + `parcel_geometry_artifacts`; commits `ac754f6`/`62875fd`; ADR-026–029; notes `sprints/058-sprint-c-chain-indexer.md`, `sprints/063-sprint-q033-q035.md` |
| **D** | Devnet hosting: real deploy, hosted API + managed PostGIS + private RPC, env-driven clients, Pages on hosted API, neutral globe focus | **in progress / partially delivered** | Render API `https://terra-gscx.onrender.com` healthy; Pages `https://mrobadiahej.github.io/terra/`; devnet programs deployed (registry `GaEDbk...bage`, identity `HyjpaH...Vo6p`); Supabase `qgtstweelyankeqftepx` migrated 0001–0056; browser-wallet E2E still pending; `sprints/059-sprint-d-devnet-hosting.md` |
| **E** | Routing/economics hardening: VRF-verified routing (server-side `GET /geo/route` with `from_node_id`/`to_node_id` topology), anti-spam staging bond, premium-category routing, reliability derived from challenge results | **partially delivered** | `geo-engine/src/reach.rs` `route_between` Dijkstra; `api/migrations/0049_provider_content_address.sql`; `storage_providers.stake_bond` + `wallet_address` (migration 0053); `GET /providers/replication-audit` enforces N-of-M ≥2 provider classes; commits `8cd7aa9`, `0f0a052`; note `sprints/060-sprint-e-routing-economics.md` |
| **F** | Evidence durability: IPFS backend option (`IPFS_API_URL`), quarantine + TTL columns on `evidence_uploads`, `StorageBackend` enum (`Local`/`Ipfs`), honest labeling until paid | **delivered** | `api/src/storage.rs` (IPFS backend, quarantine/TTL); migration `0053_evidence_durability_and_provider_economy.sql`; commit `0f0a052`; note `sprints/061-sprint-f-evidence-durability.md` |
| **G** | Ratified vision features: institutional credentials, residency attestation, lien/encumbrance, one real reconstruction job hashed on-chain | planned — **Q-003/Q-004 ratification; Q-005 hard gate** | `sprints/062-sprint-g-ratified-vision.md` |

## Delivered this cycle (Sprints B, C, E-partially, F)

- **Sprint B** — Real write path: deleted `tx_prep.rs` placeholder builders and route registration; every write now uses client-side IDL `program.methods` with real PDAs, discriminators and signers; fixed `identityPda`/`successionPda` to derive on `terra_identity` program ID; generated and checked in IDLs (`idl/terra_registry.json` 176 instructions, `idl/terra_identity.json` 8 instructions); wired `/console` and parcel panels to the real loop; commit `76d85df`.
- **Sprint C** — Chain indexer: `api/src/indexer.rs` polls `terra_registry` program accounts, decodes Anchor `Parcel` and `VerificationAttestation` accounts by discriminator, upserts mirror rows with `chain_verified=true`, `chain_slot`, `chain_signature`; `GET /mirror/audit` compares DB to chain across 7 tables including `verification_attestations_onchain`; migration `0051_chain_verified_columns.sql` adds `chain_verified`/`chain_slot`/`chain_signature` to all mirror tables; commit `ac754f6`.
- **Sprint C P0 hardening (2026-11-27, committed)** — source audit found three P0 defects in the read path: a missing `claims_onchain` table could abort the whole 15 s poll on the first Claim PDA; spatial/geometry upserts never wrote chain provenance; and `chain_signature`/`chain_slot` were decorative (empty string + poll slot). Fixed with forward migration `0057_claims_onchain_mirror.sql` (applied to Supabase; columns verified), per-account error isolation, and real `getSignaturesForAddress` provenance for all five decoded account types (ADR-026; R-039–R-041). The new DB-backed test also caught a missing INSERT expression and a base58/hex FK mismatch in the geometry upsert, both fixed. Gates: `cargo check -p terra-api` clean; `cargo test -p terra-api` **173/173** (7 new indexer tests; 3 pre-existing `runtime.rs` warnings).
- **Sprint 063 — Q-033(b) staged rollout + Q-035(a) content-addressed geometry (2026-11-27):** founder authorized both options and commit/merge. All IDL account types now have generated decoders and a provenance-tagged generic `chain_accounts` mirror (migration 0060, applied; 82 per-type acceptance tests at that stage, 83 after the artifact account; CI `--check`), while the eight typed projections stay unchanged. New `ParcelGeometryArtifact` PDA/instruction registers `sha256(raw GeoJSON)` + storage URI without changing the `Parcel` layout; signed upload route and chain-only materialization require tx provenance + content-hash + ring-hash agreement (migration 0061, applied). `/mirror/audit` covers both new tables. ADR-028/029; detail in `sprints/063-sprint-q033-q035.md`.
- **Sprint E (partial)** — Routing/economics hardening: server-side `GET /geo/route` with `route_between` Dijkstra using `from_node_id`/`to_node_id` node-id topology (migration 0049 adds `content_address` to `provider_replicas` fixing the `prove_replica` column bug); `prove_replica` **fails closed** (503) without `VRF_ORACLE_PUBKEY` — verifies Ed25519 oracle signature over `challenge + content_address`; legacy SHA-256 fallback removed; oracle independence enforced (`oracle ≠ winner/candidate/requester`); `storage_providers.stake_bond` + `wallet_address` (migration 0053) for anti-spam; `GET /providers/replication-audit` enforces N-of-M ≥2 provider classes; `reliability_score` derived from `last_proof_at` challenge results (not self-declared); commits `8cd7aa9`, `0f0a052`.
- **Sprint F** — Evidence durability: `StorageBackend` enum (`Local`/`Ipfs`); IPFS pinning via `IPFS_API_URL`/`IPFS_GATEWAY_URL` env vars; quarantine + TTL columns on `evidence_uploads` (`quarantined`, `quarantine_reason`, `ttl_seconds`, `expires_at`); honest labeling in `storage.rs` until paid IPFS/S3 backend exists; migration `0053_evidence_durability_and_provider_economy.sql`; commit `0f0a052`.
- **ZK (out-of-sprint)** — Full Groth16 prover→verifier E2E: `zk_membership_prover_verifier_e2e` registers a zone set, pins VK hash, generates an ownership root, arkworks produces a real Groth16 proof over `terra-zk-membership::MembershipCircuit`, and on-chain `verify_ownership_proof` accepts via `alt_bn128` pairing; tampered proofs rejected. `IMPLEMENTATION_STATUS.md` A14 → `1 – IMPLEMENTED AND VERIFIED`; BPF suite now **308/308**; `ARCHITECTURAL_DECISIONS.md` ADR-025.
- **Edition 2024 migration** (`1d495d5`): all 5 workspace crates migrated from Rust edition 2021 to 2024 — `terra-core/Cargo.toml` (workspace.package), `programs/terra_registry/Cargo.toml`, `programs/terra_identity/Cargo.toml`, `api/Cargo.toml`, `geo-engine/Cargo.toml`, `zk-membership/Cargo.toml`. `cargo check --workspace` clean; all tests green (registry lib 164/164, API 166/166, identity 23/23, geo 45/45+1, zk-membership 6/6).
- **MVP demo release** (`15af9fc`): terra-web v0.1.0 — `/demo` scenario player, `/mvp` Intent 5 flow, `/verify`, `/atlas`, `/lab` all functional; `tsc` clean, vitest 138/138, build:check PASS (entry 303 KB gz, Cesium 1098 KB gz, total 1908 KB gz). Built for GitHub Pages at `TERRA_BASE_PATH=/terra/`.
- **Branch sync + public-demo bundle hygiene** (2026-11-27): `dev` upstream corrected to `origin/dev` and `2cbfb55..dfc65de` pushed; `main` untouched. Frontend now strips loopback API bases and keyed RPC URLs from production (`lib/constants.ts` + 5 tests), `check-bundle.mjs` §5 fails leaking bundles, and the Pages workflow uses public devnet RPC with `pnpm build:check`. Gates: `tsc -b` clean, vitest 143/143, `build:check` PASS. Site: <https://mrobadiahej.github.io/terra/> (Pages run `45941285816` green); browser-checked `/welcome`, `/atlas`, `/demo`, `/mvp`, `/verify`, `/stories` — all render, demo runs 12 events via API engine; live `/geo/buildings` 400 and `/manifests` 500 confirm R-035 while the frontend degrades cleanly. Old bundles contain a Helius key — founder must rotate it (R-034). ESLint: 27 pre-existing errors, not a CI gate.

## Deep-audit report

- **`DEEP_AUDIT_REPORT.md`** (updated): security architecture, dependency analysis, BPF adversarial test results (308/308 passing, 0 failed — CI run `41474466105` green), authority/pause key map, known gaps, recommendations.

## Known blockers

- **Q-001 devnet funding / Sprint D hosting** — deployer `AK3LmJmQijTJbsqj1PGt1hfXtdHirhNZybhPrYeRPvs3`
  is funded (~0.09 SOL on devnet at last check) and programs are deployed
  (`terra_registry` `GaEDbktvpZ3qiqp4PmFgHwDSa6JsFfVjXFqNb2nTbage`,
  `terra_identity` `HyjpaHY9yQg1KneLZGjpLDQfP9WV4LUEbNZcb7C4Vo6p`). Hosted API
  is live on Render (`https://terra-gscx.onrender.com`, `/health` OK, DB up),
  frontend on GitHub Pages (`https://mrobadiahej.github.io/terra/`), Supabase
  project `qgtstweelyankeqftepx` migrated through 0061 (`claims_onchain`, `chain_accounts` and
   `parcel_geometry_artifacts` present). Remaining Sprint D
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
- **Hosted production provisioning (Q-032 / R-035–R-038)** — live `/health`
  reports `osm_loaded:false` (`OSM_PBF_PATH` unset → road access and
  `GET /geo/route` disabled), no durable object-store creds on ephemeral
  Render disk (bytes can vanish on redeploy), `VRF_ORACLE_PUBKEY` unset
  (fail-closed: no challenges, `prove_replica` 503), `TERRA_ZK_MODE=disabled`,
  and free-tier sleep stops the API-owned TTL/PoP loop and indexer. No uptime
  monitor or epoch-close scheduler confirmed; single ~0.09 SOL deployer key is
  also the deploy authority. Founder-gated — do **not** weaken fail-closed
  behaviour for the demo. The ZK ceremony/external audit, bounded pause +
  multisig, reviewer randomization, RFC-005 governance decision, external
  audit/fuzzing and time-warp CI remain open (R-006/R-022–R-024/R-029/R-038).
- **Indexer RPC load (R-041/R-045)** — 2026-11-27 update: the hard-coded
   15 s full-account poll was replaced by a 60 s default, env-driven cadence
   (`TERRA_INDEXER_*`), an 8-byte header-only discriminator scan, batched
   `getMultipleAccounts` and exponential backoff. ADR-026 still adds one
   `getSignaturesForAddress` call per mirrored account per poll; hosted
   rate-limit impact is unverified. Failed lookups downgrade rows honestly
   (`FALSE`/`NULL`/`NULL`), so throttling cannot fabricate chain verification.
- **Indexer coverage + chain-only parcels (REQ-060/Q-033/Q-035)** — Q-033(b)
   and Q-035(a) resolved (ADR-028/029): every IDL account type is decoded and
   generically mirrored with provenance; typed tables/APIs still cover only the
   original eight (stage 2 not funded). Chain-only parcels are now materialized
   only from a chain-verified content-addressed artifact whose bytes and ring
   digest both match the chain; no polygon is fabricated. Q-034 pagination and
   Q-032 hosting remain founder/vendor-gated.
- **Rotating the exposed Helius RPC key (R-034)** — old published Pages bundles
  contain `api-key=…`; new builds no longer embed it, but the founder must
  revoke/rotate the old key in the Helius dashboard.
- **Frontend lint debt** — `pnpm exec eslint .` reports 27 pre-existing errors
  (10 warnings) in 8 files; ESLint is not currently a CI gate. Build, typecheck
  and 143 vitest cases are green.

## Next up

1. Confirm CI is green including the `localnet-e2e` job; if red, fix before any new work.
2. Sprint D (founder-gated: Q-001 devnet funding): real devnet deploy on
   `AK3LmJmQijTJbsqj1PGt1hfXtdHirhNZybhPrYeRPvs3`, hosted API + managed
   PostGIS (Supabase `qgtstweelyankeqftepx`) + private RPC for the **backend**
   (Helius devnet; never embed its key in the public Pages bundle),
   env-driven `TERRA_RPC_URL`/`API_BASE`, Pages on hosted API, neutral globe
   focus. Render settings: root `terra-core`, build `cargo build --release -p
   terra-api`, start `./target/release/terra-api`, health `/health`.
3. Sprint E remainder (founder-gated partially): **on-chain** protocol-side candidate discovery (not requester-supplied) and the on-chain requested-category validator guarantee. The off-chain runtime premium-category guarantee is already implemented and CI-tested (`runtime.rs:462-491`, `premium_category_guarantees_one_reputable_replica` runs with `DATABASE_URL`); do not re-implement it in the API. Worker re-regression test still pending.
4. Continue Sprint F remainder if Q-001 unblocks: ≥2 replicas for “verified” evidence status, scheduled PoP challenges.
5. Docs/book-of-intents fully synced to code reality (edition 2024, test baselines, MVP demo).
6. Confirm the Pages deployment triggered by the frontend commit serves the
   config-clean bundle at <https://mrobadiahej.github.io/terra/>, then rotate
   the exposed Helius key (R-034).
7. Investor demo walkthrough: `/welcome` → `/atlas` → `/demo` → `/mvp` →
   `/verify` → `/stories`, with demo labels visible and no live-chain claim.
8. Verify the P0 + 1d/1e indexer fixes on hosted devnet: create a real Claim
   PDA, confirm `claims_onchain` receives a non-empty `chain_signature` and
   matching `chain_slot`, check geometry/rights/3D/snapshot mirrors are
   `chain_verified`, register a parcel through the panel and confirm the
   geometry-hash gate, and watch RPC error logs for rate limiting at the new
   60 s cadence. Also exercise the Q-035 upload→register→materialize flow with
   a real tx. Founder authorization for commit + `dev`→`main` merge is granted;
   verify local gates and CI BPF/localnet results before publishing.
