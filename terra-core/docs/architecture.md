# Terra Architecture

## System Overview

Terra is a decentralized land registry system on Solana built with Anchor. It manages land parcels, rights, escrow, staking, verification, and cross-border governance through two on-chain programs.

## Programs

| Program | ID | Crate | Purpose |
|---------|-----|-------|---------|
| `terra_registry` | `GaEDbktvpZ3qiqp4PmFgHwDSa6JsFfVjXFqNb2nTbage` | `terra-registry` | Core land registry, escrow, staking, verification, vaults, ZK proofs |
| `terra_identity` | `68urV9nGcRcoWT1QjzZfXuCnTS9921x2se1SybKJr1U4` | `terra-identity` | Identity management, succession, guardianship |

## Module Map

```
terra_registry/
├── lib.rs                    # Entry point, context structs, TerraError (155 codes)
├── cross_border.rs           # Cross-border jurisdiction + identity binding
├── dispute.rs                # Parcel dispute filing, freeze, adjudicate, execute
├── escrow.rs                 # Parcel escrow (create, deposit, accept, settle, cancel)
├── staking.rs                # Validator staking pool, slashing, rewards
├── vault.rs                  # Encrypted vault, shard holders, rotation
├── zk.rs                     # ZK ownership proofs, threshold credentials
├── quorum.rs                 # Quorum signers verification utility
├── world_registry.rs         # Country allocation, genesis confirmation
├── subdivision.rs            # Parcel subdivision and amalgamation
├── time_bound.rs             # Time-bound rights (grant, renew, sweep)
├── recovery.rs               # Validator liveness, emergency injection
├── validator_registry.rs     # Validator onboarding, nomination, pause
├── ipfs_docs.rs              # IPFS document registration
└── verification/
    ├── claim.rs              # Verification claims (14 types)
    ├── evidence.rs           # Claim evidence (13 types)
    ├── observation.rs        # Validator observations
    ├── attestation.rs        # Verification attestations
    ├── session.rs            # Verification sessions
    ├── challenge.rs          # Claim challenges
    ├── reputation.rs         # Validator reputation scoring + jail/unjail
    ├── quorum_config.rs      # Quorum configuration per parcel type
    ├── quorum_voting.rs      # Weighted quorum voting
    ├── observer.rs           # Observer registry
    ├── guardian_claim.rs     # Guardian claim bridge
    ├── cross_border_bridge.rs # Cross-border verification
    ├── bridge.rs             # Legacy attestation-to-claim migration
    └── audit_trail.rs        # Append-only audit log
```

## Data Model

### Core Accounts

| Account | PDA Seeds | Description |
|---------|-----------|-------------|
| `Parcel` | `["parcel", id]` | Land parcel record |
| `Rights` | `["rights", parcel, nonce]` | Wallet-based right |
| `IdentityRights` | `["identity_rights", identity, parcel, rights_kind]` | Identity-based right |
| `Attestation` | `["attestation", parcel, specifier]` | Legacy attestation (deprecated) |

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

## Lifecycle Flows

### Parcel Lifecycle

```
REGISTER (signer → owner)
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
SUBMIT ATTESTATION (validator attests)
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

1. **Dual ownership:** Wallet-based (`Parcel.owner`) and identity-based (`IdentityRights`) ownership paths via `is_authorized_owner()`
2. **Anti-grief:** Minimum 2 validators for disputes, self-dealing checks everywhere
3. **Progressive decentralization:** Bootstrap phase (1-3 validators) then peer-consensus endorsement
4. **Emergency pause:** Admin can pause the program; most handlers check `require_not_paused()`
5. **Deprecation bridge:** Legacy `Attestation` → `Claim` via `migrate_attestation_to_claim`

## Error Codes

155 custom error codes (6000-6154) defined in the `TerraError` enum in `lib.rs`.

## Constants

| Module | Key Constants |
|--------|--------------|
| `parcel_status` | PENDING=0 through AMALGAMATED=9 |
| `right_kind` | OWNERSHIP=0, TENANCY=1, LEASE=2, LIEN=3 |
| `escrow` | SETTLEMENT_WINDOW=3 days, CANCEL_WINDOW=7 days, MIN=0.1 SOL, MAX=1M SOL |
| `staking` | UNBONDING_PERIOD=7 days, MIN_STAKE=1 SOL, REWARD_INTERVAL=1 day |
| `vault` | MAX_SHARD_HOLDERS=8, PING_INTERVAL=7 days, ROTATION_TIMELOCK=7 days |
| `dispute` | MIN_VALIDATORS=2, EXPIRY=90 days |
| `reputation` | MAX=10,000 bps, JAIL_DURATION=7 days, AUTO_JAIL_THRESHOLD=2,000 bps |
| `session` | TIMEOUT=30 days |
| `challenge` | REVIEW_PERIOD=14 days |
| `cross_border` | MAX_PROOF_LEN=512, MAX_JURISDICTION_NAME=64 |
| `world_registry` | GENESIS_MIN_CONFIRMATIONS=5, MIN_DISTINCT_COUNTRIES=3 |
