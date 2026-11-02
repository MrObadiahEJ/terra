import { describe, expect, it, vi, beforeEach, afterEach } from 'vitest'
import {
  rankProviders,
  cacheKey,
  getCachedAsset,
  setCachedAsset,
  clearComponentCache,
  cachedComponentCount,
  runtimeLodFromHeight,
  fetchAssetWithFallback,
  type AssetRetrieval,
} from './runtime'

// ---------------------------------------------------------------------------
// rankProviders — pure, no network
// ---------------------------------------------------------------------------

describe('rankProviders', () => {
  const base: AssetRetrieval = {
    component_id: 'comp-a',
    component_version: 'v1',
    provider_id: 'p1',
    provider_name: 'Provider A',
    storage_reference: 'https://storage.example.com/a.glb',
    latency_ms: 100,
    reliability_score: 0.9,
  }

  it('sorts by highest reliability first', () => {
    const retrievals: AssetRetrieval[] = [
      { ...base, provider_name: 'Low', reliability_score: 0.5 },
      { ...base, provider_name: 'High', reliability_score: 0.95 },
      { ...base, provider_name: 'Mid', reliability_score: 0.7 },
    ]
    const ranked = rankProviders(retrievals)
    expect(ranked[0].provider_name).toBe('High')
    expect(ranked[1].provider_name).toBe('Mid')
    expect(ranked[2].provider_name).toBe('Low')
  })

  it('breaks reliability ties by lowest latency', () => {
    const retrievals: AssetRetrieval[] = [
      { ...base, provider_name: 'Slow', latency_ms: 500, reliability_score: 0.9 },
      { ...base, provider_name: 'Fast', latency_ms: 50, reliability_score: 0.9 },
    ]
    const ranked = rankProviders(retrievals)
    expect(ranked[0].provider_name).toBe('Fast')
    expect(ranked[1].provider_name).toBe('Slow')
  })

  it('breaks latency ties by provider name ascending', () => {
    const retrievals: AssetRetrieval[] = [
      { ...base, provider_name: 'Zeta', latency_ms: 100, reliability_score: 0.9 },
      { ...base, provider_name: 'Alpha', latency_ms: 100, reliability_score: 0.9 },
      { ...base, provider_name: 'Beta', latency_ms: 100, reliability_score: 0.9 },
    ]
    const ranked = rankProviders(retrievals)
    expect(ranked.map((r) => r.provider_name)).toEqual(['Alpha', 'Beta', 'Zeta'])
  })

  it('treats missing latency as worst', () => {
    const retrievals: AssetRetrieval[] = [
      { ...base, provider_name: 'NoLatency', latency_ms: null, reliability_score: 0.9 },
      { ...base, provider_name: 'HasLatency', latency_ms: 999, reliability_score: 0.9 },
    ]
    const ranked = rankProviders(retrievals)
    expect(ranked[0].provider_name).toBe('HasLatency')
    expect(ranked[1].provider_name).toBe('NoLatency')
  })

  it('does not mutate the input array', () => {
    const retrievals: AssetRetrieval[] = [
      { ...base, provider_name: 'B', reliability_score: 0.5 },
      { ...base, provider_name: 'A', reliability_score: 0.9 },
    ]
    const before = [...retrievals]
    rankProviders(retrievals)
    expect(retrievals).toEqual(before)
  })
})

// ---------------------------------------------------------------------------
// cacheKey — pure
// ---------------------------------------------------------------------------

describe('cacheKey', () => {
  it('produces deterministic keys', () => {
    expect(cacheKey('comp-1', 'v1')).toBe('comp-1::v1')
    expect(cacheKey('comp-2', 'v1')).not.toBe(cacheKey('comp-1', 'v1'))
    expect(cacheKey('comp-1', 'v2')).not.toBe(cacheKey('comp-1', 'v1'))
  })
})

// ---------------------------------------------------------------------------
// in-memory cache — set/get/clear/count
// ---------------------------------------------------------------------------

describe('component cache', () => {
  beforeEach(() => {
    clearComponentCache()
  })

  afterEach(() => {
    clearComponentCache()
  })

  it('getCachedAsset returns null for empty cache', () => {
    expect(getCachedAsset('comp-1', 'v1')).toBeNull()
    expect(cachedComponentCount()).toBe(0)
  })

  it('set then get returns the same bytes', () => {
    const data = new Uint8Array([1, 2, 3]).buffer as ArrayBuffer
    setCachedAsset('comp-1', 'v1', data)
    expect(getCachedAsset('comp-1', 'v1')).toBe(data)
    expect(cachedComponentCount()).toBe(1)
  })

  it('different component/version are independent entries', () => {
    setCachedAsset('a', 'v1', new Uint8Array([1]).buffer)
    setCachedAsset('a', 'v2', new Uint8Array([2]).buffer)
    setCachedAsset('b', 'v1', new Uint8Array([3]).buffer)
    expect(cachedComponentCount()).toBe(3)
    expect(getCachedAsset('a', 'v1')?.byteLength).toBe(1)
    expect(getCachedAsset('a', 'v2')?.byteLength).toBe(1)
    expect(getCachedAsset('b', 'v1')?.byteLength).toBe(1)
  })

  it('clearing removes all entries', () => {
    setCachedAsset('a', 'v1', new Uint8Array([1]).buffer)
    setCachedAsset('b', 'v1', new Uint8Array([2]).buffer)
    clearComponentCache()
    expect(cachedComponentCount()).toBe(0)
    expect(getCachedAsset('a', 'v1')).toBeNull()
    expect(getCachedAsset('b', 'v1')).toBeNull()
  })
})

// ---------------------------------------------------------------------------
// runtimeLodFromHeight — pure
// ---------------------------------------------------------------------------

describe('runtimeLodFromHeight', () => {
  it('maps globe height to coarse runtime LOD', () => {
    expect(runtimeLodFromHeight(10_000_000)).toBe(0)
    expect(runtimeLodFromHeight(1_500_000)).toBe(1)
    expect(runtimeLodFromHeight(400_000)).toBe(2)
  })

  it('maps city/parcel height to mid-fine runtime LOD', () => {
    expect(runtimeLodFromHeight(60_000)).toBe(3)
    expect(runtimeLodFromHeight(12_000)).toBe(4)
    expect(runtimeLodFromHeight(1_500)).toBe(4)
  })

  it('maps close height to fine runtime LOD, clamped at 4', () => {
    expect(runtimeLodFromHeight(250)).toBe(4)
    expect(runtimeLodFromHeight(40)).toBe(4)
    expect(runtimeLodFromHeight(1)).toBe(4)
  })

  it('always returns 0-4', () => {
    for (const h of [0, 100, 1000, 10000, 100000, 1000000, 10000000]) {
      const lod = runtimeLodFromHeight(h)
      expect(lod).toBeGreaterThanOrEqual(0)
      expect(lod).toBeLessThanOrEqual(4)
    }
  })
})

// ---------------------------------------------------------------------------
// fetchAssetWithFallback — network, mocked fetch
// ---------------------------------------------------------------------------

const okResponse = (body: ArrayBuffer, status = 200) =>
  new Response(body, { status, headers: { 'Content-Type': 'application/octet-stream' } })

describe('fetchAssetWithFallback', () => {
  const calls: string[] = []

  beforeEach(() => {
    calls.length = 0
    vi.restoreAllMocks()
    vi.stubGlobal(
      'fetch',
      vi.fn(async (input: RequestInfo | URL) => {
        const url = String(input)
        calls.push(url)
        if (url.includes('fail.example.com')) {
          return new Response('boom', { status: 502 })
        }
        if (url.includes('unavailable.example.com')) {
          throw new TypeError('Failed to fetch')
        }
        return okResponse(new Uint8Array([1, 2, 3]).buffer)
      }) as unknown as typeof fetch,
    )
  })

  it('returns bytes from the first (best-ranked) provider', async () => {
    const retrievals: AssetRetrieval[] = [
      {
        component_id: 'c1',
        component_version: 'v1',
        provider_id: 'p1',
        provider_name: 'Best',
        storage_reference: 'https://best.example.com/c1.glb',
        latency_ms: 10,
        reliability_score: 0.99,
      },
      {
        component_id: 'c1',
        component_version: 'v1',
        provider_id: 'p2',
        provider_name: 'Worst',
        storage_reference: 'https://worst.example.com/c1.glb',
        latency_ms: 900,
        reliability_score: 0.1,
      },
    ]
    const data = await fetchAssetWithFallback(retrievals)
    expect(new Uint8Array(data)).toEqual(new Uint8Array([1, 2, 3]))
    expect(calls).toEqual(['https://best.example.com/c1.glb'])
  })

  it('falls back to the next provider when the first fails', async () => {
    const retrievals: AssetRetrieval[] = [
      {
        component_id: 'c1',
        component_version: 'v1',
        provider_id: 'p1',
        provider_name: 'Failing',
        storage_reference: 'https://fail.example.com/c1.glb',
        latency_ms: 10,
        reliability_score: 0.99,
      },
      {
        component_id: 'c1',
        component_version: 'v1',
        provider_id: 'p2',
        provider_name: 'Working',
        storage_reference: 'https://best.example.com/c1.glb',
        latency_ms: 500,
        reliability_score: 0.5,
      },
    ]
    const data = await fetchAssetWithFallback(retrievals)
    expect(new Uint8Array(data)).toEqual(new Uint8Array([1, 2, 3]))
    expect(calls).toEqual([
      'https://fail.example.com/c1.glb',
      'https://best.example.com/c1.glb',
    ])
  })

  it('throws when every provider fails', async () => {
    const retrievals: AssetRetrieval[] = [
      {
        component_id: 'c1',
        component_version: 'v1',
        provider_id: 'p1',
        provider_name: 'Down1',
        storage_reference: 'https://fail.example.com/c1.glb',
        latency_ms: 10,
        reliability_score: 0.99,
      },
      {
        component_id: 'c1',
        component_version: 'v1',
        provider_id: 'p2',
        provider_name: 'Down2',
        storage_reference: 'https://unavailable.example.com/c1.glb',
        latency_ms: 10,
        reliability_score: 0.5,
      },
    ]
    await expect(fetchAssetWithFallback(retrievals)).rejects.toThrow()
  })

  it('ranks before trying, so the most reliable provider is attempted first even if listed later', async () => {
    const retrievals: AssetRetrieval[] = [
      {
        component_id: 'c1',
        component_version: 'v1',
        provider_id: 'p2',
        provider_name: 'Unreliable',
        storage_reference: 'https://fail.example.com/c1.glb',
        latency_ms: 10,
        reliability_score: 0.1,
      },
      {
        component_id: 'c1',
        component_version: 'v1',
        provider_id: 'p1',
        provider_name: 'Reliable',
        storage_reference: 'https://best.example.com/c1.glb',
        latency_ms: 100,
        reliability_score: 0.99,
      },
    ]
    const data = await fetchAssetWithFallback(retrievals)
    expect(new Uint8Array(data)).toEqual(new Uint8Array([1, 2, 3]))
    expect(calls).toEqual(['https://best.example.com/c1.glb'])
  })
})
