import * as Cesium from 'cesium'
import { TIER_SETTINGS, type GraphicsTier } from './perfTier'

const KEYLESS_TERRAIN_URL = 'https://terrain.reearth.land/cesium-mesh/ellipsoid'
const RETRY_STATUSES = [429, 502, 503, 504]
const RETRY_ATTEMPTS = 3
const RETRY_BASE_MS = 750
const RETRY_MAX_MS = 15_000
const RETRY_SPREAD_MS = 1_500

const ION_TOKEN_KEY = 'terra-ion-token'

export type TerrainStatus = 'flat' | 'loading' | 'keyless' | 'ion'

export interface TerrainResolution {
  provider: Cesium.TerrainProvider
  source: 'keyless' | 'ion'
}

export function loadIonToken(): string {
  try {
    return localStorage.getItem(ION_TOKEN_KEY) ?? ''
  } catch {
    return ''
  }
}

export function saveIonToken(token: string): void {
  try {
    localStorage.setItem(ION_TOKEN_KEY, token.trim())
  } catch {
    /* private mode */
  }
}

export function clearIonToken(): void {
  try {
    localStorage.removeItem(ION_TOKEN_KEY)
  } catch {
    /* private mode */
  }
}

function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms))
}

function headerValue(headers: unknown, name: string): string | null {
  if (!headers || typeof headers !== 'object') return null
  for (const [key, value] of Object.entries(headers as Record<string, unknown>)) {
    if (key.toLowerCase() === name.toLowerCase()) return String(value)
  }
  return null
}

function parseRetryAfter(value: string | null, nowMs: number): number | null {
  if (value == null) return null
  const text = value.trim()
  if (!text) return null
  if (/^\d+$/.test(text)) return Number(text) * 1000
  const at = Date.parse(text)
  if (!Number.isFinite(at)) return null
  return Math.max(0, at - nowMs)
}

const REQUEST_CANCELLED = 4

function requestCancelled(resource: Cesium.Resource | undefined): boolean {
  const request = resource?.request as
    | { cancelled?: boolean; state?: number }
    | undefined
  return request?.cancelled === true || request?.state === REQUEST_CANCELLED
}

// Shared-cooldown retry policy for the keyless quantized-mesh tiles.
// Re:Earth answers zoom-out bursts with 429; Cesium marks throttled tiles
// FAILED and upsamples the parent mesh, so the terrain stays coarse. The
// retryCallback is copied by Cesium onto every derived tile fetch.
function createRetryPolicy(): {
  retryCallback: Cesium.Resource.RetryCallback
  retryAttempts: number
} {
  const attemptsByResource = new WeakMap<Cesium.Resource, number>()
  let cooldownUntil = 0

  const retryCallback: Cesium.Resource.RetryCallback = async (resource, error) => {
    const statusCode = (error as Cesium.RequestErrorEvent | undefined)?.statusCode
    if (statusCode === undefined || !RETRY_STATUSES.includes(statusCode)) return false
    if (!resource || requestCancelled(resource)) return false

    const attempt = attemptsByResource.get(resource) ?? 0
    attemptsByResource.set(resource, attempt + 1)

    const started = Date.now()
    const backoff = Math.min(RETRY_MAX_MS, RETRY_BASE_MS * 2 ** attempt)
    const retryAfter = parseRetryAfter(
      headerValue(
        (error as Cesium.RequestErrorEvent | undefined)?.responseHeaders,
        'retry-after',
      ),
      started,
    )
    const delay = Math.min(RETRY_MAX_MS, Math.max(backoff, retryAfter ?? 0))
    if (cooldownUntil <= started) cooldownUntil = started + delay
    else if (started + delay > cooldownUntil) cooldownUntil = started + delay

    const jitter = Math.random() * RETRY_SPREAD_MS
    const deadline = started + RETRY_MAX_MS + RETRY_SPREAD_MS
    for (let i = 0; i < 32; i += 1) {
      const remaining = Math.min(cooldownUntil + jitter, deadline) - Date.now()
      if (remaining <= 0) break
      await sleep(remaining)
      if (requestCancelled(resource)) return false
    }
    return !requestCancelled(resource)
  }

  return { retryCallback, retryAttempts: RETRY_ATTEMPTS }
}

async function createIonTerrain(token: string): Promise<Cesium.TerrainProvider> {
  const resource = await Cesium.IonResource.fromAssetId(1, { accessToken: token })
  return Cesium.CesiumTerrainProvider.fromUrl(resource, {
    requestVertexNormals: true,
    requestWaterMask: false,
    ellipsoid: Cesium.Ellipsoid.WGS84,
  })
}

async function createKeylessTerrain(): Promise<Cesium.TerrainProvider> {
  const policy = createRetryPolicy()
  const resource = new Cesium.Resource({
    url: KEYLESS_TERRAIN_URL,
    retryCallback: policy.retryCallback,
    retryAttempts: policy.retryAttempts,
  })
  return Cesium.CesiumTerrainProvider.fromUrl(resource, {
    requestVertexNormals: true,
    requestWaterMask: false,
    ellipsoid: Cesium.Ellipsoid.WGS84,
  })
}

export async function resolveTerrain(tier: GraphicsTier): Promise<TerrainResolution | null> {
  if (!TIER_SETTINGS[tier].terrain) return null
  const ion = loadIonToken()
  if (ion) {
    try {
      return { provider: await createIonTerrain(ion), source: 'ion' }
    } catch {
      /* transient ion failure — fall back to the keyless mesh, keep the token */
    }
  }
  try {
    return { provider: await createKeylessTerrain(), source: 'keyless' }
  } catch {
    return null
  }
}
