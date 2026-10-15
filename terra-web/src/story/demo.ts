// Embedded demo stories (RFC-014). These play fully offline — no API, no
// database — so a link always demonstrates the engine even on a machine with
// no server. They are clearly labelled demo content, never presented as
// verified world facts.

import { bearingDeg, haversineM, buildTrackFromStops } from './timeline'
import type { StoryDefinition, StoryStop } from './types'

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
    }),
    stop('v3', 'V3 — drone observation', 'version', 8, DEMO_AREA[0] + 0.002, DEMO_AREA[1] + 0.001, 800, {
      asset_pubkey: asset,
      geometry_version: 1,
      heading_deg: 80,
      caption: 'Later versions add evidence. Unverified ≠ established.',
    }),
    stop('v5', 'V5 — LiDAR update', 'version', 16, DEMO_AREA[0] + 0.001, DEMO_AREA[1] + 0.002, 700, {
      asset_pubkey: asset,
      geometry_version: 2,
      heading_deg: 150,
      caption: 'Provenance and validation state travel with every version.',
    }),
  ]
  return {
    schema_version: 1,
    id: 'demo-history',
    story_type: 'history',
    title: 'Demo history — geometry versions of one parcel',
    description:
      'Offline demo story over seeded mirror data. Geometry history ≠ ownership history.',
    stops,
    camera_track: buildTrackFromStops(stops),
    routes: [],
    captions: stops
      .filter((s) => s.caption)
      .map((s) => ({ time_s: s.time_s + 0.4, duration_s: 5, text: s.caption as string, stop_id: s.id })),
    duration_s: 20,
  }
}

export const DEMO_STORIES: Record<string, () => StoryDefinition> = {
  'demo-journey': demoJourney,
  'demo-history': demoHistory,
}

export function isDemoStoryId(id: string): boolean {
  return id in DEMO_STORIES
}
