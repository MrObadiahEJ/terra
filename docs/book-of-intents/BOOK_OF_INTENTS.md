# Book of Intents — Central Index

Terra's constitutional knowledge system. It records what Terra intends, what is
built, what is proven, what is uncertain, and how decisions were made. It is
maintained continuously and must stay sufficient context for a competent AI
agent or human to continue the project without the founder repeating
instructions.

**Audit basis:** repository state at commit `e9b2e7d` (dev branch),
re-verified 2026-11-02 (API 133/133, vitest 65/65, eslint clean, build + bundle
PASS; historical suite baselines recorded per their last verified runs — see
IMPLEMENTATION_STATUS).

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
- [chapters/09-external-replay.md](chapters/09-external-replay.md) — verified (REQ-018)
- [chapters/10-derived-compute-and-provenance.md](chapters/10-derived-compute-and-provenance.md) — verified (REQ-019/024, R-016/R-017)

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
- [sprints/003-derived-compute.md](sprints/003-derived-compute.md) — RFC-014 Phase 04 (incremental compute, REQ-019).
- [sprints/004-request-auth.md](sprints/004-request-auth.md) — R-016/R-017 privileged-endpoint auth.
- [sprints/005-external-replay.md](sprints/005-external-replay.md) — REQ-018 external replay.
- [sprints/006-verification-refresh.md](sprints/006-verification-refresh.md) — full verification refresh.
- [sprints/007-frontend-local-verification.md](sprints/007-frontend-local-verification.md) — local frontend verification (no SOL).
- [sprints/008-ai-provenance.md](sprints/008-ai-provenance.md) — REQ-024/R-014 AI provenance.
- [sprints/009-book-volume-10.md](sprints/009-book-volume-10.md) — Book volume 10.
- [sprints/010-bundle-discipline.md](sprints/010-bundle-discipline.md) — R-012 bundle discipline gate.
- [sprints/011-nonchain-wrapup.md](sprints/011-nonchain-wrapup.md) — non-chain work wrap-up.
- [sprints/012-api-security-headers.md](sprints/012-api-security-headers.md) — API response security headers.
- [sprints/013-api-body-limit.md](sprints/013-api-body-limit.md) — API request body-size limit.
- [sprints/014-frontend-logic-tests.md](sprints/014-frontend-logic-tests.md) — frontend logic unit tests.
- [sprints/015-api-request-timeout.md](sprints/015-api-request-timeout.md) — API request timeout.
- [sprints/016-book-index-sync.md](sprints/016-book-index-sync.md) — Book index consistency pass.
- [sprints/017-frontend-protocol-status-tests.md](sprints/017-frontend-protocol-status-tests.md) — frontend logic unit tests (protocol status).
- [sprints/018-book-index-sync-2.md](sprints/018-book-index-sync-2.md) — Book index consistency pass (part 2).
- [sprints/019-frontend-demo-polish.md](sprints/019-frontend-demo-polish.md) — Frontend/UX polish: demo header source chip.
- [sprints/020-frontend-welcome-polish.md](sprints/020-frontend-welcome-polish.md) — Frontend/UX polish: welcome offline badge.
- [sprints/021-book-consistency-traceability.md](sprints/021-book-consistency-traceability.md) — Book consistency catch-up + requirements traceability (REQ-025).
- [sprints/022-frontend-activity-store-tests.md](sprints/022-frontend-activity-store-tests.md) — Frontend logic unit tests — activity store + API GeoJSON parse.
- [sprints/023-research-q003-q004.md](sprints/023-research-q003-q004.md) — Research: institutional bootstrap (Q-003) and residency attestation (Q-004).

## Planned volumes (not yet written; see ROADMAP Stage 3)

Transactions & Lineage · AI-Assisted Understanding · Developer Ecosystem ·
Governance & Security Resilience · Land-Administration Research (Cameroon +
comparative). Each becomes a chapter only when grounded in repository evidence
or explicit research — never as filler.
