// Spatial account mirror client (RFC-013 Phases C–E): on-chain SpatialAsset /
// GeometryVersion / ThreeDModel / SpatialSnapshot rows served by the API
// mirror (REST /spatial-accounts). Read-only; the indexer owns the tables.

import { API_BASE } from './constants'

export interface SpatialAssetMirror {
  pubkey: string
  parcel_pubkey: string
  authority: string
  dimensionality: number
  elevation_min_mm: number
  elevation_max_mm: number
  geometry_version_count: number
  latest_geometry_hash: string
  chain_created_at: string | null
  chain_updated_at: string | null
}

export interface GeometryVersionMirror {
  pubkey: string
  asset_pubkey: string
  parcel_pubkey: string
  version: number
  geometry_hash: string
  source: number
  dimension: number
  storage_reference: string
  submitted_by: string
  chain_submitted_at: string | null
  verified: boolean
  verified_by: string | null
  chain_verified_at: string | null
  evidence_manifest: string | null
  elevation_min_mm: number
  elevation_max_mm: number
  elevation_source: number
  required: number
  attest_count: number
  attestors: string[]
}

/** On-chain ThreeDModel claim (distinct from reconstruction pipeline models). */
export interface SpatialModelMirror {
  pubkey: string
  asset_pubkey: string
  parcel_pubkey: string
  model_version: number
  geometry_version: number
  model_hash: string
  storage_reference: string
  model_format: number
  submitted_by: string
  chain_submitted_at: string | null
  verified: boolean
  verified_by: string | null
  chain_verified_at: string | null
}

export interface SpatialSnapshotMirror {
  pubkey: string
  asset_pubkey: string
  parcel_pubkey: string
  snapshot_index: number
  geometry_version_count: number
  latest_geometry_hash: string
  dimensionality: number
  elevation_min_mm: number
  elevation_max_mm: number
  label: string
  created_by: string
  chain_created_at: string | null
}

export interface PointInTimeHistory {
  snapshot: SpatialSnapshotMirror
  versions: GeometryVersionMirror[]
}

async function getJson<T>(path: string): Promise<T> {
  const res = await fetch(`${API_BASE}${path}`)
  if (!res.ok) throw new Error(`spatial-accounts ${path} failed: ${res.status}`)
  return (await res.json()) as T
}

export async function listAssetsByParcel(parcelPubkey: string): Promise<SpatialAssetMirror[]> {
  return getJson<SpatialAssetMirror[]>(
    `/spatial-accounts/assets?parcel=${encodeURIComponent(parcelPubkey)}`,
  )
}

export async function listGeometryVersions(assetPubkey: string): Promise<GeometryVersionMirror[]> {
  return getJson<GeometryVersionMirror[]>(
    `/spatial-accounts/assets/${encodeURIComponent(assetPubkey)}/versions`,
  )
}

export async function listSpatialModels(assetPubkey: string): Promise<SpatialModelMirror[]> {
  return getJson<SpatialModelMirror[]>(
    `/spatial-accounts/assets/${encodeURIComponent(assetPubkey)}/models`,
  )
}

export async function listSpatialSnapshots(assetPubkey: string): Promise<SpatialSnapshotMirror[]> {
  return getJson<SpatialSnapshotMirror[]>(
    `/spatial-accounts/assets/${encodeURIComponent(assetPubkey)}/snapshots`,
  )
}

export async function pointInTimeHistory(
  assetPubkey: string,
  atUnix: number,
): Promise<PointInTimeHistory> {
  return getJson<PointInTimeHistory>(
    `/spatial-accounts/assets/${encodeURIComponent(assetPubkey)}/history?at=${atUnix}`,
  )
}

// --- on-chain code → i18n key maps (mirror terra_registry modules) ---------

export const GEOMETRY_SOURCE_KEYS = [
  'srcManual',
  'srcSurvey',
  'srcDrone',
  'srcSatellite',
  'srcLidar',
  'srcPhotogrammetry',
  'srcAi',
  'srcApiImport',
] as const

export const SPATIAL_DIMENSION_KEYS = ['dim2d', 'dim25d', 'dim3d'] as const

export const ELEVATION_SOURCE_KEYS = [
  'elevNone',
  'elevSurveySrc',
  'elevDemSrc',
  'elevLidarSrc',
  'elevPhotoSrc',
  'elevManualSrc',
] as const

export const MODEL_FORMAT_KEYS = [
  'fmtGltf',
  'fmtObj',
  'fmtTiles',
  'fmtPly',
] as const

/** i18n key for a code, falling back to the raw number for unknown values. */
export function codeKey(
  keys: readonly string[],
  code: number,
): string {
  return keys[code] ?? String(code)
}
