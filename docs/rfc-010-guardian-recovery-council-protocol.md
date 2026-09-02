# RFC-010: Guardian & Recovery Council Protocol

## 1. Status

- **Status:** Draft
- **Created:** 2026-09-03
- **Supersedes:** None
- **Implements:** `succession_kind::GUARDIANSHIP` (3) and `succession_kind::COURT_APPOINTED_GUARDIAN` (4) — prep hooks already landed in `lib.rs:137-139`

> **This is NOT a separate protocol.** The Guardian & Recovery Council is a specialization of the existing Succession system (RFC-001). No new account types, no new instruction set. We extend `succession_kind` with two additional variants and enforce stricter guard rails on claims using those variants.

## 2. Summary

This RFC specifies the **Guardian & Recovery Council Protocol** — a mechanism for appointing and claiming guardianship over an identity (and its associated parcels) when the identity owner cannot act on their own behalf. The protocol addresses three populations that cannot hold keys or exercise control:

1. **Deceased persons with no known heirs.** No successor wallet exists; the estate must be managed by an appointed guardian.
2. **Minors who cannot hold keys.** A parent or court-appointed guardian must manage parcels until the minor reaches the age of majority.
3. **Persons with cognitive impairment.** A court-appointed guardian is authorized to manage parcels on the subject's behalf.

The design folds directly into the existing `Succession` account and instruction set. A guardianship request is a `Succession` with `kind = GUARDIANSHIP` (3) or `kind = COURT_APPOINTED_GUARDIAN` (4). The claim gate is the same two-gate mechanism (grace period + validator endorsements), but with **strictly higher thresholds** to reflect the larger and more open-ended scope of control a guardian receives compared to an ordinary heir.

**Key design decisions:**

- Higher validator endorsement threshold for guardianship: `MIN_SUCCESSION_VALIDATIONS + 2` (minimum 3) vs 1 for ordinary succession.
- Longer minimum grace period: 90 days vs 7 days for ordinary succession, giving the subject (or their advocates) time to object.
- Court-appointed guardianships require a non-zero `case_hash` binding the claim to a specific legal proceeding.
- Guardian scope can be limited via the existing `notes` field on the Succession (e.g., "limited to parcel X only").
- Revocation requires either a court order or the subject's recovery wallet signaling recovery.

## 3. Threat Model

### 3.1 Adversary Classes

| Class | Description | Mitigation |
|-------|-------------|------------|
| **Rogue guardian** | A guardian appointed via collusion seizes control of all parcels | Higher validator threshold (≥3), longer grace period (≥90 days), court-order revocation path |
| **Colluding validators** | Validators endorse a fraudulent guardianship claim | Minimum 3 independent validators; self-dealing check prevents identity owner from being their own validator |
| **Adversary posing as heir** | Thief files a guardianship claim to seize land | Guardian kind requires ≥3 validator endorsements (vs 1 for succession); 90-day grace period allows objection |
| **Absent subject** | Subject is deceased or incapacitated and cannot cancel | Recovery wallet can cancel; court-appointed path has case_hash anchor; validators act as independent verification |
| **Scope creep** | Guardian appointed for one parcel expands to all parcels | `notes` field limits scope; on-chain claim only transfers ownership if the claim passes the full gate; parcels are individually re-pointed via `remaining_accounts` |
| **Self-dealing guardian** | Identity owner appoints themselves as guardian | Explicit check: `successor != identity.owner` enforced in `request_succession` |
| **Validator self-dealing** | Identity owner endorses their own guardianship | `ValidatorOwnsAsset` check: validator must not be the identity owner |

### 3.2 In Scope

- Guardianship appointment via Succession with `kind ∈ {GUARDIANSHIP, COURT_APPOINTED_GUARDIAN}`
- Validator endorsement with elevated threshold
- Grace period enforcement (minimum 90 days for guardianship)
- Court-appointed guardian path requiring a `case_hash`
- Revocation via court order or subject's recovery wallet
- Scope limitation via `notes` field
- On-chain audit trail of all guardianship transitions

### 3.3 Out of Scope

- The actual legal process of appointing a guardian (off-chain, jurisdiction-specific)
- Medical diagnosis or cognitive impairment assessment (off-chain, professional judgment)
- Vault shard protocol interactions (covered by RFC-003)
- Dispute resolution and parcel freeze flows (covered by RFC-007)
- Validator identity verification (covered by AuthorityRegistry)

## 4. Cryptographic Choices

### 4.1 Signatures: Ed25519

- Solana native — all validator keys are Ed25519.
- Used for transaction signing (on-chain endorsement and claim).
- Each validator endorsement is an independent Ed25519 signature (the validator signs the `endorse_succession` transaction with their wallet).

### 4.2 Hashing: SHA-256

- `case_hash` for court-appointed guardianships: SHA-256 of the off-chain court order or guardianship decree.
- On-chain identity hash: SHA-256 of the identity credential (already part of the Identity account).
- Binding integrity: the `case_hash` immutably anchors a specific legal proceeding to the on-chain record.

### 4.3 No Additional Cryptography Required

The guardianship protocol inherits all cryptographic primitives from the existing Succession system. No new key generation, encryption, or threshold schemes are introduced. This is a deliberate design choice — the guardianship protocol is a policy layer on top of existing mechanics, not a new cryptographic protocol.

## 5. Data Model

### 5.1 Succession Account (on-chain PDA)

**PDA seed:** `["succession", identity, successor]`

The Succession account is reused without modification. The `kind` field determines whether the claim is an ordinary succession, a guardianship, or a court-appointed guardianship.

| Field | Type | Description |
|-------|------|-------------|
| `identity` | `Pubkey` | Identity PDA key (the person whose control is being passed) |
| `successor` | `Pubkey` | The wallet that will take over once gated |
| `kind` | `u8` | 0=Successor, 1=Recovery, 2=Transfer, 3=Guardianship, 4=CourtAppointedGuardian |
| `requested_at` | `i64` | Unix timestamp of the request |
| `effective_at` | `i64` | `requested_at + grace_secs` — claim only allowed after this |
| `grace_secs` | `i64` | Configurable per-request grace period (clamped to [MIN, MAX]) |
| `required` | `u8` | Number of validator endorsements required before claim |
| `validations_count` | `u8` | Number of endorsements collected so far |
| `validators` | `[Pubkey; 8]` | Declared local-authority validator set acting as testifiers |

### 5.2 Guardianship-Specific Constants

| Constant | Value | Description |
|----------|-------|-------------|
| `MIN_GUARDIANSHIP_GRACE_SECS` | `90 * 24 * 3600` (90 days) | Minimum grace period for any guardianship claim |
| `MIN_GUARDIANSHIP_VALIDATIONS` | `MIN_SUCCESSION_VALIDATIONS + 2` (= 3) | Minimum validator endorsements for guardianship |
| `MAX_GUARDIANSHIP_GRACE_SECS` | `MAX_SUCCESSION_GRACE_SECS` (180 days) | Maximum grace period (inherited) |
| `DEFAULT_GUARDIANSHIP_GRACE_SECS` | `180 * 24 * 3600` (180 days) | Default grace when requester passes 0 |

### 5.3 Succession Kind Enum

| Variant | Value | Description | Required Validators | Min Grace |
|---------|-------|-------------|---------------------|-----------|
| `SUCCESSOR` | 0 | Estate/inheritance passation | ≥ 1 | 7 days |
| `RECOVERY` | 1 | Lost/stolen key recovery | ≥ 1 | 7 days |
| `TRANSFER` | 2 | Deliberate control transfer | ≥ 1 | 7 days |
| `GUARDIANSHIP` | 3 | Voluntary guardianship appointment | ≥ 3 | 90 days |
| `COURT_APPOINTED_GUARDIAN` | 4 | Court-ordered guardianship | ≥ 3 | 90 days |

### 5.4 Identity Account (on-chain PDA)

**PDA seed:** `["identity", identity_hash]`

| Field | Type | Description |
|-------|------|-------------|
| `identity_hash` | `[u8; 32]` | SHA-256 of the identity credential |
| `owner` | `Pubkey` | Active wallet acting on behalf of this identity |
| `recovery` | `Pubkey` | Backup/recovery wallet (used to cancel guardianship) |
| `parcel_count` | `u16` | Number of parcels currently owned |
| `created_at` | `i64` | Creation timestamp |
| `updated_at` | `i64` | Last update timestamp |

### 5.5 Scope Limitation via `notes`

The `Succession` account inherits a `notes` field from the generic succession mechanism (max 128 bytes). For guardianships, this field is used to limit the guardian's scope:

- `""` (empty) — full guardianship over all parcels owned by the identity
- `"limited_to_parcel_<base58>"` — guardianship applies only to the specified parcel
- `"limited_to_parcels_<hash>"` — guardianship applies to a set of parcels (hash of the set)
- `"until_<date>"` — guardianship expires on a specific date (enforced off-chain; on-chain claim is still gated by the grace period and validators)

> **Note:** Scope enforcement is advisory and relies on validator judgment. Validators endorsing a guardianship with limited scope are implicitly attesting that the scope is appropriate. The on-chain claim mechanism does not enforce parcel-level scope — it transfers the entire identity. Parcel-level scoping is an off-chain convention enforced by social consensus and validator oversight.

## 6. Instructions

### 6.1 `request_succession` (with `kind = 3` or `kind = 4`)

**Purpose:** Request guardianship over an identity. This is the existing `request_succession` instruction with a guardianship-specific `kind` value. No new instruction is needed.

**Accounts:**
- `identity` (mut, PDA `["identity", identity_hash]`)
- `succession` (init, PDA `["succession", identity, successor]`)
- `signer` (signer, mut — pays rent; must be identity owner, recovery wallet, or registry admin)
- `system_program`

**Args:**
- `successor: Pubkey` — the guardian's wallet
- `kind: u8` — must be 3 (GUARDIANSHIP) or 4 (COURT_APPOINTED_GUARDIAN)
- `grace_secs: i64` — must be ≥ `MIN_GUARDIANSHIP_GRACE_SECS` (90 days) if kind is 3 or 4
- `required_validations: u8` — must be ≥ `MIN_GUARDIANSHIP_VALIDATIONS` (3) if kind is 3 or 4
- `validators: [Pubkey; 8]` — declared validator set

**Guards (in addition to existing `request_succession` guards):**
- If `kind ∈ {GUARDIANSHIP, COURT_APPOINTED_GUARDIAN}`:
  - `grace_secs >= MIN_GUARDIANSHIP_GRACE_SECS` (or 0 for default 180 days)
  - `required_validations >= MIN_GUARDIANSHIP_VALIDATIONS` (3)
  - `successor != identity.owner` (self-dealing check)
- If `kind == COURT_APPOINTED_GUARDIAN`:
  - The `case_hash` is embedded in the `notes` field or passed as a separate argument (see §6.1.1)
  - `case_hash` must be non-zero

**Effects:**
- Succession account initialized with the specified `kind`, `grace_secs`, `required`, and `validators`
- `effective_at = now + grace_secs`

**Emits:** `SuccessionRequested` (existing event)

#### 6.1.1 Court-Appointed Guardian: `case_hash` Binding

For `kind == COURT_APPOINTED_GUARDIAN`, the `case_hash` (SHA-256 of the court order) is passed as part of the `notes` field or as an additional argument. The on-chain program enforces that `case_hash != [0; 32]` for court-appointed guardianships.

Implementation options:
1. **Extend `request_succession` args** — add `case_hash: [u8; 32]` parameter, validated only when `kind == 4`.
2. **Embed in `notes`** — the caller encodes `case_hash` as a hex string in the `notes` field. The program validates non-zero length for `kind == 4`.

Option 1 is cleaner. The program adds a conditional guard:

```
if kind == succession_kind::COURT_APPOINTED_GUARDIAN {
    require!(case_hash != [0; 32], TerraError::EmptyCaseHash);
}
```

### 6.2 `endorse_succession` (existing instruction, higher threshold)

**Purpose:** Record one validator's endorsement of a pending guardianship. The existing `endorse_succession` instruction is used without modification. The threshold enforcement is in `claim_succession`.

**Accounts:**
- `identity` (mut, PDA `["identity", identity_hash]`)
- `succession` (mut, PDA `["succession", identity, successor]`)
- `validator` (signer — declared validator for this succession)
- `system_program`

**Args:** None

**Guards:**
- `now < succession.effective_at` (still within grace period)
- `succession.validations_count < succession.required` (limit not reached)
- `validator` is in `succession.validators`
- `validator != identity.owner` (self-dealing check)

**Effects:**
- `succession.validations_count += 1`

**Emits:** `SuccessionEndorsed` (existing event)

### 6.3 `claim_succession` (existing instruction, higher threshold enforced)

**Purpose:** Claim guardianship once BOTH the grace period has elapsed AND the required number of validators have endorsed it. The existing `claim_succession` instruction is used. The threshold check (`validations_count >= required`) naturally enforces the higher threshold because `required` was set to ≥3 when the guardianship was requested.

**Accounts:**
- `identity` (mut, PDA `["identity", identity_hash]`)
- `succession` (mut, PDA `["succession", identity, successor]`, close → signer)
- `signer` (signer, mut — must be the `successor`)
- `system_program`

**Args:** None

**Guards:**
- `succession.successor == signer.key()` (only the named successor may claim)
- `now >= succession.effective_at` (grace period elapsed)
- `succession.validations_count >= succession.required` (validator quorum met)
- `succession.identity == identity.key()` (identity match)

**Effects:**
- `identity.owner = successor`
- `identity.recovery = Pubkey::default()` (recovery wallet reset)
- `identity.updated_at = now`
- Parcels supplied via `remaining_accounts` are re-pointed to the successor
- Succession account is closed (lamports returned to signer)

**Emits:** `SuccessionClaimed` (existing event)

### 6.4 `cancel_succession` (existing instruction, extended revocation)

**Purpose:** Cancel a pending guardianship. The existing `cancel_succession` instruction is used with extended authorization.

**Accounts:**
- `identity` (mut, PDA `["identity", identity_hash]`)
- `succession` (mut, PDA `["succession", identity, successor]`, close → signer)
- `signer` (signer, mut)
- `system_program`

**Args:** None

**Guards:**
- `now < succession.effective_at` (still within grace period)
- One of:
  - `signer == identity.owner` (current owner cancels)
  - `signer == identity.recovery` (recovery wallet cancels)
  - `signer == registry.admin` (registry admin cancels — for court-ordered revocation)

**Effects:**
- Succession account is closed (lamports returned to signer)

**Emits:** `SuccessionCancelled` (existing event)

### 6.5 `revoke_guardianship` (NEW instruction)

**Purpose:** Revoke an already-claimed guardianship. This is the only new instruction in this RFC. It handles the case where a guardian has already claimed control and needs to be removed — either because the subject has recovered capacity, or a court has ordered revocation.

**Accounts:**
- `identity` (mut, PDA `["identity", identity_hash]`)
- `revoker` (signer, mut)
- `system_program`

**Args:** `new_owner: Pubkey`

**Guards:**
- `revoker` is one of:
  - `identity.recovery` (subject's recovery wallet — signals the subject has recovered)
  - Registry admin (acting on a court order)
- `new_owner != Pubkey::default()` (must specify who takes over)
- `new_owner != identity.owner` (prevents no-op transfer)

**Effects:**
- `identity.owner = new_owner`
- `identity.recovery = Pubkey::default()` (reset recovery)
- `identity.updated_at = now`

**Emits:** `GuardianshipRevoked`

```
#[event]
pub struct GuardianshipRevoked {
    pub identity: Pubkey,
    pub previous_guardian: Pubkey,
    pub new_owner: Pubkey,
    pub revoked_by: Pubkey,
    pub block_time: i64,
}
```

### 6.6 Instruction Summary

| Instruction | New? | Purpose | Guard Rails |
|-------------|------|---------|-------------|
| `request_succession` | No | Request guardianship (kind=3 or 4) | ≥3 validators, ≥90-day grace, case_hash for kind=4 |
| `endorse_succession` | No | Validator endorses pending guardianship | Same as existing |
| `claim_succession` | No | Claim after grace + endorsements | Same as existing (threshold enforced by `required` field) |
| `cancel_succession` | No | Cancel pending guardianship | Owner, recovery, or admin |
| `revoke_guardianship` | **Yes** | Revoke claimed guardianship | Recovery wallet or admin only |

## 7. Off-Chain Protocol

### 7.1 Guardianship Appointment Ceremony

1. **Identification of need.** A party (family member, social worker, court) identifies that an identity owner needs a guardian.
2. **Validator selection.** The petitioner selects ≥3 independent validators from the AuthorityRegistry. Validators must NOT be the identity owner, the proposed guardian, or have a conflict of interest.
3. **Guardian selection.** The petitioner selects a guardian wallet (the `successor` in the Succession account).
4. **Scope definition.** The petitioner defines the guardian's scope in the `notes` field (e.g., "limited to parcel X", "full guardianship", "until 2027-01-01").
5. **Court order (if court-appointed).** For `kind = COURT_APPOINTED_GUARDIAN`, the petitioner obtains a court order and computes `case_hash = SHA-256(court_order_document)`.
6. **On-chain request.** The petitioner (or a relayer) calls `request_succession` with `kind = 3` or `kind = 4`, the guardian wallet, ≥90-day grace, ≥3 required validations, and the validator set.
7. **Grace period begins.** The 90-day clock starts. The identity owner (or their recovery wallet) can cancel at any time during this window.
8. **Validator endorsement.** Each of the ≥3 validators independently reviews the guardianship request (off-chain: verifies the court order, confirms the subject's incapacity, checks for conflicts of interest). Each calls `endorse_succession`.
9. **Claim.** After the grace period elapses AND ≥3 validators have endorsed, the guardian calls `claim_succession`. The identity owner changes to the guardian's wallet. Parcels are re-pointed.

### 7.2 Revocation Ceremony

1. **Recovery event.** The subject regains capacity (or a court revokes the guardianship).
2. **Revocation request.** The subject's recovery wallet (or registry admin with court order) calls `revoke_guardianship`.
3. **New owner assignment.** The revoker specifies `new_owner` — either the subject's new active wallet (if they've recovered) or another guardian.
4. **On-chain transfer.** The identity owner is updated. Parcels are re-pointed if necessary (via separate `transfer_parcel` calls or a batch operation).

### 7.3 Court-Appointed Guardian Verification

Validators endorsing a court-appointed guardianship should verify:

1. The `case_hash` matches a real court order (off-chain verification).
2. The court order names the proposed guardian as the appointed guardian.
3. The court order specifies the scope of guardianship (which parcels, what authority).
4. The court order is from a jurisdiction with proper authority over the identity.

This verification happens entirely off-chain. The on-chain program only checks that `case_hash != [0; 32]`.

### 7.4 Multi-Subject Guardianship

A single guardian may be appointed for multiple identities (e.g., a parent managing parcels for multiple minor children). Each identity requires a separate `request_succession` call with the same guardian wallet but different identity PDAs.

### 7.5 Guardian Scope Conventions

The `notes` field supports the following scope conventions (enforced by validator social consensus, not on-chain):

| Scope | Notes Value | Meaning |
|-------|-------------|---------|
| Full | `""` (empty) | Guardian controls all parcels |
| Limited | `"limited_to_parcel_<b58>"` | Guardian controls only the specified parcel |
| Temporal | `"until_<unix_ts>"` | Guardian control expires at the given timestamp |
| Conditional | `"conditional_on_<event>"` | Guardian control is contingent on an off-chain event |

## 8. Collusion Resistance

### 8.1 Elevated Validator Threshold

| Operation | Required Validators | Rationale |
|-----------|---------------------|-----------|
| Ordinary succession | ≥ 1 | Low bar for legitimate heirs |
| Guardianship claim | ≥ 3 | High bar — guardian gets broad control |
| Court-appointed guardian | ≥ 3 | Same high bar — court order provides additional off-chain verification |

The threshold is enforced at claim time: `claim_succession` checks `validations_count >= required`. Since `request_succession` requires `required >= MIN_GUARDIANSHIP_VALIDATIONS (3)` for guardianship kinds, a guardian cannot claim with fewer than 3 endorsements.

### 8.2 Extended Grace Period

| Operation | Minimum Grace | Rationale |
|-----------|---------------|-----------|
| Ordinary succession | 7 days | Heir may be far from a validator |
| Guardianship | 90 days | Subject (or advocates) need time to object |

The 90-day grace period is enforced at request time: `request_succession` clamps `grace_secs` to `[MIN_GUARDIANSHIP_GRACE_SECS, MAX_SUCCESSION_GRACE_SECS]` for guardianship kinds.

### 8.3 Self-Dealing Prevention

- The identity owner cannot be their own guardian: `successor != identity.owner` (checked in `request_succession`).
- The identity owner cannot be a validator for their own guardianship: `ValidatorOwnsAsset` (checked in `endorse_succession`).
- A validator cannot endorse their own rotation (existing check in `endorse_succession`).

### 8.4 Court Order Anchoring

For `kind == COURT_APPOINTED_GUARDIAN`, the `case_hash` provides:

- **Auditability:** The court order is immutably bound to the on-chain record.
- **Non-repudiation:** The petitioner cannot later claim a different court order was the basis.
- **Verification:** Validators can independently verify the `case_hash` against the actual court document.

### 8.5 Revocation Path

Even after a guardianship is claimed, the subject's recovery wallet can revoke it via `revoke_guardianship`. This ensures that a guardian cannot permanently seize control if the subject recovers capacity.

## 9. Liveness Guarantees

### 9.1 Grace Period as Liveness Window

The 90-day grace period serves dual purposes:
1. **Objection window.** The subject (or their recovery wallet) can cancel at any time.
2. **Verification window.** Validators can independently verify the guardianship request before endorsing.

### 9.2 Validator Availability

If fewer than 3 validators are available to endorse (e.g., validators have left the pilot, lost keys), the guardianship cannot be claimed. Options:

1. **Rotate the validator set** via `rotate_validators` on the attestation (existing mechanism) before requesting guardianship.
2. **Request a new guardianship** with a different validator set.
3. **Use the court-appointed path** — the court order provides additional legitimacy, and the registry admin can facilitate validator rotation.

### 9.3 Recovery Wallet Liveness

The recovery wallet is the primary revocation path. If the recovery wallet is also lost:

1. The subject (or their advocate) can bind a new recovery wallet via a new `bind_identity` call (if they have access to a new wallet).
2. The registry admin can intervene via the court-appointed path.
3. Emergency succession (`kind = RECOVERY`) can be used to recover the identity first, then revoke the guardianship.

### 9.4 Subject Death

If the subject dies during the grace period:

1. The recovery wallet (if controlled by an executor) can let the guardianship proceed.
2. The recovery wallet can cancel and file a new succession (`kind = SUCCESSOR`) for the estate.
3. The guardianship claim proceeds normally after the grace period if not cancelled.

## 10. Storage Architecture

### 10.1 On-Chain State

All guardianship state lives on-chain in the existing `Succession` PDA:

- `kind` field discriminates guardianship from ordinary succession
- `required` field enforces the elevated validator threshold
- `grace_secs` field enforces the extended grace period
- `case_hash` (for court-appointed) is embedded in `notes` or passed as an argument

### 10.2 Off-Chain Documents

Court orders and guardianship decrees are stored off-chain (IPFS/Arweave) and anchored on-chain via `case_hash`. The same storage architecture as RFC-003 (Vault Shard Protocol) applies:

- **IPFS:** Content-addressed. CID is the integrity check.
- **Arweave:** Permanent, pay-once mirror.
- **PostGIS:** Local cache + indexing.

### 10.3 No New Account Types

The guardianship protocol introduces zero new account types. All state is carried by the existing `Succession` account, which already has the necessary fields.

## 11. Replay / Nonce Hygiene

### 11.1 PDA Uniqueness

The Succession PDA seed is `["succession", identity, successor]`. This means:

- Only one guardianship request can exist per (identity, guardian) pair at a time.
- A new request for the same guardian requires closing the previous one first.
- Different guardians for the same identity produce different PDAs and can coexist.

### 11.2 Monotonic Identity Version

The `identity.updated_at` timestamp is bumped on every ownership change. This provides a monotonic ordering of transitions and prevents stale references.

### 11.3 Succession Account Closure

When a guardianship is claimed or cancelled, the Succession account is closed (lamports returned to the signer). This prevents:

- Replay of a claimed guardianship (account no longer exists)
- Double-claiming (account is closed after first claim)
- Stale endorsements (account is closed, no new endorsements possible)

### 11.4 Validator Endorsement Idempotency

Each validator can endorse a succession at most once (the `validations_count` check prevents double-endorsing). If a validator attempts to endorse twice, the transaction fails with `ValidationLimitReached`.

## 12. Post-Quantum Migration Path

### 12.1 No Cryptographic Migration Needed

The guardianship protocol uses only Ed25519 signatures and SHA-256 hashes — both of which have post-quantum migration paths:

- **Ed25519 → Dilithium:** Solana's signature scheme can be upgraded to a post-quantum variant via SIMD proposals. The guardianship protocol is agnostic to the signature scheme.
- **SHA-256 → SHA-3:** Hash functions are generally quantum-resistant (Grover's algorithm provides only a quadratic speedup, reducing 256-bit security to 128-bit — still secure).

### 12.2 No Key Encapsulation

The guardianship protocol does not use key encapsulation (KEM) or public-key encryption. There is no ciphertext to migrate. This is a deliberate design choice — the protocol is a policy layer, not a cryptographic protocol.

### 12.3 Future-Proofing

If Solana introduces post-quantum signature schemes, the guardianship protocol benefits automatically:

- Validator endorsements use the native signature scheme.
- The `case_hash` uses the native hash scheme.
- No program upgrade is required for the guardianship-specific logic.

## 13. Operational Security

### 13.1 Validator Selection Criteria

Validators for guardianship endorsements should be:

1. **Independent.** No conflicts of interest with the guardian or the subject.
2. **Local.** Familiar with the subject's situation and jurisdiction.
3. **Responsive.** Available to endorse within the 90-day grace period.
4. **Trusted.** Known to the community or AuthorityRegistry.

### 13.2 Court Order Handling

For court-appointed guardianships:

1. The court order document should be stored on IPFS/Arweave (not just the hash).
2. The `case_hash` should be computed as `SHA-256(canonical_form(court_order))`.
3. Validators should verify the `case_hash` against the actual document before endorsing.
4. The court order should specify the scope of guardianship (which parcels, what authority, what duration).

### 13.3 Guardian Scope Enforcement

Since scope enforcement is advisory (not on-chain), the following operational procedures are recommended:

1. **Guardian acknowledgment.** The guardian should sign an off-chain acknowledgment of their scope limitations.
2. **Validator monitoring.** Validators should monitor for scope creep (guardian acting outside their authorized scope).
3. **Revocation triggers.** If the guardian acts outside their scope, the recovery wallet or admin can revoke via `revoke_guardianship`.
4. **Periodic review.** For long-term guardianships, periodic review by validators is recommended (e.g., annually).

### 13.4 Key Management for Minors

For minor subjects:

1. The guardian should manage a dedicated wallet for the minor's parcels.
2. The minor's identity should have a separate recovery wallet (controlled by a trusted third party).
3. When the minor reaches the age of majority, the guardian should call `revoke_guardianship` with `new_owner` set to the minor's new active wallet.

### 13.5 Emergency Procedures

1. **Guardian becomes unavailable.** If the guardian loses their key or becomes unavailable, the recovery wallet (or admin) can revoke and appoint a new guardian.
2. **Subject recovers capacity.** The subject binds a new active wallet and calls `revoke_guardianship` with `new_owner` set to their new wallet.
3. **Court order reversal.** The admin calls `revoke_guardianship` with `new_owner` set to the subject's wallet (or a new guardian).

## 14. Test Vectors

### 14.1 Basic Guardianship Request

- Identity: wallet A (owner), wallet B (recovery)
- Guardian: wallet C
- Validators: wallets D, E, F (3 validators)
- Kind: GUARDIANSHIP (3)
- Grace: 90 days (default)
- Required: 3
- Expected: `SuccessionRequested` event with `kind = 3`, `grace_secs = 7776000`, `required = 3`, `count = 3`

### 14.2 Court-Appointed Guardian Request

- Identity: wallet A (owner), wallet B (recovery)
- Guardian: wallet C
- Validators: wallets D, E, F (3 validators)
- Kind: COURT_APPOINTED_GUARDIAN (4)
- Case hash: `SHA-256("court_order_2026_001")` = `0xa1b2c3...`
- Grace: 90 days
- Required: 3
- Expected: `SuccessionRequested` event with `kind = 4`, `case_hash` embedded

### 14.3 Guardianship Endorsement + Claim

- Pending guardianship: wallet A → wallet C, kind = 3, required = 3
- Validators D, E, F each call `endorse_succession`
- After 90 days, wallet C calls `claim_succession`
- Expected: Identity owner changes from A to C, parcels re-pointed, `SuccessionClaimed` event emitted

### 14.4 Guardianship Cancel During Grace Period

- Pending guardianship: wallet A → wallet C, kind = 3
- Wallet B (recovery) calls `cancel_succession` on day 30
- Expected: Succession account closed, `SuccessionCancelled` event emitted

### 14.5 Insufficient Validators

- Identity: wallet A
- Guardian: wallet C
- Validators: wallets D, E (only 2 validators)
- Kind: GUARDIANSHIP (3)
- Required: 3
- Expected: `InvalidThreshold` error (required > count)

### 14.6 Self-Dealing Prevention

- Identity: wallet A (owner)
- Guardian: wallet A (same as owner)
- Expected: `SuccessorIsOwner` error

### 14.7 Insufficient Grace Period

- Identity: wallet A
- Guardian: wallet C
- Grace: 30 days (< 90-day minimum)
- Kind: GUARDIANSHIP (3)
- Expected: Grace clamped to 90 days (or error if the program rejects sub-minimum grace)

### 14.8 Claim Before Grace Period

- Pending guardianship: wallet A → wallet C, kind = 3, effective_at = 90 days from now
- Wallet C calls `claim_succession` on day 30
- Expected: `SuccessionNotYetEffective` error

### 14.9 Claim Without Enough Endorsements

- Pending guardianship: wallet A → wallet C, kind = 3, required = 3
- Only 2 validators endorsed
- Wallet C calls `claim_succession` after 90 days
- Expected: `InsufficientValidations` error

### 14.10 Revocation After Claim

- Identity: wallet C (guardian has claimed)
- Recovery: wallet B
- Wallet B calls `revoke_guardianship` with `new_owner = wallet A`
- Expected: Identity owner changes from C to A, `GuardianshipRevoked` event emitted

### 14.11 Revocation by Admin

- Identity: wallet C (guardian has claimed)
- Registry admin calls `revoke_guardianship` with `new_owner = wallet D`
- Expected: Identity owner changes from C to D, `GuardianshipRevoked` event emitted

### 14.12 Court-Appointed Without Case Hash

- Identity: wallet A
- Guardian: wallet C
- Kind: COURT_APPOINTED_GUARDIAN (4)
- Case hash: `[0; 32]`
- Expected: `EmptyCaseHash` error

### 14.13 Concurrent Guardianships

- Identity: wallet A
- Guardian 1: wallet C (kind = 3, pending)
- Guardian 2: wallet D (kind = 3, request attempted)
- Expected: Second request succeeds (different `successor` produces different PDA), but only one can claim (the other must be cancelled or expire)

### 14.14 Full Lifecycle

1. `request_succession` with kind = 3, 5 validators, 3 required, 90-day grace
2. Validators D, E, F endorse (3 of 5)
3. 90 days elapse
4. `claim_succession` → identity owner becomes guardian
5. Subject recovers → recovery wallet calls `revoke_guardianship` with `new_owner = subject's new wallet`
6. Identity owner reverts to subject
7. All parcels re-pointed to subject

Expected: All events emitted in order, identity transitions correct, parcels re-pointed correctly.
