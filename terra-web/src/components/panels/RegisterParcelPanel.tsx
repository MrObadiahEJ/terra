import { useEffect, useState } from 'react'
import { Transaction } from '@solana/web3.js'
import { useWallet } from '../../lib/wallet'
import { useAppStore } from '../../store/appStore'
import { getProgram, parcelPda } from '../../lib/program'
import { api, type OffChainParcel, type ReachabilityResult } from '../../lib/api'
import { bytesToHex } from '../../lib/codec'
import {
  sha256Hex,
  polygonAreaM2,
  polygonCentroid,
  polygonPerimeterM,
  lineLengthM,
  validateRing,
  type LonLat,
} from '../../lib/geo'
import { reportTx } from '../../lib/txStore'
import type { DrawVertex } from '../map/TerraGlobe'
import { PencilRuler, Square, Loader2, Undo2, Check, Magnet, Grid3x3 } from 'lucide-react'

interface Props {
  drawing: boolean
  drawVertices: DrawVertex[]
  onToggleDrawing: () => void
  onClearDrawing: () => void
  onUndoVertex: () => void
  onRemoveVertex: (index: number) => void
  onFinishDrawing: () => void
  snapGeom: boolean
  snapGrid: boolean
  onToggleSnap: (key: 'geom' | 'grid') => void
}

function fmtDist(m: number): string {
  return m >= 1000 ? `${(m / 1000).toFixed(2)} km` : `${m.toFixed(1)} m`
}

function fmtArea(m2: number): string {
  return m2 >= 10000 ? `${(m2 / 10000).toFixed(2)} ha` : `${m2.toFixed(1)} m²`
}

export default function RegisterParcelPanel({
  drawing,
  drawVertices,
  onToggleDrawing,
  onClearDrawing,
  onUndoVertex,
  onRemoveVertex,
  onFinishDrawing,
  snapGeom,
  snapGrid,
  onToggleSnap,
}: Props) {
  const { publicKey, send } = useWallet()
  const refreshParcels = useAppStore((s) => s.refreshParcels)
  const refreshOffChain = useAppStore((s) => s.refreshOffChain)
  const setLastSignature = useAppStore((s) => s.setLastSignature)

  const [name, setName] = useState('')
  const [busy, setBusy] = useState(false)
  const [msg, setMsg] = useState<string | null>(null)
  const [err, setErr] = useState<string | null>(null)
  const [report, setReport] = useState<ReachabilityResult | null>(null)

  const addLocalParcel = useAppStore((s) => s.addLocalParcel)

  const n = drawVertices.length
  const pts = drawVertices.map((v) => [v.lon, v.lat] as LonLat)
  const perimeter = n >= 3 ? polygonPerimeterM(pts) : n === 2 ? lineLengthM(pts) : 0
  const area = n >= 3 ? polygonAreaM2(pts) : 0
  const validation = n >= 3 ? validateRing(pts) : null
  const canSubmit = name.trim() !== '' && validation?.ok === true

  // Backspace undoes the last vertex while drawing (but not while typing).
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key !== 'Backspace' || !drawing) return
      const t = e.target as HTMLElement | null
      if (t && (t.tagName === 'INPUT' || t.tagName === 'TEXTAREA' || t.isContentEditable)) return
      e.preventDefault()
      onUndoVertex()
    }
    window.addEventListener('keydown', onKey)
    return () => window.removeEventListener('keydown', onKey)
  }, [drawing, onUndoVertex])

  /** Closed ring [lon, lat] from the drawn vertices. */
  const buildRing = (): LonLat[] => {
    const ring = drawVertices.map((v) => [v.lon, v.lat] as LonLat)
    ring.push(ring[0])
    return ring
  }

  /**
   * Offline demo registration (no wallet / no API): same real SHA-256 hashes
   * and geometry metrics as the on-chain flow, stored on this device and
   * shown immediately on the globe.
   */
  const onDemoRegister = async () => {
    setBusy(true)
    setMsg(null)
    setErr(null)
    setReport(null)
    try {
      const ring = buildRing()
      const geometry = { type: 'Polygon', coordinates: [ring] }

      // Same derivation as the on-chain flow: id over the geometry, hash over
      // the ring — both real SHA-256, computed locally.
      const idHex = await sha256Hex(JSON.stringify(geometry))
      const geometryHash = await sha256Hex(JSON.stringify(ring))
      const area = polygonAreaM2(ring)
      const centroid = polygonCentroid(ring)
      const now = new Date().toISOString()

      // Best-effort persistence when the API happens to be up.
      let savedToApi = false
      try {
        await api.createParcel({
          name: name.trim(),
          holder: 'demo:local',
          status: 'registered',
          geometry,
        })
        savedToApi = true
        await refreshOffChain()
      } catch {
        // offline — the local record below is enough for the demo
      }

      const local: OffChainParcel = {
        id: idHex,
        name: name.trim(),
        holder: 'demo:local',
        status: 'registered',
        geometry: JSON.stringify(geometry),
        area_m2: area,
        created_at: now,
        updated_at: now,
      }
      addLocalParcel(local)

      const demoMsg =
        `Demo parcel stored ${savedToApi ? 'locally + API' : 'on this device'} · ` +
        `${area.toFixed(0)} m² · centroid ${centroid[1].toFixed(6)}, ${centroid[0].toFixed(6)} · ` +
        `sha256 ${geometryHash.slice(0, 16)}…`
      setMsg(demoMsg)
      reportTx('register_parcel', true, demoMsg)
      onClearDrawing()
      setName('')
    } catch (e) {
      setErr(e instanceof Error ? e.message : 'Demo registration failed')
    } finally {
      setBusy(false)
    }
  }

  const onRegister = async () => {
    if (!publicKey) return
    setBusy(true)
    setMsg(null)
    setErr(null)
    setReport(null)
    try {
      // Build a closed ring [lon, lat] from the drawn vertices.
      const ring = buildRing()
      const geometry = { type: 'Polygon', coordinates: [ring] }

      // 1) 32-byte parcel id = sha256 over the geometry (stable, unique).
      const idBytes = await crypto.subtle.digest('SHA-256', new TextEncoder().encode(JSON.stringify(geometry)))
      const id = new Uint8Array(idBytes)
      const idHex = bytesToHex(id)

      // 2) geometry hash = sha256 over the ring (what we anchor).
      const geoHash = await crypto.subtle.digest(
        'SHA-256',
        new TextEncoder().encode(JSON.stringify(ring)),
      )
      const geometryHash = new Uint8Array(geoHash)

      const program = getProgram()
      const [pda] = parcelPda(id)
      const ix = await program.methods
        .registerParcel(Array.from(id), name.trim(), Array.from(geometryHash))
        .accounts({ parcel: pda, owner: publicKey } as never)
        .instruction()

      const sig = await send(new Transaction().add(ix))
      setLastSignature(sig)

      // 3) Run off-chain reachability analysis to derive infra flags + digest.
      try {
        const reach = await api.reachability(idHex, geometry)
        setReport(reach)
      } catch {
        // reachability only available when the server loaded OSM data
      }

      // 4) Persist the parcel + geometry off-chain (PostGIS) for display.
      try {
        await api.createParcel({
          name: name.trim(),
          holder: publicKey.toBase58(),
          status: 'registered',
          geometry,
        })
        await refreshOffChain()
      } catch (e) {
        setErr(
          `On-chain registered (${sig.slice(0, 12)}…), but off-chain save failed: ${
            e instanceof Error ? e.message : e
          }`,
        )
      }

      const regMsg = `Parcel registered on-chain: ${sig.slice(0, 12)}…`
      setMsg(regMsg)
      reportTx('register_parcel', true, regMsg, { sig, source: 'wallet' })
      await refreshParcels()
      onClearDrawing()
      setName('')
    } catch (e) {
      setErr(e instanceof Error ? e.message : 'Registration failed')
    } finally {
      setBusy(false)
    }
  }

  return (
    <div className="p-3 text-sm space-y-3">
      <div className="flex items-center justify-between">
        <h3 className="font-semibold">Register a parcel</h3>
        <span className="text-[11px] text-muted">{drawVertices.length} vertices</span>
      </div>

      {!publicKey && (
        <p className="text-[12px] text-amber-700">
          No wallet? Use the demo register below — real SHA-256 hashes and geometry metrics,
          stored on this device.
        </p>
      )}

      <div className="flex gap-2">
        <button className="btn btn-secondary flex-1 justify-center" onClick={onToggleDrawing}>
          {drawing ? <Square size={14} /> : <PencilRuler size={14} />}
          {drawing ? 'Stop drawing' : 'Draw parcel'}
        </button>
        <button
          className="btn btn-ghost"
          onClick={onClearDrawing}
          disabled={drawVertices.length === 0}
        >
          Clear
        </button>
      </div>

      {drawing && (
        <p className="text-[12px] text-muted">
          Click the map to add corners · double-click (or click the first corner) to close ·
          Backspace undoes the last corner.
        </p>
      )}

      {n >= 2 && (
        <div className="text-[12px] text-muted flex gap-3">
          <span>
            Perimeter <b>{fmtDist(perimeter)}</b>
          </span>
          {n >= 3 && (
            <span>
              Area <b>{fmtArea(area)}</b>
            </span>
          )}
        </div>
      )}

      {n > 0 && (
        <div className="border rounded max-h-36 overflow-y-auto text-[11px] font-mono">
          {drawVertices.map((v, i) => (
            <div
              key={i}
              className="flex items-center justify-between gap-2 px-2 py-0.5 border-b last:border-b-0"
            >
              <span className="truncate">
                #{i + 1} {v.lat.toFixed(7)}, {v.lon.toFixed(7)}
              </span>
              <button
                className="btn btn-ghost p-0.5 shrink-0"
                onClick={() => onRemoveVertex(i)}
                title="Remove vertex"
              >
                ✕
              </button>
            </div>
          ))}
        </div>
      )}

      {n > 0 && (
        <div className="flex gap-2">
          <button className="btn btn-secondary flex-1 justify-center" onClick={onUndoVertex}>
            <Undo2 size={14} />
            Undo
          </button>
          <button
            className="btn btn-secondary flex-1 justify-center"
            onClick={onFinishDrawing}
            disabled={!drawing || n < 3}
          >
            <Check size={14} />
            Close ring
          </button>
        </div>
      )}

      {drawing && (
        <div className="flex gap-2 text-[11px]">
          <button
            className={`btn btn-ghost px-2 py-1 gap-1 ${snapGeom ? 'text-emerald-700' : 'text-muted'}`}
            onClick={() => onToggleSnap('geom')}
            title="Snap corners and edges to parcel boundaries"
          >
            <Magnet size={12} />
            Snap
          </button>
          <button
            className={`btn btn-ghost px-2 py-1 gap-1 ${snapGrid ? 'text-emerald-700' : 'text-muted'}`}
            onClick={() => onToggleSnap('grid')}
            title="Round free picks to a 1e-6° grid"
          >
            <Grid3x3 size={12} />
            Grid
          </button>
        </div>
      )}

      {validation && !validation.ok && (
        <p className="text-red-700 text-[12px]">{validation.error}</p>
      )}

      <input
        className="text-input"
        placeholder="Parcel name"
        value={name}
        onChange={(e) => setName(e.target.value)}
      />

      <button
        className="btn btn-primary w-full justify-center"
        disabled={!canSubmit || busy}
        onClick={publicKey ? onRegister : onDemoRegister}
      >
        {busy ? <Loader2 size={14} className="animate-spin" /> : null}
        {busy
          ? 'Registering…'
          : publicKey
            ? 'Register on-chain'
            : 'Demo register (local)'}
      </button>

      {report && (
        <div className="border rounded p-2 text-[12px] space-y-1">
          <h4 className="font-semibold">Road-access report</h4>
          <p>Nearest road: <b>{Math.round(report.nearest_road_m)} m</b></p>
          <p>Boundary accesses: <b>{report.boundary_accesses}</b></p>
          <p>Network component: <b>{report.component_km.toFixed(2)} km</b></p>
          <p>
            Sealed reachable:{' '}
            <b>{report.sealed_reachable ? 'Yes' : 'No'}</b>
          </p>
          <p className="font-mono text-[10px] break-all">
            infra: 0b{report.flags.toString(2)}
          </p>
        </div>
      )}

      {msg && <p className="text-emerald-700 text-[12px]">{msg}</p>}
      {err && <p className="text-red-700 text-[12px]">{err}</p>}
    </div>
  )
}
