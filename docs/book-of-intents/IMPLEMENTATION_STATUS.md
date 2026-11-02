# IMPLEMENTATION_STATUS — Evidence-Based Feature Register

**Audit basis:** commit `775e20a` (dev), re-verified 2026-10-31.
**Method:** source inspection + test execution where machine limits allow.
Vitest 44/44 executed this date. Cargo suites: baselines below are from their
last verified runs (BPF count re-confirmed by source: 307 `#[tokio::test]`);
re-run before any release — tracked in [ROADMAP.md](ROADMAP.md) Stage 1.

**Status vocabulary:** see [BOOK_OF_INTENTS.md](BOOK_OF_INTENTS.md) (7 levels,
1 = best).

---

## A. On-chain programs (`terra-core/programs/`)

| # | Capability | Status | Evidence | Gap / note |
| --- | --- | --- | --- | --- |
| A1 | Program builds + BPF suite | 1 – IMPLEMENTED AND VERIFIED | `tests/integration.rs` (307 `#[tokio::test]`, incl. stage3 lifecycle suites); `.so` built | Never deployed to a public cluster (see B6) |
| A2 | Parcel registry + transfer | 1 | `lib.rs` `register_parcel` (L771), `transfer_parcel` (L813); migration 0001 | — |
| A3 | Rights model (RRR: rights live in Rights PDA, not parcel column) | 1 | `grant_right`/`revoke_right`/`renew_right`/`sweep_expired_rights`/`grant_conditional_right`; migration 0025 (drops `parcels.owner`) | — |
| A4 | Time-bound rights (30-day warning, ≤1-yr grace, lazy status sweep) | 1 | `time_bound.rs` (`EXPIRING_WARNING_SECS`, `MAX_GRACE_PERIOD_SECS`, `update_right_status`) | — |
| A5 | Vault shard protocol | 1 | `vault.rs`; `create_vault`…`ping_shard` (L1336–1391); RFC-003 | — |
| A6 | Escrow sale lifecycle (create/deposit/accept/settle/cancel/mutual-cancel/dispute/expire) | 1 | `escrow.rs`; L1565–1603; API mirror + SettlementCenter (commit `d1c518a`, CDP smoke 33/33) | — |
| A7 | Dispute lifecycle (file/freeze/adjudicate/execute/cancel) | 1 | `dispute.rs`; L1532–1561; RFC-007; migration 0009 | — |
| A8 | Staking, slashing (10%/100% tiers, reporter bond 1%, 24-h review), rewards, slashing disputes | 1 | `staking.rs` (tier constants), `verify_and_slash`/`dispute_slashing`/`dismiss_report` (L1683–1703); migrations 0014/0019/0022 | — |
| A9 | Validator registry, nominations, endorsements, pause | 1 | `validator_registry.rs`; `add_validator…`→`endorse_validator_removal`, `pause_program` (L1449–1528) | — |
| A10 | Validator profiles, capabilities, presence, relationships | 1 | `validator_profile.rs`; L2166–2261; migration 0022 | — |
| A11 | Dynamic anti-fraud routing (`route_task`, nine-factor, ≤8 candidates, seed=(task_id, blockhash, slot)) | 1 | `routing.rs` header + L2416; `verification_task.rs` `can_assign` | "Requested category guaranteed" flow not implemented (see INT-003 note) |
| A12 | Verification sessions, claims, attestations, reputation | 1 | `verification/{attestation,quorum_voting,reputation}.rs`; `create_claim`…`slash_validator` (L2040–2163) | — |
| A13 | Evidence manifests (artifact kind/storage/hash/index guards) | 1 | `evidence_manifest.rs` helpers; API `/evidence`; migrations 0023/0024/0028 | — |
| A14 | ZK ownership proof (Groth16), VK pinning, proof invalidation | 2 – IMPLEMENTED, VERIFICATION INCOMPLETE | `zk/groth16.rs`; `generate_ownership_root`/`verify_ownership_proof`/`invalidate_proof` (L1715–1763); migrations 0016/0027 | Trusted-setup ceremony tooling absent; no live-cluster run |
| A15 | Time-bound credentials (request/sign/finalize/verify) | 1 | `request_credential`…`verify_credential` (L1766–1791); RFC-009 | Generic credential — no residency semantics (see C3) |
| A16 | Cross-border identity (jurisdictions, bindings, verification records) | 1 | `cross_border.rs`; L1794–1906; migrations 0012/0020 | — |
| A17 | World registry: 249 ISO codes, genesis council (≥5 confirmations, ≥3 countries), heartbeat, emergency injection | 1 | `world_registry.rs` constants; L1394–1446 | — |
| A18 | Spatial assets: versioned geometry, evidence-linked append, quorum verify | 1 | `spatial_asset.rs`; `init_spatial_asset`→`verify_geometry_version` (L1909–1976); stage3 suites; RFC-013 Phases C–E | — |
| A19 | 3D models + spatial snapshots (volumetric anchors) | 1 | `three_d_model.rs`, `spatial_snapshot.rs`; L1979–2013; stage3 3D suites | — |
| A20 | Subdivision / amalgamation / succession-with-parcels / judicial forfeiture | 1 | `subdivision.rs`; L2015–2030, L1136, L1252; RFC-008 | — |
| A21 | Identity program: bind identity, succession (request/endorse/cancel/claim), guardianship | 1 | `terra_identity/src/instructions/{bind_identity,succession,guardianship}.rs` | — |
| A22 | Institutional identity (`register_institutional_identity`, InstitutionalValidator PDA) | 5 – PROPOSED | — (directive §4/§8; building blocks A10/A12/A8 exist) | Not specified in-repo yet; needs research + founder authorization |
| A23 | Proof-of-location / residency credential (dual-signed) | 5 – PROPOSED | Building blocks: A15 (credentials), A14 (ZK disclosure), A3 (rights) | No residency spec in repo |
| A24 | Mortgage/liens encumbrance lifecycle | 5 – PROPOSED | Closest: A3 (RRR restrictions), A7 (freeze) | No lien spec in repo |
| A25 | Tokenized fractional investment + escrowed collateral guarantees | 5 – PROPOSED | Closest: A6 (pairwise escrow) | Securities-law research required first (OPEN_QUESTIONS Q-005) |
| A26 | Transaction lineage / redemption (Alice→Bob→Francky→Roger→Marc) | 5 – PROPOSED | — | Double-replay, transfer auth unresolved |

## B. Backend API (`terra-core/api/`)

| # | Capability | Status | Evidence | Gap / note |
| --- | --- | --- | --- | --- |
| B1 | REST mirror surface (~26 routers: parcels, identities, vaults, disputes, escrows, rights, cross-border, subdivision, staking, zk, spatial, spatial-accounts, geo, fusion, world-registry, recovery, credentials, evidence, reconstruction, derived, stories, tx, demo, activity…) | 1 | `routes/mod.rs`; migrations 0001–0033 | Mirror is a read model; chain is source of truth |
| B2 | Story CRUD + export receipts + validation + external replay/GeoJSON/schema (REQ-018) | 1 | `routes/stories.rs` (registered via `.merge`, absolute paths); migration 0032; `terra.story-replay/v1`; API suite 128/128 (2026-11-02); live smoke (hash tracks edits) | Mutations wallet-signed (commit `11be94c`); replay is read-only presentation data, not proof |
| B3 | Reconstruction worker: lease (SKIP LOCKED, 15-min), progress, complete→geometry draft→anchor_prep, drafts, anchor; drafts record AI provenance (REQ-024, migration 0034) | 2 | `routes/reconstruction.rs`; migrations 0031/0034; live smoke 22/22 + provenance 13/13 (2026-11-02) | anchor_prep produces a transaction; never executed on a live cluster (B6) |
| B4 | Geo endpoints (`nearest-roads`, `pois`, `buildings`, `stats`) | 1 | `routes/geo.rs` L88–93 | No server-side `/geo/route` — routing runs client-side (D4) |
| B5 | Signed multipart evidence upload | 1 | API `/evidence`; migrations 0023/0024 | — |
| B6 | Devnet deployment | 2 – IMPLEMENTED, VERIFICATION INCOMPLETE | `deploy.sh devnet` verifies canonical keypairs vs `declare_id!` then `anchor deploy`; `.so` fresh | **Blocked: deployer wallet 0 SOL** (founder action; OPEN_QUESTIONS Q-001) |
| B7 | IPFS registry mirror | 3 – PARTIALLY IMPLEMENTED | migration 0018_registry_ipfs_mirror | Independence from any single storage provider still a goal (RISK R-010) |
| B8 | Incremental derived compute: `processing_key` UNIQUE dedup, atomic create-or-reuse, idempotent complete, guarded fail, recursive-CTE dependency invalidation; reconstruction-draft ledger; all mutations signature-verified (R-016) | 1 | `routes/derived.rs`; migration 0033; API **128/128** (7 derived tests; 2026-11-02); live smoke **16/16** signed checks (unsigned/wrong key → 401); 2 identical reconstruction computations → 2 drafts, 1 ledger row, equal hashes | Cache, not a fact: correctness = same key ⇒ same output. Nothing on-chain; not a consensus input |
| B9 | Privileged-endpoint signature verification (R-016) | 1 – IMPLEMENTED AND VERIFIED | 8 handlers now `auth.verify(&key, …)?` (fail-closed); unsigned `complete` → 401 (was 200); signed suites still green (2026-11-02) | Client shipped: `api/examples/sign_request.rs` (R-017) |
| B10 | AI provenance on geometry drafts (REQ-024, R-014): `provenance_kind` + model provider/name/version/confidence, fail-closed validation, DB constraints | 1 | migration 0034; `routes/reconstruction.rs`; API **128/128** (7 provenance tests); live smoke **13/13**; constraints verified via psql (2026-11-02) | Provenance is description, not verification — an AI draft is still a draft until anchored + reviewed |

## C. Geo-engine & spatial compute (`terra-core/geo-engine`)

| # | Capability | Status | Evidence | Gap / note |
| --- | --- | --- | --- | --- |
| C1 | Sealed-road reachability via Dijkstra | 1 | `reach.rs` (O(1) sealed checks, Dijkstra to nearest sealed vertex); geo suite ≥6 (baseline) | — |
| C2 | Canonical geometry digest `terra-geom-v1` | 1 | commit `775e20a`; `docs/canonical-geometry-digest.md`; geo tests | Hash commits to representation, not real-world correctness (principle §7 of directive) |
| C3 | OSM road-graph structures | 3 – PARTIALLY IMPLEMENTED | `graph.rs` `RoadSegment` (LineString) | No node-id topology server-side; two-point routing not implemented here (client-side D4) |

## D. Frontend (`terra-web/`)

| # | Capability | Status | Evidence | Gap / note |
| --- | --- | --- | --- | --- |
| D1 | Atlas/globe (Cesium), basemap/terrain/measure/draw/search/share, land lab, settlement center, demo scenarios | 1 | GlobePage/LabPage/SettlementCenter; CDP smokes (spatial 0 exceptions; settlement 33/33) | — |
| D2 | Story engine: deterministic timeline, deep links (`?time=`/`?step=`), player, library, offline demos | 1 | `src/story/{timeline,url,demo}.ts`; vitest 44/44 (incl. timeline suite); commits `c9faf54`…`ae20af8` | — |
| D3 | Story video export (MediaRecorder, aspect presets, caption burn-in, cancel, PNG still, receipts) | 1 | `story/export.ts`; commit `0f0cc2e` | Browser-dependent codecs; no server-side render |
| D4 | Road-graph journey navigation (parse, snap, build graph, route, build route) | 1 | `story/roads.ts` + `roads.test.ts` (vitest); commit `54ad2bd` | Runs **client-side**; server `/geo/route` endpoint absent (OPEN_QUESTIONS Q-007) |
| D5 | i18n en/fr/ru/zh | 1 | `lib/locale.tsx`; fr/zh leak checks in settlement CDP smoke | — |
| D6 | Bundle discipline (R-012): route-level lazy loading enforced by a build gate | 1 | `scripts/check-bundle.mjs`; `pnpm build:check`; CI frontend job | Ceilings: entry 340 kB gz, initial 360 kB, Cesium 1200 kB, total 1900 kB; raising one is a Book decision |

## E. Repository & operations

| # | Capability | Status | Evidence | Gap / note |
| --- | --- | --- | --- | --- |
| E1 | CI (typecheck `tsc -b` + vitest + build/bundle-budget on frontend job; Rust lib/rfc012/geo/API suites; SBF build + 307-test BPF integration suite) | 1 | `.github/workflows/ci.yml`; commit `b2ddf80`; R-012 bundle gate added 2026-11-02; CI run `41474466105` green (2026-11-02) | Solana toolchain pinned `v3.1.14` — bump together with `solana-program-test`; `stable` builds are rejected by the pinned runtime (R-008 resolved) |
| E2 | License Apache-2.0 + TRADEMARKS.md | 1 | `LICENSE`, `TRADEMARKS.md`, workspace license field | — |
| E3 | Deploy keypair hygiene | 1 | `deploy.sh` verifies canonical keypairs vs `declare_id!` (registry `GaEDbkt…Tbage`, identity `HyjpaHY…Vo6p`) | — |
| E4 | Book of Intents (this document set) | 3 – PARTIALLY IMPLEMENTED | this directory; sprint 002; chapters 01–10 (09 replay, 10 derived compute/auth/provenance) | Remaining volumes depend on research-gated intents (Q-003/Q-005); registers are current |

## Verification baselines (last verified runs)

| Suite | Baseline | Last verified | Re-verify due |
| --- | --- | --- | --- |
| BPF integration (`cargo test -p terra-registry --test integration -- --test-threads=1`) | **307** | **2026-11-02 (executed; 295.9 s)** | before release |
| Registry lib | **153** | **2026-11-02 (executed)** | before release |
| API (`CARGO_INCREMENTAL=0 cargo test -p terra-api`) | **128** | **2026-11-02 (executed)** | before release |
| Geo-engine | **12 (+1 ignored)** | **2026-11-02 (executed; suite grew from the 6 baseline)** | before release |
| Identity BPF | **23** | **2026-11-02 (executed; required `anchor build --skip-lint` — stale-`.so` guard)** | before release |
| Frontend vitest (`pnpm exec vitest run`) | **44/44** | **2026-11-02 (executed)** | each story change |
| Frontend gates (`tsc -b` + eslint + build) | green | **2026-11-02 (executed)** | each story change |
| Frontend bundle budget (`pnpm build:check`) | entry **295 kB gz** / initial **301 kB** / Cesium **1098 kB lazy** / total **1678 kB** | **2026-11-02 (executed)** | each frontend build |
