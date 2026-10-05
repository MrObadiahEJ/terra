import { Fragment } from 'react'
import type { LucideIcon } from 'lucide-react'
import { Camera, CheckCheck, Eye, FileText, Flag, Fingerprint, MapPin, Share2, Users } from 'lucide-react'
import type { DemoScenarioEvent } from '../../lib/api'
import { STAGE_DEFS, type StageId, type StageTrack } from '../../lib/protocolStages'

const STAGE_ICON: Record<StageId, LucideIcon> = {
  observation: Camera,
  evidence: FileText,
  hash: Fingerprint,
  routing: Share2,
  review: Eye,
  attestation: CheckCheck,
  quorum: Users,
  transition: Flag,
  geometry: MapPin,
}

interface ProtocolVizProps {
  track: StageTrack
  events: DemoScenarioEvent[]
  phase: number
  validators: { labels: string[]; present: boolean }
  spatial: { label: string; stage: StageId | null }
}

type ValidatorState = 'idle' | 'queued' | 'checking' | 'attested'

interface NarrationBarProps {
  track: StageTrack
  events: DemoScenarioEvent[]
  phase: number
  variant?: 'panel' | 'scene'
}

export function NarrationBar({ track, events, phase, variant = 'panel' }: NarrationBarProps) {
  const activeNode = track.nodes[track.active]
  const current = phase >= 0 ? events[phase] : undefined
  const Icon = activeNode ? STAGE_ICON[activeNode.id] : null
  return (
    <div
      className={`dm-narrbar dm-narrbar--${variant} ${activeNode ? `st-${activeNode.id}` : 'st-idle'}`}
      role="status"
    >
      {Icon && (
        <span className="dm-narrbar-ico" aria-hidden="true">
          <Icon size={15} />
        </span>
      )}
      <span className="dm-narrbar-tx">
        {activeNode ? (
          <>
            <strong>{activeNode.def.headline}</strong>
            <span>{activeNode.def.human}</span>
          </>
        ) : (
          <span>Run the scenario to watch the protocol happen step by step.</span>
        )}
      </span>
      {activeNode && track.nodes.length > 0 && (
        <span className="dm-narrbar-step">
          STEP {track.active + 1}/{track.nodes.length}
        </span>
      )}
      {current && <code className="dm-narrbar-kind">{current.kind}</code>}
    </div>
  )
}

export default function ProtocolViz({ track, events, phase, validators, spatial }: ProtocolVizProps) {
  const limit = Math.min(phase, events.length - 1)
  const rankOf = (id: StageId) => track.nodes.findIndex((node) => node.id === id)

  let revealedRouting = 0
  let revealedAttestation = 0
  for (let i = 0; i <= limit; i += 1) {
    const stage = track.stages[i]
    if (stage === 'routing') revealedRouting += 1
    if (stage === 'attestation') revealedAttestation += 1
  }

  const networkRanks = (['routing', 'review', 'attestation', 'quorum'] as StageId[])
    .map(rankOf)
    .filter((r) => r >= 0)
  const hasNetwork = networkRanks.length > 0
  const networkActive = hasNetwork && track.active >= Math.min(...networkRanks)
  const quorumRank = rankOf('quorum')
  const quorumDone = quorumRank >= 0 && track.active >= quorumRank
  const labels = validators.labels.length ? validators.labels : ['V-01', 'V-02', 'V-03']
  const validatorCount = hasNetwork ? labels.length : 0

  const stateOf = (index: number): ValidatorState => {
    if (quorumDone || revealedAttestation >= index + 1) return 'attested'
    if (!networkActive) return 'idle'
    if (revealedRouting > index) return 'checking'
    return 'queued'
  }

  const attestedCount = hasNetwork ? labels.filter((_, i) => stateOf(i) === 'attested').length : 0
  const activeNode = track.nodes[track.active]
  const current = phase >= 0 ? events[phase] : undefined

  const lineClass = (index: number) => {
    const state = stateOf(index)
    if (state === 'attested') return 'ln done'
    if (networkActive) return 'ln flow'
    return 'ln'
  }

  const ys = Array.from({ length: Math.max(validatorCount, 1) }, (_, i) =>
    validatorCount <= 1 ? 85 : 30 + (i * 110) / (validatorCount - 1),
  )

  const shortLabel = (index: number) => {
    const raw = labels[index] ?? `V-${index + 1}`
    return raw.length > 10 ? `V-${String(index + 1).padStart(2, '0')}` : raw.toUpperCase()
  }

  return (
    <div className="dm-viz">
      <div className="dm-pipe">
        {track.nodes.map((node, i) => {
          const Icon = STAGE_ICON[node.id]
          const cls = i < track.active ? 'done' : i === track.active ? 'active' : ''
          return (
            <Fragment key={node.id}>
              {i > 0 && (
                <span
                  className={`dm-plink st-${track.nodes[i - 1].id} ${i <= track.active ? 'done' : ''}`}
                />
              )}
              <span className={`dm-pnode st-${node.id} ${cls}`}>
                <span className="dm-pnode-ico"><Icon size={16} /></span>
                <span className="dm-pnode-lb">{node.def.label}</span>
                <small>ev&nbsp;{node.firstSeq}</small>
              </span>
            </Fragment>
          )
        })}
        {track.nodes.length === 0 && <span className="text-xs text-muted">Press Run…</span>}
      </div>

      <div className="dm-viz-cap">
        {activeNode ? (
          <>
            <strong>{activeNode.def.headline}</strong>
            <span>{activeNode.def.human}</span>
            {current && <code className="dm-viz-code">{current.kind}</code>}
          </>
        ) : (
          <span>Run the scenario to watch the protocol happen step by step.</span>
        )}
      </div>

      <div className="dm-viz-row">
        {hasNetwork ? (
          <div className="dm-net">
            <div className="dm-net-h">
              <span>Validator network</span>
              <span className="dm-sim-badge">SIMULATION</span>
            </div>
            <svg viewBox="0 0 340 170" role="img" aria-label="Validator network for the current scenario">
              <g className={spatial.stage ? `st-${spatial.stage}` : 'st-idle'}>
                <rect x="14" y="64" width="96" height="42" rx="9" className="net-parcel" />
                <text x="62" y="82" textAnchor="middle" className="net-hd">PARCEL</text>
                <text x="62" y="97" textAnchor="middle" className="net-state">{spatial.label}</text>
              </g>
              {ys.slice(0, validatorCount).map((y, i) => {
                const state = stateOf(i)
                return (
                  <Fragment key={labels[i] ?? i}>
                    <line x1="110" y1="85" x2="252" y2={y} className={lineClass(i)} />
                    <circle cx="272" cy={y} r="15" className={`nv nv-${state}`} />
                    <text x="272" y={y + 3.5} textAnchor="middle" className="net-num">{i + 1}</text>
                    <text x="291" y={y + 3.5} className="net-lb">
                      <title>{labels[i]}</title>
                      {shortLabel(i)}
                    </text>
                    {state === 'attested' && <text x="281" y={y - 7} textAnchor="middle" className="net-ok">✓</text>}
                  </Fragment>
                )
              })}
            </svg>
          </div>
        ) : (
          <div className="dm-net dm-net-note">
            <span>Validator review</span>
            <p className="text-xs text-muted">This scenario does not route work to validators — it exercises a different part of the protocol.</p>
          </div>
        )}

        <div className="dm-sum">
          <div className="dm-sum-h">What is happening</div>
          {track.nodes.map((node) => {
            const reached = track.active >= rankOf(node.id)
            const value =
              node.id === 'attestation' && hasNetwork
                ? `${revealedAttestation}/${validatorCount}`
                : reached
                  ? '✓'
                  : 'pending'
            return (
              <div className="dm-sum-row" key={node.id}>
                <b className={`st-${node.id} dm-sum-lb`}>{node.def.label}</b>
                <span className={`dm-sum-val ${reached ? 'ok' : ''}`}>{value}</span>
              </div>
            )
          })}
          {hasNetwork && (
            <div className="dm-sum-row">
              <b>Consensus</b>
              <span className="dm-sum-val">{attestedCount}/{validatorCount}</span>
            </div>
          )}
          <div className="dm-sum-bar" aria-hidden>
            <i style={{ width: `${track.nodes.length ? ((track.active + 1) / track.nodes.length) * 100 : 0}%` }} />
          </div>
          <div className="dm-sum-row">
            <b>Spatial state</b>
            <span className={`dm-spatial-chip ${spatial.stage ? `st-${spatial.stage}` : 'st-idle'}`}>{spatial.label}</span>
          </div>
        </div>
      </div>

      <div className="dm-nar-wrap">
        <div className="dm-sum-h">Human timeline <span className="text-muted">— the same events, in plain language</span></div>
        <div className="dm-nar">
          {phase < 0 && <div className="text-xs text-muted">Press Run…</div>}
          {events.slice(0, limit + 1).map((event, i) => {
            const stage = track.stages[i]
            const def = STAGE_DEFS[stage]
            const Icon = STAGE_ICON[stage]
            return (
              <div className={`dm-nar-item st-${stage}`} key={event.seq}>
                <span className="dm-nar-ico"><Icon size={13} /></span>
                <div className="min-w-0">
                  <div className="dm-nar-lb">{def.headline}</div>
                  <div className="dm-nar-tx">{event.detail}</div>
                </div>
              </div>
            )
          })}
        </div>
      </div>
    </div>
  )
}
