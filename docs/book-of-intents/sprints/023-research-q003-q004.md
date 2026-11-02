# Sprint 023 — Research: institutional bootstrap (Q-003) and residency attestation (Q-004)

**Loop:** Research → Understand → Compare → Identify Gaps → Implement → Validate → Record
**Phase derived from evidence:** Research (no code change — both questions stay open)

## Research

### Q-003 — institutional bootstrap / trust council

Studied five DNS-anchored identity designs to answer "what stops the bootstrap
council from being captured, and is DNS-TXT enough?":

- **Open-Attestation ADR** `decentralized_identity_proof_DNS-TXT.md` — DNS-TXT
  binds a domain owner to a smart-contract address. Non-repudiation is weak
  without DNSSEC, which adoption has not reached.
- **IETF draft `draft-noss-jeftovic-groundmark-core-00`** (Groundmark) —
  `_agentid` TXT publishes a signing key; agents authenticate via HTTP Message
  Signatures (RFC 9421). DNSSEC is *required*. DNS is explicitly a discovery
  layer, not a claims database; there is no single point of control.
- **Froglet `IDENTITY_ATTESTATION.md`** — subject signs a bind statement, publishes
  a `_froglet.<zone>` TXT, and a marketplace attestation service re-verifies the
  evidence live every 30 days (180-day expiry, DoH from a non-local resolver).
- **HAVID spec** — cross-endorsement between X.509 / DID / DNS without collapsing
  their distinct governance properties.
- **MetaMail DON paper** — N-of-M multi-sig among permissioned, reputation-gated
  oracles; 51% threshold; Sybil resistance comes from whitelisting, not openness.

**Findings.** Trust-minimising designs converge on four elements: (a) DNS as
discovery only, (b) an external attestation service that re-verifies evidence
live rather than caching it, (c) N-of-M consensus, (d) Sybil resistance via
permissioning. A pure DNS-TXT root is weaker than a council and must not be the
sole gate for `register_institutional_identity`.

### Q-004 — residency attestation recognition

Surveyed six jurisdictions' proof-of-residence regimes: Portugal (Junta de
Freguesia `Atestado de Residência`, Lei 7/2001), Queensland Australia, DC US,
New York (CUNY), Monaco (Sovereign Ordinance 8.566/8.372), Luxembourg (Act of
19 Jun 2013).

**Findings.** Every jurisdiction requires *two* independent proofs of address
from distinct sources, a recency window (3–12 months), and a named attestor with
a relationship to the subject. **None** issues a selective-disclosure credential
— the resident always hands over the underlying document. A dual-signed residency
attestation is therefore a novel instrument whose recognition cannot be assumed.

## Compare

| Question | Before | After |
| --- | --- | --- |
| Q-003 | status 6, no evidence | status 5 — research recorded; four-element pattern identified; founder still names the council |
| Q-004 | status 6, no evidence | status 5 — six jurisdictions surveyed; none recognises the novel instrument; REQ-021 stays `PROPOSED` |

## Gaps

- No jurisdiction-specific *legal* review of dual-signed attestations — that is
  outside agent authority (directive §5/§15) and remains a founder decision.
- No code written for either question, per the research gates.

## Record

- `RESEARCH_REFERENCES.md` — two consulted-source entries (dates, claims,
  fact vs interpretation separated).
- `OPEN_QUESTIONS.md` — Q-003 → 5, Q-004 → 5, with findings.
- `chapters/02-proof-of-location-residency.md` §2.3 — research summary added.
- `chapters/08-institutional-identity.md` §8.6 — research summary added.
- No `IMPLEMENTATION_STATUS` / `REQUIREMENTS_TRACEABILITY` change: REQ-020 and
  REQ-021 remain `PROPOSED` with no source or test.

## Loop

- Both questions are now `5 – PROPOSED` (research done, founder decision pending).
  Neither blocks independent work: Stage 2 is founder-gated on Q-001 (funding),
  and Stage 3 specs for REQ-020/021/022 can be drafted from the building blocks
  already recorded in Chapters 6/8.
- Next: draft the Stage 3 specifications (REQ-020/021/022) as chapters + intents
  with acceptance criteria and founder review flags, per ROADMAP Stage 3 exit.