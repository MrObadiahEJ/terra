// ---- Runtime API: /api/v1/runtime -------------------------------------------
//
// Client for the distributed spatial runtime (REQ-027 / REQ-028).
// Resolves scene graphs from World Representation Manifests, fetches
// independently stored components from storage providers with fallback, and
// drives progressive LOD loading. Mirrors the request/retry pattern in api.ts.

import { API_BASE } from './constants'
import { lodFromHeight } from './lod'

// ---------------------------------------------------------------------------
// Types (mirror the backend route schemas)
// ---------------------------------------------------------------------------

export interface SceneGraphRow {
  id: string
  manifest_id: string
  region_id: string
  lod_level: number
  node_count: number
  status: string
  created_at: string
}

export interface SceneNodeRow {
  id: string
  scene_graph_id: string
  object_id: string
  component_version_id: string
  parent_node_id: string | null
  transform: any | null
  lod_level: number
  visibility: string
  loading_state: string
  created_at: string
  spatial_extent: any
}

export interface SceneWithNodes {
  scene: SceneGraphRow
  nodes: SceneNodeRow[]
}

export interface AssetRetrieval {
  component_id: string
  component_version: string
  provider_id: string
  provider_name: string
  storage_reference: string
  latency_ms: number | null
  reliability_score: number
  global_transform: any | null
  crs: string | null
}

export interface LodLevel {
  level: number
  name: string
  description: string
  max_distance_m: number
}

export interface RecordCacheInput {
  component_id: string
  component_version: string
  storage_reference: string
  cache_path: string
  size_bytes?: number
}

export interface CacheRow {
  id: string
  component_id: string
  component_version: string
  storage_reference: string
  cache_path: string
  size_bytes: number | null
  cached_at: string
  last_accessed_at: string
  access_count: number
}

// ---- Manifest creation (World Representation Manifest, REQ-027) ----

export interface CreateManifestRequest {
  region_id: string
  region_name: string
  parent_manifest_id?: string
  neighbor_manifest_ids?: string[]
  description?: string
}

export interface ManifestRow {
  id: string
  region_id: string
  region_name: string
  version: number
  parent_manifest_id: string | null
  neighbor_manifest_ids: string[]
  description: string | null
  created_at: string
  updated_at: string
}

// ---------------------------------------------------------------------------
// Internal fetch helper (mirrors api.ts requestUrl)
// ---------------------------------------------------------------------------

async function requestUrl<T>(
  url: string,
  init?: RequestInit,
  retries = 1,
  timeoutMs = 30_000,
): Promise<T> {
  const controller = new AbortController()
  const timeout = setTimeout(() => controller.abort(), timeoutMs)
  try {
    const res = await fetch(url, {
      headers: { 'Content-Type': 'application/json' },
      ...init,
      signal: controller.signal,
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
  } catch (err) {
    const networkFailure =
      err instanceof DOMException && err.name === 'AbortError' ||
      err instanceof TypeError
    const method = init?.method?.toUpperCase() ?? 'GET'
    if (retries > 0 && networkFailure && (method === 'GET' || method === 'HEAD')) {
      await new Promise((r) => setTimeout(r, 1000))
      return requestUrl<T>(url, init, retries - 1, timeoutMs)
    }
    throw err
  } finally {
    clearTimeout(timeout)
  }
}

async function request<T>(
  path: string,
  init?: RequestInit,
  retries = 1,
  timeoutMs = 30_000,
): Promise<T> {
  return requestUrl<T>(`${API_BASE}${path}`, init, retries, timeoutMs)
}

// ---------------------------------------------------------------------------
// In-memory component asset cache (bounded, LRU-ish)
// ---------------------------------------------------------------------------

interface CachedAsset {
  componentId: string
  componentVersion: string
  data: ArrayBuffer | null
  fetchedAt: number
}

const componentCache = new Map<string, CachedAsset>()
const MAX_CACHE_ENTRIES = 200

export function cacheKey(componentId: string, componentVersion: string): string {
  return `${componentId}::${componentVersion}`
}

export function getCachedAsset(
  componentId: string,
  componentVersion: string,
): ArrayBuffer | null {
  const entry = componentCache.get(cacheKey(componentId, componentVersion))
  if (!entry) return null
  return entry.data
}

export function setCachedAsset(
  componentId: string,
  componentVersion: string,
  data: ArrayBuffer | null,
): void {
  if (componentCache.size >= MAX_CACHE_ENTRIES) {
    const firstKey = componentCache.keys().next().value
    if (firstKey !== undefined) componentCache.delete(firstKey)
  }
  componentCache.set(cacheKey(componentId, componentVersion), {
    componentId,
    componentVersion,
    data,
    fetchedAt: Date.now(),
  })
}

export function clearComponentCache(): void {
  componentCache.clear()
}

export function cachedComponentCount(): number {
  return componentCache.size
}

// ---------------------------------------------------------------------------
// Provider fallback asset retrieval
// ---------------------------------------------------------------------------

/**
 * Rank candidate retrievals for a component. Best provider first: highest
 * reliability, then lowest latency. Ties broken by provider name for stable
 * ordering. Pure and testable.
 */
export function rankProviders(retrievals: AssetRetrieval[]): AssetRetrieval[] {
  return [...retrievals].sort((a, b) => {
    const rb = b.reliability_score - a.reliability_score
    if (Math.abs(rb) > 1e-6) return rb
    const la = a.latency_ms ?? Number.MAX_SAFE_INTEGER
    const lb = b.latency_ms ?? Number.MAX_SAFE_INTEGER
    if (la !== lb) return la - lb
    return a.provider_name.localeCompare(b.provider_name)
  })
}

/**
 * Fetch an asset's bytes from a ranked list of provider storage references.
 * Returns the first successful response. Throws the last error if every
 * provider fails. Pure network side effect; callers should catch and surface
 * a "component unavailable" state.
 */
export async function fetchAssetWithFallback(
  retrievals: AssetRetrieval[],
): Promise<ArrayBuffer> {
  const ranked = rankProviders(retrievals)
  let lastError: unknown = null
  for (const r of ranked) {
    try {
      const res = await fetch(r.storage_reference)
      if (!res.ok) throw new Error(`${res.status}: ${res.statusText}`)
      return res.arrayBuffer()
    } catch (err) {
      lastError = err
    }
  }
  throw lastError ?? new Error('no providers available')
}

/**
 * Resolve the best provider for a component version and fetch its asset bytes.
 * Caches the result; on cache miss discovers providers via the runtime API and
 * fetches with fallback. Throws if no provider can serve the component.
 */
export async function fetchComponentAsset(
  componentId: string,
  componentVersion: string,
): Promise<{ data: ArrayBuffer; retrieval: AssetRetrieval }> {
  const cached = getCachedAsset(componentId, componentVersion)
  if (cached) {
    // Return cached data with an empty retrieval (transform not cached)
    return { data: cached, retrieval: { component_id: componentId, component_version: componentVersion, provider_id: '', provider_name: '', storage_reference: '', latency_ms: null, reliability_score: 0, global_transform: null, crs: null } }
  }

  const retrieval = await request<AssetRetrieval>(
    `/runtime/assets/${encodeURIComponent(componentId)}/${encodeURIComponent(
      componentVersion,
    )}`,
  )

  const data = await fetchAssetWithFallback([retrieval])
  setCachedAsset(componentId, componentVersion, data)
  return { data, retrieval }
}

// ---------------------------------------------------------------------------
// Scene graph resolution
// ---------------------------------------------------------------------------

export async function resolveScene(manifestId: string): Promise<SceneWithNodes> {
  return request<SceneWithNodes>(
    `/runtime/scene/${encodeURIComponent(manifestId)}`,
  )
}

export async function resolveSceneAtLod(
  manifestId: string,
  lod: number,
): Promise<SceneWithNodes> {
  return request<SceneWithNodes>(
    `/runtime/scene/${encodeURIComponent(manifestId)}/${lod}`,
  )
}

export async function listLodLevels(maxLevel?: number): Promise<LodLevel[]> {
  const q = maxLevel != null ? `?max_level=${maxLevel}` : ''
  return request<LodLevel[]>(`/runtime/lod${q}`)
}

export async function recordCache(input: RecordCacheInput): Promise<CacheRow> {
  return request<CacheRow>(`/runtime/cache`, {
    method: 'POST',
    body: JSON.stringify(input),
  })
}

// ---------------------------------------------------------------------------
// World Representation Manifest (REQ-027)
// ---------------------------------------------------------------------------

export async function createManifest(
  input: CreateManifestRequest,
): Promise<ManifestRow> {
  return request<ManifestRow>(`/manifests`, {
    method: 'POST',
    body: JSON.stringify(input),
  })
}

export async function listManifests(
  regionId?: string,
  maxLevel?: number,
): Promise<ManifestRow[]> {
  const q = new URLSearchParams()
  if (regionId) q.set('region_id', regionId)
  if (maxLevel != null) q.set('limit', String(maxLevel))
  return request<ManifestRow[]>(`/manifests${q.size > 0 ? `?${q}` : ''}`)
}

// ---------------------------------------------------------------------------
// Progressive LOD selection
// ---------------------------------------------------------------------------

/**
 * Choose a runtime LOD (0-4) from camera height. Maps the frontend LOD ladder
 * (0-7) onto the runtime's 0-4 range so coarse views resolve small scene
 * graphs and close views resolve detailed ones.
 */
export function runtimeLodFromHeight(heightM: number): number {
  const frontendLod = lodFromHeight(heightM)
  return Math.min(4, frontendLod)
}
