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

/** Outer ring of a building footprint, or null when the geometry is unusable. */
export function buildingRing(footprint: OsmBuildingFootprint): LonLat[] | null {
  const ring = footprint.geometry?.coordinates?.[0]
  if (!ring || ring.length < 4) return null
  return ring.map(([lon, lat]) => [lon, lat] as LonLat)
}
