# Book of Intents — Central Index

Terra's constitutional knowledge system. It records what Terra intends, what is
built, what is proven, what is uncertain, and how decisions were made. It is
maintained continuously and must stay sufficient context for a competent AI
agent or human to continue the project without the founder repeating
instructions.

**Audit basis:** repository state at commit `775e20a` (dev branch),
re-verified 2026-10-31 (vitest 44/44 executed; historical suite baselines
recorded per their last verified runs — see IMPLEMENTATION_STATUS).

## Canonical trust principle

```
OBSERVATION ≠ EVIDENCE ≠ VERIFICATION ≠ CONSENSUS ≠ LEGAL RECOGNITION
```

Full chain the protocol preserves:

```
Physical world → Observation → Evidence & provenance → Independent evaluation
  → Verification attestations → Consensus under defined requirements
  → Protocol-recorded result → Legal/institutional recognition (where applicable)
```

Terra's machinery spans up to *protocol-recorded result*. It never crosses into
legal recognition and must never be described as if it does.

## Status vocabulary (mandatory, seven levels)

| # | Status | Meaning |
| --- | --- | --- |
| 1 | `IMPLEMENTED AND VERIFIED` | Code exists and tests/execution evidence support the claim. Evidence links required. |
| 2 | `IMPLEMENTED, VERIFICATION INCOMPLETE` | Code exists; testing, integration, or runtime evidence insufficient. |
| 3 | `PARTIALLY IMPLEMENTED` | Some layers exist; the complete capability does not. Missing layers must be named. |
| 4 | `SPECIFIED ONLY` | Described in an RFC/spec in this repo; no code. |
| 5 | `PROPOSED` | Design direction; not yet formally specified. |
| 6 | `RESEARCH REQUIRED` | Important technical/legal/economic questions unresolved. |
| 7 | `DEPRECATED OR REJECTED` | Replaced or deliberately rejected; reasoning preserved. |

Never inflate a status. When evidence is unavailable, say so.

## Documents

### Registers (continuously maintained)

- [TERRA_VISION.md](TERRA_VISION.md) — authoritative vision and principles.
- [IMPLEMENTATION_STATUS.md](IMPLEMENTATION_STATUS.md) — evidence-based feature register.
- [REQUIREMENTS_TRACEABILITY.md](REQUIREMENTS_TRACEABILITY.md) — intent → requirement → code → test mapping.
- [ARCHITECTURAL_DECISIONS.md](ARCHITECTURAL_DECISIONS.md) — decision log with alternatives.
- [OPEN_QUESTIONS.md](OPEN_QUESTIONS.md) — unresolved questions and founder decisions.
- [RISK_REGISTER.md](RISK_REGISTER.md) — technical, security, legal, economic risks.
- [RESEARCH_REFERENCES.md](RESEARCH_REFERENCES.md) — research sources and dates.
- [ROADMAP.md](ROADMAP.md) — staged evolution with acceptance criteria.
- [CHANGELOG.md](CHANGELOG.md) — history of Book changes.

### Operating manual

- [AGENT_GUIDE.md](AGENT_GUIDE.md) — how any agent continues the work.

### Volumes (chapters)

- [chapters/01-core-identity-and-philosophy.md](chapters/01-core-identity-and-philosophy.md) — foundations
- [chapters/02-proof-of-location-residency.md](chapters/02-proof-of-location-residency.md) — `PROPOSED`
- [chapters/03-3d-spatial-intelligence.md](chapters/03-3d-spatial-intelligence.md) — verified / partial
- [chapters/04-validator-routing.md](chapters/04-validator-routing.md) — verified (extension proposed)
- [chapters/05-fractional-investment.md](chapters/05-fractional-investment.md) — `PROPOSED` (gated)
- [chapters/06-lien-encumbrance.md](chapters/06-lien-encumbrance.md) — `PROPOSED`
- [chapters/07-postgis-geo-engine.md](chapters/07-postgis-geo-engine.md) — partial (gaps listed)
- [chapters/08-institutional-identity.md](chapters/08-institutional-identity.md) — `PROPOSED`.

### Intents

- [INT-001 spatial identity](intents/INT-001-spatial-identity.md) — `IMPLEMENTED AND VERIFIED`
- [INT-002 evidence pipeline](intents/INT-002-evidence-pipeline.md) — `IMPLEMENTED AND VERIFIED`
- [INT-003 anti-fraud validators](intents/INT-003-anti-fraud-validators.md) — `IMPLEMENTED AND VERIFIED`
- [INT-004 GIS bridge](intents/INT-004-gis-bridge.md) — `PARTIALLY IMPLEMENTED`
- [INT-005 institutional identity](intents/INT-005-institutional-identity.md) — `PROPOSED`

### Work & sprints

- [work/CURRENT.md](work/CURRENT.md) — active execution state.
- [sprints/001-story-engine.md](sprints/001-story-engine.md) — RFC-014 Phases 01–03 (committed).
- [sprints/002-book-constitution.md](sprints/002-book-constitution.md) — audit + constitution build.

## Planned volumes (not yet written; see ROADMAP Stage 3)

Transactions & Lineage · AI-Assisted Understanding · Developer Ecosystem ·
Governance & Security Resilience · Land-Administration Research (Cameroon +
comparative). Each becomes a chapter only when grounded in repository evidence
or explicit research — never as filler.
