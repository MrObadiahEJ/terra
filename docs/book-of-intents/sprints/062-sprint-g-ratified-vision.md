# Sprint G: Ratified vision features

**Date:** planned · **Status:** planned — **Q-003/Q-004 ratification; Q-005 hard gate**

## Trigger

REQ-020 (institutional identity), REQ-021 (residency attestation) and REQ-022
(lien/encumbrance) are `SPECIFIED ONLY`. They cannot be implemented until the
founder ratifies Q-003 (sunset of the bootstrap council) and Q-004 (residency
credential recognition). Q-005 remains a hard gate: no token, fee-yield or
profit-sharing work.

## Scope (after ratification)

1. **Institutional credentials** (REQ-020): scoped authority, vLEI/DNSSEC
   evidence bundles, InstitutionalValidator PDA.
2. **Residency attestation** (REQ-021): selective-disclosure VC profile with a
   two-proof evidence bundle; no plaintext PII on chain.
3. **Lien / encumbrance** (REQ-022): lifecycle on top of the existing rights
   and dispute primitives; no unilateral conversion.
4. **One real reconstruction job** (COLMAP/gsplat), with the resulting
   geometry digest anchored on chain and exported as proof.

## Acceptance

- A ratified institutional credential can be issued, scoped, verified and
  revoked without granting protocol-level authority.
- A residency credential verifies without exposing the underlying document.
- A lien can be filed, discharged and challenged with due process.
- The reconstruction job’s geometry hash is on chain and matches the exported
  artifact byte-for-byte.

## Blocked by

- **Q-003 / Q-004 founder ratification** — required before code.
- **Q-005** — hard gate; this sprint performs no token or yield work.
