import { DEFAULT_GEOMETRY_QUOROM, MAX_VALIDATORS } from '../../lib/labStore'
import { Check, CircleUser } from 'lucide-react'

// Validator-quorum gauge for GeometryVersion attestation (RFC-013 Phase B §3).
// A progress arc + per-slot attestor nodes around a circle — plain SVG with
// CSS transitions, no WebGL.

interface Props {
  attestors: string[]
  required: number
  verified: boolean
  quorumConfig: number // 0 = no global QuorumConfig configured (fallback 2)
  validators: readonly string[]
  onAttest: (validator: string) => void
  disabled?: boolean
}

const W = 380
const H = 210
const CX = 132
const CY = 104
const R = 66
const C = 2 * Math.PI * R

export default function QuorumRing({
  attestors,
  required,
  verified,
  quorumConfig,
  validators,
  onAttest,
  disabled = false,
}: Props) {
  const count = attestors.length
  const frac = required > 0 ? Math.min(1, count / required) : 0
  const remaining = Math.max(0, required - count)
  const configured = quorumConfig > 0

  return (
    <div className="qring">
      <svg
        viewBox={`0 0 ${W} ${H}`}
        width="100%"
        height={H}
        preserveAspectRatio="xMidYMid meet"
        role="img"
        aria-label="Validator quorum ring (SVG, no WebGL)"
      >
        {/* track */}
        <circle cx={CX} cy={CY} r={R} fill="none" stroke="#e5e7eb" strokeWidth="12" />
        {/* progress arc — starts at 12 o'clock, clockwise */}
        <circle
          cx={CX}
          cy={CY}
          r={R}
          fill="none"
          stroke={verified ? '#10b981' : configured ? '#6366f1' : '#0ea473'}
          strokeWidth="12"
          strokeLinecap="round"
          strokeDasharray={`${(C * frac).toFixed(2)} ${C.toFixed(2)}`}
          transform={`rotate(-90 ${CX} ${CY})`}
          className={`qring-arc ${verified ? 'done' : ''}`}
        />
        {/* threshold tick at 12 o'clock */}
        <line
          x1={CX}
          y1={CY - R - 9}
          x2={CX}
          y2={CY - R + 9}
          stroke="#111827"
          strokeWidth="2"
        />

        {/* slot nodes around the ring (up to MAX_VALIDATORS) */}
        {Array.from({ length: MAX_VALIDATORS }).map((_, i) => {
          const a = -Math.PI / 2 + (i / MAX_VALIDATORS) * Math.PI * 2
          const cx = CX + Math.cos(a) * R
          const cy = CY + Math.sin(a) * R
          const isThresholdSlot = i < required
          const attested = i < count
          return (
            <circle
              key={i}
              cx={cx}
              cy={cy}
              r={attested ? 7 : isThresholdSlot ? 5.5 : 4}
              className={`qring-node ${attested ? 'on' : isThresholdSlot ? 'slot' : 'spare'}`}
              stroke="#fff"
              strokeWidth="2"
            />
          )
        })}

        {/* centre */}
        <text x={CX} y={CY - 6} textAnchor="middle" className="qring-big">
          {count}/{required}
        </text>
        <text x={CX} y={CY + 14} textAnchor="middle" className="qring-sub">
          {verified ? 'QUORUM MET' : remaining > 0 ? `need ${remaining} more` : 'ready'}
        </text>
        {verified && (
          <path
            d={`M ${CX - 9} ${CY + 28} l 6 7 l 13 -14`}
            stroke="#10b981"
            strokeWidth="2.5"
            fill="none"
            strokeLinecap="round"
            strokeLinejoin="round"
          />
        )}

        {/* legend column */}
        <g className="qring-legend">
          <text x={246} y={28} className="qring-legend-h">
            {configured ? 'QuorumConfig (0,[0,0])' : 'fallback DEFAULT_GEOMETRY_QUOROM'}
          </text>
          <text x={246} y={46} className="qring-legend-v">
            required = {configured ? quorumConfig : DEFAULT_GEOMETRY_QUOROM}
            {configured ? '' : ' (unset)'}
          </text>
          <text x={246} y={72} className="qring-legend-h">
            attestors [Pubkey; 8]
          </text>
          <text x={246} y={90} className="qring-legend-v">
            {count} filled · {MAX_VALIDATORS - count} zero-padding
          </text>
          <text x={246} y={116} className="qring-legend-h">
            dedup / bounds
          </text>
          <text x={246} y={134} className="qring-legend-v">
            6238 dup · 6240 full
          </text>
          <text x={246} y={160} className="qring-legend-h">
            flip condition
          </text>
          <text x={246} y={178} className="qring-legend-v">
            count ≥ required → verified
          </text>
        </g>
      </svg>

      <div className="qring-actions">
        {validators.map((v, i) => {
          const has = attestors.includes(v)
          return (
            <button
              key={v}
              className={`btn ${has ? 'btn-secondary' : 'btn-primary'} gap-1.5`}
              disabled={disabled || has || verified}
              onClick={() => onAttest(v)}
              title={has ? 'already attested (6238)' : `attest as demo validator ${i + 1}`}
            >
              {has ? <Check size={12} /> : <CircleUser size={12} />} v{i + 1}
              {has ? ' attested' : ' attest'}
            </button>
          )
        })}
      </div>
    </div>
  )
}
