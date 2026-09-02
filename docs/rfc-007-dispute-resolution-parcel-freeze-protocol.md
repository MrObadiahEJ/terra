# RFC-007: Dispute Resolution & Parcel Freeze Protocol

## 1. Status

- **Status:** Implemented (Phase 5)
- **Created:** 2026-09-02
- **Supersedes:** None

## 2. Summary

This RFC specifies the **Dispute Resolution & Parcel Freeze Protocol** — a lifecycle-gated dispute mechanism for Terra parcels. When a legal or factual dispute arises over parcel ownership, the parcel must be frozen (preventing trades, transfers, and sales) while the dispute progresses through adjudication. The protocol introduces a five-state lifecycle on a `Dispute` PDA: `FILED → FROZEN → ADJUDICATED → EXECUTED`, with an exit path to `CANCELLED`.

The on-chain program never handles court documents or evidence — it only anchors a SHA-256 hash of the off-chain case file (`case_hash`) and records state transitions gated by the existing AuthorityRegistry validator quorum. Filing requires 2+ validator co-signatures (anti-grief), freezing requires the same 2+ validators, adjudication requires a court authority signer, and execution finalizes the outcome (owner wins → parcel returns to REGISTERED; owner loses → parcel forfeited to `new_owner`). Disputes auto-cancel after 90 days if not adjudicated.

## 3. Threat Model

### 3.1 Adversary Classes

| Class | Description | Mitigation |
|-------|-------------|------------|
| **Griefing filer** | A single party files a frivolous dispute to freeze a parcel | Minimum 2 validator co-signatures required to file |
| **Colluding validators** | k validators freeze a parcel without legitimate cause | Same 2+ validators required for filing and freezing; public audit trail via events |
| **Parcel owner (self-dealing)** | Owner files dispute against themselves, or installs themselves as validator | Filer cannot be their own validator; parcel owner cannot be a dispute validator |
| **Stale dispute** | A filed dispute is never adjudicated, leaving the parcel frozen indefinitely | 90-day expiry window; disputes auto-cancel if not adjudicated in time |
| **Unauthorized execution** | Someone executes a judgment on a dispute that hasn't been adjudicated | Strict state machine: EXECUTED only reachable from ADJUDICATED |
| **Unauthorized cancellation** | A third party cancels a legitimate dispute | Only the filer or parcel owner can cancel before adjudication |

### 3.2 In Scope

- Parcel freeze during active disputes (prevents FOR_SALE, TRANSFERRED status transitions)
- Validator-gated dispute filing and freezing (anti-grief co-signatures)
- Court authority adjudication (OWNER_WINS or OWNER_LOSES outcomes)
- Judgment execution (parcel returns to REGISTERED or forfeits to new owner)
- Dispute lifecycle management (filing, freezing, adjudication, execution, cancellation)
- Auto-expiry of stale disputes (90-day window)

### 3.3 Out of Scope

- Vault shard protocol and encrypted data protection (covered by RFC-003)
- Attestation and succession flows (covered by RFC-001/002)
- The actual legal process or court document validation (application-layer concern)
- The specific criteria for what constitutes a valid dispute (validator judgment)

## 4. Cryptographic Choices

### 4.1 Hashing: SHA-256

- `case_hash` is a SHA-256 digest over the off-chain court document or complaint
- Anchors evidence immutably on-chain without storing the document itself
- Anyone can verify the hash matches the original document

### 4.2 Signatures: Ed25519

- Solana native — all validator and authority keys are Ed25519
- Used for transaction signing (on-chain authorization)
- Used for off-chain protocol messages (validator coordination)

### 4.3 PDA Derivation: SHA-256

- Dispute PDA: `["dispute", parcel_key, case_hash]`
- Deterministic, collision-resistant — one dispute per parcel per case_hash
- Parcel PDA: `["parcel", parcel_id]` (existing)

### 4.4 No Additional Cryptographic Primitives

The dispute protocol does not require encryption, zero-knowledge proofs, or threshold cryptography. All security is derived from Ed25519 signatures and the AuthorityRegistry validator quorum.

## 5. Data Model

### 5.1 Dispute (on-chain PDA)

**PDA seed:** `["dispute", parcel, case_hash]`

| Field | Type | Description |
|-------|------|-------------|
| `parcel` | `Pubkey` | The parcel under dispute |
| `filed_by` | `Pubkey` | The wallet that filed the dispute |
| `case_hash` | `[u8; 32]` | SHA-256 of the off-chain court document / complaint |
| `status` | `u8` | Current dispute status (FILED=0, FROZEN=1, ADJUDICATED=2, EXECUTED=3, CANCELLED=4) |
| `required` | `u8` | Required validator co-signatures to advance this dispute |
| `count` | `u8` | Number of validators who have co-signed so far |
| `validators` | `[Pubkey; MAX_VALIDATORS]` | Declared validator set for this dispute (max 8) |
| `filed_at` | `i64` | Unix timestamp when dispute was filed |
| `frozen_at` | `i64` | Unix timestamp when parcel was frozen (0 if not yet frozen) |
| `adjudicated_at` | `i64` | Unix timestamp when dispute was adjudicated (0 if not yet adjudicated) |
| `outcome` | `u8` | Outcome of adjudication: OWNER_WINS=0 or OWNER_LOSES=1 |
| `new_owner` | `Pubkey` | New owner if outcome is OWNER_LOSES (Pubkey::default otherwise) |

### 5.2 Dispute Status Enum

| Value | Name | Description |
|-------|------|-------------|
| 0 | `FILED` | Dispute filed, parcel moved to DISPUTED status |
| 1 | `FROZEN` | Parcel frozen, no trading allowed |
| 2 | `ADJUDICATED` | Court authority has ruled |
| 3 | `EXECUTED` | Judgment executed, parcel returned or forfeited |
| 4 | `CANCELLED` | Dispute cancelled before adjudication |

### 5.3 Dispute Outcome Enum

| Value | Name | Description |
|-------|------|-------------|
| 0 | `OWNER_WINS` | Owner retains parcel, returned to REGISTERED |
| 1 | `OWNER_LOSES` | Owner loses, parcel forfeited to `new_owner` |

### 5.4 Parcel Status Transitions (Dispute-Related)

```
REGISTERED → DISPUTED  (file_dispute)
DISPUTED → FROZEN      (freeze_parcel)
FROZEN → ADJUDICATED   (adjudicate_dispute)
ADJUDICATED → REGISTERED  (execute_judgment, OWNER_WINS)
ADJUDICATED → FORFEITED   (execute_judgment, OWNER_LOSES)
DISPUTED → REGISTERED  (cancel_dispute, before freeze)
```

### 5.5 Events

| Event | Fields | Description |
|-------|--------|-------------|
| `DisputeFiled` | dispute, parcel, filed_by, case_hash, required, count | Emitted when a dispute is filed |
| `ParcelFrozen` | dispute, parcel, frozen_at | Emitted when a parcel is frozen |
| `DisputeAdjudicated` | dispute, parcel, outcome, new_owner, adjudicated_at | Emitted when a dispute is adjudicated |
| `JudgmentExecuted` | dispute, parcel, outcome, new_owner, executed_at | Emitted when a judgment is executed |
| `DisputeCancelled` | dispute, parcel, cancelled_by | Emitted when a dispute is cancelled |

## 6. Instructions

### 6.1 `file_dispute`

**Purpose:** File a dispute against a parcel. The parcel moves from REGISTERED to DISPUTED. Requires 2+ validator co-signatures to prevent griefing.

**Accounts:**

| Account | Type | Mutable | Description |
|---------|------|---------|-------------|
| `dispute` | init, PDA `["dispute", parcel, case_hash]` | Yes | The dispute record (created) |
| `parcel` | Account, PDA `["parcel", id]` | Yes | The parcel under dispute |
| `filer` | Signer | Yes | The wallet filing the dispute (pays rent) |
| `system_program` | Program | No | System program |

**Args:**

| Arg | Type | Description |
|-----|------|-------------|
| `case_hash` | `[u8; 32]` | SHA-256 of the off-chain court document / complaint |
| `required` | `u8` | Required validator co-signatures (must be >= 2) |
| `validators` | `[Pubkey; MAX_VALIDATORS]` | Declared validator set (padded with Pubkey::default) |

**Guards:**

- `case_hash` is not all zeros (`EmptyCaseHash`)
- `required >= MIN_DISPUTE_VALIDATORS` (2) (`InvalidThreshold`)
- `required <= count` of non-default validators in the set (`InvalidThreshold`)
- No validator in the set is the filer (`ValidatorOwnsAsset` — self-dealing)
- No validator in the set is the parcel owner (`ValidatorOwnsAsset` — self-dealing)
- At least one non-default validator exists (`NoValidators`)
- Parcel status is `REGISTERED` (`InvalidStatus`)

**Effects:**

- `dispute.parcel = parcel.key()`
- `dispute.filed_by = filer.key()`
- `dispute.case_hash = case_hash`
- `dispute.status = FILED (0)`
- `dispute.required = required`
- `dispute.count = count` (number of non-default validators)
- `dispute.validators = validators`
- `dispute.filed_at = now`
- `parcel.status = DISPUTED (4)`
- `parcel.updated_at = now`

**Emits:** `DisputeFiled`

### 6.2 `freeze_parcel`

**Purpose:** Freeze a disputed parcel. The parcel moves from DISPUTED to FROZEN. No trades, transfers, or sales are allowed while frozen. Requires the same 2+ validators who filed the dispute.

**Accounts:**

| Account | Type | Mutable | Description |
|---------|------|---------|-------------|
| `dispute` | Account, PDA `["dispute", parcel, case_hash]` | Yes | The dispute record |
| `parcel` | Account, PDA `["parcel", id]` | Yes | The parcel to freeze |
| `validator` | Signer | No | A validator signing the freeze |

**Args:** None

**Guards:**

- `dispute.status == FILED` (`InvalidDisputeStatus`)
- `now < dispute.filed_at + DISPUTE_EXPIRY_SECS` (90 days) (`DisputeExpired`)
- `dispute.count >= dispute.required` (validator quorum met) (`InsufficientValidatorSigners`)
- `parcel.status == DISPUTED` (`InvalidStatus`)

**Effects:**

- `dispute.status = FROZEN (1)`
- `dispute.frozen_at = now`
- `parcel.status = FROZEN (5)`
- `parcel.updated_at = now`

**Emits:** `ParcelFrozen`

### 6.3 `adjudicate_dispute`

**Purpose:** Court authority rules on the dispute. The parcel moves from FROZEN to ADJUDICATED. The outcome is recorded (OWNER_WINS or OWNER_LOSES).

**Accounts:**

| Account | Type | Mutable | Description |
|---------|------|---------|-------------|
| `dispute` | Account, PDA `["dispute", parcel, case_hash]` | Yes | The dispute record |
| `parcel` | Account, PDA `["parcel", id]` | Yes | The parcel under dispute |
| `authority` | Signer | No | Court authority or designated adjudicator |

**Args:**

| Arg | Type | Description |
|-----|------|-------------|
| `outcome` | `u8` | OWNER_WINS (0) or OWNER_LOSES (1) |
| `new_owner` | `Pubkey` | New owner if OWNER_LOSES (Pubkey::default if OWNER_WINS) |

**Guards:**

- `outcome <= OWNER_LOSES` (1) (`InvalidDisputeOutcome`)
- `dispute.status == FROZEN` (`InvalidDisputeStatus`)
- `now < dispute.filed_at + DISPUTE_EXPIRY_SECS` (90 days) (`DisputeExpired`)
- `dispute.count >= dispute.required` (validator quorum met) (`InsufficientValidatorSigners`)
- If `outcome == OWNER_LOSES`: `new_owner != Pubkey::default` (`EmptyNewOwner`)

**Effects:**

- `dispute.status = ADJUDICATED (2)`
- `dispute.adjudicated_at = now`
- `dispute.outcome = outcome`
- `dispute.new_owner = new_owner`
- `parcel.status = ADJUDICATED (6)`
- `parcel.updated_at = now`

**Emits:** `DisputeAdjudicated`

### 6.4 `execute_judgment`

**Purpose:** Finalize the adjudicated dispute. If OWNER_WINS, parcel returns to REGISTERED. If OWNER_LOSES, parcel is forfeited to `new_owner`.

**Accounts:**

| Account | Type | Mutable | Description |
|---------|------|---------|-------------|
| `dispute` | Account, PDA `["dispute", parcel, case_hash]` | Yes | The dispute record |
| `parcel` | Account, PDA `["parcel", id]` | Yes | The parcel under dispute |
| `authority` | Signer | No | Authority executing the judgment |

**Args:** None

**Guards:**

- `dispute.status == ADJUDICATED` (`InvalidDisputeStatus`)
- If `dispute.outcome == OWNER_LOSES`: `dispute.new_owner != Pubkey::default` (`EmptyNewOwner`)

**Effects:**

- If `OWNER_WINS`: `parcel.status = REGISTERED (1)`
- If `OWNER_LOSES`: `parcel.owner = dispute.new_owner`, `parcel.status = FORFEITED (7)`
- `parcel.updated_at = now`
- `dispute.status = EXECUTED (3)`

**Emits:** `JudgmentExecuted`

### 6.5 `cancel_dispute`

**Purpose:** Cancel a filed dispute before adjudication. Only the filer or parcel owner can cancel. If the parcel is in DISPUTED status (not yet frozen), it returns to REGISTERED.

**Accounts:**

| Account | Type | Mutable | Description |
|---------|------|---------|-------------|
| `dispute` | Account, PDA `["dispute", parcel, case_hash]` | Yes | The dispute record |
| `parcel` | Account, PDA `["parcel", id]` | Yes | The parcel under dispute |
| `signer` | Signer | No | The filer or parcel owner |

**Args:** None

**Guards:**

- `dispute.status == FILED` (`InvalidDisputeStatus`)
- `signer` is either `dispute.filed_by` or `parcel.owner` (`NotAuthorized`)

**Effects:**

- If `parcel.status == DISPUTED`: `parcel.status = REGISTERED (1)`
- `parcel.updated_at = now`
- `dispute.status = CANCELLED (4)`

**Emits:** `DisputeCancelled`

## 7. Off-Chain Protocol

### 7.1 Dispute Filing Ceremony

1. A filer (plaintiff, regulatory authority, or concerned party) gathers evidence of a dispute.
2. The filer computes `case_hash = SHA-256(case_document)` over the court document or complaint.
3. The filer coordinates with 2+ validators via a secure side-channel (Signal group, in-person meeting).
4. Each validator confirms their identity and willingness to co-sign the dispute.
5. The filer calls `file_dispute` on-chain with the `case_hash`, `required` threshold, and `validators` list.
6. The parcel moves to DISPUTED status. No trades, transfers, or sales are possible.
7. The filer and validators separately agree on a timeline for adjudication.

### 7.2 Freeze Ceremony

1. After filing, the filer or any validator calls `freeze_parcel` on-chain.
2. The dispute status moves from FILED to FROZEN. The parcel status moves from DISPUTED to FROZEN.
3. Freezing requires the same validator quorum that was declared at filing (2+ validators).
4. Freezing must occur within 90 days of filing (auto-expiry window).

### 7.3 Adjudication Ceremony

1. A court authority or designated adjudicator reviews the evidence off-chain.
2. The authority calls `adjudicate_dispute` on-chain with the outcome (OWNER_WINS or OWNER_LOSES) and, if applicable, the `new_owner`.
3. The dispute moves to ADJUDICATED status. The parcel moves to ADJUDICATED status.
4. Adjudication must occur within 90 days of filing (auto-expiry window).

### 7.4 Judgment Execution

1. After adjudication, anyone calls `execute_judgment` on-chain.
2. If OWNER_WINS: the parcel returns to REGISTERED status. The owner retains full rights.
3. If OWNER_LOSES: the parcel owner is changed to `new_owner`, and the status moves to FORFEITED.
4. The dispute moves to EXECUTED status. The lifecycle is complete.

### 7.5 Dispute Cancellation

1. If the dispute is resolved off-settlement or withdrawn, the filer or parcel owner calls `cancel_dispute`.
2. If the parcel is still in DISPUTED status (not yet frozen), it returns to REGISTERED.
3. If the parcel was already frozen, cancellation does not unfreeze it (requires separate adjudication).
4. The dispute moves to CANCELLED status.

### 7.6 Side-Channel Requirements

- **Secure messaging:** Each dispute has a dedicated Signal group with the filer, validators, and court authority. Messages are end-to-end encrypted.
- **Evidence vault:** Court documents are stored on IPFS or Arweave, referenced by `case_hash` on-chain.
- **Timeline coordination:** Validators and the court authority agree on adjudication deadlines off-chain.

### 7.7 Auto-Expiry Protocol

1. After 90 days from `filed_at`, the dispute expires.
2. Any on-chain call to `freeze_parcel` or `adjudicate_dispute` will fail with `DisputeExpired`.
3. The filer or parcel owner can call `cancel_dispute` to clean up the expired dispute.
4. The parcel remains in DISPUTED status until explicitly cancelled or adjudicated.

## 8. Collusion Resistance

### 8.1 Anti-Grief Co-Signatures

- **Dispute filing:** Requires 2+ validator co-signatures (not just 1)
- **Parcel freezing:** Requires the same 2+ validators who declared the dispute
- **Adjudication:** Requires a court authority signer + validator quorum

This prevents a single griefing party from freezing a parcel unilaterally.

### 8.2 Self-Dealing Prevention

- The filer cannot be their own validator (`ValidatorOwnsAsset`)
- The parcel owner cannot be a dispute validator (`ValidatorOwnsAsset`)
- This prevents a parcel owner from filing a dispute against themselves and controlling the outcome

### 8.3 Public Audit Trail

Every dispute lifecycle event is emitted on-chain with:
- The dispute PDA and parcel PDA
- The filer, validators, and court authority
- Timestamps for each state transition

This creates a public, immutable audit trail. Any observer can detect:
- Unusual frequency of dispute filings
- The same group of validators always co-signing disputes
- Disputes that are filed but never adjudicated

### 8.4 Time-Limited Disputes

Disputes auto-expire after 90 days. This prevents:
- Permanent freezing of parcels through abandoned disputes
- Validators holding parcels hostage by refusing to adjudicate
- Stale disputes cluttering the on-chain state

## 9. Liveness Guarantees

### 9.1 Dispute Expiry Window

- Disputes auto-expire after 90 days (`DISPUTE_EXPIRY_SECS = 90 * 24 * 3600`)
- If not adjudicated within this window, the dispute cannot be frozen or adjudicated
- The filer or parcel owner must cancel the expired dispute to clean up

### 9.2 Cancellation Path

- If a dispute is abandoned, the filer or parcel owner can cancel it at any time (before adjudication)
- Cancellation restores the parcel to REGISTERED status (if not yet frozen)
- This ensures parcels are not permanently locked by stale disputes

### 9.3 No Forced Adjudication

- The court authority is not forced to adjudicate within a specific timeframe
- The 90-day window is a maximum, not a minimum
- If the court needs more time, the dispute must be re-filed after cancellation

### 9.4 Offline Validator Handling

- Validators who are temporarily offline cannot block the freeze or adjudication
- The quorum is based on the declared validator set, not the current online set
- If a validator is unavailable, the dispute can still proceed if the quorum is met

## 10. Storage Architecture

### 10.1 On-Chain State

- **Dispute PDA:** Contains the dispute lifecycle state, validator set, and outcome
- **Parcel PDA:** Updated with status transitions (DISPUTED, FROZEN, ADJUDICATED, FORFEITED)
- **Events:** Emitted for every state transition (DisputeFiled, ParcelFrozen, DisputeAdjudicated, JudgmentExecuted, DisputeCancelled)

### 10.2 Off-Chain Evidence

- **Court documents:** Stored on IPFS or Arweave, referenced by `case_hash` on-chain
- **Evidence vault:** Encrypted if sensitive, accessible via the Vault Shard Protocol (RFC-003)
- **Audit logs:** Off-chain mirrors of on-chain events for compliance and reporting

### 10.3 PostGIS (API Layer)

- **Disputes table:** Mirrors on-chain dispute state for fast queries
- **Fields:** id, parcel_id, filed_by, case_hash, status, required, count, validators, filed_at, frozen_at, adjudicated_at, outcome, new_owner, created_at, updated_at
- **Not the source of truth** — only a performance optimization for the API

### 10.4 CID Verification Protocol

1. Fetch court document from any IPFS gateway using the CID.
2. Compute SHA-256 of the fetched bytes.
3. Compare against `dispute.case_hash` (on-chain).
4. If they match, the document is authentic and untampered.
5. Anyone can perform this verification — no trust in the API required.

## 11. Replay / Nonce Hygiene

### 11.1 PDA Uniqueness

The dispute PDA is derived from `["dispute", parcel, case_hash]`. This ensures:
- One dispute per parcel per case_hash (no duplicate disputes for the same case)
- Different cases against the same parcel get different PDAs
- The same case cannot be filed twice against the same parcel

### 11.2 State Machine Enforcement

The program enforces strict state transitions:
- `file_dispute` requires `parcel.status == REGISTERED`
- `freeze_parcel` requires `dispute.status == FILED` and `parcel.status == DISPUTED`
- `adjudicate_dispute` requires `dispute.status == FROZEN`
- `execute_judgment` requires `dispute.status == ADJUDICATED`
- `cancel_dispute` requires `dispute.status == FILED`

This prevents replay of state transitions or execution of judgments on non-adjudicated disputes.

### 11.3 Timestamp Enforcement

- `freeze_parcel` checks `now < dispute.filed_at + DISPUTE_EXPIRY_SECS`
- `adjudicate_dispute` checks `now < dispute.filed_at + DISPUTE_EXPIRY_SECS`
- This prevents stale disputes from being frozen or adjudicated after the 90-day window

### 11.4 No Nonce Field

Unlike the Vault Shard Protocol, the Dispute Protocol does not use a nonce field. The PDA derivation (`case_hash`) serves as a unique identifier for each dispute. The state machine and timestamp checks provide sufficient replay protection.

## 12. Post-Quantum Migration Path

### 12.1 Current State

The Dispute Protocol does not use any cryptographic primitives that are vulnerable to quantum attacks:
- Ed25519 signatures are used for transaction signing
- SHA-256 is used for hashing (case_hash)
- No encryption or zero-knowledge proofs are used on-chain

### 12.2 Future Considerations

If post-quantum signature schemes become necessary:
- The AuthorityRegistry validator keys would be migrated to a post-quantum scheme
- The dispute protocol would automatically benefit from this migration (no on-chain changes required)
- The `case_hash` (SHA-256) would remain secure (quantum resistance is not required for hashing)

### 12.3 Timeline Considerations

- NIST post-quantum standards are finalized (2024)
- Ed25519 remains secure for the foreseeable future (quantum threat is theoretical)
- Migration to post-quantum signatures would happen at the AuthorityRegistry level, not the dispute protocol level

## 13. Operational Security

### 13.1 Validator Key Management

- Validators must secure their Ed25519 private keys (hardware wallets recommended)
- Key rotation is handled at the AuthorityRegistry level
- Compromised keys can be revoked by the registry admin

### 13.2 Court Authority Key Management

- The court authority signer must be a trusted, identified entity
- Key compromise requires a governance process to rotate the authority
- The authority signer is not a validator (separation of concerns)

### 13.3 Evidence Integrity

- Court documents must be hashed (SHA-256) before filing
- The hash is anchored on-chain; the document is stored on IPFS/Arweave
- Tampering with the document after filing is detectable via hash comparison

### 13.4 Dispute Lifecycle Monitoring

- Off-chain services should monitor dispute states and timestamps
- Alerts for disputes approaching the 90-day expiry window
- Alerts for parcels in DISPUTED or FROZEN status for extended periods

### 13.5 Emergency Procedures

- If the court authority key is compromised, the registry admin can rotate it
- If a dispute is frozen and the court authority is unavailable, the dispute expires after 90 days
- If a parcel is frozen and the dispute is cancelled, the parcel returns to REGISTERED

## 14. Test Vectors

### 14.1 Dispute Filing

- Parcel: registered, owner = wallet A
- Filer: wallet B
- case_hash: SHA-256("complaint_001") = [0xa3, 0xf2, ...]
- required: 2
- validators: [wallet C, wallet D, Pubkey::default, ...]
- Expected: Dispute created with status FILED, parcel status = DISPUTED

### 14.2 Dispute Filing — Self-Dealing Rejection

- Parcel: registered, owner = wallet A
- Filer: wallet A (same as owner)
- required: 2
- validators: [wallet B, wallet C, ...]
- Expected: `ValidatorOwnsAsset` error (filer cannot be their own validator)

### 14.3 Dispute Filing — Owner as Validator Rejection

- Parcel: registered, owner = wallet A
- Filer: wallet B
- required: 2
- validators: [wallet A, wallet C, ...] (wallet A is the parcel owner)
- Expected: `ValidatorOwnsAsset` error (parcel owner cannot be a dispute validator)

### 14.4 Parcel Freeze

- Dispute: FILED, filed_at = now - 1 day
- Parcel: DISPUTED
- Validator: wallet C (in declared validator set)
- dispute.count >= dispute.required
- Expected: Dispute status = FROZEN, parcel status = FROZEN

### 14.5 Parcel Freeze — Expired Dispute Rejection

- Dispute: FILED, filed_at = now - 91 days (> 90 days)
- Parcel: DISPUTED
- Expected: `DisputeExpired` error

### 14.6 Dispute Adjudication — Owner Wins

- Dispute: FROZEN, filed_at = now - 30 days
- Parcel: FROZEN
- Authority: court wallet
- outcome: OWNER_WINS (0)
- new_owner: Pubkey::default
- Expected: Dispute status = ADJUDICATED, parcel status = ADJUDICATED, outcome = OWNER_WINS

### 14.7 Dispute Adjudication — Owner Loses

- Dispute: FROZEN, filed_at = now - 30 days
- Parcel: FROZEN
- Authority: court wallet
- outcome: OWNER_LOSES (1)
- new_owner: wallet E
- Expected: Dispute status = ADJUDICATED, parcel status = ADJUDICATED, outcome = OWNER_LOSES, new_owner = wallet E

### 14.8 Judgment Execution — Owner Wins

- Dispute: ADJUDICATED, outcome = OWNER_WINS
- Parcel: ADJUDICATED
- Expected: Parcel status = REGISTERED, dispute status = EXECUTED

### 14.9 Judgment Execution — Owner Loses

- Dispute: ADJUDICATED, outcome = OWNER_LOSES, new_owner = wallet E
- Parcel: ADJUDICATED, owner = wallet A
- Expected: Parcel owner = wallet E, parcel status = FORFEITED, dispute status = EXECUTED

### 14.10 Dispute Cancellation

- Dispute: FILED, filed_by = wallet B
- Parcel: DISPUTED
- Signer: wallet B (filer)
- Expected: Dispute status = CANCELLED, parcel status = REGISTERED

### 14.11 Dispute Cancellation — Unauthorized Signer Rejection

- Dispute: FILED, filed_by = wallet B
- Parcel: DISPUTED, owner = wallet A
- Signer: wallet C (neither filer nor owner)
- Expected: `NotAuthorized` error

### 14.12 Insufficient Validators Rejection

- Dispute filing: required = 2, but only 1 non-default validator in set
- Expected: `InvalidThreshold` error

### 14.13 Duplicate Case Hash Rejection

- Dispute 1: filed against parcel with case_hash = SHA-256("complaint_001")
- Dispute 2: filing against same parcel with same case_hash
- Expected: PDA collision — the second `file_dispute` will fail because the PDA already exists

### 14.14 Status Transition Violation

- Attempt to `freeze_parcel` on a dispute with status FILED but parcel status REGISTERED (parcel never moved to DISPUTED)
- Expected: `InvalidStatus` error

### 14.15 Execute Judgment on Non-Adjudicated Dispute

- Attempt to `execute_judgment` on a dispute with status FROZEN (not yet adjudicated)
- Expected: `InvalidDisputeStatus` error
