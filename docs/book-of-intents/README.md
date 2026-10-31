# Terra Book of Intents

Terra's living constitutional record: vision, intent, architecture,
implementation status, decisions, risks, and roadmap — maintained so that any
human or AI contributor can continue the project without the founder repeating
instructions. Structure follows the Animus Sprint Loops model
(intents / work / sprints; state derived from evidence on disk).

**Entry point:** [BOOK_OF_INTENTS.md](BOOK_OF_INTENTS.md) ·
**Vision:** [TERRA_VISION.md](TERRA_VISION.md) ·
**Agents:** [AGENT_GUIDE.md](AGENT_GUIDE.md)

## Canonical trust principle

```
OBSERVATION ≠ EVIDENCE ≠ VERIFICATION ≠ CONSENSUS ≠ LEGAL RECOGNITION
```

Terra verifies the *evidence* behind claims and records a protocol result. It
never asserts legal title, and the Book must never blur the two.

## Status vocabulary

Every capability carries exactly one status (full definitions in
[BOOK_OF_INTENTS.md](BOOK_OF_INTENTS.md)):

1. `IMPLEMENTED AND VERIFIED` — code + tests/execution evidence.
2. `IMPLEMENTED, VERIFICATION INCOMPLETE` — code exists; evidence insufficient.
3. `PARTIALLY IMPLEMENTED` — named layers exist; capability incomplete.
4. `SPECIFIED ONLY` — in an RFC/spec; no code.
5. `PROPOSED` — design direction only.
6. `RESEARCH REQUIRED` — unresolved technical/legal/economic questions.
7. `DEPRECATED OR REJECTED` — replaced or refused; reasoning preserved.

## Directories

| Directory | Meaning |
| --- | --- |
| `intents/` | Desired outcomes with lifecycle status and evidence links. |
| `chapters/` | Narrative technical specifications (volumes). |
| `work/` | Current execution state. |
| `sprints/` | Sprint provenance: research → plan → build → test evidence. |
| registers | IMPLEMENTATION_STATUS · REQUIREMENTS_TRACEABILITY · ARCHITECTURAL_DECISIONS · OPEN_QUESTIONS · RISK_REGISTER · RESEARCH_REFERENCES · ROADMAP · CHANGELOG |

Start at [SUMMARY.md](SUMMARY.md).
