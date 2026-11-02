# Chapter 5 — Fractional Investment and Coverage Economics

**Status:** `PROPOSED` — **hard-gated on legal research.** No code may be
written for fractional investment until Q-005 is resolved.
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

## 5.6 Reconstruction financing vs fractional ownership

Spatial reconstruction financing and fractional ownership of physical property
are **distinct mechanisms** and must not be confused:

| Mechanism | What it pays for | What rights it confers |
|---|---|---|
| Paid reconstruction (Part VIII §35) | A completed spatial reconstruction/service task | Payment for accepted work; no property rights |
| Coverage investment (Part VIII §38) | Capital/compute/storage toward a coverage objective in an underserved region | Attribution within the contribution program; bounded, revocable allocation of eligible future revenue |
| Staking / collateral (Part VIII §39) | A performance guarantee locked by a provider | Release on fulfillment; slashing on verified failure |

**No automatic property rights from compute or funding** (Principle 14).
Reconstructing or financing a spatial object does **not** create ownership of
the underlying land or building, rights over future owners or tenants, control
over transactions, access to private data, rights over all future versions, or
rights over all future revenue generated around the geographic area. Rights are
limited to a defined contribution program and reviewed for applicable legal
requirements.

**Do not confuse fractional investment with reconstruction payment.** A worker
paid to reconstruct a building has completed a service; they are not an
investor in the property. A contributor financing reconstruction in an
underserved region has supplied capital toward a coverage objective; they are
not a partial owner of the reconstructed object.

Keep service credits and SOL roles distinct according to the existing
architecture. Do not introduce a speculative Terra token as a prerequisite.
Keep large spatial assets off-chain. Do not hardcode proposed revenue
percentages (e.g. 40/60 or 70/20/10) as final protocol rules; any example is
illustrative and subject to simulation, governance, and legal review.

Research securities, crowdfunding, custody, tax, and consumer-protection
implications before recommending a real investment product.

## 5.6.1 Diagram — financial lifecycle

```mermaid
flowchart TB
    REQ[Requester<br>creates Task Manifest]
    VAL[Terra validates<br>task specification]
    PROV[Eligible providers<br>discovered]
    BID[Offers / bids<br>collected]
    SCH[Scheduler selects<br>providers]
    ESC[Funds reserved<br>in escrow]
    WK[Work assigned<br>executed]
    REC[Resource usage<br>receipts reported]
    SUB[Outputs submitted<br>as component packages]
    TECH[Automated technical<br>checks]
    VAL2[Independent validation<br>when required]
    ACC[Accepted work settled]
    REJ[Rejected work: dispute /<br>retry / refund rules]
    
    REQ --> VAL --> PROV --> BID --> SCH --> ESC --> WK --> REC --> SUB --> TECH --> VAL2 --> ACC
    VAL2 -.-> REJ
    
    style REQ fill:#e1f5e1
    style ESC fill:#d4e6f1
    style ACC fill:#e1f5e1
    style REJ fill:#f8d7da

---

## 5.7 Acceptance criteria (before status leaves `PROPOSED`)

- [ ] Written securities/custody research in RESEARCH_REFERENCES covering ≥2
      contrasting jurisdictions (Q-005).
- [ ] Founder authorization for any economic commitment (directive §15).
- [ ] Lineage single-spend design reviewed for replay/double-redemption.
- [ ] Institutional-originator requirement (Chapter 8) specified as a guard.

**This chapter is a placeholder of intent, not a build ticket.**
