# Sprint 041 — Terra File protocol architecture directive

**Date:** 2026-11-13 · **Status:** done, uncommitted (`TERRA_PROTOCOL_DIRECTIVE.md` + `SUMMARY.md` + `BOOK_OF_INTENTS.md` + `CHANGELOG.md`)
**Gate:** doc-only — no code, no tests, no build required. Verified by re-reading
existing docs for overlap (`TERRA_VISION.md`, `ARCHITECTURAL_DECISIONS.md`,
chapters 11/13, `rfc-015`) to avoid duplicating covered concepts.

## Trigger

Founder supplied the Terra long-term protocol architecture directive (Terra File
`.terra` portable spatial world package, modular SDK boundaries, Rust-first SDK
strategy, two package modes, distributed packaging, work process, first
deliverables). Per the working rule ("create file only when it's really a new
concept"), this directive introduces genuinely new concepts not covered by
existing Book docs (§2–4, §5–16, §12, §19–20), so it is captured as a single
governing doc rather than scattered edits.

## Implementation

**New file:** `docs/book-of-intents/TERRA_PROTOCOL_DIRECTIVE.md` — full directive
text (§1–20 + core design principle + relationship-to-existing-docs table).

**Overlap mapping (intentional, to avoid duplication):**

| Directive section | Existing coverage | Handling |
| --- | --- | --- |
| §1 Vision | `TERRA_VISION.md` | Referenced, not duplicated |
| §17 Decentralization | `TERRA_VISION.md` §5, `ARCHITECTURAL_DECISIONS.md` | Referenced; directive adds failure-analysis requirement |
| §18 Interoperability | Chapters 11/13, `rfc-015` | Referenced; directive extends to container/package readers |
| §2–4, §5–16, §12, §19–20 | Not covered | New — this directive is the governing source |

**Wiring:** added to `SUMMARY.md` (registers) and `BOOK_OF_INTENTS.md`
(Documents/Registers); CHANGELOG entry.

## Verification

- Doc-only change; no code/tests/build affected.
- Confirmed no existing doc already covers the Terra File `.terra` package,
  modular SDK boundaries, Rust-first SDK strategy, two package modes, or the
  required work process / first deliverables.

## What is still missing (per directive §20 first deliverables)

These are future spec work, not code:

1. Evidence-based assessment of existing Terra package, geometry, object,
   component and spatial APIs.
2. Proposed Terra File specification.
3. ZIP/ZIP64 vs alternative container comparison.
4. Sample manifest + illustrative package layout.
5. Geographical selection and layer-based export design.
6. Whole-world / offline / distributed package strategy.
7. Rust-first SDK architecture with language bindings.
8. Interoperability plan.
9. Backward-compatibility and migration strategy.
10. Tests and security considerations.
11. Stable-core vs replaceable-capabilities classification.
12. Decentralization threat and failure analysis.
13. Prioritized implementation roadmap.
14. Code paths and tests supporting every finding.

## Book updates

New `TERRA_PROTOCOL_DIRECTIVE.md`; `SUMMARY.md` registers; `BOOK_OF_INTENTS.md`
Documents/Registers; `CHANGELOG.md`; this note.
