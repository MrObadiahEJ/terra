# Chapter 8 — Institutional Identity

**Status:** `PROPOSED` — no code. Anti-fraud foundations are
`IMPLEMENTED AND VERIFIED`.
**See:** [INT-005](../intents/INT-005-institutional-identity.md),
[OPEN_QUESTIONS.md](../OPEN_QUESTIONS.md) Q-003,
[REQUIREMENTS_TRACEABILITY.md](../REQUIREMENTS_TRACEABILITY.md) REQ-020.

---

## 8.1 The problem

Public chains cannot tell a transaction from the "Ministry of Lands" or a real
bank. Without verified institutional keys, institutional participation —
liens (Chapter 6), originating fractional deals (Chapter 5) — is too risky to
attempt. This is Failure 3 / the institutional identity void in
[Chapter 1](01-core-identity-and-philosophy.md).

## 8.2 Desired outcome

An institution bridges a DID published in an auditable **off-chain registry**
into an on-chain `InstitutionalValidator` PDA, then participates under the
**same** evidence, slashing, and reputation rules as any individual validator —
with **scoped** authority, never blanket trust.

## 8.3 The flow (specified, not implemented)

1. **Off-chain:** institution generates a DID; publishes its public key in an
   auditable registry (e.g. a `.gov` DNS TXT record, or a recognized
   chamber-of-commerce entry).
2. **On-chain bridge:** `register_institutional_identity` — a Bootstrap Trust
   Council or decentralized oracle verifies the off-chain registry match.
3. **Result:** entity receives an `InstitutionalValidator` PDA with a
   **capability + scope** (which countries, which task types, which actions).
4. **Enforcement:** identical evidence/slashing/reputation protocol. A
   recognized institution gets **no** immunity. Chapter 5/6 origins can only be
   initiated by a holder of this PDA.

## 8.4 Capabilities and scopes (why not blanket authority)

A government, bank, surveyor, company, or NGO must not gain unlimited protocol
authority for being real-world-recognized. Authority is:
- **capability-scoped** (may place liens / may attest surveys / may originate deals),
- **jurisdiction-scoped** (which country's parcels),
- **revocable** (by governance or court order), and
- **slashed** like anyone else.

## 8.5 Building blocks (real)

| Block | Evidence |
| --- | --- |
| Validator profiles, capabilities, tiers, presence | `validator_profile.rs`; `init_validator_profile`…`admin_verify_validator_capability` (L2166–2248) |
| Reputation recording | `verification/reputation.rs`; `record_attestation_outcome` |
| Slashing (10%/100%, bond, appeal) | `staking.rs`; `slash_validator`, `dispute_slashing` |
| Cross-jurisdiction identity bindings | `cross_border.rs`; migrations 0012/0020 |
| Country registry + genesis consensus | `world_registry.rs`; `allocate_country`, `confirm_genesis` |

## 8.6 Open questions (unresolved — do not assume)

- **Q-003:** who composes the Bootstrap Trust Council / oracle that validates
  off-chain registries, and what stops its capture? **Founder decision required**
  (a trust commitment).
- Is DNS-TXT an adequate root of trust, or is a decentralized oracle required
  from day one?
- **Reputation must not be purchasable** through staking — how is that
  guaranteed for institutional actors with large treasuries?

## 8.7 Acceptance criteria (before status leaves `PROPOSED`)

- [ ] Q-003 research + founder authorization of the bootstrap mechanism.
- [ ] `register_institutional_identity` with capability + jurisdiction scope.
- [ ] Off-chain registry verification path specified and testable.
- [ ] Revocation + slashing parity test (institution slashed like an individual).
- [ ] Chapters 5/6 origin guards reference this PDA.
- [ ] REQ-020 updated with evidence links.
