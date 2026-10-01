import type { DemoScenarioEvent, DemoScenarioResult } from './api'

const SCENARIOS: Record<string, { result: string; steps: [string, string][]; entities: Record<string, unknown> }> = {
  'land-registration': {
    result: 'PARCEL_REGISTERED',
    steps: [
      ['PARCEL_REGISTERED', 'Parcel boundary and ownership right recorded.'],
      ['SPATIAL_ASSET_INITIALIZED', 'Spatial asset initialized for the parcel.'],
      ['GEOMETRY_VERSION_APPENDED', 'Canonical geometry version 0 appended.'],
    ],
    entities: { parcel: 'parcel', owner: 'owner', spatial_asset: 'spatial_asset', geometry_version: 'geometry_v0' },
  },
  verification: {
    result: 'CLAIM_VERIFIED',
    steps: [
      ['CLAIM_SUBMITTED', 'Ownership claim submitted for independent review.'],
      ['EVIDENCE_COMMITTED', 'Evidence digest committed to the review.'],
      ['OBSERVATION_SUBMITTED', 'Validator observation added to the task.'],
      ['TASK_ROUTED', 'Eligible validators evaluated and task routed.'],
      ['QUORUM_REACHED', 'Independent validator quorum reached.'],
      ['CLAIM_VERIFIED', 'Claim verified; this preview writes no chain state.'],
    ],
    entities: { claim: 'claim', evidence: 'evidence', task: 'verification_task', validators: ['validator_01', 'validator_02', 'validator_03'] },
  },
  'cross-border': {
    result: 'CROSS_BORDER_VERIFIED',
    steps: [
      ['JURISDICTIONS_REGISTERED', 'Source and receiving jurisdictions loaded.'],
      ['IDENTITY_BOUND', 'Demo identity commitment bound to a wallet.'],
      ['MEMBERSHIP_VERIFIED', 'Receiving authority membership checked.'],
      ['CROSS_BORDER_VERIFIED', 'Cross-border verification recorded in the demo.'],
    ],
    entities: { identity: 'identity_commitment', source_jurisdiction: 'CM', destination_jurisdiction: 'GH' },
  },
  dispute: {
    result: 'DISPUTE_EXECUTED',
    steps: [
      ['DISPUTE_FILED', 'A parcel ownership dispute was filed.'],
      ['PARCEL_FROZEN', 'Transfer actions paused while evidence is reviewed.'],
      ['ADJUDICATION_OPENED', 'Independent adjudicators opened the case.'],
      ['JUDGMENT_RECORDED', 'A quorum reached a deterministic demo judgment.'],
      ['JUDGMENT_EXECUTED', 'The demo applied the judgment to the parcel.'],
    ],
    entities: { dispute: 'dispute_case', parcel: 'parcel', panel: ['adjudicator_01', 'adjudicator_02', 'adjudicator_03'] },
  },
  'validator-routing': {
    result: 'TASK_ROUTED',
    steps: [
      ['TASK_CREATED', 'Task requirements and location loaded.'],
      ['ELIGIBILITY_CHECKED', 'Availability, tier, reputation, capability and jurisdiction gates checked.'],
      ['CANDIDATES_FILTERED', 'Ineligible validators excluded from the candidate set.'],
      ['WINNER_SELECTED', 'A deterministic seeded draw selected the task winner.'],
    ],
    entities: { task: 'verification_task', candidates: ['validator_01', 'validator_02', 'validator_03'], routing: 'seeded deterministic preview' },
  },
  'spatial-update': {
    result: 'GEOMETRY_VERSION_VERIFIED',
    steps: [
      ['GEOMETRY_V1_SUBMITTED', 'A new geometry observation was submitted.'],
      ['OBSERVATION_RECORDED', 'The geometry observation was attached to the parcel.'],
      ['VERIFICATION_TASK_ROUTED', 'The candidate version was sent for review.'],
      ['GEOMETRY_VERSION_VERIFIED', 'Quorum verified the candidate geometry version.'],
    ],
    entities: { parcel: 'parcel', previous_version: 'geometry_v1', candidate_version: 'geometry_v2' },
  },
  'fraud-review': {
    result: 'FRAUD_REVIEW_FINALIZED',
    steps: [
      ['FRAUD_REPORTED', 'A fraud report was submitted for committee review.'],
      ['COMMITTEE_ASSEMBLED', 'Eligible committee members were selected.'],
      ['COMMITTEE_VOTE_RECORDED', 'Committee votes were recorded against the report.'],
      ['FRAUD_REVIEW_FINALIZED', 'The review reached its configured threshold.'],
    ],
    entities: { report: 'fraud_report', committee: ['member_01', 'member_02', 'member_03'], outcome: 'threshold met' },
  },
  recovery: {
    result: 'VALIDATOR_RECOVERED',
    steps: [
      ['HEARTBEATS_OBSERVED', 'Validator heartbeat window evaluated.'],
      ['VALIDATOR_MARKED_OFFLINE', 'Missed heartbeat threshold reached.'],
      ['RECOVERY_QUORUM_REACHED', 'Recovery quorum approved the recovery action.'],
      ['VALIDATOR_RECOVERED', 'Validator restored in the demo state.'],
    ],
    entities: { validator: 'validator_01', recovery_committee: ['member_01', 'member_02', 'member_03'] },
  },
}

function hashText(text: string, salt: number) {
  let value = (0x811c9dc5 ^ salt) >>> 0
  for (let i = 0; i < text.length; i += 1) {
    value ^= text.charCodeAt(i)
    value = Math.imul(value, 0x01000193) >>> 0
  }
  return value.toString(16).padStart(8, '0')
}

function stableId(name: string, seed: string, label: string) {
  return `demo:${hashText(`${name}|${seed}|${label}`, 0)}${hashText(`${label}|${seed}|${name}`, 0x9e3779b9)}`
}

export function buildLocalDemoScenario(name: string, seed = 'default'): DemoScenarioResult {
  const blueprint = SCENARIOS[name]
  if (!blueprint) throw new Error(`No offline preview is available for "${name}".`)

  let elapsed = 0
  const events: DemoScenarioEvent[] = blueprint.steps.map(([kind, detail], seq) => {
    elapsed += 360 + (parseInt(hashText(`${seed}:${seq}`, 0).slice(0, 4), 16) % 680)
    return {
      seq,
      t_offset_ms: elapsed,
      kind,
      detail,
      ref: stableId(name, seed, `event-${seq}`),
    }
  })
  const entities = Object.fromEntries(
    Object.entries(blueprint.entities).map(([key, value]) => [
      key,
      Array.isArray(value)
        ? value.map((_, index) => stableId(name, seed, `${key}-${index}`))
        : stableId(name, seed, key),
    ]),
  )

  return {
    demo: true,
    deterministic: true,
    scenario: name,
    seed,
    result: blueprint.result,
    events,
    note: 'Local deterministic preview — no API or chain state was used or written.',
    source: 'local',
    ...entities,
  }
}
