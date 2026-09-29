// Terra Lab — Geometry Vault (RFC-013) state.
// Mirrors the on-chain SpatialAsset / GeometryVersion handlers 1:1, including
// guard order and error codes; all hashes are real SHA-256 computed locally.

import { create } from 'zustand'
import { persist } from 'zustand/middleware'
import { DEFAULT_FOCUS } from './constants'
import type { LonLat } from './geo'
import { sha256Hex } from './geo'

/** terra-core spatial_asset.rs: MAX_GEOMETRY_VERSIONS */
export const MAX_GEOMETRY_VERSIONS = 64

export const SPATIAL_DIMENSIONS = [
  { code: 0, id: 'D2', label: '2D', note: 'flat cadastral polygon' },
  { code: 1, id: 'D2_5', label: '2.5D', note: 'boundary + elevation/terrain' },
  { code: 2, id: 'D3', label: '3D', note: 'buildings, floors, air rights' },
] as const

export const GEOMETRY_SOURCES = [
  { code: 0, id: 'MANUAL', label: 'Manual' },
  { code: 1, id: 'SURVEY', label: 'Survey' },
  { code: 2, id: 'DRONE_PHOTO', label: 'Drone photo' },
  { code: 3, id: 'SATELLITE', label: 'Satellite' },
  { code: 4, id: 'LIDAR', label: 'LiDAR' },
  { code: 5, id: 'PHOTOGRAMMETRY', label: 'Photogrammetry' },
  { code: 6, id: 'AI_SEGMENTATION', label: 'AI segmentation' },
  { code: 7, id: 'API_IMPORT', label: 'API import' },
] as const

export interface VaultVersion {
  version: number
  geometryHash: string // hex (64 chars) — real sha256 of the ring JSON
  source: number
  dimension: number
  storageReference: string
  submittedBy: string // demo wallet label (base58 derived in UI)
  submittedAt: string // ISO
  verified: boolean
  verifiedBy: string | null
  verifiedAt: string | null
  ring: LonLat[] // snapshot of the anchored geometry (closed ring)
}

export type LabResult = { ok: true; msg: string } | { ok: false; msg: string }

interface VaultState {
  initialized: boolean
  authority: string
  dimensionality: number
  elevationMinMm: number
  elevationMaxMm: number
  versions: VaultVersion[]
  createdAt: string | null
  updatedAt: string | null
  shapeIndex: number

  initAsset: (authority: string, dim: number, minMm: number, maxMm: number) => LabResult
  appendVersion: (
    ring: LonLat[],
    source: number,
    dimension: number,
    storageReference: string,
    submittedBy: string,
  ) => Promise<LabResult>
  verifyVersion: (version: number, validator: string) => LabResult
  nextShape: () => void
  reset: () => void
}

/**
 * Deterministic demo geometry: a 5-vertex closed ring near DEFAULT_FOCUS.
 * Seeded by index so history/undo of "New shape" stays reproducible.
 */
export function labShape(index: number): LonLat[] {
  // mulberry32 — tiny seeded PRNG, no dependencies.
  let a = (index + 1) * 0x9e3779b9
  const rand = () => {
    a |= 0
    a = (a + 0x6d2b79f5) | 0
    let t = Math.imul(a ^ (a >>> 15), 1 | a)
    t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296
  }
  const baseLon = DEFAULT_FOCUS.longitude + (rand() - 0.5) * 0.02
  const baseLat = DEFAULT_FOCUS.latitude + (rand() - 0.5) * 0.015
  const size = 0.004 + rand() * 0.004
  const wobble = size * (0.4 + rand() * 0.5)
  const corners: LonLat[] = [
    [baseLon - size, baseLat - size],
    [baseLon + size, baseLat - size + wobble * 0.3],
    [baseLon + size * 0.8, baseLat + size],
    [baseLon - size * 0.9, baseLat + size * 0.7],
    [baseLon - size * 1.1, baseLat - size * 0.2],
  ]
  const ring: LonLat[] = corners.map(([lon, lat]) => [
    Math.round(lon * 1e6) / 1e6,
    Math.round(lat * 1e6) / 1e6,
  ])
  ring.push(ring[0])
  return ring
}

const zeroHash = '0'.repeat(64)

export const useLabVault = create<VaultState>()(
  persist(
    (set, get) => ({
      initialized: false,
      authority: '',
      dimensionality: 0,
      elevationMinMm: 0,
      elevationMaxMm: 0,
      versions: [],
      createdAt: null,
      updatedAt: null,
      shapeIndex: 0,

      initAsset: (authority, dim, minMm, maxMm) => {
        // Guard order mirrors spatial_asset::init_spatial_asset.
        if (dim < 0 || dim > 2) {
          return { ok: false, msg: 'InvalidSpatialDimension (6231): dimension must be 0..=2' }
        }
        if (minMm > maxMm) {
          return { ok: false, msg: 'InvalidElevationRange (6235): elevation_min_mm > elevation_max_mm' }
        }
        const now = new Date().toISOString()
        set({
          initialized: true,
          authority,
          dimensionality: dim,
          elevationMinMm: minMm,
          elevationMaxMm: maxMm,
          versions: [],
          createdAt: now,
          updatedAt: now,
        })
        return { ok: true, msg: 'SpatialAssetCreated — PDA ["spatial_asset", parcel] initialized' }
      },

      appendVersion: async (ring, source, dimension, storageReference, submittedBy) => {
        const s = get()
        if (!s.initialized) {
          return { ok: false, msg: 'SpatialAsset not initialized — run init_spatial_asset first' }
        }
        // Guard order mirrors spatial_asset::append_geometry_version.
        if (source < 0 || source > 7) {
          return { ok: false, msg: 'InvalidGeometrySource (6232): source must be 0..=7' }
        }
        if (dimension < 0 || dimension > 2) {
          return { ok: false, msg: 'InvalidSpatialDimension (6231): dimension must be 0..=2' }
        }
        const geometryHash = await sha256Hex(JSON.stringify(ring))
        if (geometryHash === zeroHash) {
          return { ok: false, msg: 'EmptyGeometryHash (6002): all-zero geometry hash' }
        }
        if (storageReference.trim().length === 0 || storageReference.length > 128) {
          return { ok: false, msg: 'EmptyStorageReference (6134): 1..=128 chars required' }
        }
        if (dimension > s.dimensionality) {
          return {
            ok: false,
            msg: `InvalidSpatialDimension (6231): version dimension ${dimension} exceeds asset dimensionality ${s.dimensionality}`,
          }
        }
        if (s.versions.length >= MAX_GEOMETRY_VERSIONS) {
          return {
            ok: false,
            msg: `GeometryVersionsFull (6234): geometry_version_count == ${MAX_GEOMETRY_VERSIONS}`,
          }
        }
        const version = s.versions.length
        const entry: VaultVersion = {
          version,
          geometryHash,
          source,
          dimension,
          storageReference: storageReference.trim(),
          submittedBy,
          submittedAt: new Date().toISOString(),
          verified: false,
          verifiedBy: null,
          verifiedAt: null,
          ring,
        }
        set({
          versions: [...s.versions, entry],
          updatedAt: new Date().toISOString(),
        })
        return {
          ok: true,
          msg: `GeometryVersionAppended — version ${version}, PDA ["geometry_version", asset, ${version}]`,
        }
      },

      verifyVersion: (version, validator) => {
        const s = get()
        const entry = s.versions.find((v) => v.version === version)
        if (!entry) {
          return { ok: false, msg: `GeometryVersion ${version} not found` }
        }
        if (entry.verified) {
          return {
            ok: false,
            msg: 'GeometryAlreadyVerified (6236): a version can only be verified once',
          }
        }
        const now = new Date().toISOString()
        set({
          versions: s.versions.map((v) =>
            v.version === version
              ? { ...v, verified: true, verifiedBy: validator, verifiedAt: now }
              : v,
          ),
          updatedAt: now,
        })
        return {
          ok: true,
          msg: `GeometryVersionVerified — claim → fact, signed by ${validator.slice(0, 12)}…`,
        }
      },

      nextShape: () => set({ shapeIndex: get().shapeIndex + 1 }),

      reset: () =>
        set({
          initialized: false,
          authority: '',
          dimensionality: 0,
          elevationMinMm: 0,
          elevationMaxMm: 0,
          versions: [],
          createdAt: null,
          updatedAt: null,
          shapeIndex: 0,
        }),
    }),
    { name: 'terra.lab.vault' },
  ),
)
