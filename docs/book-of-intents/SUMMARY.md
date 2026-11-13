# Book of Intents — Summary

Navigation only; never a state store. Current phase derives from evidence in
`work/`, `sprints/`, and the registers.

## Registers

- [BOOK_OF_INTENTS.md](BOOK_OF_INTENTS.md) — central index & vocabulary
- [TERRA_VISION.md](TERRA_VISION.md) — authoritative vision
- [IMPLEMENTATION_STATUS.md](IMPLEMENTATION_STATUS.md) — evidence register
- [REQUIREMENTS_TRACEABILITY.md](REQUIREMENTS_TRACEABILITY.md) — REQ-001…REQ-040
- [ARCHITECTURAL_DECISIONS.md](ARCHITECTURAL_DECISIONS.md) — ADR-001…019
- [OPEN_QUESTIONS.md](OPEN_QUESTIONS.md) — Q-001…018
- [RISK_REGISTER.md](RISK_REGISTER.md) — R-001…026
- [RESEARCH_REFERENCES.md](RESEARCH_REFERENCES.md) — sources & dates
- [ROADMAP.md](ROADMAP.md) — Stages 0–5
- [CHANGELOG.md](CHANGELOG.md) — Book history
- [AGENT_GUIDE.md](AGENT_GUIDE.md) — operating manual

## RFCs

- [rfc-015-spatial-component-package.md](../rfc-015-spatial-component-package.md) — Spatial Component Package schema & conformance (REQ-026, SPECIFIED ONLY)
- [rfc-016-protocol-security-decentralization-privacy.md](../rfc-016-protocol-security-decentralization-privacy.md) — Protocol security, decentralization, privacy & censorship resilience (REQ-029…REQ-040, SPECIFIED ONLY)


## Volumes

- [Chapter 1 — Core Identity & Philosophy](chapters/01-core-identity-and-philosophy.md) — `IMPLEMENTED AND VERIFIED` narrative
- [Chapter 2 — Proof of Location & Residency Tokens](chapters/02-proof-of-location-residency.md) — `SPECIFIED ONLY` (founder review flag, Q-004)
- [Chapter 3 — Multi-Layer 3D Spatial Intelligence](chapters/03-3d-spatial-intelligence.md) — `IMPLEMENTED AND VERIFIED` (on-chain) / `PARTIALLY IMPLEMENTED` (worker anchor) / `SPECIFIED ONLY` (distributed reconstruction, Ch11–13)
- [Chapter 4 — Validator Routing](chapters/04-validator-routing.md) — `IMPLEMENTED AND VERIFIED` (category-guarantee extension `PROPOSED`)
- [Chapter 5 — Tokenized Fractional Investment](chapters/05-fractional-investment.md) — `PROPOSED` (hard-gated, Q-005)
- [Chapter 6 — Mortgage & Lien Encumbrance](chapters/06-lien-encumbrance.md) — `SPECIFIED ONLY` (founder review flag; depends on Ch8/Q-003)
- [Chapter 7 — PostGIS Mirror & Geo-Engine](chapters/07-postgis-geo-engine.md) — `PARTIALLY IMPLEMENTED` (gaps listed)
- [Chapter 8 — Institutional Identity](chapters/08-institutional-identity.md) — `SPECIFIED ONLY` (founder review flag, Q-003)
- [Chapter 9 — External Read-Only Story Replay](chapters/09-external-replay.md) — `IMPLEMENTED AND VERIFIED` (REQ-018)
- [Chapter 10 — Derived Compute, Request Authenticity, AI Provenance](chapters/10-derived-compute-and-provenance.md) — `IMPLEMENTED AND VERIFIED` (REQ-019/024, R-016/R-017)
- [Chapter 11 — Distributed Spatial Reconstruction and Assembly](chapters/11-distributed-spatial-reconstruction-and-assembly.md) — `IMPLEMENTED, NOT FULLY VERIFIED` (RFC-015 validator + `components`/`assemblies` routers; gated on B6)
- [Chapter 12 — World Runtime and Synchronized Spatial Rendering](chapters/12-world-runtime-and-synchronized-rendering.md) — `IMPLEMENTED, NOT FULLY VERIFIED` (`manifests`/`runtime` routers + quadtree/signed-manifest library + frontend `runtime.ts`; service wiring and B6 still outstanding)
- [Chapter 13 — Global Spatial Data Delivery and Progressive Visualization](chapters/13-global-spatial-data-delivery-and-progressive-visualization.md) — `IMPLEMENTED, NOT FULLY VERIFIED` (LOD 0–4, provider discovery + fallback, progressive tier loading, quadtree discovery cells; tiling/culling and B6 still outstanding)
- [Chapter 14 — Implementation Preparation](chapters/14-implementation-preparation.md) — code-area mapping for REQ-020…REQ-028, dependency order, what not to build yet (`IMPLEMENTED, NOT FULLY VERIFIED` for REQ-026/027/028; `SPECIFIED ONLY` for REQ-020/021/022)
- [Chapter 15 — First End-to-End Implementation Prototype](chapters/15-first-end-to-end-implementation-prototype.md) — one parcel, one building, five components, two workers, manifest → assembly → runtime → progressive render (`SPECIFIED ONLY`)

Further volumes (Transactions & Lineage · AI-Assisted Understanding · Developer
Ecosystem · Governance & Resilience · Land-Administration Research) are written
only when evidence or research grounds them — see [ROADMAP.md](ROADMAP.md)
Stage 3.

## Intents

- [INT-001 spatial identity](intents/INT-001-spatial-identity.md) — `IMPLEMENTED AND VERIFIED`
- [INT-002 evidence pipeline](intents/INT-002-evidence-pipeline.md) — `IMPLEMENTED AND VERIFIED`
- [INT-003 anti-fraud validators](intents/INT-003-anti-fraud-validators.md) — `IMPLEMENTED AND VERIFIED`
- [INT-004 GIS bridge](intents/INT-004-gis-bridge.md) — `PARTIALLY IMPLEMENTED`
- [INT-005 institutional identity](intents/INT-005-institutional-identity.md) — `SPECIFIED ONLY` (founder review flag, Q-003)

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
- [sprints/024-architecture-spec.md](sprints/024-architecture-spec.md) — Architecture spec: distributed reconstruction, world runtime, progressive delivery (Chapters 11–13)
- [sprints/025-req-026-027-028-implementation.md](sprints/025-req-026-027-028-implementation.md) — REQ-026/027/028 implementation: RFC-015 component package validator + components/assemblies/manifests/providers/runtime/workers routers + frontend runtime client
- [sprints/026-rfc-016-on-chain-security-core.md](sprints/026-rfc-016-on-chain-security-core.md) — RFC-016 on-chain security/privacy governance core (REQ-029–040): 11 PDAs + 11 instructions + 13 TerraError codes
- [sprints/027-atlas-16-point-audit.md](sprints/027-atlas-16-point-audit.md) — Atlas 16-point requirement audit: ThreeAtlasFallback host restore, TerraGlobe typed camera accessors, GlobePage failedRegionsRef dedup
- [sprints/028-book-of-intents-doc-reconciliation.md](sprints/028-book-of-intents-doc-reconciliation.md) — Book of Intents doc reconciliation: all `.md` files layer-focused, statuses cite verified evidence
- [sprints/029-merge-and-pages-publication.md](sprints/029-merge-and-pages-publication.md) — merge `origin/dev` + GitHub Pages publication of the frontend demo (base-path assets, SPA fallback, live verification)
- [sprints/030-immersive-atlas-ux.md](sprints/030-immersive-atlas-ux.md) — Google-Earth-style immersive atlas: floating chrome, responsive land panel/bottom sheet, camera-preserving 2D↔3D, collapsible navigation pad, live viewport verification
- [sprints/031-rfc-016-dispatch-repair.md](sprints/031-rfc-016-dispatch-repair.md) — RFC-016 registry dispatch repair: re-export generated `__client_accounts_*` modules at the crate root, rustfmt security files, restore `cargo check -p terra-registry`
- [sprints/032-spatial-index-signed-manifests.md](sprints/032-spatial-index-signed-manifests.md) — spatial index decision (ADR-020) + quadtree discovery cells and content-addressed Ed25519-signed manifest library in `terra-geo`
