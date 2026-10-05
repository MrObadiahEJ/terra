import { useMemo } from 'react'
import { Box, X } from 'lucide-react'
import type { OsmBuildingFootprint } from '../../lib/api'
import { polygonAreaM2, polygonCentroid } from '../../lib/geo'
import { buildingRing } from '../../lib/atlasLayers'
import { useLocale } from '../../lib/locale'

interface Props {
  footprint: OsmBuildingFootprint
  onView3D: () => void
  onClose: () => void
}

export default function BuildingPanel({ footprint, onView3D, onClose }: Props) {
  const { t } = useLocale()
  const ring = useMemo(() => buildingRing(footprint), [footprint])
  const metrics = useMemo(() => {
    if (!ring) return null
    const closed =
      ring.length > 1 &&
      ring[0][0] === ring[ring.length - 1][0] &&
      ring[0][1] === ring[ring.length - 1][1]
    return {
      area: polygonAreaM2(ring),
      centroid: polygonCentroid(ring),
      vertices: closed ? ring.length - 1 : ring.length,
      estimate: Math.min(120, Math.max(6, Math.sqrt(polygonAreaM2(ring)) * 0.35)),
    }
  }, [ring])

  return (
    <div className="p-3 text-sm space-y-3">
      <div className="flex items-center justify-between gap-2">
        <h3 className="font-semibold truncate">
          {footprint.name ?? t('buildingFallback', { id: footprint.osm_id })}
        </h3>
        <button className="btn btn-ghost p-1 shrink-0" onClick={onClose} title={t('close')}>
          <X size={14} />
        </button>
      </div>

      <p className="text-[11px] text-muted -mt-1">{t('bldFootnote')}</p>

      <dl className="grid grid-cols-2 gap-x-4 gap-y-1 text-[12px]">
        <div className="flex justify-between">
          <dt className="text-muted">{t('labelArea')}</dt>
          <dd className="font-medium">{metrics ? `${metrics.area.toFixed(0)} m²` : '—'}</dd>
        </div>
        <div className="flex justify-between">
          <dt className="text-muted">{t('labelType')}</dt>
          <dd className="font-medium capitalize">{footprint.building || 'yes'}</dd>
        </div>
        <div className="flex justify-between">
          <dt className="text-muted">{t('labelHeightEst')}</dt>
          <dd className="font-medium">{metrics ? `~${Math.round(metrics.estimate)} m` : '—'}</dd>
        </div>
        <div className="flex justify-between">
          <dt className="text-muted">{t('labelVertices')}</dt>
          <dd className="font-medium">{metrics ? metrics.vertices : '—'}</dd>
        </div>
      </dl>

      {metrics && (
        <p className="font-mono text-[11px] text-muted break-all">
          osm: {footprint.osm_id} · centroid: {metrics.centroid[1].toFixed(5)},{' '}
          {metrics.centroid[0].toFixed(5)}
        </p>
      )}

      <button className="btn btn-primary w-full justify-center" onClick={onView3D} disabled={!ring}>
        <Box size={14} /> {t('viewIn3D')}
      </button>
      <p className="text-[10px] text-muted">{t('view3dFootnote')}</p>
    </div>
  )
}
