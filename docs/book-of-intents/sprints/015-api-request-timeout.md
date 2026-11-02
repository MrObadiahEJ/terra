# Sprint 015 — API request timeout (defense-in-depth) (2026-11-02)

**Loop:** Research → UNDERSTAND → COMPARE → IDENTIFY GAPS → FORM INTENT → DESIGN
  → DOCUMENT → TRACE TO CODE → DEFINE TESTS → REVIEW SECURITY → VALIDATE → UPDATE THE BOOK
**Phase derived from evidence:** Non-chain hardening continuation (post Stage 1 exit).

## Why

`work/CURRENT.md` next-up: small API hardening backlog. Sprints 012 (response headers)
and 013 (body limit) delivered; this sprint delivers the per-request execution cap.

## What was delivered

| Item | Detail |
| --- | --- |
| Config | `REQUEST_TIMEOUT_SECS` env var (seconds; default **60**) in `api/src/config.rs` |
| Layer | `TimeoutLayer` (`tower-http` `timeout` feature) in `api/src/main.rs` (default response **408**) |
| Tests | `tests::slow_request_times_out_with_408` (in `api/src/main.rs`) |
| Gate | `CARGO_INCREMENTAL=0 cargo test -p terra-api` → **133/133** (was **132**; +1 new; 2026-11-02) |

## Current state (2026-11-02)

- Code + tests + gate green; default 60 s, tunable via env.
- Book updated: CHANGELOG (2026-11-02 Sprint 015), IMPLEMENTATION_STATUS (B13 + API
  baseline 133), work/CURRENT.md, ROADMAP post-exit hardening note.
- Not deployed anywhere (no SOL / no deploy); local stack only.

## Loop

- Delivered the third small-API-hardening item of the post-Stage-1 backlog.
- Remaining backlog: frontend/UX polish; further API hardening (e.g., per-route limits,
  rate limiting design); research-gated REQs.
- Founder action unchanged: Q-001 devnet funding (Stage 2) remains last.
