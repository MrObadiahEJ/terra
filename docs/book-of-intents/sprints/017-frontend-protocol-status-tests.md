# Sprint 017 — Frontend logic unit tests (protocol status) (2026-11-02)

**Loop:** RESEARCH → UNDERSTAND → COMPARE → IDENTIFY GAPS → FORM INTENT → DESIGN
  → DOCUMENT → TRACE TO CODE → DEFINE TESTS → REVIEW SECURITY → VALIDATE → UPDATE THE BOOK
**Phase derived from evidence:** Non-chain hardening/coverage continuation.

## Why

`work/CURRENT.md` next-up: frontend/UX polish and small API hardening backlog. API
hardening completed through sprint 015; frontend logic test coverage extended through
sprint 017.

## What was delivered

| Item | Detail |
| --- | --- |
| Tests added | `terra-web/src/lib/protocolStatus.test.ts` (status levels/tracks, meta tables, `statusCounts` default + custom modules) |
| Gate | `pnpm exec vitest run` → **56/56** (was **52/52**; +4 new); tsc/eslint/vite build/bundle-check green |

## Current state (2026-11-02)

- Code + tests + gates green; no production behavior change (tests only).
- Book updated: CHANGELOG (2026-11-02 Sprint 017), IMPLEMENTATION_STATUS (D8 + frontend
  vitest baseline 56/56), work/CURRENT.md, sprints/017.

## Loop

- Delivered the fourth item of the post-Stage-1 non-chain improvement stream (API hardening
  ×3, frontend tests ×2).
- Remaining backlog: frontend/UX polish; further hardening; research-gated REQs.
- Founder action unchanged: Q-001 devnet funding (Stage 2) remains last.
