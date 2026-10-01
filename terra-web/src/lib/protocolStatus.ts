// Protocol status — B10 honest progress dashboard.
//
// Status reflects end-to-end capability verified against the `dev` source tree
// (handler + IDL + integration tests where applicable) — a Rust fn existing is
// NOT enough for "implemented". Every `evidence` string points at real source
// locations; every `gaps` string names a concrete, verified deficiency.
//
// Status levels:
//   implemented   — handler(s) + tests (or shipped frontend) prove the feature
//   partial       — core works, but a named gap remains
//   experimental  — real code exists, but not production-proven
//   planned       — not started (designed in the advancement directive)

export type StatusLevel = 'implemented' | 'partial' | 'experimental' | 'planned'
export type StatusTrack = 'protocol' | 'infra' | 'frontend'

export interface StatusModule {
  id: string
  name: string
  track: StatusTrack
  status: StatusLevel
  evidence: string
  verified: string[]
  gaps: string[]
}

export const STATUS_META: Record<StatusLevel, { label: string; badge: string }> = {
  implemented: { label: 'IMPLEMENTED', badge: 'lab-badge-ok' },
  partial: { label: 'PARTIAL', badge: 'lab-badge-warn' },
  experimental: { label: 'EXPERIMENTAL', badge: 'lab-badge-info' },
  planned: { label: 'PLANNED', badge: 'lab-badge-mut' },
}

export const TRACK_META: Record<StatusTrack, { label: string; hint: string }> = {
  protocol: { label: 'Protocol', hint: 'terra_registry + terra_identity (Anchor)' },
  infra: { label: 'Infrastructure', hint: 'API, demo engine, devnet loop' },
  frontend: { label: 'Frontend', hint: 'terra-web observability layer' },
}

export const STATUS_LEVELS: StatusLevel[] = ['implemented', 'partial', 'experimental', 'planned']
export const STATUS_TRACKS: StatusTrack[] = ['protocol', 'infra', 'frontend']

export const STATUS_MODULES: StatusModule[] = [
  // ---------------------------------------------------------------- protocol
  {
    id: 'parcel-registry',
    name: 'Parcel registry & ownership',
    track: 'protocol',
    status: 'implemented',
    evidence: 'lib.rs (register/transfer/update_status) · tests/integration.rs',
    verified: [
      'register_parcel, transfer_parcel, update_status handlers in the 162-ix IDL',
      'Canonical OWNERSHIP Rights PDA — Parcel has no owner field',
      'transfer_rejects_non_owner + wallet/Identity holder authorization tests',
    ],
    gaps: [],
  },
  {
    id: 'rights',
    name: 'Rights lifecycle',
    track: 'protocol',
    status: 'implemented',
    evidence: 'lib.rs grant/revoke/renew/sweep/conditional · integration.rs',
    verified: [
      'grant_right (×29 test refs), revoke_right, renew_right',
      'sweep_expired_rights + time_bound.rs expiry logic',
      'grant_conditional_right; wrong-kind / revoked / expired rejection tests',
    ],
    gaps: [],
  },
  {
    id: 'identity',
    name: 'Identity program',
    track: 'protocol',
    status: 'implemented',
    evidence: 'terra_identity/src/instructions/*.rs · identity tests',
    verified: [
      'bind_identity, guardianship (3), succession (4) — 8 instructions',
      'bind_identity ×75 refs in identity integration tests',
      'Identity PDA usable as a Rights holder in registry authorization',
    ],
    gaps: [],
  },
  {
    id: 'world-registry',
    name: 'World / jurisdiction registry',
    track: 'protocol',
    status: 'partial',
    evidence: 'world_registry.rs (5 handlers) · integration.rs ×9/12',
    verified: [
      'create_world_registry, allocate_country, request_genesis, confirm_genesis',
      'Country/jurisdiction allocation tests (allocate_country ×12)',
    ],
    gaps: ['Bootstrap authority is still a single signer — rotation/decentralization not implemented'],
  },
  {
    id: 'validator-registry',
    name: 'Validator registry & governance',
    track: 'protocol',
    status: 'implemented',
    evidence: 'validator_registry.rs (19 handlers) · integration.rs',
    verified: [
      'add/endorse/nominate/confirm/propose/removal + pause/unpause_program',
      'Quorum + effective-mode logic in validator_registry.rs',
      'bootstrap_self_proclaim + second/third validator bootstraps tested',
    ],
    gaps: [],
  },
  {
    id: 'validator-profile',
    name: 'Validator profile & capability',
    track: 'protocol',
    status: 'partial',
    evidence: 'validator_profile.rs (18 handlers) · integration.rs ×10',
    verified: [
      'init/update profile, jurisdiction, tier, presence, availability, suspend',
      'declare_capability + admin_verify_capability, relationship edges',
    ],
    gaps: ['Capability verification is admin-signed — evidence/task-based verification path not built'],
  },
  {
    id: 'routing',
    name: 'Validator routing',
    track: 'protocol',
    status: 'implemented',
    evidence: 'routing.rs (15 fns) · integration.rs ×15',
    verified: [
      'distance / availability / tier / reputation / capability / geo / jurisdiction gates',
      'route_seed + random gate (draw_bps), select_winner, filter_eligible',
      'route_task end-to-end integration tests (×15 refs)',
    ],
    gaps: [],
  },
  {
    id: 'verification-tasks',
    name: 'Verification tasks',
    track: 'protocol',
    status: 'implemented',
    evidence: 'verification_task.rs (18 handlers) · integration.rs',
    verified: [
      'create/add_requirement/assign/claim/submit_result/cancel',
      'route_task wiring + validator assignment tested',
    ],
    gaps: [],
  },
  {
    id: 'task-economics',
    name: 'Task economics & escrow rewards',
    track: 'protocol',
    status: 'implemented',
    evidence: 'task_economics.rs (14 handlers) · fund_task_escrow ×13',
    verified: [
      'set_fee_policy, set_coverage_incentive, quote_task_resources',
      'fund/claim/refund task escrow paths integration-tested',
    ],
    gaps: [],
  },
  {
    id: 'claims-observations',
    name: 'Claims, observations & attestations',
    track: 'protocol',
    status: 'implemented',
    evidence: 'verification/*.rs · observation_v2.rs (6) · session ×16',
    verified: [
      'create_claim, add_evidence, submit_verification_attestation, verify_claim',
      'V2: submit_observation_v2, submit_evidence_manifest, add_evidence_artifact',
      'Session lifecycle: open/close ×16, record evidence/observation/attestation',
    ],
    gaps: [],
  },
  {
    id: 'evidence-manifest',
    name: 'Evidence manifests (RFC-013)',
    track: 'protocol',
    status: 'partial',
    evidence: 'evidence_manifest.rs (8 handlers) · integration.rs ×3',
    verified: [
      'Artifact kind/storage ref/content hash/index/capacity validation',
      'Claim/task status guards + 6239/6240 error paths',
    ],
    gaps: ['Real-world evidence acquisition & privacy layer (off-chain ingestion) not built'],
  },
  {
    id: 'quorum',
    name: 'Geometry & claim quorum',
    track: 'protocol',
    status: 'implemented',
    evidence: 'quorum.rs (4) · verification/quorum_config/quorum_voting.rs',
    verified: [
      'Quorum config + voting; verifier-set/effective-mode handling',
      'RFC-013 geometry quorum: 6240 GeometryQuorumFull attestor cap tested',
    ],
    gaps: [],
  },
  {
    id: 'disputes',
    name: 'Disputes & judgment',
    track: 'protocol',
    status: 'implemented',
    evidence: 'dispute.rs (5 handlers) · integration.rs ×16/3',
    verified: [
      'file_dispute (×16), freeze_parcel, adjudicate_dispute',
      'execute_judgment (×3), cancel_dispute — full lifecycle tested',
    ],
    gaps: [],
  },
  {
    id: 'escrow',
    name: 'Escrow',
    track: 'protocol',
    status: 'implemented',
    evidence: 'escrow.rs (8 handlers) · integration.rs ×26',
    verified: [
      'create/deposit/accept/settle, cancel/mutual_cancel',
      'dispute_escrow + expire_escrow paths (×26 refs total)',
    ],
    gaps: [],
  },
  {
    id: 'cross-border',
    name: 'Cross-border identity & verification',
    track: 'protocol',
    status: 'implemented',
    evidence: 'cross_border.rs (13 handlers) · integration.rs',
    verified: [
      'register/update_jurisdiction (×36), bind/verify/revoke membership',
      'create_cross_border_binding + session linkage (link_cross_border_to_session)',
    ],
    gaps: [],
  },
  {
    id: 'fraud-governance',
    name: 'Fraud governance',
    track: 'protocol',
    status: 'implemented',
    evidence: 'fraud_governance.rs (29 handlers) · integration.rs ×10',
    verified: [
      'report → open_review → cast_vote → finalize; appeal + rehabilitation flow',
      'Committee selection: filter_committee_pool, committee_seed, select_committee, draw_bps',
    ],
    gaps: [],
  },
  {
    id: 'staking',
    name: 'Staking & slashing',
    track: 'protocol',
    status: 'partial',
    evidence: 'staking.rs (12 handlers) · deposit ×36 / unbonding ×7',
    verified: [
      'create_stake_pool, deposit, initiate_unbonding, withdraw',
      'report_equivocation/offense → verify_and_slash (×1), rewards claim/distribute',
      'slashing dispute + dismiss_report',
    ],
    gaps: ['Economic coupling with reputation/routing/validator selection not finalized'],
  },
  {
    id: 'device-identity',
    name: 'Device identity',
    track: 'protocol',
    status: 'partial',
    evidence: 'device_identity.rs (9 handlers) · integration.rs ×9',
    verified: [
      'register/update/rotate_key/status/calibration/verify_device',
    ],
    gaps: ['Registered ≠ hardware-attested — off-chain platform attestation layer not implemented'],
  },
  {
    id: 'vault',
    name: 'Evidence vault & shard rotation',
    track: 'protocol',
    status: 'partial',
    evidence: 'vault.rs (8 handlers) · vault.rs:26',
    verified: [
      'create_vault, authorize access, multi-step shard rotation (initiate/endorse/execute/cancel), ping_shard',
    ],
    gaps: ['Automatic keeper-driven rotation explicitly not wired (vault.rs:26)'],
  },
  {
    id: 'recovery',
    name: 'Recovery & quorum health',
    track: 'protocol',
    status: 'implemented',
    evidence: 'recovery.rs (5 handlers) · heartbeat ×8',
    verified: [
      'heartbeat, set_validator_active, check_quorum_reachable',
      'queue/execute_emergency_injection',
    ],
    gaps: [],
  },
  {
    id: 'spatial-versions',
    name: 'Spatial registry & geometry versions',
    track: 'protocol',
    status: 'implemented',
    evidence: 'spatial_asset.rs (16) · integration.rs ×2 · RFC-013',
    verified: [
      'init_spatial_asset, append_geometry_version, verify_geometry_version',
      'Dimension / source / elevation envelope / version cap guards (6237–6240)',
    ],
    gaps: [],
  },
  {
    id: 'spatial-pipeline',
    name: 'Spatial intelligence pipeline',
    track: 'protocol',
    status: 'planned',
    evidence: 'designed: observation → evidence → candidate → AI/GIS → 3D → quorum',
    verified: [],
    gaps: [
      'Candidate-geometry generation from observations',
      'AI/GIS fusion step and validator-verified canonical transition beyond version append',
    ],
  },
  {
    id: 'subdivision',
    name: 'Subdivision & amalgamation',
    track: 'protocol',
    status: 'implemented',
    evidence: 'subdivision.rs (3 handlers) · integration.rs',
    verified: ['Subdivision/amalgamation instructions gated by ownership-rights authorization'],
    gaps: [],
  },
  {
    id: 'zk',
    name: 'ZK ownership proofs',
    track: 'protocol',
    status: 'experimental',
    evidence: 'zk/groth16.rs · api/routes/zk_proofs.rs · integration.rs ×27',
    verified: [
      'Real Groth16/BN254 pairing verification via alt_bn128 syscalls (verify_groth16)',
      'Integration tests ×27; API nullifier store + stale-root rejection',
    ],
    gaps: [
      'Production circuit + trusted setup + external audit = release gate',
      'VK lifecycle / selective disclosure policy not finalized',
    ],
  },

  // ------------------------------------------------------------------ infra
  {
    id: 'api-rest',
    name: 'REST API (Axum + PostGIS)',
    track: 'infra',
    status: 'partial',
    evidence: 'terra-core/api/src/routes/mod.rs (21 nests)',
    verified: [
      '21 route groups: parcels, rights, identities, disputes, escrows, evidence, spatial, geo, fusion, zk, staking, recovery, world-registry, vaults, tx_prep…',
    ],
    gaps: ['No /validators, /tasks, /claims, /fraud listing endpoints yet — frontend must use demo data for those'],
  },
  {
    id: 'event-stream',
    name: 'Event stream / indexer',
    track: 'infra',
    status: 'planned',
    evidence: 'no websocket/SSE/indexer in api/src',
    verified: [],
    gaps: ['On-chain event indexing + live push channel for the activity feed (B4)'],
  },
  {
    id: 'demo-engine',
    name: 'Demo scenario engine',
    track: 'infra',
    status: 'implemented',
    evidence:
      'api/src/routes/demo_scenarios.rs (GET/POST /api/v1/demo/scenarios/*, 7 unit tests — cargo test -p terra-api: 75 passed) + shipped /demo player; live chain-event streaming tracked under activity-feed (B4)',
    verified: [
      '8 scenario builders (verification, land-registration, cross-border, dispute, validator-routing, spatial-update, fraud-review, recovery)',
      'Deterministic seeded output (FNV-1a + xorshift64*, byte-identical per seed — unit-tested + in-page "Verify determinism")',
      'Every payload carries demo:true + demo-prefixed accounts (never presented as chain state)',
      'Shipped /demo player: scenario picker, seeded runs, animated event timeline (play/pause/step), entity inspector',
    ],
    gaps: [],
  },
  {
    id: 'devnet-e2e',
    name: 'Full devnet end-to-end loop',
    track: 'infra',
    status: 'experimental',
    evidence: 'wallet + tx prep exist; faucet funding blocked (P0-DEPLOY)',
    verified: [
      'Frontend wallet signing path, tx prep route, devnet status checks in source',
    ],
    gaps: ['Live run blocked — faucet dead/rate-limited; needs funded authority to prove the loop'],
  },

  // --------------------------------------------------------------- frontend
  {
    id: 'globe',
    name: 'Globe (3D + 2D fallback)',
    track: 'frontend',
    status: 'implemented',
    evidence: 'pages/GlobePage.tsx · components/map/*',
    verified: [
      '3D globe with Leaflet preferCanvas 2D fallback (zero-GPU path)',
      'On-chain + PostGIS parcels, roads, POIs, stats, demo-vs-live labelling',
      'Parcel registration + browsing from the globe',
    ],
    gaps: [],
  },
  {
    id: 'wallet-parcels',
    name: 'Wallet & real parcel I/O',
    track: 'frontend',
    status: 'partial',
    evidence: 'lib/wallet.tsx · store/appStore.ts · lib/program.ts',
    verified: [
      'Wallet connect/disconnect, on-chain parcel reads, ownership Rights PDA holder derivation',
      'Real parcel transaction path in source (register/transfer)',
    ],
    gaps: ['Live devnet signing run blocked by faucet funding (P0-DEPLOY)'],
  },
  {
    id: 'transactions-ui',
    name: 'Transaction observability',
    track: 'frontend',
    status: 'partial',
    evidence: 'pages/TransactionsPage.tsx · lib/txStore.ts',
    verified: [
      'Instruction/status/signature/slot/logs/compute records, version timeline deep-link',
    ],
    gaps: ['DEMO/DEVNET mode switch + explorer deep links (B8/B9) not built', 'Demo signatures are random, not deterministic (B6)'],
  },
  {
    id: 'lab-geometry',
    name: 'Lab — Geometry Vault & land view',
    track: 'frontend',
    status: 'implemented',
    evidence: 'components/lab/{GeometryVault,IsoTerrain,LandViewer,…}.tsx · /lab/land',
    verified: [
      'RFC-013 A+B vault: append/attest, elevation provenance, evidence chain, quorum ring',
      'Interactive WebGL-free LandViewer: 2D/3D, pan/zoom/rotate',
      'CadaSPACE-style rights volumes (version strata) + land detail page with full props',
    ],
    gaps: [],
  },
  {
    id: 'lab-verification',
    name: 'Lab — Verification pipeline',
    track: 'frontend',
    status: 'implemented',
    evidence: 'components/lab/VerificationPipeline.tsx',
    verified: ['Claim → evidence → observation → task → quorum → verified fact walkthrough'],
    gaps: [],
  },
  {
    id: 'lab-crossborder',
    name: 'Lab — Cross-border gate',
    track: 'frontend',
    status: 'implemented',
    evidence: 'components/lab/CrossBorderGate.tsx',
    verified: ['RFC-012 Phase 10 jurisdiction binding walkthrough'],
    gaps: [],
  },
  {
    id: 'parcel-panel',
    name: 'Parcel intelligence panel (B2)',
    track: 'frontend',
    status: 'partial',
    evidence: 'components/panels/* (register/list/detail exist)',
    verified: ['Parcel registration, list, and basic detail panels'],
    gaps: ['Combined ownership + geometry + verification + history + network intelligence view'],
  },
  {
    id: 'network-page',
    name: 'Validator network page (B3)',
    track: 'frontend',
    status: 'partial',
    evidence: 'pages/NetworkPage.tsx · lib/routingSim.ts (mirror of routing.rs)',
    verified: [
      '/network: eligibility matrix running all 6 route_task gates + random gate + winner pick, "why selected" details',
      'Deterministic demo network (9 validators, 4 tasks) with equirectangular presence plot and profile cards',
    ],
    gaps: ['Demo dataset only — live validator data awaits the /validators API (planned)'],
  },
  {
    id: 'activity-feed',
    name: 'Live network activity (B4)',
    track: 'frontend',
    status: 'planned',
    evidence: 'no feed component',
    verified: [],
    gaps: ['Observation/task/routing/attestation/quorum event feed (needs event stream or demo engine)'],
  },
  {
    id: 'demo-scenarios-ui',
    name: 'Demo scenario runner & protocol playground (B5–B7)',
    track: 'frontend',
    status: 'planned',
    evidence: 'Lab has 3 tabs (vault, pipeline, crossborder)',
    verified: [],
    gaps: ['Deterministic scenario runner + playground tabs: routing, escrow, disputes, fraud, staking, devices, recovery, ZK'],
  },
  {
    id: 'protocol-status',
    name: 'Protocol status dashboard (B10)',
    track: 'frontend',
    status: 'implemented',
    evidence: 'pages/StatusPage.tsx (this page)',
    verified: ['Honest IMPLEMENTED/PARTIAL/EXPERIMENTAL/PLANNED view with source evidence per module'],
    gaps: [],
  },
]

export function statusCounts(modules: StatusModule[] = STATUS_MODULES): Record<StatusLevel, number> {
  const counts: Record<StatusLevel, number> = { implemented: 0, partial: 0, experimental: 0, planned: 0 }
  for (const m of modules) counts[m.status] += 1
  return counts
}
