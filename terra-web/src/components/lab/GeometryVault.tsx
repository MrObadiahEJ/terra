import { useEffect, useMemo, useState } from 'react'
import { PublicKey } from '@solana/web3.js'
import {
  useLabVault,
  labShape,
  SPATIAL_DIMENSIONS,
  GEOMETRY_SOURCES,
  MAX_GEOMETRY_VERSIONS,
  type LabResult,
} from '../../lib/labStore'
import { sha256Bytes, sha256Hex, type LonLat } from '../../lib/geo'
import { Boxes, FilePlus2, BadgeCheck, RotateCcw } from 'lucide-react'

// --- borsh-style account serialization (Anchor layout, little-endian) -------

interface Field {
  name: string
  offset: number
  len: number
  value: string
}

function builder() {
  const out: number[] = []
  const fields: Field[] = []
  const add = (name: string, bytes: number[] | Uint8Array, value: string) => {
    const arr = Array.from(bytes)
    fields.push({ name, offset: out.length, len: arr.length, value })
    out.push(...arr)
  }
  return { add, finish: () => new Uint8Array(out), fields }
}

const u8 = (v: number) => [v & 0xff]
const u32 = (v: number) => [v & 0xff, (v >>> 8) & 0xff, (v >>> 16) & 0xff, (v >>> 24) & 0xff]
const i32 = u32
const i64 = (v: number) => {
  const b = new BigInt64Array(1)
  b[0] = BigInt(Math.trunc(v))
  return Array.from(new Uint8Array(b.buffer))
}
const bool = (v: boolean) => [v ? 1 : 0]
const utf8 = (s: string) => Array.from(new TextEncoder().encode(s))

interface Serialized {
  bytes: Uint8Array
  fields: Field[]
}

function hexDump(bytes: Uint8Array): string {
  const lines: string[] = []
  for (let i = 0; i < bytes.length; i += 16) {
    const slice = Array.from(bytes.slice(i, i + 16))
    const hex = slice.map((b) => b.toString(16).padStart(2, '0')).join(' ').padEnd(47, ' ')
    const ascii = slice.map((b) => (b >= 32 && b < 127 ? String.fromCharCode(b) : '.')).join('')
    lines.push(`${i.toString(16).padStart(4, '0')}  ${hex}  |${ascii}|`)
  }
  return lines.join('\n')
}

function shortHex(h: string): string {
  return `${h.slice(0, 16)}…`
}

// --- shape preview (SVG, no map dependency) ---------------------------------

function ShapePreview({ ring }: { ring: LonLat[] }) {
  const { points, areaLabel } = useMemo(() => {
    const lons = ring.map((p) => p[0])
    const lats = ring.map((p) => p[1])
    const minLon = Math.min(...lons)
    const maxLon = Math.max(...lons)
    const minLat = Math.min(...lats)
    const maxLat = Math.max(...lats)
    const spanLon = maxLon - minLon || 1e-6
    const spanLat = maxLat - minLat || 1e-6
    const pad = 8
    const w = 280
    const h = 140
    const pts = ring
      .map(([lon, lat]) => {
        const x = pad + ((lon - minLon) / spanLon) * (w - 2 * pad)
        const y = h - pad - ((lat - minLat) / spanLat) * (h - 2 * pad)
        return `${x.toFixed(1)},${y.toFixed(1)}`
      })
      .join(' ')
    return {
      points: pts,
      areaLabel: `${ring.length} vertices · bbox ${spanLon.toFixed(4)}°×${spanLat.toFixed(4)}°`,
    }
  }, [ring])

  return (
    <div className="lab-shape">
      <svg viewBox="0 0 280 140" width="100%" height="140" role="img" aria-label="geometry shape">
        <polygon points={points} fill="rgba(16,185,129,0.18)" stroke="#10b981" strokeWidth="1.5" />
      </svg>
      <p className="text-[10px] text-muted text-center">{areaLabel}</p>
    </div>
  )
}

// --- main component ---------------------------------------------------------

export default function GeometryVault() {
  const v = useLabVault()
  const [msg, setMsg] = useState<LabResult | null>(null)

  // Form state
  const [dim, setDim] = useState(1)
  const [elevMin, setElevMin] = useState(0) // metres
  const [elevMax, setElevMax] = useState(1200) // metres
  const [source, setSource] = useState(3)
  const [vDim, setVDim] = useState(0)
  const [storageRef, setStorageRef] = useState('')
  const [inspected, setInspected] = useState<number | null>(null) // null → SpatialAsset

  // Deterministic demo keys (SHA-256 of labels → 32 bytes).
  const [keys, setKeys] = useState<{
    parcel: Uint8Array
    asset: Uint8Array
    authority: Uint8Array
    validator: Uint8Array
    authorityB58: string
    validatorB58: string
    discAsset: Uint8Array
    discVersion: Uint8Array
  } | null>(null)

  useEffect(() => {
    let cancelled = false
    ;(async () => {
      const [parcel, asset, authority, validator, dAsset, dVersion] = await Promise.all([
        sha256Bytes('terra-lab-parcel'),
        sha256Bytes('terra-lab-spatial-asset-pda'),
        sha256Bytes('terra-lab-registrar'),
        sha256Bytes('terra-lab-validator'),
        sha256Bytes('account:SpatialAsset'),
        sha256Bytes('account:GeometryVersion'),
      ])
      if (cancelled) return
      setKeys({
        parcel,
        asset,
        authority,
        validator,
        authorityB58: new PublicKey(authority).toBase58(),
        validatorB58: new PublicKey(validator).toBase58(),
        discAsset: dAsset.slice(0, 8),
        discVersion: dVersion.slice(0, 8),
      })
    })()
    return () => {
      cancelled = true
    }
  }, [])

  const shape = useMemo(() => labShape(v.shapeIndex), [v.shapeIndex])
  const [digest, setDigest] = useState<string | null>(null)
  useEffect(() => {
    let cancelled = false
    sha256Hex(JSON.stringify(shape)).then((d) => !cancelled && setDigest(d))
    return () => {
      cancelled = true
    }
  }, [shape])

  const nextIndex = v.versions.length
  const defaultStorageRef = `postgis://lab/geometry-v${nextIndex}.geojson`
  const activeStorageRef = storageRef.trim() || defaultStorageRef

  // Serialized accounts for the byte-level inspector.
  const serialized = useMemo<Serialized | null>(() => {
    if (!keys) return null
    const ts = (iso: string | null) => (iso ? Date.parse(iso) : 0)
    if (inspected === null) {
      if (!v.initialized) return null
      const b = builder()
      b.add('discriminator', keys.discAsset, keys.discAsset.map((x) => x.toString(16).padStart(2, '0')).join(''))
      b.add('parcel (Pubkey)', keys.parcel, new PublicKey(keys.parcel).toBase58())
      b.add('authority (Pubkey)', keys.authority, new PublicKey(keys.authority).toBase58())
      b.add('dimensionality (u8)', u8(v.dimensionality), String(v.dimensionality))
      b.add('elevation_min_mm (i32)', i32(v.elevationMinMm), `${v.elevationMinMm} mm`)
      b.add('elevation_max_mm (i32)', i32(v.elevationMaxMm), `${v.elevationMaxMm} mm`)
      b.add('geometry_version_count (u32)', u32(v.versions.length), String(v.versions.length))
      const latest = v.versions.length > 0 ? v.versions[v.versions.length - 1].geometryHash : '0'.repeat(64)
      b.add('latest_geometry ([u8;32])', hexToBytes(latest), shortHex(latest))
      b.add('created_at (i64)', i64(ts(v.createdAt)), v.createdAt ?? '0')
      b.add('updated_at (i64)', i64(ts(v.updatedAt)), v.updatedAt ?? '0')
      return { bytes: b.finish(), fields: b.fields }
    }
    const entry = v.versions.find((x) => x.version === inspected)
    if (!entry) return null
    const b = builder()
    b.add('discriminator', keys.discVersion, keys.discVersion.map((x) => x.toString(16).padStart(2, '0')).join(''))
    b.add('asset (Pubkey)', keys.asset, new PublicKey(keys.asset).toBase58())
    b.add('parcel (Pubkey)', keys.parcel, new PublicKey(keys.parcel).toBase58())
    b.add('version (u32)', u32(entry.version), String(entry.version))
    b.add('geometry_hash ([u8;32])', hexToBytes(entry.geometryHash), shortHex(entry.geometryHash))
    b.add('source (u8)', u8(entry.source), `${entry.source} (${GEOMETRY_SOURCES[entry.source]?.id ?? '?'})`)
    b.add('dimension (u8)', u8(entry.dimension), `${entry.dimension} (${SPATIAL_DIMENSIONS[entry.dimension]?.id ?? '?'})`)
    b.add('storage_reference.len (u32)', u32(new TextEncoder().encode(entry.storageReference).length), '')
    b.add('storage_reference (utf8)', utf8(entry.storageReference), entry.storageReference)
    b.add('submitted_by (Pubkey)', keys.validator, entry.submittedBy)
    b.add('submitted_at (i64)', i64(Date.parse(entry.submittedAt)), entry.submittedAt)
    b.add('verified (bool)', bool(entry.verified), String(entry.verified))
    b.add('verified_by (Pubkey)', keys.validator, entry.verifiedBy ?? '111…111 (zero)')
    b.add('verified_at (i64)', i64(entry.verifiedAt ? Date.parse(entry.verifiedAt) : 0), entry.verifiedAt ?? '0')
    return { bytes: b.finish(), fields: b.fields }
  }, [keys, inspected, v])

  const disabled = !keys

  return (
    <div className="lab-body">
      <div className="lab-grid">
        {/* left: shape + forms */}
        <div className="lab-col space-y-3">
          <div className="lab-card">
            <div className="flex items-center justify-between mb-2">
              <h3 className="text-[13px] font-semibold flex items-center gap-1.5">
                <Boxes size={14} /> Current geometry claim
              </h3>
              <button className="btn btn-secondary" onClick={v.nextShape}>
                New shape
              </button>
            </div>
            <ShapePreview ring={shape} />
            <p className="font-mono text-[10px] break-all mt-1">
              sha256(ring JSON) ={' '}
              <span className="text-emerald-700">{digest ? `${digest.slice(0, 32)}…` : 'hashing…'}</span>
            </p>
          </div>

          {!v.initialized ? (
            <div className="lab-card space-y-2">
              <h3 className="text-[13px] font-semibold">init_spatial_asset</h3>
              <p className="text-[11px] text-muted">
                Permissionless anchor for a parcel — PDA <span className="font-mono">["spatial_asset", parcel]</span>.
                Registrar: <span className="font-mono break-all">{keys?.authorityB58 ?? '…'}</span>
              </p>
              <label className="text-[11px] text-muted">Dimensionality (u8)</label>
              <select className="select-input" value={dim} onChange={(e) => setDim(Number(e.target.value))}>
                {SPATIAL_DIMENSIONS.map((d) => (
                  <option key={d.code} value={d.code}>
                    {d.code} = {d.label} — {d.note}
                  </option>
                ))}
              </select>
              <div className="grid grid-cols-2 gap-x-3 gap-y-1">
                <label className="text-[11px] text-muted">
                  Elevation min (m)
                  <input
                    className="text-input"
                    type="number"
                    value={elevMin}
                    onChange={(e) => setElevMin(Number(e.target.value))}
                  />
                </label>
                <label className="text-[11px] text-muted">
                  Elevation max (m)
                  <input
                    className="text-input"
                    type="number"
                    value={elevMax}
                    onChange={(e) => setElevMax(Number(e.target.value))}
                  />
                </label>
              </div>
              <button
                className="btn btn-primary w-full justify-center"
                disabled={disabled}
                onClick={() =>
                  setMsg(
                    v.initAsset(
                      keys?.authorityB58 ?? '',
                      dim,
                      Math.round(elevMin * 1000),
                      Math.round(elevMax * 1000),
                    ),
                  )
                }
              >
                Initialize asset
              </button>
            </div>
          ) : (
            <div className="lab-card space-y-2">
              <h3 className="text-[13px] font-semibold flex items-center gap-1.5">
                <FilePlus2 size={14} /> append_geometry_version
              </h3>
              <div className="grid grid-cols-2 gap-x-3 gap-y-1">
                <label className="text-[11px] text-muted">
                  Source (u8)
                  <select className="select-input" value={source} onChange={(e) => setSource(Number(e.target.value))}>
                    {GEOMETRY_SOURCES.map((s) => (
                      <option key={s.code} value={s.code}>
                        {s.code} = {s.label}
                      </option>
                    ))}
                  </select>
                </label>
                <label className="text-[11px] text-muted">
                  Dimension (u8)
                  <select className="select-input" value={vDim} onChange={(e) => setVDim(Number(e.target.value))}>
                    {SPATIAL_DIMENSIONS.map((d) => (
                      <option key={d.code} value={d.code}>
                        {d.code} = {d.label}
                      </option>
                    ))}
                  </select>
                </label>
              </div>
              <label className="text-[11px] text-muted">
                storage_reference (1..=128 chars)
                <input
                  className="text-input"
                  value={storageRef}
                  placeholder={defaultStorageRef}
                  onChange={(e) => setStorageRef(e.target.value)}
                />
              </label>
              <p className="text-[11px] text-muted">
                Cursor: <b className="font-mono">{v.versions.length}</b> / {MAX_GEOMETRY_VERSIONS}
                {v.versions.length >= MAX_GEOMETRY_VERSIONS && (
                  <span className="text-red-700"> — cap reached</span>
                )}
              </p>
              <button
                className="btn btn-primary w-full justify-center"
                disabled={disabled}
                onClick={async () =>
                  setMsg(
                    await v.appendVersion(
                      shape,
                      source,
                      vDim,
                      activeStorageRef,
                      keys?.validatorB58 ?? 'payer',
                    ),
                  )
                }
              >
                Anchor version {nextIndex}
              </button>
            </div>
          )}
        </div>

        {/* right: history + inspector */}
        <div className="lab-col space-y-3">
          <div className="lab-card">
            <div className="flex items-center justify-between mb-2">
              <h3 className="text-[13px] font-semibold">Geometry history (append-only)</h3>
              <button
                className="btn btn-ghost p-1"
                title="Reset experiment"
                onClick={() => {
                  v.reset()
                  setMsg(null)
                  setStorageRef('')
                  setInspected(null)
                }}
              >
                <RotateCcw size={13} />
              </button>
            </div>
            {v.versions.length === 0 ? (
              <p className="text-[12px] text-muted">No versions anchored yet.</p>
            ) : (
              <ul className="space-y-1">
                {v.versions.map((entry) => (
                  <li key={entry.version}>
                    <div
                      className={`lab-version ${inspected === entry.version ? 'active' : ''}`}
                      onClick={() => setInspected(entry.version)}
                      role="button"
                      tabIndex={0}
                      onKeyDown={(e) => e.key === 'Enter' && setInspected(entry.version)}
                    >
                      <div className="flex items-center gap-2 min-w-0">
                        <span className="font-mono text-[11px] shrink-0">v{entry.version}</span>
                        <span className="text-[11px] text-muted truncate">
                          {GEOMETRY_SOURCES[entry.source]?.label} · {SPATIAL_DIMENSIONS[entry.dimension]?.label}
                        </span>
                        <span className="flex-1" />
                        {entry.verified ? (
                          <span className="lab-badge lab-badge-ok">verified</span>
                        ) : (
                          <button
                            className="btn btn-secondary px-2 py-0.5 gap-1"
                            disabled={!keys}
                            onClick={(e) => {
                              e.stopPropagation()
                              setMsg(v.verifyVersion(entry.version, keys?.validatorB58 ?? 'validator'))
                            }}
                          >
                            <BadgeCheck size={12} /> Verify
                          </button>
                        )}
                      </div>
                      <div className="font-mono text-[10px] text-muted break-all">
                        {shortHex(entry.geometryHash)} · {entry.storageReference}
                      </div>
                      {entry.verified && entry.verifiedBy && (
                        <div className="font-mono text-[10px] text-emerald-700 break-all">
                          ✓ {entry.verifiedBy.slice(0, 16)}… at {entry.verifiedAt?.slice(0, 19).replace('T', ' ')}
                        </div>
                      )}
                    </div>
                  </li>
                ))}
              </ul>
            )}
            {v.initialized && (
              <div className="mt-2 flex gap-1 flex-wrap">
                <button
                  className={`btn btn-ghost px-2 py-1 ${inspected === null ? 'text-emerald-700' : 'text-muted'}`}
                  onClick={() => setInspected(null)}
                >
                  Inspect SpatialAsset
                </button>
                {v.versions.map((entry) => (
                  <button
                    key={entry.version}
                    className={`btn btn-ghost px-2 py-1 ${inspected === entry.version ? 'text-emerald-700' : 'text-muted'}`}
                    onClick={() => setInspected(entry.version)}
                  >
                    GeometryVersion v{entry.version}
                  </button>
                ))}
              </div>
            )}
          </div>

          {/* byte-level inspector */}
          {serialized && keys && (
            <div className="lab-card">
              <h3 className="text-[13px] font-semibold mb-2">
                Account inspector — exact bytes{' '}
                <span className="text-muted font-normal">
                  ({inspected === null ? 'SpatialAsset' : `GeometryVersion v${inspected}`},{' '}
                  {serialized.bytes.length} B)
                </span>
              </h3>
              <div className="lab-field-table">
                {serialized.fields.map((f) => (
                  <div key={f.name} className="lab-field-row">
                    <span className="font-mono text-[10px] text-muted">
                      +{f.offset} ({f.len} B)
                    </span>
                    <span className="font-mono text-[10px]">{f.name}</span>
                    <span className="font-mono text-[10px] truncate" title={f.value}>
                      {f.value}
                    </span>
                  </div>
                ))}
              </div>
              <pre className="hexdump mt-2">{hexDump(serialized.bytes)}</pre>
              <p className="text-[10px] text-muted mt-1">
                Anchor layout: 8-byte discriminator (sha256("account:NAME")[0..8]) + borsh fields, little-endian —
                the literal account data <span className="font-mono">init_spatial_asset</span> /
                <span className="font-mono"> append_geometry_version</span> would write.
              </p>
            </div>
          )}
        </div>
      </div>

      {msg && (
        <p className={msg.ok ? 'lab-msg ok' : 'lab-msg err'}>{msg.msg}</p>
      )}
    </div>
  )
}

function hexToBytes(hex: string): Uint8Array {
  const out = new Uint8Array(hex.length / 2)
  for (let i = 0; i < out.length; i++) out[i] = parseInt(hex.slice(i * 2, i * 2 + 2), 16)
  return out
}
