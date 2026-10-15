import { describe, it, expect } from 'vitest'
import {
  parseRoadSegments,
  snapToRoadNetwork,
  routeOnRoads,
  buildRoadRoute,
} from './roads'
import { DEMO_ROADS } from '../lib/demoData'
import type { RoadRow } from '../lib/api'

function lineString(id: number, name: string, coords: number[][], oneway = false): RoadRow {
  return {
    id,
    name,
    highway: 'primary',
    oneway,
    geometry: JSON.stringify({ type: 'LineString', coordinates: coords }),
  } as unknown as RoadRow
}

describe('parseRoadSegments', () => {
  it('flattens a LineString into one segment per edge', () => {
    const rows = [
      lineString(1, 'A', [
        [11.0, 3.0],
        [11.001, 3.0],
        [11.002, 3.0],
      ]),
    ]
    const segs = parseRoadSegments(rows)
    expect(segs).toHaveLength(2)
    expect(segs.every((s) => s.lengthM > 0)).toBe(true)
  })

  it('skips unusable geometry without throwing', () => {
    const rows = [
      { id: 2, name: null, highway: 'x', oneway: false, geometry: null } as unknown as RoadRow,
      { id: 3, name: null, highway: 'x', oneway: false, geometry: '{bad json' } as unknown as RoadRow,
      lineString(4, 'ok', [
        [11.0, 3.0],
        [11.001, 3.0],
      ]),
    ]
    const segs = parseRoadSegments(rows)
    expect(segs).toHaveLength(1)
  })
})

describe('snapToRoadNetwork', () => {
  it('projects a point onto the nearest segment', () => {
    const segs = parseRoadSegments([
      lineString(1, 'A', [
        [11.0, 3.0],
        [11.01, 3.0],
      ]),
    ])
    const snapped = snapToRoadNetwork(segs, [11.005, 3.0005])
    expect(snapped).not.toBeNull()
    expect(snapped?.distanceM).toBeLessThan(100)
    expect(snapped?.t).toBeGreaterThan(0.4)
    expect(snapped?.t).toBeLessThan(0.6)
  })

  it('returns null for an empty network', () => {
    expect(snapToRoadNetwork([], [11, 3])).toBeNull()
  })
})

describe('routeOnRoads on the bundled demo network', () => {
  const segments = parseRoadSegments(DEMO_ROADS)
  const rn1 = DEMO_ROADS.find((r) => r.id === 1)!
  const coords: [number, number][] = JSON.parse(rn1.geometry!).coordinates

  it('parses the demo dataset', () => {
    expect(segments.length).toBeGreaterThan(0)
  })

  it('routes along RN1 west to east', () => {
    const route = routeOnRoads(segments, coords[0], coords[coords.length - 1])
    expect(route).not.toBeNull()
    expect(route!.coordinates.length).toBeGreaterThanOrEqual(2)
    expect(route!.lengthM).toBeGreaterThan(0)
  })

  it('never fabricates a road between disconnected points', () => {
    expect(buildRoadRoute(DEMO_ROADS, [0, 0], [0.01, 0.01], 'leg-x')).toBeNull()
  })
})

describe('buildRoadRoute', () => {
  it('accepts pre-parsed segments and gives the same result as parsing', () => {
    const rn1 = DEMO_ROADS.find((r) => r.id === 1)!
    const coords: [number, number][] = JSON.parse(rn1.geometry!).coordinates
    const from = coords[0]
    const to = coords[coords.length - 1]
    const segments = parseRoadSegments(DEMO_ROADS)
    const a = buildRoadRoute(DEMO_ROADS, from, to, 'leg-1')
    const b = buildRoadRoute(DEMO_ROADS, from, to, 'leg-1', { segments })
    expect(a).toEqual(b)
    expect(a?.method).toBe('road-graph')
  })

  it('rejects endpoints too far from any mapped road', () => {
    const rn1 = DEMO_ROADS.find((r) => r.id === 1)!
    const coords: [number, number][] = JSON.parse(rn1.geometry!).coordinates
    const from = coords[0]
    const far: [number, number] = [from[0] + 1, from[1] + 1]
    expect(buildRoadRoute(DEMO_ROADS, from, far, 'leg-2', { maxSnapM: 250 })).toBeNull()
  })
})
