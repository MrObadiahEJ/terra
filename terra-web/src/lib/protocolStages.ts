import type { DemoScenarioEvent, DemoScenarioResult } from './api'

export type StageId =
  | 'observation'
  | 'evidence'
  | 'hash'
  | 'routing'
  | 'review'
  | 'attestation'
  | 'quorum'
  | 'transition'
  | 'geometry'

export interface StageDef {
  id: StageId
  label: string
  headline: string
  human: string
}

export const STAGE_DEFS: Record<StageId, StageDef> = {
  observation: {
    id: 'observation',
    label: 'Observation',
    headline: 'Observation captured',
    human: 'A real-world event enters Terra: a claim, survey, photo or dispute.',
  },
  evidence: {
    id: 'evidence',
    label: 'Evidence',
    headline: 'Evidence assembled',
    human: 'The evidence package is gathered: photo, location, timestamp, device.',
  },
  hash: {
    id: 'hash',
    label: 'Commit',
    headline: 'Cryptographically committed',
    human: 'A SHA-256 fingerprint locks the evidence — the commitment is immutable.',
  },
  routing: {
    id: 'routing',
    label: 'Routing',
    headline: 'Verification task routed',
    human: 'The task travels through the network to eligible validators.',
  },
  review: {
    id: 'review',
    label: 'Review',
    headline: 'Independent review',
    human: 'Validators examine the evidence independently.',
  },
  attestation: {
    id: 'attestation',
    label: 'Attestation',
    headline: 'Signed attestations',
    human: 'Validators sign their findings — checks are recorded one by one.',
  },
  quorum: {
    id: 'quorum',
    label: 'Consensus',
    headline: 'Quorum reached',
    human: 'The consensus threshold is met; the verdict is final.',
  },
  transition: {
    id: 'transition',
    label: 'State',
    headline: 'State transition',
    human: 'The protocol state advances — the decision becomes the new truth.',
  },
  geometry: {
    id: 'geometry',
    label: 'Spatial',
    headline: 'Spatial state updated',
    human: 'The parcel on the map reflects the verified reality.',
  },
}

const KIND_STAGE: Record<string, StageId> = {
  PARCEL_REGISTERED: 'observation',
  CLAIM_CREATED: 'observation',
  CLAIM_SUBMITTED: 'observation',
  DISPUTE_FILED: 'observation',
  FRAUD_REPORT_SUBMITTED: 'observation',
  FRAUD_REPORTED: 'observation',
  GEOMETRY_V1_CANONICAL: 'observation',
  GEOMETRY_V1_SUBMITTED: 'observation',
  TASK_REQUIREMENTS_SET: 'observation',
  JURISDICTION_REGISTERED: 'observation',
  JURISDICTIONS_REGISTERED: 'observation',
  HEARTBEAT_OK: 'observation',
  HEARTBEATS_OBSERVED: 'observation',

  EVIDENCE_SUBMITTED: 'evidence',
  EVIDENCE_MANIFEST_SUBMITTED: 'evidence',
  OWNERSHIP_RIGHT_GRANTED: 'evidence',
  IDENTITY_BOUND: 'evidence',

  EVIDENCE_COMMITTED: 'hash',
  SPATIAL_ASSET_INITIALIZED: 'hash',

  TASK_CREATED: 'routing',
  TASK_ROUTED: 'routing',
  VALIDATOR_ASSIGNED: 'routing',
  VERIFICATION_TASK_ROUTED: 'routing',
  CANDIDATES_GATHERED: 'routing',
  CANDIDATES_FILTERED: 'routing',
  ELIGIBILITY_FILTERED: 'routing',
  ELIGIBILITY_CHECKED: 'routing',
  ASSIGNMENT_CREATED: 'routing',
  RANDOM_GATE: 'routing',
  WINNER_SELECTED: 'routing',
  CROSS_BORDER_BINDING_CREATED: 'routing',
  COMMITTEE_SELECTED: 'routing',
  COMMITTEE_ASSEMBLED: 'routing',
  EMERGENCY_INJECTION_QUEUED: 'routing',

  OBSERVATION_RECORDED: 'review',
  OBSERVATION_SUBMITTED: 'review',
  ADJUDICATION_OPENED: 'review',
  FRAUD_REVIEW_OPENED: 'review',
  VALIDATOR_OFFLINE: 'review',
  VALIDATOR_MARKED_OFFLINE: 'review',
  MEMBERSHIP_VERIFIED: 'review',
  CANDIDATE_GEOMETRY_DERIVED: 'review',

  ATTESTATION_SUBMITTED: 'attestation',
  VOTES_CAST: 'attestation',
  COMMITTEE_VOTE_RECORDED: 'attestation',
  EMERGENCY_INJECTION_EXECUTED: 'attestation',

  QUORUM_REACHED: 'quorum',
  QUORUM_CHECK: 'quorum',
  JUDGMENT_RENDERED: 'quorum',
  JUDGMENT_RECORDED: 'quorum',
  REVIEW_FINALIZED: 'quorum',
  FRAUD_REVIEW_FINALIZED: 'quorum',
  RECOVERY_QUORUM_REACHED: 'quorum',

  CLAIM_VERIFIED: 'transition',
  JUDGMENT_EXECUTED: 'transition',
  PARCEL_FROZEN: 'transition',
  CROSS_BORDER_VERIFIED: 'transition',
  BINDING_STATUS_ACTIVE: 'transition',
  RESTRICTION_APPLIED: 'transition',
  REGISTRATION_COMPLETE: 'transition',
  APPEAL_WINDOW_OPEN: 'transition',
  SET_VALIDATOR_ACTIVE: 'transition',
  VALIDATOR_RECOVERED: 'transition',

  GEOMETRY_VERSION_APPENDED: 'geometry',
  GEOMETRY_VERSION_VERIFIED: 'geometry',
  GEOMETRY_V2_APPENDED: 'geometry',
  GEOMETRY_V2_VERIFIED: 'geometry',
}

const FALLBACKS: { re: RegExp; stage: StageId }[] = [
  { re: /EVIDENCE|MANIFEST/, stage: 'evidence' },
  { re: /COMMIT|DIGEST|FINGERPRINT|SHA/, stage: 'hash' },
  { re: /ROUT|ASSIGN|CANDIDATE|ELIGIB|QUEUE|GATE/, stage: 'routing' },
  { re: /ATTEST|VOTE/, stage: 'attestation' },
  { re: /QUORUM|CONSENSUS/, stage: 'quorum' },
  { re: /OBSERVAT|FILED|REPORT|CREATED|SUBMIT/, stage: 'observation' },
  { re: /VERIF|COMPLETE|EXECUT|FINAL|RECOVER|ACTIVE|RESTRICT|FROZEN|GRANTED|REGISTERED|INITIALIZ|APPLIED/, stage: 'transition' },
  { re: /GEOMETRY|SPATIAL/, stage: 'geometry' },
]

export function classifyEvent(kind: string): StageId {
  const known = KIND_STAGE[kind]
  if (known) return known
  for (const rule of FALLBACKS) if (rule.re.test(kind)) return rule.stage
  return 'review'
}

export interface StageNode {
  id: StageId
  def: StageDef
  firstSeq: number
}

export interface StageTrack {
  nodes: StageNode[]
  stages: StageId[]
  active: number
}

export function buildStageTrack(events: DemoScenarioEvent[], phase: number): StageTrack {
  const stages = events.map((event) => classifyEvent(event.kind))
  const nodes: StageNode[] = []
  const rank = new Map<StageId, number>()
  stages.forEach((stage, index) => {
    if (!rank.has(stage)) {
      rank.set(stage, nodes.length)
      nodes.push({ id: stage, def: STAGE_DEFS[stage], firstSeq: events[index].seq })
    }
  })
  let active = -1
  const limit = Math.min(phase, events.length - 1)
  for (let i = 0; i <= limit; i += 1) {
    const r = rank.get(stages[i]) ?? -1
    if (r > active) active = r
  }
  return { nodes, stages, active }
}

const SPATIAL_LABEL: Record<StageId, string> = {
  observation: 'OBSERVED',
  evidence: 'EVIDENCE GATHERED',
  hash: 'COMMITTED',
  routing: 'ROUTED',
  review: 'UNDER VERIFICATION',
  attestation: 'UNDER VERIFICATION',
  quorum: 'UNDER VERIFICATION',
  transition: 'STATE UPDATED',
  geometry: 'GEOMETRY VERIFIED',
}

export function spatialStateFor(track: StageTrack): { label: string; stage: StageId | null } {
  const node = track.nodes[track.active]
  if (!node) return { label: 'IDLE', stage: null }
  return { label: SPATIAL_LABEL[node.id], stage: node.id }
}

export function validatorInfo(result: DemoScenarioResult): { labels: string[]; present: boolean } {
  for (const key of ['validators', 'candidates', 'committee', 'panel', 'recovery_committee']) {
    const raw = result[key]
    if (Array.isArray(raw)) {
      const labels = raw.filter((value): value is string => typeof value === 'string').slice(0, 4)
      if (labels.length) return { labels, present: true }
    }
  }
  return { labels: [], present: false }
}
