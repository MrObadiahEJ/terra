import { describe, it, expect } from 'vitest'
import { LOD_LEVELS, lodFromHeight, lodFromLeafletZoom } from './lod'

describe('lodash — LOD level derivation', () => {
  it('exposes the full ordered LOD level table', () => {
    expect(LOD_LEVELS).toHaveLength(8)
    expect(LOD_LEVELS.map((l) => l.id)).toEqual([0, 1, 2, 3, 4, 5, 6, 7])
  })

  it('derives LOD level from height (strictly greater than the break)', () => {
    // HEIGHT_BREAKS_M = [5_000_000, 800_000, 150_000, 30_000, 3_000, 500, 80]
    expect(lodFromHeight(10_000_000)).toBe(0)
    expect(lodFromHeight(1_000_000)).toBe(1)
    expect(lodFromHeight(100_000)).toBe(3)
    expect(lodFromHeight(10_000)).toBe(4)
    expect(lodFromHeight(1_000)).toBe(5)
  })

  it('returns the finest level when height exceeds every break', () => {
    expect(lodFromHeight(Number.MAX_SAFE_INTEGER)).toBe(0)
  })

  it('derives LOD from leaflet zoom with clamping', () => {
    // ZOOM_TO_LOD has 21 entries (zoom 0..20).
    expect(lodFromLeafletZoom(0)).toBe(0)
    expect(lodFromLeafletZoom(2)).toBe(1)
    expect(lodFromLeafletZoom(5)).toBe(2)
    expect(lodFromLeafletZoom(10)).toBe(4)
    expect(lodFromLeafletZoom(20)).toBe(7)
    expect(lodFromLeafletZoom(99)).toBe(7) // clamped to max
    expect(lodFromLeafletZoom(-3)).toBe(0) // clamped to min
  })
})
