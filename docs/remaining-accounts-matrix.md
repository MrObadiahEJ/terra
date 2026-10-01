# TERRA `remaining_accounts` security matrix (P0-REMAINING)

Audit deliverable for the remaining-accounts security pass of the P0 plan.
Scope: every place `terra_registry` reads data out of `ctx.remaining_accounts`
(or out of an unchecked identity slot), what the on-chain contract for that
slot is, whether it is enforced, and what was fixed vs. left open.

**Rules honored:** no architecture redesign, no RRR rework, no `Parcel.owner`
reintroduction, no new `TerraError` variants beyond the RFC-gated appends
(code = 6000 + variant index, 241 variants, append-only). All fixes reuse
existing variants (`6190
RouteAccountMismatch`, `6019 IdentityMismatch`, `6028 NotValidator`).
F1/F2/F6/F7/F8 are behaviour-only (no IDL change); F4 (`judicial_forfeiture`)
and F3 (`open_fraud_review`/`open_appeal_review`) add a `registry` account —
the IDL was regenerated and synced to `terra-web` (F5 needed no IDL change;
`file_dispute` already carried the registry).

---

## 1. Findings summary

| ID | Site | Issue | Severity | Status |
|----|------|-------|----------|--------|
| F1 | `routing.rs` + `fraud_governance.rs` `deser()` | No `ai.owner == &crate::ID` check — a foreign program could forge discriminator+body data (profiles, availability, reputation, capability, committee pools) | High | **Fixed** |
| F2 | `routing.rs` route_task capability-restriction slot | Fail-open: any undeserializable slot counted as "no restriction", so junk could mask an ACTIVE `CapabilityRestriction` | High | **Fixed** |
| F3 | `fraud_governance.rs` `open_fraud_review` / `open_appeal_review` | No registry account in the instruction → committee pool membership is not bounded by `registry.validators` | Medium (residual) | **Fixed** — registry account + membership check on pool entry |
| F4 | `lib.rs` `judicial_forfeiture` | `validators` + `threshold` are instruction args; no registry account → any `MIN_FORFEIT_THRESHOLD` keys colluding with any relay authority can force-transfer a parcel | High (design) | **Fixed** — registry membership binding |
| F5 | `dispute.rs` `file_dispute` | `validators` arg stored without cross-checking `registry.validators` → declared quorum set is unbound at file time | Medium (design) | **Fixed** — registry cross-check at file time |
| F6 | `vault.rs` `deserialize_identity()` | No owner check despite callers' `/// CHECK:` comments → forged `Identity{owner,recovery}` → bogus vaults / future identity-PDA squatting | Medium | **Fixed** |
| F7 | `lib.rs` `attach_parcel` | Identity slot read with no program-owner check; `identity.owner == signer` gate is forgeable → provenance pollution / PDA squatting | Medium | **Fixed** |
| F8 | `lib.rs` `grant_identity_right` | "valid Identity PDA" claimed but never verified (no owner check) → forged identity binds into `IdentityRights` | Medium | **Fixed** |

Minor notes (no action needed):

* `open_fraud_review`'s pool loop is `while i + 1 < len` — a trailing odd
  account is silently ignored (benign: it cannot influence the pool).
* `quorum::verify_quorum_signers` is sound *given the declared set*; the set
  itself is the F5/F4 problem, not the helper.

---

## 2. The six patterns (and the identity slots)

62 textual `remaining_accounts` references in `programs/terra_registry/src`
(56 before RFC-013 Phase B, which added 6: the geometry append/verify paths
plumb `ctx.remaining_accounts` into the **existing** `try_load_quorum_config`
global-`(0,[0,0])` PDA loader — owner-checked, bounded 1..=8, same contract
as `create_claim`, so no new pattern class) distill into 6 read patterns +
the identity-slot readers:

### Pattern 1 — `is_authorized_holder` (lib.rs:170) — 19 call sites — SOLID

| File | Sites |
|------|-------|
| `dispute.rs` | 83 (file_filer gate), 316 |
| `escrow.rs` | 64, 173, 216 |
| `lib.rs` | 812, 838, 866, 918, 966, 1036, 1070 (transfer / update / grant / custody / …) |
| `subdivision.rs` | 122, 214, 221, 281 |
| `time_bound.rs` | 99, 151, 193 |

Contract: rights_kind == OWNERSHIP, parcel match, status ACTIVE, wallet
holder **or** identity overlay — and the overlay path checks
`acc.owner == terra_identity::ID` before reading (lib.rs:194). The
`ownership` argument itself comes from anchor `Account<'info, Rights>` at
every call site, so Anchor enforces program ownership + discriminator.
Covered by the `ownership_invariant` suite (o1–o19), incl. o15 (old holder
rejected post-transfer, 6003) and o16 (forged PDA substitution).

### Pattern 2 — `quorum::verify_quorum_signers` (quorum.rs:43) — 5 sites — SOLID (set-bound is F5/F4)

`dispute.rs:173` (freeze), `dispute.rs:224` (adjudicate),
`lib.rs:1269` (judicial_forfeiture), `validator_registry.rs:654`,
`vault.rs:188` (authorize_vault_access — set = `vault.shard_holders`, bound
at create ✓).

Contract: every slot must be a signer, appear in the declared set, deduped;
count ≥ threshold.

### Pattern 3 — route_task stride (`routing.rs` `route_task`) — F1 + F2 FIXED

Stride = `3 + geo + 2×need_cap + juris` per candidate:
profile, availability, reputation, [presence], [capability, restriction],
[binding].

Contract now enforced per slot:

* profile/availability: owner check + `wallet == cand` (F1)
* reputation: owner check, soft-fail → score 0
* presence/capability: owner check + wallet/code match (hard error)
* restriction: **key must equal the canonical PDA**
  `["capability_restriction", cand, [req.capability_code]]` (F2) — existing
  account or never-created ghost both accepted; junk → 6190.
  *Client note (tx-prep / IDL consumers): when the requirement scopes a
  capability code, derive and pass that PDA for every candidate slot.*
* binding: `binding_slot_ok` already checked owner + PDA before this pass.

### Pattern 4 — fraud committee pool (`fraud_governance.rs`) — F1 + F3 FIXED

Builders: `open_fraud_review` (356–357) and `open_appeal_review` (613–614),
pairs `(profile, reputation)`.

Contract: both desers now owner-checked (F1); `rep.validator == profile.wallet`;
accused/appellant excluded; each remaining candidate's wallet must be in
`registry.validators` (F3: new `registry` account, `NotValidator` 6028 —
membership checked on pool entry, so 6190/6194 ordering is preserved);
`filter_committee_pool` (min reputation) then seeded `select_committee`.

### Pattern 5 — succession iterate (`lib.rs:1192`, `claim_succession`) — SOLID

`require_keys_eq!(info.owner, crate::ID)` before reading each Rights PDA,
plus OWNERSHIP kind + parcel match. This is the template F1/F6–F8 now follow.

### Pattern 6 — `migrate_rights` chunks (`subdivision.rs:305`) — SOLID

`verify_rights_account` (subdivision.rs:75) checks `owner == program_id`
before deser; pairs must be even-length; expected PDA per source account.

### Identity-slot readers (UncheckedAccount, no anchor owner constraint)

| Site | Before | Now |
|------|--------|-----|
| `vault::deserialize_identity` (create_vault:115, endorse_shard_rotation:302) | comment-only ownership claim | owner == `terra_identity::ID` (F6) |
| `attach_parcel` (lib.rs:~1076) | only `identity.owner == signer` | owner check added (F7) |
| `grant_identity_right` (lib.rs:~972) | deser only ("valid Identity PDA" unverified) | owner check added (F8) |

### Completion census — every `UncheckedAccount` slot (both programs)

Full sweep after F1–F8 to confirm no reader was missed:

* `terra_registry` has **50** `UncheckedAccount` fields (lib.rs) + the 8
  audited data-read helpers; `terra_identity` has **1**.
* All **23 raw data-read sites** (`try_borrow_data` / `try_from_slice` /
  `try_deserialize`) are covered: the F1/F6/F7/F8 fixes, Pattern 1
  (`is_authorized_holder` — owner checked at lib.rs:199 before deser),
  Pattern 5 (`claim_succession_with_parcels` — owner checks on identity,
  succession, and `identity_program.key() == terra_identity::ID` before
  deser/CPI; rights PDAs owner-checked in the remaining-accounts loop),
  Pattern 6 (`verify_rights_account` / `migrate_rights`), and
  `task_economics::deser_policy` (owner + empty, pre-existing).
* Every other unchecked slot is a **pubkey target** (`seller`, `buyer`,
  `accused`, `validator`, `authority`, `reporter`, `zone_id`, …): only
  `.key()` is ever read — nothing to forge. No CPI besides the audited
  `claim_succession_with_parcels` invoke.
* Write paths need no check: the runtime rejects data mutation of
  foreign-owned accounts (`ExternalAccountDataModified`).
* ZK: `verify_precompiled_ed25519` requires
  `instructions.key == INSTRUCTIONS_SYSVAR_ID` (6087) before
  `load_instruction_at_checked` — a fake sysvar account cannot be
  substituted.
* `terra_identity`: no `remaining_accounts`; its single `UncheckedAccount`
  (`new_owner` in `RevokeGuardianship`) is never data-read (the handler
  uses the instruction argument), and `ExecuteRevokeGuardianship` demands
  a `Signer` matching `identity.pending_new_owner`.

**Result: no open findings remain in this audit class (F1–F8 all Fixed).**

Other `deser_policy` (task_economics.rs:260) already had `owner + empty`
checks — no change.

---

## 3. What changed (files)

* `programs/terra_registry/src/routing.rs`
  * `deser()` — `require!(ai.owner == &crate::ID, RouteAccountMismatch)` (F1)
  * restriction slot — canonical-PDA `require!` before optional deser (F2)
* `programs/terra_registry/src/fraud_governance.rs`
  * `deser()` — same owner check (F1); covers both pool builders
  * `open_fraud_review` / `open_appeal_review` — every pool entrant (after
    the accused/appellant exclusion) must be in `registry.validators`,
    else `NotValidator` (F3); check sits after the 6190 deser/rep checks
    inside the loop, so existing reject codes are unchanged
* `programs/terra_registry/src/vault.rs`
  * `deserialize_identity()` — `require_keys_eq!(owner, terra_identity::ID,
    IdentityMismatch)` (F6)
* `programs/terra_registry/src/lib.rs`
  * `OpenFraudReview` / `OpenAppealReview` — new `registry` account (seeds
    `validator_registry`), inserted before the signer (F3) → **IDL change**
  * `attach_parcel` — owner check before identity deser (F7)
  * `grant_identity_right` — owner check before identity deser (F8)
  * `judicial_forfeiture` — `registry` account (seeds `validator_registry`)
    + every declared validator must be in `registry.validators` (F4);
    no `judicial_threshold` field exists, so the threshold policy stays
    `>= MIN_FORFEIT_VALIDATORS` — binding membership makes every accepted
    threshold ≥2 a quorum of *registered* validators transitively
  * (earlier P0 batch: stale `rights_count` doc comment corrected)
* `programs/terra_registry/src/dispute.rs`
  * `file_dispute` — every declared quorum validator must be in
    `registry.validators` (F5); check sits after the threshold guards so
    existing reject codes (6016 etc.) are unchanged
* `programs/terra_registry/tests/integration.rs` — adversarial tests (below)
* `tests/terra_registry.ts`, `Makefile` — earlier P0-TEST-01 batch

Soft-fail callers (`.ok()`, `Err(_) => default`) keep optional-account
semantics — they just can no longer read foreign-owned data.

## 4. New adversarial tests (all in `tests/integration.rs`)

| Test | Proves |
|------|--------|
| `remaining_accounts_route_foreign_profile_rejected` | foreign-owned profile → 6190, `assigned_count == 0` |
| `remaining_accounts_fraud_foreign_profile_rejected` | foreign-owned pool profile → 6190, no `ReviewCase` created |
| `remaining_accounts_route_restriction_slot_must_be_pda` | junk restriction slot → 6190 + no assignment; canonical ghost PDA → route assigns (fix is not over-strict) |
| `create_vault_foreign_identity_rejected` | real identity data + foreign owner → 6019, no `VaultRecord` |
| `attach_and_grant_foreign_identity_rejected` | attach → 6019; grant → 6019 + no `IdentityRights` |
| `judicial_forfeiture_rejects_unregistered_validators` | F4: colluding unregistered keys + relay authority → 6028 (`NotValidator`), holder unchanged |
| `judicial_forfeiture_transfers_ownership` (updated) | registered validator quorum still succeeds after the binding |
| `file_dispute_rejects_unregistered_validators` | F5: unregistered declared quorum → 6028, parcel stays REGISTERED, no dispute PDA |
| `open_fraud_review_rejects_unregistered_pool` | F3: well-formed but unregistered pool profiles → 6028, no `ReviewCase`, report stays not-`UNDER_REVIEW` |

Success-path updates: the fraud/pool sites (`phase7_*`, the foreign-profile
test) and dispute sites now pass the new registry account; pool-based
`phase7` tests register their 6 pool validators (4 bootstrap via
`add_validator_ok`, then 2 via `add_validator_consensus_ok` — propose +
endorsements — since the registry flips to PEER_CONSENSUS after 4 adds).

The first five rows each corrupt a real account's owner via
`ctx.set_account` while keeping the data (incl. discriminator) intact — the
exact pre-fix bypass.

## 5. Findings — open recommendations & status

* **F3 — DONE (see §3 / §4):** `OpenFraudReview`/`OpenAppealReview` now
  carry the registry account and `open_fraud_review`/`open_appeal_review`
  reject any pool entrant not in `registry.validators` (test: 6028, no
  partial write). Verified why this mattered: `remove_validator`
  (`validator_registry.rs:370`) only removes the wallet from
  `registry.validators` — it does **not** close or invalidate the
  reputation/profile accounts, so removed validators linger as stale pool
  candidates; F3 now excludes them at pool-entry time.
* **F4 — DONE (see §3 / §4):** `JudicialForfeiture` now takes the
  registry account and rejects any declared validator not in
  `registry.validators` (test: 6028, holder unchanged).
* **F5 — DONE (see §3 / §4):** `file_dispute` now rejects declared
  validators missing from `registry.validators` (test: 6028, parcel
  unchanged). Existing dispute suites register their jurors via
  `add_validator_ok`.

## 6. Verification

```bash
cd terra-core
cargo fmt --check
cargo clippy -- -D warnings              # CI-exact (libs only — CI does not lint test targets)
cargo test -p terra-registry --lib       # 142 unit tests
cargo test -p terra-registry --test integration   # 305 tests (make test: --test-threads=1; CI does not run this suite)
anchor build --skip-lint                 # CI build path (fresh .so needed before running tests)
```

Latest local run: 305/305 integration, 142/142 lib, fmt clean, clippy clean.
TS smoke tests (`make test-ts`) require a local validator — `solana-test-validator`
core-dumps on this machine (no AVX); CI/devnet only.

Tooling note: build via `make build` (build.sh uses the package-manifest
form) or `anchor build --skip-lint`. A bare workspace-root
`cargo build-sbf` resolves features differently and fails compiling
`getrandom 0.3.4` for the SBF target — do not use it.

## 7. P0-ZK follow-up (Ed25519 presentation verification)

Proof presentations for `verify_ownership_proof` / `verify_credential` /
`generate_ownership_root` are now cryptographically verified on-chain —
not just structurally parsed. Design (measured: ed25519-dalek verify costs
>1.4M CU on SBF, so in-program verification is not viable at 500k CU):

* **Verification path:** the transaction carries a Solana Ed25519
  precompile instruction; `zk::verify_precompiled_ed25519` finds it via
  the `Instructions` sysvar (appended as `instructions: UncheckedAccount`
  to all three instruction account structs), checks it targets its own
  instruction (u16::MAX = "current"), and checks pubkey/message/signature
  against what the program expects. The runtime does the actual curve math
  natively (~0 program CU); tx atomicity covers failure. `ed25519-dalek`
  is now a `[dev-dependencies]`-only (test-side signing).
* **Statement formats** (`zk`, RFC-011 §6.2/§6.3 domain-separated,
  length-delimited): root attestation `merkle_root(32) || version(LE u32)`;
  ownership/credential statements are `b"TERRA_ZK_*_V1" || … || purpose(u8
  len) || purpose || disclosure(u8)`, signed by the prover.
* **Error codes:** proof-structure/signature failures → `InvalidProofData`
  (6087); ordering of guards in `lib.rs` unchanged so existing reject
  tests keep their codes (6111/6112/6110/6107 etc.).
* **Tests:** `zk_generate_rejects_invalid_root_attestation`, garbage/rogue/
  tampered-proof attempts (6087), success paths with real signatures, and
  the whole existing zk/credential suite — 11 zk/credential tests within
  the 299 integration run. `assert_custom_error` now matches any
  instruction index (precompile lands at index 0, program at index 1).
* **IDL regenerated** (`make idl` + build.sh copy step): new
  `authority_signature` arg, `instructions` account on the three structs —
  synced to `terra-web/src/idl/`.
* **P0-ZK-01 (2026-10-01) — framed Groth16 layer** on top of the above:
  ownership `proof_data` must now be the 901-byte frame of RFC-011 §6.3.1
  (`TG16` magic + version + Ed25519 statement signature + 576-byte
  verification key + 256-byte Groth16 proof), all in `zk/groth16.rs`.
  The VK is hash-pinned on the root (`update_verification_key_hash`
  before first proof); the pairing check runs through `solana-bn254`
  `alt_bn128` syscalls with public input `SHA-256(statement) mod r`
  (~99k CU full verify); every proof-layer failure maps to
  `InvalidProofData` (6087) after the existing guards, so the codes
  above are unchanged. Verified by the framed happy path plus
  `zk_groth16_verification_rejections` (corrupted proof byte,
  cross-statement proof splice, corrupted VK, wrong pinned hash — each
  rejected). Test harness `send_tx` additionally guards against banks
  signature-dedup phantom results (identical tx inside one ~6.4 ms
  blockhash window would otherwise return the cached status without
  executing).
