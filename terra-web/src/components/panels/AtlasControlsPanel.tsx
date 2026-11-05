import { useMemo, useState } from 'react'
import {
  Box,
  Check,
  Compass,
  Crosshair,
  Layers,
  MapPin,
  Move3d,
  RotateCcw,
  Ruler,
  Search,
  Square,
  Tag,
  Building2,
  Route,
} from 'lucide-react'
import { useLocale, type TranslationKey } from '../../lib/locale'
import { searchLocal, searchPlaces, type SearchResult } from '../../lib/search'
import {
  DEFAULT_ATLAS_LAYERS,
  type AtlasLayerKey,
  type AtlasLayers,
} from '../../lib/atlasLayers'
import type { TierPreference } from '../../lib/perfTier'
import { LOD_LEVELS } from '../../lib/lod'

interface Props {
  /** Camera jump target picked from the search results. */
  onFlyTo: (result: SearchResult) => void
  /** Exact coordinate navigation: decimal degrees + altitude in metres. */
  onGotoCoordinates: (longitude: number, latitude: number, height: number) => void
  /** One-shot camera commands (reset / north-up / tilt). */
  onCommand: (kind: 'reset' | 'north' | 'tilt') => void
  onToggleMeasure: () => void
  measuring: boolean
  layers: AtlasLayers
  onToggleLayer: (key: AtlasLayerKey) => void
  basemap: 'imagery' | 'terrain' | 'osm'
  onBasemap: (style: 'imagery' | 'terrain' | 'osm') => void
  tierPref: TierPreference
  onTierPref: (pref: TierPreference) => void
  fps: number | null
  lod: number
  onJumpLod: (lod: number) => void
  /** Rendered beneath the controls (shared with the map stage). */
  children?: React.ReactNode
}

const LAYER_ROWS: { key: AtlasLayerKey; icon: typeof Square; labelKey: TranslationKey }[] = [
  { key: 'parcels', icon: Square, labelKey: 'layerParcels' },
  { key: 'roads', icon: Route, labelKey: 'layerRoads' },
  { key: 'buildings', icon: Building2, labelKey: 'layerBuildings' },
  { key: 'pois', icon: MapPin, labelKey: 'layerPlaces' },
  { key: 'labels', icon: Tag, labelKey: 'layerLabels' },
]

const BASEMAPS: { style: 'imagery' | 'terrain' | 'osm'; labelKey: TranslationKey }[] = [
  { style: 'imagery', labelKey: 'basemapSatellite' },
  { style: 'terrain', labelKey: 'basemapTerrain' },
  { style: 'osm', labelKey: 'basemapStreets' },
]

const TIERS: { value: TierPreference; labelKey: TranslationKey }[] = [
  { value: 'auto', labelKey: 'tierAuto' },
  { value: 'low', labelKey: 'tierLow' },
  { value: 'medium', labelKey: 'tierMedium' },
  { value: 'high', labelKey: 'tierHigh' },
  { value: 'ultra', labelKey: 'tierUltra' },
]

/**
 * Every atlas control that used to float over the map now lives here, inside
 * the single sidebar: place search, exact coordinate navigation, view
 * commands, layer filters, basemap, graphics tier and the progressive zoom
 * ladder. The map stage keeps only the transient measure readout.
 */
export default function AtlasControlsPanel({
  onFlyTo,
  onGotoCoordinates,
  onCommand,
  onToggleMeasure,
  measuring,
  layers,
  onToggleLayer,
  basemap,
  onBasemap,
  tierPref,
  onTierPref,
  fps,
  lod,
  onJumpLod,
  children,
}: Props) {
  const { t } = useLocale()
  const [open, setOpen] = useState(false)
  const [query, setQuery] = useState('')
  const [busy, setBusy] = useState(false)
  const [results, setResults] = useState<SearchResult[]>([])
  const [coords, setCoords] = useState({ lat: '', lon: '', alt: '300' })
  const [coordError, setCoordError] = useState<string | null>(null)

  const localResults = useMemo(() => searchLocal(query, 5), [query])

  const pick = (r: SearchResult) => {
    onFlyTo(r)
    setQuery(r.label)
    setResults([])
  }

  const submit = async () => {
    if (localResults.length > 0) {
      pick(localResults[0])
      return
    }
    const q = query.trim()
    if (!q) return
    setBusy(true)
    try {
      const remote = await searchPlaces(q)
      setResults(remote.slice(0, 6))
      if (remote.length > 0) pick(remote[0])
    } finally {
      setBusy(false)
    }
  }

  const goto = () => {
    const lat = Number.parseFloat(coords.lat)
    const lon = Number.parseFloat(coords.lon)
    const alt = Number.parseFloat(coords.alt)
    if (!Number.isFinite(lat) || !Number.isFinite(lon) || Math.abs(lat) > 90 || Math.abs(lon) > 180) {
      setCoordError(t('gotoInvalid'))
      return
    }
    if (!Number.isFinite(alt) || alt < 1 || alt > 20_000_000) {
      setCoordError(t('gotoInvalid'))
      return
    }
    setCoordError(null)
    onGotoCoordinates(lon, lat, alt)
  }

  const active = LOD_LEVELS[lod] ?? LOD_LEVELS[0]

  return (
    <div className="atlas-controls">
      <label className="atlas-search">
        <Search size={15} />
        <input
          value={query}
          onChange={(event) => {
            setQuery(event.target.value)
            setResults([])
          }}
          onKeyDown={(event) => {
            if (event.key === 'Enter') void submit()
            if (event.key === 'Escape') {
              setQuery('')
              setResults([])
            }
          }}
          placeholder={t('searchPlaceholder')}
          aria-label={t('searchTitle')}
        />
        {query && (
          <button type="button" onClick={() => setQuery('')} aria-label={t('clearAction')}>
            ✕
          </button>
        )}
      </label>
      {(localResults.length > 0 || results.length > 0) && (
        <div className="atlas-search-results" role="listbox">
          {localResults.map((r) => (
            <button key={r.id} type="button" onClick={() => pick(r)}>
              <span>{r.label}</span>
              <small>{r.source === 'coords' ? t('searchCoords') : t('searchNearby')}</small>
            </button>
          ))}
          {localResults.length === 0 && results.length > 0 && (
            <>
              {results.map((r) => (
                <button key={r.id} type="button" onClick={() => pick(r)}>
                  <span>{r.label}</span>
                  <small>{t('searchOnline')}</small>
                </button>
              ))}
            </>
          )}
          {localResults.length === 0 && results.length === 0 && (
            <button type="button" disabled={busy || !query.trim()} onClick={() => void submit()}>
              <span>{busy ? '…' : t('searchOnline')}</span>
            </button>
          )}
        </div>
      )}

      <button
        type="button"
        className="atlas-controls-toggle"
        aria-expanded={open}
        onClick={() => setOpen((v) => !v)}
      >
        <Layers size={13} />
        {t('mapControlsTitle')}
        <em>{open ? '▾' : '▸'}</em>
      </button>

      {open && (
        <div className="atlas-controls-body">
          <section className="atlas-controls-section">
            <h4>{t('cameraSectionTitle')}</h4>
            <div className="atlas-coord-grid">
              <input
                inputMode="decimal"
                value={coords.lat}
                onChange={(e) => setCoords((c) => ({ ...c, lat: e.target.value }))}
                placeholder={t('labelLatitude')}
                aria-label={t('labelLatitude')}
              />
              <input
                inputMode="decimal"
                value={coords.lon}
                onChange={(e) => setCoords((c) => ({ ...c, lon: e.target.value }))}
                placeholder={t('labelLongitude')}
                aria-label={t('labelLongitude')}
              />
              <input
                inputMode="decimal"
                value={coords.alt}
                onChange={(e) => setCoords((c) => ({ ...c, alt: e.target.value }))}
                placeholder={t('labelAltitude')}
                aria-label={t('labelAltitude')}
              />
              <button type="button" className="btn btn-primary atlas-coord-go" onClick={goto}>
                <Crosshair size={13} /> {t('gotoAction')}
              </button>
            </div>
            {coordError && <p className="atlas-controls-error">{coordError}</p>}

            <div className="atlas-command-row">
              <button type="button" className="btn btn-ghost" onClick={() => onCommand('reset')} title={t('resetViewTitle')}>
                <RotateCcw size={13} /> {t('resetViewTitle')}
              </button>
              <button type="button" className="btn btn-ghost" onClick={() => onCommand('north')} title={t('northUpTitle')}>
                <Compass size={13} /> {t('northUpTitle')}
              </button>
              <button type="button" className="btn btn-ghost" onClick={() => onCommand('tilt')} title={t('tiltTitle')}>
                <Move3d size={13} /> {t('tiltTitle')}
              </button>
              <button
                type="button"
                className={`btn btn-ghost ${measuring ? 'is-active' : ''}`}
                aria-pressed={measuring}
                onClick={onToggleMeasure}
                title={t('measureTitle')}
              >
                <Ruler size={13} /> {t('measureAction')}
              </button>
            </div>
          </section>

          <section className="atlas-controls-section">
            <h4>{t('mapDetails')}</h4>
            <div className="atlas-layer-list">
              {LAYER_ROWS.map(({ key, icon: Icon, labelKey }) => (
                <button
                  key={key}
                  type="button"
                  className="atlas-layer-row"
                  aria-pressed={layers[key]}
                  onClick={() => onToggleLayer(key)}
                >
                  <span className={`layer-box ${layers[key] ? 'on' : ''}`} aria-hidden="true">
                    {layers[key] ? <Check size={11} /> : null}
                  </span>
                  <Icon size={13} />
                  <span>{t(labelKey)}</span>
                </button>
              ))}
            </div>
          </section>

          <section className="atlas-controls-section">
            <h4>{t('basemapSectionTitle')}</h4>
            <div className="atlas-seg" role="group" aria-label={t('basemapSectionTitle')}>
              {BASEMAPS.map(({ style, labelKey }) => (
                <button
                  key={style}
                  type="button"
                  className={basemap === style ? 'active' : ''}
                  aria-pressed={basemap === style}
                  onClick={() => onBasemap(style)}
                >
                  {t(labelKey)}
                </button>
              ))}
            </div>
            <div className="atlas-tier-row">
              <label htmlFor="atlas-tier">{t('graphicsTierLabel')}</label>
              <select
                id="atlas-tier"
                value={tierPref}
                onChange={(event) => onTierPref(event.target.value as TierPreference)}
              >
                {TIERS.map(({ value, labelKey }) => (
                  <option key={value} value={value}>
                    {t(labelKey)}
                  </option>
                ))}
              </select>
              <span className="perf-readout">
                {fps !== null ? t('fpsLabel', { n: fps }) : t('fpsIdle')}
              </span>
            </div>
          </section>

          <section className="atlas-controls-section">
            <h4>{t('zoomDetail')}</h4>
            <ol className="atlas-lod-list" aria-label={t('lodAria')}>
              {LOD_LEVELS.map((level) => (
                <li
                  key={level.id}
                  className={level.id === lod ? 'is-active' : level.id < lod ? 'is-past' : ''}
                >
                  <button type="button" onClick={() => onJumpLod(level.id)} title={t('jumpToLod', { name: t(level.nameKey) })}>
                    <span className="lod-dot" />
                    <span>{t(level.nameKey)}</span>
                  </button>
                </li>
              ))}
            </ol>
            <p className="atlas-lod-reveals">
              <Box size={11} /> {t(active.revealsKey)}
            </p>
          </section>

          {children}
        </div>
      )}
    </div>
  )
}

export { DEFAULT_ATLAS_LAYERS }
