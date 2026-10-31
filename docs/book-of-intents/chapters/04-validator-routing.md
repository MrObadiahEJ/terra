# Chapter 4 — Dynamic Anti-Fraud Validator Routing

**Status:** `IMPLEMENTED AND VERIFIED` (routing core). The "guaranteed
requested-category" premium flow is **not implemented** — see §4.6.
**Evidence:** `terra_registry/src/routing.rs`; `lib.rs` `route_task` (L2416);
`verification_task.rs`; RFC-012 Phase 6.

---

## 4.1 The problem

Physical-world verification is only as trustworthy as the people/devices doing
it. Centralized, pickable, or easily bribed validator sets destroy the value of
an attestation — Failure 5 in [Chapter 1](01-core-identity-and-philosophy.md).

## 4.2 Desired outcome

A verification task is evaluated by an **unpredictable, independent, qualified**
quorum that a requester cannot hand-pick and an attacker cannot cheaply
capture.

## 4.3 The selection algorithm (`routing.rs`)

Nine-factor filter, applied in order, then a seeded random pick:

```
Eligibility → Capability → Jurisdiction → Geographic relevance
  → Availability → Reputation → Independence → Conflict → Randomized pick
```

- **Candidate cap:** `MAX_ROUTE_CANDIDATES = 8` per `route_task` call.
- **Unpredictability:** winner selected with a seed derived from
  `(task_id, recent_blockhash, slot)` — a requester cannot grind a result.
- **Geographic relevance:** haversine distance (`distance_m`, e7 fixed-point)
  between task and validator presence.
- **Availability:** only `AVAILABLE`/`ONLINE` pass (`passes_availability`).
- **Single-candidate calls (n=1)** always select that candidate if eligible —
  used by tests and by requesters who already short-listed off-chain.
- The requester supplies an explicit candidate list as instruction args;
  candidates' PDAs are verified in `remaining_accounts` before evaluation.

## 4.4 Why this resists fraud

| Attack | Defense |
| --- | --- |
| Requester picks a corrupt validator | Seeded randomness over the eligible set; requester never names the winner |
| Sybil validators | Capability + profile-tier + jurisdiction filters; staking and reputation gates |
| Bribing a known set | Set is unknown until the slot is fixed |
| Geographic capture | Geographic-relevance filter keeps evaluation local-but-not-single-node |
| Correlated insiders | Independence + conflict-of-interest edges (`create_validator_relationship_edge`) |

Complementary **slashing** (`staking.rs`, `IMPLEMENTED AND VERIFIED`): 10%
first-offense / 100% repeat-offense tiers, 1% reporter bond, 24-hour review
window, `dispute_slashing` appeal. Reputation is recorded outcomes
(`record_attestation_outcome`), not purchased.

## 4.5 Accounts & guards

- `route_task` (L2416) takes the candidate list as args and verifies each
  candidate PDA in `remaining_accounts`; an Assignment PDA is created for the
  winner only.
- `verification_task.rs` `can_assign` gates task state transitions.

## 4.6 Honest gap: requested-category guarantee

A user may want *at least one* validator from a category (e.g. a licensed
surveyor). The current implementation filters **by** capability but does **not**
implement the premium "weighted-random attempt to include ≥1 from the
requested pool, with seamless fallback to a random high-reputation quorum"
flow described in the blueprint. The Book marks this `PROPOSED`, not shipped —
the protocol must never *promise* a specific validator type unless the
implementation guarantees it.

## 4.7 Acceptance criteria for the category-guarantee extension

- [ ] Requested-category arg with explicit fallback semantics.
- [ ] Weights are auditable; empty/insufficient pool falls back without error.
- [ ] Event records whether the category was satisfied or fell back.
- [ ] BPF tests: satisfied, fallback, and empty-pool paths.
- [ ] REQ-006 row updated; this chapter's status becomes
      "IMPLEMENTED AND VERIFIED" only when §4.6 is closed.
