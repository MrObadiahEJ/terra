import { describe, it, expect } from 'vitest'
import {
  evaluateCamera,
  activeStop,
  activeCaptions,
  stopIndexAt,
  buildTrackFromStops,
  haversineM,
  bearingDeg,
  lerpHeading,
  suggestLegSeconds,
} from './timeline'
import type { CameraTrack, CaptionCue, StoryStop } from './types'

const kf = (time_s: number, lon: number, lat: number) => ({
  time_s,
  lon,
  lat,
  height_m: 400,
  heading_deg: 0,
  pitch_deg: -30,
  roll_deg: 0,
})

const track: CameraTrack = {
  keyframes: [kf(0, 11.4, 3.8), kf(10, 11.6, 3.9), kf(20, 11.8, 4.0)],
}

const stop = (id: string, time_s: number, kind: StoryStop['kind'] = 'waypoint'): StoryStop => ({
  id,
  label: id,
  kind,
  time_s,
  lon: 11.5,
  lat: 3.85,
  height_m: 400,
})

describe('evaluateCamera', () => {
  it('is a pure function of time', () => {
    for (const t of [0, 3.3, 10, 17.7, 20]) {
      expect(evaluateCamera(track, t)).toEqual(evaluateCamera(track, t))
    }
  })

  it('returns a finite in-range state at every sample', () => {
    for (let t = 0; t <= 20; t += 0.25) {
      const c = evaluateCamera(track, t)
      expect(Number.isFinite(c.lon) && c.lon >= -180 && c.lon <= 180).toBe(true)
      expect(Number.isFinite(c.lat) && c.lat >= -90 && c.lat <= 90).toBe(true)
      expect(Number.isFinite(c.heightM)).toBe(true)
      expect(Number.isFinite(c.headingDeg)).toBe(true)
    }
  })

  it('clamps outside the keyframe range', () => {
    expect(evaluateCamera(track, -5)).toEqual(evaluateCamera(track, 0))
    expect(evaluateCamera(track, 1e6)).toEqual(evaluateCamera(track, 20))
  })

  it('interpolates (the camera actually moves)', () => {
    const a = evaluateCamera(track, 0)
    const b = evaluateCamera(track, 10)
    expect(b.lon).not.toBe(a.lon)
  })

  it('handles an empty track without throwing', () => {
    expect(() => evaluateCamera({ keyframes: [] }, 0)).not.toThrow()
  })
})

describe('lerpHeading', () => {
  it('takes the shortest arc across the ±180 seam', () => {
    // 170 -> -170 should pass through 180, i.e. increase, not go the long way.
    const mid = lerpHeading(170, -170, 0.5)
    expect(Math.abs(mid)).toBe(180)
  })
})

describe('activeStop / stopIndexAt', () => {
  const stops = [stop('a', 0, 'start'), stop('b', 5), stop('c', 12, 'arrival')]

  it('returns the last stop at or before t', () => {
    expect(activeStop(stops, -1)).toBeNull()
    expect(activeStop(stops, 0)?.id).toBe('a')
    expect(activeStop(stops, 5)?.id).toBe('b')
    expect(activeStop(stops, 11.9)?.id).toBe('b')
    expect(activeStop(stops, 12)?.id).toBe('c')
    expect(activeStop(stops, 99)?.id).toBe('c')
  })

  it('stop index never goes backwards as t increases', () => {
    let last = -1
    for (let t = 0; t <= 12; t += 0.25) {
      const idx = stopIndexAt(stops, t)
      expect(idx).toBeGreaterThanOrEqual(last)
      last = idx
    }
  })
})

describe('activeCaptions', () => {
  const cues: CaptionCue[] = [
    { time_s: 1, duration_s: 2, text: 'one' },
    { time_s: 4, duration_s: 3, text: 'two' },
  ]
  it('returns cues whose window contains t', () => {
    expect(activeCaptions(cues, 0.5)).toEqual([])
    expect(activeCaptions(cues, 1).map((c) => c.text)).toEqual(['one'])
    expect(activeCaptions(cues, 3).map((c) => c.text)).toEqual([])
    expect(activeCaptions(cues, 4).map((c) => c.text)).toEqual(['two'])
    expect(activeCaptions(cues, 7)).toEqual([])
  })
})

describe('buildTrackFromStops', () => {
  it('is deterministic with strictly increasing keyframes', () => {
    const stops = [stop('a', 0, 'start'), stop('b', 9), stop('c', 20, 'arrival')]
    const t1 = buildTrackFromStops(stops)
    const t2 = buildTrackFromStops(stops)
    expect(t1).toEqual(t2)
    for (let i = 1; i < t1.keyframes.length; i++) {
      expect(t1.keyframes[i].time_s).toBeGreaterThan(t1.keyframes[i - 1].time_s)
    }
  })

  it('inserts an aerial rise between distant stops', () => {
    const near: StoryStop = { ...stop('a', 0, 'start'), lon: 11.5, lat: 3.85 }
    const far: StoryStop = { ...stop('b', 20, 'arrival'), lon: 11.6, lat: 3.9 }
    const t = buildTrackFromStops([near, far])
    // 2 stops + 1 mid keyframe
    expect(t.keyframes.length).toBe(3)
    expect(t.keyframes[1].height_m).toBeGreaterThan(near.height_m)
  })
})

describe('geometry helpers', () => {
  it('haversine is zero for identical points', () => {
    expect(haversineM([0, 0], [0, 0])).toBeLessThan(1e-6)
  })
  it('haversine matches a known short hop scale', () => {
    const d = haversineM([9.515, 47.141], [9.5205, 47.1438])
    expect(d).toBeGreaterThan(400)
    expect(d).toBeLessThan(800)
  })
  it('bearing is 0 due north and ±90 due east/west', () => {
    expect(Math.round(bearingDeg([0, 0], [0, 1]))).toBe(0)
    expect(Math.round(bearingDeg([0, 0], [1, 0]))).toBe(90)
  })
  it('suggestLegSeconds is clamped', () => {
    expect(suggestLegSeconds([0, 0], [0, 0])).toBe(4)
    expect(suggestLegSeconds([0, 0], [10, 10])).toBe(14)
  })
})
