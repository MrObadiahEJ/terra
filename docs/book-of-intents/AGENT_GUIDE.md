# AGENT_GUIDE — Operating Manual for AI Agents and Contributors

This Book is Terra's memory. You can continue the project from it without the
founder repeating anything. Follow this manual exactly.

## 0. Permanent agent rule

Before asking the founder anything: (1) consult this Book, (2) inspect the
relevant code, (3) determine whether the answer is reachable independently.
Ask only for decisions that genuinely require founder authority or personal
intent. Never ask the founder to repeat something documented here.

## 1. Read order (first session in this repo)

1. [BOOK_OF_INTENTS.md](BOOK_OF_INTENTS.md) — structure + status vocabulary.
2. [TERRA_VISION.md](TERRA_VISION.md) — non-negotiables.
3. [IMPLEMENTATION_STATUS.md](IMPLEMENTATION_STATUS.md) — what is real.
4. [ROADMAP.md](ROADMAP.md) — what is next.
5. [work/CURRENT.md](work/CURRENT.md) — active execution state.
6. Relevant intent + chapter + the registers below.

## 2. Evidence rules (non-negotiable)

- Never claim a feature is implemented because docs describe it, a struct or
  instruction exists, or a previous summary said so. Verify code **and** tests.
- Every capability statement cites: file path + test (or an honest gap).
- Use only the 7 statuses. If unsure, use the *lower* status and note why.
- When repository access fails, report the limitation — never fabricate an
  audit. (This Book's audit basis is always recorded at the top of
  IMPLEMENTATION_STATUS.md.)
- Geometry hashes commit to representations, not real-world truth. Never write
  prose that blurs protocol results with legal rights (RISK R-005).

## 3. Working cycle (every substantial change)

```
RESEARCH → UNDERSTAND → COMPARE → IDENTIFY GAPS → FORM INTENT → DESIGN
  → DOCUMENT → TRACE TO CODE → DEFINE TESTS → REVIEW SECURITY
  → VALIDATE → UPDATE THE BOOK
```

Code-first discipline: specs live in the Book; code lives in
`terra-core`/`terra-web`; both move together in one change.

## 4. Update protocol (what to touch when code changes)

| Change | Update |
| --- | --- |
| Feature completed/tested | IMPLEMENTATION_STATUS row → status 1 with test name; REQUIREMENTS_TRACEABILITY row; CHANGELOG |
| New decision | ARCHITECTURAL_DECISIONS.md (append-only) |
| New risk | RISK_REGISTER.md |
| New unknown | OPEN_QUESTIONS.md |
| External source consulted | RESEARCH_REFERENCES.md (date + claims supported) |
| Sprint finished | `sprints/NNN-*.md` record + work/CURRENT.md |
| Stage exits | ROADMAP.md stage status |

Do not rewrite unaffected chapters. Keep changes modular; SUMMARY.md is
navigation only and never stores state.

## 5. Decision authority

**Decide autonomously (then record):** wording/formatting; chapter
reorganization; deduplication; resolving doc inconsistencies using stronger
code evidence; adding explanations/diagrams; proposing architecture
improvements; adding acceptance criteria; updating statuses when evidence
justifies; logging alternatives; small reversible technical choices that
preserve VISION principles.

**MUST stop and request founder authorization:** changing TERRA_VISION §1–§3;
irreversible protocol/economic commitments; mandatory centralized dependencies;
tokenomics; legal claims on the founder's behalf; production/mainnet deploys;
major breaking changes; exposing private information; anything with substantial
legal/financial/security consequence unresolved from evidence.

While a founder question is open: continue all independent work; keep the
question registered (OPEN_QUESTIONS.md).

## 6. Repository map & gates

```
terra-core/programs/terra_registry   on-chain program (instructions in lib.rs; modules per protocol area)
terra-core/programs/terra_identity   identity/succession/guardianship program
terra-core/api                       Axum REST mirror + stories/geo/reconstruction routes; migrations/
terra-core/geo-engine                road graph, sealed-reach Dijkstra, geometry digest
terra-core/zk-membership             membership proofs crate (consumers: see Q-009)
terra-web/                           React 19 + Vite + Cesium app (no Tailwind; hand-rolled CSS)
docs/                                RFCs + VISION (historical); docs/book-of-intents/ = this Book
```

Gates (run before claiming green; machine is resource-constrained — sequential
builds only, `CARGO_INCREMENTAL=0` for cargo):

```
cd terra-web && pnpm exec tsc -b --noEmit && pnpm exec eslint . && pnpm build && pnpm exec vitest run
CARGO_INCREMENTAL=0 cargo test -p terra-api
cargo test -p terra-registry --lib
cargo test -p terra-registry --test integration -- --test-threads=1   # 307 baseline
cargo test -p terra-geo-engine
```

Local machine constraints (this dev box): ~1.6 GB free disk, 8 GB RAM, load
~3. Prune `terra-core/target/debug/incremental` when disk < 1 GB; never run
heavy builds in parallel. Vite :5173 and terra-api :8080 are the dev servers;
Postgres via `PGPASSWORD=terra psql -h localhost -U terra -d terra_dev`.

## 7. Commit discipline

- Work on `dev`; sprints open one `dev → main` checkpoint (Animus model).
- **Do not commit unless the founder asks.** When asked: focused commits,
  no target/ or generated artifacts, message cites gates run.
- Never rewrite applied migrations (ADR-011). Forward-fix only.

## 8. Definition of done (any feature)

1. Code + tests exist and gates run green.
2. IMPLEMENTATION_STATUS row = status 1 with evidence links.
3. REQUIREMENTS_TRACEABILITY row updated (or new REQ ID minted).
4. CHANGELOG entry. 5. Sprint record updated. 6. Honest labels (method,
   provenance, demo-vs-saved) everywhere user- or integrator-facing (ADR-010).
