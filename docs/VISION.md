# TERRA — The Master Vision

> **Terra is an open physical-world trust and spatial infrastructure protocol that
> gives real-world entities persistent spatial identity and verifiable history by
> connecting observations, evidence, independent validation, rights and transactions
> across time and jurisdictions.**
>
> **Land is the first domain.** 2D parcels are the starting point. The long-term
> architecture expands into 3D/4D digital twins, physical-world observation,
> AI-assisted geospatial intelligence, decentralized verification, privacy-preserving
> identity, cross-border interoperability, government participation, and transaction
> infrastructure for the physical world.
>
> **Architectural rule:** Terra never treats a single AI model, sensor, person,
> company, government, oracle or database as absolute truth. The canonical transition
> is `observation → evidence → validation → consensus → state`.

**This is the north-star document** from which future RFCs, code, AI agents,
fundraising decks, partnerships and product decisions are derived. Read it before
touching anything else in this repository.

Related: [`../README.md`](../README.md) (repo status) ·
[`rfc-012-global-physical-digital-trust-architecture.md`](rfc-012-global-physical-digital-trust-architecture.md)
(the architecture contract) · [`../terra-core/SECURITY.md`](../terra-core/SECURITY.md).

---

## Where we are today (verified 2026-09-25)

| Layer | State |
|-------|-------|
| On-chain programs | `terra_registry` **148 instructions · 62 accounts · 135 events · 220 errors**; `terra_identity` **8 · 2 · 8 · 29** — IDL checked in and synced |
| Protocol specs | **RFC-003 → RFC-012 all delivered** (vault, escrow, staking, cross-border, disputes, subdivision, time-bound, guardianship, ZK, global architecture) |
| Hardening | RFC-012 **Phases 0–8 complete** + Tier A1–A3 + B6 program boundary + P0-2 removal path |
| Tests | Registry BPF integration **281/281**, identity BPF **23/23**, unit libs **112 + 6**, RFC-012 structure **21**, geo **4**, API **68** — all green; CI **4/4** |
| Release | `main` = stable (`d22ef61`), CI green; active work on `dev` |
| Off-chain | PostGIS mirror (migrations `0001…0026`), Axum REST API (21 route modules), OSM geo-engine with on-chain road-access digest |
| Frontend | React 19 + Vite: 3D Cesium globe with automatic **2D Leaflet fallback when WebGL is unavailable**, investor progress page (`/progress`), typed API client, wallet adapter |
| Next | **RFC-012 Phase 9 — physical infrastructure** (devices, imagery, 3D scanning), then Phase 10 (cross-border privacy); Devnet deployment; mainnet gates (RFC-005 reconfirm, ZK audit) |

---

## 1. What Terra is trying to become

> Terra is an open physical-world trust and spatial infrastructure protocol.

It is designed to transform observations of the physical world into verifiable
digital records, connect those records to persistent spatial identities, coordinate
independent participants to validate them, preserve their history, and make the
resulting information interoperable across people, organizations, applications and
jurisdictions.

**Land is the first major domain.** The deeper ambition:

> Build an open digital infrastructure through which the physical world can be
> observed, identified, verified, represented, updated and interacted with without
> requiring a single organization to be the sole source of truth.

The blockchain is not the vision. Solana is not the vision. AI is not the vision.
3D is not the vision. **The vision is trusted digital representation of physical
reality.** Those technologies are components that help make that possible.

The architecture explicitly applies beyond land: buildings, infrastructure, natural
resources, agricultural areas, borders/geographic features, physical assets and
environmental observations (see RFC-012 §1).

---

## 2. The fundamental problem

The physical world exists independently of our databases. A piece of land can exist
for centuries while its boundaries are disputed, its owner changes, buildings are
constructed, roads move, rivers change, governments change, documents disappear,
survey measurements disagree, countries use incompatible systems, databases become
inaccessible, and people make competing claims.

Today these realities are fragmented between: government registries, surveyors, maps,
satellite imagery, GPS, drone imagery, paper documents, courts, banks, property
companies, individuals, databases and local knowledge.

Terra's question:

> Can we create an interoperable protocol that connects these observations and claims
> to persistent spatial entities while preserving provenance, uncertainty, validation
> and history?

---

## 3. The mental model

```
PHYSICAL WORLD
                  │
   ┌──────────────┼──────────────┐
   │              │              │
 Land         Buildings    Infrastructure
   │              │              │
   └──────────────┼──────────────┘
                  ↓
           OBSERVATIONS
                  ↓
     EVIDENCE + PROVENANCE
                  ↓
        SPATIAL IDENTITY
                  ↓
   INDEPENDENT VALIDATION
                  ↓
          ATTESTATIONS
                  ↓
            QUORUM
                  ↓
       PROTOCOL FACT
                  ↓
 PERSISTENT SPATIAL RECORD
                  ↓
   ┌──────────────┼──────────────┐
   ↓              ↓              ↓
Rights        History      Transactions
   └──────────────┼──────────────┘
                  ↓
       APPLICATION ECOSYSTEM
```

That is Terra.

---

## 4. The ten major domains

### Domain 1 — Spatial identity

Every important physical entity needs a persistent digital reference. Initially a
parcel; eventually buildings, apartments, roads, bridges, utilities, agricultural
areas, water bodies, forests, infrastructure, natural resources and physical assets.

The identity should not merely be an arbitrary number. It connects:

```
WHO/WHAT + WHERE + GEOMETRY + TIME + EVIDENCE + RELATIONSHIPS
```

A photograph tells you something existed. GPS tells you where something was observed.
A polygon describes geometry. A blockchain transaction records an event. **Identity
connects these things through time.**

### Domain 2 — Physical world observation

RFC-012 treats physical observation as a first-class architectural layer. The planned
ecosystem includes smartphones, GNSS, cameras, drones, satellites, survey equipment,
human observations, documents, APIs and eventually 3D scanning.

The architecture deliberately separates **Person / Device / Location / Observation /
Validation / Registry** rather than treating them as one identity (RFC-012 §4,
Design Rule 7):

> Alice owns the land. Bob operates the drone. The drone captures the image.
> Charlie submits the evidence. David validates the observation.

Five different roles — Terra preserves that distinction.

### Domain 3 — Evidence & provenance

Terra does not say "someone uploaded a picture." It knows: what, who, when, where,
with what device, under what conditions, what was observed, who submitted it, who
validated it, and what other evidence corroborates it.

Implemented vocabulary: `Observation`, `ObservationSource`, `ObservationProvenance`,
`Evidence`, `EvidenceArtifact`, `EvidenceManifest` (RFC-012 Phases 4–5 delivered) —
eventually evidence **chains**, not isolated files.

### Domain 4 — AI / world understanding

> AI becomes Terra's perception and interpretation engine — never its judge.

Satellite/drone imagery → candidate buildings, roads, vegetation, boundary changes,
construction, flooding, land-use change → candidate observations → human/sensor/
validator verification. AI contributes computer vision, geospatial interpretation,
document understanding (deeds, survey plans, permits), anomaly detection (boundary
inconsistencies, duplicated claims, conflicting observations) and reasoning about
*what evidence should be collected*.

**AI never gets unilateral authority to turn an observation into canonical
ownership.** That is RFC-012's core invariant (§0): no AI, oracle, API or validator
can skip `observation → evidence → validation → consensus → state`.

### Domain 5 — 2D → 3D → 4D digital twin

Terra evolves from a 2D parcel into a `SpatialAsset` with X, Y, Z and TIME.
RFC-012 already defines `SpatialAsset`, `GeometryVersion`, `ThreeDModel` and
`SpatialSnapshot`, and states 2D and 3D are native (Design Rule 9):

- **2D** — parcel boundary
- **2.5D** — elevation, terrain
- **3D** — buildings, floors, apartments, air rights, underground structures
- **4D** — spatial state + time: "What was physically present in 2022?", "What
  changed in 2025?", "Which geometry version existed at that moment?"

3D models stay **off-chain**; Terra records hashes, metadata, versions and references
(Design Rule 10).

### Domain 6 — Land rights & identity

```
Person | Organization | Trust | Estate | Government | Cooperative
                              │
                             RIGHT
                              │
                         SPATIAL ASSET
```

`LegalSubject`, `OwnershipRight`/`LandRight`, jurisdiction, start/end dates,
restrictions and evidence are already distinguished in the architecture.

> Terra does not say "the blockchain declares you legally own this land." It records
> a verifiable protocol history concerning this claim/right, while legal recognition
> remains a matter of the applicable jurisdiction.

### Domain 7 — Identity, privacy & cross-border recognition

Identity **without** a mandatory centralized identity provider: `Identity`,
`LegalSubject`, `Credential`, `DeviceIdentity`, `ZKProof` (RFC-006, RFC-011).

```
Cameroon identity → Terra identity → cross-border proof → Country B
```

…without exposing the person's full identity, portfolio or private transactions:
**prove what needs to be proven without revealing everything.** ZK is a privacy
mechanism, not an authority mechanism.

### Domain 8 — Verification network

Instead of "trust the database": **"Who independently verified this, using what
evidence and capability?"**

- Basic validators — people who can perform basic verification
- Specialized validators — surveyors, legal professionals, drone operators, GIS
  specialists, physical inspectors, document validators
- High-trust participants — organizations and highly capable validators for hard cases

Capability, presence, availability, reputation and relationships are modeled
separately. Selection is multi-factor (eligibility, capability, jurisdiction,
geographic relevance, availability, reputation, independence, conflict filtering,
randomization) — never "nearest person wins" (RFC-012 Design Rule 3, delivered in
Phase 6 `routing.rs`).

### Domain 9 — Verification marketplace / physical data network

> "Verify this parcel." → VERIFICATION TASK → requirements, evidence needs, geo
> constraints, capabilities, min reputation, validator count, confidence target,
> reward, deadline → routed to suitable participants → observations → evidence →
> attestations → consensus → requester pays → validators rewarded → coverage
> incentives subsidize strategically important areas.

The full task/economics pipeline is delivered on-chain (RFC-012 Phases 3 and 6–8:
`verification_task.rs`, `routing.rs`, `task_economics.rs`). Terra can become not
merely a database of land but **a global network for producing trustworthy
physical-world data**.

### Domain 10 — Transactions & real-world applications

Identity + spatial asset + rights + evidence + trust + history enable transactions:
property sales (buyer → property → verified evidence → rights → escrow → settlement —
RFC-004 delivered), leasing, mortgages/collateral verification, insurance, development,
construction, marketplaces, valuation, agricultural finance, analytics, due diligence,
cross-border transactions.

Terra does not build every application — it provides the infrastructure others build
on. **Anchor abstracts banking infrastructure; Terra can abstract trusted
physical-world infrastructure.**

---

## 5. Government is not outside the system

Terra is not "government vs Terra":

```
                    TERRA
                      │
      ┌───────────────┼───────────────┐
      ↓               ↓               ↓
  Citizens       Companies       Governments
      ↓               ↓               ↓
  Surveyors          Banks            Courts
      └───────────────┼───────────────┘
                      ↓
              Shared protocol
```

A government can publish "according to the national cadastral authority, this
document is recognized" — recorded as a **jurisdictional assertion** — without
becoming the central operator of Terra. RFC-012 §3 keeps three distinct: **protocol
authority** (smart-contract rules), **jurisdictional authority** (external
organization publishes information), **evidence authority** (surveyor, citizen,
drone operator, satellite provider).

Terra doesn't eliminate governments. It creates a protocol through which governments,
courts, surveyors, citizens, companies and other evidence providers can participate
without one actor operating the whole system.

---

## 6. Global jurisdiction layer

Every country has different tenure systems, legal definitions, cadastral structures,
ownership rules, documentation, authorities and privacy requirements. Terra needs a
**country configuration layer, not a different blockchain per country**:

```
              TERRA CORE
                   │
     ┌─────────────┼─────────────┐
     ↓             ↓             ↓
 Cameroon       Nigeria        Kenya
   config         config        config
     ↓             ↓             ↓
 local rules    local rules   local rules
```

Same protocol, different jurisdictional configuration — how an African prototype
becomes a global platform (product roadmap: country configuration layer).

---

## 7. Disputes & correction

A world-scale system cannot assume everyone is honest:

```
Claim → Evidence → Validation → Consensus → Dispute?
      → Independent review → Decision → Appeal
```

Delivered: disputes, parcel freezes (RFC-007), fraud reports, review committees,
capability restrictions and appeals (RFC-012 Phase 7 `fraud_governance.rs`).

> **Fraud detection must not become automatic punishment.** Suspicious correlation
> ≠ confirmed fraud — due process applies (RFC-012 Design Rules 12–14).

---

## 8. Temporal world history

Terra remembers change: not just "what is this parcel?" but "how did it become what
it is?"

```
2019 Parcel A → 2021 subdivision → 2022 A1+A2 → 2024 building →
2026 expansion → 2030 transfer
```

Geometry versions, spatial snapshots, lineage (RFC-008 subdivision/amalgamation) and
temporal credentials (RFC-009) make the history itself an asset. **Terra becomes a
temporal record of the physical world.**

---

## 9. The complete machine

```
PHYSICAL EARTH
        │
   ┌────┼────┐
   ↓    ↓    ↓
Satellite  Drone   Humans
GNSS       LiDAR   Documents
   └────┼────┘
        ↓
   AI / GIS
        ↓
  OBSERVATION
        ↓
EVIDENCE + PROVENANCE
        ↓
 SPATIAL IDENTITY
        ↓
2D / 3D / 4D MODEL
        ↓
VALIDATOR NETWORK
        ↓
   CONSENSUS
        ↓
TRUSTED DIGITAL STATE
        ↓
RIGHTS + RELATIONSHIPS
        ↓
 TRANSACTIONS
        ↓
APPLICATION ECOSYSTEM
```

---

## 10. What the blockchain does (and does not do)

The blockchain is **not** the map, the satellite, the AI, the database of giant 3D
models, the government, the surveyor or the legal system. It provides a shared,
tamper-evident **coordination and history layer**:

> The blockchain records who attested to what. It does not record how the physical
> verification itself was performed, and it never touches key material.

## 11. What AI does (and does not do)

- **AI =** perception, extraction, classification, prediction, anomaly detection,
  evidence assistance, task planning, reasoning.
- **Terra =** identity, provenance, validation, consensus, rights, history,
  interoperability.

> AI helps Terra understand the world. Terra helps establish how that understanding
> becomes a trusted record.

---

## 12. The ultimate ecosystem

```
                        TERRA
       ┌──────────────────┼──────────────────┐
       ↓                  ↓                  ↓
  PHYSICAL             TRUST              IDENTITY
    WORLD               LAYER              LAYER
 sensors             validators           people
 satellites          evidence             orgs
 drones             consensus             devices
 GNSS                disputes               ZK
       └──────────────────┼──────────────────┘
                          ↓
                   SPATIAL LAYER
              ┌───────────┼───────────┐
              ↓           ↓           ↓
            2D            3D          4D
         parcels      structures   history
                          ↓
                 RIGHTS / ASSETS
                          ↓
                  TRANSACTIONS
                          ↓
                APPLICATION LAYER
     ┌─────────┬──────────┼─────────┬─────────┐
     ↓         ↓          ↓         ↓         ↓
  Finance  Insurance    Gov    PropTech  Agriculture
```

---

## 13. The development journey (with current status)

| Stage | Name | Status | Anchored in |
|-------|------|--------|-------------|
| **0** | Architecture | ✅ **Done** | LADM research, country-agnostic model, RFC-003 → RFC-012, 281 on-chain tests |
| **1** | Protocol hardening | ✅ **Done** | RFC-012 Phases 0–8, Tier A1–A3, B6 boundary, P0-2 removal path — security, tasks, observations, provenance, routing, reputation, economics |
| **2** | Physical Terra | ⬜ **NEXT** | RFC-012 Phase 9 — smartphones, GNSS, cameras, drones, survey devices, satellite imagery, 3D scanning (reuses Phase 4 observation sources/provenance) |
| **3** | Spatial intelligence | ⬜ Planned | Raw observation → AI/GIS → geometry → parcel → SpatialAsset → 3D model (computer vision, photogrammetry, remote sensing, LiDAR) |
| **4** | Digital twin | ⬜ Planned | Identity + geometry + elevation + structure + evidence + history + relationships per spatial entity (`SpatialAsset`, `GeometryVersion`, `ThreeDModel`, `SpatialSnapshot` already specified) |
| **5** | 3D / vertical / subsurface | ⬜ Planned | Surface → buildings → floors → units → airspace → underground (product roadmap: legal 3D/air-rights layer) |
| **6** | Jurisdiction integration | ⬜ Planned | Country configurations: tenure, legal categories, authorities, credentials, evidence rules (product roadmap: country config layer) |
| **7** | Cross-border Terra | ⬜ Planned | RFC-012 Phase 10 — jurisdiction bindings, ZK identity/ownership, selective disclosure |
| **8** | Transaction layer | 🟡 **Seeded** | RFC-004 escrow delivered; marketplaces, finance, insurance, leasing to build on verified spatial assets |
| **9** | Terra infrastructure | ⬜ Future | Terra becomes the shared layer others build on (Anchor analogy): Terra-powered finance, insurance, government services, PropTech, agriculture |
| **10** | Global physical-world network | ⬜ Future | Not a global land registry — **a global interoperable trust layer for physical reality**: land, buildings, infrastructure, agriculture, natural resources, environment, borders, physical assets |

---

## 14. How the repository maps to this vision

| Layer | Where | What |
|-------|-------|------|
| On-chain source of truth | `terra-core/programs/terra_registry` | parcels, rights, claims, evidence, validation, tasks, routing, economics, disputes, ZK |
| Identity program | `terra-core/programs/terra_identity` | identity, succession, guardianship (RFC-010) |
| Mirror + API | `terra-core/api` | PostGIS mirror of every on-chain account, REST `/api/v1`, spatial endpoints |
| Geo engine | `terra-core/geo-engine` | OSM fusion, road-access digests anchored on-chain |
| Frontend | `terra-web` | Cesium globe (2D Leaflet fallback), investor progress page `/progress`, wallet adapter, typed API client |
| Architecture contract | `docs/rfc-012-…` | core invariant, entity catalog, design rules, phases 0–10 |
| Protocol specs | `docs/rfc-003…011` | vault, escrow, staking, cross-border, disputes, subdivision, time-bound, guardianship, ZK |
| Repo status & commands | `../README.md`, `../terra-core/README.md` | current numbers, CI, run/test instructions |
| Security posture | `../terra-core/SECURITY.md` | closed findings, open mainnet gates |

**Reading order for a new human or AI:** this file → root `README.md` →
`terra-core/SECURITY.md` → `terra-core/README.md` → `terra-core/docs/architecture.md`
→ `docs/rfc-012-…` §8.1 handoff → individual RFCs.

---

## 15. The three sentences to keep at the top

1. > Terra is an open physical-world trust infrastructure protocol that gives
   > real-world entities persistent spatial identity and verifiable history by
   > connecting observations, evidence, independent validation, rights and
   > transactions across time and jurisdictions.
2. > Land is the first domain. 2D parcels are the starting point. The architecture
   > expands into 3D/4D digital twins, physical observation, AI-assisted spatial
   > intelligence, decentralized verification, privacy-preserving identity,
   > cross-border interoperability, government participation and transaction
   > infrastructure.
3. > Terra never treats a single AI model, sensor, person, company, government,
   > oracle or database as absolute truth. The canonical transition is
   > `observation → evidence → validation → consensus → state`.

**Terra — Infrastructure for a Verifiable Digital Representation of the Physical World.
Land is the first proving ground.**
