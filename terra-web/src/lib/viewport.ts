// Viewport / culling / screen-space-error helpers (Chapter 13 §13.6).
//
// Pure, dependency-free math so every function is unit-testable without
// Cesium or a renderer. All distances are approximate flat-earth / great-circle
// estimates in metres; angles are degrees.
//
//   - screenSpaceError: projected pixel size of a feature's bounding radius at
//     the current camera height / FOV. Drives distance-based LOD.
//   - frustum cull: a component is culled when its bounding box centre falls
//     outside the camera's horizontal / vertical view cone or beyond range.
//   - occlusion cull: coarse backface / behind-camera heuristic (no depth
//     buffer access in a pure module; refined client-side by Cesium).

export interface Camera {
  lon: number            // degrees
  lat: number            // degrees
  heightM: number        // metres above ellipsoid
  headingDeg: number     // 0 = north, clockwise
  pitchDeg: number       // 0 = horizon, negative = looking down
  rollDeg: number        // unused, reserved
  viewportFovDeg?: number // overrides viewport.fovDeg for culling
}

export interface Viewport {
  widthPx: number
  heightPx: number
  fovDeg: number
}

export interface Bounds {
  minLon: number
  minLat: number
  maxLon: number
  maxLat: number
}

export interface CullResult {
  componentId: string
  visible: boolean
  reason: 'frustum' | 'occlusion' | 'range' | 'ok'
  screenSpaceError: number
  lod: number
  distanceM: number
}

export interface CullOptions {
  minLod?: number
  maxLod?: number
  sseThreshold?: number        // pixels; features above this stay at high LOD
  maxRangeM?: number           // hard range cull
  lodStepSse?: number          // SSE pixels per LOD step down
}

const DEG2RAD = Math.PI / 180
const RAD2DEG = 180 / Math.PI
const EARTH_RADIUS_M = 6_371_000
const DEFAULT_FOV_DEG = 60
export const DEFAULT_SSE_THRESHOLD = 40 // pixels
export const DEFAULT_LOD_STEP_SSE = 20 // pixels per LOD step down
const DEFAULT_MAX_RANGE_M = 50_000

// ---------------------------------------------------------------------------
// Earth geometry
// ---------------------------------------------------------------------------

export function metersPerDegreeLat(latDeg: number): number {
  const lat = latDeg * DEG2RAD
  // WGS84 radius of curvature in the meridian, approx.
  return 111_132.92 - 559.82 * Math.cos(2 * lat) + 1.175 * Math.cos(4 * lat)
}

export function metersPerDegreeLon(latDeg: number): number {
  return metersPerDegreeLat(latDeg) * Math.cos(latDeg * DEG2RAD)
}

export function haversineDeg(lat1: number, lon1: number, lat2: number, lon2: number): number {
  const phi1 = lat1 * DEG2RAD
  const phi2 = lat2 * DEG2RAD
  const dPhi = (lat2 - lat1) * DEG2RAD
  const dLambda = (lon2 - lon1) * DEG2RAD
  const a =
    Math.sin(dPhi / 2) ** 2 +
    Math.cos(phi1) * Math.cos(phi2) * Math.sin(dLambda / 2) ** 2
  return 2 * EARTH_RADIUS_M * Math.asin(Math.sqrt(a))
}

// Bounding-box centre + approximate radius (metres) at the box's latitude.
export function boundsCenterAndRadius(bounds: Bounds): {
  lon: number
  lat: number
  radiusM: number
} {
  const lat = (bounds.minLat + bounds.maxLat) / 2
  const lon = (bounds.minLon + bounds.maxLon) / 2
  const halfWidthDeg = (bounds.maxLon - bounds.minLon) / 2
  const halfHeightDeg = (bounds.maxLat - bounds.minLat) / 2
  const dxM = halfWidthDeg * metersPerDegreeLon(lat)
  const dyM = halfHeightDeg * metersPerDegreeLat(lat)
  const radiusM = Math.hypot(dxM, dyM)
  return { lon, lat, radiusM: Math.max(radiusM, 1) }
}

// ---------------------------------------------------------------------------
// Screen-space error
// ---------------------------------------------------------------------------

/** Projected screen size (pixels) of a feature of `radiusM` at `heightM`. */
export function screenSpaceError(
  radiusM: number,
  camera: Camera,
  viewport: Viewport,
): number {
  const fov = (viewport.fovDeg || DEFAULT_FOV_DEG) * DEG2RAD
  const distanceM = Math.max(camera.heightM, 1)
  // Projected size ≈ radius / distance * viewportHeight / (2 * tan(fov/2)).
  const pixelsPerMetre =
    viewport.heightPx / (2 * distanceM * Math.tan(fov / 2))
  return radiusM * pixelsPerMetre
}

/**
 * Pick a LOD from screen-space error: the larger the projected SSE, the higher
 * the LOD (more detail). Clamped to [minLod, maxLod].
 */
export function lodFromScreenSpaceError(
  sse: number,
  opts: {
    minLod?: number
    maxLod?: number
    sseThreshold?: number
    lodStepSse?: number
  } = {},
): number {
  const minLod = opts.minLod ?? 0
  const maxLod = opts.maxLod ?? 4
  const threshold = opts.sseThreshold ?? DEFAULT_SSE_THRESHOLD
  const step = opts.lodStepSse ?? DEFAULT_LOD_STEP_SSE

  let lod = minLod
  let sseLeft = Math.max(sse, 0)
  while (
    lod < maxLod &&
    sseLeft > threshold + (lod - minLod) * step
  ) {
    lod += 1
    sseLeft -= step
  }
  return lod
}

// ---------------------------------------------------------------------------
// Frustum / occlusion culling
// ---------------------------------------------------------------------------

/** Normalise a heading to [0, 360). */
export function normaliseHeading(deg: number): number {
  return ((deg % 360) + 360) % 360
}

/** Smallest signed difference from `from` to `to` in degrees, [-180, 180]. */
export function headingDelta(fromDeg: number, toDeg: number): number {
  let d = normaliseHeading(toDeg) - normaliseHeading(fromDeg)
  while (d > 180) d -= 360
  while (d < -180) d += 360
  return d
}

/** Bearing (degrees clockwise from north) from (lat1,lon1) to (lat2,lon2). */
export function bearingDeg(lat1: number, lon1: number, lat2: number, lon2: number): number {
  const phi1 = lat1 * DEG2RAD
  const phi2 = lat2 * DEG2RAD
  const dLambda = (lon2 - lon1) * DEG2RAD
  const y = Math.sin(dLambda) * Math.cos(phi2)
  const x =
    Math.cos(phi1) * Math.sin(phi2) -
    Math.sin(phi1) * Math.cos(phi2) * Math.cos(dLambda)
  return normaliseHeading((Math.atan2(y, x) * RAD2DEG + 360) % 360)
}

/** Destination point given a start, a bearing (degrees clockwise from north)
 * and a great-circle distance in metres. Wraps longitude to [-180, 180]. */
export function destinationPoint(
  latDeg: number,
  lonDeg: number,
  bearing: number,
  distanceM: number,
): { lat: number; lon: number } {
  const lat1 = latDeg * DEG2RAD
  const lon1 = lonDeg * DEG2RAD
  const brg = bearing * DEG2RAD
  const d = distanceM / EARTH_RADIUS_M
  const lat2 = Math.asin(
    Math.sin(lat1) * Math.cos(d) +
      Math.cos(lat1) * Math.sin(d) * Math.cos(brg),
  )
  const lon2 =
    lon1 +
    Math.atan2(
      Math.sin(brg) * Math.sin(d) * Math.cos(lat1),
      Math.cos(d) - Math.sin(lat1) * Math.sin(lat2),
    )
  return {
    lat: lat2 * RAD2DEG,
    lon: ((lon2 * RAD2DEG + 540) % 360) - 180,
  }
}

/** Project the camera frustum to a ground bounding box (Chapter 13 §13.3:
 * visible frustum → spatial partitions). Uses the camera heading as the view
 * centre, the FOV as the left/right extent, and pitch-derived near/far
 * distances along the look direction. */
export function cameraViewBounds(
  camera: Camera,
  viewport: Viewport,
): Bounds {
  const fov = (viewport.fovDeg || DEFAULT_FOV_DEG) * DEG2RAD
  const halfFov = fov / 2
  const h = Math.max(camera.heightM, 10)
  const pitch = Math.abs(camera.pitchDeg) * DEG2RAD
  const centerDistM = h / Math.max(Math.tan(pitch), 0.05)
  const nearDistM = centerDistM * 0.5
  const farDistM = centerDistM * 2
  const brgCenter = normaliseHeading(camera.headingDeg)
  const corners = [
    [nearDistM, -halfFov],
    [nearDistM, +halfFov],
    [farDistM, -halfFov],
    [farDistM, +halfFov],
  ]
  let minLon = Infinity
  let minLat = Infinity
  let maxLon = -Infinity
  let maxLat = -Infinity
  for (const [distM, bearingOffset] of corners) {
    const brg = normaliseHeading(brgCenter + bearingOffset * RAD2DEG)
    const dest = destinationPoint(camera.lat, camera.lon, brg, distM)
    minLon = Math.min(minLon, dest.lon)
    maxLon = Math.max(maxLon, dest.lon)
    minLat = Math.min(minLat, dest.lat)
    maxLat = Math.max(maxLat, dest.lat)
  }
  return { minLon, minLat, maxLon, maxLat }
}

/** True when `bounds` centre is outside the camera's horizontal view cone. */
export function outsideHorizontalFov(bounds: Bounds, camera: Camera): boolean {
  const { lon, lat } = boundsCenterAndRadius(bounds)
  const brg = bearingDeg(camera.lat, camera.lon, lat, lon)
  const halfFov = (camera.viewportFovDeg ?? DEFAULT_FOV_DEG) / 2
  return Math.abs(headingDelta(camera.headingDeg, brg)) > halfFov
}

/** True when `bounds` centre is above the camera's pitch horizon. */
export function outsideVerticalFov(bounds: Bounds, camera: Camera): boolean {
  const { lon, lat, radiusM } = boundsCenterAndRadius(bounds)
  const dist = haversineDeg(camera.lat, camera.lon, lat, lon)
  if (dist < radiusM) return false // directly underneath
  const elevationDeg =
    (Math.atan2(camera.heightM, dist) * RAD2DEG -
      (Math.asin(radiusM / Math.max(dist, 1)) * RAD2DEG)) *
    Math.cos(camera.pitchDeg * DEG2RAD)
  return camera.pitchDeg > elevationDeg
}

/** True when `bounds` centre is behind the camera (backface / occlusion heuristic). */
export function isOccluded(bounds: Bounds, camera: Camera): boolean {
  const { lon, lat } = boundsCenterAndRadius(bounds)
  const brg = bearingDeg(camera.lat, camera.lon, lat, lon)
  return Math.abs(headingDelta(camera.headingDeg, brg)) > 90
}

// ---------------------------------------------------------------------------
// Combined cull + LOD selection
// ---------------------------------------------------------------------------

export function defaultOptions(): Required<CullOptions> {
  return {
    minLod: 0,
    maxLod: 4,
    sseThreshold: DEFAULT_SSE_THRESHOLD,
    maxRangeM: DEFAULT_MAX_RANGE_M,
    lodStepSse: DEFAULT_LOD_STEP_SSE,
  }
}

/**
 * Cull a single component and pick a LOD from screen-space error.
 * `camera.viewportFovDeg` may be set to override the default FOV for culling.
 */
export function cullComponent(
  componentId: string,
  bounds: Bounds,
  camera: Camera,
  viewport: Viewport,
  opts: CullOptions = {},
): CullResult {
  const o = { ...defaultOptions(), ...opts }
  const { lon, lat, radiusM } = boundsCenterAndRadius(bounds)
  const distM = haversineDeg(camera.lat, camera.lon, lat, lon)
  const sse = screenSpaceError(radiusM, camera, viewport)

  let reason: CullResult['reason'] = 'ok'
  let visible = true

  // Occlusion / backface: component lies behind the camera's look direction.
  if (isOccluded(bounds, camera)) {
    reason = 'occlusion'
    visible = false
  } else if (
    outsideHorizontalFov(bounds, camera) ||
    outsideVerticalFov(bounds, camera)
  ) {
    reason = 'frustum'
    visible = false
  } else if (distM > o.maxRangeM) {
    reason = 'range'
    visible = false
  }

  // Distance-based LOD from screen-space error: coarse views get low LOD.
  const lod = visible
    ? lodFromScreenSpaceError(sse, {
        minLod: o.minLod,
        maxLod: o.maxLod,
        sseThreshold: o.sseThreshold,
        lodStepSse: o.lodStepSse,
      })
    : o.minLod

  return { componentId, visible, reason, screenSpaceError: sse, lod, distanceM: distM }
}

/**
 * Cull + LOD-select a list of components. Returns results sorted by distance
 * (near first) with `visible` flags and per-component LOD.
 */
export function selectVisible(
  components: { componentId: string; bounds: Bounds }[],
  camera: Camera,
  viewport: Viewport,
  opts: CullOptions = {},
): CullResult[] {
  return components
    .map(({ componentId, bounds }) =>
      cullComponent(componentId, bounds, camera, viewport, opts),
    )
    .sort((a, b) => a.distanceM - b.distanceM)
}
