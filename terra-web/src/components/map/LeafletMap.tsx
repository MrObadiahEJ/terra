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
import { parseGeoJSON, type OffChainParcel, type RoadRow, type PoiRow } from '../../lib/api'
import type { DrawVertex } from './TerraGlobe'

export interface LeafletMapProps {
  offChainParcels: OffChainParcel[]
  roads: RoadRow[]
  pois: PoiRow[]
  drawing: boolean
  drawVertices: DrawVertex[]
  onDrawVertexAdd: (v: DrawVertex) => void
  onDrawFinish: () => void
  onParcelClick: (id: string) => void
  focus?: { longitude: number; latitude: number; height: number } | null
  basemap?: 'imagery' | 'osm'
}

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
    click(e) {
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
    map.flyTo([focus.latitude, focus.longitude], DEFAULT_ZOOM, { duration: 1.2 })
  }, [focus, map])
  return null
}

type Ring = [number, number][]

function parcelRing(parcel: OffChainParcel): Ring | null {
  const poly = parseGeoJSON<{ type: string; coordinates: number[][][] }>(parcel.geometry)
  if (!poly || poly.type !== 'Polygon') return null
  return poly.coordinates[0].map(([lon, lat]) => [lat, lon] as [number, number])
}

function roadLine(road: RoadRow): Ring | null {
  const line = parseGeoJSON<{ type: string; coordinates: number[][] }>(road.geometry)
  if (!line || line.type !== 'LineString') return null
  return line.coordinates.map(([lon, lat]) => [lat, lon] as [number, number])
}

export default function LeafletMap({
  offChainParcels,
  roads,
  pois,
  drawing,
  drawVertices,
  onDrawVertexAdd,
  onDrawFinish,
  onParcelClick,
  focus,
  basemap = 'imagery',
}: LeafletMapProps) {
  const suppressUntilRef = useRef(0)

  const parcels = offChainParcels
    .map((p) => ({ parcel: p, ring: parcelRing(p) }))
    .filter((x): x is { parcel: OffChainParcel; ring: Ring } => x.ring !== null)

  const lines = roads
    .map((r) => ({ road: r, line: roadLine(r) }))
    .filter((x): x is { road: RoadRow; line: Ring } => x.line !== null)

  const drawPath: Ring = drawVertices.map((v) => [v.lat, v.lon])
  if (drawVertices.length >= 3) drawPath.push(drawPath[0])

  return (
    <MapContainer
      center={CENTER}
      zoom={DEFAULT_ZOOM}
      zoomControl={false}
      className="leaflet-container"
      style={{ width: '100%', height: '100%' }}
    >
      <ZoomBottomLeft />
      <TileLayer
        url={basemap === 'osm'
          ? 'https://tile.openstreetmap.org/{z}/{x}/{y}.png'
          : 'https://server.arcgisonline.com/ArcGIS/rest/services/World_Imagery/MapServer/tile/{z}/{y}/{x}'}
        maxNativeZoom={basemap === 'osm' ? 19 : 20}
        maxZoom={22}
        attribution={basemap === 'osm' ? '© OpenStreetMap contributors' : 'Esri, Maxar, Earthstar Geographics and the GIS User Community'}
      />
      <ClickHandler
        drawing={drawing}
        zoomLock={drawing || drawVertices.length > 0}
        onVertexAdd={onDrawVertexAdd}
        onFinish={onDrawFinish}
        suppressUntilRef={suppressUntilRef}
      />
      <FocusController focus={focus} />

      {parcels.map(({ parcel, ring }) => (
        <Polygon
          key={parcel.id}
          positions={ring}
          pathOptions={{
            color: '#f97316',
            fillColor: '#f97316',
            fillOpacity: 0.45,
            weight: 1,
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

      {lines.map(({ road, line }) => (
        <Polyline
          key={road.id}
          positions={line}
          pathOptions={{ color: '#4169e1', weight: 3, opacity: 0.8 }}
        />
      ))}

      {pois.map((poi) => {
        const g = parseGeoJSON<{ type: string; coordinates: number[] }>(poi.geometry)
        if (!g) return null
        const [lon, lat] = g.coordinates
        return (
          <CircleMarker
            key={poi.id}
            center={[lat, lon]}
            radius={5}
            pathOptions={{ color: '#dc2626', fillColor: '#dc2626', fillOpacity: 1 }}
          >
            <Tooltip>{poi.name ?? poi.category}</Tooltip>
          </CircleMarker>
        )
      })}

      {drawVertices.map((v, i) => (
        <CircleMarker
          key={`draw-${i + 1}`}
          center={[v.lat, v.lon]}
          radius={4}
          pathOptions={{ color: '#000000', fillColor: '#7fff00', fillOpacity: 1, weight: 1 }}
        />
      ))}
      {drawVertices.length > 1 && (
        <Polyline
          positions={drawPath}
          pathOptions={{ color: '#7fff00', weight: 2, dashArray: '4 6' }}
        />
      )}
    </MapContainer>
  )
}
