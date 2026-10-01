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

// Compact illustrative lot envelopes for offline UI. Not OSM features or
// cadastral boundaries; the live demo prefers actual mapped OSM footprints.
const DEMO_LOT_SEEDS: { id: string; name: string; dx: number; dy: number; w: number; h: number }[] = [
  { id: 'a11e0001d50c0a9f6b2e4c7d8f3a5b1c0d2e3f4a5b6c7d8e9f0a1b2c3d4e5f60', name: 'Soa Sample Lot 01', dx: 0, dy: 0, w: 0.00026, h: 0.00031 },
  { id: 'a11e0002d50c0a9f6b2e4c7d8f3a5b1c0d2e3f4a5b6c7d8e9f0a1b2c3d4e5f61', name: 'Soa Sample Lot 02', dx: 0.00034, dy: 0.00003, w: 0.00021, h: 0.00029 },
  { id: 'a11e0003d50c0a9f6b2e4c7d8f3a5b1c0d2e3f4a5b6c7d8e9f0a1b2c3d4e5f62', name: 'Soa Sample Lot 03', dx: -0.00031, dy: 0.00015, w: 0.00022, h: 0.00025 },
  { id: 'a11e0004d50c0a9f6b2e4c7d8f3a5b1c0d2e3f4a5b6c7d8e9f0a1b2c3d4e5f63', name: 'Soa Sample Lot 04', dx: 0.00018, dy: 0.00038, w: 0.00029, h: 0.00024 },
  { id: 'a11e0005d50c0a9f6b2e4c7d8f3a5b1c0d2e3f4a5b6c7d8e9f0a1b2c3d4e5f64', name: 'Soa Sample Lot 05', dx: -0.00016, dy: -0.00034, w: 0.0002, h: 0.00026 },
  { id: 'a11e0006d50c0a9f6b2e4c7d8f3a5b1c0d2e3f4a5b6c7d8e9f0a1b2c3d4e5f65', name: 'Soa Sample Lot 06', dx: 0.00043, dy: -0.00028, w: 0.00024, h: 0.00023 },
  { id: 'a11e0007d50c0a9f6b2e4c7d8f3a5b1c0d2e3f4a5b6c7d8e9f0a1b2c3d4e5f66', name: 'Soa Sample Lot 07', dx: -0.00048, dy: -0.00023, w: 0.0002, h: 0.00028 },
  { id: 'a11e0008d50c0a9f6b2e4c7d8f3a5b1c0d2e3f4a5b6c7d8e9f0a1b2c3d4e5f67', name: 'Soa Sample Lot 08', dx: 0.00061, dy: 0.00012, w: 0.0002, h: 0.00025 },
  { id: 'a11e0009d50c0a9f6b2e4c7d8f3a5b1c0d2e3f4a5b6c7d8e9f0a1b2c3d4e5f68', name: 'Soa Sample Lot 09', dx: -0.00055, dy: 0.00038, w: 0.00023, h: 0.00022 },
  { id: 'a11e0010d50c0a9f6b2e4c7d8f3a5b1c0d2e3f4a5b6c7d8e9f0a1b2c3d4e5f69', name: 'Soa Sample Lot 10', dx: 0.00075, dy: -0.00016, w: 0.00018, h: 0.00027 },
  { id: 'a11e0011d50c0a9f6b2e4c7d8f3a5b1c0d2e3f4a5b6c7d8e9f0a1b2c3d4e5f6a', name: 'Soa Sample Lot 11', dx: -0.00075, dy: -0.00008, w: 0.00024, h: 0.0002 },
  { id: 'a11e0012d50c0a9f6b2e4c7d8f3a5b1c0d2e3f4a5b6c7d8e9f0a1b2c3d4e5f6b', name: 'Soa Sample Lot 12', dx: 0.00002, dy: 0.00069, w: 0.0002, h: 0.00026 },
]

export const DEMO_PARCELS: OffChainParcel[] = DEMO_LOT_SEEDS.map((lot, index) => {
  const west = 11.501 + lot.dx
  const south = 3.847 + lot.dy
  const ring: LonLat[] = [
    [west, south],
    [west + lot.w, south + lot.h * 0.08],
    [west + lot.w * 0.92, south + lot.h],
    [west + lot.w * 0.08, south + lot.h * 0.94],
    [west, south],
  ]
  return parcel(lot.id, lot.name, ['registered', 'verified', 'pending'][index % 3], ring)
})

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
