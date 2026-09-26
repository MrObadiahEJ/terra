# Terra Architecture

> Part of **Terra — Infrastructure for a Verifiable Digital Representation of the
> Physical World** (land is the first domain). North star:
> [`../../../docs/VISION.md`](../../../docs/VISION.md); repo status:
> [`../../README.md`](../../README.md).

## System Overview

Terra is a decentralized land claim & verification network on Solana built with Anchor — the on-chain core of the wider Terra vision. It manages land parcels, rights, escrow, staking, verification, vaults, ZK proofs, and cross-border governance through two on-chain programs, a PostGIS mirror API, and a geo engine.

**Program IDs** (devnet / localnet / mainnet slots in `Anchor.toml`):

| Program | ID | Crate | Purpose |
|---------|-----|-------|---------|
| `terra_registry` | `GaEDbktvpZ3qiqp4PmFgHwDSa6JsFfVjXFqNb2nTbage` | `terra-registry` | Core land registry, escrow, staking, verification, vaults, ZK proofs |
| `terra_identity` | `68urV9nGcRcoWT1QjzZfXuCnTS9921x2se1SybKJr1U4` | `terra-identity` | Identity management, succession, guardianship |

**Source counts** (as of 2026-09-26, RFC-012 Phase 9 device identities after legacy sweep + B6 boundary split + P0-2 removal-endorsement path): `terra_registry` — 154 instructions, 63 `#[account]` types, 141 events, 225 `TerraError` codes. `terra_identity` — 8 instructions, 2 accounts, 8 events, 29 `IdentityError` codes. Regenerate IDL with `make idl` after program changes.

## Module Map

```
terra_registry/
├── lib.rs                    # Entry point, context structs, TerraError (225 codes)
├── device_identity.rs        # RFC-012 Phase 9 device identity registry
├── task_economics.rs         # RFC-012 Phase 8 quotes/escrow/claims/refunds + coverage subsidy
├── fraud_governance.rs       # RFC-012 Phase 7 fraud report/committee/appeal/restriction
├── evidence_manifest.rs     # RFC-012 Phase 5 evidence manifest/artifacts
├── routing.rs               # RFC-012 Phase 6 multi-factor dynamic routing
├── observation_v2.rs         # RFC-012 Phase 4 multi-source ObservationV2
├── verification_task.rs      # RFC-012 Phase 3 task PDAs
├── cross_border.rs           # Cross-border jurisdiction + identity binding
├── dispute.rs                # Parcel dispute filing, freeze, adjudicate, execute
├── escrow.rs                 # Parcel escrow (create, deposit, accept, settle, cancel)
├── staking.rs                # Validator staking pool, slashing, rewards
├── vault.rs                  # Encrypted vault, shard holders, rotation
├── zk.rs                     # ZK ownership proofs, threshold credentials
├── quorum.rs                 # Quorum signers + unique-validator utilities
├── world_registry.rs         # Country allocation, genesis confirmation
├── subdivision.rs            # Parcel subdivision and amalgamation
├── time_bound.rs             # Time-bound rights (grant, renew, sweep)
├── recovery.rs               # Validator liveness, emergency injection
├── validator_registry.rs     # Validator onboarding, nomination, endorse add/remove
└── verification/
    ├── claim.rs              # Verification claims (14 types)
    ├── evidence.rs           # Claim evidence (13 types)
    ├── observation.rs        # Validator observations
    ├── attestation.rs        # Verification attestations
    ├── session.rs            # Verification sessions + quorum_config load
    ├── challenge.rs          # Claim challenges
    ├── reputation.rs         # Validator reputation scoring + auto-jail
    ├── quorum_config.rs      # Quorum configuration per parcel type
    ├── quorum_voting.rs      # Weighted quorum voting
    ├── observer.rs           # Observer registry
    ├── guardian_claim.rs     # Guardian claim bridge
    ├── cross_border_bridge.rs # Cross-border verification
    └── audit_trail.rs        # Append-only audit log

terra_identity/
├── lib.rs                    # Program entry (8 instructions)
├── state.rs                  # Identity, Succession accounts
├── errors.rs                 # IdentityError (29 codes, 6000+)
├── helpers.rs                # count_unique_validators + unit tests
└── instructions/
    ├── bind_identity.rs
    ├── succession.rs          # request/endorse/cancel/claim — identity-only (B6)
    └── guardianship.rs       # RFC-010 court guardianship
```

## Data Model

### Core Accounts

| Account | PDA Seeds | Description |
|---------|-----------|-------------|
| `Parcel` | `["parcel", id]` | Land parcel record |
| `Rights` | `["rights", parcel, nonce]` | Wallet-based right |
| `IdentityRights` | `["identity_rights", identity, parcel, rights_kind]` | Identity-based right |

### Escrow & Staking

| Account | PDA Seeds | Description |
|---------|-----------|-------------|
| `EscrowRecord` | `["escrow", parcel]` | One escrow per parcel |
| `StakePool` | `["stake_pool", region_registry]` | One pool per region |
| `ValidatorStake` | `["validator_stake", pool, validator]` | Per-validator stake |
| `SlashingReport` | `["slashing_report", pool, reporter, evidence_hash]` | Slashing evidence |

### Vault & ZK

| Account | PDA Seeds | Description |
|---------|-----------|-------------|
| `VaultRecord` | `["vault_record", subject]` | Encrypted vault per identity |
| `VaultShardRotation` | `["vault_shard_rotation", vault, new_hash]` | Time-locked rotation |
| `ZoneSet` | `["zone_set", zone_id]` | ZK zone |
| `OwnershipRoot` | `["ownership_root", zone_set]` | Merkle root |
| `NullifierRecord` | `["nullifier", nullifier_hash]` | Prevents proof double-use |
| `CredentialRequest` | `["credential_request", request_hash]` | Threshold credential request |
| `ThresholdCredential` | `["threshold_credential", request_hash]` | Issued credential |

### Verification Pipeline

| Account | PDA Seeds | Description |
|---------|-----------|-------------|
| `Claim` | `["claim", parcel, claim_id]` | Verification claim |
| `Evidence` | `["evidence", claim, nonce]` | Claim evidence |
| `Observation` | `["observation", claim, validator]` | Validator observation |
| `VerificationAttestation` | `["verification_attestation", claim, validator]` | Attestation |
| `VerificationSession` | `["verification_session", claim, session_id]` | Session |
| `Challenge` | `["challenge", claim, challenger]` | Claim challenge |
| `ValidatorReputation` | `["validator_reputation", validator]` | Reputation record |
| `QuorumConfig` | `["quorum_config", parcel_type, region]` | Quorum settings |
| `QuorumVote` | `["quorum_vote", claim, voter]` | Weighted vote |
| `QuorumTally` | `["quorum_tally", claim]` | Aggregated tally |

### World & Cross-Border

| Account | PDA Seeds | Description |
|---------|-----------|-------------|
| `WorldRegistry` | `["world_registry"]` | Global singleton |
| `GenesisRequest` | `["genesis_request", country_code]` | Genesis confirmation |
| `Jurisdiction` | `["jurisdiction", country_code]` | Cross-border jurisdiction |
| `JurisdictionBinding` | `["cross_border_identity", jurisdiction_key, identity_hash]` | Identity binding |
| `CrossBorderVerification` | `["cross_border_verification", binding]` | Cross-border verification |

### RFC-012 Phase 2 — Validator Profiles

| Account | PDA Seeds | Description |
|---------|-----------|-------------|
| `ValidatorProfile` | `["validator_profile", wallet]` | Tier / jurisdiction / metadata |
| `ValidatorPresence` | `["validator_presence", wallet]` | Mobile presence fix |
| `ValidatorAvailability` | `["validator_availability", wallet]` | Online / offline / busy |
| `ValidatorCapability` | `["validator_capability", wallet, capability_code]` | Declared / verified capability |
| `ValidatorRelationshipEdge` | `["validator_edge", from, to, edge_type]` | Independence graph |

### RFC-012 Phase 3 — Verification Tasks

| Account | PDA Seeds | Description |
|---------|-----------|-------------|
| `VerificationTask` | `["task", task_id]` | First-class work unit |
| `TaskRequirement` | `["task_requirement", task_id, req_index]` | Eligibility / evidence constraints |
| `TaskAssignment` | `["task_assignment", task_id, validator]` | Selected validator |

### RFC-012 Phase 4 — Multi-Source Observations

| Account | PDA Seeds | Description |
|---------|-----------|-------------|
| `ObservationV2` | `["observation_v2", task_id, observer, nonce]` | Subject / capture_device / observer roles + source + provenance |

### RFC-012 Phase 5 — Evidence Provenance

| Account | PDA Seeds | Description |
|---------|-----------|-------------|
| `EvidenceManifest` | `["evidence_manifest", task_id, submitter, nonce]` | Structured artifact list + root hash |
| `EvidenceArtifact` | `["evidence_artifact", manifest, artifact_index]` | One photo/document/video/model/geometry |

### RFC-012 Phase 9 — Device Identity

| Account | PDA Seeds | Description |
|---------|-----------|-------------|
| `DeviceIdentity` | `["device_identity", owner, device_nonce]` | Device key, capture source, capabilities, calibration, status (u16 LE nonce) |

### Identity Program

| Account | Program | Description |
|---------|---------|-------------|
| `Identity` | terra_identity | Cryptographic identity anchor |
| `Succession` | terra_identity | Succession / guardianship request |

## Lifecycle Flows

### Parcel Lifecycle

```
REGISTER (signer → holder)
  ↓
ACTIVE → FOR_SALE (escrow) → SOLD
       → DISPUTED (freeze) → RESOLVED
       → SUBDIVIDED → 2 child parcels
       → AMALGAMATED → 1 merged parcel
       → REVOKED
```

### Escrow Flow

```
CREATE (seller initiates, parcel must be FOR_SALE)
  ↓
DEPOSIT (buyer deposits SOL)
  ↓
ACCEPT (seller accepts, starts 3-day settlement window)
  ↓
SETTLE (after deadline: parcel → buyer, SOL → seller)
```

### Verification Pipeline

```
CREATE CLAIM (submitter, parcel, claim_type)
  ↓
ADD EVIDENCE (submitter attaches proofs)
  ↓
SUBMIT OBSERVATION (validator observes)
  ↓
SUBMIT VERIFICATION ATTESTATION (validator signs)
  ↓
VERIFY CLAIM (auto-resolves when quorum met)
  ↓
[optional] FILE CHALLENGE (challenger disputes)
  ↓
VOTE CHALLENGE (validators vote, 14-day review)
```

### Staking Flow

```
CREATE STAKE POOL (admin, per region)
  ↓
DEPOSIT STAKE (validator bonds SOL)
  ↓
[active validation, rewards accrue]
  ↓
INITIATE UNBONDING (7-day lockup)
  ↓
WITHDRAW STAKE (after unbonding)
```

### Vault Shard Rotation

```
CREATE VAULT (identity owner)
  ↓
INITIATE ROTATION (time-locked, 7 days)
  ↓
ENDORSE (2/3 shard holders sign)
  ↓
EXECUTE (after timelock + endorsements)
```

## Key Patterns

1. **Single-source ownership (RRR):** Every parcel's canonical holder lives in the `Rights` PDA `["ownership", parcel]` (`rights_kind == OWNERSHIP`, field `holder`) — a wallet **or** an Identity PDA. `is_authorized_holder()` checks the signer directly (fast path) or resolves a `terra_identity`-owned Identity holder via `remaining_accounts`. `Parcel.owner` and `is_authorized_owner()` were hard-removed (RFC-012 §9).
2. **Anti-grief:** Minimum 2 validators for disputes/forfeiture, self-dealing checks everywhere, **unique validator sets** (`require_unique_validators`).
3. **Progressive decentralization:** Bootstrap phase (1-3 validators) then peer-consensus endorsement (`endorse_validator_add` enforces ADD action).
4. **Emergency pause:** Admin can pause the program; most handlers check `require_not_paused()`.
5. **Deprecation bridge (closed 2026-09-25):** the legacy `Attestation` account and its `attest` / `rotate_validators` / `register_document` / `migrate_attestation_to_claim` / `migrate_attestations` instructions were hard-removed; the claim pipeline (Claim → Evidence → Observation → VerificationAttestation) is the only path.
6. **Remaining-accounts hygiene:** Session, quorum_config, reputation, and observation loaders verify `acc.owner == &crate::ID` before deserialize.

## Program Boundary Rules (B6, 2026-09-25)

The two programs are strictly layered — no program may write another's accounts:

1. **`terra_registry` reads `terra_identity` (read-only):** `is_authorized_holder()` resolves Identity PDAs via `remaining_accounts`; ownership and provenance checks deserialize identity state without mutating it.
2. **`terra_registry` invokes `terra_identity` via CPI:** `claim_succession_with_parcels` claims a succession (CPI into `terra_identity::claim_succession` with the hard-coded discriminator const) and then re-points the registry-owned `["ownership", parcel]` `Rights` PDAs to the successor — one atomic transaction; either leg failing rolls both back.
3. **`terra_identity` has no dependency on `terra_registry`** and never reads or writes registry state. Its stale pre-RRR `Parcel` mirror was deleted in B6; the `Parcel*` identity error codes are retained (29 codes unchanged) for IDL/error stability.
4. **New cross-cutting flows follow the same shape:** the registry-side instruction composes (CPI + registry-owned writes only); the identity-side instruction stays identity-only. CPI targets are explicit `UncheckedAccount` fields checked with `require_keys_eq!`, and the callee program account is included in the outer instruction's metas.
5. **Test layout:** identity-subject tests (bind, succession endorse/claim/cancel, guardianship) live in `programs/terra_identity/tests/integration.rs` (B6 split, 23 tests). Composition tests live at the end of `programs/terra_registry/tests/integration.rs` (`b6_composite_*`, incl. atomic-rollback and error-mapping cases).

## Error Codes

- `terra_registry`: **220** custom codes in `TerraError` (starts at Anchor 6000; ends with `NothingToRefund`).
- `terra_identity`: **29** custom codes in `IdentityError` (starts at 6000; ends with `DuplicateValidator`).

## Constants

| Module | Key Constants |
|--------|--------------|
| `parcel_status` | PENDING=0 … AMALGAMATED=9 |
| `right_kind` | OWNERSHIP=0, USAGE=1, EASEMENT=2, SERVITUDE=3, LIEN=4 |
| `escrow` | SETTLEMENT_WINDOW=3 days, CANCEL_WINDOW=7 days, MIN=0.1 SOL, MAX=1M SOL |
| `staking` | UNBONDING_PERIOD=7 days, MIN_STAKE=1 SOL, REWARD_INTERVAL=1 day, FIRST_OFFENSE=10%, REPEAT=100% |
| `vault` | MAX_SHARD_HOLDERS=8, PING_INTERVAL=7 days, ROTATION_TIMELOCK=7 days |
| `dispute` | MIN_DISPUTE_VALIDATORS=2, MIN_FORFEIT_VALIDATORS=2, EXPIRY=90 days |
| `reputation` | MAX=10,000 bps, JAIL_DURATION=7 days, auto-jail below 2,000 bps |
| `session` | TIMEOUT=30 days |
| `challenge` | REVIEW_PERIOD=14 days |
| `cross_border` | MAX_PROOF_LEN=512, MAX_JURISDICTION_NAME=64 |
| `world_registry` | GENESIS_MIN_CONFIRMATIONS=5, MIN_DISTINCT_COUNTRIES=3 |
| `MAX_VALIDATORS` | 8 (per declared set) |

## Workspace & Ops

| Item | Location |
|------|----------|
| Workspace members | `programs/terra_registry`, `programs/terra_identity`, `api`, `geo-engine` |
| API routes | `terra-core/api/src/routes/` — 21 modules |
| Migrations | `terra-core/api/migrations/` — `0001`…`0026` |
| Build | `cargo build-sbf` (see `build.sh`, `SBF_OUT_DIR` in `.cargo/config.toml`) |
| Make targets | `build`, `test`, `lint`, `fix`, `idl`, `deploy-*`, `clean`, `size`, `verify-devnet` |
| CI | `.github/workflows/ci.yml` — fmt, clippy, registry lib tests, API+PostGIS, tsc |

See also: [RFC-012](../../docs/rfc-012-global-physical-digital-trust-architecture.md) (architecture contract) and [SECURITY.md](../SECURITY.md) (audit status).

## Current State & Next Steps (handoff)

**Complete as of 2026-09-23:**
- All modules in the map above are implemented; source counts (§Source counts) match `dev`.
- 2026-09-25: RFC-012 legacy sweep (commit `72d052d`) and the B6 program-boundary split are merged — see §Program Boundary Rules above; identity-subject tests moved to `terra_identity/tests/integration.rs`.
- RFC-012 Phase 0 (contract + `rfc012_structure` 21 tests) and Phase 1 (unique validator sets, endorsement binding, ownership checks, admin constraints) — see SECURITY.md and RFC-012 §8.

**Next work (do not skip order):**
1. Security residuals before mainnet: RFC-005 staking reconfirm, ZK audit (SECURITY.md Recommendations). M-2/L-1/C-4 closed in A1; IDL regen done in A2; test/CI baseline done in A3 (`make test-fast`).
2. `make idl` — refresh `terra-web/src/idl/` after any program edit (checked-in IDL matches 154/63/141/225 as of RFC-012 Phase 9, 2026-09-26).
3. Devnet: `./deploy.sh devnet` + local `solana-test-validator` (AVX required).
4. ZK: pick circuit (Groth16/PLONK), external audit — `zk.rs` is structural only.
5. RFC-005 staking: governance reconfirm before mainnet (code path exists).
6. **RFC-012 Phase 2+** — new PDAs from RFC-012 §10 (`ValidatorProfile`, presence, tasks, …); update this module map, migration map, and `rfc012_structure` tests when adding entities. Start from RFC-012 §8–§10.

Entry point for new contributors/AI: root `README.md` → “How to continue (handoff)”.
