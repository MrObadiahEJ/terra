# Sprint 002 — Repository audit + Book of Intents constitution

**Loop:** Research → Understand → Compare → Identify Gaps → Document → Trace → Validate
**Phase derived from evidence:** Test (validated) → Loop (this record)

## Research

Audited repository at commit `775e20a` (dev) per the founder's master
directive: inspect code, not summaries. Findings that corrected earlier
assumptions:

- RFC-014 Phases 01–03 are **already committed** (story engine, road-graph
  navigation, export refinements) — including client-side road routing
  (`terra-web/src/story/roads.ts`) that previous notes had listed as "not
  started".
- `geo-engine/src/reach.rs` contains a real Dijkstra (sealed-road
  reachability); `graph.rs` still lacks node-id topology for server-side
  two-point routing, and no `GET /geo/route` endpoint exists.
- 100+ on-chain instructions across registry/identity; ~25 API routers;
  migrations 0001–0032; BPF test count re-confirmed at 307 in source.
- No code exists for institutional identity, residency credentials, liens,
  fractional investment, or transaction lineage → PROPOSED/RESEARCH REQUIRED.

## Plan

Establish the constitutional Book: entry point, vision, evidence-based status
register, traceability matrix, decision log, risk/question/research registers,
roadmap, agent manual, changelog — with the mandatory 7-level status vocabulary
and evidence links for every claim.

## Build

Created/updated (this directory + root entry):
`BOOK_OF_INTENTS.md` (root pointer), `docs/book-of-intents/{BOOK_OF_INTENTS,
TERRA_VISION, IMPLEMENTATION_STATUS, REQUIREMENTS_TRACEABILITY,
ARCHITECTURAL_DECISIONS, OPEN_QUESTIONS, RISK_REGISTER, RESEARCH_REFERENCES,
ROADMAP, AGENT_GUIDE, CHANGELOG, README, SUMMARY}.md`, intents INT-001…005
(status-corrected), `work/CURRENT.md` (stale rows corrected), this sprint
record.

## Test (evidence)

| Check | Result |
| --- | --- |
| Frontend vitest executed | **44/44 passed** (2026-10-31, 4 files) |
| BPF test count re-confirmed in source | 307 `#[tokio::test]` |
| Cargo suite re-runs | **not executed this sprint** (resource-constrained; baselines flagged stale in IMPLEMENTATION_STATUS, tracked Stage 1) |
| Internal link/status consistency of Book | validated at write time |

## Loop

Constitution v0.1.0 delivered. Carry forward: verification refresh; volumes
per ROADMAP Stage 3; registers updated on every subsequent sprint.
