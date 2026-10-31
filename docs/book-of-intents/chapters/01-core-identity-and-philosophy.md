# Chapter 1 — Core Identity & Philosophy

> **Status:** `IMPLEMENTED AND VERIFIED` as a narrative (every claim cites code
> or tests; audit basis `775e20a`, 2026-10-31)
> **Evidence:** `terra-core/programs/terra_registry`, `terra-core/programs/terra_identity`,
> `terra-core/api`, migrations `0001`–`0032`, suites listed at the end.

This chapter is the non-negotiable foundation. Every later chapter assumes it.
Nothing in this chapter is aspirational; each statement is traceable to code or
a test in this repository.

---

## 1.1 What Terra is

Terra is an **open-source, country-agnostic spatial trust and verification
protocol** (a DePIN) built on Solana. It gives real-world entities a
**persistent spatial identity** and a **verifiable history** that any party can
inspect without asking Terra for permission.

Integrators can plug into Terra as a permissionless layer. Developers can build
upon the on-chain accounts and the off-chain mirror to solve their own problems
— land registries, PropTech due-diligence, AI ground-truth feeds, cross-border
identity — without a commercial agreement.

## 1.2 What Terra is not

Terra is **not** a business, a SaaS company, or a proprietary product. It is
public infrastructure. Two consequences follow, and they are enforced in the
architecture, not merely stated:

- **No geographic gatekeeping.** `world_registry.rs` validates ISO 3166-1
  alpha-2 codes for all 249 officially assigned countries (excluding
  user-assigned ranges). There is no privileged jurisdiction. Any country can
  register; the genesis council itself requires confirmations from at least
  **3 distinct countries** (`GENESIS_MIN_DISTINCT_COUNTRIES = 3`) before the
  registry flips to peer consensus (`CONSENSUS_FLIP_THRESHOLD = 4`).
- **Terra does not assert legal ownership.** The protocol records *claims* and
  the *evidence* behind them, then cryptographically verifies that evidence
  through independent validators. A court, not Terra, adjudicates title.

## 1.3 The Golden Rule: `Claims ≠ Facts`

The single most important invariant in the system is the canonical
distinction:

```
OBSERVATION ≠ EVIDENCE ≠ VERIFICATION ≠ CONSENSUS ≠ LEGAL RECOGNITION
```

Within the protocol, that distinction is enforced by the state transition:

```
observation → evidence → validation → consensus → state
```

A claim ("Alice owns parcel P") is only promoted to state after it has passed
through this pipeline. Each stage is a distinct, separately tested layer:

| Stage | What it means | Where it lives |
| --- | --- | --- |
| `observation` | A physical-world reading is submitted (human, document, API, or 3D scan). | `observation_v2.rs`, `device_identity.rs` |
| `evidence` | The observation is bound to artifacts whose content is hashed (SHA-256) and referenced on-chain. | `evidence_manifest.rs`, `canonical-geometry-digest.md` |
| `validation` | Independent validators attest to the evidence under capability, jurisdiction, and reputation constraints. | `verification/` (`attestation`, `quorum_voting`, `reputation`), `routing.rs` |
| `consensus` | A quorum of independent attestations must agree before state changes. | `quorum.rs` |
| `state` | The parcel's versioned geometry / rights / status is advanced, and prior versions remain readable. | `spatial_asset.rs`, `spatial_snapshot.rs`, `right_status` |

The practical rule this encodes: **you may always trace a piece of state back to
the observations and validator signatures that produced it.** A claim that
cannot be traced is not state; it is an unverified claim, and the protocol will
not let it advance.

## 1.4 Why this architecture — the five failures it answers

Terra is built to remove specific, named failures in the physical-world trust
stack. Each one maps to concrete subsystems.

### Failure 1 — Opaque land markets and hidden liens

**Pain.** Buyers (especially diaspora investors) cannot cheaply verify whether a
property carries hidden mortgages, caveats, or court orders. Losses are large
and frequent.

**Terra's answer.** Rights and encumbrances are first-class on-chain state with
an immutable, time-ordered history rather than a paper trail. The building
blocks are `IMPLEMENTED AND VERIFIED`: time-bound rights with expiry, grace, and a 30-day
warning window (`time_bound.rs`), rights status transitions
(`right_status`, `update_right_status`), and parcel freezes under dispute
(`dispute.rs::freeze_parcel`). The **full lien lifecycle** (institutional
flag → transfer blocked without bank signature → auto-release on payment
verification) is specified in [Chapter 6](06-lien-encumbrance.md) and is
currently `PROPOSED` (no code).

### Failure 2 — The "ground truth" gap for AI and PropTech

**Pain.** AI models and PropTech platforms lack reliable, tamper-evident
physical data to reason against, so they hallucinate and make execution errors
about the built environment.

**Terra's answer.** Physical-world observations are a first-class architectural
layer (`observation_v2.rs`, RFC-012 Domain 2), and multi-layer 3D reconstructions
are hashed to a canonical SHA-256 geometry digest and linked to on-chain
ownership and volumetric coordinates. This gives downstream systems a
tamper-evident ground-truth feed instead of an unsourced raster.
See [Chapter 3](03-3d-spatial-intelligence.md) and `docs/rfc-013-spatial-intelligence-pipeline.md`.

### Failure 3 — The institutional identity void

**Pain.** Public chains have no formal way to identify and verify that a
transaction genuinely originates from a government ministry, an NGO, or a
corporation, which makes institutional participation risky.

**Terra's answer.** A DID is published in an auditable off-chain registry (e.g.
a `.gov` DNS TXT record or a recognized chamber-of-commerce entry), then bridged
on-chain by `register_institutional_identity`, verified by a Bootstrap Trust
Council or a decentralized oracle. The entity receives an
`InstitutionalValidator` PDA and is subject to the **same** evidence, slashing,
and reputation rules as any individual validator. This is specified in
[Chapter 8](08-institutional-identity.md) and is `PROPOSED` (no code); the
anti-fraud foundations it depends on (validator profiles, reputation, slashing)
are `IMPLEMENTED AND VERIFIED`.

### Failure 4 — Development economics and soft costs

**Pain.** Friction in due diligence, slow permitting, and fragmented data make
the economics of new housing and infrastructure unviable.

**Terra's answer.** By making evidence and history cheaply verifiable and
portable, Terra compresses due-diligence soft costs. The off-chain PostGIS
mirror and pure-Rust geo-engine expose spatial queries (road-graph reachability,
nearest-neighbor) so that planning and permitting tooling can reason over the
same verified data that the chain anchors. See [Chapter 7](07-postgis-geo-engine.md).

### Failure 5 — Fraud in validator networks

**Pain.** Centralized or easily bribed validator networks compromise the
integrity of physical-world attestations.

**Terra's answer.** Multi-factor, randomized validator routing with weighted
randomness, reputation, independence, and conflict filters — combined with
pattern-based slashing severity tiers and a reporter bond. See
[Chapter 4](04-validator-routing.md). This layer is `IMPLEMENTED AND VERIFIED`.

---

## 1.5 The state transition, in code

This section exists so a reader can verify §1.3 against the repository rather
than trusting prose.

- **Observation → evidence.** `observation_v2.rs` captures the observation;
  `evidence_manifest.rs` binds it to artifacts. Artifact kinds are validated
  (`is_valid_artifact_kind`), storage references are validated
  (`is_valid_storage_reference`), and content hashes must be non-zero
  (`is_nonzero_hash`). Evidence can only be attached in a valid state
  (`can_attach_evidence`), and artifact indices must be sequential
  (`is_expected_artifact_index`), which prevents out-of-order or gap-ridden
  manifests.
- **Evidence → validation.** `verification/attestation.rs`,
  `verification/quorum_voting.rs`, and `verification/reputation.rs` validate
  attestations. Routing (`routing.rs`) enforces the nine-factor selection
  (eligibility → capability → jurisdiction → geographic relevance → availability
  → reputation → independence → conflict → randomized pick), so validation is
  produced by an unpredictable, independent quorum rather than a pickable set.
- **Validation → consensus.** `quorum.rs` gates the state change on a quorum of
  independent attestations.
- **Consensus → state.** `spatial_asset.rs` / `spatial_snapshot.rs` advance
  versioned spatial state; `right_status` transitions advance rights. Prior
  versions stay readable, giving the "verifiable history" property.

## 1.6 Evidence over assertion — how this Book proves itself

Every architectural decision in this Book carries two things:

1. **Why** — the specific pain point from §1.4 it removes.
2. **Proof** — the code path and the test that verify it.

The verification mandates are concrete. The canonical suites must stay green:

| Suite | Command | Baseline |
| --- | --- | --- |
| Registry lib tests | `cargo test -p terra-registry --lib` | ≥ 153 passing |
| BPF integration | `cargo test -p terra-registry --test integration -- --test-threads=1` | ≥ 307 passing |
| API tests | `CARGO_INCREMENTAL=0 cargo test -p terra-api` | ≥ 108 passing |
| Geo-engine tests | `cargo test -p terra-geo-engine` | ≥ 6 passing |
| Identity BPF | (identity integration suite) | ≥ 23 passing |
| Frontend unit (vitest) | `pnpm exec vitest run` (in `terra-web`) | ≥ 44 passing (verified 2026-10-31) |

A chapter that claims a feature is `IMPLEMENTED AND VERIFIED` must name a suite and a test in
this table. A chapter that cannot must use a lower status from the Book's
seven-level vocabulary ([BOOK_OF_INTENTS.md](../BOOK_OF_INTENTS.md)).

## 1.7 Geographic scope

Completely global and country-agnostic. There are no geographic restrictions and
no single-nation focus. The protocol scales anywhere because jurisdiction is
modeled as data (`cross_border.rs`: `register_jurisdiction`,
`verify_jurisdiction_membership`, `bind_cross_border_identity`,
`revoke_jurisdictional_identity`, `rebind_cross_border_identity`) rather than
as a hard-coded region. Binding statuses have explicit, guarded transitions
(`can_transition_binding`) and expiry (`binding_is_expired`).

## 1.8 Sprint-loop discipline

This Book advances through **Research → Plan → Build → Test → Loop**. The
current phase is never a mutable pointer; it is derived from the evidence files
under `work/` and `sprints/`. Work accumulates on `dev`; each sprint opens one
`dev → main` checkpoint as a reversible merge.

The reader should treat any feature in this Book as trustworthy precisely when
it is backed by a passing suite — and should treat `PROPOSED`/`SPECIFIED ONLY` features as
invitations to build, with the specification acting as the contract they must
satisfy.

---

*Continue to [Chapter 2 — Proof of Location & Residency Tokens](02-proof-of-location-residency.md) (`PROPOSED`), or see [Chapter 4 — Validator Routing](04-validator-routing.md) for a fully `IMPLEMENTED AND VERIFIED` feature.*
