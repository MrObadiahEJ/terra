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
    map.flyTo([focus.latitude, focus.longitude], zoom, { duration: 1.2 })
  }, [focus, map])
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
  drawing,
  drawVertices,
  onDrawVertexAdd,
  onDrawFinish,
  onParcelClick,
  focus,
}: LeafletMapProps) {
  const suppressUntilRef = useRef(0)

  const parcels = offChainParcels
    .map((p) => ({ parcel: p, ring: parcelRing(p) }))
    .filter((x): x is { parcel: OffChainParcel; ring: Ring } => x.ring !== null)

  const drawPath: Ring = drawVertices.map((v) => [v.lat, v.lon])
  if (drawVertices.length >= 3) drawPath.push(drawPath[0])

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
          url: 'https://server.arcgisonline.com/ArcGIS/rest/services/World_Imagery/MapServer/tile/{z}/{y}/{x}',
          maxNativeZoom: 20,
          maxZoom: 22,
          attribution: 'Esri, Maxar, Earthstar Geographics and the GIS User Community',
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
          pathOptions={{ color: '#7fff00', weight: 2, dashArray: '4 6' }}
        />
      )}
    </MapContainer>
  )
}
