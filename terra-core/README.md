# Terra

A decentralized land registry and governance platform built on Solana using Anchor.

## Overview

Terra is a multi-program Solana system for managing land parcels, rights, escrow, staking, verification, and cross-border governance. It consists of two on-chain programs and supporting infrastructure.

### Programs

| Program | ID | Purpose |
|---------|-----|---------|
| `terra_registry` | `GaEDbktvpZ3qiqp4PmFgHwDSa6JsFfVjXFqNb2nTbage` | Core land registry, escrow, staking, verification |
| `terra_identity` | `68urV9nGcRcoWT1QjzZfXuCnTS9921x2se1SybKJr1U4` | Identity management, succession, guardianship |

### Architecture

```
┌─────────────────────────────────────────────────────────┐
│                     terra_registry                       │
├─────────────┬──────────────┬──────────────┬─────────────┤
│   Parcel    │   Escrow     │   Staking    │ Verification│
│  Registry   │   System     │   System     │   System    │
├─────────────┼──────────────┼──────────────┼─────────────┤
│  Vault &    │   Dispute    │  Cross-Border│  World      │
│  Shard Mgmt │   System     │  Bridge      │  Registry   │
├─────────────┼──────────────┼──────────────┼─────────────┤
│  ZK         │  Time-Bound  │  Quorum      │  Audit      │
│  Ownership  │  Rights      │  Voting      │  Trail      │
└──────┬──────┴──────────────┴──────────────┴──────┬──────┘
       │                                           │
       └──────────── CPI ──────────────────────────┘
                          │
              ┌───────────▼───────────┐
              │     terra_identity    │
              │  (Identity, Rights,   │
              │   Succession,         │
              │   Guardianship)       │
              └───────────────────────┘
```

See [docs/architecture.md](docs/architecture.md) for detailed module descriptions.

## Prerequisites

- [Rust](https://rustup.rs/) (stable)
- [Solana CLI](https://docs.solanalabs.com/cli/install) v3.0.0+
- [Anchor CLI](https://www.anchor-lang.com/docs/installation) v0.31.0+

## Quick Start

```bash
# Clone
git clone https://github.com/MrObadiahEJ/terra.git
cd terra/terra-core

# Build
make build

# Test
make test

# Lint
make lint
```

## Development

### Build

```bash
# Build all programs + IDL
./build.sh
# or
make build

# Programs only (no IDL)
cargo build-sbf --manifest-path programs/terra_registry/Cargo.toml
cargo build-sbf --manifest-path programs/terra_identity/Cargo.toml

# IDL only
make idl
```

### Test

```bash
# Registry unit tests (guards, quorum, staking, …)
cargo test -p terra-registry --lib

# Identity unit tests
cargo test -p terra-identity --lib

# RFC-012 structural tests
cargo test -p terra-registry --test rfc012_structure

# Identity BPF integration tests (requires SBF build)
cargo test -p terra-identity --test integration

# Registry BPF integration tests (long-running)
cargo test -p terra-registry --test integration -- --test-threads=1

# API unit tests (needs DATABASE_URL for PostGIS-backed cases)
cargo test -p terra-api

# Geo engine
cargo test -p terra-geo

# Via Make (integration only)
make test
```

### Lint

```bash
# Check formatting and clippy
make lint

# Auto-fix
make fix
```

## Deployment

See [deploy.sh](deploy.sh) for deployment scripts.

```bash
# Deploy to devnet
./deploy.sh devnet

# Deploy to localnet (requires solana-test-validator)
./deploy.sh localnet

# Deploy to mainnet (requires confirmation)
CONFIRM_MAINNET=yes ./deploy.sh mainnet
```

## Project Structure

```
terra-core/
├── Anchor.toml                 # Anchor configuration (program IDs, cluster)
├── Cargo.toml                  # Workspace root
├── Makefile                    # Build/test/deploy shortcuts
├── build.sh                    # Build script (cargo build-sbf + anchor build)
├── deploy.sh                   # Deployment script
├── SECURITY.md                 # Security audit status (Phase 1 closed)
├── programs/
│   ├── terra_registry/         # Core registry program
│   │   ├── src/
│   │   │   ├── lib.rs          # Entry point, contexts, error codes (160)
│   │   │   ├── cross_border.rs # Cross-border identity bridge
│   │   │   ├── dispute.rs      # Parcel dispute system
│   │   │   ├── escrow.rs       # Parcel escrow (buy/sell)
│   │   │   ├── staking.rs      # Validator staking
│   │   │   ├── vault.rs        # Encrypted vault & shard rotation
│   │   │   ├── zk.rs           # ZK ownership proofs
│   │   │   ├── quorum.rs       # Quorum + unique-validator utilities
│   │   │   ├── validator_registry.rs # Peer-consensus endorsements
│   │   │   ├── world_registry.rs # Country allocation & genesis
│   │   │   └── verification/   # Claims, sessions, challenges, …
│   │   └── tests/
│   │       ├── integration.rs  # 269 BPF integration tests
│   │       └── rfc012_structure.rs # 21 structural tests
│   └── terra_identity/         # Identity program
│       ├── src/
│       │   ├── helpers.rs      # count_unique_validators + tests
│       │   └── instructions/   # bind, succession, guardianship
│       └── tests/integration.rs # 18 BPF tests
├── api/                        # REST API backend (Axum + Postgres)
│   ├── src/routes/             # 23 route modules
│   └── migrations/             # 0001…0024 (PostGIS + mirrors)
├── geo-engine/                 # OSM data processing engine (terra-geo)
├── docs/                       # Documentation
│   └── architecture.md         # Detailed architecture
└── .github/workflows/ci.yml   # CI/CD pipeline
```

## Modules

### Parcel Registry
Register, transfer, and manage land parcels with infrastructure flags and access control.

### Escrow System
Buy/sell parcels with SOL escrow, dispute resolution, and settlement flows.

### Staking
Validator staking with deposit, unbonding, reward distribution, and slashing.

### Verification
Claims, observations, attestations, quorum voting, challenges, and guardian claims.

### Vault & Shard Management
Encrypted vaults with threshold shard holders, rotation, and ping mechanisms.

### ZK Ownership
Zero-knowledge proofs for parcel ownership without revealing identity.

### Cross-Border Bridge
Jurisdiction binding, verification, and revocation for cross-border identity.

### World Registry
Country allocation, genesis requests, and confirmer diversity checks.

### Dispute System
File disputes, freeze parcels, adjudicate, and execute judgments.

### Quorum Voting
Configurable quorum-based voting with confirm/dispute choices.

### Time-Bound Rights
Grant, renew, revoke, and sweep time-limited parcel rights.

### Audit Trail
Immutable audit entries for tracking system events.

## Test Coverage

| Suite | Count | Notes |
|-------|------:|-------|
| terra-registry lib | 62 | Guards, quorum, staking, subdivision, zk, … |
| terra-identity lib | 6 | Unique-validator helpers |
| rfc012_structure | 21 | RFC document structural checks |
| terra-identity integration | 18 | BPF happy paths + guard rails |
| terra-registry integration | 269 | Full instruction matrix (long-running) |
| terra-api | 73 | Route validation + storage helpers |
| terra-geo | 4 | Graph reachability |

Verified on `dev` (2026-09-24): registry lib 68/68, identity lib 6/6, rfc012 21/21, identity BPF 18/18, registry BPF phase2 tests 4/4, API 73/73, geo 4/4, `cargo fmt` + `clippy -D warnings` clean, both programs `cargo build-sbf` OK, checked-in IDL matches source (A2). Full registry BPF suite (275) is maintained but not re-run on constrained machines. Fast baseline: `make test-fast`.

## Current Status (as of 2026-09-24)

**Done:**
- All RFC-003…011 protocol modules implemented on-chain (see [architecture.md](docs/architecture.md)).
- RFC-012 **Phase 0** (architecture contract + `rfc012_structure` tests) and **Phase 1** (security hardening: unique validator sets, endorsement action binding, `remaining_accounts` ownership checks, admin constraints) — details in [SECURITY.md](SECURITY.md).
- **A1 security residuals** (2026-09-24): M-2 foreign-entity audit guard, C-4 session-record signers + registry, L-1 `dec_rights_count`; unit + BPF tests green.
- **A2 IDL regeneration** (2026-09-24): `ValidatorSlashed` collision resolved → `ReputationSlashed` (staking keeps `ValidatorSlashed` per RFC-005); checked-in `terra-web` IDL synced to **119/44/108/160**; **Phase 2** re-synced to **128/49/115/169**; `make idl`/`build.sh` now use `anchor idl build -p …`.
- **A3 test/CI baseline** (2026-09-24): fixed `StoredArtifact` API test compile break; CI runs identity lib + rfc012 + geo; `make test-fast` for constrained machines.
- PostGIS mirror API (23 routes, migrations `0001`…`0024`) + geo-engine + workspace CI green on `dev`.

**Open / next (in priority order):**
1. Close remaining SECURITY.md items before mainnet: RFC-005 staking governance reconfirm, ZK circuit choice (RFC-006/011). L-3 is cosmetic only. IDL regen is done (A2, 119/44/108/160; Phase 2, 128/49/115/169).
2. Regenerate checked-in IDL: `make idl` (or `./build.sh`) after any program change; `terra-web/src/idl/terra_registry.json` matches source as of Phase 2 (128/49/115/169, 2026-09-24).
3. Devnet deploy: `./deploy.sh devnet` (needs AVX-capable machine for `solana-test-validator`).
4. ZK circuit selection + external audit (RFC-006/011) — proof bytes still opaque, no on-chain Groth16.
5. Governance reconfirm on RFC-005 staking before mainnet (code exists; RFC originally cautioned against implementing without a decision).
6. RFC-012 Phases 2–10 (generalized validator → tasks → observations → routing → reputation governance → economics → infrastructure → cross-border/privacy) — see [RFC-012 §8](../../docs/rfc-012-global-physical-digital-trust-architecture.md).

Anyone picking this up: start from root [README.md](../README.md) Status + Devnet checklist, then [SECURITY.md](SECURITY.md) Recommendations, then RFC-012 phase table. Do not invent counts — regenerate with `rg`/tests or `make idl`.

## Error Codes

- **terra_registry:** 169 custom error codes (`TerraError` in `lib.rs`).
- **terra_identity:** 29 custom error codes (`IdentityError` in `errors.rs`, 6000+).

## License

This project is proprietary. All rights reserved.
