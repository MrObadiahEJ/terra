# Terra — A Universal Land Registry for Humankind

> A country-agnostic, blockchain-anchored land administration platform. Built first for Cameroon. Designed from day one to belong to the world.

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
        ┌─────────────▼──────────────┐
        │ Layer 2 — Infrastructure   │  roads, zoning, access flags
        ├─────────────────────────────┤
        │ Layer 1 — Legal Volume     │  rights, air/mineral, subdivisions
        ├─────────────────────────────┤
        │ Layer 0 — Ground Parcel    │  surveyed boundary + owner
        └─────────────────────────────┘
```

---

## 🧱 Technology Stack

### Frontend — `terra-web`
- **React 18** + **Vite** (chosen over Create React App for lower memory/build overhead on constrained hardware)
- **Tailwind CSS** for styling
- **Leaflet / MapLibre GL** for map rendering (open-source, no per-tile billing — unlike Google Maps)
- **@solana/web3.js** + **@coral-xyz/anchor** for on-chain interaction
- **Zustand** or React Context for lightweight state management (avoiding heavier Redux overhead)

### Backend — `terra-core`
- **Rust** — core language across both on-chain and off-chain services
- **Anchor** framework — Solana smart contract (program) development
- **PostGIS** (PostgreSQL + spatial extension) — off-chain geospatial fusion database for parcels, roads, and terrain
- **Axum** or **Actix-web** — REST/API service layer between the frontend and both PostGIS + Solana
- **Solana Devnet → Mainnet** — blockchain layer for immutable ownership records and validation-flag hashes

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

- **Reachability check**: graph search from parcel to nearest public road node (BFS/Dijkstra over parcel-adjacency graph)
- **Frontage check**: distance from parcel boundary to nearest road-graph edge
- Result stored on-chain as `{status, evidence_hash, computed_at, computed_by}` — geometry itself stays off-chain in PostGIS; only the auditable result and its hash go on Solana

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
- [ ] **Phase 1 — Devnet MVP** *(current, 7-day build)*: flat parcel registry, Anchor program, basic ownership transfer
- [ ] **Phase 2 — Pilot data layer**: PostGIS fusion database, OSM road graph ingestion, drone photogrammetry for one pilot zone (Soa/Biteng)
- [ ] **Phase 3 — Road-access validation**: off-chain reachability algorithm, on-chain flag hashing
- [ ] **Phase 4 — 3D/air-rights layer**: legal volume extension (LADM Part 1 extension)
- [ ] **Phase 5 — Country config layer**: tenure-type abstraction, multi-authority attestation
- [ ] **Phase 6 — Regional expansion**: Central Africa -> Africa -> World
- [ ] **Phase 7 — Global platform**: smart-city digital twin integration, cross-border interoperability

---

## 📁 Monorepo Structure

```
terra/
├── terra-web/              # React + Vite frontend
│   ├── src/
│   │   ├── components/
│   │   ├── pages/
│   │   ├── hooks/
│   │   └── lib/             # Solana/Anchor client bindings
│   └── package.json
├── terra-core/              # Rust workspace
│   ├── programs/
│   │   └── terra_registry/  # Anchor on-chain program
│   ├── api/                 # Axum/Actix off-chain API service
│   └── geo-engine/          # Rust geospatial validation logic (road-access, boundary checks)
├── data/
│   ├── postgis/              # DB schema + migrations
│   └── scripts/              # OSM ingestion, photogrammetry pipeline helpers
├── docs/
│   └── ladm-mapping.md       # LADM layer-to-schema mapping reference
└── README.md
```

---

## ⚙️ Setup — Commands to Start the Project

> Run these in order. Frontend and backend are separate workspaces inside one monorepo.

### 1. Create the monorepo root

```bash
mkdir terra && cd terra
git init
```

### 2. Scaffold the frontend (`terra-web`)

```bash
npm create vite@latest terra-web -- --template react
cd terra-web
npm install
npm install tailwindcss @tailwindcss/vite leaflet react-leaflet @solana/web3.js @coral-xyz/anchor zustand
cd ..
```

### 3. Scaffold the backend Rust workspace (`terra-core`)

```bash
cargo new terra-core --name terra-core
cd terra-core
```

Create a `Cargo.toml` workspace at `terra-core/Cargo.toml`:

```toml
[workspace]
members = ["programs/terra_registry", "api", "geo-engine"]
resolver = "2"
```

### 4. Initialize the Anchor on-chain program

```bash
# install Anchor CLI if not already present
cargo install --git https://github.com/coral-xyz/anchor avm --locked --force
avm install latest
avm use latest

cd terra-core
anchor init programs/terra_registry --no-git
```

### 5. Scaffold the off-chain API service

```bash
cargo new api --name terra-api
cd api
cargo add axum tokio --features tokio/full
cargo add sqlx --features postgres,runtime-tokio-rustls
cd ..
```

### 6. Scaffold the geospatial validation engine

```bash
cargo new geo-engine --name terra-geo
cd geo-engine
cargo add geo geo-types postgis
cd ../..
```

### 7. Set up PostGIS (local development database)

```bash
# using Docker — lightweight, isolated, avoids native install overhead
docker run --name terra-postgis -e POSTGRES_PASSWORD=terra -e POSTGRES_DB=terra_dev -p 5432:5432 -d postgis/postgis:16-3.4
```

### 8. Verify everything is wired

```bash
# from terra-core/
anchor build
anchor test

# from terra-web/
npm run dev
```

---

## 🤝 Contributing

This project is intended to remain open-source and community-governed as it grows beyond its country of origin. Contribution guidelines, code of conduct, and governance structure will be published as the project moves past the Devnet pilot phase.

## 📜 License

*(To be finalized — recommend a permissive open-source license such as Apache-2.0 or MIT to maximize adoption across jurisdictions, given the multi-country ambition of this project.)*

## ⚠️ Disclaimer

Terra is a technical infrastructure project and does not itself confer legal title to land. Legal ownership remains governed by the applicable national law of the jurisdiction in which a parcel is located. Terra's role is to make locally-recognized rights more verifiable, portable, and fraud-resistant.