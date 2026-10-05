import { useCallback, useEffect, useMemo, useState } from 'react'
import { Coins, FileText, RefreshCw, Scale } from 'lucide-react'
import { api, type Dispute, type Escrow, type OffChainParcel } from '../../lib/api'
import { DEFAULT_FOCUS } from '../../lib/constants'
import { useLocale, type TranslationKey } from '../../lib/locale'
import { useWallet } from '../../lib/wallet'

type Translate = (key: TranslationKey, vars?: Record<string, string | number>) => string

const ESC_STATUS_KEY: Record<string, TranslationKey> = {
  created: 'escStCreated',
  deposited: 'escStDeposited',
  accepted: 'escStAccepted',
  settled: 'escStSettled',
  cancelled: 'escStCancelled',
  disputed: 'escStDisputed',
}

const ESC_STATUS_BADGE: Record<string, string> = {
  created: 'lab-badge-info',
  deposited: 'lab-badge-info',
  accepted: 'lab-badge-warn',
  settled: 'lab-badge-ok',
  cancelled: 'lab-badge-mut',
  disputed: 'lab-badge-err',
}

const DST_STATUS_KEY: Record<string, TranslationKey> = {
  filed: 'dstFiled',
  frozen: 'dstFrozen',
  adjudicated: 'dstAdjudicated',
  executed: 'dstExecuted',
  cancelled: 'dstCancelled',
}

const DST_STATUS_BADGE: Record<string, string> = {
  filed: 'lab-badge-info',
  frozen: 'lab-badge-warn',
  adjudicated: 'lab-badge-warn',
  executed: 'lab-badge-ok',
  cancelled: 'lab-badge-mut',
}

const DEMO_SELLER = 'AKnL4NNf3DGWZJS6cPknBuEGnVsV4A4m5tgebLHaRSZ9'
const DEMO_BUYER = '9hSR6S7WPtxmTojgo6GG3k4yDPecgJY292j7xrsUGWBu'
const DEMO_AUTHORITY = 'GyGKxMyg1p9SsHfm15MkNUu1u9TN2JtTspcdmrtGUdse'
const DEMO_VALIDATORS = [
  'EdmxWPmx2WH6WgFfTdu9xfkYf3k1g5wD1zccTVySEEh1',
  '8SFqwqnq4whPhs8icwHA2hQg3hUoN1qrCLK1SBx3WKwe',
  'AKkzLhjhyFtM9j7WAhbaqYpFe49cXeJBg2kzLRC2PnNa',
]

function errMsg(err: unknown): string {
  return err instanceof Error ? err.message : String(err)
}

function randomCaseHash(): string {
  const bytes = new Uint8Array(32)
  crypto.getRandomValues(bytes)
  return Array.from(bytes, (b) => b.toString(16).padStart(2, '0')).join('')
}

function shortAddr(addr: string): string {
  return addr.length > 16 ? `${addr.slice(0, 4)}…${addr.slice(-4)}` : addr
}

function fmtAmount(lamports: number): string {
  return (lamports / 1e9).toLocaleString('en-US', { maximumFractionDigits: 3 })
}

function fmtIso(iso: string | null): string {
  if (!iso) return '—'
  return iso.replace('T', ' ').slice(0, 19) + ' UTC'
}

function fmtDur(ms: number, t: Translate): string {
  const total = Math.max(0, Math.floor(ms / 1000))
  const d = Math.floor(total / 86400)
  const h = Math.floor((total % 86400) / 3600)
  const m = Math.floor((total % 3600) / 60)
  const s = total % 60
  if (d > 0) return t('durD', { d, h })
  if (h > 0) return t('durH', { h, m })
  if (m > 0) return t('durM', { m, s })
  return t('durS', { s })
}

function statusLabel(keys: Record<string, TranslationKey>, status: string, t: Translate): string {
  const key = keys[status]
  return key ? t(key) : status
}

function squareGeometry(offset: number): { type: 'Polygon'; coordinates: number[][][] } {
  const lon = DEFAULT_FOCUS.longitude + offset
  const lat = DEFAULT_FOCUS.latitude
  const d = 0.0004
  const ring: number[][] = [
    [lon - d, lat - d],
    [lon + d, lat - d],
    [lon + d, lat + d],
    [lon - d, lat + d],
    [lon - d, lat - d],
  ]
  return { type: 'Polygon', coordinates: [ring] }
}

function parseValidators(raw: string): string[] {
  return raw
    .split(',')
    .map((v) => v.trim())
    .filter((v) => v.length > 0)
}

export default function SettlementCenter() {
  const { t } = useLocale()
  const { publicKey } = useWallet()
  const walletAddr = publicKey ? publicKey.toBase58() : null

  const [escrows, setEscrows] = useState<Escrow[]>([])
  const [disputes, setDisputes] = useState<Dispute[]>([])
  const [parcels, setParcels] = useState<OffChainParcel[]>([])
  const [ready, setReady] = useState(false)
  const [busy, setBusy] = useState<string | null>(null)
  const [err, setErr] = useState<string | null>(null)
  const [ok, setOk] = useState<string | null>(null)
  const [now, setNow] = useState(() => Date.now())

  const [mode, setMode] = useState<'new' | 'existing'>('new')
  const [listingName, setListingName] = useState('')
  const [eParcelId, setEParcelId] = useState('')
  const [sellerRaw, setSellerRaw] = useState<string | null>(null)
  const [buyerRaw, setBuyerRaw] = useState<string | null>(null)
  const [priceSol, setPriceSol] = useState('1')

  const [dParcelId, setDParcelId] = useState('')
  const [filerRaw, setFilerRaw] = useState<string | null>(null)
  const [validatorsRaw, setValidatorsRaw] = useState(() => DEMO_VALIDATORS.join(', '))
  const [requiredRaw, setRequiredRaw] = useState('2')
  const [caseHash, setCaseHash] = useState(() => randomCaseHash())

  const [adjId, setAdjId] = useState<string | null>(null)
  const [adjOutcome, setAdjOutcome] = useState<'owner_wins' | 'owner_loses'>('owner_wins')
  const [adjAuthorityRaw, setAdjAuthorityRaw] = useState<string | null>(null)
  const [adjNewOwnerRaw, setAdjNewOwnerRaw] = useState<string | null>(null)

  const seller = (sellerRaw ?? walletAddr ?? DEMO_SELLER).trim()
  const buyer = (buyerRaw ?? DEMO_BUYER).trim()
  const filer = (filerRaw ?? walletAddr ?? DEMO_SELLER).trim()
  const authority = (adjAuthorityRaw ?? walletAddr ?? DEMO_AUTHORITY).trim()
  const newOwner = (adjNewOwnerRaw ?? DEMO_BUYER).trim()

  const refresh = useCallback(async (): Promise<string | null> => {
    const [e, d, p] = await Promise.allSettled([
      api.listEscrows(),
      api.listDisputes(),
      api.listParcels(),
    ])
    if (e.status === 'fulfilled') setEscrows(e.value)
    if (d.status === 'fulfilled') setDisputes(d.value)
    if (p.status === 'fulfilled') setParcels(p.value)
    const failed = [e, d, p].find((r) => r.status === 'rejected')
    if (failed && failed.status === 'rejected') return errMsg(failed.reason)
    return null
  }, [])

  useEffect(() => {
    let live = true
    const init = async () => {
      const loadErr = await refresh()
      if (!live) return
      setErr(loadErr)
      setReady(true)
    }
    void init()
    return () => {
      live = false
    }
  }, [refresh])

  const hasWindows = escrows.some(
    (e) => e.status === 'created' || e.status === 'deposited' || e.status === 'accepted',
  )

  useEffect(() => {
    if (!hasWindows) return
    const timer = setInterval(() => setNow(Date.now()), 1000)
    return () => clearInterval(timer)
  }, [hasWindows])

  const fail = (msg: string) => {
    setOk(null)
    setErr(msg)
  }

  const run = async (id: string, action: () => Promise<unknown>) => {
    setErr(null)
    setOk(null)
    setBusy(id)
    try {
      await action()
      const loadErr = await refresh()
      if (loadErr) setErr(loadErr)
      else setOk(t('stlActionOk'))
    } catch (e) {
      setErr(errMsg(e))
    } finally {
      setBusy(null)
    }
  }

  const onRefresh = () => {
    setErr(null)
    setOk(null)
    void refresh().then(setErr)
  }

  const forSaleParcels = useMemo(
    () => parcels.filter((p) => p.status === 'for_sale' && p.holder === seller),
    [parcels, seller],
  )

  const parcelById = useMemo(() => new Map(parcels.map((p) => [p.id, p])), [parcels])

  const nameOf = (id: string) => parcelById.get(id)?.name ?? `${id.slice(0, 8)}…`

  const onOpenEscrow = () => {
    const amount = Math.round(Number(priceSol.replace(',', '.')) * 1e9)
    if (mode === 'new' && !listingName.trim()) return fail(t('stlErrName'))
    if (!seller || !buyer) return fail(t('stlErrWallet'))
    if (seller === buyer) return fail(t('stlPartiesDiffer'))
    if (!Number.isFinite(amount) || amount < 100_000_000 || amount > 1_000_000_000_000) {
      return fail(t('stlErrAmount'))
    }
    if (mode === 'existing' && !eParcelId) return fail(t('stlErrParcel'))
    void run('escrow:new', async () => {
      let parcelId = eParcelId
      if (mode === 'new') {
        const parcel = await api.createParcel({
          name: listingName.trim(),
          holder: seller,
          status: 'for_sale',
          geometry: squareGeometry(parcels.length * 0.0006),
        })
        parcelId = parcel.id
        setListingName('')
      }
      await api.createEscrow({ parcel_id: parcelId, seller, buyer, amount })
    })
  }

  const onFileDispute = () => {
    const validators = parseValidators(validatorsRaw)
    const required = Number(requiredRaw)
    if (!dParcelId) return fail(t('stlErrParcel'))
    if (!filer) return fail(t('stlErrWallet'))
    if (validators.length < 2) return fail(t('stlErrValidators'))
    if (!Number.isInteger(required) || required < 2 || required > validators.length) {
      return fail(t('stlErrRequired'))
    }
    if (!/^[0-9a-f]{64}$/i.test(caseHash)) return fail(t('stlErrHash'))
    void run('dispute:new', async () => {
      await api.fileDispute(dParcelId, {
        case_hash: caseHash.toLowerCase(),
        required,
        validators,
        filer,
      })
      setCaseHash(randomCaseHash())
    })
  }

  const onAdjudicate = (d: Dispute) => {
    if (!authority) return fail(t('stlErrWallet'))
    if (adjOutcome === 'owner_loses' && !newOwner) return fail(t('stlErrWallet'))
    void run(`dst:${d.id}:adj`, async () => {
      await api.adjudicateDispute(
        d.id,
        adjOutcome === 'owner_loses'
          ? { outcome: adjOutcome, authority, new_owner: newOwner }
          : { outcome: adjOutcome, authority },
      )
      setAdjId(null)
    })
  }

  return (
    <div className="stl">
      <p className="stl-intro">{t('stlIntro')}</p>

      {err && (
        <div className="stl-msg err">
          <b>{t('stlActionFailed')}</b> <code>{err}</code>
        </div>
      )}
      {ok && <div className="stl-msg ok">{ok}</div>}

      <div className="net-row">
        <section className="lab-card">
          <div className="stl-head">
            <h3>
              <Coins size={14} /> {t('stlEscrowsHeading')}
            </h3>
            <button className="btn btn-ghost p-1" disabled={busy !== null} onClick={onRefresh}>
              <RefreshCw size={12} /> {t('stlRefresh')}
            </button>
          </div>

          <div className="stl-form">
            <div className="stl-modes">
              <button
                className={`stl-mode ${mode === 'new' ? 'active' : ''}`}
                onClick={() => setMode('new')}
              >
                {t('stlModeNew')}
              </button>
              <button
                className={`stl-mode ${mode === 'existing' ? 'active' : ''}`}
                onClick={() => setMode('existing')}
              >
                {t('stlModeExisting')}
              </button>
            </div>

            <div className="stl-form-row">
              {mode === 'new' ? (
                <label className="stl-field">
                  {t('stlParcelName')}
                  <input
                    className="text-input"
                    value={listingName}
                    onChange={(ev) => setListingName(ev.target.value)}
                  />
                </label>
              ) : (
                <label className="stl-field">
                  {t('stlParcel')}
                  <select
                    className="select-input"
                    value={eParcelId}
                    onChange={(ev) => setEParcelId(ev.target.value)}
                  >
                    <option value="">{t('stlParcel')}</option>
                    {forSaleParcels.map((p) => (
                      <option key={p.id} value={p.id}>
                        {p.name}
                      </option>
                    ))}
                  </select>
                </label>
              )}
              <label className="stl-field">
                {t('stlSeller')}
                <input
                  className="text-input"
                  value={seller}
                  onChange={(ev) => setSellerRaw(ev.target.value)}
                />
              </label>
              <label className="stl-field">
                {t('stlBuyer')}
                <input
                  className="text-input"
                  value={buyer}
                  onChange={(ev) => setBuyerRaw(ev.target.value)}
                />
              </label>
              <label className="stl-field">
                {t('stlPrice')}
                <input
                  className="text-input"
                  inputMode="decimal"
                  value={priceSol}
                  onChange={(ev) => setPriceSol(ev.target.value)}
                />
              </label>
            </div>

            <div className="stl-form-foot">
              <button
                className="btn btn-primary"
                disabled={busy !== null}
                onClick={onOpenEscrow}
              >
                {mode === 'new' ? t('stlOpenBtn') : t('stlOpenExisting')}
              </button>
              <span className="stl-hint">{t('stlMinAmount')}</span>
            </div>

            {mode === 'existing' && forSaleParcels.length === 0 && (
              <p className="stl-warn">{t('stlNoForSale')}</p>
            )}
          </div>

          {!ready ? (
            <p className="stl-empty">{t('stlLoading')}</p>
          ) : escrows.length === 0 ? (
            <p className="stl-empty">{t('stlNoEscrows')}</p>
          ) : (
            escrows.map((e) => {
              const remaining = e.amount - e.deposit_amount
              const cancelAt = Date.parse(e.cancel_deadline)
              const settleAt = e.settle_deadline ? Date.parse(e.settle_deadline) : Number.NaN
              const canDeposit = e.status === 'created' && remaining > 0
              const canAccept = e.status === 'deposited' && e.deposit_amount >= e.amount
              const canCancel =
                (e.status === 'created' && e.deposit_amount === 0) ||
                (e.status === 'deposited' && now < cancelAt)
              const canExpire =
                e.status === 'created' && e.deposit_amount === 0 && now >= cancelAt
              const canSettle =
                e.status === 'accepted' && now >= settleAt && e.deposit_amount >= e.amount
              const canDispute =
                e.status === 'created' || e.status === 'deposited' || e.status === 'accepted'
              const canceller = e.status === 'created' ? e.seller : e.buyer
              return (
                <div className="stl-row" key={e.id}>
                  <div className="stl-row-top">
                    <span className={`lab-badge ${ESC_STATUS_BADGE[e.status] ?? 'lab-badge-info'}`}>
                      {statusLabel(ESC_STATUS_KEY, e.status, t)}
                    </span>
                    <span className="stl-row-title">{nameOf(e.parcel_id)}</span>
                    <span className="stl-meta">{fmtAmount(e.amount)} SOL</span>
                  </div>
                  <div className="stl-meta">
                    <span title={e.seller}>
                      {shortAddr(e.seller)} → {shortAddr(e.buyer)}
                    </span>
                    <span>{t('stlDepositedOf', { a: fmtAmount(e.deposit_amount), b: fmtAmount(e.amount) })}</span>
                    <span>
                      {t('stlCancelWindow')} {fmtIso(e.cancel_deadline)} ·{' '}
                      {now < cancelAt ? t('stlIn', { d: fmtDur(cancelAt - now, t) }) : t('stlWindowOpen')}
                    </span>
                    {e.settle_deadline && (
                      <span>
                        {t('stlSettleWindow')} {fmtIso(e.settle_deadline)} ·{' '}
                        {now < settleAt ? t('stlIn', { d: fmtDur(settleAt - now, t) }) : t('stlWindowOpen')}
                      </span>
                    )}
                    {e.dispute_case_hash && (
                      <span title={e.dispute_case_hash}>#{e.dispute_case_hash.slice(0, 8)}</span>
                    )}
                  </div>
                  <div className="stl-actions">
                    {canDeposit && (
                      <button
                        className="btn btn-primary p-1"
                        disabled={busy !== null}
                        onClick={() =>
                          void run(`esc:${e.id}:dep`, () =>
                            api.depositEscrow(e.id, {
                              escrow_id: e.id,
                              buyer: e.buyer,
                              deposit_amount: remaining,
                            }),
                          )
                        }
                      >
                        {t('stlDeposit')} {fmtAmount(remaining)} SOL
                      </button>
                    )}
                    {canAccept && (
                      <button
                        className="btn btn-primary p-1"
                        disabled={busy !== null}
                        onClick={() => void run(`esc:${e.id}:acc`, () => api.acceptEscrow(e.id, e.seller))}
                      >
                        {t('stlAccept')}
                      </button>
                    )}
                    {canSettle && (
                      <button
                        className="btn btn-primary p-1"
                        disabled={busy !== null}
                        onClick={() => void run(`esc:${e.id}:set`, () => api.settleEscrow(e.id, e.seller))}
                      >
                        {t('stlSettle')}
                      </button>
                    )}
                    {canCancel && (
                      <button
                        className="btn btn-secondary p-1"
                        disabled={busy !== null}
                        onClick={() =>
                          void run(`esc:${e.id}:can`, () => api.cancelEscrow(e.id, canceller, e.buyer))
                        }
                      >
                        {t('stlCancel')}
                      </button>
                    )}
                    {canExpire && (
                      <button
                        className="btn btn-secondary p-1"
                        disabled={busy !== null}
                        onClick={() => void run(`esc:${e.id}:exp`, () => api.expireEscrow(e.id, e.seller))}
                      >
                        {t('stlExpire')}
                      </button>
                    )}
                    {canDispute && (
                      <button
                        className="btn btn-secondary p-1"
                        disabled={busy !== null}
                        onClick={() =>
                          void run(`esc:${e.id}:dis`, () =>
                            api.disputeEscrow(e.id, {
                              case_hash: randomCaseHash(),
                              required: 2,
                              validators: DEMO_VALIDATORS,
                              filer: e.seller,
                            }),
                          )
                        }
                      >
                        {t('stlDispute')}
                      </button>
                    )}
                  </div>
                </div>
              )
            })
          )}
        </section>

        <section className="lab-card">
          <div className="stl-head">
            <h3>
              <Scale size={14} /> {t('stlDisputesHeading')}
            </h3>
            <button className="btn btn-ghost p-1" disabled={busy !== null} onClick={onRefresh}>
              <RefreshCw size={12} /> {t('stlRefresh')}
            </button>
          </div>

          <div className="stl-form">
            <div className="stl-form-row">
              <label className="stl-field">
                {t('stlParcel')}
                <select
                  className="select-input"
                  value={dParcelId}
                  onChange={(ev) => setDParcelId(ev.target.value)}
                >
                  <option value="">{t('stlParcel')}</option>
                  {parcels.map((p) => (
                    <option key={p.id} value={p.id}>
                      {p.name} · {p.status}
                    </option>
                  ))}
                </select>
              </label>
              <label className="stl-field">
                {t('stlFiler')}
                <input
                  className="text-input"
                  value={filer}
                  onChange={(ev) => setFilerRaw(ev.target.value)}
                />
              </label>
              <label className="stl-field">
                {t('stlRequired')}
                <input
                  className="text-input"
                  inputMode="numeric"
                  value={requiredRaw}
                  onChange={(ev) => setRequiredRaw(ev.target.value)}
                />
              </label>
            </div>

            <label className="stl-field">
              {t('stlValidators')}
              <input
                className="text-input"
                value={validatorsRaw}
                onChange={(ev) => setValidatorsRaw(ev.target.value)}
              />
            </label>

            <div className="stl-field">
              <span>{t('stlCaseHash')}</span>
              <div className="stl-hash">
                <input
                  className="text-input"
                  value={caseHash}
                  onChange={(ev) => setCaseHash(ev.target.value)}
                />
                <button className="btn btn-ghost p-1" onClick={() => setCaseHash(randomCaseHash())}>
                  <RefreshCw size={12} /> {t('stlRegen')}
                </button>
              </div>
            </div>

            <div className="stl-form-foot">
              <button
                className="btn btn-primary"
                disabled={busy !== null}
                onClick={onFileDispute}
              >
                <FileText size={13} /> {t('stlFileDispute')}
              </button>
            </div>

            {parcels.length === 0 && <p className="stl-warn">{t('stlNoParcels')}</p>}
          </div>

          {!ready ? (
            <p className="stl-empty">{t('stlLoading')}</p>
          ) : disputes.length === 0 ? (
            <p className="stl-empty">{t('stlNoDisputes')}</p>
          ) : (
            disputes.map((d) => (
              <div className="stl-row" key={d.id}>
                <div className="stl-row-top">
                  <span className={`lab-badge ${DST_STATUS_BADGE[d.status] ?? 'lab-badge-info'}`}>
                    {statusLabel(DST_STATUS_KEY, d.status, t)}
                  </span>
                  <span className="stl-row-title">{nameOf(d.parcel_id)}</span>
                  <span className="stl-meta">{t('stlEndorse', { c: d.count, r: d.required })}</span>
                </div>
                <div className="stl-meta">
                  <span title={d.filed_by}>{shortAddr(d.filed_by)}</span>
                  <span title={d.case_hash}>#{d.case_hash.slice(0, 8)}</span>
                  <span>{fmtIso(d.filed_at)}</span>
                  {d.outcome && (
                    <span>
                      {d.outcome === 'owner_loses' ? t('stlOwnerLoses') : t('stlOwnerWins')}
                    </span>
                  )}
                  {d.new_owner && (
                    <span title={d.new_owner}>→ {shortAddr(d.new_owner)}</span>
                  )}
                </div>
                <div className="stl-actions">
                  {d.status === 'filed' && (
                    <>
                      <button
                        className="btn btn-primary p-1"
                        disabled={busy !== null}
                        onClick={() => void run(`dst:${d.id}:frz`, () => api.freezeDispute(d.id))}
                      >
                        {t('stlFreeze')}
                      </button>
                      <button
                        className="btn btn-secondary p-1"
                        disabled={busy !== null}
                        onClick={() => void run(`dst:${d.id}:can`, () => api.cancelDispute(d.id))}
                      >
                        {t('stlCancelCase')}
                      </button>
                    </>
                  )}
                  {d.status === 'frozen' && (
                    <button
                      className="btn btn-primary p-1"
                      disabled={busy !== null}
                      onClick={() => setAdjId(adjId === d.id ? null : d.id)}
                    >
                      {t('stlAdjudicate')}
                    </button>
                  )}
                  {d.status === 'adjudicated' && (
                    <button
                      className="btn btn-primary p-1"
                      disabled={busy !== null}
                      onClick={() => void run(`dst:${d.id}:exe`, () => api.executeJudgment(d.id))}
                    >
                      {t('stlExecute')}
                    </button>
                  )}
                </div>

                {adjId === d.id && d.status === 'frozen' && (
                  <div className="stl-adj">
                    <div className="stl-form-row">
                      <label className="stl-field">
                        {t('stlOutcome')}
                        <select
                          className="select-input"
                          value={adjOutcome}
                          onChange={(ev) =>
                            setAdjOutcome(ev.target.value === 'owner_loses' ? 'owner_loses' : 'owner_wins')
                          }
                        >
                          <option value="owner_wins">{t('stlOwnerWins')}</option>
                          <option value="owner_loses">{t('stlOwnerLoses')}</option>
                        </select>
                      </label>
                      <label className="stl-field">
                        {t('stlAuthority')}
                        <input
                          className="text-input"
                          value={authority}
                          onChange={(ev) => setAdjAuthorityRaw(ev.target.value)}
                        />
                      </label>
                      {adjOutcome === 'owner_loses' && (
                        <label className="stl-field">
                          {t('stlNewOwner')}
                          <input
                            className="text-input"
                            value={newOwner}
                            onChange={(ev) => setAdjNewOwnerRaw(ev.target.value)}
                          />
                        </label>
                      )}
                    </div>
                    <div className="stl-form-foot">
                      <button
                        className="btn btn-primary p-1"
                        disabled={busy !== null}
                        onClick={() => onAdjudicate(d)}
                      >
                        {t('stlSubmitAdjudication')}
                      </button>
                    </div>
                  </div>
                )}
              </div>
            ))
          )}
        </section>
      </div>

      <p className="stl-foot">
        {t('stlMirrorNote')} {t('stlDemoWallets')}
      </p>
    </div>
  )
}
