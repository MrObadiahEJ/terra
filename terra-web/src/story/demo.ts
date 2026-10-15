// Embedded demo stories (RFC-014). These play fully offline — no API, no
// database — so a link always demonstrates the engine even on a machine with
// no server. They are clearly labelled demo content, never presented as
// verified world facts.

import { bearingDeg, haversineM, buildTrackFromStops, suggestLegSeconds } from './timeline'
import { buildRoadRoute } from './roads'
import { DEMO_ROADS } from '../lib/demoData'
import type { CaptionCue, NavigationPath, StoryDefinition, StoryStop } from './types'

const DEMO_AREA: [number, number] = [9.515, 47.141] // atlas demo focus

function stop(
  id: string,
  label: string,
  kind: StoryStop['kind'],
  time_s: number,
  lon: number,
  lat: number,
  height_m: number,
  extra: Partial<StoryStop> = {},
): StoryStop {
  return {
    id,
    label,
    kind,
    time_s,
    lon,
    lat,
    height_m,
    pitch_deg: -28,
    ...extra,
  }
}

/**
 * A caption cue for each captioned stop, clipped so the window always stays
 * inside the story duration (schema v1 rejects overrunning captions).
 */
function captionCues(
  stops: StoryStop[],
  duration_s: number,
  lead: number,
  want: number,
): CaptionCue[] {
  const cues: CaptionCue[] = []
  for (const s of stops) {
    if (!s.caption) continue
    const time_s = s.time_s + lead
    const room = duration_s - time_s
    if (room < 0.1) continue
    cues.push({
      time_s: Number(time_s.toFixed(1)),
      duration_s: Number(Math.min(want, room).toFixed(1)),
      text: s.caption,
      stop_id: s.id,
    })
  }
  return cues
}

/** Journey: gate → aerial transition → second building arrival. */
export function demoJourney(): StoryDefinition {
  const a = stop('gate', 'Courtyard gate', 'start', 0, DEMO_AREA[0], DEMO_AREA[1], 420, {
    heading_deg: bearingDeg(DEMO_AREA, [9.5205, 47.1438]),
    caption: 'Start: property identity at the entrance.',
  })
  const b = stop(
    'rise',
    'Neighborhood overview',
    'waypoint',
    9,
    (DEMO_AREA[0] + 9.5205) / 2,
    (DEMO_AREA[1] + 47.1438) / 2,
    1100,
    { pitch_deg: -48, heading_deg: bearingDeg(DEMO_AREA, [9.5205, 47.1438]) },
  )
  const c = stop('hall', 'Rathaus approach', 'arrival', 20, 9.5205, 47.1438, 430, {
    heading_deg: bearingDeg([9.5205, 47.1438], [9.517, 47.139]),
    caption: 'Arrival: destination revealed.',
  })
  const stops = [a, b, c]
  const camera_track = buildTrackFromStops(stops)
  const leg1: [number, number][] = [
    [a.lon, a.lat],
    [b.lon, b.lat],
  ]
  const leg2: [number, number][] = [
    [b.lon, b.lat],
    [c.lon, c.lat],
  ]
  return {
    schema_version: 1,
    id: 'demo-journey',
    story_type: 'journey',
    title: 'Demo journey — courtyard to Rathaus',
    description:
      'Offline demo story. The waypoint leg is an aerial transition, not a drivable route.',
    stops,
    camera_track,
    routes: [
      {
        id: 'leg-1',
        method: 'great-circle',
        coordinates: leg1,
        length_m: haversineM([a.lon, a.lat], [b.lon, b.lat]),
        label: 'Illustrative transition (aerial)',
      },
      {
        id: 'leg-2',
        method: 'great-circle',
        coordinates: leg2,
        length_m: haversineM([b.lon, b.lat], [c.lon, c.lat]),
        label: 'Illustrative transition (aerial)',
      },
    ],
    captions: [
      { time_s: 0.5, duration_s: 4, text: 'A world you can navigate, replay and share.', stop_id: 'gate' },
      { time_s: 9.5, duration_s: 4, text: 'Rising above the neighborhood — camera track, not a road.', stop_id: 'rise' },
      { time_s: 20.5, duration_s: 3.5, text: 'Arrival. The same link replays this for everyone.', stop_id: 'hall' },
    ],
    duration_s: 24,
  }
}

/** History: v1 → v5 style progression over one parcel's geometry versions. */
export function demoHistory(): StoryDefinition {
  // References the seeded spatial-accounts mirror demo asset (RFC-013 Phase E).
  const asset = 'SpAtDemo1111111111111111111111111111111111'
  const stops: StoryStop[] = [
    stop('v1', 'V1 — initial geometry', 'version', 0, DEMO_AREA[0], DEMO_AREA[1], 650, {
      asset_pubkey: asset,
      geometry_version: 0,
      heading_deg: 20,
      caption: 'V1: SURVEY claim, D2 — an observation, not yet a verified fact.',
      provenance: {
        source: 1,
        dimension: 0,
        elevation_source: 0,
        verified: false,
        attest_count: 0,
      },
    }),
    stop('v3', 'V3 — drone observation', 'version', 8, DEMO_AREA[0] + 0.002, DEMO_AREA[1] + 0.001, 800, {
      asset_pubkey: asset,
      geometry_version: 1,
      heading_deg: 80,
      caption: 'Later versions add evidence. Unverified ≠ established.',
      provenance: {
        source: 2,
        dimension: 1,
        elevation_source: 3,
        verified: true,
        verified_by: 'DemoValidator111111111111111111111111111111',
        attest_count: 2,
        attestors: ['DemoValidator111111111111111111111111111111'],
        evidence_manifest: 'demo-manifest-g1',
        geometry_hash: 'demo-g1',
      },
    }),
    stop('v5', 'V5 — LiDAR update', 'version', 16, DEMO_AREA[0] + 0.001, DEMO_AREA[1] + 0.002, 700, {
      asset_pubkey: asset,
      geometry_version: 2,
      heading_deg: 150,
      caption: 'Provenance and validation state travel with every version.',
      provenance: {
        source: 4,
        dimension: 2,
        elevation_source: 3,
        verified: true,
        verified_by: 'DemoValidator111111111111111111111111111111',
        attest_count: 3,
        geometry_hash: 'demo-g2',
      },
    }),
  ]
  return {
    schema_version: 1,
    id: 'demo-history',
    story_type: 'history',
    title: 'Demo history — geometry versions of one parcel',
    description:
      'Offline demo story over seeded mirror data, with a read-only provenance snapshot per version. Geometry history ≠ ownership history.',
    stops,
    camera_track: buildTrackFromStops(stops),
    routes: [],
    captions: captionCues(stops, 20, 0.4, 5),
    duration_s: 20,
  }
}

/** One honest illustrative leg when the road graph cannot carry it. */
function greatCircleLeg(
  id: string,
  from: [number, number],
  to: [number, number],
): NavigationPath {
  return {
    id,
    method: 'great-circle',
    coordinates: [from, to],
    length_m: haversineM(from, to),
    label: 'Illustrative transition (aerial)',
  }
}

/**
 * Journey along mapped roads: stops sit on vertices of the RN1 Soa Link in
 * the bundled demo road dataset, and each leg is routed on the road graph
 * (`method: 'road-graph'`) with a great-circle fallback if routing fails.
 * Offline and deterministic — no API needed.
 */
export function demoRoadJourney(): StoryDefinition {
  const rn1 = DEMO_ROADS.find((r) => r.id === 1)
  const coords: [number, number][] = rn1?.geometry
    ? (JSON.parse(rn1.geometry).coordinates as [number, number][])
    : []
  const west = coords[0] ?? [11.478, 3.836]
  const mid = coords[Math.floor(coords.length / 2)] ?? [11.507, 3.85]
  const east = coords[coords.length - 1] ?? [11.522, 3.857]

  const leg1S = suggestLegSeconds(west, mid)
  const leg2S = suggestLegSeconds(mid, east)
  const clock1 = Number(leg1S.toFixed(1))
  const clock2 = Number((leg1S + leg2S).toFixed(1))

  const stops: StoryStop[] = [
    stop('rn1-west', 'RN1 — Soa west', 'start', 0, west[0], west[1], 380, {
      heading_deg: bearingDeg(west, mid),
      caption: 'Following mapped roads, not a straight line across the map.',
    }),
    stop('rn1-mid', 'RN1 — market junction', 'waypoint', clock1, mid[0], mid[1], 400, {
      heading_deg: bearingDeg(mid, east),
    }),
    stop('rn1-east', 'RN1 — Soa east', 'arrival', clock2, east[0], east[1], 420, {
      heading_deg: bearingDeg(mid, east),
      caption: 'Arrival. Every leg is labelled with the method that built it.',
    }),
  ]

  const routes: NavigationPath[] = [
    buildRoadRoute(DEMO_ROADS, west, mid, 'rn1-leg-1') ?? greatCircleLeg('rn1-leg-1', west, mid),
    buildRoadRoute(DEMO_ROADS, mid, east, 'rn1-leg-2') ?? greatCircleLeg('rn1-leg-2', mid, east),
  ]
  const duration_s = Number((clock2 + 3).toFixed(1))

  return {
    schema_version: 1,
    id: 'demo-road-journey',
    story_type: 'journey',
    title: 'Demo journey — RN1 Soa Link on the road graph',
    description:
      'Offline demo. Stops sit on the bundled road dataset and each leg is routed on the mapped network; disconnected gaps fall back to a dashed great-circle transition.',
    stops,
    camera_track: buildTrackFromStops(stops),
    routes,
    captions: captionCues(stops, duration_s, 0.4, 4),
    duration_s,
  }
}

export const DEMO_STORIES: Record<string, () => StoryDefinition> = {
  'demo-journey': demoJourney,
  'demo-history': demoHistory,
  'demo-road-journey': demoRoadJourney,
}

export function isDemoStoryId(id: string): boolean {
  return id in DEMO_STORIES
}
