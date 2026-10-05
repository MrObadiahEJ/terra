import { useEffect, useMemo, useState } from 'react'
import { ArrowLeft, Boxes, Loader2 } from 'lucide-react'
import LandViewer from '../lab/LandViewer'
import {
  api,
  parseGeoJSON,
  type PhotogrammetryAsset,
  type PilotZone,
} from '../../lib/api'
import { polygonAreaM2, polygonBbox, polygonCentroid, ringDigest, type LonLat } from '../../lib/geo'
import { ELEVATION_SOURCES } from '../../lib/labStore'

interface Props {
  title: string
  ring: LonLat[]
  source: 'parcel' | 'building'
  onBack: () => void
}

interface BBox {
  minLon: number
  minLat: number
  maxLon: number
  maxLat: number
}

function zoneCoversZone(geometry: string | null, target: BBox): boolean {
  const poly = parseGeoJSON<{ type: string; coordinates: number[][][] }>(geometry)
  if (!poly || poly.type !== 'Polygon' || poly.coordinates[0].length < 3) return false
  let minLon = Infinity
  let minLat = Infinity
  let maxLon = -Infinity
  let maxLat = -Infinity
  for (const [lon, lat] of poly.coordinates[0]) {
    if (lon < minLon) minLon = lon
    if (lon > maxLon) maxLon = lon
    if (lat < minLat) minLat = lat
    if (lat > maxLat) maxLat = lat
  }
  return minLon <= target.maxLon && maxLon >= target.minLon &&
    minLat <= target.maxLat && maxLat >= target.minLat
}

const ESTIMATED_SOURCE = ELEVATION_SOURCES.findIndex((s) => s.id === 'ESTIMATED')

export default function RepresentationPanel({ title, ring, source, onBack }: Props) {
  const metrics = useMemo(() => {
    if (ring.length < 3) return null
    const area = polygonAreaM2(ring)
    const elevation =
      source === 'building'
        ? Math.min(120, Math.max(6, Math.sqrt(area) * 0.35))
        : Math.min(60, Math.max(3, Math.sqrt(area) * 0.12))
    return {
      area,
      centroid: polygonCentroid(ring),
      elevation,
      dimension: source === 'building' ? 2 : 1,
    }
  }, [ring, source])

  const [seedHex, setSeedHex] = useState('0'.repeat(64))
  useEffect(() => {
    let cancelled = false
    ringDigest(ring).then((digest) => {
      if (!cancelled) setSeedHex(digest)
    })
    return () => {
      cancelled = true
    }
  }, [ring])

  const bbox = useMemo(() => (ring.length >= 3 ? polygonBbox(ring) : null), [ring])
  const bboxKey = bbox ? `${bbox.minLon},${bbox.minLat},${bbox.maxLon},${bbox.maxLat}` : ''
  const [assetData, setAssetData] = useState<{ key: string; assets: PhotogrammetryAsset[] } | null>(
    null,
  )
  useEffect(() => {
    if (!bboxKey) return
    let cancelled = false
    const target = bbox
    api
      .listPilotZones()
      .then(async (zones: PilotZone[]) => {
        const hits = zones.filter((zone) => zoneCoversZone(zone.geometry, target as BBox))
        const lists = await Promise.all(
          hits.map((zone) =>
            api.listAssets(zone.id).catch(() => [] as PhotogrammetryAsset[]),
          ),
        )
        return lists.flat()
      })
      .then((assets) => {
        if (!cancelled) setAssetData({ key: bboxKey, assets })
      })
      .catch(() => {
        if (!cancelled) setAssetData({ key: bboxKey, assets: [] })
      })
    return () => {
      cancelled = true
    }
  }, [bbox, bboxKey])
  const assets = assetData && assetData.key === bboxKey ? assetData.assets : null

  if (!metrics) {
    return (
      <div className="p-3 text-sm">
        <button className="btn btn-ghost mb-2" onClick={onBack}>
          <ArrowLeft size={14} /> Back
        </button>
        <p className="text-muted">This selection has no usable geometry.</p>
      </div>
    )
  }

  return (
    <div className="p-3 text-sm space-y-3">
      <div className="flex items-center justify-between gap-2">
        <button className="btn btn-ghost p-1 shrink-0" onClick={onBack} title="Back">
          <ArrowLeft size={14} />
        </button>
        <h3 className="font-semibold truncate flex-1">{title}</h3>
        <span className="text-[10px] px-1.5 py-0.5 rounded bg-emerald-50 text-emerald-800 shrink-0">
          {source === 'building' ? 'building' : 'land parcel'}
        </span>
      </div>

      <dl className="grid grid-cols-2 gap-x-4 gap-y-1 text-[12px]">
        <div className="flex justify-between">
          <dt className="text-muted">Area</dt>
          <dd className="font-medium">{metrics.area.toFixed(0)} m²</dd>
        </div>
        <div className="flex justify-between">
          <dt className="text-muted">Relief (est.)</dt>
          <dd className="font-medium">~{Math.round(metrics.elevation)} m</dd>
        </div>
        <div className="flex justify-between col-span-2">
          <dt className="text-muted">Centroid</dt>
          <dd className="font-mono">
            {metrics.centroid[1].toFixed(5)}, {metrics.centroid[0].toFixed(5)}
          </dd>
        </div>
      </dl>

      <div className="repr-viewer">
        <LandViewer
          ring={ring}
          elevMinM={0}
          elevMaxM={metrics.elevation}
          elevationSource={ESTIMATED_SOURCE >= 0 ? ESTIMATED_SOURCE : 5}
          dimension={metrics.dimension}
          seedHex={seedHex}
        />
        <p className="text-[10px] text-muted mt-1">
          Extruded representation derived from the footprint — procedural relief, not a survey or
          mesh reconstruction. Drag to pan · wheel to zoom · 2D ↔ 3D toggle above.
        </p>
      </div>

      <div className="border rounded p-2">
        <h4 className="font-semibold text-[12px] flex items-center gap-1.5 mb-1">
          <Boxes size={13} /> Reconstruction assets
        </h4>
        {assets === null ? (
          <p className="text-[11px] text-muted flex items-center gap-1.5">
            <Loader2 size={11} className="animate-spin" /> Loading pilot-zone assets…
          </p>
        ) : assets.length === 0 ? (
          <p className="text-[11px] text-muted">
            No photogrammetry assets cover this location yet. Assets appear here once a pilot
            zone over the footprint has survey imagery, point clouds or meshes registered.
          </p>
        ) : (
          <div className="space-y-1.5">
            {assets.map((asset) => (
              <div key={asset.id} className="asset-card">
                <div className="flex items-center justify-between gap-2">
                  <span className="font-medium truncate">{asset.name}</span>
                  <span className="asset-type">{asset.asset_type}</span>
                </div>
                <p className="text-[11px] text-muted">
                  {[
                    asset.format,
                    asset.resolution_m != null ? `${asset.resolution_m} m resolution` : null,
                    asset.point_count != null ? `${asset.point_count.toLocaleString()} points` : null,
                  ]
                    .filter(Boolean)
                    .join(' · ') || 'details not recorded'}
                </p>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  )
}
