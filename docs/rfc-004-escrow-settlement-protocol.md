# RFC-004: Escrow Settlement Protocol

## 1. Status

- **Status:** Draft
- **Created:** 2026-09-02
- **Supersedes:** None
- **Depends on:** RFC-003 (Vault Shard Protocol), RFC-007 (Dispute Resolution & Parcel Freeze)

## 2. Summary

This RFC specifies the **Escrow Settlement Protocol** — an on-chain escrow mechanism for SOL-denominated parcel sales. A parcel marked `FOR_SALE` currently has no on-chain payment state: buyer and seller must trust each other or a middleman, with no atomic "money flips, title flips" guarantee.

The escrow protocol closes this gap by introducing a stateful, time-locked escrow flow: the seller deposits the parcel into escrow, the buyer deposits SOL, the seller accepts payment to trigger a settlement window, and after the window expires the atomic swap executes — parcel ownership transfers to the buyer, funds transfer to the seller. Disputes at any stage route into RFC-007's `FROZEN` state rather than a bespoke dispute path.

The on-chain program never holds plaintext keys or performs currency conversion. The escrow vault holds SPL-wrapped SOL (wSOL) for composability. The design is deliberately minimal: only hashes, thresholds, and state transitions go on-chain.

## 3. Threat Model

### 3.1 Adversary Classes

| Class | Description | Mitigation |
|-------|-------------|------------|
| **Seller rug** | Seller accepts payment, refuses to transfer title | Atomic settle: ownership transfer and fund release are a single transaction |
| **Buyer rug** | Buyer deposits, seller never accepts, funds locked | Time-locked cancel: buyer can reclaim within grace period; seller can cancel before deposit |
| **Front-running** | Third party observes escrow and attempts to interfere | Escrow PDA is bound to parcel key; only designated parties can act |
| **Collusion** | Buyer and seller collude to defraud the system | Self-dealing guard: buyer cannot equal seller; validator cannot be either party |
| **Griefing** | Bad actor creates escrows to lock parcels indefinitely | Time-locked expiry: escrows auto-cancel if not settled within deadline |
| **Oracle manipulation** | Currency conversion introduces "who do we trust" | Out of scope for v1: SOL-denominated only, no oracle |
| **Validator capture** | Validator is also a party to the escrow | Self-dealing guard: validator cannot be buyer or seller |
| **Replay attack** | Replaying an old escrow instruction | PDA is unique per parcel; nonce and timestamp binding |

### 3.2 In Scope

- SOL-denominated parcel escrow (deposit → accept → time-locked settle)
- Earnest money deposits (partial escrow)
- Atomic settlement (parcel + funds swap in one transaction)
- Grace period cancellation (buyer's right to withdraw)
- Dispute routing to RFC-007 `FROZEN` state
- Self-dealing prevention (buyer ≠ seller, validator ≠ either party)

### 3.3 Out of Scope

- Stablecoin or local-currency conversion (oracle risk reintroduces "who do we trust")
- Installment tracking or payment plans
- Multi-parcel batch escrows
- Cross-chain escrow or bridging
- Non-SOL denominations (v2 concern)

## 4. Cryptographic Choices

### 4.1 Payment Token: SPL Wrapped SOL (wSOL)

- Native SOL is wrapped into an SPL Token for composability with the SPL ecosystem
- wSOL is a well-audited, widely-deployed token program on Solana
- Wrapping/unwrapping is handled at the wallet level; the escrow program operates on wSOL accounts only
- Rationale: SPL Token transfers are atomic within a single instruction; native SOL transfers require separate lamport accounting

### 4.2 Hashing: SHA-256

- On-chain escrow record hash for integrity verification
- Event log correlation via `escrow_hash = SHA-256(parcel_key || seller || buyer || amount)`
- Case hash binding for disputes routed to RFC-007

### 4.3 Signatures: Ed25519

- Solana native — all wallet keys are Ed25519
- Used for transaction signing (on-chain authorization)
- Used for off-chain protocol messages (buyer/seller coordination)

### 4.4 Clock: Solana Block Timestamp

- `Clock::get()?.unix_timestamp` used for all time-locked operations
- Settlement window, grace period, and dispute expiry are all derived from block time
- Not cryptographically secure randomness — only used for temporal ordering

## 5. Data Model

### 5.1 EscrowRecord (on-chain PDA)

**PDA seed:** `["escrow", parcel_key]`

One escrow per parcel. A parcel in escrow cannot be in another escrow simultaneously.

| Field | Type | Description |
|-------|------|-------------|
| `parcel` | `Pubkey` | The parcel being sold |
| `seller` | `Pubkey` | Wallet of the seller (must match `parcel.owner`) |
| `buyer` | `Pubkey` | Wallet of the buyer (set at deposit time) |
| `amount` | `u64` | Sale price in lamports (wSOL) |
| `deposit_amount` | `u64` | Earnest money deposited by buyer (partial or full) |
| `vault` | `Pubkey` | EscrowVault PDA holding the wSOL |
| `status` | `u8` | Current escrow status (see `escrow_status`) |
| `created_at` | `i64` | When escrow was created |
| `deposited_at` | `i64` | When buyer deposited (0 if not yet deposited) |
| `accepted_at` | `i64` | When seller accepted (triggers settlement window) |
| `settle_deadline` | `i64` | accepted_at + SETTLEMENT_WINDOW_SECS |
| `cancel_deadline` | `i64` | created_at + CANCEL_WINDOW_SECS |
| `dispute_case_hash` | `[u8; 32]` | Case hash if dispute filed (0 if none) |

### 5.2 EscrowVault (on-chain PDA)

**PDA seed:** `["escrow_vault", escrow_key]`

Holds the wSOL tokens during the escrow period. Owned by the PDA so only the escrow program can move funds.

| Field | Type | Description |
|-------|------|-------------|
| `escrow` | `Pubkey` | EscrowRecord PDA key |
| `amount` | `u64` | Total wSOL held (deposit + any additional) |

### 5.3 Escrow Status Constants

```rust
pub mod escrow_status {
    pub const CREATED: u8 = 0;       // Seller initiated, awaiting buyer deposit
    pub const DEPOSITED: u8 = 1;     // Buyer deposited, awaiting seller acceptance
    pub const ACCEPTED: u8 = 2;      // Seller accepted, settlement window ticking
    pub const SETTLED: u8 = 3;       // Atomic swap executed
    pub const CANCELLED: u8 = 4;     // Cancelled by either party
    pub const DISPUTED: u8 = 5;      // Dispute filed → routes to RFC-007
}
```

### 5.4 Constants

```rust
pub const SETTLEMENT_WINDOW_SECS: i64 = 3 * 24 * 3600;   // 3 days
pub const CANCEL_WINDOW_SECS: i64 = 7 * 24 * 3600;       // 7 days (buyer grace)
pub const MAX_ESCROW_AMOUNT: u64 = 1_000_000_000_000;    // 1M SOL in lamports
pub const MIN_ESCROW_AMOUNT: u64 = 100_000_000;          // 0.1 SOL in lamports
```

### 5.5 Status Transition Diagram

```
CREATED ──[deposit_escrow]──► DEPOSITED ──[accept_escrow]──► ACCEPTED ──[settle_escrow]──► SETTLED
   │                              │                              │
   │                              │                              │
   └──[cancel_escrow]──► CANCELLED ◄──[cancel_escrow]──┘  ┌────┘
                                                          │
                                                   [dispute_escrow]──► DISPUTED ──► RFC-007 FROZEN
```

## 6. Instructions

### 6.1 `create_escrow`

**Purpose:** Seller initiates an escrow for a parcel marked `FOR_SALE`.

**Accounts:**
- `escrow_record` (init, PDA `["escrow", parcel.key()]`)
- `parcel` (mut — must be `FOR_SALE`)
- `seller` (signer, mut — must be `parcel.owner`)
- `system_program`

**Args:** `amount: u64`, `buyer: Pubkey`

**Guards:**
- `parcel.status == parcel_status::FOR_SALE`
- `seller.key() == parcel.owner`
- `buyer != Pubkey::default()` (non-zero)
- `buyer != seller` (self-dealing prevention)
- `amount >= MIN_ESCROW_AMOUNT && amount <= MAX_ESCROW_AMOUNT`
- No existing escrow for this parcel (PDA does not already exist)

**Effects:**
- `escrow_record.parcel = parcel.key()`
- `escrow_record.seller = seller.key()`
- `escrow_record.buyer = buyer`
- `escrow_record.amount = amount`
- `escrow_record.deposit_amount = 0`
- `escrow_record.status = escrow_status::CREATED`
- `escrow_record.created_at = now`
- `escrow_record.cancel_deadline = now + CANCEL_WINDOW_SECS`
- `escrow_record.settle_deadline = 0`

**Emits:** `EscrowCreated { escrow, parcel, seller, buyer, amount, created_at }`

### 6.2 `deposit_escrow`

**Purpose:** Buyer deposits wSOL into the escrow vault. Can be partial (earnest money) or full.

**Accounts:**
- `escrow_record` (mut)
- `escrow_vault` (init, PDA `["escrow_vault", escrow.key()]`, associated token account for wSOL)
- `parcel` (readonly)
- `buyer` (signer, mut — must match `escrow.buyer`)
- `buyer_token_account` (mut — buyer's wSOL account)
- `escrow_vault_token_account` (mut — escrow vault's wSOL account)
- `token_program` (SPL Token)
- `system_program`

**Args:** `deposit_amount: u64`

**Guards:**
- `escrow_record.status == escrow_status::CREATED`
- `buyer.key() == escrow_record.buyer`
- `deposit_amount > 0`
- `escrow_record.deposit_amount + deposit_amount <= escrow_record.amount`
- `Clock::get()?.unix_timestamp < escrow_record.cancel_deadline` (still within cancel window)

**Effects:**
- `escrow_record.deposit_amount += deposit_amount`
- `escrow_record.deposited_at = now`
- Transfers `deposit_amount` wSOL from `buyer_token_account` to `escrow_vault_token_account`
- If `escrow_record.deposit_amount >= escrow_record.amount`: `escrow_record.status = DEPOSITED`

**Emits:** `EscrowDeposited { escrow, parcel, buyer, deposit_amount, total_deposited }`

### 6.3 `accept_escrow`

**Purpose:** Seller accepts the deposited payment, triggering the settlement window. The buyer now has `SETTLEMENT_WINDOW_SECS` to call `settle_escrow`.

**Accounts:**
- `escrow_record` (mut)
- `parcel` (readonly)
- `seller` (signer — must match `escrow.seller` and `parcel.owner`)

**Args:** None

**Guards:**
- `escrow_record.status == escrow_status::DEPOSITED`
- `seller.key() == escrow_record.seller`
- `seller.key() == parcel.owner`
- `escrow_record.deposit_amount >= escrow_record.amount` (full payment received)

**Effects:**
- `escrow_record.status = escrow_status::ACCEPTED`
- `escrow_record.accepted_at = now`
- `escrow_record.settle_deadline = now + SETTLEMENT_WINDOW_SECS`

**Emits:** `EscrowAccepted { escrow, parcel, seller, accepted_at, settle_deadline }`

### 6.4 `settle_escrow`

**Purpose:** After the settlement window, execute the atomic swap: parcel ownership transfers to buyer, wSOL transfers to seller. This is the core "money flips, title flips" guarantee.

**Accounts:**
- `escrow_record` (mut, close → seller)
- `escrow_vault` (mut, close → seller)
- `escrow_vault_token_account` (mut — escrow vault's wSOL account)
- `parcel` (mut — ownership transfers)
- `seller_token_account` (mut — seller's wSOL account)
- `settler` (signer — can be buyer, seller, or any third party)

**Args:** None

**Guards:**
- `escrow_record.status == escrow_status::ACCEPTED`
- `Clock::get()?.unix_timestamp >= escrow_record.settle_deadline` (settlement window expired)
- `escrow_record.deposit_amount >= escrow_record.amount` (full payment confirmed)
- `parcel.owner == escrow_record.seller` (seller still owns the parcel)

**Effects:**
- `parcel.owner = escrow_record.buyer` (title transfer)
- `parcel.status = parcel_status::TRANSFERRED`
- `parcel.updated_at = now`
- Transfers `escrow_record.amount` wSOL from `escrow_vault_token_account` to `seller_token_account`
- Returns remaining deposit (if partial) to buyer's wSOL account
- `escrow_record.status = escrow_status::SETTLED`
- Close `escrow_record` account (lamports → seller)
- Close `escrow_vault` account (lamports → seller)

**Emits:** `EscrowSettled { escrow, parcel, seller, buyer, amount, settled_at }`

### 6.5 `cancel_escrow`

**Purpose:** Cancel the escrow and return funds. Two paths:

1. **Before buyer deposits (CREATED status):** Seller can cancel at any time.
2. **After buyer deposits (DEPOSITED status):** Buyer can cancel within the grace period (`CANCEL_WINDOW_SECS` from escrow creation).

**Accounts:**
- `escrow_record` (mut, close → canceller)
- `escrow_vault` (mut, close → canceller) — only if funds are held
- `parcel` (mut — status reset to `FOR_SALE`)
- `canceller` (signer, mut — either the seller or buyer)
- `system_program`

**Args:** None

**Guards (seller cancel — CREATED status):**
- `escrow_record.status == escrow_status::CREATED`
- `canceller.key() == escrow_record.seller`
- No deposit has been made (`deposit_amount == 0`)

**Guards (buyer cancel — DEPOSITED status):**
- `escrow_record.status == escrow_status::DEPOSITED`
- `canceller.key() == escrow_record.buyer`
- `Clock::get()?.unix_timestamp < escrow_record.cancel_deadline` (within grace period)

**Effects:**
- `parcel.status = parcel_status::FOR_SALE` (reset)
- `parcel.updated_at = now`
- If funds held: transfers `deposit_amount` wSOL back to buyer's wSOL account
- `escrow_record.status = escrow_status::CANCELLED`
- Close `escrow_record` account
- Close `escrow_vault` account (if it existed)

**Emits:** `EscrowCancelled { escrow, parcel, cancelled_by, reason }`

### 6.6 `dispute_escrow`

**Purpose:** Either party triggers a dispute, routing the parcel into RFC-007's `FROZEN` state. The escrow is frozen; settlement and cancellation are blocked until the dispute is adjudicated.

**Accounts:**
- `escrow_record` (mut)
- `parcel` (mut — status transitions to `DISPUTED` then `FROZEN`)
- `dispute` (init, PDA `["dispute", parcel.key(), case_hash]` — RFC-007 dispute account)
- `filer` (signer, mut — buyer or seller)

**Args:** `case_hash: [u8; 32]`, `required: u8`, `validators: [Pubkey; MAX_VALIDATORS]`

**Guards:**
- `escrow_record.status` is `CREATED`, `DEPOSITED`, or `ACCEPTED` (not already settled/cancelled/disputed)
- `filer.key() == escrow_record.seller || filer.key() == escrow_record.buyer`
- `case_hash != [0; 32]`
- `required >= MIN_DISPUTE_VALIDATORS` (reuses RFC-007's minimum)
- All validators pass self-dealing checks (not buyer, not seller, not filer)

**Effects:**
- `escrow_record.status = escrow_status::DISPUTED`
- `escrow_record.dispute_case_hash = case_hash`
- `parcel.status = parcel_status::DISPUTED`
- `parcel.updated_at = now`
- Creates RFC-007 `Dispute` account bound to this parcel
- Dispute proceeds through RFC-007's freeze → adjudicate → execute flow

**Emits:** `EscrowDisputed { escrow, parcel, filer, case_hash, dispute }`

## 7. Off-Chain Protocol

### 7.1 Normal Sale Flow

1. **Seller lists parcel:** Seller calls `update_status` to set parcel to `FOR_SALE`.
2. **Seller creates escrow:** Seller calls `create_escrow` with the sale price and buyer's wallet.
3. **Buyer deposits:** Buyer wraps SOL into wSOL, calls `deposit_escrow` (partial or full).
4. **Seller accepts:** Seller verifies deposit, calls `accept_escrow`. Settlement window begins (3 days).
5. **Settlement:** After the window expires, anyone calls `settle_escrow`. Atomic swap executes:
   - Parcel ownership → buyer
   - wSOL → seller
   - Both accounts closed, rent returned.
6. **Post-settlement:** Buyer updates parcel status if needed. Seller unwraps wSOL to SOL.

### 7.2 Early Cancel Flow (Buyer's Grace Period)

1. Seller creates escrow.
2. Buyer deposits but has second thoughts.
3. Buyer calls `cancel_escrow` within the `CANCEL_WINDOW_SECS` (7 days from creation).
4. wSOL returned to buyer. Parcel status reset to `FOR_SALE`.
5. Escrow accounts closed.

### 7.3 Seller Cancel Flow

1. Seller creates escrow.
2. Before buyer deposits, seller decides not to sell.
3. Seller calls `cancel_escrow`. No funds to return.
4. Parcel status reset to `FOR_SALE`. Escrow account closed.

### 7.4 Dispute Flow

1. Either party files a dispute via `dispute_escrow` with a `case_hash` (SHA-256 of the off-chain complaint).
2. Parcel transitions to `DISPUTED` status (RFC-007 handles the rest).
3. Validators co-sign the dispute via RFC-007's `freeze_parcel`.
4. Adjudicator rules via RFC-007's `adjudicate_dispute`.
5. If owner wins: parcel returns to `REGISTERED`, escrow can be re-initiated.
6. If owner loses: parcel forfeited to new owner per court order.
7. The escrow remains in `DISPUTED` status until the dispute is resolved. No settlement or cancellation is possible while disputed.

### 7.5 Expiry / Timeout Flow

1. Seller creates escrow with a `cancel_deadline` (7 days from creation).
2. Buyer never deposits within the window.
3. `cancel_deadline` passes. The escrow is stale.
4. Seller calls `cancel_escrow`. The cancel guard checks `now < cancel_deadline` — this would FAIL.
5. **Resolution:** An expiry instruction `expire_escrow` allows the seller (or anyone) to close an expired escrow after the cancel deadline. This is a safety valve, not a normal path.

**Note:** The `expire_escrow` instruction is a lightweight cleanup. It does not transfer funds (none were deposited). It closes the escrow account and resets the parcel to `FOR_SALE`.

### 7.6 Side-Channel Coordination

- **Payment verification:** Buyer and seller coordinate off-chain (bank transfer, mobile money, cash). The on-chain escrow only tracks wSOL.
- **Dispute evidence:** Off-chain court documents, photographs, or correspondence. Only the `case_hash` goes on-chain.
- **Validator coordination:** For dispute resolution, validators coordinate via RFC-007's established channels.

## 8. Collusion Resistance

### 8.1 Self-Dealing Prevention

The most critical anti-collusion guard:

- **Buyer ≠ Seller:** Enforced at `create_escrow` time. The same wallet cannot be both parties.
- **Validator ≠ Party:** Validators co-signing disputes cannot be the buyer, seller, or filer. Enforced at `dispute_escrow` time.
- **Seller cannot self-forfeit via dispute:** If the seller files a dispute and the adjudicator rules against them, the parcel is forfeited to a new owner — not returned to the seller.

### 8.2 Time-Locked Settlement

The settlement window (3 days) creates a cooling-off period:

- The buyer can observe the blockchain and detect if the seller attempts to transfer the parcel to someone else before settlement.
- If the seller transfers the parcel away, `settle_escrow` will fail because `parcel.owner != escrow.seller`.
- The buyer can then dispute via RFC-007.

### 8.3 Grace Period for Buyer Protection

The 7-day cancel window protects the buyer from:

- Seller creating escrow with a compromised buyer wallet
- Buyer making an accidental deposit
- Change of mind before the deal solidifies

After the grace period, the buyer must actively cancel before the seller accepts — once accepted, only settlement or dispute can resolve the escrow.

### 8.4 No Oracle Dependency

By denominating exclusively in SOL (wSOL), we eliminate:

- Oracle manipulation risk
- Currency conversion disputes
- "Who do we trust" for price feeds
- Flash loan attacks on price oracles

## 9. Liveness Guarantees

### 9.1 Settlement Window Deadline

The settlement window is bounded to `SETTLEMENT_WINDOW_SECS` (3 days). If neither party acts:

- After the window, anyone can call `settle_escrow` to execute the atomic swap.
- The seller is incentivized to let it settle (they receive funds).
- The buyer is incentivized to let it settle (they receive the parcel).

### 9.2 Cancel Window Deadline

The cancel window is bounded to `CANCEL_WINDOW_SECS` (7 days). If the buyer never deposits:

- After the window, the seller can call `cancel_escrow` (or `expire_escrow`).
- The seller's parcel is not permanently locked.

### 9.3 Dispute Resolution Path

Disputes route into RFC-007, which has its own liveness guarantees:

- Disputes auto-expire after 90 days (`DISPUTE_EXPIRY_SECS`).
- Adjudication requires validator endorsement.
- Execution can be triggered by either party.

### 9.4 Stale Escrow Cleanup

Escrows in `CREATED` status with an expired `cancel_deadline` can be cleaned up via `expire_escrow`. This prevents orphaned escrow accounts from consuming rent indefinitely.

## 10. Storage Architecture

### 10.1 On-Chain State

- **EscrowRecord PDA:** `["escrow", parcel_key]` — ~200 bytes
- **EscrowVault PDA:** `["escrow_vault", escrow_key]` — ~50 bytes
- **EscrowVault Token Account:** SPL Token account for wSOL — ~165 bytes

Total per escrow: ~415 bytes on-chain.

### 10.2 Fund Custody

- wSOL is held in an SPL Token account owned by the `escrow_vault` PDA.
- Only the escrow program can sign transfers from this account (CPI via `token_program`).
- The vault PDA's seed includes the escrow key, so each escrow has its own isolated vault.

### 10.3 Account Lifecycle

| Phase | Account | State |
|-------|---------|-------|
| Create | EscrowRecord | Init'd, rent paid by seller |
| Deposit | EscrowVault | Init'd, rent paid by buyer |
| Settle | Both | Closed, rent returned to seller |
| Cancel | Both | Closed, rent returned to canceller |
| Dispute | EscrowRecord stays open | Dispute account init'd via RFC-007 |

### 10.4 Rent Economics

- EscrowRecord rent: paid by seller at creation
- EscrowVault rent: paid by buyer at deposit
- On settle/cancel, accounts are closed and rent returned
- If dispute is filed, escrow stays open until dispute resolves — rent is locked until then

## 11. Replay / Nonce Hygiene

### 11.1 PDA Uniqueness

Each escrow is uniquely derived from the parcel key: `PDA("escrow", parcel_key)`. A parcel can only have one active escrow at a time. This prevents:

- Multiple escrows for the same parcel
- Replay of an old `create_escrow` for a parcel that has already been sold

### 11.2 Timestamp Binding

All time-sensitive operations (`settle_deadline`, `cancel_deadline`) are derived from the block timestamp at instruction execution time. An attacker cannot:

- Extend a deadline by replaying an old transaction
- Shorten a deadline by front-running

### 11.3 Parcel Owner Check at Settlement

`settle_escrow` verifies `parcel.owner == escrow.seller` at execution time. If the seller has transferred the parcel away since creating the escrow, settlement fails. This is intentional — it prevents settlement to a buyer when the seller no longer has the parcel.

### 11.4 Escrow Status Guard

Every instruction checks the current `escrow_record.status` before acting. A settled or cancelled escrow cannot be re-settled or re-cancelled. The status is the single source of truth.

## 12. Post-Quantum Migration Path

### 12.1 Ed25519 Signatures

All transaction signatures use Ed25519, which is vulnerable to Shor's algorithm on a quantum computer. For the pilot (1-2 years), this is acceptable:

- Ed25519 is fast, well-audited, and native to Solana
- Quantum computers capable of breaking Ed25519 are not yet practical
- Migration path exists via Solana's ed25519-ml-dsa (ML-DSA) proposal

### 12.2 wSOL Token Program

SPL Token is not affected by post-quantum concerns — it's a program logic layer, not a cryptographic primitive. Token transfers remain valid regardless of signature scheme.

### 12.3 Migration Strategy

When Solana introduces post-quantum signature support:

1. New escrow instructions accept ML-DSA signatures
2. Existing escrows with Ed25519 signatures can still be settled (they're already in-flight)
3. New escrows use the new signature scheme by default
4. No program upgrade required — the instruction set is extended, not replaced

## 13. Operational Security

### 13.1 Seller Security

- **Before listing:** Verify the buyer's wallet address out-of-band. A wrong address means funds go to an unrecoverable destination.
- **After acceptance:** Do not transfer the parcel to anyone else before settlement. The `settle_escrow` guard checks `parcel.owner == escrow.seller`.
- **Fund receipt:** After settlement, unwrap wSOL to SOL promptly. wSOL earns no yield.

### 13.2 Buyer Security

- **Before depositing:** Verify the parcel is legitimately owned by the seller and is in `FOR_SALE` status.
- **Within grace period:** Use the 7-day cancel window to verify the deal terms. Cancel if anything looks wrong.
- **After deposit:** Monitor the blockchain. If the seller attempts to transfer the parcel, the settlement will fail — be ready to dispute.
- **After settlement:** The parcel is yours. Update status if needed.

### 13.3 Validator Security (Disputes)

- **Co-signing disputes:** Verify the dispute is legitimate before co-signing. Co-signing a frivolous dispute wastes validator reputation.
- **Self-dealing:** Validators must not be the buyer, seller, or filer. This is enforced on-chain, but validators should also self-check off-chain.
- **Evidence preservation:** Keep copies of off-chain dispute evidence for the duration of the dispute.

### 13.4 Wallet Security

- Use a hardware wallet (Ledger, Trezor) for escrow transactions.
- Never share private keys or seed phrases.
- For high-value parcels, use a multisig wallet as the buyer or seller.

## 14. Test Vectors

### 14.1 Create Escrow — Happy Path

- **Seller:** wallet A (parcel owner)
- **Buyer:** wallet B
- **Parcel:** key `P1`, status `FOR_SALE`
- **Amount:** 10 SOL (10_000_000_000 lamports)
- **Expected:** `EscrowRecord` created with status `CREATED`, seller=A, buyer=B, amount=10 SOL, `cancel_deadline = now + 7 days`

### 14.2 Deposit — Full Amount

- **Escrow:** status `CREATED`, amount 10 SOL
- **Buyer:** wallet B
- **Deposit:** 10 SOL (full amount)
- **Expected:** `EscrowRecord.deposit_amount = 10 SOL`, status → `DEPOSITED`, `EscrowVault` created holding 10 SOL wSOL

### 14.3 Deposit — Partial (Earnest Money)

- **Escrow:** status `CREATED`, amount 10 SOL
- **Buyer:** wallet B
- **Deposit:** 2 SOL (partial)
- **Expected:** `EscrowRecord.deposit_amount = 2 SOL`, status remains `CREATED`

### 14.4 Accept Escrow

- **Escrow:** status `DEPOSITED`, deposit_amount = 10 SOL
- **Seller:** wallet A
- **Expected:** status → `ACCEPTED`, `settle_deadline = now + 3 days`

### 14.5 Settle Escrow — Happy Path

- **Escrow:** status `ACCEPTED`, settle_deadline = 3 days ago
- **Parcel:** owner = wallet A
- **Expected:** `parcel.owner = wallet B`, `parcel.status = TRANSFERRED`, wSOL transferred to seller, both escrow accounts closed

### 14.6 Settle Escrow — Parcel Transferred Away

- **Escrow:** status `ACCEPTED`, settle_deadline passed
- **Parcel:** owner = wallet C (seller transferred to someone else)
- **Expected:** `ParcelStillOwnedBySeller` error — settlement fails

### 14.7 Cancel — Seller Before Deposit

- **Escrow:** status `CREATED`, deposit_amount = 0
- **Seller:** wallet A
- **Expected:** status → `CANCELLED`, parcel status reset to `FOR_SALE`, escrow account closed

### 14.8 Cancel — Buyer Within Grace Period

- **Escrow:** status `DEPOSITED`, created 2 days ago (within 7-day window)
- **Buyer:** wallet B
- **Expected:** status → `CANCELLED`, 10 SOL wSOL returned to buyer, parcel status reset to `FOR_SALE`, both accounts closed

### 14.9 Cancel — Buyer After Grace Period

- **Escrow:** status `DEPOSITED`, created 8 days ago (past 7-day window)
- **Buyer:** wallet B
- **Expected:** `CancelWindowExpired` error — buyer cannot cancel after grace period

### 14.10 Self-Dealing Prevention

- **Seller:** wallet A
- **Buyer:** wallet A (same as seller)
- **Expected:** `SelfDealingNotAllowed` error at `create_escrow`

### 14.11 Validator Self-Dealing in Dispute

- **Escrow:** seller=A, buyer=B
- **Filer:** wallet A (seller)
- **Validator 1:** wallet A (seller) — **self-dealing**
- **Validator 2:** wallet C
- **Expected:** `ValidatorOwnsAsset` error at `dispute_escrow`

### 14.12 Dispute — Happy Path

- **Escrow:** status `ACCEPTED`, settle_deadline not yet passed
- **Filer:** wallet B (buyer)
- **Case hash:** SHA-256 of "breach_of_contract_42"
- **Validators:** wallets D, E, F (not buyer or seller)
- **Required:** 2
- **Expected:** `escrow.status = DISPUTED`, `parcel.status = DISPUTED`, RFC-007 `Dispute` account created

### 14.13 Dispute — Filers Cannot Be Their Own Validator

- **Filer:** wallet B (buyer)
- **Validator 1:** wallet B (buyer) — **self-dealing**
- **Expected:** `ValidatorOwnsAsset` error

### 14.14 Expire Escrow — Stale Cleanup

- **Escrow:** status `CREATED`, `cancel_deadline` = 8 days ago
- **Caller:** wallet A (seller) or anyone
- **Expected:** escrow account closed, parcel status reset to `FOR_SALE`

### 14.15 Deposit Exceeds Amount

- **Escrow:** status `CREATED`, amount = 10 SOL, deposit_amount = 8 SOL
- **Buyer:** wallet B
- **Deposit:** 5 SOL (would exceed amount: 8 + 5 = 13 > 10)
- **Expected:** `DepositExceedsAmount` error

### 14.16 Accept Without Full Deposit

- **Escrow:** status `DEPOSITED`, deposit_amount = 5 SOL, amount = 10 SOL
- **Seller:** wallet A
- **Expected:** `InsufficientDeposit` error — seller cannot accept partial payment as full

### 14.17 Wrong Buyer Attempts Deposit

- **Escrow:** status `CREATED`, buyer = wallet B
- **Signer:** wallet C (not the designated buyer)
- **Expected:** `NotDesignatedBuyer` error

### 14.18 Wrong Seller Attempts Accept

- **Escrow:** status `DEPOSITED`, seller = wallet A
- **Signer:** wallet C (not the designated seller)
- **Expected:** `NotDesignatedSeller` error
