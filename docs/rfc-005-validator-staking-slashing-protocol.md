# RFC-005: Validator Staking & Slashing Protocol

## 1. Status

- **Status:** Draft (NOT SCHEDULED — architectural fork)
- **Created:** 2026-09-03
- **Supersedes:** None

> **CRITICAL: DO NOT IMPLEMENT WITHOUT GOVERNANCE DECISION.**
>
> This RFC proposes a staking/slashing layer on top of Terra's validator set. It **directly conflicts** with the trust model established by AuthorityRegistry (`authority_registry.rs`). AuthorityRegistry bootstraps trust from **institutional appointment** — a vetted admin adds validators, then peer consensus takes over. Staking bootstraps trust from **capital** — the wealthiest participant dictates outcomes. A wealthy adversary can out-stake and out-vote honest appointed validators, subverting the entire authority hierarchy.
>
> If pursued at all, the only primitive-spirited version is **stake as an additional requirement layered on top of appointment** ("must be AuthorityRegistry-appointed AND post a bond"), never staking *instead of* appointment, and never allowing delegation to let non-validators buy influence.
>
> **Target phase:** NOT SCHEDULED. Requires an explicit governance decision, not a target phase.

## 2. Summary

This RFC specifies the **Validator Staking & Slashing Protocol** — an optional economic security layer for Terra. Validators deposited SOL as a bond against misbehavior. Slashing conditions include equivocation (two conflicting signatures on the same parcel), prolonged liveness failure, and collusion evidence. Rewards are distributed proportional to stake × participation. **Delegation is explicitly prohibited** — only AuthorityRegistry-appointed validators may stake, and third parties cannot buy influence through delegation.

The on-chain program tracks stake pools, individual validator stakes, slashing reports, and reward accrual. Off-chain detection systems identify equivocation and liveness failures. Slashing is gradual (first offense = 10%, repeat = 100%) with a 7-day appeal window.

**Prerequisites:** AuthorityRegistry must be active for the target region. This protocol is scoped per region, parallel to the per-region vault and attestation models.

## 3. Threat Model

### 3.1 Adversary Classes

| Class | Description | Mitigation |
|-------|-------------|------------|
| **Rogue validator** | Signs contradictory attestations for the same parcel | Equivocation detection → slash 10-100% of stake |
| **Liveness adversary** | Validator goes offline, stalls consensus or reconstruction | Prolonged liveness failure → slash + replace via AuthorityRegistry |
| **Colluding subset** | Multiple validators conspire on succession fraud or unauthorized data access | Collusion evidence reporting → slash all participants |
| **Capital attacker** | Wealthy adversary tries to out-stake honest validators | **No delegation** — only AuthorityRegistry-appointed validators can stake. Stake does not grant appointment |
| **Slashing abuse** | False equivocation reports to grief honest validators | Reporter must post bond; failed reports lose bond. Appeal process (7 days) |
| **Bribery attacker** | Offers validators bribes to misbehave | Stake creates economic cost; slashing removes profit from bribery |
| **Sybil validator** | Tries to register multiple identities to dilute the validator set | AuthorityRegistry prevents sybil — each validator is a distinct institutional appointment |

### 3.2 In Scope

- Validator staking (deposit, withdrawal, unbonding)
- Equivocation detection and slashing
- Liveness failure detection and slashing
- Collusion evidence reporting and execution
- Reward distribution proportional to stake × participation
- Appeal process for disputed slashes

### 3.3 Out of Scope

- Vault shard protocol (covered by RFC-003)
- Attestation, succession, and forfeiture flows (covered by RFC-001/002)
- Validator identity verification (covered by AuthorityRegistry)
- Delegation pools (explicitly prohibited — see Section 3.1)

## 4. Cryptographic Choices

### 4.1 Signatures: Ed25519

- Solana native — all validator keys are Ed25519
- Used for transaction signing (on-chain staking operations)
- Equivocation detection relies on two Ed25519 signatures on the same parcel with different attestations

### 4.2 Hashing: SHA-256

- Content addressing for evidence records
- PDA seed derivation for on-chain accounts
- Reward accumulation hashes for tamper-evidence

### 4.3 Evidence Binding: Content-Addressed Hashes

- Equivocation evidence is bound by `sha256(attestation_a || attestation_b)` — the two conflicting attestations
- Collusion evidence is bound by `sha256(signer_set || action_hash)` — the set of colluding signers and the fraudulent action
- These hashes are stored on-chain as PDA seeds, anchoring off-chain evidence to on-chain accounts

### 4.4 Post-Quantum Migration: Algorithm ID

- `u8` enum in StakePool: `0 = Ed25519 (current)`, `1 = Dilithium-3 + Ed25519 hybrid (future)`
- Only variant 0 is implemented for the pilot
- Migration to post-quantum signatures is orthogonal to the staking mechanism

## 5. Data Model

### 5.1 StakePool (on-chain PDA)

**PDA seed:** `["stake_pool", region_registry_key]`

One stake pool per region, derived from the AuthorityRegistry key for that region.

| Field | Type | Description |
|-------|------|-------------|
| `region_registry` | `Pubkey` | AuthorityRegistry key for this region |
| `total_staked` | `u64` | Total SOL staked across all validators (lamports) |
| `reward_rate_bps` | `u16` | Annual reward rate in basis points (e.g., 500 = 5%) |
| `accumulated_rewards` | `u64` | Total rewards accrued but not yet distributed (lamports) |
| `last_reward_distribution` | `i64` | Timestamp of last reward distribution |
| `slash_count` | `u32` | Total slashing events executed |
| `created_at` | `i64` | Pool creation timestamp |
| `updated_at` | `i64` | Last update timestamp |

### 5.2 ValidatorStake (on-chain PDA)

**PDA seed:** `["validator_stake", stake_pool_key, validator_key]`

One record per validator per region.

| Field | Type | Description |
|-------|------|-------------|
| `stake_pool` | `Pubkey` | StakePool key |
| `validator` | `Pubkey` | Validator's Ed25519 pubkey |
| `staked_amount` | `u64` | Current stake in lamports |
| `unbonding_amount` | `u64` | Amount in unbonding period |
| `unbonding_starts_at` | `i64` | When unbonding began (0 if not unbonding) |
| `rewards_accrued` | `u64` | Rewards accumulated but not yet claimed |
| `slash_history` | `u8` | Number of past slashing events (for graduated severity) |
| `offenses` | `[u8; 4]` | Recent offense flags: [equivocation, liveness, collusion, unused] |
| `created_at` | `i64` | Stake creation timestamp |
| `updated_at` | `i64` | Last update timestamp |

### 5.3 SlashingReport (on-chain PDA)

**PDA seed:** `["slashing_report", stake_pool_key, reporter_key, evidence_hash]`

One record per report, created by the reporter.

| Field | Type | Description |
|-------|------|-------------|
| `stake_pool` | `Pubkey` | StakePool key |
| `reporter` | `Pubkey` | Wallet that filed the report |
| `evidence_hash` | `[u8; 32]` | SHA-256 of the evidence payload |
| `offender` | `Pubkey` | Validator accused of misbehavior |
| `offense_type` | `u8` | 0=equivocation, 1=liveness, 2=collusion |
| `offense_details` | `[u8; 64]` | Bounded details (e.g., two conflicting parcel hashes) |
| `reporter_bond` | `u64` | SOL bonded by reporter (for false-report penalty) |
| `status` | `u8` | 0=Pending, 1=Verified, 2=Slashed, 3=Appealed, 4=Rejected, 5=Dismissed |
| `filed_at` | `i64` | When the report was filed |
| `appeal_deadline` | `i64` | filed_at + APPEAL_WINDOW_SECS |
| `resolved_at` | `i64` | When the report was resolved (0 if pending) |

### 5.4 RewardAccrual (emitted as event)

| Field | Type | Description |
|-------|------|-------------|
| `stake_pool` | `Pubkey` | StakePool key |
| `validator` | `Pubkey` | Validator key |
| `amount` | `u64` | Reward amount in lamports |
| `block_time` | `i64` | Solana block time |

### 5.5 StakePool Constants

| Constant | Value | Description |
|----------|-------|-------------|
| `UNBONDING_PERIOD_SECS` | `7 * 24 * 3600` (7 days) | Minimum unbonding window |
| `MAX_UNBONDING_PERIOD_SECS` | `30 * 24 * 3600` (30 days) | Maximum unbonding window |
| `APPEAL_WINDOW_SECS` | `7 * 24 * 3600` (7 days) | Window to appeal a slashing report |
| `LIVENESS_THRESHOLD_SECS` | `7 * 24 * 3600` (7 days) | Max offline duration before liveness offense |
| `FIRST_OFFENSE_SLASH_BPS` | `1000` (10%) | Slash percentage for first offense |
| `REPEAT_OFFENSE_SLASH_BPS` | `10000` (100%) | Slash percentage for repeat offense |
| `MIN_STAKE_LAMPORTS` | `1_000_000_000` (1 SOL) | Minimum stake to register |
| `MAX_VALIDATORS_PER_POOL` | `8` | Maximum validators in a stake pool |
| `REPORTER_BOND_BPS` | `100` (1%) | Reporter must bond this % of potential slash amount |
| `REWARD_DISTRIBUTION_INTERVAL_SECS` | `86400` (1 day) | Minimum interval between reward distributions |

## 6. Instructions

### 6.1 `create_stake_pool`

**Purpose:** Initialize a stake pool for a region. The pool is derived from the AuthorityRegistry key, ensuring one pool per region.

**Accounts:**
- `stake_pool` (init, PDA `["stake_pool", region_registry_key]`)
- `region_registry` (readonly — AuthorityRegistry account, used as seed only)
- `authority` (signer, mut — pays rent; must be the AuthorityRegistry admin)
- `system_program`

**Args:** `reward_rate_bps: u16`

**Guards:**
- `authority` must be the AuthorityRegistry admin for the region
- `reward_rate_bps` must be > 0 and <= 2000 (max 20% annual)
- `region_registry` must exist and be a valid AuthorityRegistry PDA
- No StakePool already exists for this `region_registry`
- `region_registry.validators.len() > 0` (pool cannot be created for an empty registry)

**Effects:**
- Create StakePool account with `total_staked = 0`, `accumulated_rewards = 0`, `slash_count = 0`

**Emits:** `StakePoolCreated`

### 6.2 `deposit_stake`

**Purpose:** A validator deposits SOL as a bond. The validator must be in the AuthorityRegistry for this region.

**Accounts:**
- `stake_pool` (mut)
- `validator_stake` (init, PDA `["validator_stake", stake_pool_key, validator_key]`)
- `validator` (signer, mut — the validator depositing; pays rent + stake)
- `region_registry` (readonly — AuthorityRegistry, used to verify validator membership)
- `system_program`

**Args:** `amount: u64`

**Guards:**
- `amount >= MIN_STAKE_LAMPORTS` (minimum 1 SOL)
- `validator` is in `region_registry.validators`
- `validator` is the signer (validators cannot stake on behalf of others)
- `validator_stake.staked_amount == 0` (first deposit; subsequent increases use `increase_stake`)
- No pending unbonding (`validator_stake.unbonding_amount == 0`)

**Effects:**
- `validator_stake.staked_amount += amount`
- `stake_pool.total_staked += amount`
- Transfer `amount` SOL from `validator` to `stake_pool` PDA

**Emits:** `StakeDeposited`

### 6.3 `withdraw_stake`

**Purpose:** Validator withdraws stake after the unbonding period has elapsed.

**Accounts:**
- `stake_pool` (mut)
- `validator_stake` (mut)
- `validator` (signer, mut — receives withdrawn SOL)
- `system_program`

**Args:** None

**Guards:**
- `validator` is the owner of `validator_stake`
- `validator_stake.unbonding_amount > 0` (must have initiated unbonding)
- `Clock::get()?.unix_timestamp >= validator_stake.unbonding_starts_at + UNBONDING_PERIOD_SECS` (unbonding complete)
- `validator_stake.staked_amount == 0` (all stake must be in unbonding state)

**Effects:**
- `stake_pool.total_staked -= validator_stake.unbonding_amount`
- Transfer `validator_stake.unbonding_amount` SOL from `stake_pool` PDA to `validator`
- Close `validator_stake` account (return lamports to validator)
- `validator_stake.unbonding_amount = 0`

**Emits:** `StakeWithdrawn`

### 6.4 `initiate_unbonding`

**Purpose:** Validator begins the unbonding process. Stake enters a 7-30 day lock-up period.

**Accounts:**
- `validator_stake` (mut)
- `validator` (signer — must be the stake owner)
- `system_program`

**Args:** None

**Guards:**
- `validator` is the owner of `validator_stake`
- `validator_stake.unbonding_amount == 0` (not already unbonding)
- `validator_stake.staked_amount > 0` (has stake to unbond)

**Effects:**
- `validator_stake.unbonding_amount = validator_stake.staked_amount`
- `validator_stake.staked_amount = 0`
- `validator_stake.unbonding_starts_at = Clock::get()?.unix_timestamp`

**Emits:** `UnbondingInitiated`

### 6.5 `report_equivocation`

**Purpose:** Report a validator for signing two conflicting attestations on the same parcel. The reporter must post a bond.

**Accounts:**
- `slashing_report` (init, PDA `["slashing_report", stake_pool_key, reporter_key, evidence_hash]`)
- `stake_pool` (mut)
- `reporter` (signer, mut — pays bond + rent)
- `offender_stake` (mut — the accused validator's stake)
- `system_program`

**Args:** `evidence_hash: [u8; 32]`, `offender: Pubkey`, `offense_details: [u8; 64]`

**Guards:**
- `evidence_hash != [0; 32]`
- `offender` is in `stake_pool` validators (has a `ValidatorStake` account)
- `reporter != offender` (cannot self-report)
- `reporter_bond >= (offender_stake.staked_amount * REPORTER_BOND_BPS) / 10000`
- No duplicate report with the same `evidence_hash` from this reporter
- `offender_stake.staked_amount > 0` (validator has stake to slash)

**Effects:**
- `slashing_report.status = 0` (Pending)
- `slashing_report.reporter_bond = reporter_bond`
- Transfer `reporter_bond` SOL from `reporter` to `slashing_report` PDA (held in escrow)

**Emits:** `EquivocationReported`

### 6.6 `verify_and_slash`

**Purpose:** Execute slashing after evidence review. Can be called by the AuthorityRegistry admin or by a quorum of `ceil(2n/3)` validators who reviewed the evidence.

**Accounts:**
- `slashing_report` (mut)
- `stake_pool` (mut)
- `offender_stake` (mut)
- `authority` (signer — admin or validator quorum via remaining_accounts)
- `system_program`

**Args:** None

**Guards:**
- `slashing_report.status == 0` (Pending)
- Caller is either:
  - (a) The AuthorityRegistry admin, OR
  - (b) At least `ceil(2n/3)` validators from `stake_pool` are signers (via `remaining_accounts`)
- `Clock::get()?.unix_timestamp >= slashing_report.filed_at + 24 * 3600` (24-hour review period elapsed)
- `Clock::get()?.unix_timestamp < slashing_report.appeal_deadline` (still within appeal window — or appeal period expired without appeal)

**Effects:**
- Determine slash percentage: `FIRST_OFFENSE_SLASH_BPS` if `offender_stake.slash_history == 0`, else `REPEAT_OFFENSE_SLASH_BPS`
- `slash_amount = (offender_stake.staked_amount * slash_percent) / 10000`
- `offender_stake.staked_amount -= slash_amount`
- `stake_pool.total_staked -= slash_amount`
- `offender_stake.slash_history += 1`
- `stake_pool.slash_count += 1`
- `slashing_report.status = 2` (Slashed)
- `slashing_report.resolved_at = now`
- Transfer `slash_amount` from `stake_pool` PDA to a burn address (or community treasury)
- If `slashing_report` has a `reporter_bond`, return bond to reporter (successful report)

**Emits:** `ValidatorSlashed`

### 6.7 `claim_rewards`

**Purpose:** Validator claims accumulated rewards. Rewards accrue proportional to `staked_amount × time × reward_rate`.

**Accounts:**
- `stake_pool` (mut)
- `validator_stake` (mut)
- `validator` (signer, mut — receives rewards)
- `system_program`

**Args:** None

**Guards:**
- `validator` is the owner of `validator_stake`
- `validator_stake.rewards_accrued > 0`
- `validator_stake.staked_amount > 0` (must still be staked)

**Effects:**
- `amount = validator_stake.rewards_accrued`
- `validator_stake.rewards_accrued = 0`
- `stake_pool.accumulated_rewards -= amount`
- Transfer `amount` SOL from `stake_pool` PDA to `validator`

**Emits:** `RewardsClaimed`

### 6.8 `distribute_rewards`

**Purpose:** Distribute accumulated rewards proportionally to all staked validators. Called by the admin or automatically by an off-chain keeper.

**Accounts:**
- `stake_pool` (mut)
- `authority` (signer — admin)
- `system_program`

**Args:** None

**Guards:**
- `authority` is the StakePool admin (AuthorityRegistry admin)
- `Clock::get()?.unix_timestamp >= stake_pool.last_reward_distribution + REWARD_DISTRIBUTION_INTERVAL_SECS` (cooldown elapsed)
- `stake_pool.total_staked > 0`

**Effects:**
- `time_delta = now - stake_pool.last_reward_distribution`
- `annual_reward = stake_pool.total_staked * reward_rate_bps / 10000`
- `period_reward = annual_reward * time_delta / (365 * 24 * 3600)`
- `stake_pool.accumulated_rewards += period_reward`
- `stake_pool.last_reward_distribution = now`
- Each validator's `rewards_accrued += (validator_stake.staked_amount / stake_pool.total_staked) * period_reward`
- Note: Individual reward accrual is computed lazily when each validator calls `claim_rewards`. This instruction only updates the pool-level accumulator.

**Emits:** `RewardsDistributed`

### 6.9 `dispute_slashing`

**Purpose:** Validator disputes a slashing report during the appeal window.

**Accounts:**
- `slashing_report` (mut)
- `offender_stake` (readonly)
- `offender` (signer — the accused validator)
- `system_program`

**Args:** `appeal_reason: String`

**Guards:**
- `offender` is the owner of `offender_stake`
- `slashing_report.status == 0` (Pending)
- `Clock::get()?.unix_timestamp < slashing_report.appeal_deadline` (within appeal window)
- `slashing_report.offender == offender`
- `appeal_reason.len() <= 256`

**Effects:**
- `slashing_report.status = 3` (Appealed)
- `slashing_report.resolved_at = 0` (reset)
- The appeal reason is logged via event; resolution requires admin or validator quorum review

**Emits:** `SlashingAppealed`

### 6.10 `dismiss_report`

**Purpose:** Admin dismisses a slashing report (evidence insufficient or false).

**Accounts:**
- `slashing_report` (mut, close → reporter)
- `stake_pool` (readonly)
- `authority` (signer — admin)
- `system_program`

**Args:** None

**Guards:**
- `authority` is the StakePool admin
- `slashing_report.status == 0` (Pending) or `slashing_report.status == 3` (Appealed)

**Effects:**
- `slashing_report.status = 5` (Dismissed)
- `slashing_report.resolved_at = now`
- Return `reporter_bond` to `slashing_report.reporter`
- Close `slashing_report` account

**Emits:** `ReportDismissed`

## 7. Off-Chain Protocol

### 7.1 Equivocation Detection

1. Off-chain indexing service monitors all `attest` and `authorize_vault_access` transactions.
2. For each parcel, the service maintains a map of `(parcel_id, specifier) → validator_signatures`.
3. If the same validator signs two different attestations for the same parcel+specifier, the service flags an equivocation.
4. The service generates an evidence payload: `[attestation_a_bytes, attestation_b_bytes]` → `evidence_hash = SHA-256(concat)`.
5. A reporter calls `report_equivocation` on-chain with the evidence hash, offender pubkey, and bounded details.

### 7.2 Liveness Monitoring

1. Off-chain heartbeat service pings each validator at regular intervals (e.g., every 6 hours).
2. If a validator fails to respond for `LIVENESS_THRESHOLD_SECS` (7 days), a liveness report is filed.
3. The report follows the same flow as equivocation: `report_equivocation` with `offense_type = 1` (liveness).
4. Validators can prove liveness by calling `ping_stake` (not yet specified — would be a simple "I'm alive" transaction).

### 7.3 Collusion Detection

1. Collusion is detected when multiple validators sign actions that collectively violate protocol rules (e.g., signing conflicting successions for the same identity, or coordinating unauthorized vault access).
2. Detection requires off-chain analysis of validator signature patterns across transactions.
3. The evidence payload includes the set of colluding signers and the fraudulent action hash.
4. The report includes all offender pubkeys; each is slashed independently.

### 7.4 Reward Distribution Cadence

1. An off-chain keeper calls `distribute_rewards` daily (or at whatever cadence the admin configures).
2. The keeper computes the time delta since last distribution and updates `accumulated_rewards`.
3. Individual validators call `claim_rewards` at their convenience to withdraw accrued rewards.

### 7.5 Appeal Resolution

1. When a validator calls `dispute_slashing`, the appeal reason is emitted on-chain.
2. The admin (or a validator quorum) reviews the appeal off-chain (evidence examination, signature verification).
3. If the appeal is upheld, the admin calls `dismiss_report` to return the reporter's bond and cancel the slash.
4. If the appeal is rejected, the admin calls `verify_and_slash` to execute the slash.

## 8. Collusion Resistance

### 8.1 Appointment + Stake (Not Either/Or)

The trust model is **layered**:
1. **AuthorityRegistry appointment** — a real human (admin) vouches for the validator's identity and competence.
2. **Stake bond** — the validator puts capital at risk, creating economic cost for misbehavior.

A wealthy outsider cannot simply out-stake honest validators because they must first be appointed to the AuthorityRegistry. Stake alone does not grant validator status.

### 8.2 No Delegation

Delegation is **explicitly prohibited**. Third parties cannot:
- Delegate SOL to a validator to increase their stake weight
- Receive rewards for delegating
- Influence validator selection or voting through delegated capital

This prevents capital-based influence: a billionaire cannot buy their way into the validator set by delegating to a proxy.

### 8.3 Reporter Bond Requirement

Anyone filing a slashing report must post a bond equal to `REPORTER_BOND_BPS` (1%) of the potential slash amount. This prevents:
- Spam reports designed to grief honest validators
- False equivocation reports to force unnecessary unbonding
- If the report is dismissed, the reporter loses their bond

### 8.4 Graduated Slashing

First offense: 10% of stake slashed. Repeat offense: 100% of stake slashed. This creates:
- A learning curve for honest-but-careless validators
- Immediate economic elimination for persistent adversaries
- Historical record (`slash_history`) that other validators and the admin can inspect

### 8.5 Time-Locked Appeals

The 7-day appeal window ensures:
- The accused validator has time to present evidence of innocence
- The admin has time to review the evidence off-chain
- Rushed slashing (based on incomplete evidence) can be paused

## 9. Liveness Guarantees

### 9.1 Stake as Liveness Incentive

Validators with stake at risk are economically incentivized to stay online. Downtime means:
- No reward accrual during offline period
- Potential liveness slashing after 7 days
- Loss of stake if `slash_history` escalates

### 9.2 Liveness Failure Threshold

A validator is considered in liveness failure if they fail to produce a `ping_stake` transaction for `LIVENESS_THRESHOLD_SECS` (7 days). This is deliberately conservative — legitimate validators may have brief outages due to infrastructure maintenance, network issues, or regional outages.

### 9.3 Recovery from Liveness Slash

After a liveness slash (10%), the validator can:
1. Resume producing `ping_stake` transactions
2. Rebuild their stake via `deposit_stake`
3. Continue participating in the validator set

If they fail again (repeat offense), they face a 100% slash and are effectively removed.

### 9.4 Admin Override for Critical Liveness

If the AuthorityRegistry admin determines that a validator's liveness failure poses a systemic risk (e.g., the validator holds shards for active vaults), the admin can:
1. Remove the validator from the AuthorityRegistry (via `remove_validator_from_registry`)
2. Initiate vault shard rotation to replace the offline validator's shards
3. The staking layer handles the economic penalty separately

## 10. Storage Architecture

### 10.1 On-Chain State

- **StakePool:** One per region, ~128 bytes. Stores pool-level aggregates.
- **ValidatorStake:** One per validator per region, ~128 bytes. Stores individual stake and reward state.
- **SlashingReport:** One per report, ~256 bytes. Stores evidence and status. Closed after resolution.

### 10.2 Off-Chain Evidence

- Equivocation evidence (two conflicting attestation payloads) is stored off-chain (IPFS or local database).
- The on-chain `evidence_hash` is the SHA-256 of the evidence payload, serving as a content-addressed anchor.
- Anyone can verify the evidence by fetching the payload and recomputing the hash.

### 10.3 Reward Accounting

- Rewards are computed lazily: `accumulated_rewards` is updated at the pool level by `distribute_rewards`.
- Individual validator rewards are computed as `staked_amount / total_staked * period_reward`.
- This avoids iterating over all validators in a single transaction.

### 10.4 Account Lifecycle

- `ValidatorStake` accounts are created on first `deposit_stake` and closed on full `withdraw_stake`.
- `SlashingReport` accounts are created on `report_equivocation` and closed on `dismiss_report` or `verify_and_slash`.
- `StakePool` accounts are created once per region and never closed (they hold the pool state).

## 11. Replay / Nonce Hygiene

### 11.1 Evidence Hash Uniqueness

The `evidence_hash` in `SlashingReport` is derived from the actual evidence payload (two conflicting attestations). The same evidence cannot produce two different hashes, so replay of the same evidence is impossible (the PDA seed would collide).

### 11.2 Report Deduplication

Each `SlashingReport` PDA includes the `reporter_key` and `evidence_hash`. Two reporters filing the same evidence get separate reports (different `reporter_key` in the seed), but the admin can dismiss duplicates.

### 11.3 Stake Version Matching

`ValidatorStake` has a monotonic `updated_at` timestamp. When calling `verify_and_slash`, the program checks that the stake account has not been modified since the report was filed (within reason — the 24-hour review period provides a buffer).

### 11.4 Unbonding Nonce

`initiate_unbonding` sets `unbonding_starts_at` exactly once. The `withdraw_stake` guard checks that `unbonding_amount > 0` and the time lock has elapsed. This prevents premature withdrawal.

## 12. Post-Quantum Migration Path

### 12.1 Algorithm ID in StakePool

The `algorithm_id` field (reserved for future use) is a `u8` enum:
- `0` = Ed25519 (current)
- `1` = Dilithium-3 + Ed25519 hybrid (future)

### 12.2 Signature Migration

When a post-quantum signature scheme is standardized:
1. Validators generate Dilithium-3 keypairs alongside their Ed25519 keys.
2. Both keys are registered in the AuthorityRegistry.
3. Staking operations begin requiring dual signatures (Ed25519 + Dilithium-3).
4. The `algorithm_id` in StakePool is bumped to 1.
5. Old Ed25519-only signatures are no longer accepted for staking operations.

### 12.3 Evidence Verification

Post-quantum evidence verification requires:
- Fetching the evidence payload (two conflicting attestations)
- Verifying both Ed25519 and Dilithium-3 signatures
- This is an off-chain concern; the on-chain program only stores the evidence hash

### 12.4 Timeline Considerations

- NIST post-quantum standards are finalized (2024).
- Dilithium-3 is mature and deployed in production systems.
- For the pilot (1-2 years), Ed25519 is sufficient.
- Migration should happen before the system scales beyond the pilot region.

## 13. Operational Security

### 13.1 Validator Key Management

- Validators must use hardware wallets (Ledger, Trezor) or HSMs for signing.
- Hot wallets (software wallets) should only hold minimal operational SOL, not the full stake.
- Key rotation is handled by AuthorityRegistry (remove old key, add new key), not by the staking layer.

### 13.2 Evidence Preservation

- Equivocation evidence must be preserved off-chain for at least 90 days (the dispute window).
- Evidence should be stored on IPFS for content-addressed integrity.
- Validators should archive their own attestation history for defense against false reports.

### 13.3 Admin Key Security

- The AuthorityRegistry admin key controls stake pool creation, reward distribution, and slashing execution.
- This key should be held in a multisig or hardware wallet.
- Admin actions should be logged and monitored.

### 13.4 Incident Response

If a validator detects they have been falsely accused:
1. Call `dispute_slashing` immediately with the appeal reason.
2. Gather evidence of innocence (e.g., proof that the "conflicting" attestations are for different parcels).
3. Present evidence to the admin or validator quorum off-chain.
4. If the appeal is upheld, the admin calls `dismiss_report`.

If the admin detects a genuine attack:
1. Call `verify_and_slash` to execute the slash.
2. Remove the attacker from the AuthorityRegistry.
3. Initiate vault shard rotation if the attacker held shards.
4. Notify affected parties.

## 14. Test Vectors

### 14.1 Stake Pool Creation

- Region AuthorityRegistry: key `Ar1...`
- Admin: wallet A (matches `registry.admin`)
- Reward rate: 500 bps (5% annual)
- Expected: StakePool created with `total_staked = 0`, `reward_rate_bps = 500`, `region_registry = Ar1...`

### 14.2 Validator Stake Deposit

- Stake pool: `Sp1...`
- Validator: wallet B (in `registry.validators`)
- Amount: 5 SOL (5_000_000_000 lamports)
- Expected: ValidatorStake created with `staked_amount = 5_000_000_000`, `unbonding_amount = 0`
- Expected: StakePool `total_staked = 5_000_000_000`

### 14.3 Minimum Stake Enforcement

- Validator: wallet C
- Amount: 0.5 SOL (500_000_000 lamports)
- Expected: `InsufficientStake` error (below `MIN_STAKE_LAMPORTS`)

### 14.4 Non-Validator Stake Rejection

- Validator: wallet D (NOT in `registry.validators`)
- Amount: 5 SOL
- Expected: `NotValidator` error

### 14.5 Unbonding + Withdrawal

- Validator: wallet B, staked 5 SOL
- Call `initiate_unbonding`
- Expected: `unbonding_amount = 5_000_000_000`, `staked_amount = 0`
- Wait 7 days
- Call `withdraw_stake`
- Expected: 5 SOL returned to wallet B, ValidatorStake closed

### 14.6 Premature Withdrawal Rejection

- Validator: wallet B, unbonding for 3 days (not yet 7)
- Call `withdraw_stake`
- Expected: `UnbondingNotComplete` error

### 14.7 Equivocation Report + Slash

- Validator: wallet B, staked 5 SOL, first offense
- Reporter: wallet E, bonds 0.05 SOL (1% of 5 SOL)
- Evidence hash: `SHA256(attestation_a || attestation_b)`
- Expected: SlashingReport created, status = Pending
- Wait 24 hours (review period), no appeal
- Admin calls `verify_and_slash`
- Expected: `slash_amount = 500_000_000` (10% of 5 SOL)
- Expected: wallet B `staked_amount = 4_500_000_000`
- Expected: wallet E gets bond back (0.05 SOL)
- Expected: SlashingReport status = Slashed

### 14.8 Repeat Offense (100% Slash)

- Validator: wallet B, `slash_history = 1`, staked 4.5 SOL
- New equivocation reported
- Admin calls `verify_and_slash`
- Expected: `slash_amount = 4_500_000_000` (100% of 4.5 SOL)
- Expected: wallet B `staked_amount = 0`
- Expected: SlashingReport status = Slashed

### 14.9 Appeal + Dismissal

- Validator: wallet B, staked 5 SOL
- Reporter: wallet E, bonds 0.05 SOL
- Report filed
- Wallet B calls `dispute_slashing` with reason "different parcels"
- Expected: SlashingReport status = Appealed
- Admin reviews, finds evidence valid
- Admin calls `dismiss_report`
- Expected: wallet E gets bond back, SlashingReport closed

### 14.10 Reward Distribution + Claim

- Stake pool: 2 validators, A (3 SOL) and B (7 SOL), total 10 SOL
- Reward rate: 500 bps (5% annual)
- Time elapsed: 365 days (1 year)
- Expected annual reward: 10 SOL * 5% = 0.5 SOL
- Admin calls `distribute_rewards`
- Expected: `accumulated_rewards = 500_000_000` (0.5 SOL)
- Validator A calls `claim_rewards`
- Expected: A receives `3/10 * 0.5 = 0.15 SOL`
- Validator B calls `claim_rewards`
- Expected: B receives `7/10 * 0.5 = 0.35 SOL`

### 14.11 False Report Penalty

- Reporter: wallet F, bonds 0.05 SOL
- Files report against wallet B (evidence is fabricated)
- Admin calls `dismiss_report`
- Expected: wallet F loses 0.05 SOL bond (sent to burn address or community treasury)

### 14.12 Self-Report Prevention

- Validator: wallet B
- Wallet B tries to call `report_equivocation` against themselves
- Expected: `SelfReportNotAllowed` error

### 14.13 Duplicate Evidence Rejection

- Reporter: wallet E, files report with evidence_hash `H1`
- Reporter: wallet E tries to file another report with the same evidence_hash `H1`
- Expected: `DuplicateReport` error (PDA seed collision — same reporter + same evidence_hash)
