# Terra — A Decentralized Land Claim & Verification Network

[![CI](https://github.com/MrObadiahEJ/terra/actions/workflows/ci.yml/badge.svg)](https://github.com/MrObadiahEJ/terra/actions/workflows/ci.yml)

> A decentralized, blockchain-anchored protocol where anyone can create claims about land, validators independently verify them, and the network records attestations and immutable history. No authority provider is required to participate.

**Program ID:** `GaEDbktvpZ3qiqp4PmFgHwDSa6JsFfVjXFqNb2nTbage` (devnet / localnet)

---

## What Terra Is

Terra is a **claim/verification/evidence network** for land and property:

- Anyone can create a claim about any parcel. Validators independently evaluate evidence. The protocol records attestations and builds immutable history. Claims ≠ Facts — the protocol verifies claims, not legal ownership.

**Core data model:**

```
People → Claims → Evidence → Validators → Attestations → Consensus → Immutable History
```

Physical world observation is a first-class layer, not an afterthought. Validators can observe, photograph, and attest to the physical state of land, binding digital records to physical reality.

---

## Status

All protocol modules (RFC-003 → RFC-012) have on-chain implementations:
Anchor programs + PostGIS mirror + REST API + frontend client + IDL, with CI
green on `dev` (fmt, `clippy -D warnings`, registry lib unit tests, API unit
tests incl. live-PostGIS migration run, `tsc --noEmit`).

**Phase 0 (RFC-012 architecture contract)**, **Phase 1 (security hardening)**, **Phase 2 (generalized validator PDAs)**, **Phase 3 (verification tasks)**, **Phase 4 (multi-source observations)**,
 **Phase 5 (evidence provenance)**, **Phase 6 (dynamic routing)**, **Phase 7 (reputation governance)**,
 **A1 (security residuals)**, **A2 (IDL regeneration)**, and **A3 (test/CI
 baseline)** are complete: unique validator sets, endorsement action binding,
 account-ownership checks on `remaining_accounts` loaders, admin/authority
 constraints, M-2/C-4/L-1 session-audit guards, checked-in IDL synced to source
 (147/59/133/206), and CI covers registry/identity lib + rfc012 + geo + API.
 See [`terra-core/SECURITY.md`](terra-core/SECURITY.md).

Verified tests on `dev` (2026-09-24):

| Suite | Count |
|-------|------:|
| `terra-registry` lib unit tests | 105 |
| `terra-registry` RFC-012 structural tests | 21 |
| `terra-identity` lib unit tests | 6 |
| `terra-identity` integration (BPF) | 18 |
| `terra-api` unit tests | 73 |
| `terra-geo` unit tests | 4 |

CI runs: fmt, clippy `-D warnings`, registry/identity lib, rfc012, geo, API
(+PostGIS migrations), and `tsc --noEmit`. Identity BPF (18) and the long-form
registry BPF suite (289) are run locally / on demand.

Registry BPF integration suite (`tests/integration.rs`, 289 tests) is maintained
but not re-run on constrained CI/dev machines (full `cargo test -p terra-registry`
exceeds practical time budgets); it is the long-form regression suite for all
instruction happy paths and guard rails.

Known limits before `main`: no devnet deployment yet (see
[Devnet checklist](#devnet-checklist)); ZK circuits are structural
(proof bytes opaque, no on-chain Groth16 verification — needs audit);
time-locked paths (7-day unbonding withdraw) are guard-verified, not
time-executed, in the harness; checked-in IDL
(`terra-web/src/idl/terra_registry.json`) was regenerated in A2 (2026-09-24)
to match source (135 / 53 / 121 / 184) — re-run `make idl` / `./build.sh`
before shipping client changes after any program edit.

### How to continue (handoff)

This repo is self-describing for a new contributor or agent — read in order:

1. **This file** — Status, Protocol Catalog, Verification table, Devnet checklist, Roadmap.
2. [`terra-core/SECURITY.md`](terra-core/SECURITY.md) — Phase 1 + A1 closed Critical/High/Medium and L-1; A2 refreshed IDL (119/44/108/160); Phase 2 re-synced (128/49/115/169); Phase 3 re-synced (134/52/120/182); Phase 4 re-synced (135/53/121/184); Phase 5 re-synced (137/55/123/187); Phase 6 re-synced (138/55/124/191); Phase 7 re-synced (147/59/133/206); A3 fixed API test baseline + CI coverage. Open mainnet items: RFC-005 reconfirm, ZK audit (L-3 cosmetic).
 3. [`terra-core/README.md`](terra-core/README.md) — Current Status + numbered next steps; workspace build/test commands.
 4. [`terra-core/docs/architecture.md`](terra-core/docs/architecture.md) — module map, PDAs, constants, source counts.
 5. [`docs/rfc-012-global-physical-digital-trust-architecture.md`](docs/rfc-012-global-physical-digital-trust-architecture.md) — Phases 0–7 done; **next phase is Phase 8** (economic/resource layer); start at §8.1 Handoff, then §9 Migration Map and §10 PDA sketch.
6. Individual specs `docs/rfc-003`…`rfc-011` each end with a **Handoff — status & next steps** footer (implementation location, open items, when to run which tests / `make idl`).

Do not invent instruction/account/event/error counts or test totals — regenerate
from source (`rg`, `cargo test`, `make idl`) or trust the tables above (verified
2026-09-24). Prefer small incremental commits on `dev`; keep tests green before
merging to `main`.

## Branching Strategy

| Branch | Purpose | Stability |
|--------|---------|-----------|
| **`main`** | Production-ready code. Only fully reviewed, tested, and approved protocols ship here. | Stable |
| **`dev`** | Active development. New protocols, features, and experiments land here first. Merged to `main` after review. | Unstable |

---

## Protocol Catalog

One-line principle: **the blockchain records who attested to what. It never
records how to do it, and it never touches key material.**

| RFC | Protocol | On-chain module(s) | Instructions |
|-----|----------|--------------------|--------------|
| RFC-003 | Vault shard protocol (Shamir-shared encrypted recovery vaults) | `vault.rs` | create/authorize/rotate/endorse/execute/cancel/ping |
| RFC-004 | Escrow settlement (native-SOL vault, seller/buyer guards) | `escrow.rs` | create/deposit/accept/settle/cancel/dispute/expire |
| RFC-005 | Validator staking & slashing (graduated 10%/100% slash, 7-day unbonding + appeal) | `staking.rs` | pool/deposit/unbond/withdraw/report/slash/claim/distribute/dispute/dismiss |
| RFC-006 | Cross-border identity bridge (jurisdictions, ZK bindings, nullifiers) | `cross_border.rs` | register/update/bind/verify/revoke/rebind |
| RFC-007 | Dispute resolution & parcel freeze | `dispute.rs` | file/freeze/adjudicate/execute/cancel |
| RFC-008 | Parcel subdivision & amalgamation (lineage records) | `subdivision.rs` | subdivide/amalgamate/migrate-rights/migrate-attestations |
| RFC-009 | Time-bound credentials (expiry, grace, renewal, sweep) | `time_bound.rs` | renew/sweep/conditional-grant |
| RFC-010 | Guardian & Recovery Council (policy layer on Succession: ≥3 validators, ≥90-day grace, court `case_hash`, revocation) | `terra_identity` `guardianship.rs` + registry `guardian_claim` | request-court-guardianship/revoke-guardianship |
| RFC-011 | Zero-knowledge ownership proofs (zone Merkle roots, nullifier first-use) | `zk.rs` | register-zone/generate-root/verify-proof/invalidate |
| RFC-012 | Global physical-digital trust architecture (Phases 0–10) | design contract + Phase 1 hardening across modules | see `docs/rfc-012-…` |

Supporting modules: `validator_registry.rs` (validator registry, bootstrap →
peer-consensus), `quorum.rs` (unique-validator + signer dedup), `ipfs_docs.rs`
(document anchors), `verification/*` (claims, sessions, challenges, reputation).
Full specs live in [`docs/`](docs/) as `rfc-003…rfc-012`.

Source counts (regenerate IDL after changes): **147 instructions · 59 account
types · 133 events · 206 `TerraError` codes** in `terra_registry`; **8
instructions · 2 accounts · 9 events · 29 `IdentityError` codes** in
`terra_identity`. Checked-in
[`terra-web/src/idl/terra_registry.json`](terra-web/src/idl/terra_registry.json)
matches source as of Phase 7 (2026-09-24); re-run `make idl` after program edits.

---

## Architecture Overview

Three layers, organized around **ISO 19152 (LADM)** concepts:

- **On-chain (Solana/Anchor)** — `terra-core/programs/terra_registry`: parcel
  identity, claims, evidence, validator attestations, and **hashes** of
  off-chain validation. Minimal state, quorum primitives reused everywhere,
  region-scoped trust. No authority provider prerequisite.
- **Off-chain mirror (PostGIS + Axum)** — `terra-core/api`: every on-chain
  account has a mirror table (migrations `0001…0024`), plus the spatial engine:
  maintained parcel centroids, geometry write-guards, `parcel_spatial_stats`
  and `zone_parcel_counts` views, `/spatial/*` radius/zone endpoints.
- **Geo engine** — `terra-core/geo-engine` (`terra-geo`): pure-Rust OSM road
  graph + Dijkstra/BFS reachability + SHA-256 canonical digests anchored
  on-chain as `Parcel::access_hash`.
- **Frontend** — `terra-web`: React 19 + Vite + CesiumJS globe + Leaflet map, pnpm,
  typed API client (`src/lib/api.ts`) covering backend routes.

```
browser ──▶ terra-web ──▶ terra-core/api ───┬─▶ PostGIS (mirror + spatial)
                                            └──▶ Solana (source of truth)
```

### Canonical Mental Model

```
                    ┌──────────────────────────────────────┐
                    │           Physical World             │
                    │  (land, boundaries, structures,      │
                    │   observations, photographs)         │
                    └──────────────┬───────────────────────┘
                                   │
                    ┌──────────────▼───────────────────────┐
                    │         Claims & Evidence            │
                    │  (anyone can create, propose,        │
                    │   dispute, or corroborate)           │
                    └──────────────┬───────────────────────┘
                                   │
                    ┌──────────────▼───────────────────────┐
                    │        Validator Network             │
                    │  (progressive levels 0-3,            │
                    │   evidence-based reputation)         │
                    └──────────────┬───────────────────────┘
                                   │
                    ┌──────────────▼───────────────────────┐
                    │       Attestations & Consensus       │
                    │  (quorum rules, stake-weighted,      │
                    │   evidence-anchored)                 │
                    └──────────────┬───────────────────────┘
                                   │
                    ┌──────────────▼───────────────────────┐
                    │     Immutable History (Solana)       │
                    │  (append-only, tamper-evident,       │
                    │   zone-scoped, timestamped)          │
                    └──────────────────────────────────────┘
```

### Validator Capability Levels

| Level | Capability | Description |
|-------|------------|-------------|
| 0 | Wallet only | Can create claims and manage their own parcels |
| 1 | Basic validator | Can attest to claims, sign transactions |
| 2 | Specialized validator | Can attest to specific claim types (survey, legal, physical) |
| 3 | High-trust validator | Can co-sign disputes, manage vault shards, adjudicate |

### Key Design Principles

1. **Anyone can claim.** No authority provider is a prerequisite for parcels to exist.
2. **Claims ≠ Facts.** The protocol verifies claims, not legal ownership.
3. **Evidence is first-class.** Physical observation is a core layer, not an afterthought.
4. **Validator reputation is evidence-based.** Driven by evidence history, not scores.
5. **Free basic participation.** Creating claims and basic validation is free; storage and expensive operations are paid.
6. **Self-correcting trust.** Random audits + triggered audits for continuous verification.
7. **Progressive decentralization.** Start with fewer validators, grow to more. Never require an authority provider.

---

## Monorepo Structure

```
terra/
├── terra-web/                        # React 19 + Vite + Cesium/Leaflet frontend (pnpm)
│   ├── src/idl/                      # terra_registry.json + terraRegistry.ts
│   └── src/lib/api.ts                # typed client for API routes
├── terra-core/                       # Rust workspace (terra-registry, terra-identity, terra-api, terra-geo)
│   ├── programs/terra_registry/src/  # lib.rs + protocol modules + verification/
│   ├── programs/terra_identity/src/  # identity, succession, guardianship
│   ├── api/src/routes/               # 23 route modules (parcels, staking, zk_proofs, spatial, …)
│   ├── api/migrations/               # 0001…0024 (PostGIS schema + mirrors)
│   └── geo-engine/                   # OSM graph + reachability (terra-geo)
├── docs/                             # rfc-003 … rfc-012
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
cargo test -p terra-registry --lib       # 90 unit tests
cargo test -p terra-identity --lib       # 6 unit tests
cargo test -p terra-api                  # 73 API unit tests
DATABASE_URL=postgres://terra@127.0.0.1:5433/terra_dev PORT=18080 \
  cargo run -p terra-api                 # serves /api/v1 (migrations auto-applied)
```

### 3. On-chain program (BPF) + integration tests

Build the BPF program (manifest parsing issue in Anchor requires direct
`cargo build-sbf`):

```bash
cd terra-core
cargo build-sbf --manifest-path programs/terra_registry/Cargo.toml
cargo build-sbf --manifest-path programs/terra_identity/Cargo.toml
# Long-form registry BPF suite (283 tests; heavy — needs built SBF + time):
cargo test -p terra-registry --test integration -- --test-threads=1
# Identity BPF suite (18 tests):
cargo test -p terra-identity --test integration
```

`SBF_OUT_DIR` is pre-configured in `.cargo/config.toml` — no environment
setup needed.

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
| Unit (on-chain guards/constants) | `cargo test -p terra-registry --lib` | 74/74 |
| Unit (identity helpers) | `cargo test -p terra-identity --lib` | 6/6 |
| RFC-012 structural | `rustc --test programs/terra_registry/tests/rfc012_structure.rs` | 21/21 |
| Identity BPF | `cargo test -p terra-identity --test integration` | 18/18 |
| Registry BPF (long-form) | `cargo test -p terra-registry --test integration` | 283 (maintained; run when machine/time allow) |
| Unit (API validation logic) | `cargo test -p terra-api` | 73/73 |
| Geo engine pure logic | `cargo test -p terra-geo` | 4/4 |
| Migrations on real PostGIS 16 | CI service + local scratch instance | 24/24 apply |
| Frontend types | `tsc --noEmit` | clean |
| Lints | `cargo fmt --check`, `cargo clippy -- -D warnings` | clean |

### Devnet checklist

- [x] Permanent `SBF_OUT_DIR` config for integration tests (`.cargo/config.toml`)
- [x] Build pipeline pinned to `cargo build-sbf` (Anchor manifest issue documented)
- [x] Phase 0 — RFC-012 architecture contract + structural tests
- [x] Phase 1 — security hardening (unique validator sets, endorsement action binding, ownership checks, admin constraints)
- [ ] `solana-test-validator` run with program deployed (requires AVX-capable CPU — not available on current dev machine)
- [ ] Withdraw-after-7d-unbonding executed against real clock time
- [ ] Frontend wallet signing wired to deployed program ID
- [x] Regenerate checked-in IDL from current source (`make idl`) — A2 (2026-09-24): 119/44/108/160; Phase 2 (2026-09-24): 128/49/115/169; Phase 3 (2026-09-24): 134/52/120/182; Phase 4 (2026-09-24): 135/53/121/184; Phase 5 (2026-09-24): 137/55/123/187; Phase 6 (2026-09-24): 138/55/124/191; Phase 7 (2026-09-24): 147/59/133/206 synced to `terra-web/src/idl/`
- [ ] ZK circuit choice (Groth16/PLONK) + external audit (RFC-006/011)
- [ ] Governance decision on RFC-005 staking (RFC says do-not-implement without one; code path exists — reconfirm before mainnet)

---

## Roadmap

- [x] Architecture research — LADM, comparable repos, data-source legal review
- [x] Country-agnostic core model (Parcel / Rights / Owner / infra flags)
- [x] Phase 1 — parcel registry, transfers, rights, Cesium globe
- [x] Phase 2 — PostGIS fusion, OSM ingestion, geo-engine
- [x] Phase 3 — road-access validation + on-chain digest anchor
- [x] Phase 4 — attestations, Identity, Succession, rotation, forfeiture
- [x] RFC-003…RFC-011 protocol suite (vault, escrow, staking, cross-border, disputes, subdivision, time-bound)
- [x] RFC-010 guardianship + RFC-011 ZK proofs + PostGIS spatial architecture
- [x] Full verification (BPF execution, live-DB smoke, contract cross-checks)
- [x] RFC-012 Phase 0 — architecture contract + structural tests
- [x] RFC-012 Phase 1 — security hardening (unique validator sets, endorsement binding, ownership checks, admin constraints)
- [x] Tier A1–A3 — security residuals, IDL regen, test/CI baseline (2026-09-24)
- [ ] Devnet deployment (see checklist above)
- [ ] Phase 5 — legal 3D/air-rights layer
- [ ] Phase 6 — country config layer (tenure types, multi-authority)
- [ ] Phase 7/8 — regional expansion → global platform
- [x] RFC-012 Phase 2 — generalized validator PDAs (2026-09-24): profile/presence/availability/capability/relationship-edge, IDL 128/49/115/169
- [x] RFC-012 Phase 3 — verification tasks (2026-09-24): VerificationTask/TaskRequirement/TaskAssignment PDAs + 6 instructions, IDL 134/52/120/182
- [x] RFC-012 Phase 4 — multi-source observations (2026-09-24): ObservationV2 PDA + submit_observation_v2, IDL 135/53/121/184
- [x] RFC-012 Phase 5 — evidence provenance (2026-09-24): EvidenceManifest/EvidenceArtifact PDAs + submit_evidence_manifest/add_evidence_artifact, IDL 137/55/123/187
- [x] RFC-012 Phase 6 — dynamic routing (2026-09-24): routing.rs multi-factor eligibility + route_task + TaskRouted, errors 6187–6190, IDL 138/55/124/191
- [x] RFC-012 Phase 7 — reputation governance (2026-09-24): fraud_governance.rs fraud report → random committee → capability demotion (no jail) → appeal → rehab; errors 6191–6205, IDL 147/59/133/206
- [ ] RFC-012 Phases 8–10 — economics, infrastructure, cross-border privacy

---

## Contributing

Open-source and community-governed as it grows. Contribution guidelines and
governance will be published past the devnet pilot.

## License

*(To be finalized — Apache-2.0 or MIT recommended, given the multi-country ambition.)*

## Disclaimer

Terra is technical infrastructure for creating and verifying claims about land.
It is not a legal registry and does not itself confer legal title. Ownership
remains governed by applicable national law; Terra makes locally-recognized
rights more verifiable, portable, and fraud-resistant.
