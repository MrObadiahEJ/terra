// Pure geometry helpers used by the demo layer and the Terra Lab experiments.
// Local approximations (equirectangular projection + haversine) — accurate to
// well under 1 % for the small parcels used in the pilot region.

export type LonLat = [number, number] // [lon, lat]

const EARTH_R = 6371008.8 // mean Earth radius (m)
const DEG_LAT_M = 110540 // metres per degree latitude
const DEG_LON_M_EQ = 111320 // metres per degree longitude at the equator

/** SHA-256 raw bytes of a UTF-8 string (WebCrypto, works offline). */
export async function sha256Bytes(input: string): Promise<Uint8Array> {
  const bytes = await crypto.subtle.digest('SHA-256', new TextEncoder().encode(input))
  return new Uint8Array(bytes)
}

/** SHA-256 hex digest of a UTF-8 string (WebCrypto, works offline). */
export async function sha256Hex(input: string): Promise<string> {
  const bytes = await sha256Bytes(input)
  return Array.from(bytes)
    .map((b) => b.toString(16).padStart(2, '0'))
    .join('')
}

/** Closed ring [lon, lat] GeoJSON-style string, matching RegisterParcelPanel. */
export function ringToGeometryJson(ring: LonLat[]): string {
  return JSON.stringify({ type: 'Polygon', coordinates: [ring] })
}

export function polygonBbox(ring: LonLat[]): {
  minLon: number
  minLat: number
  maxLon: number
  maxLat: number
} {
  let minLon = Infinity
  let minLat = Infinity
  let maxLon = -Infinity
  let maxLat = -Infinity
  for (const [lon, lat] of ring) {
    if (lon < minLon) minLon = lon
    if (lat < minLat) minLat = lat
    if (lon > maxLon) maxLon = lon
    if (lat > maxLat) maxLat = lat
  }
  return { minLon, minLat, maxLon, maxLat }
}

/** Polygon area in m² (shoelace over a local equirectangular projection). */
export function polygonAreaM2(ring: LonLat[]): number {
  const n = ring.length
  if (n < 3) return 0
  const lat0 = (ring.reduce((s, p) => s + p[1], 0) / n) * (Math.PI / 180)
  const kx = DEG_LON_M_EQ * Math.cos(lat0)
  const ky = DEG_LAT_M
  let sum = 0
  for (let i = 0; i < n; i++) {
    const [x1, y1] = ring[i]
    const [x2, y2] = ring[(i + 1) % n]
    sum += x1 * kx * (y2 * ky) - x2 * kx * (y1 * ky)
  }
  return Math.abs(sum / 2)
}

/** Area-weighted polygon centroid; falls back to vertex average when degenerate. */
export function polygonCentroid(ring: LonLat[]): LonLat {
  const n = ring.length
  if (n === 0) return [0, 0]
  if (n < 3) {
    return [
      ring.reduce((s, p) => s + p[0], 0) / n,
      ring.reduce((s, p) => s + p[1], 0) / n,
    ]
  }
  let a = 0
  let cx = 0
  let cy = 0
  for (let i = 0; i < n; i++) {
    const [x1, y1] = ring[i]
    const [x2, y2] = ring[(i + 1) % n]
    const cross = x1 * y2 - x2 * y1
    a += cross
    cx += (x1 + x2) * cross
    cy += (y1 + y2) * cross
  }
  if (Math.abs(a) < 1e-12) {
    return [
      ring.reduce((s, p) => s + p[0], 0) / n,
      ring.reduce((s, p) => s + p[1], 0) / n,
    ]
  }
  return [cx / (3 * a), cy / (3 * a)]
}

function haversineM(a: LonLat, b: LonLat): number {
  const toRad = Math.PI / 180
  const dLat = (b[1] - a[1]) * toRad
  const dLon = (b[0] - a[0]) * toRad
  const s =
    Math.sin(dLat / 2) ** 2 +
    Math.cos(a[1] * toRad) * Math.cos(b[1] * toRad) * Math.sin(dLon / 2) ** 2
  return 2 * EARTH_R * Math.asin(Math.min(1, Math.sqrt(s)))
}

/** Polygon perimeter in m (haversine over the ring edges). */
export function polygonPerimeterM(ring: LonLat[]): number {
  let total = 0
  for (let i = 0; i < ring.length; i++) {
    const j = (i + 1) % ring.length
    total += haversineM(ring[i], ring[j])
  }
  return total
}

/** LineString length in m (haversine). */
export function lineLengthM(coords: LonLat[]): number {
  let total = 0
  for (let i = 1; i < coords.length; i++) total += haversineM(coords[i - 1], coords[i])
  return total
}

/**
 * Canonical geometry digest: SHA-256 over JSON.stringify(ring) — byte-identical
 * to what RegisterParcelPanel anchors on-chain.
 */
export function ringDigest(ring: LonLat[]): Promise<string> {
  return sha256Hex(JSON.stringify(ring))
}

// ---- parcel drawing: precision, snapping, validation ----------------------

/** Round a raw pick to 1e-7° (~1 cm) to kill float noise from screen rays. */
export function quantizeLonLat(lon: number, lat: number): LonLat {
  return [Math.round(lon * 1e7) / 1e7, Math.round(lat * 1e7) / 1e7]
}

/** Planar metres between two lon/lat points (local equirectangular scale). */
function metresBetween(a: LonLat, b: LonLat): number {
  const lat0 = ((a[1] + b[1]) / 2) * (Math.PI / 180)
  const dx = (b[0] - a[0]) * DEG_LON_M_EQ * Math.cos(lat0)
  const dy = (b[1] - a[1]) * DEG_LAT_M
  return Math.hypot(dx, dy)
}

/** Point on segment ab closest to p (same local planar approximation). */
function closestOnSegment(p: LonLat, a: LonLat, b: LonLat): LonLat {
  const kx = Math.cos(p[1] * (Math.PI / 180)) * DEG_LON_M_EQ
  const ky = DEG_LAT_M
  const ax = a[0] * kx
  const ay = a[1] * ky
  const dx = b[0] * kx - ax
  const dy = b[1] * ky - ay
  const len2 = dx * dx + dy * dy
  if (len2 === 0) return a
  let t = ((p[0] * kx - ax) * dx + (p[1] * ky - ay) * dy) / len2
  t = Math.max(0, Math.min(1, t))
  return [a[0] + t * (b[0] - a[0]), a[1] + t * (b[1] - a[1])]
}

export interface SnapOptions {
  /** Own vertices under edit — always snapped (dedupe + close-on-first-corner). */
  own?: LonLat[]
  /** Candidate rings (parcel boundaries) to snap corners and edges onto. */
  rings?: LonLat[][]
  /** Fall back to a 1e-6° grid when nothing is within tolerance. */
  grid?: boolean
}

/**
 * Snap a raw pick to the nearest own vertex (3 m), boundary vertex (2 m) or
 * boundary edge (1.5 m); otherwise quantize to 1e-7° (or the 1e-6° grid).
 */
export function snapPoint(p: LonLat, opts: SnapOptions): LonLat {
  let best: LonLat = p
  let bestD = Infinity
  let found = false
  for (const v of opts.own ?? []) {
    const d = metresBetween(p, v)
    if (d <= 3 && d < bestD) {
      best = v
      bestD = d
      found = true
    }
  }
  for (const ring of opts.rings ?? []) {
    const n = ring.length
    for (let i = 0; i < n; i++) {
      const v = ring[i]
      const dv = metresBetween(p, v)
      if (dv <= 2 && dv < bestD) {
        best = v
        bestD = dv
        found = true
      }
      const q = closestOnSegment(p, v, ring[(i + 1) % n])
      const ds = metresBetween(p, q)
      if (ds <= 1.5 && ds < bestD) {
        best = q
        bestD = ds
        found = true
      }
    }
  }
  if (found) return quantizeLonLat(best[0], best[1])
  if (opts.grid) return [Math.round(p[0] * 1e6) / 1e6, Math.round(p[1] * 1e6) / 1e6]
  return quantizeLonLat(p[0], p[1])
}

export type RingValidation = { ok: true } | { ok: false; error: string }

/** Orientation sign of the turn a→b→c (2D cross product in lon/lat). */
function orientation(a: LonLat, b: LonLat, c: LonLat): number {
  const v = (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0])
  return v > 0 ? 1 : v < 0 ? -1 : 0
}

function withinBox(a: LonLat, b: LonLat, c: LonLat): boolean {
  return (
    Math.min(a[0], b[0]) <= c[0] &&
    c[0] <= Math.max(a[0], b[0]) &&
    Math.min(a[1], b[1]) <= c[1] &&
    c[1] <= Math.max(a[1], b[1])
  )
}

/** True when segments ab and cd cross or touch (including collinear overlap). */
function segmentsTouch(a: LonLat, b: LonLat, c: LonLat, d: LonLat): boolean {
  const o1 = orientation(a, b, c)
  const o2 = orientation(a, b, d)
  const o3 = orientation(c, d, a)
  const o4 = orientation(c, d, b)
  if (o1 !== o2 && o3 !== o4) return true
  if (o1 === 0 && withinBox(a, b, c)) return true
  if (o2 === 0 && withinBox(a, b, d)) return true
  if (o3 === 0 && withinBox(c, d, a)) return true
  if (o4 === 0 && withinBox(c, d, b)) return true
  return false
}

/** Validate drawn vertices: ≥3 points, finite coords, no self-touch, ≥1 m². */
export function validateRing(vertices: LonLat[]): RingValidation {
  const n = vertices.length
  if (n < 3) return { ok: false, error: 'Need at least 3 vertices' }
  for (const [lon, lat] of vertices) {
    if (!Number.isFinite(lon) || !Number.isFinite(lat)) {
      return { ok: false, error: 'Vertex has invalid coordinates' }
    }
  }
  for (let i = 0; i < n; i++) {
    for (let j = i + 1; j < n; j++) {
      if (j === i + 1 || (i === 0 && j === n - 1)) continue // adjacent corners
      if (segmentsTouch(vertices[i], vertices[(i + 1) % n], vertices[j], vertices[(j + 1) % n])) {
        return { ok: false, error: 'Boundary touches or crosses itself' }
      }
    }
  }
  if (polygonAreaM2(vertices) < 1) return { ok: false, error: 'Area is under 1 m²' }
  return { ok: true }
}
