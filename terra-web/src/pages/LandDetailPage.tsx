import { useEffect, useMemo, useState } from 'react'
import { Link, useSearchParams } from 'react-router-dom'
import {
  useLabVault,
  labShape,
  SPATIAL_DIMENSIONS,
  GEOMETRY_SOURCES,
  ELEVATION_SOURCES,
  DEFAULT_GEOMETRY_QUOROM,
  MAX_GEOMETRY_VERSIONS,
  type VaultVersion,
} from '../lib/labStore'
import {
  polygonAreaM2,
  polygonBbox,
  polygonCentroid,
  polygonPerimeterM,
  ringDigest,
} from '../lib/geo'
import LandViewer from '../components/lab/LandViewer'
import ElevationCrossSection from '../components/lab/ElevationCrossSection'
import { ArrowLeft, MapPin, ShieldCheck, ScrollText, Box } from 'lucide-react'

// Land detail view (/lab/land) — click-through from the Geometry Vault.
// Shows every prop of the selected land (SpatialAsset + GeometryVersion) and
// lets you navigate the parcel in a WebGL-free 2D/3D viewer.

function fmt(n: number, digits = 1): string {
  return n.toLocaleString(undefined, { maximumFractionDigits: digits })
}

function Row({ k, v, title }: { k: string; v: string; title?: string }) {
  return (
    <div className="lab-field-row">
      <span className="font-mono text-[10px] text-muted">{k}</span>
      <span className="font-mono text-[10px] col-span-2 truncate" title={title ?? v}>
        {v}
      </span>
    </div>
  )
}

export default function LandDetailPage() {
  const v = useLabVault()
  const [params] = useSearchParams()
  const vRaw = params.get('v')

  // Resolve which land state to show: ?v=N → that version, ?v=claim → the
  // unanchored current claim, absent → latest anchored version (or claim).
  const { entry, notFound } = useMemo<{ entry: VaultVersion | null; notFound: boolean }>(() => {
    if (vRaw === 'claim') return { entry: null, notFound: false }
    if (vRaw !== null) {
      const n = Number(vRaw)
      const found = v.versions.find((x) => x.version === n) ?? null
      return { entry: found, notFound: !found }
    }
    return { entry: v.versions.length > 0 ? v.versions[v.versions.length - 1] : null, notFound: false }
  }, [vRaw, v.versions])

  const ring = useMemo(() => entry?.ring ?? labShape(v.shapeIndex), [entry, v.shapeIndex])
  const [digest, setDigest] = useState<string | null>(null)
  useEffect(() => {
    let cancelled = false
    ringDigest(ring).then((d) => !cancelled && setDigest(d))
    return () => {
      cancelled = true
    }
  }, [ring])

  const metrics = useMemo(() => {
    const area = polygonAreaM2(ring)
    const perim = polygonPerimeterM(ring)
    const [clon, clat] = polygonCentroid(ring)
    const bbox = polygonBbox(ring)
    const closed = ring.length > 1 &&
      ring[0][0] === ring[ring.length - 1][0] &&
      ring[0][1] === ring[ring.length - 1][1]
    return {
      area,
      perim,
      clon,
      clat,
      bbox,
      vertices: closed ? ring.length - 1 : ring.length,
    }
  }, [ring])

  // Viewer inputs: version provenance, else asset envelope + NONE (flat plate).
  const elevMinM = (entry ? entry.elevationMinMm : v.elevationMinMm) / 1000
  const elevMaxM = (entry ? entry.elevationMaxMm : v.elevationMaxMm) / 1000
  const elevSource = entry ? entry.elevationSource : 0
  const dimension = entry ? entry.dimension : 0
  const seedHex = entry ? entry.geometryHash : digest ?? '0'.repeat(64)

  const title = entry ? `Land — GeometryVersion v${entry.version}` : 'Land — current claim'
  const required = entry?.required ?? (v.quorumConfig > 0 ? v.quorumConfig : DEFAULT_GEOMETRY_QUOROM)

  return (
    <div className="land">
      <div className="land-crumb">
        <Link className="btn btn-ghost px-2 py-1 gap-1" to="/lab?tab=vault">
          <ArrowLeft size={13} /> Lab / Geometry Vault
        </Link>
        <span className="text-muted">/ land details</span>
        <span className="flex-1" />
        <Link className="btn btn-secondary px-2 py-1" to="/transactions?view=versions">
          Version timeline ↗
        </Link>
      </div>

      <header className="land-head">
        <div className="min-w-0">
          <h1 className="land-title">
            <MapPin size={18} /> {title}
          </h1>
          <p className="font-mono text-[11px] text-muted break-all">
            sha256(ring JSON) = {digest ? `${digest.slice(0, 40)}…` : 'hashing…'}
          </p>
        </div>
        <div className="land-badges">
          {entry ? (
            entry.verified ? (
              <span className="lab-badge lab-badge-ok">✓ verified — quorum met</span>
            ) : (
              <span className="lab-badge lab-badge-warn">
                claim · quorum {entry.attestors.length}/{required}
              </span>
            )
          ) : v.initialized ? (
            <span className="lab-badge lab-badge-info">unanchored preview</span>
          ) : (
            <span className="lab-badge lab-badge-err">asset not initialized</span>
          )}
          {entry?.evidenceManifest && <span className="lab-badge lab-badge-info">{entry.evidenceManifest}</span>}
          <span className="lab-badge lab-badge-info">3D · SVG · no WebGL</span>
        </div>
      </header>

      {notFound && (
        <p className="lab-msg err">
          GeometryVersion v{vRaw} not found — the vault has {v.versions.length} anchored version
          {v.versions.length === 1 ? '' : 's'}. <Link to="/lab/land">Back to latest land view</Link>
        </p>
      )}

      <div className="land-grid">
        {/* left: props */}
        <div className="lab-col space-y-3">
          <div className="lab-card">
            <h3 className="text-[13px] font-semibold mb-2 flex items-center gap-1.5">
              <MapPin size={14} /> Geometry claim
            </h3>
            <div className="lab-field-table">
              <Row k="state" v={entry ? `anchored v${entry.version}` : 'current claim (not anchored)'} />
              <Row k="vertices" v={`${metrics.vertices} (closed ring)`} />
              <Row k="area" v={`${fmt(metrics.area)} m²`} />
              <Row k="perimeter" v={`${fmt(metrics.perim)} m`} />
              <Row
                k="centroid"
                v={`${metrics.clon.toFixed(6)}°, ${metrics.clat.toFixed(6)}°`}
                title={`lon ${metrics.clon} lat ${metrics.clat}`}
              />
              <Row
                k="bbox"
                v={`${metrics.bbox.minLon.toFixed(5)} … ${metrics.bbox.maxLon.toFixed(5)} lon · ${metrics.bbox.minLat.toFixed(5)} … ${metrics.bbox.maxLat.toFixed(5)} lat`}
              />
              <Row k="geometry_hash" v={digest ?? '…'} title={digest ?? ''} />
            </div>
          </div>

          <div className="lab-card">
            <h3 className="text-[13px] font-semibold mb-2 flex items-center gap-1.5">
              <Box size={14} /> SpatialAsset
            </h3>
            {v.initialized ? (
              <div className="lab-field-table">
                <Row k="authority" v={v.authority} title={v.authority} />
                <Row
                  k="dimensionality"
                  v={`${v.dimensionality} (${SPATIAL_DIMENSIONS[v.dimensionality]?.id ?? '?'})`}
                />
                <Row k="elevation_envelope" v={`${v.elevationMinMm} … ${v.elevationMaxMm} mm`} />
                <Row k="geometry_version_count" v={`${v.versions.length} / ${MAX_GEOMETRY_VERSIONS}`} />
                <Row k="quorum_config" v={v.quorumConfig > 0 ? `(0,[0,0]) → ${v.quorumConfig}` : `unset → fallback ${DEFAULT_GEOMETRY_QUOROM}`} />
                <Row k="evidence_manifest" v={v.manifestNonce === null ? 'none submitted' : `#${v.manifestNonce} · ${v.manifestArtifacts} artifacts`} />
                <Row k="task" v={v.taskCancelled ? 'CANCELLED (terminal → 6174)' : 'open'} />
                <Row k="created_at" v={v.createdAt ?? '—'} />
              </div>
            ) : (
              <p className="text-[12px] text-muted">
                Not initialized — run <span className="font-mono">init_spatial_asset</span> in the{' '}
                <Link to="/lab?tab=vault">vault</Link> first.
              </p>
            )}
          </div>

          <div className="lab-card">
            <h3 className="text-[13px] font-semibold mb-2 flex items-center gap-1.5">
              {entry?.verified ? <ShieldCheck size={14} /> : <ScrollText size={14} />}
              {entry ? `GeometryVersion v${entry.version}` : 'GeometryVersion — pending'}
            </h3>
            {entry ? (
              <div className="lab-field-table">
                <Row k="source" v={`${entry.source} (${GEOMETRY_SOURCES[entry.source]?.id ?? '?'}) — ${GEOMETRY_SOURCES[entry.source]?.label}`} />
                <Row k="dimension" v={`${entry.dimension} (${SPATIAL_DIMENSIONS[entry.dimension]?.id ?? '?'})`} />
                <Row k="storage_reference" v={entry.storageReference} />
                <Row k="submitted_by" v={entry.submittedBy} title={entry.submittedBy} />
                <Row k="submitted_at" v={entry.submittedAt} />
                <Row k="elevation" v={`${entry.elevationMinMm} … ${entry.elevationMaxMm} mm`} />
                <Row k="elevation_source" v={`${entry.elevationSource} (${ELEVATION_SOURCES[entry.elevationSource]?.id ?? '?'}) — ${ELEVATION_SOURCES[entry.elevationSource]?.label}`} />
                <Row k="verified" v={String(entry.verified)} />
                <Row k="verified_by" v={entry.verifiedBy ?? '111…111 (zero)'} title={entry.verifiedBy ?? ''} />
                <Row k="verified_at" v={entry.verifiedAt ?? '0'} />
                <Row k="evidence_manifest" v={entry.evidenceManifest ?? 'default() — standalone append'} />
                <Row k="required (u8)" v={String(entry.required)} />
                <Row k="attest_count" v={`${entry.attestors.length} / 8`} />
              </div>
            ) : (
              <p className="text-[12px] text-muted">
                The current shape has not been anchored yet — append it in the{' '}
                <Link to="/lab?tab=vault">vault</Link> to mint a GeometryVersion.
              </p>
            )}
            {entry && entry.attestors.length > 0 && (
              <div className="mt-2 flex gap-1 flex-wrap">
                {entry.attestors.map((a, i) => (
                  <span key={`${a}-${i}`} className="lab-badge lab-badge-ok">
                    {i + 1}. {a}
                  </span>
                ))}
              </div>
            )}
            {entry && !entry.verified && entry.attestors.length < required && (
              <p className="text-[10px] text-muted mt-1">
                {required - entry.attestors.length} more attestation
                {required - entry.attestors.length === 1 ? '' : 's'} needed to flip verified
                (verify_geometry_version, 6238/6240 guards).
              </p>
            )}
          </div>
        </div>

        {/* right: 3D navigation + cross-section */}
        <div className="lab-col space-y-3">
          <div className="lab-card">
            <div className="flex items-center justify-between gap-2 flex-wrap mb-2">
              <h3 className="text-[13px] font-semibold">3D land view — navigate</h3>
              <span className="lab-badge lab-badge-info">
                {entry ? `v${entry.version}` : 'current shape'}
              </span>
            </div>
            <LandViewer
              ring={ring}
              elevMinM={elevMinM}
              elevMaxM={elevMaxM}
              elevationSource={elevSource}
              dimension={dimension}
              seedHex={seedHex}
              verified={entry?.verified ?? false}
            />
            <p className="text-[10px] text-muted mt-1">
              Drag to pan · wheel to zoom · switch 2D (top-down relief) ↔ 3D (isometric extrusion) ·
              90° rotate. Pure SVG — works on machines without WebGL.
            </p>
          </div>

          <div className="lab-card">
            <div className="flex items-center justify-between gap-2 flex-wrap mb-2">
              <h3 className="text-[13px] font-semibold">Elevation cross-section</h3>
              <span className={`lab-badge ${elevSource === 0 ? 'lab-badge-warn' : 'lab-badge-ok'}`}>
                {ELEVATION_SOURCES[elevSource]?.id ?? 'NONE'}
              </span>
            </div>
            <ElevationCrossSection
              assetMinM={v.elevationMinMm / 1000}
              assetMaxM={v.elevationMaxMm / 1000}
              versionMinM={elevMinM}
              versionMaxM={elevMaxM}
              elevSource={elevSource}
              dimension={dimension}
              assetDimension={v.dimensionality}
            />
          </div>
        </div>
      </div>

      {/* version timeline */}
      <div className="lab-card mt-3">
        <h3 className="text-[13px] font-semibold mb-2">Land through time</h3>
        <div className="land-timeline">
          <Link
            className={`land-tl-chip ${!entry && !notFound ? 'active' : ''}`}
            to="/lab/land?v=claim"
            title="Current unanchored claim"
          >
            <span className="land-tl-v">claim</span>
            <span className="land-tl-s">{SPATIAL_DIMENSIONS[v.dimensionality]?.id ?? 'D2'} · next append</span>
          </Link>
          {v.versions.map((x) => (
            <Link
              key={x.version}
              className={`land-tl-chip ${entry?.version === x.version ? 'active' : ''} ${x.verified ? 'verified' : ''}`}
              to={`/lab/land?v=${x.version}`}
            >
              <span className="land-tl-v">v{x.version}</span>
              <span className="land-tl-s">
                {ELEVATION_SOURCES[x.elevationSource]?.id} · {x.attestors.length}/{x.required}
              </span>
              <span className="land-tl-s">{GEOMETRY_SOURCES[x.source]?.label}</span>
            </Link>
          ))}
        </div>
      </div>
    </div>
  )
}
