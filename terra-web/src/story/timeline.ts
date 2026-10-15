// Deterministic story timeline (RFC-014).
//
// Camera state is a pure function of story time `t` — never of frame count,
// wall-clock cadence, or GPU speed. Seeking to t=18.5 always produces the
// same logical camera state, which is what makes deep links, scrubbing,
// automated tests and video export reproducible (same journey, not
// necessarily identical pixels across devices).

import type { CameraKeyframe, CameraTrack, CaptionCue, StoryStop } from './types'

export interface CameraState {
  lon: number
  lat: number
  heightM: number
  headingDeg: number
  pitchDeg: number
  rollDeg: number
}

/** Smoothstep easing — deterministic, framerate-independent. */
function smoothstep(u: number): number {
  const c = u * u * (3 - 2 * u)
  return c < 0 ? 0 : c > 1 ? 1 : c
}

function lerp(a: number, b: number, u: number): number {
  return a + (b - a) * u
}

/** Shortest-arc interpolation of headings in degrees (result in [-180, 180]). */
export function lerpHeading(a: number, b: number, u: number): number {
  let delta = ((b - a + 540) % 360) - 180
  if (delta < -180) delta += 360
  const out = a + delta * u
  return ((out + 180) % 360) - 180
}

function clamp(v: number, lo: number, hi: number): number {
  return v < lo ? lo : v > hi ? hi : v
}

/** Camera state at story time `t` (seconds). Clamps outside the keyframe range. */
export function evaluateCamera(track: CameraTrack, t: number): CameraState {
  const kfs = track.keyframes
  if (kfs.length === 0) {
    return { lon: 0, lat: 0, heightM: 1e6, headingDeg: 0, pitchDeg: -90, rollDeg: 0 }
  }
  if (kfs.length === 1 || t <= kfs[0].time_s) return stateOf(kfs[0])
  const last = kfs[kfs.length - 1]
  if (t >= last.time_s) return stateOf(last)

  // Binary search for the bracketing pair.
  let lo = 0
  let hi = kfs.length - 1
  while (hi - lo > 1) {
    const mid = (lo + hi) >> 1
    if (kfs[mid].time_s <= t) lo = mid
    else hi = mid
  }
  const a = kfs[lo]
  const b = kfs[hi]
  const span = b.time_s - a.time_s
  const u = span <= 0 ? 0 : smoothstep((t - a.time_s) / span)
  return {
    lon: lerp(a.lon, b.lon, u),
    lat: lerp(a.lat, b.lat, u),
    heightM: lerp(a.height_m, b.height_m, u),
    headingDeg: lerpHeading(a.heading_deg, b.heading_deg, u),
    pitchDeg: lerp(a.pitch_deg, b.pitch_deg, u),
    rollDeg: lerp(a.roll_deg ?? 0, b.roll_deg ?? 0, u),
  }
}

function stateOf(kf: CameraKeyframe): CameraState {
  return {
    lon: kf.lon,
    lat: kf.lat,
    heightM: kf.height_m,
    headingDeg: ((kf.heading_deg + 180) % 360) - 180,
    pitchDeg: kf.pitch_deg,
    rollDeg: kf.roll_deg ?? 0,
  }
}

/** The stop whose time is the latest one at or before `t` (null before the first). */
export function activeStop(stops: StoryStop[], t: number): StoryStop | null {
  let found: StoryStop | null = null
  for (const s of stops) {
    if (s.time_s <= t + 1e-6) found = s
    else break
  }
  return found
}

/** Caption cues visible at story time `t`. */
export function activeCaptions(captions: CaptionCue[], t: number): CaptionCue[] {
  return captions.filter((c) => t >= c.time_s && t < c.time_s + c.duration_s)
}

/** Index of the stop that a `?step=N` deep link refers to, or null. */
export function stopIndexAt(stops: StoryStop[], t: number): number {
  let idx = -1
  for (let i = 0; i < stops.length; i++) {
    if (stops[i].time_s <= t + 1e-6) idx = i
    else break
  }
  return idx
}

// ---------------------------------------------------------------------------
// Track building (creator-side helper): turns ordered stops into a camera
// track with an aerial "rise" between distant stops. Deterministic given the
// same stops. Distances are honest haversine metres.
// ---------------------------------------------------------------------------

const R_EARTH = 6_371_000

export function haversineM(a: [number, number], b: [number, number]): number {
  const toRad = Math.PI / 180
  const dLat = (b[1] - a[1]) * toRad
  const dLon = (b[0] - a[0]) * toRad
  const la1 = a[1] * toRad
  const la2 = b[1] * toRad
  const h =
    Math.sin(dLat / 2) ** 2 + Math.cos(la1) * Math.cos(la2) * Math.sin(dLon / 2) ** 2
  return 2 * R_EARTH * Math.asin(Math.min(1, Math.sqrt(h)))
}

/** Initial bearing in degrees [-180, 180] from a to b. */
export function bearingDeg(a: [number, number], b: [number, number]): number {
  const toRad = Math.PI / 180
  const dLon = (b[0] - a[0]) * toRad
  const la1 = a[1] * toRad
  const la2 = b[1] * toRad
  const y = Math.sin(dLon) * Math.cos(la2)
  const x =
    Math.cos(la1) * Math.sin(la2) - Math.sin(la1) * Math.cos(la2) * Math.cos(dLon)
  return ((Math.atan2(y, x) / toRad + 180) % 360) - 180
}

/**
 * Build a camera track from ordered stops: each stop gets a keyframe at its
 * `time_s`; between stops a mid keyframe rises proportional to distance
 * (capped) so long hops read as an explicitly aerial transition, not a
 * fake ground route.
 */
export function buildTrackFromStops(stops: StoryStop[]): CameraTrack {
  const keyframes: CameraKeyframe[] = []
  for (let i = 0; i < stops.length; i++) {
    const s = stops[i]
    const next = stops[i + 1]
    const heading =
      s.heading_deg ??
      (next ? bearingDeg([s.lon, s.lat], [next.lon, next.lat]) : 0)
    keyframes.push({
      time_s: s.time_s,
      lon: s.lon,
      lat: s.lat,
      height_m: s.height_m,
      heading_deg: heading,
      pitch_deg: s.pitch_deg ?? -28,
      roll_deg: 0,
    })
    if (next) {
      const dist = haversineM([s.lon, s.lat], [next.lon, next.lat])
      if (dist > 400) {
        const tMid = (s.time_s + next.time_s) / 2
        const rise = Math.min(2500, Math.max(s.height_m, next.height_m) + dist * 0.12)
        keyframes.push({
          time_s: tMid,
          lon: (s.lon + next.lon) / 2,
          lat: (s.lat + next.lat) / 2,
          height_m: rise,
          heading_deg: bearingDeg([s.lon, s.lat], [next.lon, next.lat]),
          pitch_deg: -45,
          roll_deg: 0,
        })
      }
    }
  }
  keyframes.sort((a, b) => a.time_s - b.time_s)
  return { keyframes }
}

/** Suggested per-leg duration (seconds) from stop distance — creator hint. */
export function suggestLegSeconds(from: [number, number], to: [number, number]): number {
  const dist = haversineM(from, to)
  return clamp(dist / 90, 4, 14)
}

export { clamp }
