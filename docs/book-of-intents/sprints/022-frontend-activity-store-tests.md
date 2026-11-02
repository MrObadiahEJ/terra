# Sprint 022 — Frontend logic unit tests — activity store + API GeoJSON parse (2026-11-02)

**Loop:** RESEARCH → UNDERSTAND → COMPARE → IDENTIFY GAPS → FORM INTENT → DESIGN
  → DOCUMENT → TRACE TO CODE → DEFINE TESTS → REVIEW SECURITY → VALIDATE → UPDATE THE BOOK
**Phase derived from evidence:** Non-chain backlog (frontend test lane completion).

## Why

The frontend logic-test lane had delivered two passes (sprints 014: LOD + codec;
017: protocol-status). The remaining pure-logic surfaces — the session-local
activity store (dedup / ordering / cap / source) and the API GeoJSON parse helper —
were untested.

## What was delivered

| Item | Detail |
| --- | --- |
| `terra-web/src/lib/activityStore.test.ts` | +6 tests: starts empty, push adds, newest-first ordering, id dedup, 200-item cap, source preservation. |
| `terra-web/src/lib/api.test.ts` | +3 tests: `parseGeoJSON` null/undefined → null, valid JSON → payload, malformed → null. |
| Evidence | `pnpm exec vitest run` → **65/65** (was **56/56**; +9 new tests); tsc/eslint/vite build/bundle-check all green. |

## Current state (2026-11-02)

- Frontend gate green; no SOL required.

## Loop

- Delivered the final pass of the frontend logic-test lane.
- Remaining backlog: optional further hardening; research-gated REQs.
- Founder action unchanged: Q-001 devnet funding (Stage 2) remains last.
