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
