import { describe, it, expect } from 'vitest'
import { DEMO_STORIES, isDemoStoryId } from './demo'
import type { StoryDefinition } from './types'

// Mirrors the schema v1 rules in terra-core api/src/routes/stories.rs
// (validate_definition). A demo that would fail to save is a bug: demos are
// the reference examples of the format.
function validate(def: StoryDefinition): string | null {
  if (def.schema_version !== 1) return 'schema_version'
  if (!['journey', 'history', 'discovery'].includes(def.story_type)) return 'story_type'
  if (def.title.trim().length < 1 || def.title.length > 200) return 'title'
  if (!(def.duration_s >= 1 && def.duration_s <= 300)) return 'duration_s'
  if (def.stops.length < 2 || def.stops.length > 20) return 'stops count'
  if (def.camera_track.keyframes.length < 2 || def.camera_track.keyframes.length > 400)
    return 'keyframes count'

  const ids = new Set<string>()
  let prevStopT = -Infinity
  for (const s of def.stops) {
    if (!s.id || ids.has(s.id)) return `stop id ${s.id}`
    ids.add(s.id)
    if (!['start', 'waypoint', 'arrival', 'version'].includes(s.kind)) return `stop kind ${s.kind}`
    if (!(s.time_s >= 0 && s.time_s <= def.duration_s)) return `stop time ${s.id}`
    if (s.time_s < prevStopT - 1e-6) return `stops unordered at ${s.id}`
    prevStopT = s.time_s
  }
  let prevT = -Infinity
  for (const k of def.camera_track.keyframes) {
    if (!(k.time_s > prevT)) return 'keyframes not increasing'
    prevT = k.time_s
    if (!(k.time_s >= 0 && k.time_s <= def.duration_s)) return 'keyframe time'
  }
  for (const r of def.routes) {
    if (!['road-graph', 'great-circle', 'drawn'].includes(r.method)) return `route method ${r.id}`
    if (r.coordinates.length < 2 || r.coordinates.length > 5000) return `route coords ${r.id}`
    if (!(r.length_m >= 0)) return `route length ${r.id}`
  }
  for (const c of def.captions) {
    if (!(c.time_s >= 0 && c.time_s <= def.duration_s)) return `caption time ${c.text}`
    if (!(c.duration_s >= 0.1 && c.duration_s <= 60)) return `caption duration ${c.text}`
    if (c.time_s + c.duration_s > def.duration_s + 0.001) return `caption window ${c.text}`
    if (c.text.trim().length < 1 || c.text.length > 500) return 'caption text'
  }
  if (def.story_type === 'journey') {
    if (!def.stops.some((s) => s.kind === 'start')) return 'journey start'
    if (!def.stops.some((s) => s.kind === 'arrival')) return 'journey arrival'
  }
  if (def.story_type === 'history') {
    if (!def.stops.some((s) => s.geometry_version !== undefined || s.asset_pubkey !== undefined))
      return 'history ref'
  }
  return null
}

const ids = Object.keys(DEMO_STORIES)

describe('demo stories', () => {
  it('registers the known demos and reports them as demos', () => {
    expect(ids.sort()).toEqual(['demo-history', 'demo-journey', 'demo-road-journey'])
    for (const id of ids) expect(isDemoStoryId(id)).toBe(true)
    expect(isDemoStoryId('not-a-demo')).toBe(false)
  })

  for (const id of ids) {
    describe(id, () => {
      const def = DEMO_STORIES[id]()

      it('has a matching id and passes schema v1 validation', () => {
        expect(def.id).toBe(id)
        expect(validate(def)).toBeNull()
      })

      it('has strictly increasing camera keyframes with one per stop', () => {
        const times = def.camera_track.keyframes.map((k) => k.time_s)
        for (const s of def.stops) expect(times).toContain(s.time_s)
      })

      it('has at least two stops and a positive duration', () => {
        expect(def.stops.length).toBeGreaterThanOrEqual(2)
        expect(def.duration_s).toBeGreaterThan(0)
      })

      it('builds is deterministic', () => {
        expect(DEMO_STORIES[id]()).toEqual(def)
      })
    })
  }

  it('the road journey includes a genuine road-graph leg', () => {
    const rj = DEMO_STORIES['demo-road-journey']()
    expect(rj.routes.some((r) => r.method === 'road-graph')).toBe(true)
    for (const r of rj.routes) {
      expect(r.coordinates.length).toBeGreaterThanOrEqual(2)
      expect(r.length_m).toBeGreaterThan(0)
    }
  })
})
