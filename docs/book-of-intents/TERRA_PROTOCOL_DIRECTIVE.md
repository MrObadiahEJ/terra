# TERRA_PROTOCOL_DIRECTIVE — Long-Term Protocol Architecture

Including the Terra File and Rust-first SDK extension.

**Status:** `SPECIFIED ONLY` — governing architectural directive. Evidence for
implementation lives in [IMPLEMENTATION_STATUS.md](IMPLEMENTATION_STATUS.md),
[REQUIREMENTS_TRACEABILITY.md](REQUIREMENTS_TRACEABILITY.md), and the commit
history. No code claims are made here merely because they are documented.

**Repository:** https://github.com/MrObadiahEJ/terra
**Active branch:** `dev`

---

## PURPOSE

Evolve Terra into an open, extensible, fully decentralized protocol for
representing, constructing, verifying, composing, packaging and continuously
updating digital representations of the physical world.

### Important working rule

Do not assume what is implemented. Inspect the actual repository, active branch,
commit history, specifications, tests, migrations, APIs, SDKs and implementation
status before making recommendations or changes. Identify what exists, what is
incomplete, what is incorrectly implemented and what is only documented.

Do not immediately implement the entire vision. First produce an evidence-based
audit and a phased architecture proposal. Do not rewrite working modules
unnecessarily, create empty packages, or claim a feature is implemented merely
because it is documented.

---

## 1. THE VISION

Terra is not simply a 3D reconstruction application, real-estate platform,
blockchain application or visualization frontend.

Terra is intended to become an open protocol and interoperable infrastructure
for digital representations of reality.

It must support:

- Existing physical objects and environments.
- Land, parcels, buildings, rooms, infrastructure and individual components.
- Creative construction: proposed, designed, simulated and assembled objects.
- Independent 3D reconstruction technologies.
- Surveying, measurements, satellite, drone, LiDAR, photogrammetry and future
  sensing technologies.
- Time-dependent and continuously evolving spatial representations.
- Personal and historical memory reconstruction from permitted photos, videos
  and other evidence.
- Applications across real estate, construction, urban planning, robotics,
  simulation, gaming, cultural heritage and other future domains.

Do not restrict the protocol to today's reconstruction engines, sensors,
hardware, coordinate systems or commercial applications.

---

## 2. SEPARATE THE PROTOCOL, SDK, ENGINES, INFRASTRUCTURE AND APPLICATIONS

Maintain clear boundaries:

### A. Protocol specifications

Stable, open, technology-independent contracts defining identities, objects,
geometry references, geospatial references, evidence, provenance, verification,
versions, relationships, manifests and interoperability.

### B. SDK

Independent, reusable implementations of those specifications. Developers should
be able to install only the capabilities they need.

### C. Engines and adapters

Replaceable integrations for AI reconstruction, surveying equipment,
photogrammetry, LiDAR, satellite imagery, storage, rendering and other
technologies.

### D. Distributed infrastructure

Independent workers, validators, storage providers, contributors, clients and
discovery mechanisms.

### E. Applications

Explorers, real-estate applications, construction tools, memory experiences,
games and other interfaces. No application or company must be required to use
the protocol.

The protocol must remain usable even if the original Terra frontend, company or
infrastructure disappears.

---

## 3. MODULAR SDK — INDEPENDENT CAPABILITIES

Design the SDK as a modular capability system.

Potential module boundaries include:

- `terra-core`
- `terra-identity`
- `terra-spatial`
- `terra-reconstruction`
- `terra-survey`
- `terra-construction`
- `terra-quality`
- `terra-evidence`
- `terra-composition`
- `terra-memory`
- `terra-storage`
- `terra-sync`
- `terra-cryptography`
- `terra-network`
- `terra-economy`

These are proposed boundaries, **not** instructions to create unnecessary
packages immediately.

A surveying developer must not need the reconstruction module. A reconstruction
developer must not need the surveying module. A memory reconstruction
application must not require the construction module.

Modules must communicate through documented, stable, versioned interfaces. Avoid
hidden dependencies and circular dependencies.

Each module must declare:

- Its capabilities.
- Supported interface and schema versions.
- Dependencies.
- Optional integrations.
- Compatibility requirements.
- Security assumptions.
- Upgrade and migration behavior.

Support independently versioned and released modules. A change to one module
must not unnecessarily force every other module or every application to upgrade.

---

## 4. PROGRESSIVE AND BACKWARD-COMPATIBLE EVOLUTION

Terra must evolve without requiring the entire network to stop.

Design for:

- Semantic versioning.
- Versioned schemas and interfaces.
- Capability negotiation.
- Backward-compatible extensions.
- Explicit migrations.
- Deprecation periods.
- Independent module releases.
- Multiple protocol versions operating during transition.
- Rolling deployments and progressive adoption.
- Immutable historical records and reproducible interpretation.

A new AI engine, surveying device, storage provider or rendering technology
must be integrable through an adapter without rewriting the protocol.

Do not make current technology assumptions permanent protocol requirements.

Distinguish stable foundational concepts from replaceable implementation
details.

---

## 5. THE TERRA FILE — PORTABLE SPATIAL WORLD PACKAGE

Develop the specification for a portable, extensible, geographically selectable
spatial package using the proposed `.terra` file extension.

A Terra File must represent a selected geographical area at any scale:

- A specific object.
- A building or room.
- A parcel or neighbourhood.
- A city or region.
- A country.
- A continent.
- The entire Earth.
- Potentially other planetary environments in the future.

The user must be able to select an area of interest, choose the desired data
layers and generate one logical Terra File containing the available information
for that area.

Do not assume that every location has complete data. Represent missing coverage,
unknown values, data age, quality and uncertainty explicitly.

A Terra File is one portable logical package, not necessarily one monolithic 3D
model or one unpartitioned binary blob.

It must be capable of holding millions of independently addressable objects,
spatial components, tiles, datasets and metadata.

---

## 6. SPATIAL SELECTION AND EXPORT

Design an export system that accepts:

- Geographic bounding boxes.
- Polygons and arbitrary areas of interest.
- Existing spatial object identifiers.
- Administrative areas where reliable boundary data exists.
- Selected spatial layers.
- Desired levels of detail.
- Time ranges or historical snapshots.
- Quality thresholds and availability requirements.

The system should calculate which spatial objects, components, tiles and
evidence belong to the selected area.

Define how objects crossing the selection boundary are handled. Support
clipping, complete-object inclusion and reference-only inclusion as explicit
export policies.

Support resumable exports, progress reporting, integrity checks, cancellation
and large-area partitioning.

Allow users and software agents to request a complete self-contained export or
a selective export.

---

## 7. OPTIONAL DATA LAYERS

The package must support independently selectable and extensible layers.

Examples:

- Terrain and elevation.
- Buildings and structures.
- Roads and transport infrastructure.
- Water bodies.
- Vegetation and natural features.
- Parcel boundaries and cadastral identifiers.
- Ownership and property-right information, where legally accessible.
- Construction and building components.
- Historical observations and temporal states.
- Infrastructure networks.
- Environmental and sensor observations.
- Personal memory reconstructions, when authorized.
- Application-specific data.

The default must not silently include sensitive or legally restricted
information.

A user should be able to request only the layers needed for their application.

Each layer must declare its schema, version, coverage, provenance, timestamp,
quality, permissions, license and dependencies.

Support future extensions without changing the core package format.

Parcel boundaries must not automatically be treated as proof of ownership.
Legal rights and ownership information must include jurisdiction, source, date,
verification status and legal limitations.

---

## 8. PORTABLE PACKAGE FORMAT

Design a versioned container specification for `.terra`.

Evaluate ZIP/ZIP64 compatibility and other suitable open container approaches,
but do not select a container technology without analyzing large-world
scalability, random access, streaming, indexing, compression, integrity,
updates, backward compatibility and implementation complexity.

The format should support:

- A required manifest.
- Package identity and version.
- Geographic coverage and spatial reference systems.
- Object and component indexes.
- Layer manifests.
- Spatial tiles and level-of-detail hierarchies.
- Native geometry and artifact references.
- Metadata and provenance.
- Quality and validation records.
- Cryptographic integrity information.
- Optional signatures and attestations.
- Temporal and version information.
- Extension manifests and dependencies.
- Access, licensing and privacy declarations.

Preserve original representations where possible. Do not force all 3D data into
one geometry format.

Support efficient indexing, partial extraction, random access and selective
loading.

A reader must be able to inspect the package manifest and understand its
contents without extracting every asset.

---

## 9. TWO PRIMARY PACKAGE MODES

Support:

### A. SELF-CONTAINED MODE

All selected assets are included. The package can be transferred, verified,
inspected and used offline without requiring a Terra-operated server or
internet connection.

### B. REFERENCED MODE

The package includes manifests, object identities, metadata, cryptographic
proofs and references to distributed artifacts. Assets can be retrieved from
independent storage providers when needed.

Also support HYBRID MODE.

Offline access must not depend on a centralized authentication, API or
discovery service. Define how locally available verification material and keys
are handled.

A referenced package must clearly identify unavailable or inaccessible assets
rather than pretending to be self-contained.

---

## 10. DISTRIBUTED LARGE-SCALE PACKAGING

Do not build a whole-world export as one memory-intensive operation.

Design for:

- Spatial partitioning and hierarchical tiles.
- Streaming package creation and reading.
- Resumable downloads.
- Content-addressed artifacts.
- Deduplication.
- Compression.
- Integrity verification.
- Selective retrieval.
- Multiple levels of detail.
- Independent storage providers.
- Distributed package assembly.

A whole-world request must be representable as one logical Terra File even when
its physical storage consists of many chunks or distributed artifacts.

Avoid mandatory central package-generation infrastructure. Independent nodes and
compatible implementations should be able to create and verify packages.

---

## 11. INTEROPERABILITY AND APPLICATION INTEGRATION

Create a stable import/export interface.

Evaluate adapters for:

- Blender.
- GIS applications.
- glTF/GLB.
- OGC 3D Tiles.
- GeoJSON and appropriate geospatial formats.
- LAS/LAZ and point-cloud formats.
- IFC and building information systems.
- Robotics and simulation environments.
- Game engines.
- AI agents and machine-learning workflows.

A Terra File should be independently readable without the official Terra
frontend.

Exports to common formats may lose Terra-specific metadata. Document these
losses and provide ways to retain the original package and associated proofs.

Design plugin interfaces so external applications can consume selected layers
without implementing the entire Terra protocol.

---

## 12. RUST-FIRST SDK STRATEGY

Evaluate Rust as the primary implementation language for the protocol core and
official reference SDK.

Prefer Rust for:

- Core data structures.
- Manifest and package readers/writers.
- Schema validation.
- Spatial indexing and transformations.
- Cryptographic integrity verification.
- Synchronization and version management.
- High-performance package operations.
- Native SDK APIs.

However, do not require all external developers to use Rust.

Plan language bindings and interoperable interfaces for:

- Python.
- JavaScript/TypeScript.
- WebAssembly.
- Mobile environments.
- Other languages when justified.

AI reconstruction engines may use Python or other appropriate technologies
behind replaceable adapters.

Do not force Rust into GPU workloads or technologies where it offers no
meaningful advantage.

Measure performance and memory usage with reproducible benchmarks.

---

## 13. RECONSTRUCTION, SURVEYING AND QUALITY

Support independent reconstruction engines through adapters. Do not privilege
one AI model or reconstruction method.

Define a quality framework that evaluates at least:

- Geometric accuracy.
- Geospatial accuracy.
- Completeness.
- Visual fidelity.
- Structural consistency.
- Uncertainty.
- Provenance and evidence reliability.

Support different quality profiles for visualization, real-estate
documentation, engineering, construction and survey-grade applications.

Do not treat visual quality metrics as proof of physical accuracy.

Require independent reference measurements and control data where the intended
quality level demands them.

Never allow a contributor to establish their own trusted validation state
merely by declaring it in a submitted package.

Preserve alternative reconstructions and conflicting evidence instead of
silently averaging or discarding them.

---

## 14. CREATIVE CONSTRUCTION AND SPATIAL COMPOSITION

Terra must support both reconstruction of existing reality and creative
construction of proposed environments.

Support:

- Individual building components.
- Component hierarchies and dependencies.
- Proposed and existing states.
- Assembly of independently created components.
- Construction stages and as-built comparisons.
- Spatial alignment and transformations.
- Overlap and conflict detection.
- Missing areas and incomplete coverage.
- Progressive detail and level of detail.
- Large-scale spatial environments.

Clearly distinguish observed, measured, inferred, simulated, designed and
proposed content.

Multiple independent objects must be composable into one logical spatial world
without requiring one central 3D engine or one giant centralized model.

---

## 15. ASYNCHRONOUS WORLD UPDATES

Design Terra as a continuously evolving spatial system.

Independent contributors should be able to update individual objects or
components without blocking the entire environment.

Consider immutable artifacts, signed version records, append-only event
histories, manifests, dependency graphs, conflict resolution and atomic
publication of accepted versions.

A client should be able to load an available version and progressively receive
newer compatible versions.

A change to a roof reconstruction should not require rebuilding an entire city.
A new surveying adapter should not break memory reconstruction applications.

Define how concurrent updates are handled, how conflicts are resolved and how
clients know which version they are viewing.

Do not equate real-time synchronization with instant physical truth.

---

## 16. MEMORY RECONSTRUCTION

Treat memory reconstruction as a future application of the general protocol,
not as a reason to hardwire personal-memory logic into the core.

A compatible application could use authorized photos, videos, audio, timestamps
and other evidence to reconstruct historical environments.

Support temporal scenes, camera trajectories, spatial relationships, avatars
and navigable experiences.

Crucially, distinguish:

- Directly observed evidence.
- Algorithmically reconstructed content.
- Inferred content.
- AI-generated or speculative content.

Preserve consent, privacy, ownership, access control and provenance. Never
represent generated details as verified historical facts.

---

## 17. FULL DECENTRALIZATION

Treat decentralization as a fundamental architectural constraint.

Avoid mandatory centralized:

- APIs.
- Databases.
- Storage providers.
- Reconstruction services.
- Validation authorities.
- Discovery servers.
- Governance operators.
- Frontends.

Support independent implementations, distributed storage, peer-to-peer
communication, multiple discovery providers, cryptographic verification and open
governance.

No single organization, company, server, validator, SDK maintainer or blockchain
should have unilateral control over the protocol.

Solana may be supported as an adapter for economic operations, transactions or
anchoring, but do not make the entire protocol dependent on one chain.

Design for provider replacement, network partitions, unavailable nodes, malicious
participants, Sybil attacks, collusion, key compromise and conflicting evidence.

Decentralization must be demonstrated through architecture and failure analysis,
not simply asserted in documentation.

---

## 18. OPEN INTEROPERABILITY

Prefer established open standards where appropriate.

Evaluate compatibility with:

- OGC standards and 3D Tiles.
- GeoJSON and relevant OGC geospatial APIs.
- glTF/GLB.
- LAS/LAZ and COPC.
- IFC and relevant building-information standards.
- Point clouds, meshes and Gaussian Splat representations.
- Relevant geodetic and coordinate reference standards.

Do not force every representation into one universal geometry format. Preserve
native data and provide interoperable references and adapters.

Support future spatial reference systems and potentially other planetary bodies
through extensible reference-frame definitions.

---

## 19. REQUIRED WORK PROCESS

Before changing code:

1. Identify the actual branch and commit.
2. Read the existing architecture and implementation.
3. Map existing modules, dependencies, APIs and data contracts.
4. Identify duplicate abstractions and architectural contradictions.
5. Inspect tests and determine what is actually verified.
6. Identify security, decentralization and upgrade risks.
7. Compare documentation with implementation.
8. Produce a gap analysis against this directive.

Then propose a phased implementation plan.

For every proposed change, state:

- Why it is needed.
- Which existing code it affects.
- Whether it changes a stable interface.
- Backward-compatibility implications.
- Migration requirements.
- Tests required.
- Decentralization implications.
- What must remain optional.

Do not rewrite working modules just to match a new naming convention. Do not
create empty SDK packages or pretend a documented feature is implemented.

Prioritize the smallest useful architectural changes that improve long-term
extensibility without destabilizing the current project.

---

## 20. REQUIRED FIRST DELIVERABLES

Before implementing the format, deliver:

1. An evidence-based assessment of existing Terra package, geometry, object,
   component and spatial APIs.
2. A proposed Terra File specification.
3. A comparison of ZIP/ZIP64 and alternative container approaches.
4. A sample manifest and illustrative package layout.
5. A design for geographical selection and layer-based exports.
6. A strategy for whole-world, offline and distributed packages.
7. A Rust-first SDK architecture with language bindings.
8. An interoperability plan.
9. A backward-compatibility and migration strategy.
10. Tests and security considerations.
11. A stable-core versus replaceable-capabilities classification.
12. A decentralization threat and failure analysis.
13. A prioritized implementation roadmap.
14. Specific code paths and tests supporting every finding.

Do not create an empty `.terra` extension or claim to support a package format
before its specification, reader, writer and verification tests exist.

---

## CORE DESIGN PRINCIPLE

**The Terra File is the portable container.** **The Terra Protocol defines its
meaning.** **The Terra SDK allows software to create, read, verify and modify
it.**

These three should evolve independently.

The goal is to make the Terra File a portable, extensible and independently
usable representation of selected parts of the physical world — not a
proprietary archive that only Terra applications can open.

A person or software agent should be able to download a Terra File containing a
city, disconnect from the internet, open or process it with compatible tools,
use its spatial information in robotics or simulation, analyze it with AI, or
build on it without asking Terra for permission or depending on Terra-operated
servers.

---

## Relationship to existing Book documents

| Directive area | Existing Book coverage | Status |
| --- | --- | --- |
| §1 Vision | `TERRA_VISION.md` | Covered — foundational vision; this directive extends it with protocol packaging |
| §17 Decentralization | `TERRA_VISION.md` §5, `ARCHITECTURAL_DECISIONS.md` | Covered in principle; this directive adds failure analysis requirement |
| §18 Interoperability | Chapters 11/13, `rfc-015` | Partially covered (adapters, component package) |
| §2–4, §5–16, §12–14, §19–20 | Not covered | **New** — this directive is the governing source |
