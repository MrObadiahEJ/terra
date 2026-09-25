import { useEffect } from 'react'
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
  onParcelClick: (id: string) => void
  focus?: { longitude: number; latitude: number; height: number } | null
}

const CENTER: [number, number] = [DEFAULT_FOCUS.latitude, DEFAULT_FOCUS.longitude]
const DEFAULT_ZOOM = 13

function ClickHandler({
  drawing,
  onVertexAdd,
}: {
  drawing: boolean
  onVertexAdd: (v: DrawVertex) => void
}) {
  useMapEvents({
    click(e) {
      if (!drawing) return
      onVertexAdd({ lon: e.latlng.lng, lat: e.latlng.lat })
    },
  })
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
  onParcelClick,
  focus,
}: LeafletMapProps) {
  const parcels = offChainParcels
    .map((p) => ({ parcel: p, ring: parcelRing(p) }))
    .filter((x): x is { parcel: OffChainParcel; ring: Ring } => x.ring !== null)

  const lines = roads
    .map((r) => ({ road: r, line: roadLine(r) }))
    .filter((x): x is { road: RoadRow; line: Ring } => x.line !== null)

  return (
    <MapContainer
      center={CENTER}
      zoom={DEFAULT_ZOOM}
      className="leaflet-container"
      style={{ width: '100%', height: '100%' }}
    >
      <TileLayer
        url="https://tile.openstreetmap.org/{z}/{x}/{y}.png"
        attribution="© OpenStreetMap contributors"
      />
      <ClickHandler drawing={drawing} onVertexAdd={onDrawVertexAdd} />
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
              if (!drawing) onParcelClick(parcel.id)
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
          positions={drawVertices.map((v) => [v.lat, v.lon] as [number, number])}
          pathOptions={{ color: '#7fff00', weight: 2, dashArray: '4 6' }}
        />
      )}
    </MapContainer>
  )
}
