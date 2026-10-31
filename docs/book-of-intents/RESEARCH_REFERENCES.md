# RESEARCH_REFERENCES

Sources grounding the Book. **Consulted** = read for a documented claim in this
repo (date recorded). **Planned** = identified, not yet read/summarized; no
claims in the Book may rest on a Planned source. No fabricated citations: if a
source is not listed here, the Book does not rely on it.

## In-repo primary sources (consulted 2026-10-31 unless noted)

| Source | Used for |
| --- | --- |
| `docs/VISION.md` | Canonical state transition; Stage 3 status; domain framing |
| `docs/rfc-003` … `docs/rfc-013` (vault shard, escrow, staking/slashing, cross-border, disputes, subdivision, time-bound credentials, guardian recovery, ZK ownership, global trust architecture, spatial pipeline) | Feature specs backing ADRs A3–A11, REQ-001…REQ-017 |
| `docs/canonical-geometry-digest.md` | terra-geom-v1 normalization rules (ADR-005) |
| `docs/remaining-accounts-matrix.md` | Account coverage audit |
| Source under `terra-core/programs/`, `terra-core/api/`, `terra-core/geo-engine/`, `terra-web/src/` | All IMPLEMENTATION_STATUS rows |
| Commit history to `775e20a` | Dates/evidence for ADRs and sprints |
| Animus Sprint Loops (github.com/crussella0129/Animus_Sprint_Loops, consulted 2026-10-31) | Book structure: intents/work/sprints; evidence-derived phase state |

## External sources — planned (targets, not yet consulted)

| Source | Purpose | Gates |
| --- | --- | --- |
| World Bank — Land governance (LANDATA / Worldwide Governance Indicators, land administration) | Cross-country tenure problem evidence for INT chapters | Volume on land administration |
| FAO — Voluntary Guidelines on the Governance of Tenure (VGGT) | Customary vs statutory tenure framing (Cameroon context) | Q-004; Volume research |
| ISO 19152 — Land Administration Domain Model (LADM) | Interoperable rights/parcels/parties model; maps to RRR | REQ-013/REQ-022 specs |
| OGC — Simple Features / GeoPackage; CityGML; IndoorGML | Geometry, storage, 3D/indoor layer standards for RFC-013 extensions | 3D volume |
| Solana & Anchor official docs (version-matched 1.1.2) | Program security patterns; account validation checklists | Security review section |
| Jurisdictional securities guidance (e.g. securities regulators'/crypto frameworks) | Fractional investment legality | **Hard gate for REQ-023 (Q-005)** |
| Academic literature: diaspora property fraud; document forgery detection | Fraud pattern chapters; validator task taxonomies | Volume research |

## Research output rule

Findings land here with: source, date accessed, claims supported, and links into
REQUIREMENTS_TRACEABILITY. Distinguish documented fact from interpretation in
the same entry. Legal conclusions require jurisdiction-specific evidence and
founder review (directive §5, §15).
