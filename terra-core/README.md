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
# Build all programs
cargo build-sbf

# Build with Anchor (generates IDLs)
anchor build --skip-lint
```

### Test

```bash
# Run integration tests (single-threaded for consistency)
cargo test --test integration -- --test-threads=1

# Or via Make
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
├── Anchor.toml                 # Anchor configuration
├── Cargo.toml                  # Workspace root
├── Makefile                    # Build/test/deploy shortcuts
├── build.sh                    # Build script
├── deploy.sh                   # Deployment script
├── SECURITY.md                 # Security audit report
├── programs/
│   ├── terra_registry/         # Core registry program
│   │   ├── src/
│   │   │   ├── lib.rs          # Entry point, contexts, error codes
│   │   │   ├── cross_border.rs # Cross-border identity bridge
│   │   │   ├── dispute.rs      # Parcel dispute system
│   │   │   ├── escrow.rs       # Parcel escrow (buy/sell)
│   │   │   ├── staking.rs      # Validator staking
│   │   │   ├── vault.rs        # Encrypted vault & shard rotation
│   │   │   ├── zk.rs           # ZK ownership proofs
│   │   │   ├── quorum.rs       # Quorum utilities
│   │   │   ├── world_registry.rs # Country allocation & genesis
│   │   │   └── verification/   # Claims, sessions, challenges, etc.
│   │   └── tests/
│   │       └── integration.rs  # 256 integration tests
│   └── terra_identity/         # Identity program
│       └── src/
├── api/                        # REST API backend (Axum + Postgres)
├── geo-engine/                 # OSM data processing engine
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

256 integration tests covering all protocol invariants:

| Proposition | Tests | Coverage |
|-------------|-------|----------|
| **O1–O14** | 14 | Ownership invariants: legacy/identity auth, revocation, transfer, subdivision, amalgamation, dispute anti-grief |
| **B1–B6** | 6 | Authority paths: USAGE/SERVITUDE/EASEMENT/LIEN rejection, wrong parcel, closed IdentityRights |
| **C1–C8** | 8 | Edge cases: sweep expired rights, dispute filing, double dispute, past expiry, nonce reuse, permanent right |
| **D1–D6** | 6 | Cross-module: identity lifecycle, right lifecycle, dispute lifecycle, attestation+subdivide, staking |
| **E1–E8** | 8 | Adversarial: wrong PDA, forged signer, threshold bypass, zero ID/name/geometry, double register, self-transfer |
| **F1–F3** | 3 | E2E lifecycles: full parcel, identity migration, rights+time-bound |

## Error Codes

The program defines 155 custom error codes (6000-6154). See `lib.rs` for the complete `TerraError` enum.

## License

This project is proprietary. All rights reserved.
