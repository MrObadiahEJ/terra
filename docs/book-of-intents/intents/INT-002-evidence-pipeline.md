# INT-002 — Verifiable evidence pipeline (`Claims ≠ Facts`)

**Status:** `IMPLEMENTED AND VERIFIED` (see [IMPLEMENTATION_STATUS.md](../IMPLEMENTATION_STATUS.md) A12–A13, B5)
**Chapter:** [1 — Core Identity & Philosophy](../chapters/01-core-identity-and-philosophy.md)

## Desired outcome

No state change occurs unless it can be traced through
`observation → evidence → validation → consensus → state`. Every artifact is
content-hashed; every transition is independently checkable.

## Boundary

Terra verifies the *process* and *integrity* of evidence. It does not judge the
truth of a physical-world claim beyond validator consensus.

## Rationale (pain it removes)

Opaque land markets (Failure 1) and AI/PropTech ground-truth gap (Failure 2):
paper evidence is forgeable and unverifiable; hashed, quorum-validated evidence
is tamper-evident and portable.

## Evidence

- `observation_v2.rs`, `evidence_manifest.rs` (kind/storage/hash/index guards),
  `verification/{attestation,quorum_voting,reputation}.rs`, `quorum.rs`
- `docs/rfc-013-spatial-intelligence-pipeline.md`, `docs/canonical-geometry-digest.md`
- Migrations `0005`, `0023`, `0028_evidence_parcel.sql`
