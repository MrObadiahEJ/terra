# INT-003 — Anti-fraud validator network

**Status:** `IMPLEMENTED AND VERIFIED` — with the honest caveat that a
"guaranteed requested category" premium flow is not implemented (see
[IMPLEMENTATION_STATUS.md](../IMPLEMENTATION_STATUS.md) A8–A11)
**Chapter:** [4 — Validator Routing](../chapters/04-validator-routing.md)

## Desired outcome

Physical-world attestations are produced by an unpredictable, independent
quorum that is expensive to bribe and cheap to punish. Requesters cannot pick
their validators.

## Boundary

Terra selects validators by weighted randomness over eligibility, capability,
jurisdiction, geography, availability, reputation, independence, and conflict.
A requester may request a *category* (e.g. "at least one surveyor"), never an
exact set.

## Rationale (pain it removes)

Fraud in validator networks (Failure 5): centralized or pickable validator sets
can be bribed or Sybil'd, destroying the integrity of attestations.

## Evidence

- `routing.rs` (nine-factor selection, `MAX_ROUTE_CANDIDATES = 8`, seed from
  `(task_id, recent_blockhash, slot)`)
- `staking.rs` (pattern-based slashing tiers 10% / 100%, reporter bond 1%,
  24-hour review window), `verification/reputation.rs`
- `docs/rfc-005-validator-staking-slashing-protocol.md`
- Migrations `0014`, `0019`, `0022`
