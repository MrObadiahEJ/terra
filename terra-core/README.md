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
│   │   │   ├── lib.rs          # Entry point, contexts, error codes (184)
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
│   │       ├── integration.rs  # 292 BPF integration tests
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
| terra-registry lib | 116 | Guards, quorum, staking, subdivision, zk, tasks, observations, fraud governance, task economics, … |
| terra-identity lib | 6 | Unique-validator helpers |
| rfc012_structure | 21 | RFC document structural checks |
| terra-identity integration | 18 | BPF happy paths + guard rails |
| terra-registry integration | 292 | Full instruction matrix (long-running) |
| terra-api | 73 | Route validation + storage helpers |
| terra-geo | 4 | Graph reachability |

Verified on `dev` (2026-09-25, RRR migration): registry lib 116/116, identity lib 6/6, rfc012 21/21, API 73/73, geo 4/4, full registry BPF suite 276/292 (the 16 failures are pre-existing at HEAD — verified against a stashed baseline: guardian/ZK/dispute-slash identity-path and `GenerateOwnershipRoot` account-count mismatches), `cargo fmt` + `clippy -D warnings` clean, `cargo build-sbf` OK, checked-in IDL matches source (no `Parcel.owner` — ownership lives in the Rights PDA), `tsc --noEmit` clean. Fast baseline: `make test-fast`.

## Current Status (as of 2026-09-24)

**Done:**
- All RFC-003…011 protocol modules implemented on-chain (see [architecture.md](docs/architecture.md)).
- RFC-012 **Phase 0** (architecture contract + `rfc012_structure` tests) and **Phase 1** (security hardening: unique validator sets, endorsement action binding, `remaining_accounts` ownership checks, admin constraints) — details in [SECURITY.md](SECURITY.md).
- **A1 security residuals** (2026-09-24): M-2 foreign-entity audit guard, C-4 session-record signers + registry, L-1 `dec_rights_count`; unit + BPF tests green.
- RFC-012 **Phase 3** (verification tasks, 2026-09-24): `verification_task.rs` with `VerificationTask`/`TaskRequirement`/`TaskAssignment` PDAs; 6 instructions (create/add/assign/claim/submit/cancel); 6 unit tests + 2 BPF tests (`phase3_*`); IDL 134/52/120/182.
- RFC-012 **Phase 4** (multi-source observations, 2026-09-24): `observation_v2.rs` with `ObservationV2` PDA (seeds `["observation_v2", task_id, observer, nonce]`); instruction `submit_observation_v2`; source PHONE/GNSS/CAMERA/DRONE/SATELLITE/HUMAN/DOCUMENT/API; provenance SELF_REPORTED…SATELLITE_CONFIRMED; subject ≠ capture_device ≠ observer (Design Rule 7); 4 unit tests + 2 BPF tests (`phase4_*`); IDL 135/53/121/184.
- RFC-012 **Phase 5** (evidence provenance, 2026-09-24): `evidence_manifest.rs` with `EvidenceManifest` (seeds `["evidence_manifest", task_id, submitter, nonce]`) + `EvidenceArtifact` (seeds `["evidence_artifact", manifest, artifact_index]`); instructions `submit_evidence_manifest` / `add_evidence_artifact`; artifact kinds PHOTO/DOCUMENT/VIDEO/MODEL/GEOMETRY/OTHER; append-only index + cap 32; 5 unit tests + 2 BPF tests (`phase5_*`); errors 6184–6186; IDL 137/55/123/187.
- RFC-012 **Phase 6** (dynamic routing, 2026-09-24): `routing.rs` multi-factor eligibility (Eligibility → Capability → Jurisdiction → Geo → Availability → Reputation → independence + randomized pick) + instruction `route_task` (requester-signed; candidates in args; per-candidate PDAs in `remaining_accounts` with stride `3 + need_geo + need_cap`) creating `TaskAssignment` for the random winner; event `TaskRouted`; errors `ValidatorNotEligible` (6187), `NotRouteWinner` (6188), `TooManyRouteCandidates` (6189), `RouteAccountMismatch` (6190); 7 unit tests + 2 BPF tests (`phase6_*`); no new PDAs (reuses Phase 2/3 accounts); IDL 138/55/124/191.
- RFC-012 **Phase 7** (reputation governance, 2026-09-24): `fraud_governance.rs` implements fraud report → random well-reputed committee vote → **capability demotion, never jail** (upheld: capability → DECLARED, tier → PROBATIONARY; Design Rules 13/14) → appeal (24 h window, fresh committee) → rehabilitation (30-day clean window, self-lift). Accounts `FraudReport`/`ReviewCase`/`CapabilityRestriction`/`Appeal`; 9 instructions; `route_task` stride extended to `3 + need_geo + 2*need_cap` with ACTIVE-restriction ineligibility gate; errors 6191–6205; 15 unit tests + 6 BPF tests (`phase7_*`); IDL 147/59/133/206. Legacy `jail_validator`/`unjail_validator` (RFC-005) is not part of this path (deprecation candidate).
- RFC-012 **Phase 8** (economic/resource layer, 2026-09-24): `task_economics.rs` implements quote → task escrow → reward + coverage subsidy → refund. Accounts `FeePolicy`/`CoverageIncentive` (admin-set policies), `ResourceQuote`, `TaskEscrow` (+ vault PDA, fee sink `task_treasury`), `RewardAllocation`; 6 instructions (`set_fee_policy`, `set_coverage_incentive`, `quote_task_resources`, `fund_task_escrow`, `claim_task_reward`, `refund_task_escrow`); subsidy = demand + deficit + difficulty + strategic class bonus, clamped; errors 6206–6219 (reuses 6175/6176/6174); 11 unit tests + 3 BPF tests (`phase8_*`); IDL 153/64/139/220. Rent-exempt guard: fund requires `amount >= rent_min × required`, fee waived when treasury would be left sub-rent-exempt, claim caps subsidy at `treasury − rent_min`.
- **A2 IDL regeneration** (2026-09-24): `ValidatorSlashed` collision resolved → `ReputationSlashed` (staking keeps `ValidatorSlashed` per RFC-005); checked-in `terra-web` IDL synced to **119/44/108/160**; **Phase 2** re-synced to **128/49/115/169**; **Phase 3** re-synced to **134/52/120/182**; **Phase 4** re-synced to **135/53/121/184**; **Phase 5** re-synced to **137/55/123/187**; **Phase 6** re-synced to **138/55/124/191**; **Phase 7** re-synced to **147/59/133/206**; **Phase 8** re-synced to **153/64/139/220**; `make idl`/`build.sh` now use `anchor idl build -p …`.
- **A3 test/CI baseline** (2026-09-24): fixed `StoredArtifact` API test compile break; CI runs identity lib + rfc012 + geo; `make test-fast` for constrained machines.
- PostGIS mirror API (23 routes, migrations `0001`…`0024`) + geo-engine + workspace CI green on `dev`.

**Open / next (in priority order):**
1. Close remaining SECURITY.md items before mainnet: RFC-005 staking governance reconfirm, ZK circuit choice (RFC-006/011). L-3 is cosmetic only. IDL regen is done (A2, 119/44/108/160; Phase 2, 128/49/115/169; Phase 3, 134/52/120/182; Phase 4, 135/53/121/184; Phase 5, 137/55/123/187; Phase 6, 138/55/124/191; Phase 7, 147/59/133/206; Phase 8, 153/64/139/220).
2. Regenerate checked-in IDL: `make idl` (or `./build.sh`) after any program change; `terra-web/src/idl/terra_registry.json` matches source as of Phase 8 (153/64/139/220, 2026-09-24).
3. Devnet deploy: `./deploy.sh devnet` (needs AVX-capable machine for `solana-test-validator`).
4. ZK circuit selection + external audit (RFC-006/011) — proof bytes still opaque, no on-chain Groth16.
5. Governance reconfirm on RFC-005 staking before mainnet (code exists; RFC originally cautioned against implementing without a decision).
6. RFC-012 Phases 8–10 (economics → infrastructure → cross-border/privacy) — see [RFC-012 §8](../../docs/rfc-012-global-physical-digital-trust-architecture.md).

Anyone picking this up: start from root [README.md](../README.md) Status + Devnet checklist, then [SECURITY.md](SECURITY.md) Recommendations, then RFC-012 phase table. Do not invent counts — regenerate with `rg`/tests or `make idl`.

## Error Codes

- **terra_registry:** 184 custom error codes (`TerraError` in `lib.rs`).
- **terra_identity:** 29 custom error codes (`IdentityError` in `errors.rs`, 6000+).

## License

This project is proprietary. All rights reserved.
