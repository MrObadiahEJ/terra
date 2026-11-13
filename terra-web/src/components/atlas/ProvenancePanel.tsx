// Provenance panel — shows evidence lineage for the selected asset.
// Honest about what is verified vs simulated: parcels, geometry digests,
// and review state are shown as they exist; nothing is self-declared
// "verified."
import { FileText, ShieldCheck, Link } from 'lucide-react'
import { useLocale, type TranslationKey } from '../../lib/locale'

interface EvidenceEntry {
  step: string
  actor: string
  action: string
  status: 'pending' | 'verified' | 'rejected' | 'simulated' | 'claimed' | 'vision'
  time: string
}

const DEMO_EVIDENCE: EvidenceEntry[] = [
  { step: '1', actor: 'submitter', action: 'Parcel registered', status: 'claimed', time: '2026-09-01' },
  { step: '2', actor: 'surveyor', action: 'Geometry digest computed', status: 'simulated', time: '2026-09-02' },
  { step: '3', actor: 'validator', action: 'Conformance review', status: 'pending', time: '—' },
  { step: '4', actor: 'consensus', action: 'Rights anchor', status: 'vision', time: '—' },
]

const STATUS_LABEL: Record<string, TranslationKey> = {
  pending: 'atlasEvPending',
  verified: 'atlasEvVerified',
  rejected: 'atlasEvRejected',
  simulated: 'atlasEvSimulated',
}

export default function ProvenancePanel() {
  const { t } = useLocale()

  return (
    <div className="atlas-window-body">
      <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 14 }}>
        <FileText size={14} className="text-muted" />
        <span style={{ fontWeight: 600, fontSize: 13, color: 'var(--ink)' }}>{t('atlasWindowProvenance')}</span>
      </div>

      <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
        {DEMO_EVIDENCE.map((entry) => (
          <div
            key={entry.step}
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: 10,
              padding: '8px 10px',
              border: '1px solid var(--glass-border)',
              borderRadius: 10,
              background: 'color-mix(in srgb, var(--ink) 6%, transparent)',
            }}
          >
            <div style={{ width: 26, height: 26, borderRadius: '50%', background: 'color-mix(in srgb, var(--ink) 15%, transparent)', display: 'flex', alignItems: 'center', justifyContent: 'center', fontSize: 11, fontWeight: 600, color: 'var(--muted)', flexShrink: 0 }}>
              {entry.step}
            </div>
            <div style={{ flex: 1, minWidth: 0 }}>
              <div style={{ fontSize: 12, color: 'var(--ink)', fontWeight: 500 }}>{t(`atlasEvActor${entry.actor}` as TranslationKey)}</div>
              <div style={{ fontSize: 11, color: 'var(--muted)', fontFamily: 'var(--font-mono)' }}>{entry.action}</div>
            </div>
            <span
              className={`atlas-rights-badge ${entry.status === 'simulated' ? 'sim' : entry.status === 'pending' ? 'proto' : entry.status === 'verified' ? 'claimed' : 'vision'}`}
              style={{ fontSize: 9, padding: '2px 6px' }}
            >
              {t(STATUS_LABEL[entry.status])}
            </span>
            <div style={{ fontSize: 10, color: 'var(--muted)', fontFamily: 'var(--font-mono)', whiteSpace: 'nowrap' }}>{entry.time}</div>
          </div>
        ))}
      </div>

      <div style={{ marginTop: 14, paddingTop: 10, borderTop: '1px solid var(--glass-border)', display: 'flex', gap: 8, flexWrap: 'wrap' }}>
        <button type="button" className="btn btn-ghost btn-p-1">
          <ShieldCheck size={14} /> {t('atlasEvReview')}
        </button>
        <button type="button" className="btn btn-ghost btn-p-1">
          <Link size={14} /> {t('atlasEvExport')}
        </button>
      </div>

      <div style={{ marginTop: 12, fontSize: 10, color: 'var(--muted)', fontFamily: 'var(--font-mono)', lineHeight: 1.4 }}>
        <AtlasHonestyLabelInline mode="simulation" />
        {' · Provenance is evidence-based, not self-declared. Geometry digests, review state, and rights anchors are shown as they exist; nothing here claims verification that has not occurred.'}
      </div>
    </div>
  )
}

function AtlasHonestyLabelInline({ mode = 'simulation' }: { mode?: 'simulation' | 'local' | 'prototype' | 'vision' }) {
  const labels: Record<string, string> = {
    simulation: 'simulation',
    local: 'local',
    prototype: 'prototype',
    vision: 'vision',
  }
  return <span className="atlas-honesty">{labels[mode]}</span>
}
