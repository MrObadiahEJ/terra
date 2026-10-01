import { useMemo, useState } from 'react'
import type { LonLat } from '../../lib/geo'
import { ELEVATION_SOURCES } from '../../lib/labStore'
import {
  cellHeights,
  fitProjector,
  normalizeRing,
  poly,
  projectIso as project,
  shade,
} from '../../lib/terrain'
import { RefreshCw, Mountain } from 'lucide-react'

// Isometric *terrain* of a parcel — a shaded heightfield of extruded columns
// rendered as plain SVG so it runs everywhere, including devices/browsers
// without WebGL (where Cesium/Leaflet 3D cannot run). Painter's algorithm
// sorts cells back-to-front; each column is a top + two side faces with
// hand-rolled hex shading (no filters, no shaders, no canvas).
// Shared math lives in lib/terrain.ts (with LandViewer).

interface Props {
  ring: LonLat[]
  elevMinM: number
  elevMaxM: number
  elevationSource: number
  dimension: number
  seedHex: string
  verified?: boolean
  attestCount?: number
  required?: number
}

const GRID = 14

export default function IsoTerrain({
  ring,
  elevMinM,
  elevMaxM,
  elevationSource,
  dimension,
  seedHex,
  verified = false,
  attestCount = 0,
  required = 2,
}: Props) {
  const [rot, setRot] = useState(0)

  const terrain = useMemo(() => {
    if (ring.length < 3) return null

    // 1) normalise ring to the unit square (rotation applied here)
    const unit = normalizeRing(ring, rot)

    // 2) deterministic per-cell heights
    const flat = dimension === 0 || elevationSource === 0
    const spanM = Math.max(0, elevMaxM - elevMinM)
    const cells = cellHeights(unit, seedHex + String(rot), flat, GRID)
    if (cells.length === 0) return null

    // 3) project: normalised footprint → centred iso plane; z = height
    const S = 2.1 // world scale inside the unit square
    const H0 = 0.14
    const H1 = 0.52 + Math.min(0.25, spanM / 2000) // taller spans read taller
    const col = ELEVATION_SOURCES[elevationSource]?.color ?? '#9ca3af'

    const base = unit.map(([x, y]) => [(x - 0.5) * S, (y - 0.5) * S])
    const cols = cells.map((c) => {
      const x0 = (c.gx / GRID - 0.5) * S
      const x1 = ((c.gx + 1) / GRID - 0.5) * S
      const y0 = (c.gy / GRID - 0.5) * S
      const y1 = ((c.gy + 1) / GRID - 0.5) * S
      const z = (H0 + (H1 - H0) * c.h) * (flat ? 0.45 : 1)
      const top = [project(x0, y0, z), project(x1, y0, z), project(x1, y1, z), project(x0, y1, z)]
      // side faces: south (+y) and east (+x) — visible in this projection
      const south = [project(x0, y1, z), project(x1, y1, z), project(x1, y1, 0), project(x0, y1, 0)]
      const east = [project(x1, y0, z), project(x1, y1, z), project(x1, y1, 0), project(x1, y0, 0)]
      return { ...c, top, south, east, z }
    })
    cols.sort((a, b) => a.gx + a.gy - (b.gx + b.gy))

    const outline = base.map(([x, y]) => project(x, y, 0))

    // 4) fit everything into the viewBox
    const all = [...outline, ...cols.flatMap((c) => [...c.top, ...c.south, ...c.east])]
    const W = 420
    const Ht = 236
    const fit = fitProjector(all, W, Ht, 16)

    const topPoly = cols.map((c) => poly(c.top.map(fit)))
    const southPoly = cols.map((c) => poly(c.south.map(fit)))
    const eastPoly = cols.map((c) => poly(c.east.map(fit)))
    const centroid = fit(project(0, 0, H1 * (flat ? 0.45 : 1)))

    return {
      outline: poly(outline.map(fit)),
      topPoly,
      southPoly,
      eastPoly,
      heights: cols.map((c) => c.h),
      cellCount: cols.length,
      color: col,
      centroid,
      baseColor: verified ? '#10b981' : col,
      flat,
      spanM,
      elevLabel: flat
        ? elevationSource === 0
          ? 'flat · elevation_source NONE'
          : 'flat slice'
        : `${elevMinM} … ${elevMaxM} m`,
    }
  }, [ring, rot, seedHex, elevationSource, dimension, elevMinM, elevMaxM, verified])

  if (!terrain) {
    return (
      <div className="iso-empty">
        No geometry yet — draw or anchor a shape first.
      </div>
    )
  }

  const prov = ELEVATION_SOURCES[elevationSource]

  return (
    <div className="isoterrain">
      <div className="isoterrain-toolbar">
        <span className="lab-badge" style={{ background: `${terrain.color}22`, color: shade(terrain.color, 0.7) }}>
          <Mountain size={11} /> {prov?.id ?? 'NONE'}
        </span>
        <span className="text-[11px] text-muted font-mono">{terrain.elevLabel}</span>
        <span className="flex-1" />
        <span className="text-[10px] text-muted">
          {terrain.cellCount} columns · {attestCount}/{required} attested
        </span>
        <button className="btn btn-ghost p-1" title="Rotate 90°" onClick={() => setRot((r) => (r + 1) % 4)}>
          <RefreshCw size={13} />
        </button>
      </div>

      <svg
        viewBox="0 0 420 236"
        width="100%"
        height={236}
        preserveAspectRatio="xMidYMid meet"
        role="img"
        aria-label="Isometric SVG terrain of the parcel (no WebGL)"
      >
        <defs>
          <linearGradient id="isot-sky" x1="0" y1="0" x2="0" y2="1">
            <stop offset="0%" stopColor="#f8fafc" />
            <stop offset="100%" stopColor="#eef2f7" />
          </linearGradient>
          <radialGradient id="isot-halo" cx="50%" cy="50%" r="50%">
            <stop offset="0%" stopColor="#10b981" stopOpacity="0.35" />
            <stop offset="100%" stopColor="#10b981" stopOpacity="0" />
          </radialGradient>
        </defs>

        <rect x="0" y="0" width="420" height="236" rx="8" fill="url(#isot-sky)" />

        {/* footplate */}
        <polygon
          points={terrain.outline}
          fill="rgba(15,23,42,0.04)"
          stroke="#94a3b8"
          strokeWidth="1"
          strokeDasharray="4 3"
        />

        {/* quorum halo under a verified landform */}
        {verified && (
          <circle cx={terrain.centroid.x} cy={terrain.centroid.y} r="64" fill="url(#isot-halo)" className="isot-halo" />
        )}

        {/* columns — painter's order, staggered rise-in */}
        {terrain.heights.map((h, i) => {
          const t = shade(terrain.baseColor, 1.18 - 0.42 * h)
          const s = shade(terrain.baseColor, 0.62)
          const e = shade(terrain.baseColor, 0.44)
          const delay = (i % 28) * 14
          return (
            <g key={i} className="isot-col" style={{ animationDelay: `${delay}ms` }}>
              <polygon points={terrain.southPoly[i]} fill={s} stroke="rgba(15,23,42,0.18)" strokeWidth="0.4" />
              <polygon points={terrain.eastPoly[i]} fill={e} stroke="rgba(15,23,42,0.18)" strokeWidth="0.4" />
              <polygon
                points={terrain.topPoly[i]}
                fill={t}
                stroke={verified ? '#10b981' : 'rgba(15,23,42,0.22)'}
                strokeWidth={verified ? 0.7 : 0.45}
              />
            </g>
          )
        })}

        {/* quorum pips at the summit */}
        {Array.from({ length: required }).map((_, i) => {
          const a = -Math.PI / 2 + (i / Math.max(1, required)) * Math.PI * 2
          const r = 13
          const cx = terrain.centroid.x + Math.cos(a) * r
          const cy = terrain.centroid.y + Math.sin(a) * r * 0.6
          const filled = i < attestCount
          return (
            <circle
              key={`pip${i}`}
              cx={cx}
              cy={cy}
              r={filled ? 3.4 : 2.4}
              className={filled ? 'isot-pip on' : 'isot-pip'}
              fill={filled ? '#10b981' : '#cbd5e1'}
              stroke="#fff"
              strokeWidth="1"
            />
          )
        })}
      </svg>

      {/* quorum gauge (segmented: one slot per required attestor) */}
      <div className="isoterrain-scale">
        <span className="text-[10px] text-muted font-mono">{Math.round(elevMinM)}m</span>
        <div className="isoterrain-segs" title={`threshold = ${required}`}>
          {Array.from({ length: required }).map((_, i) => (
            <span
              key={`seg${i}`}
              className={`isoterrain-seg ${i < attestCount ? 'on' : ''} ${verified ? 'done' : ''}`}
            />
          ))}
        </div>
        <span className="text-[10px] text-muted font-mono">{Math.round(elevMaxM)}m</span>
        <span className="lab-badge lab-badge-info">SVG · no WebGL</span>
      </div>
    </div>
  )
}
