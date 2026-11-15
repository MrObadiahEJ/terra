# Sprint 044: Open questions → ratifiable decisions

**Date:** 2026-11-13 · **Status:** done & committed (`a74851a`)

## Trigger

`OPEN_QUESTIONS.md` held long-running blockers with no decision framing, so
agents could not tell which items were code work and which required founder
authority.

## Implementation

- Rewrote Q-001–Q-006 and Q-008–Q-018 as **decision proposed** entries, each
  with a default recommendation, external grounding and an exit criterion.
- Q-001 (devnet funding) reframed as three parallel tracks: $0 local
  validator/Surfpool, faucet rotation, and shrink-rent deployment.
- Q-003 proposes sunsetting the bootstrap council with vLEI/DNSSEC evidence
  bundles; Q-004 proposes a selective-disclosure VC + EUDI path; Q-005 keeps
  the securities hard gate (protocol ≠ issuer); Q-006 defines a seven-point
  mainnet checklist.
- Q-008–Q-018 record agent-research answers with a code-vs-design split.

## Verification

Doc-only; no code gate. `work/CURRENT.md` and `OPEN_QUESTIONS.md` updated.
Founder ratification of Q-002–Q-005 remains pending (see Known blockers).

## What is still missing

- Founder ratification/sign-off for the proposed decisions.
- Implementation of ratified Stage 3/4 items (REQ-020/021/022), still gated.

## Book updates

`OPEN_QUESTIONS.md`, `work/CURRENT.md`, `CHANGELOG.md`, `SUMMARY.md`.
