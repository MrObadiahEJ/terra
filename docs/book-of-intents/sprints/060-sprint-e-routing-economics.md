# Sprint E: Routing and economics hardening

**Date:** 2026-11-16 · **Status:** completed
**Exit gate:** routing cannot be captured by a requester shortlist; a spammer
cannot buy throughput; a premium request always includes a requested-category
validator. Met — see implementation notes below.

## Trigger

`route_task` currently accepts a requester-supplied candidate list. There is no
staging bond, no reputation-weighted throughput and no protocol-side candidate
discovery. The requested-category guarantee noted in A11 is still missing.

## Scope

1. **Staging bond PDA** — refundable on honest rejection, forfeited only on
   proven spam; wire into task creation and resolution. Sizing remains Q-022.
2. **Reputation-weighted throughput** — earned, not purchased; no fixed global
   caps; anti-Sybil.
3. **Protocol-side candidate discovery** — the program (or indexer-backed
   helper) derives eligible candidates from on-chain profiles/availability/
   reputation instead of trusting a requester list.
4. **Premium category request** — fee, at least one validator from the
   requested category, remaining seats filled randomly from reputable
   eligible validators.
5. **Randomness ADR** — commit-reveal or Switchboard VRF; choose one, record
   the decision, and test the failure modes.
6. **Worker re-registration regression test** — re-registering a worker must
   not overwrite its wallet (REQ-038).

## Acceptance

- A requester cannot route only to colluding validators when better eligible
  candidates exist.
- Premium routing always contains a requested-category validator and a
  random reputable fill.
- Staging bond returns/forfeits correctly under both outcomes.
- Worker wallet takeover regression test fails on the old code and passes on
  the new code.

## Blocked by

Nothing for the on-chain pieces; Q-022 fixes bond sizing. Q-005 still forbids
token/yield work.
