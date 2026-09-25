# Terra Registry Security Audit Report

**Date:** 2026-09-14 (original audit); **updated:** 2026-09-24  
**Programs:** `terra_registry` (GaEDbktvpZ3qiqp4PmFgHwDSa6JsFfVjXFqNb2nTbage), `terra_identity` (68urV9nGcRcoWT1QjzZfXuCnTS9921x2se1SybKJr1U4)  
**Auditor:** Automated review (opencode); Phase 1 + A1 remediation + A2 IDL regen + A3 test/CI baseline applied on `dev`

---

## Summary (current status)

| Severity | Original | Open now | Fixed / mitigated |
|----------|----------|----------|-------------------|
| Critical | 9 | 0 | 9 |
| High | 3 | 0 | 3 |
| Medium | 4 | 0 | 4 |
| Low | 3 | 1 (L-3 residual) | 2 |

**Phase 0 / Phase 1 / A1** (RFC-012) closed the original Critical and High findings and all Medium findings; L-1 closed in A1. Remaining item is a cosmetic error-variant residual (L-3).

---

## Critical Findings — all closed

### C-1: `is_authorized_owner()` trusts unverified `remaining_accounts` data — FIXED

**File:** `lib.rs` (`is_authorized_owner`)

Now requires `acc.owner == &terra_identity::ID` before deserializing `IdentityRights` and `Identity`, and checks `identity.owner == signer_key`. Fake accounts from a foreign program cannot spoof the identity path. *(Function renamed to `is_authorized_holder()` in the RRR migration — RFC-012 §9; the same identity checks now gate every parcel mutation.)*

### C-2: No signer on `JailValidator`, `UnjailValidator`, `SlashValidator` — FIXED

All three contexts now have `authority: Signer` with `constraint = authority.key() == registry.admin @ TerraError::NotAuthorized`.

### C-3: No signer on `RecordAttestationOutcome` — FIXED

`authority: Signer` constrained to `registry.admin`.

### C-4: No signer on `RecordSessionEvidence` / `RecordSessionObservation` — FIXED

Both contexts require `signer: Signer` constrained to the session opener, registry admin, or a registered validator via `is_session_recorder`. `RecordSessionAttestation` (which previously had **no** signer at all) uses the same constraint. The session is PDA-validated; the handler rejects terminal sessions.

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

The sweep handler now calls `is_authorized_holder()` (ownership `Rights` PDA + optional `terra_identity` path) before sweeping. The former `parcel.owner == keeper` constraint was removed together with `Parcel.owner` in the RRR migration.

### M-2: `RecordAuditEntry` entity is unconstrained — FIXED (A1)

**File:** `lib.rs` (`RecordAuditEntry`)

`entity` now has `constraint = entity.owner == &crate::ID @ TerraError::NotAuthorized`. Only accounts owned by this program (claims, sessions, challenges, …) can be the subject of an audit entry; arbitrary system-owned keys are rejected.

### M-3: `CancelDispute` inconsistent parcel unfreeze — FIXED

Handler only unfreezes when `parcel.status == parcel_status::DISPUTED` and only allows cancel when dispute status is `FILED`.

### M-4: Challenge vote deadline uses wrong error variant — FIXED (variant renamed in practice)

Challenge deadline checks now use `TerraError::InvalidClaimStatus` (not `SettlementNotYetEffective`). No dedicated `ChallengeReviewExpired` variant yet — still a naming residual, not a logic bug.

---

## Low Findings

### L-1: `rights_count` saturating_sub could desync — FIXED (A1)

**File:** `lib.rs` (`revoke_right`)

Now uses `dec_rights_count` (`checked_sub` + `RightsLimitExceeded`) instead of `require!` + `saturating_sub`. A zero counter fails the instruction rather than silently staying at zero.

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

1. **Before mainnet:** checked-in `terra-web/src/idl/terra_registry.json` was refreshed in A2 (2026-09-24) to 119 instr / 44 acc / 108 ev / 160 err; Phase 2 re-synced to 128 instr / 49 acc / 115 ev / 169 err; Phase 3 re-synced to 134 instr / 52 acc / 120 ev / 182 err; Phase 4 re-synced to 135 instr / 53 acc / 121 ev / 184 err (matches source). Phase 5 re-synced to 137 instr / 55 acc / 123 ev / 187 err. Phase 6 re-synced to 138 instr / 55 acc / 124 ev / 191 err (matches source). Phase 7 re-synced to 147 instr / 59 acc / 133 ev / 206 err (matches source). Phase 8 re-synced to 153 instr / 64 acc / 139 ev / 220 err (matches source). A1 session-record account lists (`registry` + role-constrained `signer`) are included. Re-run `make idl` after any future program edit before client deploy.
2. **Before mainnet:** governance reconfirm on RFC-005 staking (code path exists; original RFC cautioned against shipping without a decision).
3. **Before mainnet:** ZK circuit choice + external audit (RFC-006/011) — proof verification is still opaque in `zk.rs`.
4. **Ongoing:** fuzz instruction data validation; formal verification for ownership transfer, staking, escrow.
5. **Ongoing:** re-audit after RFC-012 Phase 2+ introduces new PDAs (tasks, presence, multi-source observation) — see RFC-012 §8–§10.

Handoff map: root README “How to continue” → this file → `terra-core/README.md` Current Status → RFC-012 phase table.
