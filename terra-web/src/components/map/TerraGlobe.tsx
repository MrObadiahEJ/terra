import { useEffect, useRef, useState } from 'react'
import * as Cesium from 'cesium'
import 'cesium/Build/Cesium/Widgets/widgets.css'
import { DEFAULT_FOCUS } from '../../lib/constants'
import {
  parseGeoJSON,
  type OffChainParcel,
  type RoadRow,
  type PoiRow,
  type OsmBuildingFootprint,
} from '../../lib/api'
import { buildingHeightM, roadStyle, type PlannedRoad } from '../../lib/atlasLayers'
import { FpsWatchdog, TIER_SETTINGS, type GraphicsTier } from '../../lib/perfTier'
import { resolveTerrain, type TerrainStatus } from '../../lib/cesiumTerrain'
import { type SceneNodeRow } from '../../lib/runtime'
import LeafletMap from './LeafletMap'
import ThreeAtlasFallback from './ThreeAtlasFallback'
import { DEMO_BUILDINGS } from '../../lib/demoData'
import { assetUrl } from '../../lib/assets'

// Ion stays off by default: world terrain and 3D tiles are an explicit,
// per-browser upgrade (the token lives in localStorage, never in this file).
Cesium.Ion.defaultAccessToken = ''


export interface DrawVertex {
  lon: number
  lat: number
  /** Picked surface height in metres (terrain when available, else 0). */
  h?: number
}

export interface FocusTarget {
  longitude: number
  latitude: number
  height: number
  /** Camera fly duration in seconds (defaults to 1.2). */
  duration?: number
}

type DrawKind = 'parcel' | 'road' | 'measure'

/** One-shot camera commands driven from the page (reset / north-up / tilt). */
export interface ViewCommand {
  kind: 'reset' | 'north' | 'tilt' | 'front' | 'street'
  longitude?: number
  latitude?: number
  /** Bumped per press so repeating the same command re-fires the effect. */
  nonce: number
}

interface TerraGlobeProps {
  offChainParcels: OffChainParcel[]
  roads: RoadRow[]
  pois: PoiRow[]
  buildings?: OsmBuildingFootprint[]
  plannedRoads?: PlannedRoad[]
  drawing: boolean
  drawKind?: DrawKind
  drawVertices: DrawVertex[]
  onDrawVertexAdd: (v: DrawVertex) => void
  /** Called when the user double-clicks (or clicks the first corner) to close the ring. */
  onDrawFinish: () => void
  onParcelClick: (id: string) => void
  onBuildingClick?: (osmId: number) => void
  /** Fetch an unfetched runtime scene node's component asset. */
  onFetchComponent?: (node: SceneNodeRow) => void
  viewMode: '2d' | '3d'
  focus?: FocusTarget | null
  /** Reports WebGL status upward: error message on fallback, null on success. */
  onWebGLStatus?: (msg: string | null) => void
  /** Reports camera position (lon/lat in degrees, height above ellipsoid in metres) as the view moves. */
  onCameraMoved?: (cam: { longitude: number; latitude: number; height: number }) => void
  /** Reports Leaflet zoom level when the 2D fallback map is active. */
  onMapZoom?: (zoom: number) => void
  /** Parcel/building/POI name labels follow progressive disclosure (hidden until close zoom). */
  showLabels?: boolean
  basemap?: 'imagery' | 'terrain' | 'osm'
  /** Bumped by the layout when a panel reflows the map container; forces a viewport resize. */
  resizeKey?: number
  /** Resolved graphics tier driving terrain, imagery depth, resolution and render mode. */
  graphicsTier?: GraphicsTier
  /** True while the tier preference is "auto" — only then may the FPS watchdog step down. */
  autoTier?: boolean
  /** Bumped when the ion token changes so terrain is re-resolved. */
  ionTokenRev?: number
  /** Rolling FPS report (null while the scene is idle). */
  onFps?: (fps: number | null) => void
  /** The watchdog sustained <30 fps in auto mode — the page should step the tier down. */
  onTierCapped?: () => void
  onTerrainStatus?: (status: TerrainStatus) => void
  /** Optional 3D Tiles tileset (photogrammetry model, RFC-013 Ph.C). */
  tilesetUrl?: string | null
  /** Runtime scene graph nodes (REQ-027/REQ-028): independently stored
   *  components fetched with provider fallback. Rendered as clamped
   *  footprint polygons; fetched nodes render as extruded volumes. */
  runtimeNodes?: SceneNodeRow[]
  /** IDs of nodes whose component assets have been fetched (render as volumes). */
  fetchedNodeIds?: string[]
  /** Fetched component asset bytes + transform, keyed by component ID. */
  fetchedAssets?: Map<string, {
    data: ArrayBuffer
    transform: any | null
    crs: string | null
  }>
  /** One-shot camera commands (reset view, north-up, tilt). */
  viewCommand?: ViewCommand | null
  /** Off-chain parcel the sidebar has selected — drawn with a brighter fill. */
  highlightParcelId?: string | null
  /** Building the sidebar has selected — drawn with a brighter volume. */
  highlightBuildingId?: number | null
}

function basemapProvider(
  style: 'imagery' | 'terrain' | 'osm',
  maxLevel: number,
): Cesium.UrlTemplateImageryProvider {
  if (style === 'osm') {
    return new Cesium.UrlTemplateImageryProvider({
      url: 'https://tile.openstreetmap.org/{z}/{x}/{y}.png',
      maximumLevel: Math.min(19, maxLevel),
      credit: '© OpenStreetMap contributors',
    })
  }
  if (style === 'terrain') {
    return new Cesium.UrlTemplateImageryProvider({
      url: 'https://tile.opentopomap.org/{z}/{x}/{y}.png',
      maximumLevel: Math.min(19, maxLevel),
      credit: '© OpenTopoMap contributors, SRTM · map data © OpenStreetMap contributors',
    })
  }
  return new Cesium.UrlTemplateImageryProvider({
    // Keyless Esri World Imagery — free satellite tiles, no API key needed.
    url: 'https://server.arcgisonline.com/ArcGIS/rest/services/World_Imagery/MapServer/tile/{z}/{y}/{x}',
    maximumLevel: Math.min(19, maxLevel),
    credit: 'Esri, Maxar, Earthstar Geographics, and the GIS User Community',
  })
}

function removeEntitiesByIdPrefix(viewer: Cesium.Viewer, prefixes: string[]): void {
  const doomed = viewer.entities.values.filter((entity) =>
    prefixes.some((prefix) => String(entity.id).startsWith(prefix)),
  )
  for (const entity of doomed) viewer.entities.remove(entity)
}

export default function TerraGlobe({
  offChainParcels,
  roads,
  pois,
  buildings = [],
  plannedRoads = [],
  drawing,
  drawKind = 'parcel',
  drawVertices,
  onDrawVertexAdd,
  onDrawFinish,
  onParcelClick,
  onBuildingClick,
  viewMode,
  focus,
  onWebGLStatus,
  onCameraMoved,
  onMapZoom,
  showLabels = true,
  basemap = 'imagery',
  resizeKey = 0,
  graphicsTier = 'high',
  autoTier = true,
  ionTokenRev = 0,
  onFps,
  onTierCapped,
  onTerrainStatus,
  tilesetUrl = null,
  runtimeNodes = [],
  fetchedNodeIds = [],
  fetchedAssets = new Map<string, {
    data: ArrayBuffer
    transform: any | null
    crs: string | null
  }>(),
  onFetchComponent,
  viewCommand = null,
  highlightParcelId = null,
  highlightBuildingId = null,
}: TerraGlobeProps) {
  const containerRef = useRef<HTMLDivElement>(null)
  const viewerRef = useRef<Cesium.Viewer | null>(null)
  const drawingRef = useRef(false)
  const appliedBasemapRef = useRef<string | null>(null)
  const terrainActiveRef = useRef(false)
  // Bumped whenever the terrain provider changes so effects that sample
  // terrain heights (building bases) can re-run after tiles become available.
  const [webglError, setWebglError] = useState<string | null>(null)
  const onWebGLStatusRef = useRef(onWebGLStatus)
  useEffect(() => {
    onWebGLStatusRef.current = onWebGLStatus
  })
  const onCameraMovedRef = useRef(onCameraMoved)
  useEffect(() => {
    onCameraMovedRef.current = onCameraMoved
  })
  const onDrawFinishRef = useRef(onDrawFinish)
  useEffect(() => {
    onDrawFinishRef.current = onDrawFinish
  })
  const onFpsRef = useRef(onFps)
  useEffect(() => {
    onFpsRef.current = onFps
  })
  const onTierCappedRef = useRef(onTierCapped)
  useEffect(() => {
    onTierCappedRef.current = onTierCapped
  })
  const onTerrainStatusRef = useRef(onTerrainStatus)
  useEffect(() => {
    onTerrainStatusRef.current = onTerrainStatus
  })
  const lastClickRef = useRef<{ t: number; x: number; y: number } | null>(null)
  const suppressPickRef = useRef(0)
  // The dblclick DOM event can be delayed past suppressPickRef by render
  // jank; remember that the *next* double-click belongs to the drawing
  // gesture that just finished and must not reach Cesium's zoomTo.
  const pendingDblSwallowRef = useRef(false)
  const axisDragRef = useRef<{ axis: 'X' | 'Y' | 'Z'; x: number; y: number } | null>(null)
  const onWebGLContextLost = (event: Event) => {
    event.preventDefault()
    onWebGLStatusRef.current?.('WebGL context lost. Terra is restoring the 3D view.')
  }
  const onWebGLContextRestored = () => {
    onWebGLStatusRef.current?.(null)
    viewerRef.current?.scene.requestRender()
  }

  // ---- init viewer ---------------------------------------------------------
  useEffect(() => {
    const container = containerRef.current
    if (!container) return

    let viewer: Cesium.Viewer
    try {
      viewer = new Cesium.Viewer(container, {
        contextOptions: {
          webgl: {
            alpha: false,
            depth: true,
            stencil: false,
            antialias: true,
            premultipliedAlpha: true,
            preserveDrawingBuffer: false,
            powerPreference: 'low-power',
            failIfMajorPerformanceCaveat: false,
          },
        },
        // The custom 2D/3D + basemap controls are the single source of truth;
        // Cesium's own widget chrome (scene mode, geocoder, home, fullscreen)
        // is disabled so the two never desync.
        baseLayerPicker: false,
        geocoder: false,
        homeButton: false,
        sceneModePicker: false,
        navigationHelpButton: false,
        animation: false,
        timeline: false,
        fullscreenButton: false,
        infoBox: false,
        selectionIndicator: false,
        baseLayer: new Cesium.ImageryLayer(
          basemapProvider(basemap, TIER_SETTINGS[graphicsTier].imageryMaxLevel),
        ),
        // Terrain streams in asynchronously below, gated by the graphics tier:
        // the globe starts on the flat ellipsoid and gains relief when the
        // tier allows it (keyless mesh, or ion world terrain with a token).
        terrain: undefined,
      })
      appliedBasemapRef.current = `${basemap}@${TIER_SETTINGS[graphicsTier].imageryMaxLevel}`

      viewer.scene.globe.enableLighting = false
      viewer.scene.globe.baseColor = Cesium.Color.fromCssColorString('#dfe3e1')
      viewer.scene.globe.showGroundAtmosphere = false
      if (viewer.scene.skyAtmosphere) viewer.scene.skyAtmosphere.show = false
      const navigation = viewer.scene.screenSpaceCameraController
      navigation.minimumZoomDistance = 2
      navigation.maximumZoomDistance = 50000000
      navigation.enableCollisionDetection = true
      navigation.inertiaSpin = 0.92
      navigation.inertiaTranslate = 0.85
      navigation.inertiaZoom = 0.88
      navigation.zoomFactor = 4
      viewer.camera.setView({
        destination: Cesium.Cartesian3.fromDegrees(
          DEFAULT_FOCUS.longitude,
          DEFAULT_FOCUS.latitude,
          DEFAULT_FOCUS.height,
        ),
      })
      Cesium.GeoJsonDataSource.load(assetUrl('continents.geojson'), {
        stroke: Cesium.Color.fromCssColorString('#f0ead7'),
        fill: Cesium.Color.fromCssColorString('#b9b29a').withAlpha(0.96),
        strokeWidth: 1.4,
        clampToGround: true,
      }).then((continents) => {
        if (!viewer.isDestroyed()) viewer.dataSources.add(continents)
      }).catch((error: unknown) => {
        console.warn('Bundled continent fallback could not be loaded', error)
      })
      const canvas = viewer.scene.canvas
      canvas.addEventListener('webglcontextlost', onWebGLContextLost)
      canvas.addEventListener('webglcontextrestored', onWebGLContextRestored)
    } catch (err) {
      // WebGL unavailable (no GPU / acceleration disabled / remote session).
      // Show a fallback panel instead of letting the error kill the app.
      // React reuses this same <div> for the fallback shell (both branches
      // render a div), so purge Cesium's partially-built viewer + error panel
      // — otherwise the dead widget DOM overlays the 2D map.
      containerRef.current?.replaceChildren()
      const msg = err instanceof Error ? err.message : String(err)
      queueMicrotask(() => setWebglError(msg))
      onWebGLStatusRef.current?.(msg)
      return
    }

    viewerRef.current = viewer
    onWebGLStatusRef.current?.(null)
    if (import.meta.env.DEV) {
      // Dev-only handle so headless/CDP debugging can inspect the live scene.
      ;(window as unknown as { __terraViewer?: Cesium.Viewer }).__terraViewer = viewer
      ;(window as unknown as { __terraCesium?: typeof Cesium }).__terraCesium = Cesium
    }

    let lastReported = -1
    let lastLon = Number.NaN
    let lastLat = Number.NaN
    const reportCamera = (force: boolean) => {
      const carto = viewer.camera.positionCartographic
      const h = carto?.height ?? 0
      const lon = Cesium.Math.toDegrees(carto?.longitude ?? 0)
      const lat = Cesium.Math.toDegrees(carto?.latitude ?? 0)
      // Progressive content: presentation models only stream in once the
      // camera is within 4 km of the ground (React bails out on no-ops).
      const near = h <= 4000
      nearGroundRef.current = near
      const heightStable =
        !force && lastReported >= 0 && Math.abs(h - lastReported) <= Math.max(lastReported * 0.06, 1)
      const lonlatStable =
        !Number.isNaN(lastLon) &&
        Math.abs(lon - lastLon) < 0.0004 &&
        Math.abs(lat - lastLat) < 0.0004
      if (heightStable && lonlatStable) return
      lastReported = h
      lastLon = lon
      lastLat = lat
      onCameraMovedRef.current?.({
        longitude: lon,
        latitude: lat,
        height: h,
      })
    }
    const onCamChange = () => reportCamera(false)
    const onCamMoveEnd = () => reportCamera(true)
    reportCamera(true)
    viewer.camera.changed.addEventListener(onCamChange)
    viewer.camera.moveEnd.addEventListener(onCamMoveEnd)

    return () => {
      viewer.camera.changed.removeEventListener(onCamChange)
      viewer.camera.moveEnd.removeEventListener(onCamMoveEnd)
      viewer.scene.canvas.removeEventListener('webglcontextlost', onWebGLContextLost)
      viewer.scene.canvas.removeEventListener('webglcontextrestored', onWebGLContextRestored)
      viewer.destroy()
      viewerRef.current = null
      appliedBasemapRef.current = null
      if (import.meta.env.DEV) {
        delete (window as unknown as { __terraViewer?: Cesium.Viewer }).__terraViewer
        delete (window as unknown as { __terraCesium?: typeof Cesium }).__terraCesium
      }
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps -- viewer is built once; basemap changes swap imagery below
  }, [])

  // ---- basemap: swap imagery in place (no viewer rebuild, camera persists) --
  // The key also carries the tier's imagery depth, so a tier change re-adds
  // the layer at its new maximumLevel.
  useEffect(() => {
    const viewer = viewerRef.current
    if (!viewer) return
    const key = `${basemap}@${TIER_SETTINGS[graphicsTier].imageryMaxLevel}`
    if (appliedBasemapRef.current === key) return
    appliedBasemapRef.current = key
    viewer.imageryLayers.removeAll()
    viewer.imageryLayers.addImageryProvider(
      basemapProvider(basemap, TIER_SETTINGS[graphicsTier].imageryMaxLevel),
    )
    viewer.scene.requestRender()
  }, [basemap, graphicsTier])

  // ---- tier settings: resolution, render mode, MSAA ------------------------
  useEffect(() => {
    const viewer = viewerRef.current
    if (!viewer) return
    const settings = TIER_SETTINGS[graphicsTier]
    // Viewer.resolutionScale drives CesiumWidget's render scale (Scene has no
    // public resolutionScale in the TS types even though runtime reads one).
    viewer.resolutionScale = settings.resolutionScale
    viewer.scene.requestRenderMode = settings.requestRenderMode
    try {
      viewer.scene.msaaSamples = settings.msaaSamples
    } catch {
      /* WebGL1 has no MSAA render targets */
    }
    viewer.scene.requestRender()
  }, [graphicsTier])

  // ---- terrain: resolve per tier, fall back to the flat ellipsoid ----------
  useEffect(() => {
    const viewer = viewerRef.current
    if (!viewer) return
    let cancelled = false
    if (!TIER_SETTINGS[graphicsTier].terrain) {
      viewer.terrainProvider = new Cesium.EllipsoidTerrainProvider()
      viewer.scene.globe.depthTestAgainstTerrain = false
      viewer.scene.globe.enableLighting = true
      terrainActiveRef.current = false
      onTerrainStatusRef.current?.('flat')
      viewer.scene.requestRender()
      return
    }
    onTerrainStatusRef.current?.('loading')
    resolveTerrain(graphicsTier).then((resolution) => {
      if (cancelled) return
      if (!resolution) {
        viewer.terrainProvider = new Cesium.EllipsoidTerrainProvider()
        viewer.scene.globe.depthTestAgainstTerrain = false
        viewer.scene.globe.enableLighting = true
        terrainActiveRef.current = false
        onTerrainStatusRef.current?.('flat')
        viewer.scene.requestRender()
        return
      }
      viewer.terrainProvider = resolution.provider
      viewer.scene.globe.depthTestAgainstTerrain = true
      // The keyless Re:Earth mesh ships vertex normals that Cesium's sun
      // lighting shades almost black; ion world terrain has proper normals
      // and keeps the day/night shading. (GEV also renders this mesh unlit.)
      viewer.scene.globe.enableLighting = resolution.source === 'ion'
      terrainActiveRef.current = true
      onTerrainStatusRef.current?.(resolution.source)
      viewer.scene.requestRender()
    })
    return () => {
      cancelled = true
    }
  }, [graphicsTier, ionTokenRev])

  // ---- FPS watchdog: readout always, auto step-down only in auto mode ------
  useEffect(() => {
    const viewer = viewerRef.current
    if (!viewer) return
    const watchdog = new FpsWatchdog({
      allowDowngrade: autoTier,
      onFps: (fps) => onFpsRef.current?.(fps),
      onDowngrade: () => onTierCappedRef.current?.(),
    })
    watchdog.start((cb) => {
      // Capture the scene, not the viewer: this effect's cleanup can run
      // after the viewer is destroyed, and viewer.scene would then throw.
      const scene = viewer.scene
      const handler = () => cb()
      scene.postRender.addEventListener(handler)
      return () => {
        scene.postRender.removeEventListener(handler)
      }
    })
    return () => {
      watchdog.stop()
    }
  }, [autoTier, graphicsTier])

  useEffect(() => {
    viewerRef.current?.resize()
  }, [resizeKey])

  // ---- keep drawing flag in sync -------------------------------------------
  useEffect(() => {
    drawingRef.current = drawing
  }, [drawing])

  // ---- left-click: draw vertex OR select parcel / building -----------------
  useEffect(() => {
    const viewer = viewerRef.current
    if (!viewer) return

    const handler = viewer.screenSpaceEventHandler
    const action = (movement: Cesium.ScreenSpaceEventHandler.PositionedEvent) => {
      const now = performance.now()
      pendingDblSwallowRef.current = false
      if (drawingRef.current) {
        // Second physical click of a double click: close the ring instead of
        // adding a near-duplicate vertex (pixel tolerance, not metres — picks
        // are only ~cm apart up close but metres apart when zoomed out).
        const prev = lastClickRef.current
        lastClickRef.current = { t: now, x: movement.position.x, y: movement.position.y }
        if (
          prev &&
          now - prev.t < 300 &&
          Math.hypot(movement.position.x - prev.x, movement.position.y - prev.y) < 6
        ) {
          suppressPickRef.current = now + 400
          pendingDblSwallowRef.current = true
          onDrawFinishRef.current?.()
          return
        }
        // With terrain active the ground is the mesh, so globe.pick first —
        // the analytic ellipsoid would land metres below the surface and the
        // vertex would drift at oblique angles. On the flat ellipsoid the
        // order flips: globe.pick can return a far-side hit tens of km along
        // the ray, so only trust it if the ellipsoid misses.
        const ray = viewer.camera.getPickRay(movement.position)
        let cartesian: Cesium.Cartesian3 | undefined
        if (terrainActiveRef.current) {
          if (ray) cartesian = viewer.scene.globe.pick(ray, viewer.scene)
          if (!cartesian) {
            cartesian = viewer.camera.pickEllipsoid(movement.position, viewer.scene.globe.ellipsoid)
          }
        } else {
          cartesian = viewer.camera.pickEllipsoid(movement.position, viewer.scene.globe.ellipsoid)
          if (!cartesian && ray) cartesian = viewer.scene.globe.pick(ray, viewer.scene)
        }
        if (!cartesian) return
        const carto = Cesium.Cartographic.fromCartesian(cartesian)
        onDrawVertexAdd({
          lon: Cesium.Math.toDegrees(carto.longitude),
          lat: Cesium.Math.toDegrees(carto.latitude),
          h: carto.height,
        })
      } else {
        if (now < suppressPickRef.current) return
        const picked = viewer.scene.pick(movement.position)
        const pickedId = picked?.id != null ? String(picked.id) : ''
        const demoModelMatch = pickedId.match(/^demo-house-(-?\d+)$/)
        if (demoModelMatch) {
          onBuildingClick?.(Number(demoModelMatch[1]))
          return
        }
        const entity: Cesium.Entity | undefined =
          picked && Cesium.defined(picked.id) ? (picked.id as Cesium.Entity) : undefined
        const props = entity?.properties
        const parcelId: string | undefined = props?.terId?.getValue(undefined)
        if (parcelId) {
          onParcelClick(parcelId)
          return
        }
        const bldId: number | undefined = props?.bldId?.getValue(undefined)
        if (bldId !== undefined && bldId !== null) onBuildingClick?.(Number(bldId))
        const rtNodeId: string | undefined = props?.rtNodeId?.getValue(undefined)
        if (rtNodeId !== undefined) {
          const node = runtimeNodes.find((n) => n.id === rtNodeId)
          if (node) onFetchComponent?.(node)
          return
        }
      }
    }

    handler.setInputAction(action, Cesium.ScreenSpaceEventType.LEFT_CLICK)
    return () => {
      handler.removeInputAction(Cesium.ScreenSpaceEventType.LEFT_CLICK)
    }
  }, [onDrawVertexAdd, onParcelClick, onBuildingClick, runtimeNodes, onFetchComponent])

  // ---- double-click: finish the ring while drawing (keep Cesium default) ---
  useEffect(() => {
    const viewer = viewerRef.current
    if (!viewer) return
    const handler = viewer.screenSpaceEventHandler
    const prev =
      handler.getInputAction(Cesium.ScreenSpaceEventType.LEFT_DOUBLE_CLICK) as
        | Cesium.ScreenSpaceEventHandler.PositionedEventCallback
        | undefined
    handler.setInputAction((movement: Cesium.ScreenSpaceEventHandler.PositionedEvent) => {
      if (drawingRef.current) {
        pendingDblSwallowRef.current = false
        onDrawFinishRef.current?.()
        return
      }
      if (pendingDblSwallowRef.current || performance.now() < suppressPickRef.current) {
        pendingDblSwallowRef.current = false
        return
      }
      prev?.(movement)
    }, Cesium.ScreenSpaceEventType.LEFT_DOUBLE_CLICK)
    return () => {
      if (prev) handler.setInputAction(prev, Cesium.ScreenSpaceEventType.LEFT_DOUBLE_CLICK)
      else handler.removeInputAction(Cesium.ScreenSpaceEventType.LEFT_DOUBLE_CLICK)
    }
  }, [])

  // ---- data layers: parcels, roads, POIs, planned roads -------------------
  // (split from the draw overlay so adding a vertex never rebuilds the world;
  //  buildings live in their own effect below — they sample terrain heights)
  useEffect(() => {
    const viewer = viewerRef.current
    if (!viewer) return
    removeEntitiesByIdPrefix(viewer, ['ter-', 'road-', 'poi-', 'plan-'])

    // off-chain parcels -> footprints (transparent fill, clearer outline when
    // the sidebar selects the parcel; distance-gated so a zoomed-out globe
    // never draws the whole collection at once)
    for (const parcel of offChainParcels) {
      const poly = parseGeoJSON<{ type: string; coordinates: number[][][] }>(parcel.geometry)
      if (!poly || poly.type !== 'Polygon') continue
      const ring = poly.coordinates[0]
      const hierarchy = ring.map(([lon, lat]) => Cesium.Cartesian3.fromDegrees(lon, lat))
      const selected = parcel.id === highlightParcelId
      const parcelEntity = viewer.entities.add({
        id: `ter-${parcel.id}`,
        name: parcel.name,
        polygon: {
          hierarchy: new Cesium.PolygonHierarchy(hierarchy),
          heightReference: Cesium.HeightReference.CLAMP_TO_GROUND,
          material: selected
            ? Cesium.Color.fromCssColorString('#34d399').withAlpha(0.14)
            : Cesium.Color.fromCssColorString('#b7c0c5').withAlpha(0.02),
          outline: true,
          outlineColor: selected
            ? Cesium.Color.fromCssColorString('#34d399').withAlpha(0.95)
            : Cesium.Color.fromCssColorString('#d6c98f').withAlpha(0.58),
          classificationType: Cesium.ClassificationType.TERRAIN,
        },
        properties: {
          terId: parcel.id,
          terName: parcel.name,
          terStatus: parcel.status,
          terHolder: parcel.holder,
        },
        label: {
          show: showLabels,
          text: parcel.name,
          font: '12px sans-serif',
          fillColor: Cesium.Color.WHITE,
          pixelOffset: new Cesium.Cartesian2(0, -18),
          disableDepthTestDistance: Number.POSITIVE_INFINITY,
          eyeOffset: new Cesium.Cartesian3(0, 0, -50),
          distanceDisplayCondition: new Cesium.DistanceDisplayCondition(0, 4000),
        },
      })
      if (parcelEntity.polygon) {
        parcelEntity.polygon.distanceDisplayCondition = new Cesium.ConstantProperty(
          new Cesium.DistanceDisplayCondition(0, 9000),
        )
      }
    }

    // building footprints are extruded in their own effect (terrain-aware)

    // roads -> class-coloured clamped polylines (city level and closer)
    for (const road of roads) {
      const poly = parseGeoJSON<{ type: string; coordinates: number[][] }>(road.geometry)
      if (!poly || poly.type !== 'LineString' || poly.coordinates.length < 2) continue
      const style = roadStyle(road.highway)
      viewer.entities.add({
        id: `road-${road.id}`,
        polyline: {
          positions: poly.coordinates.map(([lon, lat]) => Cesium.Cartesian3.fromDegrees(lon, lat)),
          width: style.width,
          material: Cesium.Color.fromCssColorString(style.color).withAlpha(0.95),
          clampToGround: true,
          distanceDisplayCondition: new Cesium.DistanceDisplayCondition(0, 150_000),
        },
      })
    }

    // POIs -> point markers with optional labels (region level and closer)
    for (const poi of pois) {
      const poly = parseGeoJSON<{ type: string; coordinates: number[] }>(poi.geometry)
      if (!poly || poly.type !== 'Point') continue
      const [lon, lat] = poly.coordinates
      if (lon == null || lat == null) continue
      viewer.entities.add({
        id: `poi-${poi.id}`,
        position: Cesium.Cartesian3.fromDegrees(lon, lat),
        point: {
          pixelSize: 7,
          color: Cesium.Color.fromCssColorString('#4285f4'),
          outlineColor: Cesium.Color.WHITE,
          outlineWidth: 2,
          heightReference: Cesium.HeightReference.CLAMP_TO_GROUND,
          disableDepthTestDistance: Number.POSITIVE_INFINITY,
          distanceDisplayCondition: new Cesium.DistanceDisplayCondition(0, 60_000),
        },
        label:
          showLabels && poi.name
            ? {
                text: poi.name,
                font: '11px sans-serif',
                fillColor: Cesium.Color.WHITE,
                showBackground: true,
                backgroundColor: Cesium.Color.BLACK.withAlpha(0.45),
                pixelOffset: new Cesium.Cartesian2(0, -14),
                disableDepthTestDistance: Number.POSITIVE_INFINITY,
                distanceDisplayCondition: new Cesium.DistanceDisplayCondition(0, 12_000),
              }
            : undefined,
      })
    }

    // planned road alignments -> teal open polylines
    for (const planned of plannedRoads) {
      if (planned.vertices.length < 2) continue
      viewer.entities.add({
        id: `plan-${planned.id}`,
        polyline: {
          positions: planned.vertices.map((v) => Cesium.Cartesian3.fromDegrees(v.lon, v.lat)),
          width: 4,
          material: Cesium.Color.fromCssColorString('#2dd4bf').withAlpha(0.95),
          clampToGround: true,
        },
      })
    }
  }, [offChainParcels, roads, pois, plannedRoads, showLabels, highlightParcelId])

  useEffect(() => {
    viewerRef.current?.scene.requestRender()
  }, [offChainParcels, roads, pois, plannedRoads, showLabels, drawVertices])

  // Presentation houses aligned with DEMO_BUILDINGS. Progressive by design:
  // nothing loads at startup — models stream in only once the camera is close
  // to the ground (≤ 4 km), one GLB at a time (never concurrent), staggered,
  // cancellable, and capped per graphics tier so memory stays bounded.
  const nearGroundRef = useRef(false)
  useEffect(() => {
    const viewer = viewerRef.current
    if (!viewer || !nearGroundRef.current) return
    let cancelled = false
    const primitives: Cesium.Model[] = []
    const modelUrls = [
      assetUrl('models/house_scene.glb'),
      assetUrl('models/modern-house.glb'),
    ]
    const presentationCount =
      graphicsTier === 'low' ? 2 : graphicsTier === 'medium' ? 4 : graphicsTier === 'high' ? 6 : 9
    let chain: Promise<void> = Promise.resolve()
    DEMO_BUILDINGS.slice(0, presentationCount).forEach((building, index) => {
      const ring = building.geometry.coordinates[0]
      const usable = ring.slice(0, -1)
      const lon = usable.reduce((sum, point) => sum + point[0], 0) / usable.length
      const lat = usable.reduce((sum, point) => sum + point[1], 0) / usable.length
      const centre = Cesium.Cartesian3.fromDegrees(lon, lat, 0)
      const url = modelUrls[index % modelUrls.length]
      // Single-flight queue: wait for the previous GLB (and a short breath)
      // before starting the next request.
      chain = chain.then(async () => {
        await new Promise((resolve) => window.setTimeout(resolve, 350))
        if (cancelled) return
        try {
          const model = await Cesium.Model.fromGltfAsync({
            url,
            modelMatrix: Cesium.Transforms.headingPitchRollToFixedFrame(
              centre,
              new Cesium.HeadingPitchRoll(index * Math.PI / 9, 0, 0),
            ),
            maximumScale: 1,
          })
          if (cancelled) {
            model.destroy()
            return
          }
          // Varied footprints: alternate scale and yaw so repeated models do
          // not read as a stamped row of identical houses.
          model.scale = url.endsWith('house_scene.glb')
            ? 0.48 + (index % 3) * 0.07
            : 0.24 + (index % 4) * 0.05
          model.distanceDisplayCondition = new Cesium.DistanceDisplayCondition(0, 900)
          model.id = `demo-house-${building.osm_id}`
          viewer.scene.primitives.add(model)
          primitives.push(model)
          viewer.scene.requestRender()
        } catch (error: unknown) {
          if (!cancelled) console.warn(`Failed to load demo house ${building.osm_id}`, error)
        }
      })
    })

    return () => {
      cancelled = true
      for (const model of primitives) {
        viewer.scene.primitives.remove(model)
        if (!model.isDestroyed()) model.destroy()
      }
    }
  }, [graphicsTier])

  // ---- extruded building footprints (tier-capped) -------------------------
  // Ground-relative heights clamp roof and base to the surface; manual
  // getHeight sampling was unusable (−1.8 km over a 500 m valley), so none.
  useEffect(() => {
    const viewer = viewerRef.current
    if (!viewer) return
    removeEntitiesByIdPrefix(viewer, ['bld-'])
    if (buildings.length === 0) return
    const labelsAllowed = showLabels && graphicsTier !== 'medium'
    // Ground-relative clamping only when a real terrain mesh is active — on
    // flat tiers (and while terrain tiles are still streaming) ground
    // primitives stay invisible, so extrude against the ellipsoid instead.
    const groundClamp = terrainActiveRef.current
    for (const building of buildings) {
      const ring = building.geometry?.coordinates?.[0]
      if (!ring || ring.length < 4) continue
      const hierarchy = ring.map(([lon, lat]) => Cesium.Cartesian3.fromDegrees(lon, lat))
      const selected = building.osm_id === highlightBuildingId
      viewer.entities.add({
        id: `bld-${building.osm_id}`,
        polygon: {
          hierarchy: new Cesium.PolygonHierarchy(hierarchy),
          height: 0,
          extrudedHeight: buildingHeightM(building),
          ...(groundClamp
            ? {
                heightReference: Cesium.HeightReference.RELATIVE_TO_GROUND,
                extrudedHeightReference: Cesium.HeightReference.RELATIVE_TO_GROUND,
              }
            : {}),
          material: selected
            ? Cesium.Color.fromCssColorString('#5eead4').withAlpha(0.85)
            : Cesium.Color.fromCssColorString('#9fb6c9').withAlpha(0.72),
          distanceDisplayCondition: new Cesium.DistanceDisplayCondition(0, 6000),
        },
        properties: {
          bldId: building.osm_id,
          bldName: building.name ?? '',
        },
        label:
          labelsAllowed && building.name
            ? {
                text: building.name,
                font: '11px sans-serif',
                fillColor: Cesium.Color.WHITE,
                pixelOffset: new Cesium.Cartesian2(0, -14),
                disableDepthTestDistance: Number.POSITIVE_INFINITY,
                showBackground: true,
                backgroundColor: Cesium.Color.BLACK.withAlpha(0.45),
                distanceDisplayCondition: new Cesium.DistanceDisplayCondition(0, 2500),
              }
            : undefined,
      })
    }
    viewer.scene.requestRender()
  }, [buildings, showLabels, graphicsTier, highlightBuildingId])

  // Optional 3D Tiles model (photogrammetry scaffolding, RFC-013 Ph.C).
  // The page only supplies a URL on high/ultra tiers; a failed load never
  // breaks the globe, and the primitive is destroyed on unmount/replace.
  useEffect(() => {
    const viewer = viewerRef.current
    if (!viewer || !tilesetUrl) return
    let tileset: Cesium.Cesium3DTileset | null = null
    let cancelled = false
    Cesium.Cesium3DTileset.fromUrl(tilesetUrl)
      .then((loaded) => {
        if (cancelled) {
          loaded.destroy()
          return
        }
        tileset = loaded
        viewer.scene.primitives.add(loaded)
        viewer.scene.requestRender()
      })
      .catch((err) => {
        console.warn('3D tileset load failed:', tilesetUrl, err)
      })
    return () => {
      cancelled = true
      if (tileset) {
        viewer.scene.primitives.remove(tileset)
        if (!tileset.isDestroyed()) tileset.destroy()
        viewer.scene.requestRender()
      }
    }
  }, [tilesetUrl])

  // ---- runtime scene graph nodes (REQ-027/REQ-028) -------------------------
  // Fetched components render as extruded volumes; unfetched nodes render as
  // outlines the page can fetch on click via the fetchComponent callback.
  const fetchedNodesRef = useRef<Set<string>>(new Set())
  // Sync fetched node IDs from props into the ref for render decisions.
  fetchedNodeIds.forEach((id) => fetchedNodesRef.current.add(id))
  useEffect(() => {
    const viewer = viewerRef.current
    if (!viewer) return
    removeEntitiesByIdPrefix(viewer, ['rt-'])

    for (const node of runtimeNodes) {
      const extent = node.spatial_extent
      if (!extent || extent.type !== 'Polygon') continue
      const ring = extent.coordinates?.[0]
      if (!ring || ring.length < 4) continue
      const hierarchy = ring.map(([lon, lat]: [number, number]) => Cesium.Cartesian3.fromDegrees(lon, lat))
      const isFetched = fetchedNodesRef.current.has(node.object_id)
      viewer.entities.add({
        id: `rt-${node.id}`,
        name: node.object_id,
        polygon: {
          hierarchy: new Cesium.PolygonHierarchy(hierarchy),
          height: 0,
          extrudedHeight: isFetched ? 12 : undefined,
          ...(isFetched
            ? {
                heightReference: Cesium.HeightReference.RELATIVE_TO_GROUND,
                extrudedHeightReference: Cesium.HeightReference.RELATIVE_TO_GROUND,
              }
            : {}),
          material: isFetched
            ? Cesium.Color.CYAN.withAlpha(0.5)
            : Cesium.Color.ORANGE.withAlpha(0.25),
          outline: true,
          outlineColor: isFetched
            ? Cesium.Color.CYAN
            : Cesium.Color.ORANGE,
          classificationType: Cesium.ClassificationType.TERRAIN,
        },
        properties: {
          rtNodeId: node.id,
          rtObjectId: node.object_id,
          rtComponentVersion: node.component_version_id,
          rtLoaded: isFetched,
        },
        label:
          showLabels
            ? {
                text: node.object_id.slice(0, 10),
                font: '10px sans-serif',
                fillColor: Cesium.Color.WHITE,
                pixelOffset: new Cesium.Cartesian2(0, -14),
                disableDepthTestDistance: Number.POSITIVE_INFINITY,
                showBackground: true,
                backgroundColor: isFetched
                  ? Cesium.Color.CYAN.withAlpha(0.7)
                  : Cesium.Color.ORANGE.withAlpha(0.7),
              }
            : undefined,
      })
    }
    viewer.scene.requestRender()
  }, [runtimeNodes, fetchedNodeIds, showLabels, graphicsTier])

  // ---- render fetched 3D model assets as Cesium models -----------------------
  // Each fetched component asset (GLB bytes) is rendered as a Cesium Model at
  // the node's spatial extent centroid, implementing synchronized world
  // rendering (REQ-027/REQ-028).
  const renderedModelRef = useRef<Set<string>>(new Set())
  const modelBlobUrlRef = useRef<Map<string, string>>(new Map())
  const modelPrimitiveRef = useRef<Map<string, Cesium.Model>>(new Map())
  useEffect(() => {
    const viewer = viewerRef.current
    if (!viewer) return

    for (const node of runtimeNodes) {
      if (!fetchedAssets.has(node.object_id)) continue
      if (renderedModelRef.current.has(node.object_id)) continue

      const extent = node.spatial_extent
      if (!extent || extent.type !== 'Polygon') continue
      const ring = extent.coordinates?.[0]
      if (!ring || ring.length < 4) continue

      const data = fetchedAssets.get(node.object_id)
      if (!data) continue

      // Use the component package's global_transform for placement if
      // available; otherwise fall back to the footprint centroid.
      const transform = data.transform
      let modelMatrix: Cesium.Matrix4 | undefined
      if (transform && Array.isArray(transform) && transform.length === 16) {
        modelMatrix = new Cesium.Matrix4(
          transform[0], transform[1], transform[2], transform[3],
          transform[4], transform[5], transform[6], transform[7],
          transform[8], transform[9], transform[10], transform[11],
          transform[12], transform[13], transform[14], transform[15],
        )
      } else {
        // Footprint centroid in degrees (ECEF x/y would be meaningless here).
        const lons = ring.map(([lon]: [number, number]) => lon)
        const lats = ring.map(([, lat]: [number, number]) => lat)
        modelMatrix = Cesium.Transforms.headingPitchRollToFixedFrame(
          Cesium.Cartesian3.fromDegrees(
            lons.reduce((a: number, b: number) => a + b, 0) / lons.length,
            lats.reduce((a: number, b: number) => a + b, 0) / lats.length,
            0,
          ),
          new Cesium.HeadingPitchRoll(0, 0, 0),
        )
      }

      // Create a blob URL from the GLB bytes so Cesium.Model.fromGltf can load it
      const blob = new Blob([data.data], { type: 'model/gltf-binary' })
      const url = URL.createObjectURL(blob)
      modelBlobUrlRef.current.set(node.object_id, url)

      Cesium.Model.fromGltfAsync({
        url,
        modelMatrix,
        minimumPixelSize: 1,
      })
        .then((model: Cesium.Model) => {
          if (renderedModelRef.current.has(node.object_id)) {
            model.destroy()
            return
          }
          viewer.scene.primitives.add(model)
          model.id = node.object_id
          modelPrimitiveRef.current.set(node.object_id, model)
          viewer.scene.requestRender()
        })
        .catch((err: unknown) => {
          console.warn(`Failed to load 3D model for ${node.object_id}:`, err)
          URL.revokeObjectURL(url)
        })

      renderedModelRef.current.add(node.object_id)
    }
    viewer.scene.requestRender()

    return () => {
      // Destroy rendered models and revoke blob URLs on unmount
      modelPrimitiveRef.current.forEach((model, nodeId) => {
        model.destroy()
        const url = modelBlobUrlRef.current.get(nodeId)
        if (url) URL.revokeObjectURL(url)
        modelBlobUrlRef.current.delete(nodeId)
      })
      renderedModelRef.current.clear()
      modelPrimitiveRef.current.clear()
      modelBlobUrlRef.current.clear()
    }
  }, [runtimeNodes, fetchedAssets, showLabels])


  useEffect(() => {
    const viewer = viewerRef.current
    if (!viewer) return
    removeEntitiesByIdPrefix(viewer, ['draw-'])

    drawVertices.forEach((v, i) => {
      viewer.entities.add({
        id: `draw-v-${i + 1}`,
        position: Cesium.Cartesian3.fromDegrees(v.lon, v.lat, v.h ?? 0),
        point: {
          pixelSize: 8,
          color: drawKind === 'measure' ? Cesium.Color.ORANGE : Cesium.Color.LIME,
          outlineColor: Cesium.Color.BLACK,
          outlineWidth: 2,
        },
      })
    })
    if (drawVertices.length >= 2) {
      const path = drawVertices.map((v) => Cesium.Cartesian3.fromDegrees(v.lon, v.lat, v.h ?? 0))
      // Parcels close into a ring; road alignments and measurements stay open.
      if (drawKind === 'parcel' && drawVertices.length >= 3) path.push(path[0])
      viewer.entities.add({
        id: 'draw-line',
        polyline: {
          positions: path,
          width: drawKind === 'road' ? 3 : 2,
          material:
            drawKind === 'measure'
              ? Cesium.Color.ORANGE.withAlpha(0.95)
              : Cesium.Color.LIME.withAlpha(0.9),
          clampToGround: true,
        },
      })
    }
  }, [drawVertices, drawKind])

  // ---- one-shot camera commands (reset / north-up / tilt) -------------------
  useEffect(() => {
    const viewer = viewerRef.current
    if (!viewer || !viewCommand) return
    const camera = viewer.camera
    if (viewCommand.kind === 'reset') {
      camera.flyTo({
        destination: Cesium.Cartesian3.fromDegrees(
          DEFAULT_FOCUS.longitude,
          DEFAULT_FOCUS.latitude,
          DEFAULT_FOCUS.height,
        ),
        duration: 1.0,
      })
      return
    }
    if (viewCommand.kind === 'north') {
      camera.setView({
        destination: camera.position,
        orientation: { heading: 0, pitch: camera.pitch, roll: 0 },
      })
      viewer.scene.requestRender()
      return
    }
    if (viewCommand.kind === 'front' && viewCommand.longitude != null && viewCommand.latitude != null) {
      const target = Cesium.Cartesian3.fromDegrees(viewCommand.longitude, viewCommand.latitude, 0)
      const transform = Cesium.Transforms.eastNorthUpToFixedFrame(target)
      const localDestination = new Cesium.Cartesian3(32, -18, 10)
      const destination = Cesium.Matrix4.multiplyByPoint(transform, localDestination, new Cesium.Cartesian3())
      camera.flyTo({
        destination,
        orientation: {
          heading: 0,
          pitch: Cesium.Math.toRadians(-16),
          roll: 0,
        },
        duration: 1.4,
        maximumHeight: 500,
      })
      return
    }
    if (viewCommand.kind === 'street' && viewCommand.longitude != null && viewCommand.latitude != null) {
      const target = Cesium.Cartesian3.fromDegrees(viewCommand.longitude, viewCommand.latitude)
      const transform = Cesium.Transforms.eastNorthUpToFixedFrame(target)
      const destination = Cesium.Matrix4.multiplyByPoint(
        transform,
        new Cesium.Cartesian3(12, -8, 2),
        new Cesium.Cartesian3(),
      )
      const observer = Cesium.Matrix4.multiplyByPoint(
        transform,
        new Cesium.Cartesian3(0, 0, 2),
        new Cesium.Cartesian3(),
      )
      camera.flyTo({
        destination,
        orientation: {
          direction: Cesium.Cartesian3.normalize(
            Cesium.Cartesian3.subtract(observer, destination, new Cesium.Cartesian3()),
            new Cesium.Cartesian3(),
          ),
          up: new Cesium.Cartesian3(0, 0, 1),
        },
        duration: 1.4,
        maximumHeight: 20,
      })
      return
    }
    // tilt: swing the camera to a ~45° pitch around the ground point below.
    const carto = Cesium.Cartographic.fromCartesian(camera.position)
    const target = Cesium.Cartesian3.fromRadians(carto.longitude, carto.latitude, 0)
    const distance = camera.positionCartographic?.height ?? 5_000_000
    const offset = new Cesium.HeadingPitchRange(
      camera.heading,
      Cesium.Math.toRadians(-45),
      distance,
    )
    camera.lookAt(target, offset)
    // lookAt pins the camera to the target; release so free navigation works.
    camera.lookAtTransform(Cesium.Matrix4.IDENTITY)
    viewer.scene.requestRender()
  }, [viewCommand])

  // ---- focus camera ---------------------------------------------------------
  useEffect(() => {
    const viewer = viewerRef.current
    if (!viewer || !focus) return
    viewer.camera.flyTo({
      destination: Cesium.Cartesian3.fromDegrees(
        focus.longitude,
        focus.latitude,
        focus.height,
      ),
      duration: focus.duration ?? 1.2,
    })
  }, [focus])

  useEffect(() => {
    const scene = viewerRef.current?.scene
    if (!scene) return
    let morphed = false
    if (viewMode === '2d' && scene.mode !== Cesium.SceneMode.SCENE2D) {
      scene.morphTo2D(0.7)
      morphed = true
    } else if (viewMode === '3d' && scene.mode !== Cesium.SceneMode.SCENE3D) {
      scene.morphTo3D(0.7)
      morphed = true
    }
    if (!morphed || !TIER_SETTINGS[graphicsTier].requestRenderMode) return
    // The morph animates over 0.7 s but request-render mode only paints on
    // demand; keep requesting frames for its duration so it stays smooth.
    const tick = window.setInterval(() => scene.requestRender(), 80)
    const stop = window.setTimeout(() => {
      window.clearInterval(tick)
      scene.requestRender()
    }, 900)
    return () => {
      window.clearInterval(tick)
      window.clearTimeout(stop)
    }
  }, [viewMode, graphicsTier])

  // Cesium may append its error panel asynchronously after the constructor
  // throws; keep purging leftover widget DOM so it can't overlay the 2D map.
  // (The fallback branch reuses this same div — React sees `div` → `div`.)
  useEffect(() => {
    if (!webglError) return
    const el = containerRef.current
    if (!el) return
    const clean = () => {
      el.querySelectorAll(
        ':scope > .cesium-viewer, :scope > .cesium-widget, :scope > .cesium-widget-errorPanel',
      ).forEach((n) => n.remove())
    }
    clean()
    const mo = new MutationObserver(clean)
    mo.observe(el, { childList: true })
    return () => mo.disconnect()
  }, [webglError])

  useEffect(() => {
    const keys = new Set<string>()
    const isTypingTarget = (target: EventTarget | null) =>
      target instanceof HTMLInputElement ||
      target instanceof HTMLTextAreaElement ||
      target instanceof HTMLSelectElement ||
      (target instanceof HTMLElement && target.isContentEditable)

    const onKeyDown = (event: KeyboardEvent) => {
      if (isTypingTarget(event.target)) return
      if (['KeyW', 'KeyA', 'KeyS', 'KeyD', 'KeyQ', 'KeyE'].includes(event.code)) {
        event.preventDefault()
        keys.add(event.code)
      }
    }
    const onKeyUp = (event: KeyboardEvent) => keys.delete(event.code)
    const clearKeys = () => keys.clear()

    window.addEventListener('keydown', onKeyDown)
    window.addEventListener('keyup', onKeyUp)
    window.addEventListener('blur', clearKeys)

    const moveCameraWhileNavigating = () => {
      if (keys.size === 0) return
      const viewer = viewerRef.current
      if (!viewer) return
      const camera = viewer.camera as unknown as {
        positionCartographic: Cesium.Cartographic
        moveForward: (m: number) => void
        moveBackward: (m: number) => void
        moveLeft: (m: number) => void
        moveRight: (m: number) => void
        moveDown: (m: number) => void
        moveUp: (m: number) => void
      }
      const speed = Math.max(
        1.5,
        camera.positionCartographic.height * 0.0015 * (keys.has('ShiftLeft') || keys.has('ShiftRight') ? 5 : 1),
      )
      if (keys.has('KeyW')) camera.moveForward(speed)
      if (keys.has('KeyS')) camera.moveBackward(speed)
      if (keys.has('KeyA')) camera.moveLeft(speed)
      if (keys.has('KeyD')) camera.moveRight(speed)
      if (keys.has('KeyQ')) camera.moveDown(speed)
      if (keys.has('KeyE')) camera.moveUp(speed)
    }
    viewerRef.current?.scene.postRender.addEventListener(moveCameraWhileNavigating)

    return () => {
      window.removeEventListener('keydown', onKeyDown)
      window.removeEventListener('keyup', onKeyUp)
      window.removeEventListener('blur', clearKeys)
      viewerRef.current?.scene.postRender.removeEventListener(moveCameraWhileNavigating)
    }
  }, [webglError])

  const rotateAxisDrag = (axis: 'X' | 'Y' | 'Z', dx: number, dy: number) => {
    const viewer = viewerRef.current
    if (!viewer) return
    const amount = 0.006
    if (axis === 'X') viewer.camera.rotateRight(dx * amount)
    if (axis === 'Y') viewer.camera.rotateUp(-dy * amount)
    if (axis === 'Z') viewer.camera.twistRight((dx - dy) * amount)
    viewer.scene.requestRender()
  }

  if (webglError) {
    return (
      <div className="globe-shell">
        <div className="globe-map">
          {viewMode === '3d' ? (
            <ThreeAtlasFallback
              focus={focus}
              viewCommand={viewCommand}
              offChainParcels={offChainParcels}
              buildings={buildings}
              onParcelClick={onParcelClick}
              onBuildingClick={onBuildingClick}
              highlightParcelId={highlightParcelId}
              highlightBuildingId={highlightBuildingId}
              onCameraMoved={onCameraMoved}
            />
          ) : (
            <LeafletMap
              offChainParcels={offChainParcels}
              roads={roads}
              pois={pois}
              buildings={buildings.length > 0 ? buildings : DEMO_BUILDINGS}
              plannedRoads={plannedRoads}
              drawing={drawing}
              drawKind={drawKind}
              drawVertices={drawVertices}
              onDrawVertexAdd={onDrawVertexAdd}
              onDrawFinish={onDrawFinish}
              onParcelClick={onParcelClick}
              onBuildingClick={onBuildingClick}
              focus={focus}
              onZoom={onMapZoom}
              basemap={basemap}
              resizeKey={resizeKey}
              showLabels={showLabels}
              imageryMaxLevel={TIER_SETTINGS[graphicsTier].imageryMaxLevel}
            />
          )}
        </div>
      </div>
    )
  }

  return (
    <div className="terra-3d-shell w-full h-full">
      <div ref={containerRef} className="w-full h-full" />
      <div className="terra-navigation-hud" aria-label="3D navigation controls">
        <div className="terra-axis-gizmo" aria-label="World axes">
          <svg
            viewBox="0 0 92 92"
            role="img"
            aria-label="Drag an axis to rotate the camera"
            onPointerDown={(event) => {
              const target = event.target as SVGElement
              const label = target.textContent?.trim()
              const axis = label === 'X' || label === 'Y' || label === 'Z' ? label : 'X'
              axisDragRef.current = { axis, x: event.clientX, y: event.clientY }
              event.currentTarget.setPointerCapture(event.pointerId)
            }}
            onPointerMove={(event) => {
              const drag = axisDragRef.current
              if (!drag) return
              rotateAxisDrag(drag.axis, event.clientX - drag.x, event.clientY - drag.y)
              drag.x = event.clientX
              drag.y = event.clientY
            }}
            onPointerUp={() => { axisDragRef.current = null }}
            onPointerCancel={() => { axisDragRef.current = null }}
          >
            <circle cx="46" cy="46" r="31" className="terra-axis-ring" />
            <line x1="46" y1="46" x2="46" y2="8" className="terra-axis-z" />
            <line x1="46" y1="46" x2="82" y2="61" className="terra-axis-x" />
            <line x1="46" y1="46" x2="15" y2="70" className="terra-axis-y" />
            <circle cx="46" cy="46" r="3" fill="#fff" />
            <text x="43" y="7" className="terra-axis-label terra-axis-label-z" onClick={() => {
              const viewer = viewerRef.current
              if (!viewer) return
              const canvasCenter = new Cesium.Cartesian2(
                viewer.canvas.clientWidth / 2,
                viewer.canvas.clientHeight / 2,
              )
              const target = viewer.camera.pickEllipsoid(canvasCenter, viewer.scene.globe.ellipsoid)
                ?? viewer.scene.globe.ellipsoid.cartographicToCartesian(viewer.camera.positionCartographic)
              viewer.camera.lookAtTransform(Cesium.Matrix4.IDENTITY)
              viewer.camera.lookAt(target, new Cesium.Cartesian3(0, 0, 90))
            }}>Z</text>
            <text x="83" y="68" className="terra-axis-label terra-axis-label-x" onClick={() => {
              const viewer = viewerRef.current
              if (!viewer) return
              const canvasCenter = new Cesium.Cartesian2(viewer.canvas.clientWidth / 2, viewer.canvas.clientHeight / 2)
              const target = viewer.camera.pickEllipsoid(canvasCenter, viewer.scene.globe.ellipsoid)
              if (!target) return
              viewer.camera.lookAtTransform(Cesium.Matrix4.IDENTITY)
              viewer.camera.lookAt(
                target,
                new Cesium.Cartesian3(70, 0, 25),
              )
            }}>X</text>
            <text x="5" y="78" className="terra-axis-label terra-axis-label-y" onClick={() => {
              const viewer = viewerRef.current
              if (!viewer) return
              const canvasCenter = new Cesium.Cartesian2(viewer.canvas.clientWidth / 2, viewer.canvas.clientHeight / 2)
              const target = viewer.camera.pickEllipsoid(canvasCenter, viewer.scene.globe.ellipsoid)
              if (!target) return
              viewer.camera.lookAtTransform(Cesium.Matrix4.IDENTITY)
              viewer.camera.lookAt(
                target,
                new Cesium.Cartesian3(0, 70, 25),
              )
            }}>Y</text>
          </svg>
        </div>
        <div className="terra-axis-move" aria-label="Axis move">
          {(
            [
              ['X−', 'Move west (X−)', 'west'],
              ['Y−', 'Move south (Y−)', 'south'],
              ['Z−', 'Descend (Z−)', 'down'],
              ['X+', 'Move east (X+)', 'east'],
              ['Y+', 'Move north (Y+)', 'north'],
              ['Z+', 'Ascend (Z+)', 'up'],
            ] as const
          ).map(([label, title, dir]) => (
            <button
              key={label}
              type="button"
              title={title}
              aria-label={title}
              onClick={() => {
                const viewer = viewerRef.current
                if (!viewer) return
                const step = Math.max(
                  3,
                  viewer.camera.positionCartographic.height * 0.12,
                )
                // World-ENU movement: independent of the camera heading, so
                // X/Y/Z each slide along their own axis in both directions.
                // Cesium's Camera exposes moveWest/East/South/North at runtime
                // (they pre-transform the ENU vector into the camera frame); the
                // TS types omit them, so we call through the runtime object.
                const cam = viewer.camera as unknown as {
                  moveWest: (m: number) => void
                  moveEast: (m: number) => void
                  moveSouth: (m: number) => void
                  moveNorth: (m: number) => void
                  moveDown: (m: number) => void
                  moveUp: (m: number) => void
                }
                if (dir === 'west') cam.moveWest(step)
                else if (dir === 'east') cam.moveEast(step)
                else if (dir === 'south') cam.moveSouth(step)
                else if (dir === 'north') cam.moveNorth(step)
                else if (dir === 'down') cam.moveDown(step)
                else cam.moveUp(step)
                viewer.scene.requestRender()
              }}
            >
              {label}
            </button>
          ))}
        </div>
      </div>
      <div className="terra-flight-pad" title="WASD to move · Q/E altitude · Shift faster">
        {(['W','A','S','D','Q','E'] as const).map((k) => <button key={k} onClick={() => {
          window.dispatchEvent(new KeyboardEvent('keydown', { code: `Key${k}` }))
          window.setTimeout(() => window.dispatchEvent(new KeyboardEvent('keyup', { code: `Key${k}` })), 160)
        }}>{k}</button>)}
      </div>
    </div>
  )
}