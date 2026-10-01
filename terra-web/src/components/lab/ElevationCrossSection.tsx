import { useMemo } from 'react'
import { ELEVATION_SOURCES, SPATIAL_DIMENSIONS, elevationInvariants } from '../../lib/labStore'

// Live elevation-provenance cross-section (RFC-013 Phase B §4) — a side view
// of the asset bounding envelope with the proposed version envelope inside
// (or spilling out of) it. Plain SVG + HTML chips; renders without WebGL.

interface Props {
  assetMinM: number
  assetMaxM: number
  versionMinM: number
  versionMaxM: number
  elevSource: number
  dimension: number
  assetDimension: number
}

const W = 400
const H = 186
const LEFT = 46
const RIGHT = 384
const TOP = 14
const BOTTOM = 168

export default function ElevationCrossSection({
  assetMinM,
  assetMaxM,
  versionMinM,
  versionMaxM,
  elevSource,
  dimension,
  assetDimension,
}: Props) {
  const prov = ELEVATION_SOURCES[elevSource]
  const color = prov?.color ?? '#9ca3af'
  const dimId = SPATIAL_DIMENSIONS[dimension]?.id ?? '?'
  const assetDimId = SPATIAL_DIMENSIONS[assetDimension]?.id ?? '?'

  const view = useMemo(() => {
    const pad = Math.max(20, (assetMaxM - assetMinM) * 0.12)
    let lo = Math.min(assetMinM, versionMinM, 0) - pad
    let hi = Math.max(assetMaxM, versionMaxM, 0) + pad
    if (versionMinM > versionMaxM) {
      // inverted: still draw something readable around the midpoint
      const mid = (versionMinM + versionMaxM) / 2
      lo = Math.min(lo, mid - pad)
      hi = Math.max(hi, mid + pad)
    }
    if (hi - lo < 1) hi = lo + 1
    const y = (m: number) => BOTTOM - ((m - lo) / (hi - lo)) * (BOTTOM - TOP)
    return { lo, hi, y }
  }, [assetMinM, assetMaxM, versionMinM, versionMaxM])

  const inv = elevationInvariants(
    elevSource,
    dimension,
    versionMinM,
    versionMaxM,
    Math.round(assetMinM * 1000),
    Math.round(assetMaxM * 1000),
  )
  const allOk = inv.every((c) => c.ok)

  const assetTop = view.y(assetMaxM)
  const assetBot = view.y(assetMinM)
  const vTop = view.y(versionMaxM)
  const vBot = view.y(versionMinM)
  const inverted = versionMinM > versionMaxM
  const outside =
    !inverted && (versionMinM < assetMinM || versionMaxM > assetMaxM)
  const seaY = view.y(0)
  const showSea = 0 >= view.lo && 0 <= view.hi

  const vFill = inverted ? '#ef4444' : outside ? `${color}aa` : color
  const midY = (vTop + vBot) / 2

  return (
    <div className="xsec">
      <svg
        viewBox={`0 0 ${W} ${H}`}
        width="100%"
        height={H}
        preserveAspectRatio="xMidYMid meet"
        role="img"
        aria-label="Elevation cross-section (SVG, no WebGL)"
      >
        <defs>
          <linearGradient id="xsec-asset" x1="0" y1="0" x2="0" y2="1">
            <stop offset="0%" stopColor="#10b981" stopOpacity="0.10" />
            <stop offset="100%" stopColor="#10b981" stopOpacity="0.03" />
          </linearGradient>
          <pattern id="xsec-hatch" width="6" height="6" patternTransform="rotate(45)" patternUnits="userSpaceOnUse">
            <line x1="0" y1="0" x2="0" y2="6" stroke="#ef4444" strokeWidth="2.4" opacity="0.55" />
          </pattern>
        </defs>

        {/* axis */}
        <line x1={LEFT} y1={TOP} x2={LEFT} y2={BOTTOM} stroke="#cbd5e1" strokeWidth="1" />
        <text x={LEFT - 6} y={TOP + 8} textAnchor="end" className="xsec-tick">
          {Math.round(view.hi)}m
        </text>
        <text x={LEFT - 6} y={BOTTOM} textAnchor="end" className="xsec-tick">
          {Math.round(view.lo)}m
        </text>

        {/* asset bounding envelope */}
        <rect
          x={LEFT}
          y={assetTop}
          width={RIGHT - LEFT}
          height={Math.max(2, assetBot - assetTop)}
          fill="url(#xsec-asset)"
          stroke="#10b981"
          strokeWidth="1.2"
          strokeDasharray="5 3"
          rx="3"
        />
        <text x={LEFT + 6} y={assetTop + 12} className="xsec-label-em">
          asset envelope {Math.round(assetMinM)}…{Math.round(assetMaxM)} m · {assetDimId}
        </text>

        {/* sea level */}
        {showSea && (
          <>
            <line
              x1={LEFT}
              y1={seaY}
              x2={RIGHT}
              y2={seaY}
              stroke="#3b82f6"
              strokeWidth="1"
              strokeDasharray="2 3"
            />
            <text x={RIGHT} y={seaY - 4} textAnchor="end" className="xsec-label-sea">
              0 m
            </text>
          </>
        )}

        {/* version envelope */}
        {!inverted ? (
          <>
            <rect
              x={LEFT + 40}
              y={vTop}
              width={RIGHT - LEFT - 80}
              height={Math.max(3, vBot - vTop)}
              rx="3"
              fill={vFill}
              opacity={outside ? 0.75 : 0.9}
              className={`xsec-v ${outside ? 'bad' : ''}`}
            />
            {/* spill markers where the version leaves the asset box */}
            {outside && (
              <rect
                x={LEFT + 40}
                y={vTop}
                width={RIGHT - LEFT - 80}
                height={Math.max(3, vBot - vTop)}
                rx="3"
                fill="url(#xsec-hatch)"
                className="xsec-spill"
              />
            )}
            <text
              x={LEFT + 46}
              y={Math.max(TOP + 10, vTop - 4)}
              className="xsec-label-v"
              fill={color}
            >
              version {Math.round(versionMinM)}…{Math.round(versionMaxM)} m · {prov?.id} ({dimId})
            </text>
          </>
        ) : (
          <>
            <rect
              x={LEFT + 40}
              y={midY - 5}
              width={RIGHT - LEFT - 80}
              height="10"
              rx="3"
              fill="url(#xsec-hatch)"
              stroke="#ef4444"
              strokeWidth="1"
            />
            <text x={LEFT + 46} y={midY - 10} className="xsec-label-v" fill="#ef4444">
              inverted envelope ({versionMinM} &gt; {versionMaxM}) → 6235
            </text>
          </>
        )}

        {/* scan sweep */}
        <rect className="xsec-scan" x={LEFT} y={TOP} width="26" height={BOTTOM - TOP} fill="#10b981" opacity="0.06" />
      </svg>

      <div className="xsec-chips">
        <span className={`xsec-chip ${allOk ? 'ok' : 'bad'}`}>
          {allOk ? 'all 5 invariants ✓' : `${inv.filter((c) => !c.ok).length} violated`}
        </span>
        {inv.map((c) => (
          <span key={c.id} className={`xsec-chip mini ${c.ok ? 'ok' : 'bad'}`} title={c.label}>
            {c.ok ? '✓' : '✕'} {c.label}
          </span>
        ))}
      </div>
    </div>
  )
}
