# Terra Registry Security Audit Report

**Date:** 2026-09-14
**Programs:** `terra_registry` (GaEDbktvpZ3qiqp4PmFgHwDSa6JsFfVjXFqNb2nTbage), `terra_identity` (68urV9nGcRcoWT1QjzZfXuCnTS9921x2se1SybKJr1U4)
**Auditor:** Automated review (opencode)

---

## Summary

| Severity | Count | Status |
|----------|-------|--------|
| Critical | 9 | Open |
| High | 3 | Open |
| Medium | 4 | Open |
| Low | 3 | Open |

---

## Critical Findings

### C-1: `is_authorized_owner()` trusts unverified `remaining_accounts` data

**File:** `lib.rs:151-202`
**Category:** Authority Bypass

The `is_authorized_owner()` function manually deserializes `IdentityRights` and `Identity` from `remaining_accounts` without verifying account ownership by the `terra_identity` program. An attacker can deploy a separate program that emits accounts matching the `IdentityRights`/`Identity` byte layout, set `identity.owner = attacker_key`, and pass them as `remaining_accounts`.

**Attack:** Call `transfer_parcel` with fake accounts where `identity.owner == attacker`. The function authorizes the transfer.
**Impact:** Complete parcel theft via the identity-based ownership path.
**Fix:** Verify `acc.owner == &terra_identity::ID` for each account in `remaining_accounts` before deserialization, or validate PDA seeds.

---

### C-2: No signer on `JailValidator`, `UnjailValidator`, `SlashValidator`

**File:** `lib.rs:3754-3781`
**Category:** Missing Signer

These three contexts contain only a PDA-validated `reputation` account and **no signer account**. Any wallet can call these instructions to jail, unjail, or slash any validator's reputation.

**Impact:** Attacker can jail all validators, halting the verification pipeline.
**Fix:** Add `authority: Signer<'info>` constrained to `registry.admin`.

---

### C-3: No signer on `RecordAttestationOutcome`

**File:** `lib.rs:3744-3751`
**Category:** Missing Signer

Anyone can call `record_attestation_outcome` to arbitrarily inflate or deflate any validator's reputation score.

**Impact:** Reputation manipulation, auto-jail bypass.
**Fix:** Require `registry.admin` or designated oracle signer.

---

### C-4: No signer on `RecordSessionEvidence` / `RecordSessionObservation`

**File:** `lib.rs:3685-3702`
**Category:** Missing Signer

Anyone can inject arbitrary evidence or observations into any active verification session.

**Impact:** Verification pipeline corruption.
**Fix:** Require session opener, registered validator, or `registry.admin` as signer.

---

### C-5: No authority check on `ResolveGuardianClaim` / `DisputeGuardianClaim`

**File:** `lib.rs:3926-3945`, `verification/guardian_claim.rs:110-155`
**Category:** Authority Bypass

Both instructions accept any `Signer` as `caller` with no constraint verifying the caller is the original triggerer, a registered validator, or the registry admin.

**Impact:** Attacker can prematurely resolve or maliciously dispute guardian claims.
**Fix:** Constrain `caller.key()` to be `guardian_claim.triggered_by`, the identity's recovery wallet, or `registry.admin`.

---

### C-6: No authority check on `AdjudicateDispute`

**File:** `lib.rs:2349-2365`, `dispute.rs:201-259`
**Category:** Authority Bypass

`AdjudicateDispute` has `authority: Signer<'info>` but **no constraint** verifying the authority is a registered validator or the admin.

**Impact:** Any wallet can adjudicate disputes including parcel forfeiture.
**Fix:** Constrain `authority` to `registry.admin` or check `registry.validators.contains(&authority_key)`.

---

### C-7: No authority check on `VerifyCrossBorder`

**File:** `lib.rs:3980-3989`
**Category:** Authority Bypass

`VerifyCrossBorder` has `caller: Signer<'info>` with no constraint. Anyone can verify cross-border verifications.

**Impact:** Unauthorized cross-border verification approvals.
**Fix:** Require `caller` to be a registered validator or `registry.admin`.

---

### C-8: Missing account constraints on `AuthorizeVaultAccess`

**File:** `lib.rs:303-313`
**Category:** Account Constraints

`vault_record` has no PDA seed verification. An attacker can pass any account that deserializes as `VaultRecord`.

**Fix:** Add `seeds = [b"vault_record", subject.key().as_ref()], bump` and verify `vault_record.subject == subject.key()`.

---

### C-9: Missing account constraints on credential contexts

**File:** `lib.rs:3436-3480`
**Category:** Account Constraints

`SignCredential`, `FinalizeCredential`, and `VerifyCredential` all have `credential_request` / `threshold_credential` as `mut` with **no PDA seed constraints**.

**Fix:** Add PDA seed constraints derived from instruction data.

---

## High Findings

### H-1: `try_load_reputation()` / `try_load_session()` deserialize without ownership check

**File:** `verification/quorum_voting.rs:75-91`, `verification/challenge.rs:26-43`
**Category:** Unchecked Accounts

Helper functions iterate `remaining_accounts`, match by PDA key, then deserialize without verifying account ownership. An attacker could inject fake reputation data.

**Fix:** Add `require!(acc.owner == &crate::ID, TerraError::NotAuthorized)` before deserialization.

---

### H-2: ZK instructions have no authority check

**File:** `lib.rs:3307-3409`
**Category:** Authority Bypass

`RegisterZoneSet`, `GenerateOwnershipRoot`, `InvalidateProof`, and `UpdateVerificationKeyHash` accept any `Signer` as `authority` with no constraint.

**Impact:** Attacker can invalidate all proofs, generate fake ownership roots, or replace verification keys.
**Fix:** Require `authority` to be `registry.admin`.

---

### H-3: `InitializeValidatorReputation` has no authority check

**File:** `lib.rs:3727-3741`
**Category:** Authority Bypass

Anyone can initialize reputation tracking for any validator pubkey.

**Fix:** Require `payer` to be `registry.admin` or verify the validator is in the registry.

---

## Medium Findings

### M-1: `SweepExpiredRights` has no access control

**File:** `lib.rs:2648-2664`, `time_bound.rs`

Anyone can sweep expired rights on any parcel. While this only affects expired rights, it could be used to grief.

**Fix:** Constrain `keeper` to parcel owner, right holder, or designated keeper PDA.

---

### M-2: `RecordAuditEntry` entity is unconstrained

**File:** `lib.rs:4009-4029`

Anyone can create audit entries for any entity, polluting the audit trail.

**Fix:** Validate that `entity` is a known PDA or constrain the caller.

---

### M-3: `CancelDispute` inconsistent parcel unfreeze

**File:** `dispute.rs:307-343`

Conditional unfreeze logic could leave parcels frozen if dispute status regresses.

**Severity:** Low practical impact due to state machine.

---

### M-4: Challenge vote deadline uses wrong error variant

**File:** `verification/challenge.rs:163-166`

Uses `SettlementNotYetEffective` instead of a dedicated `ChallengeReviewExpired` error.

---

## Low Findings

### L-1: `rights_count` saturating_sub could desync

**File:** `lib.rs:910`

`saturating_sub(1)` prevents underflow but could cause count drift if the close constraint fails.

---

### L-2: Escrow vault lamports handling

**File:** `escrow.rs:226-273`

Excess lamports are correctly returned, but edge cases around rent-exempt minimums could exist.

---

### L-3: Error variant reuse in `DisputeSlashing`

**File:** `lib.rs:3264`

Uses `NotDesignatedBuyer` for a non-buyer context.

---

## Recommendations

1. **Immediate:** Fix all Critical findings before any mainnet deployment
2. **Before mainnet:** Fix all High findings
3. **Before mainnet:** Add comprehensive access control audit
4. **Ongoing:** Add fuzz testing for instruction data validation
5. **Ongoing:** Consider formal verification for critical paths (ownership transfer, staking, escrow)
