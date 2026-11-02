# Book of Intents — Summary

Navigation only; never a state store. Current phase derives from evidence in
`work/`, `sprints/`, and the registers.

## Registers

- [BOOK_OF_INTENTS.md](BOOK_OF_INTENTS.md) — central index & vocabulary
- [TERRA_VISION.md](TERRA_VISION.md) — authoritative vision
- [IMPLEMENTATION_STATUS.md](IMPLEMENTATION_STATUS.md) — evidence register
- [REQUIREMENTS_TRACEABILITY.md](REQUIREMENTS_TRACEABILITY.md) — REQ-001…REQ-025
- [ARCHITECTURAL_DECISIONS.md](ARCHITECTURAL_DECISIONS.md) — ADR-001…013
- [OPEN_QUESTIONS.md](OPEN_QUESTIONS.md) — Q-001…012
- [RISK_REGISTER.md](RISK_REGISTER.md) — R-001…017
- [RESEARCH_REFERENCES.md](RESEARCH_REFERENCES.md) — sources & dates
- [ROADMAP.md](ROADMAP.md) — Stages 0–5
- [CHANGELOG.md](CHANGELOG.md) — Book history
- [AGENT_GUIDE.md](AGENT_GUIDE.md) — operating manual

## Volumes

- [Chapter 1 — Core Identity & Philosophy](chapters/01-core-identity-and-philosophy.md) — `IMPLEMENTED AND VERIFIED` narrative
- [Chapter 2 — Proof of Location & Residency Tokens](chapters/02-proof-of-location-residency.md) — `SPECIFIED ONLY` (founder review flag, Q-004)
- [Chapter 3 — Multi-Layer 3D Spatial Intelligence](chapters/03-3d-spatial-intelligence.md) — `IMPLEMENTED AND VERIFIED` / `PARTIALLY IMPLEMENTED` (worker anchor)
- [Chapter 4 — Validator Routing](chapters/04-validator-routing.md) — `IMPLEMENTED AND VERIFIED` (category-guarantee extension `PROPOSED`)
- [Chapter 5 — Tokenized Fractional Investment](chapters/05-fractional-investment.md) — `PROPOSED` (hard-gated, Q-005)
- [Chapter 6 — Mortgage & Lien Encumbrance](chapters/06-lien-encumbrance.md) — `SPECIFIED ONLY` (founder review flag; depends on Ch8/Q-003)
- [Chapter 7 — PostGIS Mirror & Geo-Engine](chapters/07-postgis-geo-engine.md) — `PARTIALLY IMPLEMENTED` (gaps listed)
- [Chapter 8 — Institutional Identity](chapters/08-institutional-identity.md) — `SPECIFIED ONLY` (founder review flag, Q-003)
- [Chapter 9 — External Read-Only Story Replay](chapters/09-external-replay.md) — `IMPLEMENTED AND VERIFIED` (REQ-018)
- [Chapter 10 — Derived Compute, Request Authenticity, AI Provenance](chapters/10-derived-compute-and-provenance.md) — `IMPLEMENTED AND VERIFIED` (REQ-019/024, R-016/R-017)

Further volumes (Transactions & Lineage · AI-Assisted Understanding · Developer
Ecosystem · Governance & Resilience · Land-Administration Research) are written
only when evidence or research grounds them — see [ROADMAP.md](ROADMAP.md)
Stage 3.

## Intents

- [INT-001 spatial identity](intents/INT-001-spatial-identity.md) — `IMPLEMENTED AND VERIFIED`
- [INT-002 evidence pipeline](intents/INT-002-evidence-pipeline.md) — `IMPLEMENTED AND VERIFIED`
- [INT-003 anti-fraud validators](intents/INT-003-anti-fraud-validators.md) — `IMPLEMENTED AND VERIFIED`
- [INT-004 GIS bridge](intents/INT-004-gis-bridge.md) — `PARTIALLY IMPLEMENTED`
- [INT-005 institutional identity](intents/INT-005-institutional-identity.md) — `PROPOSED`

## Work & sprints

- [work/CURRENT.md](work/CURRENT.md) — active execution state
- [sprints/001-story-engine.md](sprints/001-story-engine.md) — RFC-014 Phases 01–03
- [sprints/002-book-constitution.md](sprints/002-book-constitution.md) — audit + constitution
- [sprints/003-derived-compute.md](sprints/003-derived-compute.md) — RFC-014 Phase 04 (incremental compute, REQ-019)
- [sprints/004-request-auth.md](sprints/004-request-auth.md) — R-016/R-017 privileged-endpoint auth
- [sprints/005-external-replay.md](sprints/005-external-replay.md) — REQ-018 external replay
- [sprints/006-verification-refresh.md](sprints/006-verification-refresh.md) — full verification refresh
- [sprints/007-frontend-local-verification.md](sprints/007-frontend-local-verification.md) — local frontend verification (no SOL)
- [sprints/008-ai-provenance.md](sprints/008-ai-provenance.md) — REQ-024/R-014 AI provenance
- [sprints/009-book-volume-10.md](sprints/009-book-volume-10.md) — Book volume 10
- [sprints/010-bundle-discipline.md](sprints/010-bundle-discipline.md) — R-012 bundle discipline gate
- [sprints/011-nonchain-wrapup.md](sprints/011-nonchain-wrapup.md) — non-chain work wrap-up
- [sprints/012-api-security-headers.md](sprints/012-api-security-headers.md) — API response security headers
- [sprints/013-api-body-limit.md](sprints/013-api-body-limit.md) — API request body-size limit
- [sprints/014-frontend-logic-tests.md](sprints/014-frontend-logic-tests.md) — frontend logic unit tests
- [sprints/015-api-request-timeout.md](sprints/015-api-request-timeout.md) — API request timeout
- [sprints/016-book-index-sync.md](sprints/016-book-index-sync.md) — Book index consistency pass
- [sprints/017-frontend-protocol-status-tests.md](sprints/017-frontend-protocol-status-tests.md) — frontend logic unit tests (protocol status)
- [sprints/018-book-index-sync-2.md](sprints/018-book-index-sync-2.md) — Book index consistency pass (part 2)
- [sprints/019-frontend-demo-polish.md](sprints/019-frontend-demo-polish.md) — Frontend/UX polish: demo header source chip
- [sprints/020-frontend-welcome-polish.md](sprints/020-frontend-welcome-polish.md) — Frontend/UX polish: welcome offline badge
- [sprints/021-book-consistency-traceability.md](sprints/021-book-consistency-traceability.md) — Book consistency catch-up + requirements traceability (REQ-025)
- [sprints/022-frontend-activity-store-tests.md](sprints/022-frontend-activity-store-tests.md) — Frontend logic unit tests — activity store + API GeoJSON parse
- [sprints/023-research-q003-q004.md](sprints/023-research-q003-q004.md) — Research: institutional bootstrap (Q-003) and residency attestation (Q-004)
