# Sprint 011 — Non-chain work wrap-up and transition (2026-11-02)

**Loop:** Record → Transition → Plan
**Phase derived from evidence:** All major non-chain deliverables complete; CI green; Book registers current.

## Why

Sprints 003–010 covered the Stage 1 work completed on 2026-11-02:
- RFC-014 Phases 01–05 (story engine, derived compute, external replay, verification refresh)
- R-014 / REQ-024 AI provenance on geometry drafts
- R-016 / R-017 privileged-endpoint signature verification and signing client
- R-012 bundle discipline gate
- Book volumes 09 (replay) and 10 (derived compute, auth, AI provenance)

No further code changes are required to close the current phase. The remaining open items are:
- Founder-directed research questions (Q-003/Q-005) — intentionally research-gated
- Devnet funding (Q-001 / B6) — founder directive: last step, does not block
- Incremental polish items — captured in backlog, not blocking

## What was delivered

| Sprint | Deliverable | Key Status |
|--------|------------|-----------|
| 003 | Derived compute (processing-key dedup) | API tests + live smoke; UNIQUE key + recursive CTE invalidation |
| 004 | Request-auth (Ed25519 verification) | 8 handlers fail-closed; sign_request.rs client shipped |
| 005 | External replay/access (REQ-018) | `/stories/{id}/replay`, `/replay.geojson`, `/schema/replay` |
| 006 | Verification refresh | All suites re-executed; baselines re-dated |
| 007 | Frontend local verification (no SOL) | `/welcome`, `/stories`, `/demo`, `/story/...` all render locally |
| 008 | AI provenance on drafts (REQ-024/R-014) | migration 0034; `provenance_kind` + model fields; fail-closed validation; API 128/128; smoke 13/13 |
| 009 | Book volume 10 (trust boundaries) | Ch.10: cache≠truth, sig≠identity, provenance≠verification |
| 010 | Bundle discipline gate (R-012) | `check-bundle.mjs`; CI build+budget; budgets: entry 340/initial 360/Cesium 1200/total 1900 kB gz |

## Current state (2026-11-02)

- **Dev branch**: `3ee842f` — all changes committed and pushed; `main` fast-forwarded
- **CI**: All jobs green — API 128/128, BPF 307/307, vitest 44/44, frontend typecheck + budget
- **Known blockers**: Only founder-directed (Q-001 devnet funding, Q-003/Q-005 research)
- **Next-up** (CURRENT.md): frontend/UX polish and small API hardening; research-gated reqs stay gated

## Loop

- Non-chain work phase (sprints 003–011) is complete. All registers (IMPLEMENTATION_STATUS, REQUIREMENTS_TRACEABILITY, RISK_REGISTER, ROADMAP, CURRENT, CHANGELOG, BOOK_OF_INTENTS, SUMMARY) are current.
- No SOL required for any delivered item.
- Founder action remaining: Q-001 devnet funding → Stage 2 on-chain smoke (A1/B6).
- Future work candidates: incremental frontend polish, API hardening, research-gated intents after Q-003/Q-005 resolution.