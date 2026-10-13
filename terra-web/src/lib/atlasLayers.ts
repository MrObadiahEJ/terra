/** Persisted visibility of the atlas overlays (Google Maps style "Map details"). */
import type { OsmBuildingFootprint } from './api'
import type { LonLat } from './geo'

export interface AtlasLayers {
  parcels: boolean
  roads: boolean
  buildings: boolean
  pois: boolean
  labels: boolean
}

export type AtlasLayerKey = keyof AtlasLayers

export const DEFAULT_ATLAS_LAYERS: AtlasLayers = {
  parcels: true,
  roads: false,
  buildings: false,
  pois: false,
  labels: true,
}

const LAYERS_KEY = 'terra-atlas-layers'
const PLANNED_ROADS_KEY = 'terra-atlas-planned-roads'

export function loadAtlasLayers(): AtlasLayers {
  try {
    const raw = localStorage.getItem(LAYERS_KEY)
    if (!raw) return { ...DEFAULT_ATLAS_LAYERS }
    const parsed = JSON.parse(raw) as Partial<AtlasLayers>
    return { ...DEFAULT_ATLAS_LAYERS, ...parsed }
  } catch {
    return { ...DEFAULT_ATLAS_LAYERS }
  }
}

export function saveAtlasLayers(layers: AtlasLayers): void {
  localStorage.setItem(LAYERS_KEY, JSON.stringify(layers))
}

/** A road alignment sketched on the atlas (open polyline, not a closed ring). */
export interface PlannedRoad {
  id: string
  name: string
  vertices: { lon: number; lat: number }[]
  lengthM: number
}

export function loadPlannedRoads(): PlannedRoad[] {
  try {
    const raw = localStorage.getItem(PLANNED_ROADS_KEY)
    if (!raw) return []
    const parsed = JSON.parse(raw) as unknown
    if (!Array.isArray(parsed)) return []
    return parsed.filter(
      (road): road is PlannedRoad =>
        road != null &&
        typeof road === 'object' &&
        typeof (road as PlannedRoad).id === 'string' &&
        Array.isArray((road as PlannedRoad).vertices) &&
        (road as PlannedRoad).vertices.length >= 2,
    )
  } catch {
    return []
  }
}

export function savePlannedRoads(roads: PlannedRoad[]): void {
  localStorage.setItem(PLANNED_ROADS_KEY, JSON.stringify(roads))
}

/** Google-style road palette: motorways orange, primaries yellow, rest pale. */
export function roadStyle(highway: string): { color: string; width: number } {
  const cls = highway.endsWith('_link') ? highway.slice(0, -5) : highway
  if (cls === 'motorway' || cls === 'trunk') return { color: '#f8a25c', width: 3.5 }
  if (cls === 'primary') return { color: '#f7d774', width: 3 }
  if (cls === 'secondary' || cls === 'tertiary') return { color: '#ffffff', width: 2.4 }
  return { color: '#e8e4da', width: 1.6 }
}

/** Outer ring of a building footprint, or null when the geometry is unusable */
export function buildingRing(footprint: OsmBuildingFootprint): LonLat[] | null {
  const ring = footprint.geometry?.coordinates?.[0]
  if (!ring || ring.length < 4) return null
  return ring.map(([lon, lat]) => [lon, lat] as LonLat)
}

/** Per-kind fallback heights (metres) when OSM mapped neither height nor levels. */
const KIND_HEIGHT_M: Record<string, number> = {
  house: 6,
  detached: 6,
  hut: 4,
  shed: 4,
  garage: 3.5,
  residential: 8,
  apartments: 14,
  dormitory: 14,
  commercial: 12,
  retail: 8,
  shop: 8,
  office: 16,
  industrial: 10,
  warehouse: 10,
  hotel: 18,
  hospital: 16,
  school: 10,
  university: 14,
  church: 15,
  chapel: 10,
  cathedral: 20,
  civic: 12,
  public: 12,
  government: 12,
  train_station: 12,
}

const DEFAULT_HEIGHT_M = 7
const METRES_PER_LEVEL = 3.2

/**
 * Best-effort extrusion height: mapped height > levels×3.2 > kind fallback.
 * OSM coverage in pilot zones is mostly untagged, so the fallback does the work.
 */
export function buildingHeightM(footprint: OsmBuildingFootprint): number {
  if (footprint.height_m != null && footprint.height_m > 0) return footprint.height_m
  if (footprint.levels != null && footprint.levels > 0) {
    return footprint.levels * METRES_PER_LEVEL + 1.5
  }
  return KIND_HEIGHT_M[footprint.building] ?? DEFAULT_HEIGHT_M
}

/** Approximate footprint area in m² (shoelace on a local equirect projection). */
export function ringAreaM2(ring: LonLat[]): number {
  if (ring.length < 4) return 0
  const lat0 = ring.reduce((sum, [, lat]) => sum + lat, 0) / ring.length
  const mPerLat = 110_540
  const mPerLon = Math.max(1, 111_320 * Math.cos((lat0 * Math.PI) / 180))
  let area = 0
  for (let i = 0; i < ring.length - 1; i += 1) {
    const [x1, y1] = ring[i]
    const [x2, y2] = ring[i + 1]
    area += (x1 * mPerLon) * (y2 * mPerLat) - (x2 * mPerLon) * (y1 * mPerLat)
  }
  return Math.abs(area) / 2
}

/**
 * Tier-aware building selection: largest footprints survive the cap, and
 * medium tier drops sheds/garages so the budget goes to real structures.
 */
export function selectBuildingsForTier(
  footprints: OsmBuildingFootprint[],
  cap: number,
  tier: 'low' | 'medium' | 'high' | 'ultra',
): OsmBuildingFootprint[] {
  if (cap <= 0 || footprints.length === 0) return []
  const scored = footprints
    .map((footprint) => {
      const ring = buildingRing(footprint)
      return { footprint, area: ring ? ringAreaM2(ring) : 0 }
    })
    .filter(({ footprint, area }) => {
      if (tier !== 'medium') return true
      return !['garage', 'hut', 'shed', 'roof'].includes(footprint.building) && area >= 12
    })
  scored.sort((a, b) => b.area - a.area)
  return scored.slice(0, cap).map(({ footprint }) => footprint)
}
