# Terra — A Universal Land Registry for Humankind

[![CI](https://github.com/MrObadiahEJ/terra/actions/workflows/ci.yml/badge.svg)](https://github.com/MrObadiahEJ/terra/actions/workflows/ci.yml)

> A country-agnostic, blockchain-anchored land administration platform. Built first for Cameroon. Designed from day one to belong to the world.

---

## Branching Strategy

| Branch | Purpose | Stability |
|--------|---------|-----------|
| **`main`** | Production-ready code. Only fully reviewed, tested, and approved protocols ship here. | Stable |
| **`dev`** | Active development. New protocols, features, and experiments land here first. Merged to `main` after review. | Unstable |

All work-in-progress (new RFCs, protocol implementations, breaking changes) goes to `dev`. When a feature is complete, reviewed, and all tests pass, it is merged into `main`. This keeps `main` deployable at all times.

---

## 🌍 Vision

Land is the oldest form of wealth and the most disputed form of trust. In much of the world — starting with Cameroon — land ownership records are paper-based, centralized, understaffed, and vulnerable to fraud, loss, or manipulation. Terra exists to give any person, in any country, a way to register, verify, and transfer land rights with cryptographic certainty — without requiring them to trust a single government office, company, or server.

Terra is **not** built to replace national law. It is built to make land records **verifiable, portable, and resilient** — a public infrastructure layer that governments, communities, and individuals can build on top of, regardless of legal tradition (civil law, common law, or customary/communal tenure).

**Guiding principle:** the core protocol is universal. Country-specific rules are configuration, not code.

---

## 🏗️ Architecture Overview

Terra is organized around three conceptual layers, based on **ISO 19152 (LADM — Land Administration Domain Model)**, the international standard for cadastral and land administration systems.

### Layer 0 — Ground Parcel (2D)
The base cadastral unit: a surveyed polygon representing a single plot of land, its identifier, and its registered party (owner/rights-holder).

### Layer 1 — Legal Volume (3D rights)
The same parcel extended vertically — mineral rights below, air rights above, multi-story unit subdivisions. Represented as boundary faces/volumes rather than flat 2D shapes once a project reaches this maturity stage.

### Layer 2 — Infrastructure & Plan Layer
Roads, zoning, utilities, and the **road-access / infrastructure-validity flag** — an advisory (non-blocking) layer that detects whether a parcel has legitimate physical access to public infrastructure, without preventing any transaction.

Each layer is its own data structure, cross-referenced by a shared spatial reference frame — never hard-merged into a single schema. This is what allows Terra to stay valid whether the parcel is in Soa (Cameroon), Libreville (Gabon), or Lagos (Nigeria), even though the *legal meaning* of a boundary differs by jurisdiction.

### Country Layer (added last, by design)
Legal tenure types, proof-of-ownership rules, and attesting-authority definitions live in a **configuration layer**, not in program logic. Cameroon, Gabon, Nigeria, and Ghana can each define their own `TenureType`, `AttestingAuthority`, and `ProofStandard` records without requiring a rewrite of the core protocol.

```
        ┌────────────────────────────┐
        │   Country Config Layer     │  ← added last, per-nation
        │ (tenure types, authorities)│
        └────────────┬───────────────┘
                      │
        ┌─────────────▼───────────────┐
        │ Layer 2 — Infrastructure    │  roads, zoning, access flags
        ├─────────────────────────────┤
        │ Layer 1 — Legal Volume      │  rights, air/mineral, subdivisions
        ├─────────────────────────────┤
        │ Layer 0 — Ground Parcel     │  surveyed boundary + owner
        └─────────────────────────────┘
```

---

## 🧱 Technology Stack

### Frontend — `terra-web`
- **React 19** + **Vite 8** (es2022 target), styled with custom design-system CSS utilities
- **CesiumJS 3D globe** for global 3D city rendering (parcel polygons, roads, POIs, draw-mode cadastral capture)
- **pnpm** for dependencies (shared store — deliberate for constrained storage)
- **@solana/web3.js 1.98 + @coral-xyz/anchor 0.32** for on-chain interaction
- **@solana/wallet-adapter + Wallet Standard** for hardened browser wallet signing, with **Ed25519 signature verification** before any transaction is sent
- **Zustand** for lightweight state management

### Backend — `terra-core` (Rust workspace)
- **Anchor** framework — Solana smart contract (program): parcels, rights, transfers, infra flags, and **multi-validator attestations**
- **PostGIS** (PostgreSQL + spatial extension) — off-chain geospatial fusion database for parcels, roads, and terrain
- **Axum 0.8 + sqlx** — REST/API service layer between the frontend and both PostGIS + Solana
- **`terra-geo` (geo-engine)** — pure-Rust OSM road graph + Dijkstra/BFS reachability + SHA-256 canonical digests
- **ed25519-dalek + bs58** — off-chain verification of validator signatures against wallet public keys
- **Solana Devnet → Mainnet** — blockchain layer for immutable ownership records, content-hash anchors, and validation signatures

### Data & Validation Layer
- Off-chain geometry computation (road-access reachability, boundary validation) — never run expensive geometry directly on-chain
- On-chain storage limited to: parcel identity, ownership, rights, and **hashes** of off-chain-computed validation results (auditable, tamper-evident, cheap)

---

## 🗺️ Data Resources

| Resource | Provides | License / Cost | Risk Level |
|---|---|---|---|
| **OpenStreetMap (OSM)** | Road network graph, points of interest | Free, ODbL license, bulk downloadable | ✅ Low — primary source for road-access algorithm |
| **Copernicus Sentinel-2** | Optical satellite imagery (10m resolution) | Free, open, no usage restriction (EU law) | ✅ Low — attribution required on redistribution |
| **Copernicus DEM / SRTM** | Global elevation data | Free, global coverage | ✅ Low — terrain/drainage/slope base layer |
| **Drone photogrammetry (self-captured)** | Centimeter-to-decimeter precision parcel boundaries | Hardware cost only (~$500–1000 drone) + free software (OpenDroneMap/WebODM) | ✅ Low — best precision-to-cost ratio for pilot zones |
| **Google Photorealistic 3D Tiles** | Rendered 3D city visualization | Free tier + paid per-tile beyond quota | ⚠️ Medium — visualization only, not a data export, do not use as source-of-truth geometry |
| **Commercial high-res satellite (Maxar, Planet Labs)** | 30cm–3m resolution imagery | Paid, per km² | ⚠️ Medium — check licensing terms before long-term reliance |
| **MINDCAF cadastral records (Cameroon)** | Authoritative legal ownership records | No public API; requires institutional relationship | 🔴 High uncertainty — future partnership target, not a current resource |
| **MINHDU urban master plans (Cameroon)** | Zoning / road planning documents | Paper/PDF only, no structured geodata | 🔴 High uncertainty — requires manual digitization |
| **National cadastre API (any country)** | — | Does not currently exist for Cameroon or most pilot-target nations | 🔴 Non-existent — do not assume availability |

**Legal note:** Copernicus data is governed by EU law and is free and open for global use — including commercial use, reproduction, and derivative works — with no nationality restriction. This is distinct from sanctions law, which targets specific transactions, not open scientific/earth-observation data. Always attribute per the [Copernicus Sentinel Data Legal Notice](https://sentinels.copernicus.eu/documents/247904/690755/Sentinel_Data_Legal_Notice).

---

## 🚩 Road-Access Validation (Infrastructure Flag)

A non-blocking advisory layer that flags — but never prevents — a land transaction:

- **Reachability check**: Dijkstra over the parcel-adjacent road graph (min-heap), plus a BFS connected-component labeling to measure how much sealed (paved/main) network a parcel can actually reach.
- **Frontage check**: distance from parcel boundary to the nearest road edge (with a `ROAD_ACCESS_THRESHOLD_M` of 50 m).
- Result is reduced to a **canonical SHA-256 digest** — `access_digest(parcel_id || flags || metrics)` — that is anchored on-chain in `Parcel::access_hash`. Geometry itself stays off-chain in PostGIS; only the auditable digest and the derived flag bitmask go on Solana. The API independently recomputes this digest during reconciliation and rejects an inconsistent anchor.

---

## 🧾 On-Chain Attestation & Multi-Validator Validation

Heavy off-chain data (deeds, surveys, contracts, notarizations) is bonded to the on-chain record by a content-hash anchor and **wallet-bound cryptographic signatures** — so a document can be traced to its owning wallet, and each validator can be verified by what they actually signed.

- **On-chain `Attestation` account** (PDA `["attestation", parcel, specifier]`) anchors:
  - `content_hash` — SHA-256 over the off-chain payload (documents/signing artifact)
  - the set of **validator wallets** (up to 8) and a **required threshold** — i.e. *who* must sign off, useful when several parties validate a land purchase
- **Per-validator signatures** are stored off-chain (`validations`) and cryptographically verified with Ed25519 against the wallet's public key over the fixed canonical message `content_hash || onchain_id`.
- An endpoint recomputes each signature's validity and exposes `has_quorum` (valid signatures ≥ threshold), so anyone can confirm from a validator's wallet exactly what they signed and validated.
- **Documents** are bound to a parcel + owner wallet off-chain (`documents` table) with a content hash and storage reference.

This gives non-repudiation (a validator can't deny what they signed) and multi-party approval tracking without bloating the chain with heavy payloads.

---

## 🧑‍⚖️ Identity & Wallet Passation (the failure-modes layer)

Land registries fail when a person isn't in the database, when an owner/heir has
never appeared on the platform, or when a validator/owner dies. Terra handles
these three cases with an on-chain **Identity** account and a time-boxed
**Succession (wallet passation)** mechanism, plus collective validator seizure
for judicial forfeiture.

### Person → wallet binding (even when the system creates the wallet)

Everything ultimately resolves to a **wallet**, because only a wallet can sign.
To bind a *person* to that wallet, an `Identity` account (PDA `["identity",
identity_hash]`) stores a **hash of the person's identity credential** (e.g.
national ID) plus their `owner` wallet and a separate `recovery` wallet.

- The system can **provision** a wallet for a person who has none — but the
  private key is **exported to the person** (seed/paper/biometric). The server
  never holds it, preserving the "security can't be traded" principle.
- The identity **hash** (never the raw credential) is what appears on-chain, so
  the person isn't exposed but the binding is cryptographically resolvable.
- A `validator` and a `classic owner` are the **same wallet primitive** — their
  difference is a *role in state* (`Attestation.validators` vs `Parcel.owner`),
  enforced by program logic, not a different key type.

### Wallet passation (`Succession`) — heirs, recovery, transfer of control

A `Succession` account (PDA `["succession", identity, successor]`) queues a
control transfer that becomes effective only after **two independent gates** are
met: a **configurable grace window** AND a **minimum number of validator
endorsements** (so a stolen wallet can't seize land alone):

- `request_succession` — the owner (or the recovery wallet, for key-loss) names
  a successor: kind `0`=heir, `1`=recovery, `2`=transfer. The requester picks a
  per-request grace window (default 30 days, clamped to [7d, 180d]) and a
  `required_validations` threshold (>= 1) of declared local validators.
- `endorse_succession` — each declared local validator signs the endorsement tx
  with their wallet; each endorsement bumps `validations_count` and is
  immutably recorded in `succession_endorsements` (one endorsement per validator
  per succession).
- `claim_succession` — **only after** `validations_count >= required` **AND**
  `effective_at <= now()` the successor takes over the identity; any owned
  parcels are **re-pointed** to their wallet in the same instruction
  (`remaining_accounts`).
- The **original owner can `cancel`** within the window (no theft).

This two-gate mechanism means:
- a stolen wallet can't claim without the local validators testifying;
- even a colluding thief who knows the successor still needs the full
  validator set to endorse; and
- legitimate heirs far from a local validator get a configurable grace window
  instead of a rigid 7-day cutoff.

### Dead / leaving validators — `rotate_validators`

If validators die and quorum becomes unreachable, the **parcel owner** calls
`rotate_validators` to replace the validator set on the `Attestation` (with a
monotonic `version` bump so a reconstituted set is auditable). Combined with
thresholds below `count`, this means:
- threshold already survives some deaths (`required` < `count`);
- when deaths exceed the slack, **rotation** restores reachability instead of a
  permanent deadlock; and
- if the *owner* is gone too, **succession** passes control to an heir first,
  then the heir rotates the validators.

### Judicial forfeiture (`judicial_forfeiture`) — collective validator seizure

For cases where an owner refuses to release land despite a court order (e.g.
repossession by government, or a court ruling that title passed to another
person), Terra provides a **deliberately heavier** collective forfeiture
mechanism:

- The relaying authority (court/govt channel) calls `judicial_forfeiture` with
  a `case_hash` (SHA-256 of the court order document), `new_owner` (the wallet
  receiving control), a `threshold` (>= 2 validators), and a `validators` list.
- At least `threshold` of the declared validators must **sign the same
  transaction** as `Signer` accounts in `remaining_accounts`, making each
  endorsement cryptographically undeniable (each validator's ed25519 wallet
  signs the tx).
- When the threshold is met, the program transfers `Parcel.owner` to `new_owner`
  and emits a `ParcelForfeited` event with the case hash, from/to, and the
  threshold/present counts for auditability.
- The relaying authority **cannot** be the current owner (prevents self-forfeit
  abuse).
- This is recorded in the `forfeitures` DB table with a `court_relay` fail-safe
  column for off-chain reconciliation.

This makes forfeiture deliberately heavier than a normal transfer: at least 2
validator signers are required, and each signs the actual transaction (not an
off-chain signature), so the collective validator consent is cryptographically
unimpeachable.

These four mechanisms (`Identity`, `Succession` with validator endorsement,
`rotate_validators`, `judicial_forfeiture`) are the recovery layer that makes
the registry resilient to disconnected people, unknown/absent owners, mortality,
and judicial seizure.

---

## 📚 Key References

- ISO 19152 — Land Administration Domain Model (LADM), Parts 1, 2, and 5 (spatial plan / 3D-4D urban integration)
- *Land Administration for Sustainable Development* — Williamson, Enemark, Wallace & Rajabifard
- *Fit-for-Purpose Land Administration* — World Bank / FIG / GLTN
- *The Mystery of Capital* — Hernando de Soto
- Comparable open-source references: `gujarat-landchain`, `SkyTradeLinks/solana-land`, `Auguron/solana-deed-metadata-program`

---

## 🎯 Roadmap & Milestones

- [x] Architecture research — LADM standard, comparable Solana land-registry repos, data-source legal review
- [x] Country-agnostic core data model designed (Parcel / Rights / Owner / InfrastructureFlag)
- [x] **Phase 1 — Devnet MVP**: flat parcel registry, Anchor program, ownership transfer, rights, Cesium globe
- [x] **Phase 2 — Pilot data layer**: PostGIS fusion database, OSM road-graph ingestion, geo-engine
- [x] **Phase 3 — Road-access validation**: off-chain Dijkstra/BFS reachability, on-chain flag + digest hashing
- [x] **Phase 4 — Attestation + recovery**: on-chain content-hash anchor, multi-validator Ed25519 validation, Identity binding, validator-endorsed wallet passation (configurable grace), validator rotation, and judicial forfeiture
- [ ] **Phase 5 — Legal 3D/air-rights layer**: legal volume extension (LADM Part 1 extension)
- [ ] **Phase 6 — Country config layer**: tenure-type abstraction, multi-authority attestation workflows
- [ ] **Phase 7 — Regional expansion**: Central Africa -> Africa -> World
- [ ] **Phase 8 — Global platform**: smart-city digital twin integration, cross-border interoperability

---

## 📁 Monorepo Structure

```
terra/
├── terra-web/                   # React 19 + Vite + CesiumJS frontend
│   ├── src/
│   │   ├── components/
│   │   │   ├── map/             # TerraGlobe (Cesium viewer, drawing)
│   │   │   ├── panels/          # register / parcel / list panels
│   │   │   └── layout/          # navbar (wallet buttons)
│   │   ├── pages/               # GlobePage
│   │   ├── idl/                 # generated IDL + typed Anchor client
│   │   ├── store/               # zustand app store
│   │   └── lib/                 # api, program, wallet, codec, constants
│   └── package.json             # pnpm
├── terra-core/                  # Rust + Anchor workspace
│   ├── programs/terra_registry/ # Anchor on-chain program (parcel/rights/attest)
│   ├── api/                     # Axum 0.8 + sqlx/PostGIS service
│   │   ├── src/routes/          # parcels, geo, fusion, pilot-zones, attestations
│   │   └── migrations/          # 0001..0005 PostGIS schema
│   └── geo-engine/              # OSM graph + reachability + digest (terra-geo)
├── data/
│   └── *.osm.pbf                # OSM extracts for the geo-engine
└── README.md
```

---

## ⚙️ Running Locally

> Frontend and backend are separate workspaces inside one monorepo.

### 1. Database (PostGIS)

```bash
docker run --name terra-postgis \
  -e POSTGRES_PASSWORD=terra -e POSTGRES_DB=terra_dev \
  -p 5432:5432 -d postgis/postgis:16-3.4
```

### 2. Backend (`terra-core`)

```bash
cd terra-core
# configuration lives in api/.env (see api/.env.example)
cargo check --workspace        # confirm everything compiles
cargo test -p terra-geo --lib  # road-access + digest unit tests
cargo run -p terra-api         # serves API on :8080 (migrations auto-applied)
```

The Anchor program:
```bash
anchor build                   # compiles the program + regenerates terra-web/src/idl/*
```

### 3. Frontend (`terra-web`)

```bash
cd terra-web
pnpm install
pnpm dev                       # Vite dev server (proxies /api to :8080)
```

### 4. Verification

```bash
cd terra-web && npx tsc -b && ./node_modules/.bin/eslint . && pnpm build
```

> **Note (storage):** the repo uses **pnpm** for the frontend and gitignores
> `node_modules`, `dist`, and `public/cesium` (Cesium's 7.8 MB static assets are
> copied at build/dev time). The Rust `target/` dir is also untracked.

---

## 🤝 Contributing

This project is intended to remain open-source and community-governed as it grows beyond its country of origin. Contribution guidelines, code of conduct, and governance structure will be published as the project moves past the Devnet pilot phase.

## 📜 License

*(To be finalized — recommend a permissive open-source license such as Apache-2.0 or MIT to maximize adoption across jurisdictions, given the multi-country ambition of this project.)*

## ⚠️ Disclaimer

Terra is a technical infrastructure project and does not itself confer legal title to land. Legal ownership remains governed by the applicable national law of the jurisdiction in which a parcel is located. Terra's role is to make locally-recognized rights more verifiable, portable, and fraud-resistant.