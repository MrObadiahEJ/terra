import { useCallback, useEffect, useRef, useState } from 'react'
import {
  AlertTriangle,
  Check,
  Download,
  Loader2,
  Play,
  RefreshCw,
  ShieldCheck,
  Wallet,
} from 'lucide-react'
import { LAMPORTS_PER_SOL } from '@solana/web3.js'
import { useWallet, connection } from '../../lib/wallet'
import { TERRA_CLUSTER, TERRA_IS_LOCALNET, TERRA_PROGRAM_ID } from '../../lib/constants'
import { runLifecycleStep, proofBlob, proofFileName, type LifecycleProof } from '../../lib/lifecycle'
import {
  LIFECYCLE_STEPS,
  getOrCreateLifecycleSeed,
  loadLifecycleProgress,
  resetLifecycleSession,
  saveLifecycleProgress,
  type LifecycleProgress,
  type LifecycleStepId,
  type LifecycleTx,
} from '../../lib/lifecyclePlan'

function shortSignature(signature: string): string {
  return `${signature.slice(0, 8)}…${signature.slice(-8)}`
}

function stepStatus(
  step: LifecycleStepId,
  progress: LifecycleProgress,
  runningStep: LifecycleStepId | null,
  failedStep: LifecycleStepId | null,
): string {
  if (runningStep === step) return 'running'
  if (failedStep === step) return 'error'
  return progress.done[step] ? 'done' : 'pending'
}

export default function ChainLifecyclePanel() {
  const { publicKey, connected, anchorWallet, walletName } = useWallet()
  const [seed] = useState(() => getOrCreateLifecycleSeed())
  const [progress, setProgress] = useState<LifecycleProgress>(() =>
    loadLifecycleProgress(getOrCreateLifecycleSeed()),
  )
  const progressRef = useRef(progress)
  const [runningStep, setRunningStep] = useState<LifecycleStepId | null>(null)
  const [runningAll, setRunningAll] = useState(false)
  const [failedStep, setFailedStep] = useState<LifecycleStepId | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [proof, setProof] = useState<LifecycleProof | null>(null)
  const [balance, setBalance] = useState<number | null>(null)
  const [funding, setFunding] = useState(false)

  const commitProgress = useCallback((next: LifecycleProgress) => {
    progressRef.current = next
    setProgress(next)
    saveLifecycleProgress(next)
  }, [])

  const refreshBalance = useCallback(async () => {
    if (!publicKey) {
      setBalance(null)
      return
    }
    try {
      setBalance(await connection.getBalance(publicKey))
    } catch {
      setBalance(null)
    }
  }, [publicKey])

  useEffect(() => {
    void refreshBalance()
  }, [refreshBalance])

  const downloadProof = useCallback((nextProof: LifecycleProof) => {
    const url = URL.createObjectURL(proofBlob(nextProof))
    const anchor = document.createElement('a')
    anchor.href = url
    anchor.download = proofFileName(nextProof)
    document.body.appendChild(anchor)
    anchor.click()
    anchor.remove()
    setTimeout(() => URL.revokeObjectURL(url), 0)
  }, [])

  const runOne = useCallback(
    async (step: LifecycleStepId, announceDone = true): Promise<boolean> => {
      if (!anchorWallet) {
        setError('Connect a wallet before running the on-chain lifecycle.')
        return false
      }
      setRunningStep(step)
      setFailedStep(null)
      setError(null)
      try {
        const result = await runLifecycleStep(step, {
          connection,
          wallet: anchorWallet,
          progress: progressRef.current,
        })
        const current = progressRef.current
        const merged: LifecycleTx[] = [...(current.txs[step] ?? []), ...result.txs]
        const next: LifecycleProgress = {
          ...current,
          done: { ...current.done, [step]: true },
          txs: { ...current.txs, [step]: merged },
          updatedAt: Date.now(),
        }
        commitProgress(next)
        if (result.proof) {
          setProof(result.proof)
          downloadProof(result.proof)
        }
        if (announceDone) void refreshBalance()
        return true
      } catch (cause) {
        const message = cause instanceof Error ? cause.message : String(cause)
        setFailedStep(step)
        setError(`${LIFECYCLE_STEPS.find((entry) => entry.id === step)?.title ?? step}: ${message}`)
        return false
      } finally {
        setRunningStep(null)
      }
    },
    [anchorWallet, commitProgress, downloadProof, refreshBalance],
  )

  const runAll = useCallback(async () => {
    setRunningAll(true)
    setProof(null)
    try {
      for (const step of LIFECYCLE_STEPS) {
        const ok = await runOne(step.id, false)
        if (!ok) break
      }
    } finally {
      setRunningAll(false)
      void refreshBalance()
    }
  }, [refreshBalance, runOne])

  const fundWallet = useCallback(async () => {
    if (!publicKey) return
    setFunding(true)
    setError(null)
    try {
      const signature = await connection.requestAirdrop(publicKey, 100 * LAMPORTS_PER_SOL)
      await connection.confirmTransaction(signature, 'confirmed')
      await refreshBalance()
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause))
    } finally {
      setFunding(false)
    }
  }, [publicKey, refreshBalance])

  const resetSession = useCallback(() => {
    if (!window.confirm('Reset the local lifecycle session? On-chain accounts are left untouched.')) {
      return
    }
    resetLifecycleSession()
    const freshSeed = getOrCreateLifecycleSeed()
    const fresh = loadLifecycleProgress(freshSeed)
    progressRef.current = fresh
    setProgress(fresh)
    setProof(null)
    setFailedStep(null)
    setError(null)
    // The deterministic keys change with the seed; a reload re-derives them.
    window.location.reload()
  }, [])

  const doneCount = LIFECYCLE_STEPS.filter((step) => progress.done[step.id]).length
  const busy = runningAll || runningStep !== null
  const lowBalance = balance !== null && balance < 10 * LAMPORTS_PER_SOL

  return (
    <section className="chain-lifecycle" data-testid="chain-lifecycle">
      <header className="chain-lifecycle-head">
        <div>
          <span className="backend-kicker">
            <ShieldCheck size={13} /> SPRINT B · ON-CHAIN LIFECYCLE
          </span>
          <h2>Parcel → evidence → paid verification → proof → dispute</h2>
          <p>
            Every step builds Anchor instructions from the checked-in IDL in this browser and lands
            them on <strong>{TERRA_CLUSTER}</strong>. No server-prepared transactions, no simulated
            signatures.
          </p>
        </div>
        <div className="chain-lifecycle-meta">
          <span className={`chain-lifecycle-wallet${connected ? ' connected' : ''}`}>
            <Wallet size={13} />
            {publicKey ? `${publicKey.toBase58().slice(0, 5)}…${publicKey.toBase58().slice(-4)}` : 'Connect a wallet'}
          </span>
          <span>{walletName ?? 'Wallet adapter'}</span>
          <span>{balance === null ? 'balance —' : `${(balance / LAMPORTS_PER_SOL).toFixed(2)} SOL`}</span>
        </div>
      </header>

      <div className="chain-lifecycle-toolbar">
        <div className="chain-lifecycle-progress">
          <span>{doneCount}/{LIFECYCLE_STEPS.length} steps complete</span>
          <div className="chain-lifecycle-track">
            <i style={{ width: `${(doneCount / LIFECYCLE_STEPS.length) * 100}%` }} />
          </div>
          <code>{TERRA_PROGRAM_ID.slice(0, 6)}…{TERRA_PROGRAM_ID.slice(-6)}</code>
        </div>
        <div className="chain-lifecycle-actions">
          {TERRA_IS_LOCALNET && (
            <button
              type="button"
              className="btn btn-ghost"
              onClick={() => void fundWallet()}
              disabled={funding || !publicKey}
              data-testid="lifecycle-fund-wallet"
            >
              {funding ? <RefreshCw size={14} className="animate-spin" /> : <Wallet size={14} />}
              Local faucet
            </button>
          )}
          <button type="button" className="btn btn-ghost" onClick={resetSession} disabled={busy}>
            Reset session
          </button>
          <button
            type="button"
            className="btn btn-primary"
            onClick={() => void runAll()}
            disabled={busy || !anchorWallet}
            data-testid="lifecycle-run-all"
          >
            {runningAll ? <RefreshCw size={14} className="animate-spin" /> : <Play size={14} />}
            {runningAll ? 'Running on chain…' : 'Run full lifecycle'}
          </button>
        </div>
      </div>

      {lowBalance && balance !== null && TERRA_IS_LOCALNET && (
        <div className="chain-lifecycle-note">
          <AlertTriangle size={14} />
          <span>
            This wallet holds {(balance / LAMPORTS_PER_SOL).toFixed(2)} SOL. The local faucet button
            tops it up; each lifecycle run needs roughly 0.35 SOL for rent and fees.
          </span>
        </div>
      )}

      {error && (
        <div className="chain-lifecycle-error" data-testid="lifecycle-error">
          <AlertTriangle size={15} />
          <span>{error}</span>
        </div>
      )}

      <ol className="chain-lifecycle-steps">
        {LIFECYCLE_STEPS.map((step, index) => {
          const status = stepStatus(step.id, progress, runningStep, failedStep)
          const txs = progress.txs[step.id] ?? []
          return (
            <li
              key={step.id}
              className={`chain-lifecycle-step ${status}`}
              data-testid={`lifecycle-step-${step.id}`}
            >
              <div className="chain-lifecycle-index">{String(index + 1).padStart(2, '0')}</div>
              <div className="chain-lifecycle-body">
                <div className="chain-lifecycle-title">
                  <strong>{step.title}</strong>
                  <span className={`chain-lifecycle-status ${status}`} data-testid={`lifecycle-status-${step.id}`}>
                    {status === 'done' && <Check size={11} />}
                    {status === 'running' && <Loader2 size={11} className="animate-spin" />}
                    {status}
                  </span>
                </div>
                <p>{step.description}</p>
                {txs.length > 0 && (
                  <details className="chain-lifecycle-txs">
                    <summary>{txs.length} transaction{txs.length === 1 ? '' : 's'}</summary>
                    <ul>
                      {txs.map((entry, txIndex) => (
                        <li key={`${entry.signature}-${txIndex}`}>
                          <span>{entry.label}</span>
                          <code title={entry.signature}>{shortSignature(entry.signature)}</code>
                        </li>
                      ))}
                    </ul>
                  </details>
                )}
                {step.id === 'proof' && proof && (
                  <button
                    type="button"
                    className="chain-lifecycle-proof"
                    onClick={() => downloadProof(proof)}
                    data-testid="lifecycle-proof-download"
                  >
                    <Download size={13} /> {proofFileName(proof)}
                  </button>
                )}
              </div>
              <button
                type="button"
                className="btn btn-ghost chain-lifecycle-run"
                onClick={() => void runOne(step.id)}
                disabled={busy || !anchorWallet}
                data-testid={`lifecycle-run-${step.id}`}
              >
                {runningStep === step.id ? (
                  <RefreshCw size={13} className="animate-spin" />
                ) : (
                  <Play size={13} />
                )}
                {status === 'done' ? 'Re-run' : 'Run'}
              </button>
            </li>
          )
        })}
      </ol>

      <footer className="chain-lifecycle-foot">
        <span>
          Session {seed.slice(0, 8)} · local operator keys (two validators + tenant) are derived from
          a browser seed and funded from the cluster faucet. They are test operators, but every
          write above is real.
        </span>
        {proof && (
          <span>
            Proof exported at {new Date(proof.exportedAt).toLocaleTimeString()} ·{' '}
            {Object.values(proof.transactions).reduce((count, entries) => count + (entries?.length ?? 0), 0)} signatures
          </span>
        )}
      </footer>
    </section>
  )
}
