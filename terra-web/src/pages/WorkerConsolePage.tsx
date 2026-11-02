import { useState, useEffect, useCallback } from 'react'
import {
  AlertTriangle,
  ArrowUpRight,
  Check,
  Database,
  Play,
  RefreshCw,
  Terminal,
  Users,
} from 'lucide-react'
import { useWallet } from '../lib/wallet'
import {
  registerWorker,
  listWorkers,
  heartbeatWorker,
  listWorkerAssignments,
  completeAssignment,
  type WorkerRow,
  type AssignmentRow,
  type RegisterWorkerInput,
  type CompleteAssignmentInput,
} from '../lib/runtime'
import { useLocale } from '../lib/locale'

type ResultState = { operation: string; value: unknown; at: number } | null

export default function WorkerConsolePage() {
  const { publicKey } = useWallet()
  const { t } = useLocale()

  // ---- Worker registration state ----
  const [workerId, setWorkerId] = useState('')
  const [walletAddress, setWalletAddress] = useState('')
  const [capabilities, setCapabilities] = useState(
    JSON.stringify(
      { cpu_cores: 4, gpu: true, gpu_vram_mb: 8192, storage_mb: 50000, bandwidth_mbps: 100, regions: [], engines: ['photogrammetry', 'lidar'] },
      null,
      2,
    ),
  )
  const [busy, setBusy] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [result, setResult] = useState<ResultState>(null)

  // ---- Worker listing state ----
  const [workers, setWorkers] = useState<WorkerRow[]>([])
  const [workersLoading, setWorkersLoading] = useState(false)

  // ---- Assignment state ----
  const [assignments, setAssignments] = useState<AssignmentRow[]>([])
  const [selectedWorkerId, setSelectedWorkerId] = useState<string | null>(null)

  // ---- Assignment completion state ----
  const [completeJobId, setCompleteJobId] = useState('')
  const [completeResultUri, setCompleteResultUri] = useState('')
  const [completeResourceReceipt, setCompleteResourceReceipt] = useState(
    JSON.stringify({ cpu_seconds: 120, gpu_seconds: 60, storage_mb: 500, bandwidth_mb: 50 }),
  )

  const loadWorkers = useCallback(async () => {
    setWorkersLoading(true)
    setError(null)
    try {
      const rows = await listWorkers()
      setWorkers(rows)
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : t('backendRequestFailed'))
    } finally {
      setWorkersLoading(false)
    }
  }, [])

  const register = async (event: React.FormEvent) => {
    event.preventDefault()
    setError(null)
    setResult(null)
    setBusy(true)
    try {
      const input: RegisterWorkerInput = {
        worker_id: workerId.trim(),
        wallet_address: walletAddress.trim(),
        capabilities: JSON.parse(capabilities),
      }
      if (!input.worker_id || !input.wallet_address) {
        throw new Error(t('workerIdAndWalletRequired'))
      }
      const row = await registerWorker(input)
      setResult({ operation: 'POST /workers', value: row, at: Date.now() })
      setWorkerId('')
      setWalletAddress('')
      setCapabilities(JSON.stringify({ cpu_cores: 4, gpu: true, gpu_vram_mb: 8192, storage_mb: 50000, bandwidth_mbps: 100, regions: [], engines: ['photogrammetry', 'lidar'] }, null, 2))
      await loadWorkers()
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : t('backendRequestFailed'))
    } finally {
      setBusy(false)
    }
  }

  const loadAssignments = async (workerId: string) => {
    try {
      const rows = await listWorkerAssignments(workerId)
      setAssignments(rows)
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : t('backendRequestFailed'))
    }
  }

  const completeJob = async (event: React.FormEvent) => {
    event.preventDefault()
    setError(null)
    setResult(null)
    setBusy(true)
    try {
      const input: CompleteAssignmentInput = {}
      if (completeResultUri.trim()) input.result_uri = completeResultUri.trim()
      if (completeResourceReceipt.trim()) input.resource_receipt = JSON.parse(completeResourceReceipt)
      const row = await completeAssignment(completeJobId.trim(), input)
      setResult({ operation: 'POST /assignments/:job_id/complete', value: row, at: Date.now() })
      setCompleteJobId('')
      setCompleteResultUri('')
      setCompleteResourceReceipt(JSON.stringify({ cpu_seconds: 120, gpu_seconds: 60, storage_mb: 500, bandwidth_mb: 50 }))
      await loadAssignments(selectedWorkerId || '')
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : t('backendRequestFailed'))
    } finally {
      setBusy(false)
    }
  }

  const heartbeat = async (worker: WorkerRow) => {
    setError(null)
    setBusy(true)
    try {
      const row = await heartbeatWorker(worker.id)
      setResult({ operation: 'POST /workers/:id/heartbeat', value: row, at: Date.now() })
      await loadWorkers()
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : t('backendRequestFailed'))
    } finally {
      setBusy(false)
    }
  }

  useEffect(() => {
    loadWorkers()
  }, [loadWorkers])

  const capabilitiesError = (() => {
    try { JSON.parse(capabilities); return null } catch { return t('invalidJson') }
  })()
  const receiptError = (() => {
    try { completeResourceReceipt.trim() && JSON.parse(completeResourceReceipt); return null } catch { return t('invalidJson') }
  })()

  return (
    <main className="worker-console">
      <header className="worker-console-head">
        <div>
          <span className="worker-kicker"><Users size={13} /> TERRA</span>
          <h1>{t('workerConsoleTitle')}</h1>
          <p>{t('workerConsoleDescription')}</p>
        </div>
        <span className="worker-api-badge"><span /> {t('consoleApiBadge')}</span>
      </header>

      {error && <div className="worker-error"><AlertTriangle size={14} /> {error}</div>}

      <div className="worker-console-grid">
        {/* Worker registration panel */}
        <section className="worker-panel">
          <div className="worker-panel-head">
            <span className="worker-panel-label"><Users size={12} /> {t('workerRegisterTitle')}</span>
          </div>
          <form className="worker-form" onSubmit={register}>
            <label className="worker-field">
              <span>{t('workerId')}</span>
              <input className="text-input" value={workerId} onChange={(e) => setWorkerId(e.target.value)} placeholder="worker-alpha-1" required />
            </label>
            <label className="worker-field">
              <span>{t('walletAddress')}</span>
              <input className="text-input" value={walletAddress} onChange={(e) => setWalletAddress(e.target.value)} placeholder={publicKey ? publicKey.toBase58() : 'connect wallet'} required />
            </label>
            <label className="worker-field">
              <span>{t('capabilities')}</span>
              <textarea className="worker-json-input" value={capabilities} onChange={(e) => setCapabilities(e.target.value)} spellCheck={false} placeholder='{"cpu_cores": 4, "gpu": true}' />
              {capabilitiesError && <span className="worker-field-error">{capabilitiesError}</span>}
            </label>
            <button type="submit" className="btn btn-primary" disabled={busy || !!capabilitiesError || !workerId.trim() || !walletAddress.trim()}>
              {busy ? <RefreshCw size={14} className="animate-spin" /> : <Play size={14} />}{t('workerRegisterAction')}
            </button>
          </form>
          {result && (
            <div className="worker-result success">
              <span className="worker-success"><Check size={13} /> {t('workerRegistered')}</span>
              <pre>{JSON.stringify(result.value, null, 2)}</pre>
            </div>
          )}
        </section>

        {/* Workers list */}
        <section className="worker-panel">
          <div className="worker-panel-head">
            <span className="worker-panel-label"><Database size={12} /> {t('workersTitle')}</span>
            <button className="btn btn-ghost btn-small" onClick={loadWorkers} disabled={workersLoading}>
              <RefreshCw size={12} /> {t('consoleRefresh')}
            </button>
          </div>
          {workersLoading && <p className="worker-loading">{t('consoleLoading')}</p>}
          {!workersLoading && workers.length === 0 && <p className="worker-empty">{t('workerNoWorkers')}</p>}
          {!workersLoading && workers.length > 0 && (
            <table className="worker-table">
              <thead>
                <tr>
                  <th>{t('workerId')}</th>
                  <th>{t('walletAddress')}</th>
                  <th>{t('status')}</th>
                  <th>{t('lastHeartbeat')}</th>
                  <th>{t('actions')}</th>
                </tr>
              </thead>
              <tbody>
                {workers.map((worker) => (
                  <tr key={worker.id}>
                    <td className="worker-id-cell">{worker.worker_id}</td>
                    <td className="worker-wallet-cell">{worker.wallet_address.slice(0, 8)}…{worker.wallet_address.slice(-4)}</td>
                    <td>
                      <span className={`worker-status-badge worker-status-${worker.status}`}>{worker.status}</span>
                    </td>
                    <td>{worker.last_heartbeat_at ? new Date(worker.last_heartbeat_at).toLocaleTimeString() : t('workerNever')}</td>
                    <td>
                      <button className="btn btn-ghost btn-small" onClick={() => heartbeat(worker)} disabled={busy}>
                        <RefreshCw size={12} /> {t('workerHeartbeat')}
                      </button>
                      <button className="btn btn-ghost btn-small" onClick={() => setSelectedWorkerId(worker.worker_id)}>
                        <ArrowUpRight size={12} /> {t('workerViewAssignments')}
                      </button>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}
        </section>

        {/* Assignments for selected worker */}
        {selectedWorkerId && assignments.length > 0 && (
          <section className="worker-panel">
            <div className="worker-panel-head">
              <span className="worker-panel-label"><Terminal size={12} /> {t('workerAssignmentsTitle', { id: selectedWorkerId.slice(0, 8) })}</span>
            </div>
            <table className="worker-table">
              <thead>
                <tr>
                  <th>{t('jobId')}</th>
                  <th>{t('assignedAt')}</th>
                  <th>{t('completedAt')}</th>
                  <th>{t('resultUri')}</th>
                  <th>{t('resourceReceipt')}</th>
                  <th>{t('actions')}</th>
                </tr>
              </thead>
              <tbody>
                {assignments.map((assignment) => (
                  <tr key={assignment.id}>
                    <td className="worker-id-cell">{assignment.job_id}</td>
                    <td>{assignment.assigned_at ? new Date(assignment.assigned_at).toLocaleTimeString() : '-'}</td>
                    <td>{assignment.completed_at ? new Date(assignment.completed_at).toLocaleTimeString() : t('workerPending')}</td>
                    <td className="worker-id-cell">{assignment.result_uri || '-'}</td>
                    <td className="worker-json-cell">{assignment.resource_receipt ? JSON.stringify(assignment.resource_receipt) : '-'}</td>
                    <td>
                      {!assignment.completed_at && (
                        <button className="btn btn-ghost btn-small" onClick={() => setCompleteJobId(assignment.job_id)}>
                          {t('workerCompleteJob')}
                        </button>
                      )}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
            {assignments.some((a) => !a.completed_at) && (
              <div className="worker-complete-card">
                <label className="worker-field">
                  <span>{t('jobId')}</span>
                  <input className="text-input" value={completeJobId} onChange={(e) => setCompleteJobId(e.target.value)} readOnly />
                </label>
                <label className="worker-field">
                  <span>{t('resultUri')}</span>
                  <input className="text-input" value={completeResultUri} onChange={(e) => setCompleteResultUri(e.target.value)} placeholder="https://storage.example.com/result.glb" />
                </label>
                <label className="worker-field">
                  <span>{t('resourceReceipt')}</span>
                  <textarea className="worker-json-input" value={completeResourceReceipt} onChange={(e) => setCompleteResourceReceipt(e.target.value)} spellCheck={false} />
                  {receiptError && <span className="worker-field-error">{receiptError}</span>}
                </label>
                <button type="submit" className="btn btn-primary" disabled={busy || !!receiptError || !completeJobId.trim()} onClick={completeJob}>
                  {busy ? <RefreshCw size={14} className="animate-spin" /> : <Play size={14} />}{t('workerSubmitCompletion')}
                </button>
              </div>
            )}
          </section>
        )}

        {/* Result display */}
        {result && (
          <div className="worker-result success">
            <span className="worker-success"><Check size={13} /> {t('consoleResponse').toUpperCase()}</span>
            <h3>{result.operation}</h3>
            <pre>{JSON.stringify(result.value, null, 2)}</pre>
          </div>
        )}
      </div>
    </main>
  )
}