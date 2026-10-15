// Deterministic road-graph navigation (RFC-014 Phase 02).
//
// The router is a pure function of the road dataset and the two endpoints:
// no randomness, no clock, no network. It never invents a road where none is
// mapped — when the endpoints sit on different connected components (or too
// far from any mapped road) it returns null and callers must fall back to an
// honest great-circle leg, labelled as such.
//
// Nodes are the polyline vertices shared between ways, matched to ~1 m so
// OSM ways that reference the same node merge into one junction. Edge weights
// are honest haversine metres. `oneway` edges are directed: a route never
// travels the wrong way down a one-way street.

import type { NavigationPath } from './types'
import type { RoadRow } from '../lib/api'
import { clamp, haversineM } from './timeline'

export interface RoadSegment {
  roadId: number
  name: string | null
  highway: string
  oneway: boolean
  a: [number, number]
  b: [number, number]
  lengthM: number
}

export interface SnappedPoint {
  lon: number
  lat: number
  /** Distance from the query point to the mapped road, metres. */
  distanceM: number
  segment: RoadSegment
  /** Fraction along the segment (a=0, b=1) of the projection. */
  t: number
}

export interface RoadRoute {
  coordinates: [number, number][]
  lengthM: number
  roadNames: string[]
}

/** ~1 m grid: OSM ways sharing a node carry identical coordinates. */
const NODE_PRECISION = 1e5

function nodeKey(lon: number, lat: number): string {
  return `${Math.round(lon * NODE_PRECISION)},${Math.round(lat * NODE_PRECISION)}`
}

function projectOnSegment(
  p: [number, number],
  a: [number, number],
  b: [number, number],
): { lon: number; lat: number; t: number; distanceM: number } {
  const dx = b[0] - a[0]
  const dy = b[1] - a[1]
  const denom = dx * dx + dy * dy
  const t = denom === 0 ? 0 : clamp(((p[0] - a[0]) * dx + (p[1] - a[1]) * dy) / denom, 0, 1)
  const lon = a[0] + dx * t
  const lat = a[1] + dy * t
  return { lon, lat, t, distanceM: haversineM(p, [lon, lat]) }
}

/** Flatten road rows into routable segments, skipping anything unusable. */
export function parseRoadSegments(roads: RoadRow[]): RoadSegment[] {
  const out: RoadSegment[] = []
  for (const road of roads) {
    if (!road.geometry) continue
    let coords: [number, number][]
    try {
      const g = JSON.parse(road.geometry) as { type?: string; coordinates?: unknown }
      if (g.type !== 'LineString' || !Array.isArray(g.coordinates)) continue
      coords = (g.coordinates as unknown[])
        .filter(
          (c): c is [number, number] =>
            Array.isArray(c) && c.length >= 2 && Number.isFinite(c[0]) && Number.isFinite(c[1]),
        )
        .map((c) => [c[0], c[1]])
    } catch {
      continue
    }
    for (let i = 1; i < coords.length; i++) {
      const a = coords[i - 1]
      const b = coords[i]
      const lengthM = haversineM(a, b)
      if (lengthM <= 0) continue
      out.push({
        roadId: road.id,
        name: road.name,
        highway: road.highway,
        oneway: road.oneway,
        a,
        b,
        lengthM,
      })
    }
  }
  return out
}

/** Nearest mapped road to a point, or null when the dataset is empty. */
export function snapToRoadNetwork(
  segments: RoadSegment[],
  p: [number, number],
): SnappedPoint | null {
  let best: SnappedPoint | null = null
  for (const segment of segments) {
    const pr = projectOnSegment(p, segment.a, segment.b)
    if (
      best === null ||
      pr.distanceM < best.distanceM - 1e-9 ||
      (Math.abs(pr.distanceM - best.distanceM) <= 1e-9 && segment.roadId < best.segment.roadId)
    ) {
      best = {
        lon: pr.lon,
        lat: pr.lat,
        t: pr.t,
        distanceM: pr.distanceM,
        segment,
      }
    }
  }
  return best
}

interface GraphEdge {
  to: string
  weight: number
  name: string | null
}

export interface RoadGraph {
  coord: Map<string, [number, number]>
  adj: Map<string, GraphEdge[]>
}

export function buildRoadGraph(segments: RoadSegment[]): RoadGraph {
  const coord = new Map<string, [number, number]>()
  const adj = new Map<string, GraphEdge[]>()
  const addNode = (lon: number, lat: number): string => {
    const key = nodeKey(lon, lat)
    if (!coord.has(key)) {
      coord.set(key, [lon, lat])
      adj.set(key, [])
    }
    return key
  }
  const addEdge = (from: string, to: string, weight: number, name: string | null) => {
    if (from === to) return
    adj.get(from)!.push({ to, weight, name })
  }
  for (const seg of segments) {
    const ka = addNode(seg.a[0], seg.a[1])
    const kb = addNode(seg.b[0], seg.b[1])
    addEdge(ka, kb, seg.lengthM, seg.name)
    if (!seg.oneway) addEdge(kb, ka, seg.lengthM, seg.name)
  }
  for (const list of adj.values()) {
    list.sort((x, y) => (x.to < y.to ? -1 : x.to > y.to ? 1 : 0))
  }
  return { coord, adj }
}

/** Minimal binary min-heap keyed by distance. */
class MinHeap {
  private keys: string[] = []
  private dists: number[] = []

  get size(): number {
    return this.keys.length
  }

  push(key: string, d: number): void {
    this.keys.push(key)
    this.dists.push(d)
    let i = this.keys.length - 1
    while (i > 0) {
      const parent = (i - 1) >> 1
      if (this.dists[parent] <= this.dists[i]) break
      this.swap(i, parent)
      i = parent
    }
  }

  pop(): { key: string; dist: number } | null {
    if (this.keys.length === 0) return null
    const key = this.keys[0]
    const dist = this.dists[0]
    const lastKey = this.keys.pop()!
    const lastDist = this.dists.pop()!
    if (this.keys.length > 0) {
      this.keys[0] = lastKey
      this.dists[0] = lastDist
      let i = 0
      for (;;) {
        const left = 2 * i + 1
        const right = 2 * i + 2
        let smallest = i
        if (left < this.keys.length && this.dists[left] < this.dists[smallest]) smallest = left
        if (right < this.keys.length && this.dists[right] < this.dists[smallest]) smallest = right
        if (smallest === i) break
        this.swap(i, smallest)
        i = smallest
      }
    }
    return { key, dist }
  }

  private swap(i: number, j: number): void {
    ;[this.keys[i], this.keys[j]] = [this.keys[j], this.keys[i]]
    ;[this.dists[i], this.dists[j]] = [this.dists[j], this.dists[i]]
  }
}

const VIRTUAL_START = '\u0000START'
const VIRTUAL_END = '\u0000END'

/** Directed distance along a segment from a projection to one of its ends. */
function alongToEnd(segment: RoadSegment, t: number, end: 'a' | 'b'): number {
  return Math.abs((end === 'b' ? 1 : 0) - t) * segment.lengthM
}

/**
 * Route between two points along the road graph. Returns null when no legal
 * path exists (disconnected components, or a one-way that cannot be entered
 * or exited at the chosen projection).
 */
export function routeOnRoads(
  segments: RoadSegment[],
  from: [number, number],
  to: [number, number],
): RoadRoute | null {
  if (segments.length === 0) return null
  const sFrom = snapToRoadNetwork(segments, from)
  const sTo = snapToRoadNetwork(segments, to)
  if (!sFrom || !sTo) return null
  return routeOnSnapped(segments, sFrom, sTo)
}

/** Core router for two endpoints already snapped onto the network. */
function routeOnSnapped(
  segments: RoadSegment[],
  sFrom: SnappedPoint,
  sTo: SnappedPoint,
): RoadRoute | null {
  const graph = buildRoadGraph(segments)
  const adj = new Map<string, GraphEdge[]>(graph.adj)
  const startKey = nodeKey(sFrom.segment.a[0], sFrom.segment.a[1])
  const endKey = nodeKey(sTo.segment.a[0], sTo.segment.a[1])
  const startKeyB = nodeKey(sFrom.segment.b[0], sFrom.segment.b[1])
  const endKeyB = nodeKey(sTo.segment.b[0], sTo.segment.b[1])

  const edgesFrom = (k: string): GraphEdge[] => {
    let list = adj.get(k)
    if (!list) {
      list = []
      adj.set(k, list)
    }
    return list
  }

  // Leaving the start projection: forward (towards b) always; backward
  // (towards a) only on a two-way street.
  edgesFrom(VIRTUAL_START).push({
    to: startKeyB,
    weight: alongToEnd(sFrom.segment, sFrom.t, 'b'),
    name: null,
  })
  if (!sFrom.segment.oneway) {
    edgesFrom(VIRTUAL_START).push({
      to: startKey,
      weight: alongToEnd(sFrom.segment, sFrom.t, 'a'),
      name: null,
    })
  }

  // Arriving at the destination projection: approach forward from a always;
  // approach backward from b only on a two-way street.
  edgesFrom(endKey).push({
    to: VIRTUAL_END,
    weight: alongToEnd(sTo.segment, sTo.t, 'a'),
    name: null,
  })
  if (!sTo.segment.oneway) {
    edgesFrom(endKeyB).push({
      to: VIRTUAL_END,
      weight: alongToEnd(sTo.segment, sTo.t, 'b'),
      name: null,
    })
  }

  // Same-segment shortcut when the movement is legal.
  let sameSegDirect: number | null = null
  if (sFrom.segment === sTo.segment) {
    if (!sFrom.segment.oneway) {
      sameSegDirect = Math.abs(sTo.t - sFrom.t) * sFrom.segment.lengthM
    } else if (sTo.t >= sFrom.t) {
      sameSegDirect = (sTo.t - sFrom.t) * sFrom.segment.lengthM
    }
  }
  if (sameSegDirect !== null) {
    edgesFrom(VIRTUAL_START).push({ to: VIRTUAL_END, weight: sameSegDirect, name: null })
  }

  const dist = new Map<string, number>([[VIRTUAL_START, 0]])
  const prev = new Map<string, { node: string; name: string | null }>()
  const heap = new MinHeap()
  heap.push(VIRTUAL_START, 0)

  while (heap.size > 0) {
    const { key, dist: d } = heap.pop()!
    if (d > (dist.get(key) ?? Number.POSITIVE_INFINITY) + 1e-9) continue
    if (key === VIRTUAL_END) break
    for (const edge of edgesFrom(key)) {
      const nd = d + edge.weight
      if (nd < (dist.get(edge.to) ?? Number.POSITIVE_INFINITY) - 1e-9) {
        dist.set(edge.to, nd)
        prev.set(edge.to, { node: key, name: edge.name })
        heap.push(edge.to, nd)
      }
    }
  }

  if (!dist.has(VIRTUAL_END)) return null

  // Reconstruct: walk back from the virtual end to the virtual start.
  const nodePath: string[] = []
  const names: string[] = []
  let cursor = VIRTUAL_END
  while (cursor !== VIRTUAL_START) {
    const step = prev.get(cursor)
    if (!step) return null
    if (step.name && !names.includes(step.name)) names.push(step.name)
    if (cursor !== VIRTUAL_END && cursor !== VIRTUAL_START) nodePath.push(cursor)
    cursor = step.node
  }
  nodePath.reverse()

  const coordinates: [number, number][] = [[sFrom.lon, sFrom.lat]]
  for (const k of nodePath) {
    const c = graph.coord.get(k)
    if (c) coordinates.push([c[0], c[1]])
  }
  coordinates.push([sTo.lon, sTo.lat])

  return {
    coordinates: dedupeConsecutive(coordinates),
    lengthM: dist.get(VIRTUAL_END) ?? 0,
    roadNames: names,
  }
}

function dedupeConsecutive(coords: [number, number][]): [number, number][] {
  const out: [number, number][] = []
  for (const c of coords) {
    const last = out[out.length - 1]
    if (!last || Math.abs(last[0] - c[0]) > 1e-9 || Math.abs(last[1] - c[1]) > 1e-9) out.push(c)
  }
  return out
}

export interface RoadRouteOptions {
  /** Reject when an endpoint is farther than this from any mapped road. */
  maxSnapM?: number
  /** Pre-parsed segments, to avoid re-parsing the dataset for every leg. */
  segments?: RoadSegment[]
}

/**
 * Build a `road-graph` navigation path between two points, or null when the
 * mapped network cannot carry it. The caller decides the fallback; this
 * function never returns a fake road.
 */
export function buildRoadRoute(
  roads: RoadRow[],
  from: [number, number],
  to: [number, number],
  id: string,
  opts: RoadRouteOptions = {},
): NavigationPath | null {
  const maxSnapM = opts.maxSnapM ?? 250
  const segments = opts.segments ?? parseRoadSegments(roads)
  const sFrom = snapToRoadNetwork(segments, from)
  const sTo = snapToRoadNetwork(segments, to)
  if (!sFrom || !sTo) return null
  if (sFrom.distanceM > maxSnapM || sTo.distanceM > maxSnapM) return null
  const route = routeOnSnapped(segments, sFrom, sTo)
  if (!route) return null
  return {
    id,
    method: 'road-graph',
    coordinates: route.coordinates,
    length_m: Math.round(route.lengthM),
    label: route.roadNames.length > 0 ? route.roadNames.join(' · ') : undefined,
  }
}
