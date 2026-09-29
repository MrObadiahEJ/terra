// Bundled sample dataset for the demo layer. Used whenever the off-chain API
// is unreachable or its database is empty, so the globe always has something
// to show on any device, fully offline. Shapes sit in the pilot region around
// DEFAULT_FOCUS (Soa / Yaoundé, Cameroon).

import type { OffChainParcel, RoadRow, PoiRow, GeoStats, FusionStats } from './api'
import type { LonLat } from './geo'
import { polygonAreaM2, lineLengthM } from './geo'

function parcel(
  id: string,
  name: string,
  status: string,
  ring: LonLat[],
): OffChainParcel {
  return {
    id,
    name,
    holder: 'demo1111111111111111111111111111111',
    status,
    geometry: JSON.stringify({ type: 'Polygon', coordinates: [ring] }),
    area_m2: polygonAreaM2(ring),
    created_at: '2026-09-01T08:00:00Z',
    updated_at: '2026-09-01T08:00:00Z',
  }
}

export const DEMO_PARCELS: OffChainParcel[] = [
  parcel(
    'a11e0001d50c0a9f6b2e4c7d8f3a5b1c0d2e3f4a5b6c7d8e9f0a1b2c3d4e5f60',
    'Soa Demo Block A',
    'registered',
    [
      [11.498, 3.845],
      [11.506, 3.845],
      [11.506, 3.851],
      [11.498, 3.851],
      [11.498, 3.845],
    ],
  ),
  parcel(
    'a11e0002d50c0a9f6b2e4c7d8f3a5b1c0d2e3f4a5b6c7d8e9f0a1b2c3d4e5f61',
    'Ekoa Farm Plot',
    'registered',
    [
      [11.509, 3.838],
      [11.515, 3.837],
      [11.517, 3.841],
      [11.514, 3.845],
      [11.509, 3.844],
      [11.509, 3.838],
    ],
  ),
  parcel(
    'a11e0003d50c0a9f6b2e4c7d8f3a5b1c0d2e3f4a5b6c7d8e9f0a1b2c3d4e5f62',
    'Mbankomo Test Parcel',
    'verified',
    [
      [11.484, 3.854],
      [11.491, 3.853],
      [11.493, 3.858],
      [11.487, 3.861],
      [11.484, 3.854],
    ],
  ),
  parcel(
    'a11e0004d50c0a9f6b2e4c7d8f3a5b1c0d2e3f4a5b6c7d8e9f0a1b2c3d4e5f63',
    'Nkolbisson Ridge',
    'disputed',
    [
      [11.492, 3.863],
      [11.499, 3.862],
      [11.501, 3.867],
      [11.494, 3.869],
      [11.492, 3.863],
    ],
  ),
  parcel(
    'a11e0005d50c0a9f6b2e4c7d8f3a5b1c0d2e3f4a5b6c7d8e9f0a1b2c3d4e5f64',
    'Obili Small Lot',
    'pending',
    [
      [11.4875, 3.8455],
      [11.4905, 3.8455],
      [11.4905, 3.8475],
      [11.4875, 3.8475],
      [11.4875, 3.8455],
    ],
  ),
  parcel(
    'a11e0006d50c0a9f6b2e4c7d8f3a5b1c0d2e3f4a5b6c7d8e9f0a1b2c3d4e5f65',
    'Biyem-Sud Corner',
    'registered',
    [
      [11.491, 3.837],
      [11.496, 3.837],
      [11.497, 3.842],
      [11.491, 3.843],
      [11.491, 3.837],
    ],
  ),
]

function road(
  id: number,
  name: string,
  highway: string,
  oneway: boolean,
  coords: LonLat[],
): RoadRow {
  return {
    id,
    name,
    highway,
    oneway,
    length_m: lineLengthM(coords),
    geometry: JSON.stringify({ type: 'LineString', coordinates: coords }),
    ingested_at: '2026-09-01T00:00:00Z',
  }
}

export const DEMO_ROADS: RoadRow[] = [
  road(1, 'RN1 Soa Link', 'primary', false, [
    [11.478, 3.836],
    [11.495, 3.844],
    [11.507, 3.85],
    [11.522, 3.857],
  ]),
  road(2, 'Campus Road', 'secondary', false, [
    [11.496, 3.852],
    [11.501, 3.856],
    [11.505, 3.861],
  ]),
  road(3, 'Mvog-Betsi St', 'residential', true, [
    [11.49, 3.846],
    [11.496, 3.844],
    [11.502, 3.843],
  ]),
  road(4, 'Market Access Rd', 'unclassified', false, [
    [11.504, 3.847],
    [11.509, 3.844],
    [11.513, 3.84],
  ]),
  road(5, 'Ekoa Link', 'tertiary', false, [
    [11.507, 3.845],
    [11.512, 3.848],
    [11.517, 3.851],
  ]),
]

function poi(
  id: number,
  name: string,
  category: string,
  lon: number,
  lat: number,
): PoiRow {
  return {
    id,
    name,
    category,
    kind: 'node',
    tags: { demo: true },
    geometry: JSON.stringify({ type: 'Point', coordinates: [lon, lat] }),
    ingested_at: '2026-09-01T00:00:00Z',
  }
}

export const DEMO_POIS: PoiRow[] = [
  poi(101, 'Soa Market', 'market', 11.507, 3.846),
  poi(102, 'IRIC Campus', 'university', 11.501, 3.855),
  poi(103, 'Soa Health Centre', 'clinic', 11.494, 3.85),
  poi(104, 'Banks Junction', 'crossing', 11.497, 3.843),
  poi(105, 'Ekoa Bridge', 'bridge', 11.513, 3.849),
  poi(106, "Tiku's Store", 'shop', 11.489, 3.847),
  poi(107, 'Mbankomo Depot', 'depot', 11.486, 3.857),
  poi(108, 'Water Point 3', 'water_tap', 11.493, 3.839),
]

export const DEMO_GEO_STATS: GeoStats = {
  nodes: 128400,
  roads: 942,
  road_length_km: 318.6,
  pois: 1287,
  loaded: true,
}

export const DEMO_FUSION_STATS: FusionStats = {
  roads: 942,
  pois: 1287,
  pilot_zones: 6,
  photogrammetry_assets: 3,
}
