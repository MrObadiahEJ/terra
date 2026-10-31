# Chapter 5 — Tokenized Fractional Investment

**Status:** `PROPOSED` — **hard-gated on legal research.** No code may be
written for this capability until Q-005 is resolved.
**Gate:** [OPEN_QUESTIONS.md](../OPEN_QUESTIONS.md) Q-005 ·
[REQUIREMENTS_TRACEABILITY.md](../REQUIREMENTS_TRACEABILITY.md) REQ-023
(`RESEARCH REQUIRED`).

---

## 5.1 The problem

Development economics (Failure 4 in [Chapter 1](01-core-identity-and-philosophy.md))
are often unviable for small actors: raising capital against real assets is
slow, opaque, and inaccessible. Fractional participation could widen access —
**where it is legally permitted.**

## 5.2 Desired outcome

An organization tokenizes a land asset and locks collateral into a program
escrow. Investors receive a programmable assurance: automated return, or refund
if milestones fail. Value or a redemption right may later pass through a
**transaction lineage** (Alice → Bob → Francky → Roger → Marc), and a later
holder must prove current possession, lineage, identity, and authorization
before redemption.

## 5.3 Why it is `PROPOSED` and gated

Fractional property participation engages **securities law, collective
investment schemes, custody rules, and consumer protection** in every
jurisdiction. Terra is open infrastructure, not a licensed issuer. Implementing
escrowed "investment" instructions before this research would create real legal
exposure for users and integrators. The founder's directive is explicit: research
before implementation.

## 5.4 Building blocks (real, but not sufficient)

| Block | Status | Evidence |
| --- | --- | --- |
| Escrow state machine (create/deposit/accept/settle/cancel/dispute/expire) | IMPLEMENTED AND VERIFIED | `escrow.rs`; A6 |
| Milestone/time windows | IMPLEMENTED AND VERIFIED | deadline logic in escrow + dispute |
| Institutional authority to originate a deal | PROPOSED | needs Chapter 8 / REQ-020 |
| Lineage & double-redemption defense | PROPOSED | A26; no code |

## 5.5 Open design questions (unresolved — do not assume)

- **Double redemption / replay:** a lineage must be single-spend; device
  compromise and key recovery interact with redemption authorization.
- **Milestone oracle:** who marks a construction milestone as met, and can that
  actor be bribed? (Ties back to validator routing, Chapter 4.)
- **Custody vs protocol:** does the protocol ever hold value, or only attest to
  off-chain custody?
- **Payouts never buy verification:** payments and rewards must be structurally
  unable to purchase a positive verification result (golden rule).

## 5.6 Acceptance criteria (before status leaves `PROPOSED`)

- [ ] Written securities/custody research in RESEARCH_REFERENCES covering ≥2
      contrasting jurisdictions (Q-005).
- [ ] Founder authorization for any economic commitment (directive §15).
- [ ] Lineage single-spend design reviewed for replay/double-redemption.
- [ ] Institutional-originator requirement (Chapter 8) specified as a guard.

**This chapter is a placeholder of intent, not a build ticket.**
