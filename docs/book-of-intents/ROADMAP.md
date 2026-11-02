# ROADMAP

Staged evolution derived from dependency order and repository evidence, not
aspiration. Each stage: entry criteria → deliverables → exit criteria (what
must be *verified* to proceed). Status vocabulary per
[BOOK_OF_INTENTS.md](BOOK_OF_INTENTS.md).

## Stage 0 — Foundations (complete)

Parcel registry, rights (RRR), vaults, escrow, disputes, staking/slashing,
cross-border, world registry, ZK ownership, credentials, spatial assets/3D,
PostGIS mirror, geo-engine, story engine RFC-014 Phases 01–03.
**Evidence:** IMPLEMENTATION_STATUS A1–A21, B1–B5, C1–C2, D1–D5; commits through
`775e20a`; BPF 307; vitest 44/44 (2026-10-31).

## Stage 1 — Complete (entry: Stage 0 evidence; exit met 2026-11-02)

1. **Book of Intents constitution** — this directory (E4): registers, guides,
   audit trail. *Exit:* all 12 deliverables present; statuses evidence-linked.
2. **RFC-014 Phase 04 — incremental compute** (REQ-019): derived-jobs cache,
   processing-key dedup, dependency invalidation. *Exit:* new API tests +
   migration with UNIQUE dedup keys; suites stay green.
   **Met 2026-10-31:** migration 0033 + `routes/derived.rs`; API 117/117;
   live smoke 16/16; 2 identical computations → 1 ledger row (sprints/003).
3. **RFC-014 Phase 05 — external access** (REQ-018): read-only story replay
   endpoint + stable schema export for robotics/API consumers; GeoJSON export.
   *Exit:* endpoint tests + schema documented in the Book.
   **Met 2026-11-02:** `/stories/{id}/replay`, `/replay.geojson`,
   `/schema/replay`; `terra.story-replay/v1`; API 121/121; live smoke
   (chapters/09, sprints/005).
4. **Verification refresh** — re-run all cargo suites against `775e20a+`
   (baselines in IMPLEMENTATION_STATUS); confirm Rust CI jobs run full suites
   (R-008). *Exit:* updated baseline table with dates.
   **Met 2026-11-02:** all suites re-executed — API 121, registry lib 153,
   BPF 307, identity 23, geo 12 (+1 ignored), vitest 44/44; table re-dated.
   CI depth completed: the Rust job now builds SBF artifacts (Solana
   `v3.1.14`) and runs the 307-test BPF suite; run `41474466105` green
   (R-008 resolved, E1 → status 1). **Stage 1 exit met 2026-11-02.**

## Stage 2 — Devnet truth (entry: Stage 1 exit met; founder funds wallet)

- Fund deployer (Q-001, founder). Run `./deploy.sh devnet`. Execute on-chain
  smoke: register parcel → rights → geometry version → 3D model → snapshot →
  escrow → dispute → slash dry-run.
- **Exit:** on-chain smoke transcript recorded in sprints/; A1 status moves
  from local-verified to cluster-verified; B6 → status 1.

## Stage 3 — Planned-capability specifications (entry: Stage 1; parallel-safe)

Write *specifications* (not code) with acceptance criteria for:
Institutional identity (REQ-020) — after Q-003 research; Lien encumbrance
(REQ-022) — builds on RRR+dispute; Residency credential (REQ-021) — after
Q-004 research + ZK paths complete (R-006).
**Exit:** each spec = chapter + intent + REQ row with `SPECIFIED ONLY` status
and founder review flag.

## Stage 4 — Research-gated features (entry: Stage 3 + research complete)

- Institutional identity implementation (REQ-020) — needs founder authorization
  (bootstrap council is a trust commitment, ADR-level).
- Fractional investment (REQ-023) — **hard-gated** on securities research
  (Q-005). No code before written legal research.
- Transaction lineage/redemption (A26) — after replay/double-spend design
  review.

## Stage 5 — Mainnet gate (entry: founder decision only)

Proposed criteria (not yet ratified — Q-006): devnet soak ≥ N weeks with smoke
transcripts; adversarial slashing simulation (R-003); external security review;
mirror audit tooling live (R-004); storage independence (Q-008); all
disclaimers reviewed by counsel. Mainnet deployment is founder-authorized only
(directive §15).

## Maintenance loop (continuous, every sprint)

Update statuses with evidence → CHANGELOG entry → traceability matrix → risk/
question registers → this roadmap. Chapters update only where affected.
