// Spatial story schema (RFC-014 Story Engine), schema_version = 1.
// Field names mirror the Rust structs in api/src/routes/stories.rs (snake_case)
// so documents round-trip through the API without mapping layers. Keep in sync.

export const STORY_SCHEMA_VERSION = 1
export const MAX_STOPS = 20
export const MAX_DURATION_S = 300

export interface CameraKeyframe {
  time_s: number
  lon: number
  lat: number
  height_m: number
  heading_deg: number
  pitch_deg: number
  roll_deg?: number
}

export interface CameraTrack {
  keyframes: CameraKeyframe[]
}

export type StoryStopKind = 'start' | 'waypoint' | 'arrival' | 'version'

/**
 * Provenance of a geometry version referenced by a history stop. Field codes
 * mirror the RFC-013 mirrors in lib/spatial.ts (GEOMETRY_SOURCE_KEYS,
 * SPATIAL_DIMENSION_KEYS, ELEVATION_SOURCE_KEYS). This is a read-only
 * snapshot copied into the story document — the story never becomes the
 * source of truth and never writes chain state.
 */
export interface StopProvenance {
  /** GeometrySource code (0..7): manual, survey, drone, satellite, lidar, … */
  source: number
  /** SpatialDimension code (0..2): 2D, 2.5D, 3D. */
  dimension: number
  /** ElevationSource code (0..5); 0 = NONE (flat). */
  elevation_source: number
  verified: boolean
  verified_by?: string
  attest_count: number
  attestors?: string[]
  /** Evidence manifest ref (RFC-012 evidence chain), when linked. */
  evidence_manifest?: string
  /** Anchored geometry digest, when known. */
  geometry_hash?: string
}

export interface StoryStop {
  id: string
  label: string
  kind: StoryStopKind
  time_s: number
  lon: number
  lat: number
  height_m: number
  heading_deg?: number
  pitch_deg?: number
  parcel_id?: string
  asset_pubkey?: string
  geometry_version?: number
  caption?: string
  /** Present on history version stops; absent otherwise. */
  provenance?: StopProvenance
}

export type NavigationMethod = 'road-graph' | 'great-circle' | 'drawn'

export interface NavigationPath {
  id: string
  /** Honest method label — never present a great-circle as a drivable route. */
  method: NavigationMethod
  coordinates: [number, number][]
  length_m: number
  label?: string
}

export interface CaptionCue {
  time_s: number
  duration_s: number
  text: string
  stop_id?: string
}

export type StoryType = 'journey' | 'history' | 'discovery'

export interface StoryDefinition {
  schema_version: number
  id: string
  story_type: StoryType
  title: string
  description?: string
  stops: StoryStop[]
  camera_track: CameraTrack
  routes: NavigationPath[]
  captions: CaptionCue[]
  duration_s: number
}

/** Story row as served by GET /stories (list view). */
export interface StoryRow {
  id: string
  schema_version: number
  story_type: StoryType
  title: string
  description: string | null
  author_wallet: string | null
  duration_s: number
  stop_count: number
  entity_refs: string[]
  created_at: string
  updated_at: string
}

export interface StoryWithDefinition extends StoryRow {
  definition: StoryDefinition
}

export type ExportAspect = '9:16' | '16:9' | '1:1'

export interface StoryExport {
  id: string
  story_id: string
  aspect: ExportAspect
  width: number
  height: number
  fps: number
  duration_s: number
  status: 'running' | 'done' | 'failed' | 'cancelled'
  progress: number
  error: string | null
  created_at: string
  updated_at: string
}
