# ARCHITECTURAL_DECISIONS — Decision Log

Append-only. Each decision: context → decision → alternatives → consequences →
evidence. Status: `Accepted` unless noted. Dates from commit history.

---

## ADR-001 — Monorepo: `terra-core` (Rust) + `terra-web` (TS), Apache-2.0
**Date:** 2025-early · **Status:** Accepted
**Context:** Protocol, API, geo-engine, and web client must evolve in lockstep;
open-source posture demands a permissive license.
**Decision:** One repository; workspace crates `terra_registry`, `terra_identity`,
`api`, `geo-engine`, `zk-membership`; `LICENSE` Apache-2.0 + `TRADEMARKS.md`.
**Alternatives:** multi-repo (rejected: cross-repo drift on a single maintainer);
GPL (rejected: blocks institutional adoption).
**Consequences:** one CI, one version story; large clone. **Evidence:**
`Cargo.toml` workspace; `LICENSE`; commit `7adaadb`.

## ADR-002 — Solana + Anchor 1.1.2, overflow-checks in release
**Date:** 2025-early · **Status:** Accepted
**Context:** Needs cheap state, composable PDAs, high throughput for global use.
**Decision:** Anchor programs on Solana; `[profile.release] overflow-checks = true`.
**Alternatives:** EVM L2 (rejected: account model poor fit for spatial PDAs);
private chain (rejected: permissionlessness).
**Consequences:** toolchain churn risk (RISK R-007); faucet/deploys gated on SOL.
**Evidence:** `terra-core/Cargo.toml`; `declare_id!` in both programs.

## ADR-003 — PostGIS mirror is a read model; chain is the source of truth
**Date:** RFC-012 era · **Status:** Accepted
**Context:** GIS queries (spatial joins, reachability) are impractical on-chain.
**Decision:** Every on-chain account mirrors into PostGIS; clients treat mirror
as cache; on-chain state wins conflicts.
**Alternatives:** on-chain spatial indexes (rejected: cost/size); third-party
indexer only (rejected: single-provider dependence).
**Consequences:** mirror drift is a managed risk (RISK R-004). **Evidence:**
migrations 0004/0017; `api/routes/*` (25 routers).

## ADR-004 — Ownership as a Rights PDA (RRR), not a parcel column
**Date:** migration 0025 · **Status:** Accepted
**Context:** Rights, restrictions, and responsibilities must be expressive
(liens, conditions, succession) without exploding the parcel account.
**Decision:** `Rights` PDA seeded `["ownership", parcel]`; `parcels.owner`
column dropped off-chain; holder sourced from `parcel_ownership`.
**Alternatives:** owner field on parcel (rejected: no encumbrance modeling).
**Consequences:** prerequisite for liens (REQ-022). **Evidence:** migration
`0025_parcel_ownership_rrr.sql` header comments; `grant_right` family.

## ADR-005 — Canonical geometry digest `terra-geom-v1`
**Date:** commit `775e20a` · **Status:** Accepted
**Context:** Fragmented 3D reconstruction data must be committed to on-chain
without claiming real-world correctness.
**Decision:** SHA-256 over canonicalized exact GeoJSON bytes under documented
normalization rules; hash labels the *representation* only.
**Alternatives:** raw-bytes hash (rejected: non-deterministic serialization);
mesh fingerprints (rejected: tool-specific).
**Consequences:** canonicalization changes are breaking — rules are versioned.
**Evidence:** `docs/canonical-geometry-digest.md`; geo-engine tests.

## ADR-006 — Stories replay deterministically on the client; export via MediaRecorder
**Date:** RFC-014 Ph.01/03 · **Status:** Accepted
**Context:** Replayable narratives that any device reproduces identically, plus
video export, without a rendering farm.
**Decision:** Timeline evaluated client-side (smoothstep + shortest-arc heading,
wall-clock rAF); export = `canvas.captureStream` + MediaRecorder with receipts;
`?time=`/`?step=` deep links.
**Alternatives:** server-rendered video (rejected: infra cost, privacy);
keyframe-baked pixels (rejected: device variance).
**Consequences:** browser codec variance (D3 note). **Evidence:**
`story/timeline.ts` + vitest; `story/export.ts`; commit `0f0cc2e`.

## ADR-007 — Cesium-first; no Three.js
**Date:** RFC-014 · **Status:** Accepted
**Context:** Repo already ships Cesium 1.144 for the globe; adding a second 3D
engine doubles bundle and maintenance.
**Decision:** Extend Cesium (StoryPlayer is a minimal Cesium mount).
**Alternatives:** Three.js scene (rejected: duplication).
**Consequences:** Cesium chunk size (RISK R-012, mitigated by lazy chunks).
**Evidence:** `components/story/StoryPlayer.tsx`; build output.

## ADR-008 — Story routes registered with `.merge` (absolute paths)
**Date:** RFC-014 Ph.01 · **Status:** Accepted
**Context:** Axum `.nest("/stories", …)` would double the prefix for handlers
already declaring `/stories`.
**Decision:** `.merge(stories::router())`; paths absolute. Mutations
wallet-signed (commit `11be94c`).
**Consequences:** new routers must choose merge-vs-nest deliberately.
**Evidence:** `routes/mod.rs` L48.

## ADR-009 — Worker leases: Postgres `SKIP LOCKED` + 15-min lease + expired reclaim
**Date:** RFC-013 Ph.D · **Status:** Accepted
**Context:** Multiple GIS/AI workers must share reconstruction jobs without
double-work or a broker.
**Decision:** Lease columns in `reconstruction_jobs`; lease takeover on expiry;
progress heartbeats; completion writes geometry drafts + `anchor_prep` with
real PDAs derived from on-chain parcel hex.
**Alternatives:** external queue (rejected: dependency); polling without leases
(races). **Evidence:** migration 0031; `routes/reconstruction.rs`; smoke 22/22.

## ADR-010 — Honest method labels everywhere
**Date:** RFC-014 · **Status:** Accepted
**Context:** Fallbacks (great-circle vs road-graph; demo vs saved data) must
never masquerade as primary results.
**Decision:** Every derived result carries its `method`/provenance label in API,
UI, and story schema; UI shows honesty notes.
**Consequences:** integrators can filter by method. **Evidence:** `story/types.ts`
`NavigationMethod`; settlement/demo honesty notes; directive §3 chain.

## ADR-011 — Single linear migration chain
**Date:** 0001–0032 · **Status:** Accepted
**Context:** One Postgres instance per environment; auditability beats
branching.
**Decision:** Numbered migrations, applied at API startup; never rewritten once
applied anywhere.
**Consequences:** forward-fix only. **Evidence:** `api/migrations/` listing.

## ADR-012 — Canonical deploy keypairs verified against `declare_id!`
**Date:** devnet prep · **Status:** Accepted
**Context:** A drifted program id silently breaks every PDA derivation.
**Decision:** `deploy.sh` verifies keypair files match `declare_id!` before
`anchor deploy`.
**Evidence:** `terra-core/deploy.sh`; IDs `GaEDbkt…Tbage` / `HyjpaHY…Vo6p`.

## ADR-013 — Book of Intents as agent constitution (7-status evidence vocabulary)
**Date:** 2026-10-31 · **Status:** Accepted
**Context:** Founder decisions must persist so agents don't need repeated
explanations; hallucinated "implemented" claims are the dominant failure mode
of AI documentation.
**Decision:** This Book: registers (status/requirements/decisions/risks/
questions/research), AGENT_GUIDE with authority boundaries, mandatory
evidence links, append-only decisions.
**Alternatives:** static README (rejected: rots); wiki (rejected: unversioned).
**Consequences:** every code change updates statuses + CHANGELOG (AGENT_GUIDE).
**Evidence:** this directory; sprint 002.

## ADR-014 — Control plane vs data plane; World Representation Manifest
**Date:** 2026-11-02 · **Status:** Accepted (spec only)
**Context:** Terra must coordinate independently stored spatial objects without
centralizing every mesh, texture, and point cloud.
**Decision:** Split the system into a control plane (object identity, spatial
registry, World Representation Manifest, spatial indexing, version references,
assembly relationships, access authorization) and a data plane (geometry files,
meshes, textures, point clouds, terrain tiles, model packages). A versioned
World Representation Manifest coordinates the world — it stores references and
relationships, not every model's geometry.
**Alternatives:** monolithic central geometry store (rejected: single point of
failure, storage cost, provider lock-in); fully decentralized registry with no
logical root (rejected: discovery/consistency burden). Hierarchical regional
manifests vs distributed registry with logical global root remains open (founder
decision — see Chapter 12 §12.2).
**Consequences:** runtime must discover, verify, and fetch components from
multiple providers; control plane must not become a per-vertex bottleneck.
**Evidence:** Chapter 12; Chapter 13.

## ADR-015 — Spatial Component Package schema & conformance (RFC-015 planned)
**Date:** 2026-11-02 · **Status:** Accepted (spec only)
**Context:** Independently produced reconstruction components must be
interoperable across engines and workers.
**Decision:** Every component ships a versioned Spatial Component Package
(component ID, physical entity ID, relationships, geometry artifact, format,
digest algorithm version, CRS, elevation reference, transform, bounding volume,
resolution/LOD, evidence refs, engine identity/version, provenance,
accuracy/uncertainty, validation state, version lineage). An engine is
Terra-compatible only if it satisfies the schema and passes the conformance
test — not merely because it exports GLB/GeoJSON.
**Alternatives:** ad-hoc per-engine formats (rejected: no interoperability);
forcing one engine (rejected: violates engine-independence principle).
**Consequences:** new conformance test and submission interface required
(Chapter 11 §11.3).
**Evidence:** Chapter 11.

## ADR-016 — `terra-geom-v2` is experimental; `terra-geom-v1` stays frozen
**Date:** 2026-11-02 · **Status:** Accepted
**Context:** The `terra-3d-spatial-pipeline` branch adds `canonical3d.rs`
(`terra-geom-v2`, 3D digest with elevation reference / vertical units /
precision), but it is based on pre-constitution commit `775e20a` and lacks all
post-constitution work.
**Decision:** `terra-geom-v1` remains frozen for existing parcel hashes.
`terra-geom-v2` is treated as **experimental** — not merged, not used for
production parcel hashes, pending integration review and RFC-013 §14 process.
**Alternatives:** merge immediately (rejected: branch is behind on all
constitution/auth/derived work); adopt v2 as the only digest (rejected: breaks
existing hashes).
**Consequences:** any v2 integration must be backward-compatible and
version-gated. **Evidence:** `terra-3d-spatial-pipeline` branch
`bf346f4`/`bd2cf24`; Chapter 3 §3.3; Chapter 14 (decision record pending).

## ADR-017 — On-chain enforces decision process; off-chain implements enforcement
**Date:** 2026-11-02 · **Status:** Accepted (spec only)
**Context:** RFC-016 separates protocol-level security/privacy decisions (which
require shared, independently verifiable state) from off-chain enforcement
(which actual storage providers, APIs and frontends carry out).
**Decision:** On-chain: request creation, scope/lifecycle state, requester
authorization, reviewer eligibility, reviewer assignment/commitment,
conflict-of-interest declarations, quorum thresholds, reviewer attestations,
decision finalization, emergency-protection expiry, appeal creation/assignment,
privacy-state transitions, jurisdictional restriction scope/expiry, collateral,
reward distribution, slashing after due process, audit events, versioning,
authorization, replay protection. Off-chain: raw documents/photos, sensitive
identity evidence, encrypted evidence storage, private review material, large
spatial models, detailed personal data, search indexes, data replication,
encryption/key management, content redaction, rendering/visualization,
storage-provider communication, notifications, UIs, jurisdiction-specific legal
interpretation, advanced fraud analysis, AI-assisted sensitivity detection,
external legal-document verification.
**Alternatives:** putting everything on-chain (rejected: privacy violation,
cost, immutability of sensitive data); putting everything off-chain (rejected:
no shared verifiable state, single points of control).
**Consequences:** on-chain data is minimal and privacy-preserving (no raw
private evidence on-chain); off-chain services follow protocol authorization
and privacy-state rules and cannot create protocol decisions by themselves;
interoperable interfaces required so independent services implement the same
rules. **Evidence:** RFC-016 §8.
## ADR-018 — Privacy-state semantics & versioning
**Date:** 2026-11-02 · **Status:** Accepted (spec only)
**Context:** Protected data needs versioned access states so public consumers
can tell available/restricted/superseded without receiving protected content.
**Decision:** Privacy states: `PUBLIC` / `RESTRICTED` / `PRIVATE` /
`REDACTED` / `TEMPORARILY_PROTECTED` / `UNDER_REVIEW` /
`WITHDRAWN_FROM_PUBLIC_ACCESS` / `TOMBSTONED` / `RESTORED`. States are
versioned; transitions are on-chain; a tombstone must not reveal the protected
content. Remedies: restrict access, remove public indexing, redact, replace
public representation, encrypt, revoke access, stop replication, crypto-erase,
tombstone reference, correct classification, reject request.
**Alternatives:** binary public/private (rejected: insufficient granularity);
per-item custom states (rejected: interoperability burden).
**Consequences:** API/storage implementations must respect finalized
privacy-state changes; no API may invent an unauthorized privacy decision
(except a clearly defined temporary local safety measure that is recorded and
expires). **Evidence:** RFC-016 §4.7.
## ADR-019 — Jurisdictional boundaries (scoped restrictions)
**Date:** 2026-11-02 · **Status:** Accepted (spec only)
**Context:** Terra supports local legal requirements without allowing one
jurisdiction to control the global protocol.
**Decision:** Jurisdictional restrictions are authenticated, scoped
(geographic + data scope + effective date + expiry/review date + appeal
mechanism), and recorded as external authority claims — not automatically
globally valid. Local restrictions do not automatically become global
restrictions; legal recognition remains distinct from Terra's technical
verification; disputed/overbroad/expired orders are challengeable.
**Alternatives:** one jurisdiction controls global state (rejected: violates
decentralization); no jurisdiction support (rejected: unworkable legally).
**Consequences:** challenge process for disputed orders; public vs private
separation preserved per jurisdiction. **Evidence:** RFC-016 §5.


