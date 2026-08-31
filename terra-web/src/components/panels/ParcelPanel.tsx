import { useEffect, useState } from 'react'
import { PublicKey, Transaction } from '@solana/web3.js'
import { useWallet } from '../../lib/wallet'
import { useAppStore } from '../../store/appStore'
import {
  getProgram,
  parcelPda,
  rightsPda,
} from '../../lib/program'
import { bytesToHex } from '../../lib/codec'
import {
  PARCEL_STATUS,
  RIGHT_KINDS,
  infraLabels,
} from '../../lib/constants'
import type { ParcelAccount, RightsAccount } from '../../lib/program'

interface Props {
  address: string
  account: ParcelAccount
}

export default function ParcelPanel({ address, account }: Props) {
  const { publicKey, send } = useWallet()
  const setLastSignature = useAppStore((s) => s.setLastSignature)
  const refreshParcels = useAppStore((s) => s.refreshParcels)
  const [rights, setRights] = useState<RightsAccount[]>([])
  const [loadingRights, setLoadingRights] = useState(false)

  const [transferTo, setTransferTo] = useState('')
  const [transferBusy, setBusyTransfer] = useState(false)
  const [grantBusy, setBusyGrant] = useState(false)
  const [grantHolder, setGrantHolder] = useState('')
  const [grantKind, setGrantKind] = useState(1)
  const [grantNotes, setGrantNotes] = useState('')
  const [msg, setMsg] = useState<string | null>(null)
  const [err, setErr] = useState<string | null>(null)

  const isOwner = publicKey?.toBase58() === account.owner.toBase58()
  const idHex = bytesToHex(account.id)

  const loadRights = async () => {
    if (!publicKey) return
    setLoadingRights(true)
    try {
      const program = getProgram()
      const count = account.rightsCount
      const items: RightsAccount[] = []
      for (let i = 0; i < count; i++) {
        try {
          const [pda] = rightsPda(new PublicKey(address), i)
          const acc = (await program.account.rights.fetch(pda)) as unknown as RightsAccount
          items.push(acc)
        } catch {
          // ignore missing accounts
        }
      }
      setRights(items)
    } catch (e) {
      setErr(e instanceof Error ? e.message : 'Failed to load rights')
    } finally {
      setLoadingRights(false)
    }
  }

  useEffect(() => {
    if (publicKey) {
      // defer so we don't setState synchronously within the effect body
      const t = setTimeout(loadRights, 0)
      return () => clearTimeout(t)
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [publicKey, account])

  const run =
    (setBusy: (v: boolean) => void, fn: () => Promise<string>) =>
    async () => {
      setMsg(null)
      setErr(null)
      setBusy(true)
      try {
        const sig = await fn()
        setLastSignature(sig)
        setMsg(`Transaction confirmed: ${sig.slice(0, 12)}…`)
        await refreshParcels()
        await loadRights()
      } catch (e) {
        setErr(e instanceof Error ? e.message : 'Transaction failed')
      } finally {
        setBusy(false)
      }
    }

  const onTransfer = run(setBusyTransfer, async () => {
    const to = new PublicKey(transferTo)
    const program = getProgram()
    const [pda] = parcelPda(new Uint8Array(account.id))
    const ix = await program.methods
      .transferParcel()
      .accounts({ parcel: pda, owner: publicKey!, newOwner: to } as never)
      .instruction()
    return send(new Transaction().add(ix))
  })

  const onGrant = run(setBusyGrant, async () => {
    const holder = new PublicKey(grantHolder)
    const program = getProgram()
    const [pda] = parcelPda(new Uint8Array(account.id))
    const [rPda] = rightsPda(new PublicKey(address), account.rightsCount)
    const ix = await program.methods
      .grantRight(
        account.rightsCount,
        grantKind,
        holder,
        0n,
        grantNotes,
      )
      .accounts({
        parcel: pda,
        rights: rPda,
        owner: publicKey!,
        systemProgram: PublicKey.default,
      } as never)
      .instruction()
    return send(new Transaction().add(ix))
  })

  return (
    <div className="p-3 text-sm space-y-3">
      <div className="flex flex-wrap gap-2 justify-between items-center">
        <h3 className="font-semibold text-base truncate" title={account.name}>
          {account.name}
        </h3>
        <span className="text-[11px] px-2 py-0.5 rounded bg-amber-100 text-amber-800">
          {PARCEL_STATUS[account.status] ?? 'Unknown'}
        </span>
      </div>

      <dl className="grid grid-cols-[auto_1fr] gap-x-3 gap-y-1 text-[12px]">
        <dt className="text-muted">Owner</dt>
        <dd className="font-mono break-all">{account.owner.toBase58()}</dd>
        {isOwner && <dt className="text-muted">(you)</dt>}
        <dt className="text-muted">Parcel ID</dt>
        <dd className="font-mono break-all text-[10px]">{idHex}</dd>
        <dt className="text-muted">Geometry Hash</dt>
        <dd className="font-mono break-all text-[10px]">
          {bytesToHex(account.geometryHash).slice(0, 24)}…
        </dd>
        <dt className="text-muted">Created</dt>
        <dd>{new Date(Number(account.createdAt) * 1000).toLocaleString()}</dd>
      </dl>

      <div>
        <h4 className="font-semibold mb-1">Infrastructure</h4>
        {account.infrastructureFlags ? (
          <div className="flex flex-wrap gap-1">
            {infraLabels(account.infrastructureFlags).map((l) => (
              <span key={l} className="text-[11px] px-2 py-0.5 rounded bg-emerald-100 text-emerald-800">
                {l}
              </span>
            ))}
          </div>
        ) : (
          <p className="text-muted text-[12px]">No infrastructure flags set.</p>
        )}
      </div>

      <div>
        <h4 className="font-semibold mb-1">Rights ({account.rightsCount})</h4>
        {loadingRights ? (
          <p className="text-muted text-[12px]">Loading…</p>
        ) : rights.length === 0 ? (
          <p className="text-muted text-[12px]">No rights granted.</p>
        ) : (
          <ul className="space-y-1">
            {rights.map((r, i) => (
              <li key={i} className="text-[12px] border rounded p-2">
                <span className="inline-block bg-indigo-100 text-indigo-800 text-[10px] px-1.5 py-0.5 rounded mr-2">
                  {RIGHT_KINDS[r.rightsKind] ?? r.rightsKind}
                </span>
                <span className="font-mono text-[10px]">{r.holder.toBase58().slice(0, 10)}…</span>
                {r.notes && <p className="text-muted mt-1">{r.notes}</p>}
              </li>
            ))}
          </ul>
        )}
      </div>

      {isOwner && (
        <div className="space-y-3 border-t pt-3">
          <div>
            <h4 className="font-semibold mb-1">Transfer ownership</h4>
            <div className="flex gap-1">
              <input
                className="text-input"
                placeholder="New owner address"
                value={transferTo}
                onChange={(e) => setTransferTo(e.target.value)}
              />
              <button className="btn btn-primary" disabled={!transferTo || transferBusy} onClick={onTransfer}>
                {transferBusy ? '…' : 'Transfer'}
              </button>
            </div>
          </div>

          <div>
            <h4 className="font-semibold mb-1">Grant a right</h4>
            <div className="space-y-1">
              <input
                className="text-input"
                placeholder="Holder address"
                value={grantHolder}
                onChange={(e) => setGrantHolder(e.target.value)}
              />
              <div className="flex gap-1">
                <select
                  className="select-input flex-1"
                  value={grantKind}
                  onChange={(e) => setGrantKind(Number(e.target.value))}
                >
                  {Object.entries(RIGHT_KINDS).map(([k, v]) => (
                    <option key={k} value={k}>
                      {v}
                    </option>
                  ))}
                </select>
              </div>
              <input
                className="text-input"
                placeholder="Notes (optional)"
                value={grantNotes}
                onChange={(e) => setGrantNotes(e.target.value)}
              />
              <button
                className="btn btn-secondary w-full"
                disabled={!grantHolder || grantBusy}
                onClick={onGrant}
              >
                {grantBusy ? '…' : 'Grant right'}
              </button>
            </div>
          </div>
        </div>
      )}

      {msg && <p className="text-emerald-700 text-[12px]">{msg}</p>}
      {err && <p className="text-red-700 text-[12px]">{err}</p>}
    </div>
  )
}
