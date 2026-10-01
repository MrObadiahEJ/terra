// Shared pure terrain math for the lab's WebGL-free land views (IsoTerrain
// card + LandViewer detail page). One source of truth for the isometric
// projection, seeded heightfield and face shading.

import type { LonLat } from './geo'

export type Pt = { x: number; y: number }
export type Cell = { gx: number; gy: number; h: number; on: boolean }

export const COS30 = Math.cos(Math.PI / 6)
export const SIN30 = 0.5

/** mulberry32 seeded from a hex hash → deterministic terrain. */
export function seeded(hex: string): () => number {
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

/** Classic 2:1 isometric projection: px = (x−y)·cos30, py = (x+y)·sin30 − z. */
export function projectIso(x: number, y: number, z: number): Pt {
  return { x: (x - y) * COS30, y: (x + y) * SIN30 - z }
}

/** Ray-cast point-in-polygon (closed ring in normalised unit space). */
export function inPoly(px: number, py: number, poly: number[][]): boolean {
  let hit = false
  for (let i = 0, j = poly.length - 1; i < poly.length; j = i++) {
    const [xi, yi] = poly[i]
    const [xj, yj] = poly[j]
    if (yi > py !== yj > py && px < ((xj - xi) * (py - yi)) / (yj - yi) + xi) hit = !hit
  }
  return hit
}

/** #rrggbb scaled toward black (k<1) or white (k>1). */
export function shade(hex: string, k: number): string {
  const n = parseInt(hex.slice(1), 16)
  const ch = (shift: number) => {
    const c = (n >> shift) & 0xff
    const v = k <= 1 ? Math.round(c * k) : Math.round(c + (255 - c) * (k - 1))
    return Math.max(0, Math.min(255, v))
  }
  return `#${[16, 8, 0].map((s) => ch(s).toString(16).padStart(2, '0')).join('')}`
}

/** Normalise a lon/lat ring into the unit square, with rot × 90° applied. */
export function normalizeRing(ring: LonLat[], rot: number): number[][] {
  const lons = ring.map((p) => p[0])
  const lats = ring.map((p) => p[1])
  const minLon = Math.min(...lons)
  const maxLon = Math.max(...lons)
  const minLat = Math.min(...lats)
  const maxLat = Math.max(...lats)
  const spanLon = maxLon - minLon || 1e-9
  const spanLat = maxLat - minLat || 1e-9
  return ring.map(([lon, lat]) => {
    let x = (lon - minLon) / spanLon
    let y = (lat - minLat) / spanLat
    for (let r = 0; r < rot; r++) {
      const t = x
      x = y
      y = 1 - t
    }
    return [x, y]
  })
}

/**
 * Deterministic per-cell heights for a unit-space ring.
 * `seed` should include rotation (e.g. seedHex + rot) so each orientation
 * gets its own stream. `flat` → uniform plate (D2 / NONE provenance).
 */
export function cellHeights(
  unit: number[][],
  seed: string,
  flat: boolean,
  grid = 14,
): Cell[] {
  const rand = seeded(seed)
  const cells: Cell[] = []
  for (let gy = 0; gy < grid; gy++) {
    for (let gx = 0; gx < grid; gx++) {
      const cx = (gx + 0.5) / grid
      const cy = (gy + 0.5) / grid
      const on = inPoly(cx, cy, unit)
      if (!on) continue
      const dx = cx - 0.5
      const dy = cy - 0.5
      const radial = 1 - Math.min(1, Math.sqrt(dx * dx + dy * dy) * 1.7)
      const n = rand()
      const h = flat
        ? 1
        : Math.max(0.06, Math.min(1, 0.35 * n + 0.65 * radial * (0.55 + 0.45 * n)))
      cells.push({ gx, gy, h, on })
    }
  }
  return cells
}

/** Fit a point cloud into a W×H box with padding, returning a projector. */
export function fitProjector(
  pts: Pt[],
  W: number,
  H: number,
  pad = 16,
): (p: Pt) => Pt {
  const xs = pts.map((p) => p.x)
  const ys = pts.map((p) => p.y)
  const minX = Math.min(...xs)
  const maxX = Math.max(...xs)
  const minY = Math.min(...ys)
  const maxY = Math.max(...ys)
  const scale = Math.min(
    (W - pad * 2) / Math.max(1e-6, maxX - minX),
    (H - pad * 2) / Math.max(1e-6, maxY - minY),
  )
  const ox = (W - (maxX - minX) * scale) / 2 - minX * scale
  const oy = (H - (maxY - minY) * scale) / 2 - minY * scale
  return (p) => ({ x: p.x * scale + ox, y: p.y * scale + oy })
}

/** Polygon points attribute string from fitted points. */
export function poly(pts: Pt[]): string {
  return pts.map((p) => `${p.x.toFixed(1)},${p.y.toFixed(1)}`).join(' ')
}
