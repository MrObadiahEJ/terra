import { useMemo, useState } from 'react'
import type { LonLat } from '../../lib/geo'
import { ELEVATION_SOURCES } from '../../lib/labStore'
import { RefreshCw, Mountain } from 'lucide-react'

// Isometric *terrain* of a parcel — a shaded heightfield of extruded columns
// rendered as plain SVG so it runs everywhere, including devices/browsers
// without WebGL (where Cesium/Leaflet 3D cannot run). Painter's algorithm
// sorts cells back-to-front; each column is a top + two side faces with
// hand-rolled hex shading (no filters, no shaders, no canvas).
//
// Projection: classic 2:1 isometric —
//   px = (x - y) · cos30
//   py = (x + y) · sin30 − z

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

const COS30 = Math.cos(Math.PI / 6)
const SIN30 = 0.5
const GRID = 14

type Pt = { x: number; y: number }

/** mulberry32 seeded from a hex hash → deterministic terrain. */
function seeded(hex: string): () => number {
  let a = 0x9e3779b9
  for (let i = 0; i < hex.length; i += 2) {
    a = (a + parseInt(hex.slice(i, i + 2), 16)) | 0
  }
  return () => {
    a |= 0
    a = (a + 0x6d2b79f5) | 0
    let t = Math.imul(a ^ (a >>> 15), 1 | a)
    t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296
  }
}

function project(x: number, y: number, z: number): Pt {
  return { x: (x - y) * COS30, y: (x + y) * SIN30 - z }
}

/** Ray-cast point-in-polygon (ring: closed lon/lat loop, normalised space). */
function inPoly(px: number, py: number, poly: number[][]): boolean {
  let hit = false
  for (let i = 0, j = poly.length - 1; i < poly.length; j = i++) {
    const [xi, yi] = poly[i]
    const [xj, yj] = poly[j]
    if (yi > py !== yj > py && px < ((xj - xi) * (py - yi)) / (yj - yi) + xi) hit = !hit
  }
  return hit
}

/** #rrggbb scaled toward black (k<1) or white (k>1). */
function shade(hex: string, k: number): string {
  const n = parseInt(hex.slice(1), 16)
  const ch = (shift: number) => {
    const c = (n >> shift) & 0xff
    const v = k <= 1 ? Math.round(c * k) : Math.round(c + (255 - c) * (k - 1))
    return Math.max(0, Math.min(255, v))
  }
  return `#${[16, 8, 0].map((s) => ch(s).toString(16).padStart(2, '0')).join('')}`
}

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
    const lons = ring.map((p) => p[0])
    const lats = ring.map((p) => p[1])
    const minLon = Math.min(...lons)
    const maxLon = Math.max(...lons)
    const minLat = Math.min(...lats)
    const maxLat = Math.max(...lats)
    const spanLon = maxLon - minLon || 1e-9
    const spanLat = maxLat - minLat || 1e-9
    const unit: number[][] = ring.map(([lon, lat]) => {
      let x = (lon - minLon) / spanLon
      let y = (lat - minLat) / spanLat
      for (let r = 0; r < rot; r++) {
        const t = x
        x = y
        y = 1 - t // 90° step
      }
      return [x, y]
    })

    // 2) deterministic per-cell heights
    const rand = seeded(seedHex + String(rot))
    const flat = dimension === 0 || elevationSource === 0
    const spanM = Math.max(0, elevMaxM - elevMinM)
    const cells: {
      gx: number
      gy: number
      h: number // 0..1
      on: boolean
    }[] = []
    for (let gy = 0; gy < GRID; gy++) {
      for (let gx = 0; gx < GRID; gx++) {
        const cx = (gx + 0.5) / GRID
        const cy = (gy + 0.5) / GRID
        const on = inPoly(cx, cy, unit)
        if (!on) continue
        // radial falloff + smooth-ish noise → readable landform
        const dx = cx - 0.5
        const dy = cy - 0.5
        const radial = 1 - Math.min(1, Math.sqrt(dx * dx + dy * dy) * 1.7)
        const n = rand()
        const h = flat ? 1 : Math.max(0.06, Math.min(1, 0.35 * n + 0.65 * radial * (0.55 + 0.45 * n)))
        cells.push({ gx, gy, h, on })
      }
    }
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
    const xs = all.map((p) => p.x)
    const ys = all.map((p) => p.y)
    const W = 420
    const Ht = 236
    const pad = 16
    const minX = Math.min(...xs)
    const maxX = Math.max(...xs)
    const minY = Math.min(...ys)
    const maxY = Math.max(...ys)
    const scale = Math.min(
      (W - pad * 2) / Math.max(1e-6, maxX - minX),
      (Ht - pad * 2) / Math.max(1e-6, maxY - minY),
    )
    const ox = (W - (maxX - minX) * scale) / 2 - minX * scale
    const oy = (Ht - (maxY - minY) * scale) / 2 - minY * scale
    const fit = (p: Pt): Pt => ({ x: p.x * scale + ox, y: p.y * scale + oy })
    const poly = (pts: Pt[]) => pts.map((p) => `${p.x.toFixed(1)},${p.y.toFixed(1)}`).join(' ')

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
