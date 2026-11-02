# Sprint 013 — API request body-size limit (defense-in-depth) (2026-11-02)

**Loop:** Research → Understand → COMPARE → IDENTIFY GAPS → FORM INTENT → DESIGN
  → DOCUMENT → TRACE TO CODE → DEFINE TESTS → REVIEW SECURITY → VALIDATE → UPDATE THE BOOK
**Phase derived from evidence:** Non-chain hardening continuation (post Stage 1 exit;
  founder directive 2026-11-02: local/frontend/API work unblocks by funding alone).

## Why

`work/CURRENT.md` next-up: "frontend/UX polish and small API hardening are captured
as backlog items." Sprint 012 delivered the first hardening item (response headers).
The second: cap accepted request body size on the unauthenticated mirror before
handlers/storage read unbounded payloads.

## What was delivered

| Item | Detail |
| --- | --- |
| Config | `MAX_BODY_BYTES` env var (bytes; default **64 MiB**) in `api/src/config.rs` |
| Layer | `RequestBodyLimitLayer::new(config.max_body_bytes)` (`tower-http` `limit` feature) in `api/src/main.rs` |
| Behavior | Oversized bodies → **413 Payload Too Large** (Content-Length pre-check) |
| Tests | `tests::oversized_content_length_rejected_with_413`, `tests::within_limit_is_accepted` (in `api/src/main.rs`) |
| Gate | `CARGO_INCREMENTAL=0 cargo test -p terra-api` → **132/132** (was **130**; +2 new; 2026-11-02) |

## Current state (2026-11-02)

- Code + tests + gate green; 64 MiB default covers multipart OSM ingest (PBF) and
  photo evidence uploads; tunable via env.
- Book updated: CHANGELOG (2026-11-02 Sprint 013), IMPLEMENTATION_STATUS (B12 + API
  baseline 132), work/CURRENT.md, ROADMAP post-exit hardening note.
- Not deployed anywhere (no SOL / no deploy); local stack only.

## Loop

- Delivered the second small-API-hardening item from the post-Stage 1 backlog.
- Remaining backlog: frontend/UX polish; further API hardening (e.g., per-route
  body limits, streaming) — captured in CURRENT.md next-up.
- Founder action unchanged: Q-001 devnet funding (Stage 2) remains last.
