import { useState } from 'react'
import { Transaction } from '@solana/web3.js'
import { useWallet } from '../../lib/wallet'
import { useAppStore } from '../../store/appStore'
import { getProgram, parcelPda } from '../../lib/program'
import { api, type ReachabilityResult } from '../../lib/api'
import { bytesToHex } from '../../lib/codec'
import type { DrawVertex } from '../map/TerraGlobe'
import { PencilRuler, Square, Loader2 } from 'lucide-react'

interface Props {
  drawing: boolean
  drawVertices: DrawVertex[]
  onToggleDrawing: () => void
  onClearDrawing: () => void
}

export default function RegisterParcelPanel({
  drawing,
  drawVertices,
  onToggleDrawing,
  onClearDrawing,
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

  const canSubmit =
    publicKey !== null && name.trim() !== '' && drawVertices.length >= 3

  const onRegister = async () => {
    if (!publicKey) return
    setBusy(true)
    setMsg(null)
    setErr(null)
    setReport(null)
    try {
      // Build a closed ring [lon, lat] from the drawn vertices.
      const ring = [...drawVertices.map((v) => [v.lon, v.lat] as [number, number])]
      ring.push(ring[0])
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
          owner: publicKey.toBase58(),
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

      setMsg(`Parcel registered on-chain: ${sig.slice(0, 12)}…`)
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
        <p className="text-[12px] text-amber-700">Connect a wallet to register parcels on-chain.</p>
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
          Click on the globe to add polygon corners, then register.
        </p>
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
        onClick={onRegister}
      >
        {busy ? <Loader2 size={14} className="animate-spin" /> : null}
        {busy ? 'Registering…' : 'Register on-chain'}
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
