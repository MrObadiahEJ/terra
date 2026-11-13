// Layers panel — toggles map layers (parcels, roads, POIs, buildings, terrain).
import { MapPin, Road, Compass, Building2, Layers as LayersIcon, Satellite } from 'lucide-react'
import { useLocale, type TranslationKey } from '../../lib/locale'

interface LayerItem {
  key: string
  label: TranslationKey
  icon: React.FC<{ size?: number; className?: string }>
  description: TranslationKey
}

const LAYERS: LayerItem[] = [
  { key: 'parcels', label: 'atlasLayerParcels', icon: MapPin, description: 'atlasLayerParcelsDesc' },
  { key: 'roads', label: 'atlasLayerRoads', icon: Road, description: 'atlasLayerRoadsDesc' },
  { key: 'pois', label: 'atlasLayerPois', icon: Compass, description: 'atlasLayerPoisDesc' },
  { key: 'buildings', label: 'atlasLayerBuildings', icon: Building2, description: 'atlasLayerBuildingsDesc' },
]

export default function LayersPanel() {
  const { t } = useLocale()

  return (
    <div className="atlas-window-body">
      <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 14 }}>
        <LayersIcon size={14} className="text-muted" />
        <span style={{ fontWeight: 600, fontSize: 13, color: 'var(--ink)' }}>{t('atlasWindowLayers')}</span>
      </div>

      <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
        {LAYERS.map((layer) => (
          <div
            key={layer.key}
            style={{
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'space-between',
              padding: '8px 10px',
              border: '1px solid var(--glass-border)',
              borderRadius: 10,
              background: 'color-mix(in srgb, var(--ink) 6%, transparent)',
              cursor: 'pointer',
            }}
            onMouseEnter={(e) => { (e.currentTarget as HTMLElement).style.background = 'color-mix(in srgb, var(--ink) 12%, transparent)' }}
            onMouseLeave={(e) => { (e.currentTarget as HTMLElement).style.background = 'color-mix(in srgb, var(--ink) 6%, transparent)' }}
          >
            <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
              <layer.icon size={16} className="text-muted" />
              <span style={{ fontSize: 12.5, color: 'var(--ink)' }}>{t(layer.label)}</span>
            </div>
            <div style={{ fontSize: 10, color: 'var(--muted)', fontFamily: 'var(--font-mono)' }}>{t(layer.description)}</div>
          </div>
        ))}
      </div>

      <div style={{ marginTop: 14, paddingTop: 10, borderTop: '1px solid var(--glass-border)', display: 'flex', gap: 8 }}>
        <button type="button" className="btn btn-ghost btn-p-1">
          <Satellite size={14} /> {t('basemapSatellite')}
        </button>
        <button type="button" className="btn btn-ghost btn-p-1">
          <Satellite size={14} /> {t('basemapTerrain')}
        </button>
      </div>

      <div style={{ marginTop: 12, fontSize: 10, color: 'var(--muted)', fontFamily: 'var(--font-mono)', lineHeight: 1.4 }}>
        Layers toggle the map overlays in the main globe view. Parcels use
        bundled demo data when the off-chain API is unreachable.
      </div>
    </div>
  )
}
