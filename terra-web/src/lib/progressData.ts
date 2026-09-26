/**
 * Static investor-facing progress data.
 *
 * Source of truth: /README.md (Roadmap, Protocol Catalog, Source counts) and
 * terra-core/docs/architecture.md. Figures below were verified 2026-09-26
 * on dev (CI green, registry 283/283, identity 23/23); RELEASE tracks main.
 * Re-verify with `make idl` + `make test-fast` before updating numbers.
 */

export interface Metric {
  value: string
  label: string
  detail: string
}

export interface RoadmapItem {
  done: boolean
  label: string
}

export interface RoadmapGroup {
  title: string
  items: RoadmapItem[]
}

export interface Protocol {
  rfc: string
  name: string
  note: string
  status: string
  delivered: boolean
}

export interface Finding {
  label: string
}

export const RELEASE = {
  commit: 'd22ef61',
  branch: 'main',
  date: '2026-09-25',
  ci: '4/4',
}

export const HERO = {
  title: 'Terra — Protocol Progress',
  subtitle:
    'Infrastructure for a verifiable digital representation of the physical world: an open physical-world trust and spatial protocol giving real-world entities persistent spatial identity and verifiable history. Land is the first domain — ISO 19152 (LADM) data model, on-chain rights with validator attestations, identity and guardianship, a ten-RFC suite — built, tested, and promoted to a stable main release.',
}

export const METRICS: Metric[] = [
  {
    value: '154',
    label: 'registry instructions',
    detail: '63 accounts · 141 events · 225 error codes — terra_registry',
  },
  {
    value: '8',
    label: 'identity instructions',
    detail: '2 accounts · 8 events · 29 error codes — terra_identity',
  },
  {
    value: '283/283',
    label: 'on-chain integration tests',
    detail: 'registry 100% green · identity 23/23 · incl. atomicity & composite paths',
  },
  {
    value: '217',
    label: 'unit · structure · API tests',
    detail: '124 program lib · 21 RFC-012 structure · 4 geo-engine · 68 API',
  },
  {
    value: '10',
    label: 'protocol RFCs specified',
    detail: 'RFC-003 vault → RFC-012 global trust architecture, all delivered',
  },
  {
    value: '4/4',
    label: 'CI checks green',
    detail: 'on-chain tests · frontend tsc · API build · API tests',
  },
]

export const ROADMAP: RoadmapGroup[] = [
  {
    title: 'Product phases',
    items: [
      { done: true, label: 'Phase 1 — parcel registry, transfers, rights, Cesium globe' },
      { done: true, label: 'Phase 2 — PostGIS fusion, OSM ingestion, geo-engine' },
      { done: true, label: 'Phase 3 — road-access validation + on-chain digest anchor' },
      { done: true, label: 'Phase 4 — identity, succession, rotation, forfeiture' },
      { done: false, label: 'Phase 5 — legal 3D / air-rights layer' },
      { done: false, label: 'Phase 6 — country configuration layer' },
      { done: false, label: 'Phase 7/8 — regional expansion → global platform' },
    ],
  },
  {
    title: 'Protocol suite',
    items: [
      { done: true, label: 'ISO 19152 (LADM) research + country-agnostic core model' },
      {
        done: true,
        label: 'RFC-003…011 — vault, escrow, staking, cross-border, disputes, subdivision, time-bound, guardianship, ZK',
      },
      {
        done: true,
        label: 'RFC-012 Phases 0–9 — architecture contract, hardening, boundary & composite-atomicity completion, device identities',
      },
      { done: true, label: 'Tier A1–A3 — security residuals, IDL resync, test/CI baseline' },
      { done: false, label: 'RFC-012 Phase 10 — cross-border privacy (ZK identity, selective disclosure)' },
    ],
  },
  {
    title: 'Launch',
    items: [
      { done: true, label: 'Both on-chain programs built & 100% integration-tested' },
      { done: true, label: 'Stable release promoted to main (d22ef61, CI green)' },
      { done: false, label: 'Devnet deployment' },
      { done: false, label: 'Mainnet gates — RFC-005 staking reconfirm + ZK audit' },
      { done: false, label: 'Mainnet' },
    ],
  },
]

export const PROTOCOLS: Protocol[] = [
  {
    rfc: 'RFC-003',
    name: 'Vault shard protocol',
    note: 'Shamir-shared encrypted recovery vaults — create/rotate/endorse/execute.',
    status: 'Delivered',
    delivered: true,
  },
  {
    rfc: 'RFC-004',
    name: 'Escrow settlement',
    note: 'Native-SOL vault with seller/buyer guards, dispute & expiry paths.',
    status: 'Delivered',
    delivered: true,
  },
  {
    rfc: 'RFC-005',
    name: 'Validator staking & slashing',
    note: 'Graduated 10%/100% slash, 7-day unbonding + appeal. Governance reconfirm pre-mainnet.',
    status: 'Delivered',
    delivered: true,
  },
  {
    rfc: 'RFC-006',
    name: 'Cross-border identity bridge',
    note: 'Jurisdictions, ZK bindings, nullifiers, revoke/rebind.',
    status: 'Delivered',
    delivered: true,
  },
  {
    rfc: 'RFC-007',
    name: 'Dispute resolution & parcel freeze',
    note: 'File/freeze/adjudicate/execute/cancel lifecycle.',
    status: 'Delivered',
    delivered: true,
  },
  {
    rfc: 'RFC-008',
    name: 'Parcel subdivision & amalgamation',
    note: 'Lineage records with rights migration.',
    status: 'Delivered',
    delivered: true,
  },
  {
    rfc: 'RFC-009',
    name: 'Time-bound credentials',
    note: 'Expiry, grace, renewal, sweep, conditional grant.',
    status: 'Delivered',
    delivered: true,
  },
  {
    rfc: 'RFC-010',
    name: 'Guardian & Recovery Council',
    note: 'Policy layer on Succession: ≥3 validators, ≥90-day grace, court case_hash.',
    status: 'Delivered',
    delivered: true,
  },
  {
    rfc: 'RFC-011',
    name: 'Zero-knowledge ownership proofs',
    note: 'Zone Merkle roots, nullifier first-use. Circuit audit pre-mainnet.',
    status: 'Delivered',
    delivered: true,
  },
  {
    rfc: 'RFC-012',
    name: 'Global physical-digital trust architecture',
    note: 'Phases 0–9 of 10 complete — Phase 10 (cross-border privacy) is next.',
    status: 'Phases 0–9',
    delivered: true,
  },
]

export const SECURITY_CLOSED: Finding[] = [
  { label: 'Critical / High / Medium findings closed (Tier A1 security residuals)' },
  { label: 'Session-audit guards — M-2 · L-1 · C-4 closed' },
  { label: 'Unique validator sets + endorsement binding (P0-1, P0-2 removal path)' },
  { label: 'IDL regenerated to source — 154 / 63 / 141 / 225 matches code' },
  { label: 'Composite atomicity — claim-succession-with-parcels, atomic subdivide & credit, reclaim-succeeded-swap' },
]

export const SECURITY_OPEN: Finding[] = [
  { label: 'RFC-005 staking governance reconfirm' },
  { label: 'ZK circuit audit (RFC-011)' },
  { label: 'L-3 cosmetic hardening (non-blocking)' },
]

export const NEXT_MILESTONE = {
  title: 'Devnet deployment rehearsal',
  body: 'Phase 9 device identities shipped on dev (registry at 154 instructions): deploy both programs to a devnet validator, wire the wallet adapter through the task flow, then RFC-012 Phase 10 (cross-border privacy, ZK identity/ownership, selective disclosure). Mainnet gates (staking reconfirm, ZK audit) follow.',
  tags: ['Program deploy', 'Wallet wiring', 'Phase 10 cross-border', 'Mainnet gates'],
}

export const VISION = {
  pitch:
    'Terra is an open physical-world trust and spatial infrastructure protocol that gives real-world entities persistent spatial identity and verifiable history by connecting observations, evidence, independent validation, rights and transactions across time and jurisdictions. Land is the first domain: anyone can create a claim about any parcel, validators independently evaluate evidence, and the network records attestations and immutable history — no authority provider is required to participate.',
  chain: [
    'Observation',
    'Evidence',
    'Validation',
    'Consensus',
    'State',
  ],
  calloutTitle: 'Claims ≠ Facts',
  calloutBody:
    'The canonical transition is observation → evidence → validation → consensus → state: no single AI model, sensor, person, company, government, oracle or database is treated as absolute truth. The protocol verifies claims, not legal ownership; physical-world observation is a first-class layer — validators can observe, photograph, and attest to the physical state of land, binding digital records to physical reality.',
}

export interface Layer {
  title: string
  body: string
  tags: string[]
}

export const ARCHITECTURE: Layer[] = [
  {
    title: 'On-chain — Solana / Anchor',
    body: 'Two programs hold minimal state: parcel rights, attestations, quorum, escrow, staking, disputes, ZK. Off-chain validation is anchored as hashes.',
    tags: [
      'terra_registry · 154 instructions',
      'terra_identity · 8 instructions',
      'GaEDbktvpZ3qiqp4PmFgHwDSa6JsFfVjXFqNb2nTbage',
      '68urV9nGcRcoWT1QjzZfXuCnTS9921x2se1SybKJr1U4',
    ],
  },
  {
    title: 'Off-chain mirror — PostGIS + Axum',
    body: 'Every on-chain account has a mirror table; the REST API serves wallets, verification flows, evidence, and spatial queries against a live PostGIS.',
    tags: ['26 migrations (0001…0026)', 'REST /api/*', '68 API tests incl. live-PostGIS run'],
  },
  {
    title: 'Geo engine + client',
    body: 'OSM fusion into PostGIS, road-access validation anchored on-chain, parcel spatial stats — visualised and driven from a Cesium globe client.',
    tags: ['OSM ingestion', 'road-access digest anchor', 'Cesium globe + wallet adapter'],
  },
]

export interface Scenario {
  title: string
  steps: string[]
  ref: string
}

export const SCENARIOS: Scenario[] = [
  {
    title: 'Claim → attestation → consensus',
    steps: [
      'Anyone files a claim on a parcel',
      'Attaches typed evidence (13 evidence types)',
      'Validators evaluate independently (14 claim types)',
      'Weighted quorum records attestations',
      'Append-only audit trail accumulates',
    ],
    ref: 'verification/* · quorum.rs',
  },
  {
    title: 'Succession when an owner vanishes',
    steps: [
      'Heir files a succession request on terra_identity',
      'Validators endorse — no authority provider needed',
      'Claim succeeds; ownership Rights PDAs re-point atomically',
      'RFC-010 court guardianship: ≥3 validators, ≥90-day grace',
    ],
    ref: 'terra_identity · RFC-010',
  },
  {
    title: 'Dispute & parcel freeze',
    steps: [
      'Owner or buyer files a dispute on the parcel',
      'Parcel freezes — transfers blocked while contested',
      'Registry adjudicator resolves with evidence',
      'Execute the winner path, or cancel to unfreeze',
    ],
    ref: 'dispute.rs · RFC-007',
  },
  {
    title: 'Cross-border identity bridge',
    steps: [
      'Register jurisdiction + identity binding',
      'ZK binding proves the identity link without revealing it',
      'Nullifiers prevent double-use across borders',
      'Revoke / rebind when policy changes',
    ],
    ref: 'cross_border.rs · RFC-006',
  },
  {
    title: 'Subdivision & amalgamation',
    steps: [
      'Split a parcel into children — or merge siblings',
      'Lineage record preserves provenance',
      'Rights migrate to the new parcel IDs',
      'Subdivide + credit settle atomically in one transaction',
    ],
    ref: 'subdivision.rs · RFC-008',
  },
]

export const CTA = {
  title: 'See it live',
  body: 'The Cesium globe client is the working product — connect, draw a parcel polygon, and register it against the on-chain registry.',
  bullets: [
    'Connect a wallet (Phantom / Solflare)',
    'Draw geometry and register a parcel',
    'Browse the on-chain registry',
    'Transfer rights from the parcel panel',
  ],
}
