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

## Consulted external sources (2026-11-02)

| Source | Date accessed | Claims supported | Gates |
| --- | --- | --- | --- |
| **Residency attestation — jurisdiction survey (Q-004).** Five jurisdictions examined: Portugal (Junta de Freguesia `Atestado de Residência`, Lei 7/2001; public-faith document, dual-attestation/witness rule patchy by freguesia size), Queensland, Australia (employer/doctor/educator/AEC letterhead + utility/lease/rates within 12 mo), DC DMV (two documents from distinct agencies, 60-day window, certified copies, no screenshots), New York CUNY (12-month continuous abode, two of items a–n, immigration-status carve-outs under Ed. Law 6206(7)), Monaco (Sovereign Ordinance 8.566/8.372; foreign nationals additionally hold a `carte de séjour` and declare on honour), Luxembourg (civil-registration Act of 19 Jun 2013; third-country nationals within 3 working days; utility/lease/administrative proof). | 2026-11-02 | **Fact:** every surveyed jurisdiction requires *two* independent proofs of address from distinct sources, a recency window (3–12 months), and a named attestor with a relationship to the subject. **Fact:** no surveyed jurisdiction issues a *selective-disclosure* credential — the resident always hands over the underlying document. **Interpretation:** a dual-signed residency attestation is a *novel* instrument; its recognition would need jurisdiction-specific legal review and cannot be assumed. | REQ-021 / Chapter 2 stays `PROPOSED` until ≥1 jurisdiction's recognition is evidenced in writing. |
| **DNS-anchored identity discovery (Q-003).** Open-Attestation ADR `decentralized_identity_proof_DNS-TXT.md` (DNS-TXT binds domain owner to a smart-contract address; DNSSEC adoption still not widespread, non-repudiation weak without it); IETF draft `draft-noss-jeftovic-groundmark-core-00` (Groundmark: `_agentid` TXT + HTTP Message Signatures RFC 9421, DNSSEC required, DNS is discovery not claims DB, no single point of control); Froglet `IDENTITY_ATTESTATION.md` (subject signs a bind statement, publishes `_froglet.<zone>` TXT, marketplace attestation service re-verifies live every 30 days, 180-day expiry, DoH from a non-local resolver); HAVID spec (cross-endorsement between X.509/DID/DNS without collapsing governance); MetaMail DON paper (N-of-M multi-sig among whitelisted oracles, 51% threshold, reputation-gated, Sybil-resistant because nodes are permissioned). | 2026-11-02 | **Fact:** DNS-TXT is a *discovery* layer, not a claims database — it publishes keys and points at attestations. **Fact:** without DNSSEC the binding is repudiable by the zone owner. **Fact:** trust-minimising designs converge on (a) DNS as discovery, (b) an external attestation service that re-verifies live, (c) N-of-M consensus, (d) reputation gating to resist Sybil capture. **Interpretation:** Terra's "Bootstrap Trust Council" (Chapter 8) maps onto pattern (c)/(d); a pure DNS-TXT root is weaker than a council and should not be the sole gate. | REQ-020 / INT-005 / Chapter 8 — informs the Q-003 answer below. |

## Research output rule

Findings land here with: source, date accessed, claims supported, and links into
REQUIREMENTS_TRACEABILITY. Distinguish documented fact from interpretation in
the same entry. Legal conclusions require jurisdiction-specific evidence and
founder review (directive §5, §15).
