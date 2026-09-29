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
