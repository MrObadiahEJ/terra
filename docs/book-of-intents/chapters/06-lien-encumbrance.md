# Chapter 6 — Mortgage & Lien Encumbrance Tracking

**Status:** `PROPOSED` — building blocks are `IMPLEMENTED AND VERIFIED`; the
lien lifecycle is not.
**See:** [OPEN_QUESTIONS.md](../OPEN_QUESTIONS.md),
[REQUIREMENTS_TRACEABILITY.md](../REQUIREMENTS_TRACEABILITY.md) REQ-022.

---

## 6.1 The problem

Paper-based liens are opaque, slow to check, and easy to conceal — Failure 1 in
[Chapter 1](01-core-identity-and-philosophy.md). A buyer cannot cheaply discover
a hidden mortgage or court order, and diaspora investors are disproportionately
exposed.

## 6.2 Desired outcome

An institutional validator (e.g. a bank) cryptographically **flags** a parcel
on-chain. While the lien is active, any transfer of rights requires the bank's
signature. The lien **auto-releases** on verified final payment. Every party can
see the encumbrance history.

## 6.3 Why it is `PROPOSED`

The full loop (institutional flag → transfer block → release) needs an
`InstitutionalValidator` identity (Chapter 8, `PROPOSED`) plus a lien state
machine. Building it now would ship an unenforceable, forgeable flag.

## 6.4 Building blocks (all real)

| Block | Evidence |
| --- | --- |
| Rights-as-PDA (RRR model) — restrictions are first-class | migration 0025; `grant_right`/`revoke_right` |
| Time-bound rights with expiry/grace/sweep | `time_bound.rs` |
| Parcel freeze that blocks transfer | `dispute.rs::freeze_parcel` |
| Transfer guard | `transfer_parcel` owner/rights guards (L813) |
| Institutional authority | **missing** (Chapter 8) |

## 6.5 Specified instruction flow (not implemented)

1. `place_lien(institution, parcel, terms)` — guard: signer holds
   `InstitutionalValidator` PDA with the lien capability.
2. Lien PDA blocks `transfer_parcel` unless the lien holder co-signs.
3. `record_payment(lien, evidence)` — payment evidence attached; may route
   through verification (Chapter 1 §1.3) rather than trusted self-report.
4. `release_lien(lien)` — on full payment, or automatically after sweep.
5. Events for every transition → readable encumbrance history.

## 6.6 Trust assumptions & failure modes

- **Unauthorized flag:** only an `InstitutionalValidator` with the right
  capability may place a lien (not yet enforceable — Chapter 8).
- **Stale lien:** a paid-off lien that never releases is as harmful as a hidden
  one; auto-release must be reliable.
- **Disputed payment:** payment evidence can be contested → dispute path, not a
  unilateral release.
- **Protocol ≠ title:** an *active* lien is a protocol state, not a legal
  encumbrance, unless the institution's authority is legally recognized.

## 6.7 Acceptance criteria (before status leaves `PROPOSED`)

- [ ] Chapter 8 institutional identity implemented with a lien capability scope.
- [ ] Lien state machine (place → block transfer → pay → release) with guards.
- [ ] Auto-release path + stale-lien sweep tested.
- [ ] BPF lifecycle test + dispute interaction test.
- [ ] REQ-022 row updated with evidence links.
