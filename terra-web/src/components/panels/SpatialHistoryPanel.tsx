import { useEffect, useState } from 'react'
import { History, Box, Camera } from 'lucide-react'
import { useLocale } from '../../lib/locale'
import {
  listAssetsByParcel,
  listGeometryVersions,
  listSpatialModels,
  listSpatialSnapshots,
  codeKey,
  GEOMETRY_SOURCE_KEYS,
  SPATIAL_DIMENSION_KEYS,
  ELEVATION_SOURCE_KEYS,
  MODEL_FORMAT_KEYS,
} from '../../lib/spatial'
import type {
  SpatialAssetMirror,
  GeometryVersionMirror,
  SpatialModelMirror,
  SpatialSnapshotMirror,
} from '../../lib/spatial'

interface Props {
  /** On-chain Parcel PDA (base58) — the mirror indexes assets by parcel. */
  parcelPubkey: string
  /** Open a model's storage reference on the globe (3D Tiles / glTF). */
  onOpenModel?: (uri: string) => void
}

/**
 * Geometry-version history, 3D model claims and point-in-time snapshots for
 * the selected parcel (RFC-013 Phase D), read from the spatial account
 * mirror (Phase E). Selecting a snapshot narrows the version list to the
 * cursor it copied — "what did this place look like at T?".
 */
export default function SpatialHistoryPanel({ parcelPubkey, onOpenModel }: Props) {
  const { t } = useLocale()
  const [asset, setAsset] = useState<SpatialAssetMirror | null>(null)
  const [versions, setVersions] = useState<GeometryVersionMirror[]>([])
  const [models, setModels] = useState<SpatialModelMirror[]>([])
  const [snapshots, setSnapshots] = useState<SpatialSnapshotMirror[]>([])
  const [loading, setLoading] = useState(false)
  const [err, setErr] = useState<string | null>(null)
  const [snapshotCursor, setSnapshotCursor] = useState<number | null>(null)
  const [selectedVersion, setSelectedVersion] = useState<number | null>(null)

  useEffect(() => {
    if (!parcelPubkey) return
    let cancelled = false
    // Defer so setState never runs synchronously within the effect body.
    const timer = setTimeout(() => {
      setLoading(true)
      setErr(null)
      setAsset(null)
      setVersions([])
      setModels([])
      setSnapshots([])
      setSnapshotCursor(null)
      setSelectedVersion(null)
      listAssetsByParcel(parcelPubkey)
        .then(async (assets) => {
          if (cancelled) return
          const first = assets[0] ?? null
          setAsset(first)
          if (!first) return
          const [vs, ms, ss] = await Promise.all([
            listGeometryVersions(first.pubkey),
            listSpatialModels(first.pubkey),
            listSpatialSnapshots(first.pubkey),
          ])
          if (cancelled) return
          setVersions(vs)
          setModels(ms)
          setSnapshots(ss)
        })
        .catch((e) => {
          if (!cancelled) setErr(e instanceof Error ? e.message : t('spatialHistoryUnavailable'))
        })
        .finally(() => {
          if (!cancelled) setLoading(false)
        })
    }, 0)
    return () => {
      cancelled = true
      clearTimeout(timer)
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [parcelPubkey])

  if (loading) {
    return (
      <div className="p-3 text-sm">
        <p className="text-muted text-[12px]">{t('loadingEllipsis')}</p>
      </div>
    )
  }
  if (err) {
    return (
      <div className="p-3 text-sm">
        <p className="text-red-700 text-[12px]">{t('spatialHistoryUnavailable')}</p>
      </div>
    )
  }
  if (!asset) {
    return (
      <div className="px-3 pb-3 text-sm">
        <p className="text-muted text-[12px]">{t('noSpatialHistory')}</p>
      </div>
    )
  }

  const dimLabel = t(codeKey(SPATIAL_DIMENSION_KEYS, asset.dimensionality) as never)
  const visibleVersions =
    snapshotCursor === null ? versions : versions.filter((v) => v.version < snapshotCursor)

  return (
    <div className="p-3 text-sm space-y-3 border-t">
      <div className="flex items-center gap-2">
        <History size={14} />
        <h4 className="font-semibold">{t('spatialHistory')}</h4>
        <span className="text-[10px] px-1.5 py-0.5 rounded bg-indigo-100 text-indigo-800">
          {dimLabel}
        </span>
      </div>
      <p className="text-muted text-[10px] font-mono break-all" title={asset.pubkey}>
        {asset.pubkey.slice(0, 12)}…{asset.pubkey.slice(-6)} ·{' '}
        {t('spatialAssetSummary', {
          min: (asset.elevation_min_mm / 1000).toFixed(1),
          max: (asset.elevation_max_mm / 1000).toFixed(1),
          dim: dimLabel,
        })}
      </p>

      {/* --- geometry versions --- */}
      <div>
        <h5 className="font-semibold mb-1 text-[12px]">
          {t('spatialVersions')} ({visibleVersions.length})
        </h5>
        {visibleVersions.length === 0 ? (
          <p className="text-muted text-[12px]">{t('spatialNoVersions')}</p>
        ) : (
          <ul className="space-y-1">
            {visibleVersions.map((v) => (
              <li
                key={v.pubkey}
                className={`text-[12px] border rounded p-2 cursor-pointer ${
                  selectedVersion === v.version ? 'border-indigo-400 bg-indigo-50' : ''
                }`}
                onClick={() =>
                  setSelectedVersion((prev) => (prev === v.version ? null : v.version))
                }
                title={v.geometry_hash}
              >
                <div className="flex flex-wrap items-center gap-1.5">
                  <span className="bg-slate-200 text-slate-800 text-[10px] px-1.5 py-0.5 rounded font-mono">
                    v{v.version}
                  </span>
                  <span className="bg-emerald-100 text-emerald-800 text-[10px] px-1.5 py-0.5 rounded">
                    {t(codeKey(GEOMETRY_SOURCE_KEYS, v.source) as never)}
                  </span>
                  <span className="bg-sky-100 text-sky-800 text-[10px] px-1.5 py-0.5 rounded">
                    {t(codeKey(SPATIAL_DIMENSION_KEYS, v.dimension) as never)}
                  </span>
                  {v.elevation_source > 0 && (
                    <span className="bg-amber-100 text-amber-800 text-[10px] px-1.5 py-0.5 rounded">
                      {t(codeKey(ELEVATION_SOURCE_KEYS, v.elevation_source) as never)}
                    </span>
                  )}
                  {v.verified ? (
                    <span className="bg-indigo-100 text-indigo-800 text-[10px] px-1.5 py-0.5 rounded">
                      {t('spatialVerifiedQuorum', { a: v.attest_count, r: v.required })}
                    </span>
                  ) : (
                    <span className="text-muted text-[10px]">
                      {t('spatialAttested', { n: v.attest_count })}
                    </span>
                  )}
                  {v.evidence_manifest && (
                    <span className="text-[10px] text-muted font-mono" title={v.evidence_manifest}>
                      ⚓
                    </span>
                  )}
                </div>
                <p className="text-muted text-[10px] font-mono break-all mt-1">
                  {v.geometry_hash.slice(0, 16)}…
                  {v.chain_submitted_at
                    ? ` · ${new Date(v.chain_submitted_at).toLocaleDateString()}`
                    : ''}
                </p>
              </li>
            ))}
          </ul>
        )}
      </div>

      {/* --- 3D model claims --- */}
      <div>
        <h5 className="font-semibold mb-1 text-[12px] flex items-center gap-1.5">
          <Box size={12} /> {t('spatialModels')} ({models.length})
        </h5>
        {models.length === 0 ? (
          <p className="text-muted text-[12px]">{t('spatialNoModels')}</p>
        ) : (
          <ul className="space-y-1">
            {models.map((m) => (
              <li key={m.pubkey} className="text-[12px] border rounded p-2">
                <div className="flex flex-wrap items-center gap-1.5">
                  <span className="bg-violet-100 text-violet-800 text-[10px] px-1.5 py-0.5 rounded">
                    {t(codeKey(MODEL_FORMAT_KEYS, m.model_format) as never)}
                  </span>
                  <span className="bg-slate-200 text-slate-800 text-[10px] px-1.5 py-0.5 rounded font-mono">
                    m{m.model_version}
                  </span>
                  <span className="text-muted text-[10px]">
                    {t('spatialFromVersion', { n: m.geometry_version })}
                  </span>
                  {m.verified && (
                    <span className="bg-indigo-100 text-indigo-800 text-[10px] px-1.5 py-0.5 rounded">
                      {t('verified')}
                    </span>
                  )}
                </div>
                <p className="text-muted text-[10px] font-mono break-all mt-1">
                  {m.model_hash.slice(0, 16)}…
                </p>
                {m.storage_reference &&
                  onOpenModel &&
                  (m.model_format === 3 || m.storage_reference.includes('tileset.json')) && (
                    <button
                      className="btn btn-secondary mt-1 justify-center w-full"
                      onClick={() => onOpenModel(m.storage_reference)}
                    >
                      <Camera size={12} /> {t('spatialOpenOnGlobe')}
                    </button>
                  )}
              </li>
            ))}
          </ul>
        )}
      </div>

      {/* --- point-in-time snapshots --- */}
      <div>
        <h5 className="font-semibold mb-1 text-[12px]">{t('spatialSnapshots')} ({snapshots.length})</h5>
        {snapshots.length === 0 ? (
          <p className="text-muted text-[12px]">{t('spatialNoSnapshots')}</p>
        ) : (
          <div className="space-y-1">
            {snapshotCursor !== null && (
              <button
                className="btn btn-ghost w-full justify-center text-[11px]"
                onClick={() => setSnapshotCursor(null)}
              >
                {t('spatialShowAll')}
              </button>
            )}
            <ul className="space-y-1">
              {snapshots.map((s) => (
                <li
                  key={s.pubkey}
                  className={`text-[12px] border rounded p-2 cursor-pointer ${
                    snapshotCursor === s.geometry_version_count ? 'border-amber-400 bg-amber-50' : ''
                  }`}
                  onClick={() =>
                    setSnapshotCursor((prev) =>
                      prev === s.geometry_version_count ? null : s.geometry_version_count,
                    )
                  }
                >
                  <div className="flex flex-wrap items-center gap-1.5">
                    <span className="bg-amber-100 text-amber-800 text-[10px] px-1.5 py-0.5 rounded">
                      {s.label || `#${s.snapshot_index}`}
                    </span>
                    <span className="text-muted text-[10px] font-mono">
                      v &lt; {s.geometry_version_count}
                    </span>
                    {s.chain_created_at && (
                      <span className="text-muted text-[10px]">
                        {new Date(s.chain_created_at).toLocaleDateString()}
                      </span>
                    )}
                  </div>
                </li>
              ))}
            </ul>
            {snapshotCursor !== null && (
              <p className="text-amber-800 text-[10px]">
                {t('spatialPointInTime', { n: snapshotCursor })}
              </p>
            )}
          </div>
        )}
      </div>

      {selectedVersion !== null && (
        <p className="text-indigo-800 text-[11px]">
          {t('spatialVersionSelected', {
            n: selectedVersion,
            src: (() => {
              const v = versions.find((x) => x.version === selectedVersion)
              return v ? t(codeKey(GEOMETRY_SOURCE_KEYS, v.source) as never) : ''
            })(),
            dim: (() => {
              const v = versions.find((x) => x.version === selectedVersion)
              return v ? t(codeKey(SPATIAL_DIMENSION_KEYS, v.dimension) as never) : ''
            })(),
          })}
        </p>
      )}
    </div>
  )
}
