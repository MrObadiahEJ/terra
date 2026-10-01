// Evidence → geometry chain (RFC-013 Phase B): Task → EvidenceManifest →
// artifact(GEOMETRY) → GeometryVersion, with the terminal/empty guards
// visible as blocked links. Single plain SVG row — no WebGL, no layout deps.

interface Props {
  taskCancelled: boolean
  manifestNonce: number | null
  manifestArtifacts: number
  linkedManifest: string | null // from the inspected version (null = standalone)
  linkedVersion: number | null
}

const W = 640
const H = 168

interface NodeDef {
  x: number
  title: string
  line1: string
  line2: string
  state: 'ok' | 'blocked' | 'idle'
}

export default function EvidenceChain({
  taskCancelled,
  manifestNonce,
  manifestArtifacts,
  linkedManifest,
  linkedVersion,
}: Props) {
  const hasManifest = manifestNonce !== null
  const hasArtifacts = manifestArtifacts >= 1
  const linked = linkedManifest !== null

  const nodes: NodeDef[] = [
    {
      x: 8,
      title: 'VerificationTask',
      line1: taskCancelled ? 'CANCELLED' : 'OPEN',
      line2: taskCancelled ? 'terminal → 6174' : 'can_attach_evidence ✓',
      state: taskCancelled ? 'blocked' : 'ok',
    },
    {
      x: 168,
      title: 'EvidenceManifest',
      line1: hasManifest ? `#${manifestNonce}` : 'not submitted',
      line2: hasManifest ? `${manifestArtifacts} artifact${manifestArtifacts === 1 ? '' : 's'}` : 'seeds: task, submitter, nonce',
      state: !hasManifest ? 'idle' : hasArtifacts ? 'ok' : 'blocked',
    },
    {
      x: 328,
      title: 'artifact[0] GEOMETRY',
      line1: hasArtifacts ? 'kind = 4 (GEOMETRY)' : 'empty list',
      line2: hasArtifacts ? 'hash + storage_reference ✓' : '→ 6239',
      state: hasArtifacts ? 'ok' : 'blocked',
    },
    {
      x: 488,
      title: 'GeometryVersion',
      line1:
        linked && linkedVersion !== null
          ? `v${linkedVersion} ← manifest`
          : linkedManifest === null && linkedVersion !== null
            ? `v${linkedVersion} standalone`
            : 'not anchored yet',
      line2: linked ? `evidence_manifest = ${linkedManifest}` : 'evidence_manifest = default()',
      state: linked ? 'ok' : 'idle',
    },
  ]

  const COLORS = {
    ok: { stroke: '#10b981', fill: '#ecfdf5', text: '#065f46' },
    blocked: { stroke: '#ef4444', fill: '#fef2f2', text: '#991b1b' },
    idle: { stroke: '#cbd5e1', fill: '#f8fafc', text: '#6b7280' },
  }

  const linkState = (i: number): 'ok' | 'idle' | 'blocked' => {
    const a = nodes[i].state
    const b = nodes[i + 1].state
    if (a === 'blocked' || b === 'blocked') return 'blocked'
    if (a === 'ok' && b === 'ok') return 'ok'
    return 'idle'
  }

  return (
    <div className="echain">
      <svg
        viewBox={`0 0 ${W} ${H}`}
        width="100%"
        height={H}
        preserveAspectRatio="xMidYMid meet"
        role="img"
        aria-label="Evidence chain (SVG, no WebGL)"
      >
        {/* links (behind nodes) */}
        {nodes.slice(0, -1).map((n, i) => {
          const st = linkState(i)
          const x1 = n.x + 144
          const x2 = nodes[i + 1].x
          const y = 64
          const color = st === 'ok' ? '#10b981' : st === 'blocked' ? '#fca5a5' : '#cbd5e1'
          return (
            <g key={`link${i}`}>
              <path
                d={`M ${x1} ${y} C ${x1 + 12} ${y}, ${x2 - 12} ${y}, ${x2} ${y}`}
                fill="none"
                stroke={color}
                strokeWidth="2.5"
                strokeDasharray="7 5"
                className={st === 'ok' ? 'echain-flow' : st === 'blocked' ? '' : 'echain-dim'}
              />
              {/* arrow head */}
              <path
                d={`M ${x2 - 7} ${y - 4.5} L ${x2} ${y} L ${x2 - 7} ${y + 4.5}`}
                fill="none"
                stroke={color}
                strokeWidth="2"
                strokeLinecap="round"
                strokeLinejoin="round"
              />
            </g>
          )
        })}

        {/* nodes */}
        {nodes.map((n) => {
          const c = COLORS[n.state]
          return (
            <g key={n.title} className={`echain-node ${n.state}`}>
              <rect
                x={n.x}
                y={32}
                width={144}
                height={64}
                rx="8"
                fill={c.fill}
                stroke={c.stroke}
                strokeWidth="1.6"
              />
              <text x={n.x + 10} y={50} className="echain-h" fill={c.text}>
                {n.title}
              </text>
              <text x={n.x + 10} y={68} className="echain-v">
                {n.line1}
              </text>
              <text x={n.x + 10} y={85} className="echain-s">
                {n.line2.length > 34 ? `${n.line2.slice(0, 33)}…` : n.line2}
              </text>
              {n.state === 'blocked' && (
                <g>
                  <circle cx={n.x + 132} cy={44} r="7" fill="#ef4444" />
                  <text x={n.x + 132} y={48} textAnchor="middle" className="echain-lock">
                    !
                  </text>
                </g>
              )}
              {n.state === 'ok' && (
                <circle cx={n.x + 132} cy={44} r="5" fill="#10b981" className="echain-dot" />
              )}
            </g>
          )
        })}

        {/* guards caption */}
        <text x={8} y={130} className="echain-cap">
          append_geometry_version_from_evidence: no seed instruction args — manifest PDA self-derived from
        </text>
        <text x={8} y={146} className="echain-cap">
          (task_id, submitter, nonce); guards in order: task terminal → 6174 · artifact_count ≥ 1 → 6239.
        </text>
      </svg>
    </div>
  )
}
