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
  fetchComponentArtifact,
  getQuadCell,
  getQuadCover,
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
    global_transform: null,
    crs: null,
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
        global_transform: null,
        crs: null,
      },
      {
        component_id: 'c1',
        component_version: 'v1',
        provider_id: 'p2',
        provider_name: 'Worst',
        storage_reference: 'https://worst.example.com/c1.glb',
        latency_ms: 900,
        reliability_score: 0.1,
        global_transform: null,
        crs: null,
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
        global_transform: null,
        crs: null,
      },
      {
        component_id: 'c1',
        component_version: 'v1',
        provider_id: 'p2',
        provider_name: 'Working',
        storage_reference: 'https://best.example.com/c1.glb',
        latency_ms: 500,
        reliability_score: 0.5,
        global_transform: null,
        crs: null,
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
        global_transform: null,
        crs: null,
      },
      {
        component_id: 'c1',
        component_version: 'v1',
        provider_id: 'p2',
        provider_name: 'Down2',
        storage_reference: 'https://unavailable.example.com/c1.glb',
        latency_ms: 10,
        reliability_score: 0.5,
        global_transform: null,
        crs: null,
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
        global_transform: null,
        crs: null,
      },
      {
        component_id: 'c1',
        component_version: 'v1',
        provider_id: 'p1',
        provider_name: 'Reliable',
        storage_reference: 'https://best.example.com/c1.glb',
        latency_ms: 100,
        reliability_score: 0.99,
        global_transform: null,
        crs: null,
      },
    ]
    const data = await fetchAssetWithFallback(retrievals)
    expect(new Uint8Array(data)).toEqual(new Uint8Array([1, 2, 3]))
    expect(calls).toEqual(['https://best.example.com/c1.glb'])
  })
})

// ---------------------------------------------------------------------------
// fetchComponentArtifact — data-plane content-addressed retrieval
// ---------------------------------------------------------------------------

const sha256Hex = async (data: Uint8Array): Promise<string> => {
  const digest = await crypto.subtle.digest('SHA-256', new Uint8Array(data).buffer)
  return Array.from(new Uint8Array(digest), (b) => b.toString(16).padStart(2, '0')).join('')
}

describe('fetchComponentArtifact', () => {
  beforeEach(() => {
    vi.restoreAllMocks()
  })

  it('serves bytes and verifies the declared content address', async () => {
    const bytes = new TextEncoder().encode('glTF artifact bytes')
    const hex = await sha256Hex(bytes)
    const captured: string[] = []
    vi.stubGlobal(
      'fetch',
      vi.fn(async (input: RequestInfo | URL) => {
        captured.push(String(input))
        return new Response(
          new Uint8Array(bytes),
          {
            status: 200,
            headers: new Headers({
              'Content-Type': 'model/gltf-binary',
              'x-content-sha256': `sha256:${hex}`,
            }),
          },
        )
      }) as unknown as typeof fetch,
    )

    const artifact = await fetchComponentArtifact('comp-001')
    expect(new Uint8Array(artifact.data)).toEqual(bytes)
    expect(artifact.contentType).toBe('model/gltf-binary')
    expect(artifact.contentAddress).toBe(`sha256:${hex}`)
    // The text component_id is passed as-is in the URL (not converted to a UUID).
    expect(captured[0]).toContain('/components/comp-001/artifact')
  })

  it('rejects when the fetched bytes do not match the declared digest', async () => {
    const bytes = new TextEncoder().encode('glTF artifact bytes')
    vi.stubGlobal(
      'fetch',
      vi.fn(async () => ({
        ok: true,
        status: 200,
        statusText: 'OK',
        headers: new Headers({
          'Content-Type': 'model/gltf-binary',
          'x-content-sha256': 'sha256:0000000000000000000000000000000000000000000000000000000000000000',
        }),
        arrayBuffer: async () => bytes.buffer,
      })) as unknown as typeof fetch,
    )

    await expect(fetchComponentArtifact('comp-001')).rejects.toThrow(
      /digest mismatch/,
    )
  })

  it('throws on non-OK responses', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn(async () => ({
        ok: false,
        status: 404,
        statusText: 'Not Found',
        headers: new Headers({}),
        arrayBuffer: async () => new ArrayBuffer(0),
      })) as unknown as typeof fetch,
    )

    await expect(fetchComponentArtifact('comp-001')).rejects.toThrow(/404/)
  })
})

// ---------------------------------------------------------------------------
// Quadtree spatial discovery — mocked fetch
// ---------------------------------------------------------------------------

describe('getQuadCell', () => {
  beforeEach(() => {
    vi.restoreAllMocks()
  })

  it('requests the cell for a point with default level', async () => {
    const captured: string[] = []
    vi.stubGlobal(
      'fetch',
      vi.fn(async (input: RequestInfo | URL) => {
        captured.push(String(input))
        return new Response(
          JSON.stringify({
            id: 'q/12/1234/4567',
            level: 12,
            x: 1234,
            y: 4567,
            bounds: { min_lon: 11.5, min_lat: 3.85, max_lon: 11.51, max_lat: 3.86 },
          }),
          { status: 200, headers: { 'Content-Type': 'application/json' } },
        )
      }) as unknown as typeof fetch,
    )

    const cell = await getQuadCell({ lon: 11.5, lat: 3.85 })
    expect(cell).toEqual({
      id: 'q/12/1234/4567',
      level: 12,
      x: 1234,
      y: 4567,
      bounds: { min_lon: 11.5, min_lat: 3.85, max_lon: 11.51, max_lat: 3.86 },
    })
    expect(captured[0]).toContain('/spatial/cell')
    expect(captured[0]).toContain('lon=11.5')
    expect(captured[0]).toContain('lat=3.85')
  })

  it('includes an explicit level in the query string', async () => {
    const captured: string[] = []
    vi.stubGlobal(
      'fetch',
      vi.fn(async (input: RequestInfo | URL) => {
        captured.push(String(input))
        return new Response(
          JSON.stringify({ id: 'q/8/1/2', level: 8, x: 1, y: 2, bounds: {} }),
          { status: 200, headers: { 'Content-Type': 'application/json' } },
        )
      }) as unknown as typeof fetch,
    )

    await getQuadCell({ lon: 0, lat: 0, level: 8 })
    expect(captured[0]).toContain('level=8')
  })

  it('throws on non-OK responses', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn(async () =>
        new Response('not found', { status: 404, statusText: 'Not Found' }),
      ) as unknown as typeof fetch,
    )

    await expect(getQuadCell({ lon: 0, lat: 0 })).rejects.toThrow(/404/)
  })
})

describe('getQuadCover', () => {
  beforeEach(() => {
    vi.restoreAllMocks()
  })

  it('requests cover cells for a bounds rectangle', async () => {
    const captured: string[] = []
    const body = JSON.stringify({
      level: 10,
      count: 3,
      cells: [
        { id: 'q/10/1/2', level: 10, x: 1, y: 2, bounds: {} },
        { id: 'q/10/1/3', level: 10, x: 1, y: 3, bounds: {} },
        { id: 'q/10/1/4', level: 10, x: 1, y: 4, bounds: {} },
      ],
    })
    vi.stubGlobal(
      'fetch',
      vi.fn(async (_input: RequestInfo | URL) => {
        captured.push(String(_input))
        return new Response(body, {
          status: 200,
          headers: { 'Content-Type': 'application/json' },
        })
      }) as unknown as typeof fetch,
    )

    const cover = await getQuadCover({
      min_lon: 11.49,
      min_lat: 3.84,
      max_lon: 11.53,
      max_lat: 3.88,
    })
    expect(cover.level).toBe(10)
    expect(cover.count).toBe(3)
    expect(cover.cells).toHaveLength(3)
    expect(captured[0]).toContain('/spatial/cover')
    expect(captured[0]).toContain('min_lon=11.49')
    expect(captured[0]).toContain('max_lat=3.88')
  })

  it('throws on non-OK responses', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn(async () =>
        new Response('error', { status: 500, statusText: 'Internal Error' }),
      ) as unknown as typeof fetch,
    )

    await expect(
      getQuadCover({ min_lon: 0, min_lat: 0, max_lon: 1, max_lat: 1 }),
    ).rejects.toThrow(/500/)
  })
})
