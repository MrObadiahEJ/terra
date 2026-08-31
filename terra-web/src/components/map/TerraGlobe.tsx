import { useEffect, useRef } from 'react'
import * as Cesium from 'cesium'
import 'cesium/Build/Cesium/Widgets/widgets.css'
import { DEFAULT_FOCUS } from '../../lib/constants'
import { parseGeoJSON, type OffChainParcel, type RoadRow, type PoiRow } from '../../lib/api'

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
  onParcelClick: (id: string) => void
  focus?: { longitude: number; latitude: number; height: number } | null
}

export default function TerraGlobe({
  offChainParcels,
  roads,
  pois,
  drawing,
  drawVertices,
  onDrawVertexAdd,
  onParcelClick,
  focus,
}: TerraGlobeProps) {
  const containerRef = useRef<HTMLDivElement>(null)
  const viewerRef = useRef<Cesium.Viewer | null>(null)
  const drawingRef = useRef(false)

  // ---- init viewer ---------------------------------------------------------
  useEffect(() => {
    const container = containerRef.current
    if (!container) return

    const viewer = new Cesium.Viewer(container, {
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
        new Cesium.OpenStreetMapImageryProvider({
          url: 'https://tile.openstreetmap.org/',
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
    viewer.camera.setView({
      destination: Cesium.Cartesian3.fromDegrees(
        DEFAULT_FOCUS.longitude,
        DEFAULT_FOCUS.latitude,
        DEFAULT_FOCUS.height,
      ),
    })

    viewerRef.current = viewer

    return () => {
      viewer.destroy()
      viewerRef.current = null
    }
  }, [])

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
      if (drawingRef.current) {
        const cartesian = viewer.camera.pickEllipsoid(
          movement.position,
          viewer.scene.globe.ellipsoid,
        )
        if (!cartesian) return
        const carto = Cesium.Cartographic.fromCartesian(cartesian)
        onDrawVertexAdd({
          lon: Cesium.Math.toDegrees(carto.longitude),
          lat: Cesium.Math.toDegrees(carto.latitude),
        })
      } else {
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

  // ---- render all entities (parcels + draw + roads + pois) -----------------
  useEffect(() => {
    const viewer = viewerRef.current
    if (!viewer) return
    viewer.entities.removeAll()

    // pending draw vertices
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
          material: Cesium.Color.ORANGE.withAlpha(0.45),
          outline: true,
          outlineColor: Cesium.Color.ORANGE,
          classificationType: Cesium.ClassificationType.TERRAIN,
        },
        properties: {
          terId: parcel.id,
          terName: parcel.name,
          terStatus: parcel.status,
          terOwner: parcel.owner,
        },
        label: {
          text: parcel.name,
          font: '12px sans-serif',
          fillColor: Cesium.Color.WHITE,
          pixelOffset: new Cesium.Cartesian2(0, -18),
          disableDepthTestDistance: Number.POSITIVE_INFINITY,
          eyeOffset: new Cesium.Cartesian3(0, 0, -50),
        },
      })
    }

    // roads
    for (const road of roads) {
      const line = parseGeoJSON<{ type: string; coordinates: number[][] }>(road.geometry)
      if (!line || line.type !== 'LineString') continue
      const positions = line.coordinates.map(([lon, lat]) =>
        Cesium.Cartesian3.fromDegrees(lon, lat),
      )
      viewer.entities.add({
        id: `road-${road.id}`,
        polyline: {
          positions,
          width: 3,
          material: Cesium.Color.ROYALBLUE.withAlpha(0.8),
          clampToGround: true,
        },
        properties: { terRoad: road.name ?? road.highway },
        label: {
          text: road.name ?? road.highway,
          font: '10px sans-serif',
          fillColor: Cesium.Color.PALEGOLDENROD,
          disableDepthTestDistance: Number.POSITIVE_INFINITY,
        },
      })
    }

    // POIs
    for (const poi of pois) {
      const g = parseGeoJSON<{ type: string; coordinates: number[] }>(poi.geometry)
      if (!g) continue
      const [lon, lat] = g.coordinates
      viewer.entities.add({
        id: `poi-${poi.id}`,
        position: Cesium.Cartesian3.fromDegrees(lon, lat),
        point: { pixelSize: 9, color: Cesium.Color.RED },
        label: {
          text: poi.name ?? poi.category,
          font: '10px sans-serif',
          fillColor: Cesium.Color.WHITE,
          disableDepthTestDistance: Number.POSITIVE_INFINITY,
        },
      })
    }
  }, [offChainParcels, roads, pois, drawVertices])

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

  return <div ref={containerRef} className="w-full h-full" />
}
