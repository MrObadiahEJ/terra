// Terra Lab — Geometry Vault (RFC-013 Phases A+B) state.
// Mirrors the on-chain SpatialAsset / GeometryVersion handlers 1:1, including
// guard order and error codes; all hashes are real SHA-256 computed locally.

import { create } from 'zustand'
import { persist } from 'zustand/middleware'
import { DEFAULT_FOCUS } from './constants'
import type { LonLat } from './geo'
import { sha256Hex } from './geo'

/** terra-core spatial_asset.rs: MAX_GEOMETRY_VERSIONS */
export const MAX_GEOMETRY_VERSIONS = 64
/** terra-core spatial_asset.rs: DEFAULT_GEOMETRY_QUORUM */
export const DEFAULT_GEOMETRY_QUOROM = 2
/** terra-core lib.rs: MAX_VALIDATORS (attestors array length) */
export const MAX_VALIDATORS = 8
/** terra-core evidence_manifest.rs: MAX_MANIFEST_ARTIFACTS */
export const MAX_MANIFEST_ARTIFACTS = 32

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

/** terra-core spatial_asset.rs: elevation_source (Phase B) */
export const ELEVATION_SOURCES = [
  { code: 0, id: 'NONE', label: 'None (flat layer)', color: '#9ca3af' },
  { code: 1, id: 'SURVEY', label: 'Ground survey', color: '#3b82f6' },
  { code: 2, id: 'DEM', label: 'DEM raster', color: '#8b5cf6' },
  { code: 3, id: 'LIDAR', label: 'LiDAR point cloud', color: '#06b6d4' },
  { code: 4, id: 'PHOTOGRAMMETRY', label: 'Photogrammetry mesh', color: '#f59e0b' },
  { code: 5, id: 'MANUAL', label: 'Manual entry', color: '#f43f5e' },
  { code: 6, id: 'ESTIMATED', label: 'Estimated from footprint', color: '#84cc16' },
] as const

/** Demo validator wallets for the quorum (lab only — no keys, labels). */
export const DEMO_VALIDATORS = [
  'terra-lab-validator-1',
  'terra-lab-validator-2',
  'terra-lab-validator-3',
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
  // --- Phase B (RFC-013 §7) ---
  evidenceManifest: string | null // demo manifest label; null = standalone append
  elevationMinMm: number
  elevationMaxMm: number
  elevationSource: number
  required: number // quorum threshold snapshot (fallback 2)
  attestors: string[] // unique validator labels, append-only
}

export type LabResult = { ok: true; msg: string } | { ok: false; msg: string }

// --- pure guards (mirror spatial_asset.rs helpers) --------------------------

export function elevationMatchesDimension(source: number, dimension: number): boolean {
  return (source === 0) === (dimension < 1)
}

export function elevationNoneIsZero(source: number, minMm: number, maxMm: number): boolean {
  return source !== 0 || (minMm === 0 && maxMm === 0)
}

export function elevationWithinAsset(
  minMm: number,
  maxMm: number,
  assetMin: number,
  assetMax: number,
): boolean {
  return minMm >= assetMin && maxMm <= assetMax
}

/** Phase B invariant summary for the UI (all five checks at once). */
export function elevationInvariants(
  source: number,
  dimension: number,
  minMm: number,
  maxMm: number,
  assetMin: number,
  assetMax: number,
): { id: string; label: string; ok: boolean }[] {
  return [
    { id: 'code', label: 'source code ≤ 5', ok: source >= 0 && source <= 5 },
    { id: 'range', label: 'min ≤ max', ok: minMm <= maxMm },
    { id: 'none0', label: 'NONE ⇒ (0,0)', ok: elevationNoneIsZero(source, minMm, maxMm) },
    { id: 'bij', label: 'NONE ⇔ flat D2', ok: elevationMatchesDimension(source, dimension) },
    {
      id: 'box',
      label: 'envelope ⊆ asset',
      ok: elevationWithinAsset(minMm, maxMm, assetMin, assetMax),
    },
  ]
}

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
  // --- Phase B ---
  /** Global QuorumConfig (0,[0,0]) demo value; 0 = not configured. */
  quorumConfig: number
  /** Demo verification task state (terminal tasks reject anchored appends). */
  taskCancelled: boolean
  /** Evidence manifest nonce + artifact count (null = no manifest yet). */
  manifestNonce: number | null
  manifestArtifacts: number
  manifestCreatedAt: string | null

  initAsset: (authority: string, dim: number, minMm: number, maxMm: number) => LabResult
  appendVersion: (
    ring: LonLat[],
    source: number,
    dimension: number,
    storageReference: string,
    submittedBy: string,
    elevMinMm: number,
    elevMaxMm: number,
    elevSource: number,
    fromEvidence: boolean,
  ) => Promise<LabResult>
  attestVersion: (version: number, validator: string) => LabResult
  setQuorumConfig: (required: number) => LabResult
  submitManifest: (artifactCount: number) => LabResult
  setTaskCancelled: (cancelled: boolean) => void
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

interface PersistedShape {
  initialized?: boolean
  authority?: string
  dimensionality?: number
  elevationMinMm?: number
  elevationMaxMm?: number
  versions?: Partial<VaultVersion>[]
  createdAt?: string | null
  updatedAt?: string | null
  shapeIndex?: number
  quorumConfig?: number
  taskCancelled?: boolean
  manifestNonce?: number | null
  manifestArtifacts?: number
  manifestCreatedAt?: string | null
}

/** Fill Phase B fields into state persisted before RFC-013 Phase B. */
function migrate(p: PersistedShape | undefined): PersistedShape {
  if (!p) return {}
  return {
    ...p,
    versions: (p.versions ?? []).map((v) => ({
      evidenceManifest: null,
      elevationMinMm: 0,
      elevationMaxMm: 0,
      elevationSource: 0,
      required: DEFAULT_GEOMETRY_QUOROM,
      attestors: [],
      ...v,
    })),
  }
}

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
      quorumConfig: 0,
      taskCancelled: false,
      manifestNonce: null,
      manifestArtifacts: 0,
      manifestCreatedAt: null,

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

      appendVersion: async (
        ring,
        source,
        dimension,
        storageReference,
        submittedBy,
        elevMinMm,
        elevMaxMm,
        elevSource,
        fromEvidence,
      ) => {
        const s = get()
        if (!s.initialized) {
          return { ok: false, msg: 'SpatialAsset not initialized — run init_spatial_asset first' }
        }

        // Evidence-linked variant guards first (from_evidence handler,
        // before append_version_common runs).
        if (fromEvidence) {
          if (s.taskCancelled) {
            return {
              ok: false,
              msg: 'TaskAlreadyFinalized (6174): task is terminal — anchored append rejected',
            }
          }
          if (s.manifestNonce === null || s.manifestArtifacts < 1) {
            return {
              ok: false,
              msg: 'EmptyEvidenceManifest (6239): manifest must contain ≥ 1 artifact',
            }
          }
        }

        // Guard order mirrors spatial_asset::append_version_common.
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
        // --- Phase B elevation guards ---
        if (elevSource < 0 || elevSource > 5) {
          return { ok: false, msg: 'InvalidElevationSource (6237): elevation source must be 0..=5' }
        }
        if (elevMinMm > elevMaxMm) {
          return { ok: false, msg: 'InvalidElevationRange (6235): elevation_min_mm > elevation_max_mm' }
        }
        if (!elevationNoneIsZero(elevSource, elevMinMm, elevMaxMm)) {
          return {
            ok: false,
            msg: 'InvalidElevationSource (6237): NONE provenance must carry an all-zero envelope',
          }
        }
        if (!elevationMatchesDimension(elevSource, dimension)) {
          return {
            ok: false,
            msg: 'InvalidElevationSource (6237): provenance ⇔ dimension mismatch (NONE only on flat D2)',
          }
        }
        if (!elevationWithinAsset(elevMinMm, elevMaxMm, s.elevationMinMm, s.elevationMaxMm)) {
          return {
            ok: false,
            msg: `InvalidElevationRange (6235): version envelope [${elevMinMm}…${elevMaxMm}] outside asset envelope [${s.elevationMinMm}…${s.elevationMaxMm}]`,
          }
        }

        const version = s.versions.length
        // Quorum snapshot: configured global QuorumConfig wins, else 2.
        const required = s.quorumConfig > 0 ? s.quorumConfig : DEFAULT_GEOMETRY_QUOROM
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
          evidenceManifest:
            fromEvidence && s.manifestNonce !== null
              ? `evidence_manifest #${s.manifestNonce}`
              : null,
          elevationMinMm: elevMinMm,
          elevationMaxMm: elevMaxMm,
          elevationSource: elevSource,
          required,
          attestors: [],
        }
        set({
          versions: [...s.versions, entry],
          updatedAt: new Date().toISOString(),
        })
        return {
          ok: true,
          msg: fromEvidence
            ? `GeometryVersionAppended — v${version} anchored to ${entry.evidenceManifest} (quorum required = ${required})`
            : `GeometryVersionAppended — version ${version}, PDA ["geometry_version", asset, ${version}] (quorum required = ${required})`,
        }
      },

      attestVersion: (version, validator) => {
        const s = get()
        const entry = s.versions.find((v) => v.version === version)
        if (!entry) {
          return { ok: false, msg: `GeometryVersion ${version} not found` }
        }
        // Guard order mirrors spatial_asset::verify_geometry_version.
        if (entry.verified) {
          return {
            ok: false,
            msg: 'GeometryAlreadyVerified (6236): quorum already flipped this version',
          }
        }
        if (entry.attestors.length >= MAX_VALIDATORS) {
          return {
            ok: false,
            msg: `GeometryQuorumFull (6240): attestors array full (${MAX_VALIDATORS})`,
          }
        }
        if (entry.attestors.includes(validator)) {
          return {
            ok: false,
            msg: 'GeometryAlreadyAttested (6238): this validator already attested',
          }
        }
        const now = new Date().toISOString()
        const attestors = [...entry.attestors, validator]
        const flips = attestors.length >= entry.required
        set({
          versions: s.versions.map((v) =>
            v.version === version
              ? {
                  ...v,
                  attestors,
                  verified: flips ? true : v.verified,
                  verifiedBy: flips ? validator : v.verifiedBy,
                  verifiedAt: flips ? now : v.verifiedAt,
                }
              : v,
          ),
          updatedAt: now,
        })
        if (flips) {
          return {
            ok: true,
            msg: `GeometryVersionAttested ${attestors.length}/${entry.required} → GeometryVersionVerified — claim → fact, completed by ${validator.slice(0, 24)}`,
          }
        }
        return {
          ok: true,
          msg: `GeometryVersionAttested ${attestors.length}/${entry.required} — recorded, quorum not met yet`,
        }
      },

      setQuorumConfig: (required) => {
        // Mirrors set_quorum_config bounds (1..=MAX_VALIDATORS).
        if (!Number.isInteger(required) || required < 1 || required > MAX_VALIDATORS) {
          return {
            ok: false,
            msg: `InvalidRequiredAttestations: required_attestations must be 1..=${MAX_VALIDATORS}`,
          }
        }
        set({ quorumConfig: required })
        return {
          ok: true,
          msg: `QuorumConfigSet — global (0, [0,0]) → required_attestations = ${required} (snapshots into the next append)`,
        }
      },

      submitManifest: (artifactCount) => {
        const s = get()
        if (!Number.isInteger(artifactCount) || artifactCount < 0 || artifactCount > MAX_MANIFEST_ARTIFACTS) {
          return {
            ok: false,
            msg: `artifact index must be 0..=${MAX_MANIFEST_ARTIFACTS - 1}`,
          }
        }
        const nonce = s.manifestNonce === null ? 0 : s.manifestNonce + 1
        set({
          manifestNonce: nonce,
          manifestArtifacts: artifactCount,
          manifestCreatedAt: new Date().toISOString(),
        })
        if (artifactCount < 1) {
          return {
            ok: true,
            msg: `EvidenceManifest #${nonce} submitted with 0 artifacts — anchored appends will fail EmptyEvidenceManifest (6239)`,
          }
        }
        return {
          ok: true,
          msg: `EvidenceManifest #${nonce} submitted — ${artifactCount} artifact${artifactCount > 1 ? 's' : ''} (task ${s.taskCancelled ? 'cancelled' : 'open'})`,
        }
      },

      setTaskCancelled: (cancelled) => set({ taskCancelled: cancelled }),

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
          quorumConfig: 0,
          taskCancelled: false,
          manifestNonce: null,
          manifestArtifacts: 0,
          manifestCreatedAt: null,
        }),
    }),
    {
      name: 'terra.lab.vault',
      version: 1,
      migrate: (state) => migrate(state as PersistedShape | undefined) as VaultState,
    },
  ),
)
