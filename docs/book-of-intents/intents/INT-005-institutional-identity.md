# INT-005 — Institutional participation without trusted central parties

**Status:** `SPECIFIED ONLY` — designed and acceptance-criteria'd; no code.
**Founder review flag:** ⚠ pending — Q-003 (bootstrap council composition) is a
trust commitment requiring founder authorization before implementation (see
[OPEN_QUESTIONS.md](../OPEN_QUESTIONS.md) Q-003).
**Chapter:** [8 — Institutional Identity](../chapters/08-institutional-identity.md)

## Desired outcome

A government ministry, NGO, or corporation can prove on-chain that it is who it
claims to be — by bridging a DID published in an auditable off-chain registry —
and then participate under the same slashing/reputation rules as everyone else.

## Boundary

Terra verifies an *off-chain identity registration* (DNS TXT / chamber of
commerce) via a Bootstrap Trust Council or decentralized oracle. It does not
grant legal authority; it grants a verifiable `InstitutionalValidator` PDA.

## Rationale (pain it removes)

Institutional identity void (Failure 3): without verified institutional keys,
tokenized deals and lien tracking (Chapters 5–6) cannot be restricted to real
organizations, so institutional participation is too risky to attempt.

## Evidence

- Building blocks `realized`: `validator_profile.rs`, `verification/reputation.rs`,
  `staking.rs` (slashing), `cross_border.rs` (jurisdiction), `world_registry.rs`
- Target instruction `register_institutional_identity` — **not yet implemented**
- Enforcement dependency: Chapter 5 fractional deals must require this PDA
