import { useEffect, useRef, useState } from 'react'
import * as Cesium from 'cesium'
import 'cesium/Build/Cesium/Widgets/widgets.css'
import { DEFAULT_FOCUS } from '../../lib/constants'
import { parseGeoJSON, type OffChainParcel, type RoadRow, type PoiRow } from '../../lib/api'
import LeafletMap from './LeafletMap'

// Set your Cesium Ion token here to unlock World Terrain + 3D Tiles
// (e.g. photorealistic city tiles and photogrammetry mesh support).
Cesium.Ion.defaultAccessToken = ''

export interface DrawVertex {
  lon: number
  lat: number
}

interface TerraGlobeProps {
  offChainParcels: OffChainParcel[]
  roads: RoadRow[]
  pois: PoiRow[]
  drawing: boolean
  drawVertices: DrawVertex[]
  onDrawVertexAdd: (v: DrawVertex) => void
  /** Called when the user double-clicks (or clicks the first corner) to close the ring. */
  onDrawFinish: () => void
  onParcelClick: (id: string) => void
  viewMode: '2d' | '3d'
  focus?: { longitude: number; latitude: number; height: number } | null
  /** Reports WebGL status upward: error message on fallback, null on success. */
  onWebGLStatus?: (msg: string | null) => void
  /** Reports camera position (lon/lat in degrees, height above ellipsoid in metres) as the view moves. */
  onCameraMoved?: (cam: { longitude: number; latitude: number; height: number }) => void
  /** Reports Leaflet zoom level when the 2D fallback map is active. */
  onMapZoom?: (zoom: number) => void
  /** Parcel name labels follow progressive disclosure (hidden until close zoom). */
  showParcelLabels?: boolean
  basemap?: 'imagery' | 'terrain' | 'osm'
  /** Bumped by the layout when a panel reflows the map container; forces a viewport resize. */
  resizeKey?: number
}

export default function TerraGlobe({
  offChainParcels,
  drawing,
  drawVertices,
  onDrawVertexAdd,
  onDrawFinish,
  onParcelClick,
  viewMode,
  focus,
  onWebGLStatus,
  onCameraMoved,
  onMapZoom,
  showParcelLabels = true,
  basemap = 'imagery',
  resizeKey = 0,
}: TerraGlobeProps) {
  const containerRef = useRef<HTMLDivElement>(null)
  const viewerRef = useRef<Cesium.Viewer | null>(null)
  const drawingRef = useRef(false)
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
  const lastClickRef = useRef<{ t: number; x: number; y: number } | null>(null)
  const suppressPickRef = useRef(0)
  // The dblclick DOM event can be delayed past suppressPickRef by render
  // jank; remember that the *next* double-click belongs to the drawing
  // gesture that just finished and must not reach Cesium's zoomTo.
  const pendingDblSwallowRef = useRef(false)

  // ---- init viewer ---------------------------------------------------------
  useEffect(() => {
    const container = containerRef.current
    if (!container) return

    let viewer: Cesium.Viewer
    try {
      viewer = new Cesium.Viewer(container, {
        baseLayerPicker: false,
        geocoder: true,
        homeButton: true,
        sceneModePicker: true,
        navigationHelpButton: false,
        animation: false,
        timeline: false,
        fullscreenButton: true,
        infoBox: false,
        selectionIndicator: false,
        baseLayer: new Cesium.ImageryLayer(
          basemap === 'osm'
            ? new Cesium.UrlTemplateImageryProvider({
                url: 'https://tile.openstreetmap.org/{z}/{x}/{y}.png',
                maximumLevel: 19,
                credit: '© OpenStreetMap contributors',
              })
            : basemap === 'terrain'
              ? new Cesium.UrlTemplateImageryProvider({
                  url: 'https://server.arcgisonline.com/ArcGIS/rest/services/World_Topo_Map/MapServer/tile/{z}/{y}/{x}',
                  maximumLevel: 19,
                  credit: 'Esri, HERE, Garmin, FAO, NOAA, USGS and others',
                })
              : new Cesium.UrlTemplateImageryProvider({
                  url: 'https://server.arcgisonline.com/ArcGIS/rest/services/World_Imagery/MapServer/tile/{z}/{y}/{x}',
                  maximumLevel: 20,
                  credit: 'Esri, Maxar, Earthstar Geographics and the GIS User Community',
                }),
        ),
        // World Terrain requires a Cesium Ion token. If none is configured we
        // fall back to the bare ellipsoid so the globe works out of the box.
        terrain:
          Cesium.Ion.defaultAccessToken
            ? Cesium.Terrain.fromWorldTerrain()
            : undefined,
      })

      viewer.scene.globe.enableLighting = true
      viewer.scene.screenSpaceCameraController.minimumZoomDistance = 1
      viewer.camera.setView({
        destination: Cesium.Cartesian3.fromDegrees(
          DEFAULT_FOCUS.longitude,
          DEFAULT_FOCUS.latitude,
          DEFAULT_FOCUS.height,
        ),
      })
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

    let lastReported = -1
    const reportCamera = (force: boolean) => {
      const carto = viewer.camera.positionCartographic
      const h = carto?.height ?? 0
      if (!force && lastReported >= 0 && Math.abs(h - lastReported) <= Math.max(lastReported * 0.06, 1)) {
        return
      }
      lastReported = h
      onCameraMovedRef.current?.({
        longitude: Cesium.Math.toDegrees(carto?.longitude ?? 0),
        latitude: Cesium.Math.toDegrees(carto?.latitude ?? 0),
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
      viewer.destroy()
      viewerRef.current = null
    }
  }, [basemap])

  useEffect(() => {
    viewerRef.current?.resize()
  }, [resizeKey])

  // ---- keep drawing flag in sync -------------------------------------------
  useEffect(() => {
    drawingRef.current = drawing
  }, [drawing])

  // ---- left-click: draw vertex OR select parcel ----------------------------
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
        // Terrain is off (Ion token cleared above), so the analytic ellipsoid
        // pick is ground truth; globe.pick occasionally returns a far-side hit
        // tens of km along the ray, so only trust it if the ellipsoid misses.
        const ray = viewer.camera.getPickRay(movement.position)
        let cartesian = viewer.camera.pickEllipsoid(
          movement.position,
          viewer.scene.globe.ellipsoid,
        )
        if (!cartesian && ray) cartesian = viewer.scene.globe.pick(ray, viewer.scene)
        if (!cartesian) return
        const carto = Cesium.Cartographic.fromCartesian(cartesian)
        onDrawVertexAdd({
          lon: Cesium.Math.toDegrees(carto.longitude),
          lat: Cesium.Math.toDegrees(carto.latitude),
        })
      } else {
        if (now < suppressPickRef.current) return
        const picked = viewer.scene.pick(movement.position)
        const id: string | undefined =
          picked && Cesium.defined(picked.id)
            ? picked.id.properties?.terId?.getValue(undefined)
            : undefined
        if (id) onParcelClick(id)
      }
    }

    handler.setInputAction(action, Cesium.ScreenSpaceEventType.LEFT_CLICK)
    return () => {
      handler.removeInputAction(Cesium.ScreenSpaceEventType.LEFT_CLICK)
    }
  }, [onDrawVertexAdd, onParcelClick])

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

  // ---- render parcel boundaries and the active drawing ---------------------
  useEffect(() => {
    const viewer = viewerRef.current
    if (!viewer) return
    viewer.entities.removeAll()

    // pending draw vertices + ring outline (closing edge once we have 3+)
    drawVertices.forEach((v, i) => {
      viewer.entities.add({
        id: `draw-${i + 1}`,
        position: Cesium.Cartesian3.fromDegrees(v.lon, v.lat),
        point: {
          pixelSize: 8,
          color: Cesium.Color.LIME,
          outlineColor: Cesium.Color.BLACK,
          outlineWidth: 2,
        },
      })
    })
    if (drawVertices.length >= 2) {
      const path = drawVertices.map((v) => Cesium.Cartesian3.fromDegrees(v.lon, v.lat))
      if (drawVertices.length >= 3) path.push(path[0])
      viewer.entities.add({
        id: 'draw-line',
        polyline: {
          positions: path,
          width: 2,
          material: Cesium.Color.LIME.withAlpha(0.9),
          clampToGround: true,
        },
      })
    }

    // off-chain parcels -> extruded polygons
    for (const parcel of offChainParcels) {
      const poly = parseGeoJSON<{ type: string; coordinates: number[][][] }>(parcel.geometry)
      if (!poly || poly.type !== 'Polygon') continue
      const ring = poly.coordinates[0]
      const hierarchy = ring.map(([lon, lat]) => Cesium.Cartesian3.fromDegrees(lon, lat))
      viewer.entities.add({
        id: `ter-${parcel.id}`,
        name: parcel.name,
        polygon: {
          hierarchy: new Cesium.PolygonHierarchy(hierarchy),
          heightReference: Cesium.HeightReference.CLAMP_TO_GROUND,
          material: Cesium.Color.LIME.withAlpha(0.16),
          outline: true,
          outlineColor: Cesium.Color.YELLOW,
          classificationType: Cesium.ClassificationType.TERRAIN,
        },
        properties: {
          terId: parcel.id,
          terName: parcel.name,
          terStatus: parcel.status,
          terHolder: parcel.holder,
        },
        label: {
          show: showParcelLabels,
          text: parcel.name,
          font: '12px sans-serif',
          fillColor: Cesium.Color.WHITE,
          pixelOffset: new Cesium.Cartesian2(0, -18),
          disableDepthTestDistance: Number.POSITIVE_INFINITY,
          eyeOffset: new Cesium.Cartesian3(0, 0, -50),
        },
      })
    }

  }, [offChainParcels, drawVertices, showParcelLabels])

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
      duration: 1.2,
    })
  }, [focus])

  useEffect(() => {
    const scene = viewerRef.current?.scene
    if (!scene) return
    if (viewMode === '2d' && scene.mode !== Cesium.SceneMode.SCENE2D) {
      scene.morphTo2D(0.7)
    } else if (viewMode === '3d' && scene.mode !== Cesium.SceneMode.SCENE3D) {
      scene.morphTo3D(0.7)
    }
  }, [viewMode])

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

  if (webglError) {
    return (
      <div className="globe-shell">
        <div className="globe-map">
          <LeafletMap
            offChainParcels={offChainParcels}
            roads={[]}
            pois={[]}
            drawing={drawing}
            drawVertices={drawVertices}
            onDrawVertexAdd={onDrawVertexAdd}
            onDrawFinish={onDrawFinish}
            onParcelClick={onParcelClick}
            focus={focus}
            onZoom={onMapZoom}
            basemap={basemap}
            resizeKey={resizeKey}
          />
        </div>
      </div>
    )
  }

  return <div ref={containerRef} className="w-full h-full" />
}
