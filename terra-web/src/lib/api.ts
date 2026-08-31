import { API_BASE } from './constants'

// ---- Off-chain API: /api/v1/parcels --------------------------------------

export interface OffChainParcel {
  id: string
  name: string
  owner: string
  status: string
  geometry: string | null // GeoJSON string
  area_m2: number | null
  created_at: string
  updated_at: string
}

export interface NewParcelInput {
  name: string
  owner: string
  status?: string
  geometry: unknown // GeoJSON Polygon
}

// ---- Off-chain API: /api/v1/geo ------------------------------------------

export interface RoadAccess {
  lon: number
  lat: number
  distance_m: number
  road_name: string | null
  highway: string
}

export interface Poi {
  id: number
  name: string | null
  category: string
  kind: string
  lon: number
  lat: number
}

export interface GeoStats {
  nodes?: number
  roads?: number
  road_segments?: number
  road_length_km?: number
  pois?: number
  bbox?: { min_lon: number; min_lat: number; max_lon: number; max_lat: number }
  loaded?: boolean
}

// ---- Off-chain API: /api/v1/fusion ---------------------------------------

export interface ReachabilityResult {
  nearest_road_m: number
  boundary_accesses: number
  component_km: number
  sealed_reachable: boolean
  sealed_network_m: number | null
  flags: number
  access_hash: string
}

export interface RoadRow {
  id: number
  name: string | null
  highway: string
  oneway: boolean
  length_m: number
  geometry: string | null
  ingested_at: string
}

export interface PoiRow {
  id: number
  name: string | null
  category: string
  kind: string
  tags?: Record<string, unknown>
  geometry: string | null
  ingested_at: string
}

export interface FusionStats {
  roads?: number
  pois?: number
  pilot_zones?: number
  photogrammetry_assets?: number
}

// ---- Off-chain API: /api/v1/pilot-zones ----------------------------------

export interface PilotZone {
  id: string
  name: string
  description: string | null
  geometry: string | null
  created_at: string
}

export interface PhotogrammetryAsset {
  id: string
  pilot_zone_id: string
  asset_type: string
  name: string
  format: string | null
  file_path: string | null
  resolution_m: number | null
  point_count: number | null
  metadata?: Record<string, unknown>
  geometry: string | null
  created_at: string
}

// ---- Off-chain API: attestations + documents (binding to on-chain) --------

export interface RegisterAttestationInput {
  onchain_id: string
  specifier: string
  content_hash: string
  required: number
  count: number
  validators: string[]
}

export interface Attestation {
  id: string
  parcel_id: string
  onchain_id: string
  specifier: string
  content_hash: string
  required: number
  validators: string[]
  created_at: string
}

export interface ValidationView {
  validator: string
  signature: string
  valid: boolean
  created_at: string
}

export interface AttestationDetail extends Attestation {
  has_quorum: boolean
  signatories: number
  required: number
  validations: ValidationView[]
}

export interface SubmitValidationInput {
  validator: string
  signature: string
  content_hash: string
}

export interface BoundDocument {
  id: string
  parcel_id: string
  title: string
  category: string
  content_hash: string
  storage_ref: string
  owner: string
  created_at: string
}

export interface RegisterDocumentInput {
  title: string
  category: string
  content_hash: string
  storage_ref: string
  owner: string
}

// ---- Off-chain API: identities + wallet passation --------------------------

export interface BindIdentityInput {
  identity_hash: string // hex(32) sha256 over the person's identity credential
  owner: string // base58 wallet the person holds
  recovery: string // base58 backup/recovery wallet
  display_name?: string
  national_id?: string
  phone?: string
}

export interface IdentityRow {
  id: string
  identity_hash: string
  owner: string
  recovery: string
  parcel_count: number
  created_at: string
}

export interface IdentityView extends IdentityRow {
  display_name?: string | null
  national_id?: string | null
  phone?: string | null
}

export interface RequestSuccessionInput {
  successor: string // base58 wallet gaining control
  kind: number // 0=successor(heir), 1=recovery, 2=transfer
}

export interface SuccessionRow {
  id: string
  identity_id: string
  identity_hash: string
  kind: number
  successor: string
  requested_at: string
  effective_at: string
  status: string
}

export interface RotateValidatorsInput {
  version: number
  required: number
  validators: string[]
  rotated_by: string
}

// ---- client ---------------------------------------------------------------

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const res = await fetch(`${API_BASE}${path}`, {
    headers: { 'Content-Type': 'application/json' },
    ...init,
  })
  if (!res.ok) {
    let detail = res.statusText
    try {
      const body = await res.json()
      detail = body?.message ?? body?.error ?? detail
    } catch {
      /* ignore */
    }
    throw new Error(`${res.status}: ${detail}`)
  }
  if (res.status === 204) return undefined as T
  return (await res.json()) as T
}

export const api = {
  // parcels
  listParcels: (bbox?: { minx: number; miny: number; maxx: number; maxy: number }) => {
    const q = bbox
      ? `?minx=${bbox.minx}&miny=${bbox.miny}&maxx=${bbox.maxx}&maxy=${bbox.maxy}`
      : ''
    return request<OffChainParcel[]>(`/parcels${q}`)
  },
  getParcel: (id: string) => request<OffChainParcel>(`/parcels/${id}`),
  createParcel: (input: NewParcelInput) =>
    request<OffChainParcel>(`/parcels`, { method: 'POST', body: JSON.stringify(input) }),
  deleteParcel: (id: string) => request<void>(`/parcels/${id}`, { method: 'DELETE' }),

  // geo (in-memory OSM — only when server loaded a PBF)
  nearestRoads: (lon: number, lat: number, limit = 5) =>
    request<RoadAccess[]>(`/geo/nearest-roads?lon=${lon}&lat=${lat}&limit=${limit}`),
  pois: (lon: number, lat: number, radius = 1000, category?: string, limit = 20) =>
    request<Poi[]>(
      `/geo/pois?lon=${lon}&lat=${lat}&radius=${radius}&limit=${limit}` +
        (category ? `&category=${encodeURIComponent(category)}` : ''),
    ),
  geoStats: () => request<GeoStats>(`/geo/stats`),

  // fusion (PostGIS — requires ingestion)
  fusionStats: () => request<FusionStats>(`/fusion/stats`),
  roads: (bbox?: { minx: number; miny: number; maxx: number; maxy: number }) => {
    const q = bbox
      ? `?minx=${bbox.minx}&miny=${bbox.miny}&maxx=${bbox.maxx}&maxy=${bbox.maxy}`
      : ''
    return request<RoadRow[]>(`/fusion/roads${q}`)
  },
  poisFusion: (bbox?: { minx: number; miny: number; maxx: number; maxy: number }) => {
    const q = bbox
      ? `?minx=${bbox.minx}&miny=${bbox.miny}&maxx=${bbox.maxx}&maxy=${bbox.maxy}`
      : ''
    return request<PoiRow[]>(`/fusion/pois${q}`)
  },
  ingestOsm: () => request<{ roads_upserted: number; pois_upserted: number }>(
    `/fusion/ingest`,
    { method: 'POST' },
  ),
  reachability: (parcelIdHex: string, geometry: unknown) =>
    request<ReachabilityResult>(`/fusion/reachability`, {
      method: 'POST',
      body: JSON.stringify({ parcel_id: parcelIdHex, geometry }),
    }),

  // pilot zones
  listPilotZones: () => request<PilotZone[]>(`/pilot-zones`),
  getPilotZone: (id: string) => request<PilotZone>(`/pilot-zones/${id}`),
  createPilotZone: (input: { name: string; description?: string; geometry: unknown }) =>
    request<PilotZone>(`/pilot-zones`, { method: 'POST', body: JSON.stringify(input) }),
  listAssets: (zoneId: string) => request<PhotogrammetryAsset[]>(`/pilot-zones/${zoneId}/assets`),
  createAsset: (zoneId: string, input: Partial<PhotogrammetryAsset>) =>
    request<PhotogrammetryAsset>(`/pilot-zones/${zoneId}/assets`, {
      method: 'POST',
      body: JSON.stringify(input),
    }),

  // attestations + documents (binding heavy off-chain data to on-chain)
  registerAttestation: (parcelId: string, input: RegisterAttestationInput) =>
    request<Attestation>(`/parcels/${parcelId}/attestations`, {
      method: 'POST',
      body: JSON.stringify(input),
    }),
  getAttestation: (parcelId: string, specifier: string) =>
    request<AttestationDetail>(`/parcels/${parcelId}/attestations/${specifier}`),
  submitValidation: (parcelId: string, specifier: string, input: SubmitValidationInput) =>
    request<ValidationView>(`/parcels/${parcelId}/attestations/${specifier}/validations`, {
      method: 'POST',
      body: JSON.stringify(input),
    }),
  registerDocument: (parcelId: string, input: RegisterDocumentInput) =>
    request<BoundDocument>(`/parcels/${parcelId}/documents`, {
      method: 'POST',
      body: JSON.stringify(input),
    }),
  listDocuments: (parcelId: string) =>
    request<BoundDocument[]>(`/parcels/${parcelId}/documents`),

  // identities + wallet passation (person->wallet binding, recovery, succession)
  bindIdentity: (input: BindIdentityInput) =>
    request<IdentityView>(`/identities`, { method: 'POST', body: JSON.stringify(input) }),
  getIdentityByWallet: (wallet: string) =>
    request<IdentityView>(`/identities/${wallet}`),
  requestSuccession: (identityHash: string, input: RequestSuccessionInput) =>
    request<SuccessionRow>(`/identities/${identityHash}/successions`, {
      method: 'POST',
      body: JSON.stringify(input),
    }),
  cancelSuccession: (identityHash: string, successor: string) =>
    request<SuccessionRow>(`/identities/${identityHash}/successions/${successor}/cancel`, {
      method: 'POST',
    }),
  claimSuccession: (identityHash: string, successor: string) =>
    request<IdentityRow>(`/identities/${identityHash}/successions/${successor}/claim`, {
      method: 'POST',
    }),

  // validator rotation (fix for dead/leaving validators) on a parcel's attestation
  rotateValidators: (parcelId: string, specifier: string, input: RotateValidatorsInput) =>
    request<{ attestation_id: string; version: number; required: number; validators: string[] }>(
      `/parcels/${parcelId}/attestations/${specifier}/rotation`,
      { method: 'POST', body: JSON.stringify(input) },
    ),
}

export function parseGeoJSON<T>(geoJson: string | null | undefined): T | null {
  if (!geoJson) return null
  try {
    return JSON.parse(geoJson) as T
  } catch {
    return null
  }
}
