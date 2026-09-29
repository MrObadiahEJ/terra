import type { LonLat } from '../../lib/geo'

// Isometric wireframe of a parcel — a "skeleton of the land" rendered as
// plain SVG so it works on devices without WebGL (where Cesium/Leaflet 3D
// cannot run). Pure geometry, no state, no deps.
//
// Projection: classic 2:1 isometric —
//   px = (x - y) · cos30
//   py = (x + y) · sin30 − z
interface Props {
  ring: LonLat[]
  elevMinM?: number
  elevMaxM?: number
  compact?: boolean
}

const COS30 = Math.cos(Math.PI / 6)
const SIN30 = 0.5

type Pt = { x: number; y: number }

function project(x: number, y: number, z: number): Pt {
  return { x: (x - y) * COS30, y: (x + y) * SIN30 - z }
}

export default function IsoLandSkeleton({ ring, elevMinM = 0, elevMaxM = 0, compact = false }: Props) {
  if (ring.length < 3) {
    return (
      <div className={`iso-empty ${compact ? 'compact' : ''}`}>
        {compact ? '—' : 'No geometry yet — draw or anchor a shape first.'}
      </div>
    )
  }

  // 1) geodetic → local metres (equirectangular around the centroid)
  const cLon = ring.reduce((s, p) => s + p[0], 0) / ring.length
  const cLat = ring.reduce((s, p) => s + p[1], 0) / ring.length
  const mPerLon = 111_320 * Math.cos((cLat * Math.PI) / 180)
  const mPerLat = 110_540
  const local = ring.map(([lon, lat]) => [(lon - cLon) * mPerLon, (lat - cLat) * mPerLat])

  // real footprint size (for labels)
  const lons = ring.map((p) => p[0])
  const lats = ring.map((p) => p[1])
  const widthM = (Math.max(...lons) - Math.min(...lons)) * mPerLon
  const heightM = (Math.max(...lats) - Math.min(...lats)) * mPerLat

  // 2) normalise to ±0.9 so any parcel fits the same box
  const maxAbs = Math.max(1e-6, ...local.map(([x, y]) => Math.max(Math.abs(x), Math.abs(y))))
  const s = 0.9 / maxAbs
  const norm = local.map(([x, y]) => [x * s, y * s])

  // 3) skeleton slab: ground plane + top plane lifted by a fixed isometric
  //    height (exaggerated so the 3D read is obvious at any real elevation)
  const dz = Math.max(0, elevMaxM - elevMinM)
  const H = 0.42
  const ground = norm.map(([x, y]) => project(x, y, 0))
  const top = norm.map(([x, y]) => project(x, y, H))
  const gCentroid = project(0, 0, 0)
  const tCentroid = project(0, 0, H)

  // 4) fit every projected point into the viewBox
  const all = [...ground, ...top, gCentroid, tCentroid]
  const xs = all.map((p) => p.x)
  const ys = all.map((p) => p.y)
  const pad = compact ? 5 : 14
  const W = compact ? 120 : 340
  const Ht = compact ? 74 : 180
  const minX = Math.min(...xs)
  const maxX = Math.max(...xs)
  const minY = Math.min(...ys)
  const maxY = Math.max(...ys)
  const scale = Math.min((W - pad * 2) / Math.max(1e-6, maxX - minX), (Ht - pad * 2) / Math.max(1e-6, maxY - minY))
  const ox = (W - (maxX - minX) * scale) / 2 - minX * scale
  const oy = (Ht - (maxY - minY) * scale) / 2 - minY * scale
  const fit = (p: Pt): Pt => ({ x: p.x * scale + ox, y: p.y * scale + oy })

  const gPts = ground.map(fit)
  const tPts = top.map(fit)
  const gc = fit(gCentroid)
  const tc = fit(tCentroid)
  const poly = (pts: Pt[]) => pts.map((p) => `${p.x.toFixed(1)},${p.y.toFixed(1)}`).join(' ')

  return (
    <div className={`iso ${compact ? 'compact' : ''}`}>
      <svg
        viewBox={`0 0 ${W} ${Ht}`}
        width="100%"
        height={Ht}
        preserveAspectRatio="xMidYMid meet"
        role="img"
        aria-label="Isometric wireframe of the parcel"
      >
        {/* ground plane */}
        <polygon
          points={poly(gPts)}
          fill="rgba(16,185,129,0.05)"
          stroke="#9ca3af"
          strokeWidth={compact ? 1 : 1.2}
          strokeDasharray="4 3"
        />
        {/* extrusion pillars */}
        {gPts.map((g, i) => (
          <line key={`p${i}`} x1={g.x} y1={g.y} x2={tPts[i].x} y2={tPts[i].y} stroke="#6b7280" strokeWidth={compact ? 1 : 1.2} />
        ))}
        {/* top plane (land surface) */}
        <polygon
          points={poly(tPts)}
          fill="rgba(16,185,129,0.10)"
          stroke="#10b981"
          strokeWidth={compact ? 1.3 : 1.8}
        />
        {/* skeleton fan across the surface */}
        {!compact &&
          tPts.map((t, i) => (
            <line key={`f${i}`} x1={tc.x} y1={tc.y} x2={t.x} y2={t.y} stroke="#10b981" strokeWidth="0.8" opacity="0.4" />
          ))}
        {/* centre axis */}
        <line x1={gc.x} y1={gc.y} x2={tc.x} y2={tc.y} stroke="#10b981" strokeWidth="1.2" strokeDasharray="3 2" />
        {!compact && <circle cx={gc.x} cy={gc.y} r="2" fill="#6b7280" />}
        <circle cx={tc.x} cy={tc.y} r={compact ? 1.6 : 2.4} fill="#10b981" />
        {/* top vertices */}
        {tPts.map((t, i) => (
          <circle key={`v${i}`} cx={t.x} cy={t.y} r={compact ? 1.2 : 2} fill="#065f46" />
        ))}
      </svg>
      {!compact && (
        <div className="iso-meta">
          <span>
            footprint {Math.round(widthM)} × {Math.round(heightM)} m
          </span>
          <span>
            Δz = {Math.round(dz)} m ({elevMinM}…{elevMaxM})
          </span>
          <span>SVG isometric · no WebGL</span>
        </div>
      )}
    </div>
  )
}
