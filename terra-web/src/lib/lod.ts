export interface LodLevel {
  id: number
  name: string
  reveals: string
}

export const LOD_LEVELS: LodLevel[] = [
  { id: 0, name: 'Planet', reveals: 'Earth and activity regions' },
  { id: 1, name: 'Country', reveals: 'National context and data density' },
  { id: 2, name: 'Region', reveals: 'Cities and pilot areas' },
  { id: 3, name: 'City', reveals: 'Roads and mapped areas' },
  { id: 4, name: 'Neighborhood', reveals: 'Parcel outlines and labels' },
  { id: 5, name: 'Parcel', reveals: 'Boundary, ID, area and state' },
  { id: 6, name: 'Asset', reveals: 'Structure, rights and activity' },
  { id: 7, name: 'Evidence', reveals: 'Observations and geometry versions' },
]

const HEIGHT_BREAKS_M = [5_000_000, 800_000, 150_000, 30_000, 3_000, 500, 80]

export function lodFromHeight(heightM: number): number {
  for (let i = 0; i < HEIGHT_BREAKS_M.length; i++) {
    if (heightM > HEIGHT_BREAKS_M[i]) return i
  }
  return HEIGHT_BREAKS_M.length
}

const ZOOM_TO_LOD = [0, 0, 1, 1, 1, 2, 2, 3, 3, 3, 4, 4, 4, 5, 5, 5, 6, 6, 7, 7, 7]

export function lodFromLeafletZoom(zoom: number): number {
  const z = Math.max(0, Math.min(Math.round(zoom), ZOOM_TO_LOD.length - 1))
  return ZOOM_TO_LOD[z]
}
