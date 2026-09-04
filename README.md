# Terra — A Universal Land Registry for Humankind

[![CI](https://github.com/MrObadiahEJ/terra/actions/workflows/ci.yml/badge.svg)](https://github.com/MrObadiahEJ/terra/actions/workflows/ci.yml)

> A country-agnostic, blockchain-anchored land administration platform. Built first for Cameroon. Designed from day one to belong to the world.

**Program ID:** `GaEDbktvpZ3qiqp4PmFgHwDSa6JsFfVjXFqNb2nTbage` (devnet / localnet)

---

## Status

All protocol modules (RFC-003 → RFC-011) are implemented end-to-end: on-chain
Anchor program + PostGIS mirror + REST API + frontend client + IDL, with CI
green on `dev` (fmt, `clippy -D warnings`, 54 lib unit tests, 47 API unit
tests incl. live-PostGIS migration run, `tsc --noEmit`).

Beyond CI, every protocol is verified by **BPF integration tests** (real
program execution via `solana-program-test`, 9/9 passing) and a **76-check API
smoke suite** against live PostGIS covering happy paths and guard rails
(double-proof replay, early claims, stale roots, forged signatures, …).

Known limits before `main`: no devnet deployment yet (see
[Devnet checklist](#devnet-checklist)); ZK circuits are structural
(proof bytes opaque, no on-chain Groth16 verification — needs audit);
time-locked paths (7-day unbonding withdraw) are guard-verified, not
time-executed, in the harness.

## Branching Strategy

| Branch | Purpose | Stability |
|--------|---------|-----------|
| **`main`** | Production-ready code. Only fully reviewed, tested, and approved protocols ship here. | Stable |
| **`dev`** | Active development. New protocols, features, and experiments land here first. Merged to `main` after review. | Unstable |

---

## Protocol Catalog

One-line principle: **the blockchain records who authorized what. It never
records how to do it, and it never touches key material.**

| RFC | Protocol | On-chain module | Instructions |
|-----|----------|-----------------|--------------|
| RFC-003 | Vault shard protocol (Shamir-shared encrypted recovery vaults) | `vault.rs` | create/authorize/rotate/endorse/execute/cancel/ping |
| RFC-004 | Escrow settlement (native-SOL vault, seller/buyer guards) | `escrow.rs` | create/deposit/accept/settle/cancel/dispute/expire |
| RFC-005 | Validator staking & slashing (graduated 10%/100% slash, 7-day unbonding + appeal) | `staking.rs` | pool/deposit/unbond/withdraw/report/slash/claim/distribute/dispute/dismiss |
| RFC-006 | Cross-border identity bridge (jurisdictions, ZK bindings, nullifiers) | `cross_border.rs` | register/update/bind/verify/revoke/rebind |
| RFC-007 | Dispute resolution & parcel freeze | `dispute.rs` | file/freeze/adjudicate/execute/cancel |
| RFC-008 | Parcel subdivision & amalgamation (lineage records) | `subdivision.rs` | subdivide/amalgamate/migrate-rights/migrate-attestations |
| RFC-009 | Time-bound credentials (expiry, grace, renewal, sweep) | `time_bound.rs` | renew/sweep/conditional-grant |
| RFC-010 | Guardian & Recovery Council (policy layer on Succession: ≥3 validators, ≥90-day grace, court `case_hash`, revocation) | `guardian.rs` | request-court-guardianship/revoke-guardianship |
| RFC-011 | Zero-knowledge ownership proofs (zone Merkle roots, nullifier first-use) | `zk.rs` | register-zone/generate-root/verify-proof/invalidate |

Supporting modules: `authority_registry.rs` (validator registry, bootstrap →
peer-consensus), `ipfs_docs.rs` (document anchors). Full specs live in
[`docs/`](docs/) as `rfc-003…rfc-011`.

69 instructions · 22 accounts · 57 events · 113 errors — see
[`terra-web/src/idl/terra_registry.json`](terra-web/src/idl/terra_registry.json).

---

## Architecture Overview

Three layers, organized around **ISO 19152 (LADM)** concepts:

- **On-chain (Solana/Anchor)** — `terra-core/programs/terra_registry`: parcel
  identity, ownership, rights, and **hashes** of off-chain validation. Minimal
  state, quorum primitives reused everywhere, region-scoped trust.
- **Off-chain mirror (PostGIS + Axum)** — `terra-core/api`: every on-chain
  account has a mirror table (migrations `0001…0019`), plus the spatial engine:
  maintained parcel centroids, geometry write-guards, `parcel_spatial_stats`
  and `zone_parcel_counts` views, `/spatial/*` radius/zone endpoints.
- **Geo engine** — `terra-core/geo-engine` (`terra-geo`): pure-Rust OSM road
  graph + Dijkstra/BFS reachability + SHA-256 canonical digests anchored
  on-chain as `Parcel::access_hash`.
- **Frontend** — `terra-web`: React 19 + Vite + CesiumJS globe, pnpm, typed
  API client (`src/lib/api.ts`) covering every backend route.

```
browser ──▶ terra-web ──▶ terra-core/api ──┬──▶ PostGIS (mirror + spatial)
                                            └──▶ Solana (source of truth)
```

---

## Monorepo Structure

```
terra/
├── terra-web/                        # React 19 + Vite + CesiumJS frontend (pnpm)
│   ├── src/idl/                      # terra_registry.json + terraRegistry.ts
│   └── src/lib/api.ts                # typed client for all ~118 API routes
├── terra-core/                       # Rust workspace (terra-registry, terra-api, terra-geo)
│   ├── programs/terra_registry/src/  # lib.rs + 11 protocol modules + tests/
│   ├── api/src/routes/               # 18 route modules (parcels, staking, zk_proofs, spatial, …)
│   ├── api/migrations/               # 0001…0019 (PostGIS schema + mirrors)
│   └── geo-engine/                   # OSM graph + reachability (terra-geo)
├── docs/                             # rfc-003 … rfc-011
└── .github/workflows/ci.yml          # fmt, clippy, lib/api tests (PostGIS svc), tsc
```

---

## Running Locally

Prerequisites: Rust stable, Node 22 + pnpm 10, PostgreSQL 16 + PostGIS 3,
Solana CLI (for `solana-test-validator`), `cargo-build-sbf` + platform tools
for BPF builds.

### 1. Database

Any PostgreSQL 16 with PostGIS works — a scratch instance needs no root:

```bash
export PATH=/usr/lib/postgresql/16/bin:$PATH
initdb -D /tmp/pgdata -U terra --auth=trust
pg_ctl -D /tmp/pgdata -o "-p 5433" -l /tmp/pg.log start
createdb -h localhost -p 5433 -U terra terra_dev
```

(The API auto-applies migrations on boot via `sqlx::migrate!()`.)

### 2. Backend

```bash
cd terra-core
cargo check -p terra-registry            # on-chain program (native)
cargo test -p terra-registry --lib       # 54 unit tests
cargo test -p terra-api                  # 51 API unit tests
DATABASE_URL=postgres://terra@127.0.0.1:5433/terra_dev PORT=18080 \
  cargo run -p terra-api                 # serves /api/v1 (migrations auto-applied)
```

### 3. On-chain program (BPF) + integration tests

`anchor build` is not used (manifest parsing is broken in this environment);
build the BPF program directly:

```bash
cd terra-core/programs/terra_registry
cargo build-sbf                          # → ../../target/deploy/terra_registry.so
cd ../..
BPF_OUT_DIR=$PWD/target/deploy cargo test -p terra-registry --test integration
```

`BPF_OUT_DIR` is required so `solana-program-test` finds the `.so`. (A
`terra-core/.cargo/config.toml` setting this permanently is on the todo list.)

### 4. Frontend

```bash
cd terra-web
pnpm install --frozen-lockfile
pnpm exec tsc --noEmit
pnpm dev
```

---

## Verification

| Layer | How | Status |
|-------|-----|--------|
| Unit (on-chain guards/constants) | `cargo test -p terra-registry --lib` | 54/54 |
| Unit (API validation logic) | `cargo test -p terra-api` | 51/51 |
| BPF execution (9 scenarios incl. negative guards, replay, time-guard code) | `cargo test --test integration` + `BPF_OUT_DIR` | 9/9 |
| API end-to-end vs live PostGIS (76 checks, happy + guard paths) | smoke suite over `/api/v1` | 76/76 |
| Migrations on real PostGIS 16 | CI service + local scratch instance | 19/19 apply |
| Frontend↔API contract (112 calls vs 118 routes, method+path) | static cross-check | 100% match |
| IDL vs program (instructions/args/errors/accounts) | static cross-check | match |
| Frontend types | `tsc --noEmit` | clean |
| Lints | `cargo fmt --check`, `cargo clippy -- -D warnings` | clean |

### Devnet checklist

- [ ] `solana-test-validator` run with program deployed (loads the audited `.so`)
- [ ] Withdraw-after-7d-unbonding executed against real clock time
- [ ] `anchor build` manifest issue resolved or build pipeline pinned to `cargo build-sbf`
- [ ] Permanent `BPF_OUT_DIR` config for integration tests
- [ ] Frontend wallet signing wired to deployed program ID
- [ ] ZK circuit choice (Groth16/PLONK) + external audit (RFC-006/011)
- [ ] Governance decision on RFC-005 staking (RFC says do-not-implement without one)

---

## Roadmap

- [x] Architecture research — LADM, comparable repos, data-source legal review
- [x] Country-agnostic core model (Parcel / Rights / Owner / infra flags)
- [x] Phase 1 — parcel registry, transfers, rights, Cesium globe
- [x] Phase 2 — PostGIS fusion, OSM ingestion, geo-engine
- [x] Phase 3 — road-access validation + on-chain digest anchor
- [x] Phase 4 — attestations, Identity, Succession, rotation, forfeiture
- [x] RFC-003…RFC-009 protocol suite (vault, escrow, staking, cross-border, disputes, subdivision, time-bound)
- [x] RFC-010 guardianship + RFC-011 ZK proofs + PostGIS spatial architecture
- [x] Full verification (BPF execution, live-DB smoke, contract cross-checks)
- [ ] Devnet deployment (see checklist above)
- [ ] Phase 5 — legal 3D/air-rights layer
- [ ] Phase 6 — country config layer (tenure types, multi-authority)
- [ ] Phase 7/8 — regional expansion → global platform

---

## 🤝 Contributing

Open-source and community-governed as it grows. Contribution guidelines and
governance will be published past the devnet pilot.

## 📜 License

*(To be finalized — Apache-2.0 or MIT recommended, given the multi-country ambition.)*

## ⚠️ Disclaimer

Terra is technical infrastructure and does not itself confer legal title.
Ownership remains governed by applicable national law; Terra makes
locally-recognized rights more verifiable, portable, and fraud-resistant.
