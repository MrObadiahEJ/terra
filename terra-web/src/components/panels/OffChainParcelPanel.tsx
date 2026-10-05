import { useEffect, useMemo, useState } from 'react'
import type { OffChainParcel } from '../../lib/api'
import type { LonLat } from '../../lib/geo'
import { polygonAreaM2, polygonBbox, polygonCentroid, polygonPerimeterM, ringDigest } from '../../lib/geo'
import { Box, MapPin, Loader2 } from 'lucide-react'

interface Props {
  parcel: OffChainParcel
  onView3D?: () => void
}

function parseRing(geometry: string | null): LonLat[] | null {
  if (!geometry) return null
  try {
    const g = JSON.parse(geometry) as { type: string; coordinates: number[][][] }
    if (g.type !== 'Polygon' || !g.coordinates?.[0]) return null
    return g.coordinates[0].map(([lon, lat]) => [lon, lat] as LonLat)
  } catch {
    return null
  }
}

/** Detail view for parcels that have geometry but no on-chain account
 *  (demo seed data, local demo registrations, API-only records). */
export default function OffChainParcelPanel({ parcel, onView3D }: Props) {
  const ring = useMemo(() => parseRing(parcel.geometry), [parcel.geometry])
  const metrics = useMemo(() => {
    if (!ring || ring.length < 4) return null
    const c = polygonCentroid(ring)
    const b = polygonBbox(ring)
    const closed =
      ring.length > 1 &&
      ring[0][0] === ring[ring.length - 1][0] &&
      ring[0][1] === ring[ring.length - 1][1]
    return {
      area: parcel.area_m2 ?? polygonAreaM2(ring),
      centroid: c,
      bbox: b,
      perimeter: polygonPerimeterM(ring),
      vertices: closed ? ring.length - 1 : ring.length,
    }
  }, [ring, parcel.area_m2])

  const ringKey = useMemo(() => (ring ? JSON.stringify(ring) : ''), [ring])
  const [digestFor, setDigestFor] = useState<{ key: string; digest: string } | null>(null)
  useEffect(() => {
    if (!ring) return
    let cancelled = false
    ringDigest(ring).then((d) => {
      if (!cancelled) setDigestFor({ key: ringKey, digest: d })
    })
    return () => {
      cancelled = true
    }
  }, [ring, ringKey])
  const digest = digestFor && digestFor.key === ringKey ? digestFor.digest : null

  const isDemo = parcel.holder.startsWith('demo')

  return (
    <div className="p-3 text-sm space-y-3">
      <div className="flex items-center justify-between">
        <h3 className="font-semibold truncate">{parcel.name}</h3>
        <span className="flex items-center gap-1.5 shrink-0">
          {isDemo && (
            <span className="text-[10px] px-1.5 py-0.5 rounded bg-indigo-100 text-indigo-800">demo</span>
          )}
          <span className="text-[10px] px-1.5 py-0.5 rounded bg-emerald-50 text-emerald-800 capitalize">
            {parcel.status}
          </span>
        </span>
      </div>

      <p className="text-[11px] text-muted -mt-2">
        Off-chain geometry record — no on-chain account matched (select via holder + name).
      </p>

      <dl className="grid grid-cols-2 gap-x-4 gap-y-1 text-[12px]">
        <div className="flex justify-between">
          <dt className="text-muted">Area</dt>
          <dd className="font-medium">{metrics ? `${metrics.area.toFixed(0)} m²` : '—'}</dd>
        </div>
        <div className="flex justify-between">
          <dt className="text-muted">Perimeter</dt>
          <dd className="font-medium">{metrics ? `${Math.round(metrics.perimeter)} m` : '—'}</dd>
        </div>
        <div className="flex justify-between">
          <dt className="text-muted">Vertices</dt>
          <dd className="font-medium">{metrics ? metrics.vertices : '—'}</dd>
        </div>
        <div className="flex justify-between">
          <dt className="text-muted">Created</dt>
          <dd className="font-medium">{parcel.created_at.slice(0, 10)}</dd>
        </div>
      </dl>

      {metrics && (
        <div className="border rounded p-2 text-[11px] space-y-1">
          <h4 className="font-semibold flex items-center gap-1">
            <MapPin size={12} /> Geometry
          </h4>
          <p className="font-mono text-[11px] break-all">
            centroid: {metrics.centroid[1].toFixed(5)}, {metrics.centroid[0].toFixed(5)}
          </p>
          <p className="font-mono text-[11px] break-all">
            bbox: [{metrics.bbox.minLon.toFixed(4)}, {metrics.bbox.minLat.toFixed(4)},{' '}
            {metrics.bbox.maxLon.toFixed(4)}, {metrics.bbox.maxLat.toFixed(4)}]
          </p>
          <p className="font-mono text-[11px] break-all">
            sha256:{' '}
            {digest ? (
              <span className="text-emerald-700">{digest}</span>
            ) : (
              <span className="inline-flex items-center gap-1">
                <Loader2 size={11} className="animate-spin" /> hashing…
              </span>
            )}
          </p>
        </div>
      )}

      {onView3D && ring && ring.length >= 3 && (
        <button className="btn btn-secondary w-full justify-center" onClick={onView3D}>
          <Box size={14} /> View in 3D
        </button>
      )}

      <p className="font-mono text-[10px] text-muted break-all">holder: {parcel.holder}</p>
    </div>
  )
}
