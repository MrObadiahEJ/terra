import { useEffect, useRef } from 'react'
import * as L from 'leaflet'
import {
  MapContainer,
  TileLayer,
  Polygon,
  Polyline,
  CircleMarker,
  Tooltip,
  useMap,
  useMapEvents,
} from 'react-leaflet'
import 'leaflet/dist/leaflet.css'
import { DEFAULT_FOCUS } from '../../lib/constants'
import {
  parseGeoJSON,
  type OffChainParcel,
  type RoadRow,
  type PoiRow,
  type OsmBuildingFootprint,
} from '../../lib/api'
import { roadStyle, type PlannedRoad } from '../../lib/atlasLayers'
import type { DrawVertex } from './TerraGlobe'

export interface LeafletMapProps {
  offChainParcels: OffChainParcel[]
  roads: RoadRow[]
  pois: PoiRow[]
  buildings: OsmBuildingFootprint[]
  plannedRoads: PlannedRoad[]
  drawing: boolean
  drawKind: 'parcel' | 'road'
  drawVertices: DrawVertex[]
  onDrawVertexAdd: (v: DrawVertex) => void
  onDrawFinish: () => void
  onParcelClick: (id: string) => void
  onBuildingClick?: (osmId: number) => void
  focus?: { longitude: number; latitude: number; height: number; duration?: number } | null
  basemap?: 'imagery' | 'terrain' | 'osm'
  onZoom?: (zoom: number) => void
  showLabels?: boolean
  /** Bumped by the layout when a panel reflows the map container; invalidates size. */
  resizeKey?: number
}

const TILE_STYLES = {
  imagery: {
    url: 'https://server.arcgisonline.com/ArcGIS/rest/services/World_Imagery/MapServer/tile/{z}/{y}/{x}',
    maxNativeZoom: 20,
    attribution: 'Esri, Maxar, Earthstar Geographics and the GIS User Community',
  },
  terrain: {
    url: 'https://server.arcgisonline.com/ArcGIS/rest/services/World_Topographic_Map/MapServer/tile/{z}/{y}/{x}',
    maxNativeZoom: 19,
    attribution: 'Esri, HERE, Garmin, FAO, NOAA, USGS and others',
  },
  osm: {
    url: 'https://tile.openstreetmap.org/{z}/{x}/{y}.png',
    maxNativeZoom: 19,
    attribution: '© OpenStreetMap contributors',
  },
} as const

const CENTER: [number, number] = [DEFAULT_FOCUS.latitude, DEFAULT_FOCUS.longitude]
const DEFAULT_ZOOM = 13

function ClickHandler({
  drawing,
  zoomLock,
  onVertexAdd,
  onFinish,
  suppressUntilRef,
}: {
  drawing: boolean
  zoomLock: boolean
  onVertexAdd: (v: DrawVertex) => void
  onFinish: () => void
  suppressUntilRef: { current: number }
}) {
  const map = useMap()
  const lastClickRef = useRef<{ t: number; x: number; y: number } | null>(null)

  // Lock double-click zoom while a draw is in progress (or a ring is pending),
  // so closing the ring never zooms the map underneath the cursor.
  useEffect(() => {
    if (zoomLock) map.doubleClickZoom.disable()
    else map.doubleClickZoom.enable()
  }, [zoomLock, map])

  useMapEvents({
    click(e: { containerPoint: { x: number; y: number }; latlng: { lat: number; lng: number } }) {
      if (!drawing) return
      const now = performance.now()
      const prev = lastClickRef.current
      lastClickRef.current = { t: now, x: e.containerPoint.x, y: e.containerPoint.y }
      // Second physical click of a double click closes the ring.
      if (
        prev &&
        now - prev.t < 300 &&
        Math.hypot(e.containerPoint.x - prev.x, e.containerPoint.y - prev.y) < 6
      ) {
        suppressUntilRef.current = now + 400
        onFinish()
        return
      }
      onVertexAdd({ lon: e.latlng.lng, lat: e.latlng.lat })
    },
  })
  return null
}

function ResizeInvalidator({ resizeKey }: { resizeKey: number }) {
  const map = useMap()
  useEffect(() => {
    map.invalidateSize()
  }, [map, resizeKey])
  return null
}

/** Default zoom control sits top-left where the stats pill overlaps it. */
function ZoomBottomLeft() {
  const map = useMap()
  useEffect(() => {
    const control = L.control.zoom({ position: 'bottomleft' })
    control.addTo(map)
    return () => {
      control.remove()
    }
  }, [map])
  return null
}

function FocusController({ focus }: { focus: LeafletMapProps['focus'] }) {
  const map = useMap()
  useEffect(() => {
    if (!focus) return
    const zoom = Math.max(12, Math.min(20, Math.round(20 - Math.log2(focus.height / 60))))
    map.flyTo([focus.latitude, focus.longitude], zoom, { duration: focus.duration ?? 1.2 })
  }, [focus, map])
  return null
}

function ZoomReporter({ onZoom }: { onZoom?: (zoom: number) => void }) {
  const map = useMap()
  useEffect(() => {
    if (!onZoom) return
    const report = () => onZoom(map.getZoom())
    report()
    map.on('zoomend', report)
    return () => {
      map.off('zoomend', report)
    }
  }, [map, onZoom])
  return null
}

type Ring = [number, number][]

function parcelRing(parcel: OffChainParcel): Ring | null {
  const poly = parseGeoJSON<{ type: string; coordinates: number[][][] }>(parcel.geometry)
  if (!poly || poly.type !== 'Polygon') return null
  return poly.coordinates[0].map(([lon, lat]) => [lat, lon] as [number, number])
}

export default function LeafletMap({
  offChainParcels,
  roads,
  pois,
  buildings,
  plannedRoads,
  drawing,
  drawKind,
  drawVertices,
  onDrawVertexAdd,
  onDrawFinish,
  onParcelClick,
  onBuildingClick,
  focus,
  onZoom,
  basemap = 'imagery',
  resizeKey = 0,
  showLabels = true,
}: LeafletMapProps) {
  const tiles = TILE_STYLES[basemap]
  const suppressUntilRef = useRef(0)

  const parcels = offChainParcels
    .map((p) => ({ parcel: p, ring: parcelRing(p) }))
    .filter((x): x is { parcel: OffChainParcel; ring: Ring } => x.ring !== null)

  const drawPath: Ring = drawVertices.map((v) => [v.lat, v.lon])
  if (drawKind !== 'road' && drawVertices.length >= 3) drawPath.push(drawPath[0])

  return (
    <MapContainer
      {...{
        center: CENTER,
        zoom: DEFAULT_ZOOM,
        zoomControl: false,
        className: 'leaflet-container',
        style: { width: '100%', height: '100%' },
      }}
    >
      <ZoomBottomLeft />
      <TileLayer
        {...{
          url: tiles.url,
          maxNativeZoom: tiles.maxNativeZoom,
          maxZoom: 22,
          attribution: tiles.attribution,
        }}
      />
      <ClickHandler
        drawing={drawing}
        zoomLock={drawing || drawVertices.length > 0}
        onVertexAdd={onDrawVertexAdd}
        onFinish={onDrawFinish}
        suppressUntilRef={suppressUntilRef}
      />
      <FocusController focus={focus} />
      <ZoomReporter onZoom={onZoom} />
      <ResizeInvalidator resizeKey={resizeKey} />

      {roads.map((road) => {
        const poly = parseGeoJSON<{ type: string; coordinates: number[][] }>(road.geometry)
        if (!poly || poly.type !== 'LineString') return null
        const style = roadStyle(road.highway)
        return (
          <Polyline
            key={`road-${road.id}`}
            positions={poly.coordinates.map(([lon, lat]) => [lat, lon] as [number, number])}
            pathOptions={{ color: style.color, weight: style.width, opacity: 0.95 }}
          />
        )
      })}

      {buildings.map((building) => {
        const ring = building.geometry?.coordinates?.[0]
        if (!ring || ring.length < 4) return null
        return (
          <Polygon
            key={`bld-${building.osm_id}`}
            positions={ring.map(([lon, lat]) => [lat, lon] as [number, number])}
            pathOptions={{
              color: '#5c7387',
              fillColor: '#9fb6c9',
              fillOpacity: 0.45,
              weight: 1,
            }}
            eventHandlers={{
              click: () => {
                if (drawing) return
                if (performance.now() < suppressUntilRef.current) return
                onBuildingClick?.(building.osm_id)
              },
            }}
          >
            {showLabels && building.name ? <Tooltip>{building.name}</Tooltip> : null}
          </Polygon>
        )
      })}

      {pois.map((poi) => {
        const poly = parseGeoJSON<{ type: string; coordinates: number[] }>(poi.geometry)
        if (!poly || poly.type !== 'Point') return null
        const [lon, lat] = poly.coordinates
        if (lon == null || lat == null) return null
        return (
          <CircleMarker
            key={`poi-${poi.id}`}
            {...{
              center: [lat, lon],
              radius: 5,
              pathOptions: {
                color: '#ffffff',
                fillColor: '#4285f4',
                fillOpacity: 0.95,
                weight: 1.5,
              },
            }}
          >
            {showLabels && poi.name ? <Tooltip>{poi.name}</Tooltip> : null}
          </CircleMarker>
        )
      })}

      {plannedRoads.map((road) => (
        <Polyline
          key={`plan-${road.id}`}
          positions={road.vertices.map((v) => [v.lat, v.lon] as [number, number])}
          pathOptions={{ color: '#2dd4bf', weight: 4, opacity: 0.95 }}
        />
      ))}

      {parcels.map(({ parcel, ring }) => (
        <Polygon
          key={parcel.id}
          positions={ring}
          pathOptions={{
            color: '#e7c86e',
            fillColor: '#8dcc98',
            fillOpacity: 0.12,
            weight: 2,
          }}
          eventHandlers={{
            click: () => {
              if (drawing) return
              if (performance.now() < suppressUntilRef.current) return
              onParcelClick(parcel.id)
            },
          }}
        >
          <Tooltip>{parcel.name}</Tooltip>
        </Polygon>
      ))}

      {drawVertices.map((v, i) => (
        <CircleMarker
          key={`draw-${i + 1}`}
          {...{
            center: [v.lat, v.lon],
            radius: 4,
            pathOptions: { color: '#000000', fillColor: '#7fff00', fillOpacity: 1, weight: 1 },
          }}
        />
      ))}
      {drawVertices.length > 1 && (
        <Polyline
          positions={drawPath}
          pathOptions={{ color: '#7fff00', weight: drawKind === 'road' ? 3 : 2, dashArray: drawKind === 'road' ? undefined : '4 6' }}
        />
      )}
    </MapContainer>
  )
}
