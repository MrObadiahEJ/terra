import { describe, expect, it } from 'vitest'
import {
  bearingDeg,
  boundsCenterAndRadius,
  cameraViewBounds,
  cullComponent,
  defaultOptions,
  DEFAULT_LOD_STEP_SSE,
  DEFAULT_SSE_THRESHOLD,
  destinationPoint,
  haversineDeg,
  headingDelta,
  isOccluded,
  lodFromScreenSpaceError,
  metersPerDegreeLat,
  metersPerDegreeLon,
  normaliseHeading,
  outsideHorizontalFov,
  outsideVerticalFov,
  screenSpaceError,
  selectVisible,
  type Bounds,
  type Camera,
  type Viewport,
} from './viewport'

// ---------------------------------------------------------------------------
// Geometry
// ---------------------------------------------------------------------------

describe('metersPerDegree', () => {
  it('lat is ~111 km everywhere; lon shrinks with latitude', () => {
    const lat0 = metersPerDegreeLat(0)
    const lat45 = metersPerDegreeLat(45)
    expect(lat0).toBeGreaterThan(110_000)
    expect(lat0).toBeLessThan(112_000)
    // Meridian curvature barely changes with latitude.
    expect(lat45).toBeGreaterThan(lat0 * 0.99)
    expect(lat45).toBeLessThan(lat0 * 1.01)
    expect(metersPerDegreeLon(60)).toBeCloseTo(metersPerDegreeLat(60) * 0.5, 0)
  })
})

describe('haversineDeg', () => {
  it('same point is zero', () => {
    expect(haversineDeg(47.4, 8.5, 47.4, 8.5)).toBe(0)
  })
  it('1 degree of latitude is ~111 km', () => {
    const d = haversineDeg(0, 0, 1, 0)
    expect(d).toBeGreaterThan(110_000)
    expect(d).toBeLessThan(112_000)
  })
  it('is symmetric', () => {
    const a = haversineDeg(10, 20, 30, 40)
    const b = haversineDeg(30, 40, 10, 20)
    expect(a).toBeCloseTo(b, 6)
  })
})

describe('boundsCenterAndRadius', () => {
  it('centre is the box midpoint', () => {
    const b: Bounds = { minLon: 8, minLat: 47, maxLon: 9, maxLat: 48 }
    const c = boundsCenterAndRadius(b)
    expect(c.lon).toBe(8.5)
    expect(c.lat).toBe(47.5)
    expect(c.radiusM).toBeGreaterThan(0)
  })
})

describe('normaliseHeading / headingDelta', () => {
  it('normalises negative and >360 headings', () => {
    expect(normaliseHeading(-90)).toBe(270)
    expect(normaliseHeading(450)).toBe(90)
    expect(normaliseHeading(0)).toBe(0)
  })
  it('delta is the shortest signed difference', () => {
    expect(headingDelta(0, 90)).toBe(90)
    expect(headingDelta(0, -90)).toBe(-90)
    expect(headingDelta(350, 10)).toBe(20)
    expect(headingDelta(10, 350)).toBe(-20)
  })
})

describe('bearingDeg', () => {
  it('north is 0', () => {
    expect(bearingDeg(0, 0, 1, 0)).toBeCloseTo(0, 1)
  })
  it('east is 90', () => {
    expect(bearingDeg(0, 0, 0, 1)).toBeCloseTo(90, 1)
  })
})

describe('destinationPoint', () => {
  it('moving north 1 degree of latitude lands ~1 deg lat higher', () => {
    const d = destinationPoint(0, 0, 0, haversineDeg(0, 0, 1, 0))
    expect(d.lat).toBeCloseTo(1, 1)
    expect(d.lon).toBeCloseTo(0, 1)
  })
  it('moving east 1 degree of longitude at equator lands ~1 deg lon higher', () => {
    const d = destinationPoint(0, 0, 90, haversineDeg(0, 0, 0, 1))
    expect(d.lon).toBeCloseTo(1, 1)
    expect(d.lat).toBeCloseTo(0, 1)
  })
  it('south then north returns to the start', () => {
    const south = destinationPoint(10, 20, 180, 500_000)
    const back = destinationPoint(south.lat, south.lon, 0, 500_000)
    expect(back.lat).toBeCloseTo(10, 1)
    expect(back.lon).toBeCloseTo(20, 1)
  })
})

describe('cameraViewBounds', () => {
  const camera: Camera = {
    lon: 8.5,
    lat: 47.4,
    heightM: 1000,
    headingDeg: 0,
    pitchDeg: -45,
    rollDeg: 0,
  }
  const viewport: Viewport = { widthPx: 1920, heightPx: 1080, fovDeg: 60 }

  it('returns a valid bounds extending in the view direction', () => {
    const b = cameraViewBounds(camera, viewport)
    // Non-empty box.
    expect(b.minLon).toBeLessThan(b.maxLon)
    expect(b.minLat).toBeLessThan(b.maxLat)
    // Camera longitude is near the box centre; box extends north (heading 0).
    expect(b.minLon).toBeLessThanOrEqual(camera.lon)
    expect(b.maxLon).toBeGreaterThanOrEqual(camera.lon)
    expect(b.maxLat).toBeGreaterThan(camera.lat)
  })
  it('grows with camera height', () => {
    const near = cameraViewBounds({ ...camera, heightM: 500 }, viewport)
    const far = cameraViewBounds({ ...camera, heightM: 5000 }, viewport)
    expect(far.maxLon - far.minLon).toBeGreaterThan(
      near.maxLon - near.minLon,
    )
    expect(far.maxLat - far.minLat).toBeGreaterThan(
      near.maxLat - near.minLat,
    )
  })
})

// ---------------------------------------------------------------------------
// Screen-space error
// ---------------------------------------------------------------------------

describe('screenSpaceError', () => {
  const camera: Camera = {
    lon: 8.5,
    lat: 47.4,
    heightM: 1000,
    headingDeg: 0,
    pitchDeg: -45,
    rollDeg: 0,
  }
  const viewport: Viewport = { widthPx: 1920, heightPx: 1080, fovDeg: 60 }

  it('grows with feature radius', () => {
    expect(screenSpaceError(100, camera, viewport)).toBeLessThan(
      screenSpaceError(1000, camera, viewport),
    )
  })
  it('shrinks with camera height', () => {
    const near = screenSpaceError(100, { ...camera, heightM: 500 }, viewport)
    const far = screenSpaceError(100, { ...camera, heightM: 5000 }, viewport)
    expect(far).toBeLessThan(near)
  })
})

describe('lodFromScreenSpaceError', () => {
  const camera: Camera = {
    lon: 8.5,
    lat: 47.4,
    heightM: 1000,
    headingDeg: 0,
    pitchDeg: -45,
    rollDeg: 0,
  }
  const viewport: Viewport = { widthPx: 1920, heightPx: 1080, fovDeg: 60 }

  it('returns minLod when SSE is below threshold', () => {
    const sse = screenSpaceError(1, camera, viewport)
    const lod = lodFromScreenSpaceError(sse, {
      minLod: 1,
      maxLod: 4,
      sseThreshold: DEFAULT_SSE_THRESHOLD,
      lodStepSse: DEFAULT_LOD_STEP_SSE,
    })
    expect(lod).toBe(1)
  })
  it('increases LOD as SSE grows', () => {
    const small = lodFromScreenSpaceError(10, {
      minLod: 0,
      maxLod: 4,
      sseThreshold: DEFAULT_SSE_THRESHOLD,
      lodStepSse: DEFAULT_LOD_STEP_SSE,
    })
    const big = lodFromScreenSpaceError(500, {
      minLod: 0,
      maxLod: 4,
      sseThreshold: DEFAULT_SSE_THRESHOLD,
      lodStepSse: DEFAULT_LOD_STEP_SSE,
    })
    expect(big).toBeGreaterThanOrEqual(small)
    expect(big).toBeLessThanOrEqual(4)
  })
})

// ---------------------------------------------------------------------------
// Frustum / occlusion
// ---------------------------------------------------------------------------

describe('outsideHorizontalFov', () => {
  const camera: Camera = {
    lon: 8.5,
    lat: 47.4,
    heightM: 1000,
    headingDeg: 0,
    pitchDeg: -45,
    rollDeg: 0,
  }

  it('ahead is inside', () => {
    const b: Bounds = { minLon: 8.5, minLat: 47.41, maxLon: 8.51, maxLat: 47.42 }
    expect(outsideHorizontalFov(b, camera)).toBe(false)
  })
  it('far to the side is outside', () => {
    const b: Bounds = { minLon: 9, minLat: 47.4, maxLon: 9.1, maxLat: 47.5 }
    expect(outsideHorizontalFov(b, camera)).toBe(true)
  })
})

describe('outsideVerticalFov', () => {
  const camera: Camera = {
    lon: 8.5,
    lat: 47.4,
    heightM: 1000,
    headingDeg: 0,
    pitchDeg: -80, // looking near straight down
    rollDeg: 0,
  }

  it('near ground directly below is inside', () => {
    const b: Bounds = { minLon: 8.499, minLat: 47.399, maxLon: 8.501, maxLat: 47.401 }
    expect(outsideVerticalFov(b, camera)).toBe(false)
  })
})

describe('isOccluded', () => {
  const camera: Camera = {
    lon: 8.5,
    lat: 47.4,
    heightM: 1000,
    headingDeg: 0,
    pitchDeg: 0,
    rollDeg: 0,
  }

  it('behind camera is occluded', () => {
    const b: Bounds = { minLon: 8.5, minLat: 47.39, maxLon: 8.51, maxLat: 47.4 } // south
    expect(isOccluded(b, camera)).toBe(true)
  })
  it('ahead is not occluded', () => {
    const b: Bounds = { minLon: 8.5, minLat: 47.41, maxLon: 8.51, maxLat: 47.42 } // north
    expect(isOccluded(b, camera)).toBe(false)
  })
})

// ---------------------------------------------------------------------------
// Combined cull
// ---------------------------------------------------------------------------

describe('cullComponent', () => {
  const camera: Camera = {
    lon: 8.5,
    lat: 47.4,
    heightM: 1000,
    headingDeg: 0,
    pitchDeg: -45,
    rollDeg: 0,
    viewportFovDeg: 60,
  }
  const viewport: Viewport = { widthPx: 1920, heightPx: 1080, fovDeg: 60 }

  it('near ahead component is visible at low LOD', () => {
    const b: Bounds = { minLon: 8.499, minLat: 47.399, maxLon: 8.501, maxLat: 47.401 }
    const r = cullComponent('c1', b, camera, viewport)
    expect(r.visible).toBe(true)
    expect(r.reason).toBe('ok')
    expect(r.lod).toBeGreaterThanOrEqual(0)
    expect(r.distanceM).toBeLessThan(1000)
  })

  it('far side component is frustum-culled (ahead but outside cone)', () => {
    // bearing ~45 deg (north-east), ahead but outside the 30 deg half-FOV cone.
    const b: Bounds = { minLon: 8.9, minLat: 47.49, maxLon: 9.0, maxLat: 47.5 }
    const r = cullComponent('c2', b, camera, viewport)
    expect(r.visible).toBe(false)
    expect(r.reason).toBe('frustum')
  })

  it('behind camera is occlusion-culled', () => {
    const b: Bounds = { minLon: 8.5, minLat: 47.39, maxLon: 8.51, maxLat: 47.4 }
    const r = cullComponent('c3', b, camera, viewport)
    expect(r.visible).toBe(false)
    expect(r.reason).toBe('occlusion')
  })

  it('beyond max range is range-culled (ahead, in FOV)', () => {
    // ~0.05 deg lat north ≈ 5.5 km, ahead (bearing ~0) so inside FOV.
    const b: Bounds = {
      minLon: 8.4995,
      minLat: 47.4495,
      maxLon: 8.5005,
      maxLat: 47.5005,
    }
    const r = cullComponent('c4', b, camera, viewport, { maxRangeM: 5000 })
    expect(r.visible).toBe(false)
    expect(r.reason).toBe('range')
    expect(r.distanceM).toBeGreaterThan(5000)
  })
})

describe('selectVisible', () => {
  const camera: Camera = {
    lon: 8.5,
    lat: 47.4,
    heightM: 1000,
    headingDeg: 0,
    pitchDeg: -45,
    rollDeg: 0,
    viewportFovDeg: 60,
  }
  const viewport: Viewport = { widthPx: 1920, heightPx: 1080, fovDeg: 60 }

  it('sorts near-first and marks visibility', () => {
    const comps = [
      { componentId: 'far', bounds: { minLon: 9, minLat: 47.4, maxLon: 9.1, maxLat: 47.5 } },
      { componentId: 'near', bounds: { minLon: 8.499, minLat: 47.399, maxLon: 8.501, maxLat: 47.401 } },
    ]
    const res = selectVisible(comps, camera, viewport)
    expect(res[0].componentId).toBe('near')
    expect(res[1].componentId).toBe('far')
    expect(res[0].visible).toBe(true)
    expect(res.map((r) => r.componentId)).toEqual(['near', 'far'])
  })

  it('defaultOptions returns sane defaults', () => {
    const o = defaultOptions()
    expect(o.minLod).toBe(0)
    expect(o.maxLod).toBe(4)
    expect(o.sseThreshold).toBe(40)
    expect(o.maxRangeM).toBe(50_000)
    expect(o.lodStepSse).toBe(20)
  })
})
