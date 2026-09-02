# RFC-009: Time-Bound Credential Protocol

## 1. Status

- **Status:** Draft
- **Created:** 2026-09-03
- **Supersedes:** None

## 2. Summary

This RFC specifies the **Time-Bound Credential Protocol** — an expiry enforcement layer for Terra Rights. `Rights.expires_at` already exists in the data model but nothing enforces it. Not all land rights are permanent: leases expire, construction permits are time-limited, easements can be temporary. This protocol closes that gap by adding a `status` field to Rights, a `grace_period_secs` field for grace periods, and three new instructions that together provide lazy evaluation for on-chain enforcement, off-chain keeper sweeps for proactive cleanup, and renewal ceremonies for extending expiring rights. The program never runs a background process — "automated expiry" means any instruction touching a Rights account checks `expires_at` against `Clock::get()` inline (lazy evaluation), and a separate keeper instruction allows an off-chain cron to sweep stale rights into an expired state.

## 3. Threat Model

### 3.1 Adversary Classes

| Class | Description | Mitigation |
|-------|-------------|------------|
| **Negligent holder** | Holder lets a right expire without noticing | Lazy evaluation flags expiry on next touch; keeper sweeps periodically; API surfaces upcoming expirations |
| **Negligent granter** | Landlord forgets to renew a tenant's lease | Renewal ceremony requires granter co-sign; grace period provides vacate window |
| **Malicious holder** | Holder attempts to use an expired right to transfer or subdivide | On-chain guard rejects any write instruction on expired rights; status must be ACTIVE |
| **Malicious granter** | Landlord revokes a right that is still valid | Revocation requires owner OR original granter; holder retains rights until expiry or grace period ends |
| **Keeper downtime** | Off-chain sweeper goes offline and misses expiry windows | Lazy evaluation is the primary enforcement; keeper is a convenience layer only |
| **Clock manipulation** | Adversary manipulates `Clock::get()` to fake expiry | `Clock::get()` is a Solana runtime primitive; validators must agree on block time; manipulation requires consensus-level corruption |
| **Renewal griefing** | Holder requests renewal but granter refuses to co-sign | Renewal is a voluntary ceremony; holder retains current right until it expires |
| **Front-running renewal** | Adversary front-runs a renewal transaction to claim the right first | Renewal requires both holder + granter signatures; no third party can intercept |

### 3.2 In Scope

- Enforcement of `Rights.expires_at` via lazy evaluation and keeper sweeps
- Right status lifecycle: ACTIVE → EXPIRING → EXPIRED → GRACE → REVOKED (or RENEWED)
- Grace period semantics for leases and temporary occupancy
- Renewal ceremonies requiring holder + granter co-sign
- Conditional rights with additional time-bound predicates (e.g., construction must start within N days)
- On-chain event emission for all status transitions

### 3.3 Out of Scope

- Rent/fee collection for time-bound rights (covered by future RFC)
- Dispute resolution for contested renewals (covered by RFC-007)
- Judicial override of expiry (covered by RFC-007 adjudication)
- Token-gated access or NFT representation of rights (separate concern)

## 4. Cryptographic Choices

### 4.1 Signatures: Ed25519

- Solana native — all authority keys are Ed25519
- Used for transaction signing (holder + granter co-sign for renewal)
- Used for keeper instruction signing (authorized sweeper wallet)

### 4.2 Hashing: SHA-256

- On-chain status transitions are logged as events, not content-addressed
- No new cryptographic primitives are introduced by this RFC
- Existing `parcel.parcel_hash` and `rights.notes` fields are unchanged

### 4.3 Clock Source: Solana Runtime

- `Clock::get()?.unix_timestamp` is the authoritative time source
- Validators agree on block time via consensus; manipulation requires consensus-level corruption
- All expiry checks use the same clock source — no drift between on-chain and off-chain evaluation

### 4.4 Post-Quantum Considerations

- This RFC adds no new key material or encryption
- Ed25519 signatures remain secure against known quantum attacks for the foreseeable horizon
- Status transitions are logged events, not encrypted — no post-quantum migration needed for this layer

## 5. Data Model

### 5.1 Rights (existing account — modifications)

**PDA seed:** `["rights", parcel, nonce]`

| Field | Type | Before | After |
|-------|------|--------|-------|
| `parcel` | `Pubkey` | unchanged | unchanged |
| `rights_kind` | `u8` | unchanged | unchanged |
| `holder` | `Pubkey` | unchanged | unchanged |
| `granter` | `Pubkey` | unchanged | unchanged |
| `created_at` | `i64` | unchanged | unchanged |
| `expires_at` | `i64` | exists, unenforced | enforced via lazy evaluation |
| `notes` | `String` | unchanged | unchanged |
| `status` | `u8` | — | **NEW:** ACTIVE=0, EXPIRING=1, EXPIRED=2, GRACE=3, RENEWED=4, REVOKED=5 |
| `grace_period_secs` | `i64` | — | **NEW:** Grace period in seconds after expiry (0 = no grace) |

**Space increase:** 8 bytes (1 byte `status` + 8 bytes `grace_period_secs` = 9 bytes, padded to 16 for alignment).

### 5.2 right_status module

```rust
pub mod right_status {
    pub const ACTIVE: u8 = 0;
    pub const EXPIRING: u8 = 1;   // within warning window
    pub const EXPIRED: u8 = 2;    // past expires_at, no grace
    pub const GRACE: u8 = 3;      // past expires_at, within grace period
    pub const RENEWED: u8 = 4;    // replaced by a new right via renewal
    pub const REVOKED: u8 = 5;    // explicitly revoked before expiry
    pub const MAX: u8 = REVOKED;
}
```

### 5.3 RightStatusTransition (emitted as event)

| Field | Type | Description |
|-------|------|-------------|
| `parcel` | `Pubkey` | Parcel key |
| `rights` | `Pubkey` | Rights PDA key |
| `holder` | `Pubkey` | Party holding the right |
| `old_status` | `u8` | Status before transition |
| `new_status` | `u8` | Status after transition |
| `expires_at` | `i64` | The original expiry timestamp |
| `block_time` | `i64` | Solana block time |

### 5.4 RightRenewed (emitted as event)

| Field | Type | Description |
|-------|------|-------------|
| `parcel` | `Pubkey` | Parcel key |
| `old_rights` | `Pubkey` | Original Rights PDA |
| `new_rights` | `Pubkey` | New Rights PDA (if a new account is created) |
| `holder` | `Pubkey` | Party holding the right |
| `granter` | `Pubkey` | Party who granted the right |
| `old_expires_at` | `i64` | Previous expiry |
| `new_expires_at` | `i64` | Extended expiry |
| `block_time` | `i64` | Solana block time |

## 6. Instructions

### 6.1 `renew_right`

**Purpose:** Extend an expiring or expired right by updating `expires_at`. Requires both holder and granter co-sign.

**Accounts:**
- `rights` (mut — the Rights PDA)
- `parcel` (readonly — parent parcel, for authority checks)
- `holder` (signer — the party holding the right)
- `granter` (signer — the party who granted the right, must be parcel owner or original granter)
- `system_program`

**Args:** `new_expires_at: i64`, `new_notes: String`

**Guards:**
- `rights.holder == holder.key()` — only the current holder can request renewal
- `rights.granter == granter.key()` or `parcel.owner == granter.key()` — granter must be the original granter or current parcel owner
- `rights.status != right_status::REVOKED` — cannot renew a revoked right
- `rights.status != right_status::RENEWED` — cannot renew an already-renewed right
- `new_expires_at > Clock::get()?.unix_timestamp` — new expiry must be in the future
- `new_expires_at > rights.expires_at` — renewal must extend, not shorten
- `new_notes.len() <= 128`

**Effects:**
- `rights.expires_at = new_expires_at`
- `rights.status = right_status::ACTIVE` — reset to active
- `rights.notes = new_notes` (if non-empty, otherwise unchanged)

**Emits:** `RightRenewed`

### 6.2 `sweep_expired_rights`

**Purpose:** Keeper instruction. Marks expired rights as EXPIRED or GRACE based on current time and grace period. Callable by an authorized keeper wallet or any transaction that touches the right (lazy evaluation).

**Accounts:**
- `rights` (mut — the Rights PDA)
- `parcel` (readonly — parent parcel, for authority checks)
- `keeper` (signer — authorized sweeper wallet, or the holder/granter)
- `system_program`

**Args:** None

**Guards:**
- `rights.status == right_status::ACTIVE || rights.status == right_status::EXPIRING` — only sweep active or expiring rights
- `rights.expires_at != 0` — permanent rights (expires_at == 0) are never swept
- `keeper` is either:
  - The authorized keeper wallet (stored in a program-level config PDA), OR
  - `rights.holder`, OR
  - `rights.granter`, OR
  - `parcel.owner`

**Effects (evaluated in order):**
1. If `rights.expires_at == 0` → no-op (permanent right), return early
2. If `now < rights.expires_at` → no-op (not yet expired), return early
3. If `rights.grace_period_secs > 0 && now < rights.expires_at + rights.grace_period_secs`:
   - `rights.status = right_status::GRACE`
4. Else:
   - `rights.status = right_status::EXPIRED`

**Emits:** `RightStatusTransition`

### 6.3 `grant_conditional_right`

**Purpose:** Grant a right with an additional time-bound condition. The right is ACTIVE but carries a condition deadline. If the condition is not met by the deadline, the right can be swept.

**Accounts:**
- `rights` (init, PDA `["rights", parcel, nonce]`)
- `parcel` (mut — parent parcel)
- `owner` (signer, mut — parcel owner, pays rent)
- `system_program`

**Args:** `nonce: u8`, `rights_kind: u8`, `holder: Pubkey`, `expires_at: i64`, `condition_deadline: i64`, `condition_desc: String`, `grace_period_secs: i64`, `notes: String`

**Guards:**
- `owner.key() == parcel.owner` — only parcel owner can grant
- `rights_kind <= right_kind::MAX`
- `nonce == parcel.rights_count` — PDA uniqueness
- `parcel.rights_count < u8::MAX` — rights limit not exceeded
- `notes.len() <= 128`
- `condition_desc.len() <= 128`
- `expires_at == 0 || expires_at > now` — expiry must be in the future or permanent
- `condition_deadline > now` — condition deadline must be in the future
- `condition_deadline < expires_at || expires_at == 0` — condition deadline must be before expiry (or expiry is permanent)

**Effects:**
- All standard `grant_right` effects apply
- `rights.status = right_status::ACTIVE`
- `rights.grace_period_secs = grace_period_secs`
- Condition metadata stored in `rights.notes` as `COND:<deadline>:<desc>` prefix (appended to user notes)

**Emits:** `RightGranted` (standard event) + `RightStatusTransition` (ACTIVE)

### 6.4 `update_right_status` (inline lazy evaluation helper)

**Purpose:** Called at the start of any instruction that touches a Rights account. Not a standalone instruction — it is a helper invoked inline by other instructions (e.g., `transfer_parcel` when a right is attached, `subdivide_parcel` when rights must be re-evaluated). This is the core of lazy evaluation.

**Accounts:** Same as the calling instruction (rights account is already present).

**Args:** None (operates on `rights` account passed via context).

**Guards:** None (this is an internal helper, not user-facing).

**Effects:**
1. If `rights.expires_at == 0` → no-op (permanent right)
2. If `rights.status == right_status::REVOKED || rights.status == right_status::RENEWED || rights.status == right_status::EXPIRED` → no-op (already terminal)
3. If `now < rights.expires_at`:
   - If `now > rights.expires_at - EXPIRING_WARNING_SECS` → `rights.status = right_status::EXPIRING`
   - Else → `rights.status = right_status::ACTIVE`
4. If `now >= rights.expires_at`:
   - If `rights.grace_period_secs > 0 && now < rights.expires_at + rights.grace_period_secs`:
     - `rights.status = right_status::GRACE`
   - Else:
     - `rights.status = right_status::EXPIRED`

**Emits:** `RightStatusTransition` (only if status actually changes)

**Constant:**
```rust
pub const EXPIRING_WARNING_SECS: i64 = 30 * 24 * 60 * 60; // 30 days
```

## 7. Off-Chain Protocol

### 7.1 Lazy Evaluation Flow

Lazy evaluation is the primary enforcement mechanism. Every instruction that reads or writes a Rights account calls `update_right_status` before processing:

1. Instruction receives a Rights account via Anchor context.
2. Before any business logic, the instruction calls `update_right_status(&mut rights)`.
3. `update_right_status` reads `Clock::get()?.unix_timestamp` and compares against `rights.expires_at`.
4. Status is updated in-place if the right has crossed an expiry boundary.
5. The calling instruction then checks `rights.status` — if EXPIRED or GRACE, write operations are rejected.
6. Read-only operations (e.g., querying a right) still work but see the updated status.

**Key property:** No background process is needed. The right's status is always correct the moment any instruction touches it.

### 7.2 Keeper Sweep Flow

The keeper is a convenience layer on top of lazy evaluation. It proactively sweeps rights that nobody has touched:

1. `terra-api` runs a cron job (e.g., hourly) that queries the database for Rights where `expires_at < now` and `status == ACTIVE || status == EXPIRING`.
2. For each such right, the keeper submits a `sweep_expired_rights` transaction.
3. The transaction updates the on-chain status to EXPIRED or GRACE.
4. The API mirrors the new status to the PostGIS database.
5. If the keeper goes down, lazy evaluation still catches the right the next time anyone touches it.

**Failure mode:** If the keeper is down AND nobody touches the right, the right remains technically ACTIVE on-chain until someone does touch it. This is acceptable because:
- The right cannot be used for writes (the holder can still read it).
- The granter can still revoke it.
- The holder can still request renewal.
- The API can surface the "actually expired" state based on `expires_at` even if on-chain status is stale.

### 7.3 Renewal Ceremony Flow

1. Holder notices their right is approaching expiry (via API notification or on-chain query).
2. Holder calls `renew_right` with a new expiry date. The transaction must also be signed by the granter.
3. The program verifies both signatures and extends `expires_at`.
4. Status is reset to ACTIVE.
5. If the holder and granter cannot agree on terms, the right expires normally.

**No grace period for renewal:** Renewal must happen before expiry. Once in GRACE, the right is in its final window — renewal is still possible but the grace period continues counting.

### 7.4 Conditional Right Evaluation

1. Grantor creates a conditional right via `grant_conditional_right`.
2. The condition deadline is stored in the on-chain record.
3. The off-chain API monitors condition deadlines and flags rights that are approaching their condition deadline.
4. If the condition is not met by the deadline, the right remains ACTIVE (on-chain has no way to verify external conditions). The granter or an oracle must call `revoke_right` to enforce the condition.
5. This is a deliberate design choice: on-chain programs cannot observe off-chain state. The condition is a social/legal commitment, not a cryptographic one.

### 7.5 Expiry Notification Flow

1. `terra-api` queries Rights where `expires_at - now < NOTIFICATION_WINDOW_SECS` (e.g., 30 days).
2. API sends notifications to the holder and granter via their registered communication channels.
3. Notifications include: right ID, parcel ID, expiry date, current status, renewal instructions.
4. This is entirely off-chain — no on-chain changes.

## 8. Collusion Resistance

### 8.1 Holder + Granter Co-Sign for Renewal

Renewal requires both parties to agree. A colluding subset cannot:
- Renew a right without the granter's consent (holder alone cannot renew).
- Force renewal on a right the granter wants to let expire (granter alone cannot renew without holder request).

### 8.2 Revocation Requires Authority

Revocation requires parcel owner OR original granter. A holder cannot:
- Self-revoke to claim a refund or grace period benefit.
- Forge a revocation to trigger a dispute.

### 8.3 Keeper Cannot Manipulate Status

The keeper can only move rights forward in the status lifecycle (ACTIVE → EXPIRING → EXPIRED/GRACE). It cannot:
- Move a right back to ACTIVE.
- Skip the GRACE period.
- Renew a right (requires holder + granter co-sign).

### 8.4 Status Lifecycle is Monotonic

The status transitions form a directed acyclic graph:

```
ACTIVE → EXPIRING → EXPIRED
                      ↓
                    GRACE → EXPIRED (after grace period ends)
ACTIVE → RENEWED (new right created)
ACTIVE → REVOKED (explicit revocation)
```

A right can never move backward (EXPIRED → ACTIVE is impossible).

## 9. Liveness Guarantees

### 9.1 Lazy Evaluation is Always Available

Lazy evaluation does not depend on any off-chain component. As long as the Solana network is operational, any transaction that touches a Rights account will correctly evaluate its status. This is the primary liveness guarantee.

### 9.2 Keeper Degrades Gracefully

If the keeper goes down:
- Rights that have expired but are not yet touched remain in their last known status (ACTIVE or EXPIRING).
- The API can still surface the correct status based on `expires_at` and `Clock::get()`.
- The next transaction that touches the right will update the on-chain status.
- No rights are "stuck" — they can still be revoked, renewed, or queried.

### 9.3 Grace Period Provides Transition Window

The grace period ensures that a holder who loses track of time still has a window to:
- Vacate the premises (for leases).
- Complete construction (for permits).
- Transfer the right to a successor.

During GRACE, the holder retains read access and can still request renewal.

### 9.4 Permanent Rights Are Unaffected

Rights with `expires_at == 0` are never swept, never expire, and never enter GRACE. This RFC does not change the semantics of permanent rights.

## 10. Storage Architecture

### 10.1 On-Chain State

- `Rights` account gains two new fields: `status` (1 byte) and `grace_period_secs` (8 bytes).
- Existing fields are unchanged. No migration required — new fields default to zero, which means `status = ACTIVE` and `grace_period_secs = 0`.
- Space increase is minimal (~9 bytes per Rights account).

### 10.2 Event Emission

- `RightStatusTransition` events are emitted on every status change.
- `RightRenewed` events are emitted on every renewal.
- Events are indexed by the API for historical queries and audit trails.

### 10.3 API Mirror

- `terra-api` mirrors the new `status` and `grace_period_secs` fields to PostGIS.
- API queries can filter by status (e.g., "show me all expiring rights").
- API can compute derived fields (e.g., "days until expiry") without on-chain changes.

### 10.4 Account Migration

- No migration is required. The new fields are appended to the existing `Rights` struct.
- Anchor's `#[account]` macro handles zero-initialization of new fields.
- Existing Rights accounts gain `status = 0` (ACTIVE) and `grace_period_secs = 0` (no grace) automatically.

## 11. Replay / Nonce Hygiene

### 11.1 Status Transitions Are Idempotent

Calling `sweep_expired_rights` on an already-expired right is a no-op. The guard checks `rights.status == ACTIVE || rights.status == EXPIRING` and rejects if already terminal. This prevents replay of sweep transactions.

### 11.2 Renewal Creates a Unique Event

Each `renew_right` call emits a `RightRenewed` event with the block timestamp. Two renewals of the same right produce distinct events with different timestamps.

### 11.3 Nonce Reuse Prevention

The `nonce` field in `grant_conditional_right` follows the same pattern as `grant_right`: `nonce == parcel.rights_count`, and `rights_count` is incremented. No two Rights PDAs share the same parcel+nonce pair.

### 11.4 Clock Uniqueness

All time comparisons use `Clock::get()?.unix_timestamp` from the current transaction's block. Two transactions in the same block see the same clock. Two transactions in different blocks may see different clocks. This is by design — the block timestamp is the canonical time source.

## 12. Post-Quantum Migration Path

### 12.1 No New Key Material

This RFC introduces no new cryptographic keys or signatures. The Ed25519 signatures used for transaction authorization are unchanged. Post-quantum migration for Terra's signature scheme is handled by a future RFC, not by this one.

### 12.2 Status Transitions Are Plain Data

Status transitions are emitted as events, not as signed messages. They do not need to be post-quantum secure — they are log entries, not credentials.

### 12.3 Conditional Conditions Are Off-Chain

The conditional right mechanism relies on off-chain enforcement (oracle, social commitment). Post-quantum migration of the oracle or commitment scheme is a separate concern.

## 13. Operational Security

### 13.1 Keeper Wallet Security

- The keeper wallet should be a hot wallet with minimal authority — it can only call `sweep_expired_rights`.
- The keeper wallet cannot renew, revoke, or grant rights.
- If the keeper wallet is compromised, the attacker can only mark rights as expired (which they would become anyway via lazy evaluation).

### 13.2 Notification System Security

- Expiry notifications are sent off-chain via the API.
- Notifications must not include sensitive information (right details are public on-chain).
- The notification system must not be used for phishing — notifications should only link to the official dApp.

### 13.3 Grace Period Configuration

- `grace_period_secs` is set at grant time and cannot be changed after granting.
- The granter controls the grace period length. Typical values:
  - Leases: 30 days (2,592,000 seconds)
  - Construction permits: 60 days (5,184,000 seconds)
  - Easements: 0 (no grace, immediate expiry)
- The maximum grace period should be capped at 1 year (31,536,000 seconds) to prevent abuse.

### 13.4 Emergency Revocation

- The parcel owner can always revoke a right via the existing `revoke_right` instruction.
- Revocation is immediate and does not respect grace periods.
- Revoked rights are closed and their lamports returned to the granter.

## 14. Test Vectors

### 14.1 Lazy Evaluation — Right Not Yet Expired

- Right: `expires_at = now + 86400` (1 day from now), `status = ACTIVE`
- Instruction: any instruction that touches the right
- Expected: `update_right_status` runs, `now < expires_at`, status remains ACTIVE. Instruction proceeds.

### 14.2 Lazy Evaluation — Right Entering Expiring Window

- Right: `expires_at = now + 15 * 86400` (15 days from now), `status = ACTIVE`
- `EXPIRING_WARNING_SECS = 30 * 86400`
- `now > expires_at - EXPIRING_WARNING_SECS` is false (15 days < 30 days warning? No — 15 days is within the 30-day window)
- Expected: `update_right_status` sets `status = EXPIRING`. Instruction proceeds.

### 14.3 Lazy Evaluation — Right Expired, No Grace

- Right: `expires_at = now - 86400` (1 day ago), `grace_period_secs = 0`, `status = ACTIVE`
- Instruction: any instruction that touches the right
- Expected: `update_right_status` sets `status = EXPIRED`. Write instructions reject with error. Read instructions proceed.

### 14.4 Lazy Evaluation — Right Expired, Within Grace Period

- Right: `expires_at = now - 86400` (1 day ago), `grace_period_secs = 30 * 86400` (30 days), `status = ACTIVE`
- `now < expires_at + grace_period_secs` is true
- Expected: `update_right_status` sets `status = GRACE`. Holder retains read access. Write instructions may still proceed (grace period is a warning, not a lock).

### 14.5 Lazy Evaluation — Right Expired, Grace Period Ended

- Right: `expires_at = now - 60 * 86400` (60 days ago), `grace_period_secs = 30 * 86400` (30 days), `status = ACTIVE`
- `now >= expires_at + grace_period_secs`
- Expected: `update_right_status` sets `status = EXPIRED`. Write instructions reject.

### 14.6 Keeper Sweep — Active Right Past Expiry

- Right: `expires_at = now - 86400`, `grace_period_secs = 0`, `status = ACTIVE`
- Keeper calls `sweep_expired_rights`
- Expected: `rights.status = right_status::EXPIRED`. Event emitted.

### 14.7 Keeper Sweep — Active Right in Grace Window

- Right: `expires_at = now - 86400`, `grace_period_secs = 30 * 86400`, `status = ACTIVE`
- Keeper calls `sweep_expired_rights`
- Expected: `rights.status = right_status::GRACE`. Event emitted.

### 14.8 Keeper Sweep — Already Expired (No-Op)

- Right: `expires_at = now - 86400`, `status = right_status::EXPIRED`
- Keeper calls `sweep_expired_rights`
- Expected: Guard rejects `status != ACTIVE && status != EXPIRING`. No state change.

### 14.9 Keeper Sweep — Permanent Right (No-Op)

- Right: `expires_at = 0`, `status = right_status::ACTIVE`
- Keeper calls `sweep_expired_rights`
- Expected: Guard rejects `expires_at == 0`. No state change.

### 14.10 Renewal — Success

- Right: `expires_at = now + 86400`, `status = ACTIVE`
- Holder signs `renew_right` with `new_expires_at = now + 365 * 86400`
- Granter co-signs
- Expected: `rights.expires_at = new_expires_at`, `rights.status = ACTIVE`. Event emitted.

### 14.11 Renewal — Holder Alone (Rejected)

- Right: `expires_at = now + 86400`, `status = ACTIVE`
- Holder calls `renew_right` without granter signature
- Expected: Transaction fails (missing required signer).

### 14.12 Renewal — Granter Alone (Rejected)

- Right: `expires_at = now + 86400`, `status = ACTIVE`
- Granter calls `renew_right` without holder signature
- Expected: Transaction fails (missing required signer).

### 14.13 Renewal — New Expiry in the Past (Rejected)

- Right: `expires_at = now + 86400`, `status = ACTIVE`
- Holder + granter call `renew_right` with `new_expires_at = now - 86400`
- Expected: Guard rejects `new_expires_at <= now`.

### 14.14 Renewal — Shorter Expiry (Rejected)

- Right: `expires_at = now + 365 * 86400`, `status = ACTIVE`
- Holder + granter call `renew_right` with `new_expires_at = now + 30 * 86400`
- Expected: Guard rejects `new_expires_at <= rights.expires_at`.

### 14.15 Renewal — Revoked Right (Rejected)

- Right: `expires_at = now + 86400`, `status = REVOKED`
- Holder + granter call `renew_right`
- Expected: Guard rejects `status == REVOKED`.

### 14.16 Grant Conditional Right — Success

- Parcel: `rights_count = 0`, `owner = wallet A`
- `grant_conditional_right(nonce=0, rights_kind=USAGE, holder=B, expires_at=now+365*86400, condition_deadline=now+180*86400, condition_desc="Construction must start within 6 months")`
- Expected: Rights PDA created, status = ACTIVE, condition metadata in notes.

### 14.17 Grant Conditional Right — Condition Deadline After Expiry (Rejected)

- `expires_at = now + 90 * 86400`, `condition_deadline = now + 180 * 86400`
- Expected: Guard rejects `condition_deadline >= expires_at`.

### 14.18 Grant Conditional Right — Condition Deadline in the Past (Rejected)

- `condition_deadline = now - 86400`
- Expected: Guard rejects `condition_deadline <= now`.

### 14.19 Status Lifecycle — Full Cycle

1. Grant right: status = ACTIVE
2. Time passes, enters 30-day window: status = EXPIRING (via lazy evaluation)
3. Time passes, expires: status = EXPIRED (via keeper or lazy evaluation)
4. No grace period → terminal state reached

### 14.20 Status Lifecycle — With Grace

1. Grant right with `grace_period_secs = 30 * 86400`: status = ACTIVE
2. Time passes, expires: status = GRACE (via lazy evaluation or keeper)
3. 30 days pass: status = EXPIRED (via lazy evaluation or keeper)
4. Terminal state reached

### 14.21 Status Lifecycle — Renewal

1. Grant right: status = ACTIVE
2. Time passes, expires: status = EXPIRED
3. Holder + granter call `renew_right` with new expiry: status = ACTIVE (new right or same account)
4. Cycle can repeat indefinitely
