import type { TranslationKey } from './locale'

export interface LodLevel {
  id: number
  nameKey: TranslationKey
  revealsKey: TranslationKey
}

export const LOD_LEVELS: LodLevel[] = [
  { id: 0, nameKey: 'lodPlanet', revealsKey: 'lodRevealPlanet' },
  { id: 1, nameKey: 'lodCountry', revealsKey: 'lodRevealCountry' },
  { id: 2, nameKey: 'lodRegion', revealsKey: 'lodRevealRegion' },
  { id: 3, nameKey: 'lodCity', revealsKey: 'lodRevealCity' },
  { id: 4, nameKey: 'lodNeighborhood', revealsKey: 'lodRevealNeighborhood' },
  { id: 5, nameKey: 'lodParcel', revealsKey: 'lodRevealParcel' },
  { id: 6, nameKey: 'lodAsset', revealsKey: 'lodRevealAsset' },
  { id: 7, nameKey: 'lodEvidence', revealsKey: 'lodRevealEvidence' },
]

const HEIGHT_BREAKS_M = [5_000_000, 800_000, 150_000, 30_000, 3_000, 500, 80]

export const LOD_TARGET_HEIGHT: number[] = [
  10_000_000,
  1_500_000,
  400_000,
  60_000,
  12_000,
  1_500,
  250,
  40,
]

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
