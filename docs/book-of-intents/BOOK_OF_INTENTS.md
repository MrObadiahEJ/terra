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
- [chapters/02-proof-of-location-residency.md](chapters/02-proof-of-location-residency.md) — `SPECIFIED ONLY` (founder review flag, Q-004)
- [chapters/03-3d-spatial-intelligence.md](chapters/03-3d-spatial-intelligence.md) — verified / partial (on-chain verified; distributed reconstruction specified only)
- [chapters/04-validator-routing.md](chapters/04-validator-routing.md) — verified (extension proposed)
- [chapters/05-fractional-investment.md](chapters/05-fractional-investment.md) — `PROPOSED` (hard-gated, Q-005)
- [chapters/06-lien-encumbrance.md](chapters/06-lien-encumbrance.md) — `SPECIFIED ONLY` (founder review flag; depends on Ch8/Q-003)
- [chapters/07-postgis-geo-engine.md](chapters/07-postgis-geo-engine.md) — partial (gaps listed)
- [chapters/08-institutional-identity.md](chapters/08-institutional-identity.md) — `SPECIFIED ONLY` (founder review flag, Q-003)
- [chapters/09-external-replay.md](chapters/09-external-replay.md) — verified (REQ-018)
- [chapters/10-derived-compute-and-provenance.md](chapters/10-derived-compute-and-provenance.md) — verified (REQ-019/024, R-016/R-017)
- [chapters/11-distributed-spatial-reconstruction-and-assembly.md](chapters/11-distributed-spatial-reconstruction-and-assembly.md) — `SPECIFIED ONLY`
- [chapters/12-world-runtime-and-synchronized-rendering.md](chapters/12-world-runtime-and-synchronized-rendering.md) — `SPECIFIED ONLY`
- [chapters/13-global-spatial-data-delivery-and-progressive-visualization.md](chapters/13-global-spatial-data-delivery-and-progressive-visualization.md) — `SPECIFIED ONLY`
- [chapters/14-implementation-preparation.md](chapters/14-implementation-preparation.md) — code-area mapping for REQ-020…REQ-028, dependency order, what not to build yet (`SPECIFIED ONLY`)

### Intents

- [INT-001 spatial identity](intents/INT-001-spatial-identity.md) — `IMPLEMENTED AND VERIFIED`
- [INT-002 evidence pipeline](intents/INT-002-evidence-pipeline.md) — `IMPLEMENTED AND VERIFIED`
- [INT-003 anti-fraud validators](intents/INT-003-anti-fraud-validators.md) — `IMPLEMENTED AND VERIFIED`
- [INT-004 GIS bridge](intents/INT-004-gis-bridge.md) — `PARTIALLY IMPLEMENTED`
- [INT-005 institutional identity](intents/INT-005-institutional-identity.md) — `PROPOSED`

### RFCs

- [rfc-015-spatial-component-package.md](../rfc-015-spatial-component-package.md) — Spatial Component Package schema & conformance (REQ-026, `SPECIFIED ONLY`)

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

## Planned volumes (not yet written; see ROADMAP Stage 3–4)

Transactions & Lineage · AI-Assisted Understanding · Developer Ecosystem ·
Governance & Security Resilience · Land-Administration Research (Cameroon +
comparative). Each becomes a chapter only when grounded in repository evidence
or explicit research — never as filler.

Architecture chapters now written (specified only, no code yet):

- Chapter 11 — Distributed Spatial Reconstruction and Assembly.
- Chapter 12 — World Runtime and Synchronized Spatial Rendering.
- Chapter 13 — Global Spatial Data Delivery and Progressive Visualization.

---

## Executive summary

**What does Terra already have?**
- On-chain: parcel registry + transfer (RRR rights-as-PDA), vault shard protocol, escrow settlement, dispute lifecycle, staking/slashing, validator registry + profiles + reputation, verification sessions/claims/attestations, evidence manifests, ZK ownership proofs (Groth16, ceremony tooling absent), time-bound credentials, cross-border identity, world registry (249 ISO codes), spatial assets + versioned geometry + spatial snapshots, 3D model registration, subdivision/amalgamation/succession/forfeiture, identity program (bind/succession/guardianship). All `IMPLEMENTED AND VERIFIED` (A1–A21), 307 BPF tests, local-only (never on devnet).
- Backend API: ~26 routers including reconstruction, derived compute, stories (replay/GeoJSON/schema), spatial, geo, evidence, ZK, tx. `IMPLEMENTED AND VERIFIED` (B1–B5, B8–B13), API 133/133. Derived compute dedup cache (`processing_key` UNIQUE, recursive-CTE invalidation) is a cache, not a fact. Privileged endpoints fail-closed on Ed25519 signature (R-016/R-017 fixed). AI provenance fields on geometry drafts (REQ-024/R-014).
- Geo-engine: sealed-road reachability (Dijkstra), canonical 2D geometry digest `terra-geom-v1` (frozen), OSM road-graph structures (partial — no server-side route), nearest-roads/pois/buildings/stats. C1 verified, C2 verified, C3 partial.
- Frontend: Cesium globe + Leaflet fallback, story engine (timeline/roads/demo/export), video export, road-graph navigation client-side, i18n, LOD_LEVELS (8 levels), terrain loading with retry, bundle discipline gate. D1–D11 verified, vitest 65/65.

**What is currently missing?**
- World Representation Manifest / central coordination object (REQ-027).
- Spatial Component Package schema + conformance test (REQ-026).
- Assembly system connecting independently produced components into one building (REQ-026).
- Distributed runtime: spatial index, manifest resolver, provider discovery/fallback, scene graph, progressive streaming, cache invalidation (REQ-027/028).
- Provider discovery and storage-node fallback (no central geometry repository).
- 3D geometry digest `terra-geom-v2` (experimental only, on `terra-3d-spatial-pipeline` branch, not merged).
- Reconstruction marketplace: task manifests, offers/bids, scheduling, escrow, settlement (task_economics applies to verification tasks only).
- Coverage investment program, jurisdictional profiles.
- On-chain institutional identity (`register_institutional_identity`), residency credential, lien encumbrance (all `SPECIFIED ONLY`, founder-gated).

**What is incorrectly described or implemented?**
- `terra-geom-v2`/`canonical3d.rs` exists only on the `terra-3d-spatial-pipeline` branch (based on pre-constitution `775e20a`) — it is **not** production; do not treat as merged or verified.
- `ThreeDModel` registration ≠ verified physical object; `GeometryDraft` ≠ complete 3D representation; anchor_prep builds a transaction but never executed on devnet.
- `task_economics.rs` reward/escrow applies to verification tasks, not reconstruction; it is not a reconstruction marketplace.
- A registered model reference on-chain is not a reconstruction engine; a Cesium globe displaying a model is not a distributed synchronized rendering runtime.

**How will independently stored spatial objects become one synchronized world representation?**
Via the World Representation Manifest (Chapter 12): a logical, versioned reference structure storing object IDs, current component versions, assembly versions, spatial extents, dependencies, storage references, LODs, and validation states — references and relationships, not geometry. Components stay on independent storage providers; the runtime resolves versions, discovers providers, fetches only needed components, verifies digests, transforms coordinates, assembles a scene graph, and renders. Immutable versions + explicit manifest transitions keep clients on a consistent assembly until they safely transition.

**How will runtime discovery, lazy loading and progressive visualization work?**
The runtime queries a hierarchical spatial index (not local filtering) for visible partitions, resolves component versions via the manifest, discovers providers (latency, authorization, cost, reliability — not just proximity), checks local/shared caches, fetches missing components with range requests, verifies digests, applies local→global transforms, and assembles the scene graph. Loading is hierarchical: L0 globe → L1 region → L2 city → L3 parcel/building → L4 interior, driven by camera distance, screen-space error, visibility, LOD, network, and device capability. Rendering begins with parcel/terrain/footprint and replaces previews with higher-detail assets; every visual carries a status label (imported/observed/surveyed/reconstructed/inferred/validated/legally recognized).

**How will reconstruction engines and workers communicate?**
Workers are engine-agnostic: each produces a Spatial Component Package (RFC-015 schema) with component ID, physical entity ID, relationships, geometry artifact, format/digest algorithm version, CRS, elevation reference, transform, bounding volume, resolution/LOD, evidence refs, engine identity/version, provenance, accuracy/uncertainty, validation state, version lineage. Packages go to a component registry; an assembly system registers/aligns them (control points, shared anchors, overlap, feature correspondences, local↔global transforms, residual errors) and connects accepted components into assemblies. Workers do not share an engine or machine; Terra depends on none.

**How will the financial system pay for work and encourage global spatial coverage?**
Three distinct mechanisms, never conflated: (1) payment for completed reconstruction work (task escrow → acceptance → settlement; payment never guarantees verification); (2) coverage investment — optional contributions (money/compute/storage/bandwidth/capture) toward a geographic coverage objective, with attribution and bounded, revocable revenue allocation; (3) staking/collateral — provider locks funds as a performance guarantee, slashed on verified failure, never proof of reconstruction accuracy. Existing `task_economics.rs` primitives (FeePolicy, CoverageIncentive, ResourceQuote, TaskEscrow, RewardAllocation) are verification-task primitives; reconstruction needs new task types and accounting. No speculative Terra token; no hardcoded revenue splits; no automatic property rights from compute/funding (Principle 14). Securities/crowdfunding/custody/tax research (Q-005) required before any investment product.

**What should be built first?**
Stage 1 is complete (constitution, derived compute, external replay, verification refresh, auth fix, AI provenance, bundle gate). Stage 2 (devnet) is blocked only on Q-001 founder funding — all local work runs with zero SOL. Once funded: devnet smoke (register parcel → rights → geometry version → 3D model → snapshot → escrow → dispute → slash dry-run). Then Stage 4 implementation: institutional identity (REQ-020), residency credential (REQ-021), lien encumbrance (REQ-022), and the distributed reconstruction/runtime architecture (REQ-026/027/028) per Chapters 11–13 — after Q-003/Q-004/Q-005 resolve.

**What should not be built yet?**
- Fractional investment (REQ-023) — hard-gated on Q-005 securities research; no code before written legal research.
- A speculative Terra token as a prerequisite.
- Mainnet (Stage 5) — founder decision only (Q-006), after devnet soak, adversarial slashing simulation, external security review, mirror audit tooling.
- Merging `terra-geom-v2` into production until an integrate-or-deprecate decision is recorded (ADR-016).
- A reconstruction marketplace before task_economics primitives are extended to reconstruction task types.
- Any feature that blurs protocol results with legal title/ownership (RISK R-005).

---

## Non-negotiable design principles

The following principles guide every proposed architecture and implementation:

1. **One world, many independent components.** Terra makes independently produced
   and stored spatial objects interoperable.
2. **Central coordination does not mean centralized geometry storage.** A logical
   global structure may coordinate the world while the underlying assets remain
   independently stored.
3. **One coherent experience, distributed execution.** Users experience a
   connected spatial representation even when its components are retrieved from
   different providers.
4. **Progressive availability.** Useful representations are visible before the
   highest-detail reconstruction is complete.
5. **Incremental updates.** Updating one roof, facade, parcel boundary, or terrain
   component does not require rebuilding the entire region.
6. **Engine independence.** Terra does not depend on one reconstruction engine,
   one AI model, one vendor, or one GPU architecture.
7. **Hardware inclusivity.** Workers participate according to their capabilities,
   with optional high-performance processing rather than mandatory high-end
   hardware.
8. **Versioned truth claims.** A model is a representation supported by evidence,
   not automatically the physical truth.
9. **Independent validation.** Financial incentives, provider reputation, staking,
   and model confidence do not determine whether a physical-world claim is true.
10. **Global interoperability with local adaptation.** Local requirements are
    supported without fragmenting global identity and data structures.
11. **Privacy by component.** Public spatial information does not expose private
    information by association.
12. **Open interfaces.** Third-party engines, storage providers, visualization
    applications, and institutions integrate without requiring permission from one
    proprietary platform, subject to security and compliance requirements.
13. **Economic transparency.** Every funded task has defined deliverables,
    resource accounting, acceptance criteria, and settlement conditions.
14. **No automatic property rights from compute or funding.** Reconstructing or
    financing a spatial object does not create ownership of the underlying land
    or building.
15. **Build foundations before global scale.** A small working system with correct
    identity, packaging, assembly, runtime, and retrieval is more valuable than a
    large collection of disconnected features.


