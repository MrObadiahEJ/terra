# Terra Registry Security Audit Report

**Date:** 2026-09-14 (original audit); **updated:** 2026-09-23  
**Programs:** `terra_registry` (GaEDbktvpZ3qiqp4PmFgHwDSa6JsFfVjXFqNb2nTbage), `terra_identity` (68urV9nGcRcoWT1QjzZfXuCnTS9921x2se1SybKJr1U4)  
**Auditor:** Automated review (opencode); Phase 1 remediation applied on `dev`

---

## Summary (current status)

| Severity | Original | Open now | Fixed / mitigated |
|----------|----------|----------|-------------------|
| Critical | 9 | 0 | 9 |
| High | 3 | 0 | 3 |
| Medium | 4 | 1 (M-2 residual) | 3 |
| Low | 3 | 2 (L-1, L-3 residual) | 1 |

**Phase 0 / Phase 1** (RFC-012) closed the original Critical and High findings and most Medium/Low items. Remaining items are residual/low-impact and documented below.

---

## Critical Findings — all closed

### C-1: `is_authorized_owner()` trusts unverified `remaining_accounts` data — FIXED

**File:** `lib.rs` (`is_authorized_owner`)

Now requires `acc.owner == &terra_identity::ID` before deserializing `IdentityRights` and `Identity`, and checks `identity.owner == signer_key`. Fake accounts from a foreign program cannot spoof the identity path.

### C-2: No signer on `JailValidator`, `UnjailValidator`, `SlashValidator` — FIXED

All three contexts now have `authority: Signer` with `constraint = authority.key() == registry.admin @ TerraError::NotAuthorized`.

### C-3: No signer on `RecordAttestationOutcome` — FIXED

`authority: Signer` constrained to `registry.admin`.

### C-4: No signer on `RecordSessionEvidence` / `RecordSessionObservation` — FIXED (residual: signer role not fully constrained)

Both contexts now require `signer: Signer` (PDA-seeded session). The session is PDA-validated; the handler rejects terminal sessions. **Residual:** the signer is not yet required to equal `session.opened_by` or be a registry validator in the constraint — any wallet can increment counters on a non-terminal session if it passes the PDA. Treat as Medium residual (see M-2 pattern below for related session hygiene).

### C-5: No authority check on `ResolveGuardianClaim` / `DisputeGuardianClaim` — FIXED

Both require `caller.key() == guardian_claim.triggered_by || caller.key() == registry.admin`.

### C-6: No authority check on `AdjudicateDispute` — FIXED

`authority` constrained to `registry.admin`.

### C-7: No authority check on `VerifyCrossBorder` — FIXED

`caller` constrained to `registry.admin` or `registry.validators.contains(&caller.key())`.

### C-8: Missing account constraints on `AuthorizeVaultAccess` — FIXED

`vault_record` uses `seeds = [b"vault_record", subject.key()]` and `constraint = vault_record.subject == subject.key()`. Handler also re-checks subject match and requires `authority` ∈ `vault.shard_holders` with quorum via `verify_quorum_signers`.

### C-9: Missing account constraints on credential contexts — FIXED

`SignCredential`, `FinalizeCredential`, `VerifyCredential` all use PDA seeds (`credential_request`, `threshold_credential`, `credential_nullifier`). `sign_credential` additionally requires `registry.validators.contains(&signer_key)`.

---

## High Findings — all closed

### H-1: `try_load_reputation()` / `try_load_session()` deserialize without ownership check — FIXED

Both helpers (and `try_load_quorum_config`) now `require!(acc.owner == &crate::ID, TerraError::NotAuthorized)` before deserialize. Observation remaining-accounts loop skips non-owned accounts.

### H-2: ZK instructions have no authority check — FIXED

`RegisterZoneSet`, `GenerateOwnershipRoot`, `InvalidateProof`, `UpdateVerificationKeyHash` all constrain `authority.key() == registry.admin`.

### H-3: `InitializeValidatorReputation` has no authority check — FIXED

`payer` constrained to `registry.admin`.

---

## Medium Findings

### M-1: `SweepExpiredRights` has no access control — FIXED

`parcel` has `constraint = parcel.owner == keeper.key() @ TerraError::NotOwner`.

### M-2: `RecordAuditEntry` entity is unconstrained — OPEN (Low practical impact)

**File:** `lib.rs` (`RecordAuditEntry`)

`entity` is `UncheckedAccount` with no owner/PDA validation; any signer (`actor`) can create an audit entry for any entity key. The entry is append-only PDA-seeded (`audit_entry`, entity, sequence) so pollution is possible but not consensus-breaking.

**Fix (optional):** validate `entity.owner == &crate::ID` (or expected program) and/or require `actor` to be a registered validator / admin / known actor set.

### M-3: `CancelDispute` inconsistent parcel unfreeze — FIXED

Handler only unfreezes when `parcel.status == parcel_status::DISPUTED` and only allows cancel when dispute status is `FILED`.

### M-4: Challenge vote deadline uses wrong error variant — FIXED (variant renamed in practice)

Challenge deadline checks now use `TerraError::InvalidClaimStatus` (not `SettlementNotYetEffective`). No dedicated `ChallengeReviewExpired` variant yet — still a naming residual, not a logic bug.

---

## Low Findings

### L-1: `rights_count` saturating_sub could desync — OPEN (Low)

**File:** `lib.rs` (~line 931)

`require!(rights_count > 0)` then `saturating_sub(1)`. Underflow is prevented; drift only if a later close constraint fails in the same tx (currently atomic). Documented, not patched.

### L-2: Escrow vault lamports handling — ACCEPTED (reviewed)

Settle path caps transfer at `vault_lamports` and returns excess to buyer. Rent-exempt edge cases remain a theoretical concern; covered by escrow unit tests for window/min/max constants.

### L-3: Error variant reuse in slashing appeal — FIXED (partial residual)

`dispute_slashing` / appeal path on `SlashingReport` uses `NotDesignatedBuyer` for non-buyer context in `staking.rs` (report.offender mismatch). `DisputeSlashing` context also uses `NotDesignatedOffender` for the offender signer. Residual: one staking path still reuses `NotDesignatedBuyer` — cosmetic.

---

## Phase 1 additions (RFC-012) — not in original audit

1. **Unique validator sets** — `quorum::require_unique_validators` on `attest`, `rotate_validators`, `judicial_forfeiture`, `file_dispute`, `dispute_escrow`; identity `count_unique_validators` on succession/guardianship. Error: `DuplicateValidator`.
2. **Endorsement action binding** — `endorse_validator_add` requires `endorsement.action == ADD` (`WrongEndorsementAction`).
3. **Session/quorum remaining-accounts ownership** — owner checks on `try_load_session`, `try_load_quorum_config`, observation observer loop.

---

## Recommendations (next steps — ordered)

For a fresh contributor/agent with no prior context: implement these against `dev`, add/adjust unit or BPF tests, keep `clippy -D warnings` green, then re-run the Verification table in the root README.

1. **Before mainnet:** close M-2 (audit entity ownership), L-1 (explicit rights_count guard), and C-4 residual (constrain session recorder signer to opener/validator).
2. **Before mainnet:** regenerate IDL from source (`make idl`); checked-in `terra-web/src/idl/terra_registry.json` may lag (source: 118 instr / 47 acc / 108 ev / 160 err).
3. **Before mainnet:** governance reconfirm on RFC-005 staking (code path exists; original RFC cautioned against shipping without a decision).
4. **Before mainnet:** ZK circuit choice + external audit (RFC-006/011) — proof verification is still opaque in `zk.rs`.
5. **Ongoing:** fuzz instruction data validation; formal verification for ownership transfer, staking, escrow.
6. **Ongoing:** re-audit after RFC-012 Phase 2+ introduces new PDAs (tasks, presence, multi-source observation) — see RFC-012 §8–§10.

Handoff map: root README “How to continue” → this file → `terra-core/README.md` Current Status → RFC-012 phase table.
