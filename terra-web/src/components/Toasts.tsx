import { useEffect, useState } from 'react'
import { Link } from 'react-router-dom'
import { X } from 'lucide-react'
import type { DemoTx } from '../lib/txStore'
import { useTxStore } from '../lib/txStore'
import { useLocale } from '../lib/locale'

export default function Toasts() {
  const { t } = useLocale()
  const [tx, setTx] = useState<DemoTx | null>(null)

  useEffect(() => {
    let timer: ReturnType<typeof setTimeout> | undefined
    const unsub = useTxStore.subscribe((state, prev) => {
      const newest = state.txs[0]
      if (state.txs.length !== prev.txs.length && newest && newest !== prev.txs[0]) {
        setTx(newest)
        if (timer) clearTimeout(timer)
        timer = setTimeout(() => setTx(null), 5000)
      }
    })
    return () => {
      unsub()
      if (timer) clearTimeout(timer)
    }
  }, [])

  if (!tx) return null

  return (
    <div className={`tx-toast ${tx.status}`} role="status">
      <span className={`tx-dot ${tx.status}`} />
      <div className="tx-toast-body">
        <span className="tx-toast-title">
          {tx.status === 'confirmed' ? t('confirmedWord') : t('failedWord')} · {tx.instruction}
        </span>
        <span className="tx-toast-sum">{tx.summary}</span>
        <Link className="tx-toast-link" to="/transactions" onClick={() => setTx(null)}>
          {t('viewInExplorer')}
        </Link>
      </div>
      <button className="tx-toast-x" onClick={() => setTx(null)} aria-label={t('dismissLabel')}>
        <X size={13} />
      </button>
    </div>
  )
}
