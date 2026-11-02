# Sprint 012 — API security response headers (defense-in-depth) (2026-11-02)

**Loop:** Research → Understand → COMPARE → IDENTIFY GAPS → FORM INTENT → DESIGN
  → DOCUMENT → TRACE TO CODE → DEFINE TESTS → REVIEW SECURITY → VALIDATE → UPDATE THE BOOK
**Phase derived from evidence:** Non-chain hardening continuation (post Stage 1 exit;
  founder directive 2026-11-02: local/frontend/API work unblocks by funding alone).

## Why

`work/CURRENT.md` next-up: "frontend/UX polish and small API hardening are captured
as backlog items; larger requirements (REQ-020…REQ-023) stay research-gated (Q-003/Q-005)."

The unauthenticated mirror API (`terra-core/api`) already had defense-in-depth in
progress: CORS locked to configured origins (`api/src/main.rs`), fail-closed Ed25519
privileged auth (R-016/R-017). Gap: no browser hardening response headers — without
`X-Content-Type-Options`, browsers could MIME-sniff JSON responses, frame responses,
or leak referrer context. Small, reversible, no SOL required.

## What was delivered

| Item | Detail |
| --- | --- |
| Middleware | `security_headers` in `terra-core/api/src/main.rs`, applied via `axum::middleware::from_fn` as outermost layer (after CORS + Trace) |
| Headers | `X-Content-Type-Options: nosniff`, `X-Frame-Options: DENY`, `Referrer-Policy: no-referrer`, `Permissions-Policy: camera=(), microphone=(), geolocation=()` |
| Rationale | JSON-only API surface (plus multipart evidence/OSM ingest) → no CSP required |
| Tests | `tests::ok_response_carries_security_headers`, `tests::error_response_also_carries_security_headers` (in `api/src/main.rs`) |
| Gate | `CARGO_INCREMENTAL=0 cargo test -p terra-api` → **130/130** (was **128**; +2 new; 2026-11-02) |

## Current state (2026-11-02)

- Code + tests + gate green; no behavior change for valid clients (headers only).
- Book updated: CHANGELOG (2026-11-02 Sprint 012), IMPLEMENTATION_STATUS (B11 + API
  baseline 130), work/CURRENT.md, ROADMAP post-exit hardening note.
- Not deployed anywhere (no SOL / no deploy); local stack only.

## Loop

- Sprint 012 delivered the first small-API-hardening item from the post-Stage-1 backlog.
- Remaining backlog: further API hardening (e.g., body-size limit design for multipart
  ingest), frontend/UX polish — captured in CURRENT.md next-up.
- Founder action unchanged: Q-001 devnet funding (Stage 2) remains last.
