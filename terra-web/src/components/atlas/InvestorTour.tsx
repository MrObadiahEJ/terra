// Guided investor tour — 6-stop narrative overlay.
// Each stop flies the camera, opens the relevant window, and shows a
// plain-language caption + honest status chip. Skippable, replayable,
// reduced-motion safe.
import { useCallback } from 'react'
import { Sparkles, Compass, ArrowRight, X } from 'lucide-react'
import { useAtlasStore } from '../../lib/atlasStore'
import { useLocale, type TranslationKey } from '../../lib/locale'

interface TourStop {
  id: string
  title: TranslationKey
  caption: TranslationKey
  status: TranslationKey
  statusChip: 'claimed' | 'proto' | 'sim' | 'vision'
  action?: () => void
}

const TOUR_STOPS: TourStop[] = [
  {
    id: 'claim',
    title: 'atlasTourStep1Title',
    caption: 'atlasTourStep1Caption',
    status: 'atlasTourStep1Status',
    statusChip: 'sim',
  },
  {
    id: 'evidence',
    title: 'atlasTourStep2Title',
    caption: 'atlasTourStep2Caption',
    status: 'atlasTourStep2Status',
    statusChip: 'proto',
  },
  {
    id: 'validators',
    title: 'atlasTourStep3Title',
    caption: 'atlasTourStep3Caption',
    status: 'atlasTourStep3Status',
    statusChip: 'vision',
  },
  {
    id: 'consensus',
    title: 'atlasTourStep4Title',
    caption: 'atlasTourStep4Caption',
    status: 'atlasTourStep4Status',
    statusChip: 'vision',
  },
  {
    id: 'history',
    title: 'atlasTourStep5Title',
    caption: 'atlasTourStep5Caption',
    status: 'atlasTourStep5Status',
    statusChip: 'claimed',
  },
  {
    id: 'asset',
    title: 'atlasTourStep6Title',
    caption: 'atlasTourStep6Caption',
    status: 'atlasTourStep6Status',
    statusChip: 'sim',
  },
]

const STATUS_LABEL: Record<string, TranslationKey> = {
  claimed: 'atlasStatusClaimed',
  proto: 'atlasStatusPrototype',
  sim: 'atlasStatusSimulation',
  vision: 'atlasStatusVision',
}

export default function InvestorTour() {
  const { t } = useLocale()
  const { tourActive, tourStep, endTour, nextTourStep } = useAtlasStore(
    (s) => ({ tourActive: s.tourActive, tourStep: s.tourStep, endTour: s.endTour, nextTourStep: s.nextTourStep }),
  )
  const prefersReduced = typeof window !== 'undefined' && window.matchMedia('(prefers-reduced-motion: reduce)').matches

  const handleSkip = useCallback(() => endTour(), [endTour])
  const handleNext = useCallback(() => {
    if (tourStep < TOUR_STOPS.length - 1) nextTourStep()
    else endTour()
  }, [tourStep, nextTourStep, endTour])

  if (!tourActive) return null

  const stop = TOUR_STOPS[tourStep]
  const progress = ((tourStep) / (TOUR_STOPS.length - 1)) * 100

  return (
    <div className="atlas-tour" role="dialog" aria-modal="true" aria-label="Investor tour">
      <div className="atlas-tour-card">
        <div className="atlas-tour-header">
          <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
            <Sparkles size={16} className="text-muted" />
            <span style={{ fontWeight: 600, fontSize: 14, color: 'var(--ink)' }}>{t('atlasTourTitle')}</span>
          </div>
          <button
            type="button"
            className="atlas-window-btn"
            aria-label="Skip tour"
            onClick={handleSkip}
          >
            <X size={14} />
          </button>
        </div>
        <div className="atlas-tour-progress">
          <div className="atlas-tour-progress-fill" style={{ width: `${progress}%` }} />
        </div>
        <div className="atlas-tour-body">
          <div className="atlas-tour-stop">
            <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', gap: 8, marginBottom: 12 }}>
              <Compass size={20} className="text-muted" />
              <span style={{ fontWeight: 600, fontSize: 12, color: 'var(--muted)' }}>
                {tourStep + 1} / {TOUR_STOPS.length}
              </span>
            </div>
            <h3>{t(stop.title)}</h3>
            <p>{t(stop.caption)}</p>
            <div style={{ marginTop: 10, display: 'flex', alignItems: 'center', gap: 8, justifyContent: 'center' }}>
              <span
                className={`atlas-rights-badge ${stop.statusChip}`}
                style={{ fontSize: 9, padding: '3px 8px' }}
              >
                {t(STATUS_LABEL[stop.statusChip])}
              </span>
              <span style={{ fontSize: 11, color: 'var(--muted)', fontFamily: 'var(--font-mono)' }}>{t(stop.status)}</span>
            </div>
          </div>
        </div>
        <div className="atlas-tour-actions">
          <button type="button" className="btn btn-ghost btn-p-1" onClick={handleSkip} disabled={prefersReduced && tourStep === TOUR_STOPS.length - 1}>
            {t('atlasTourSkip')}
          </button>
          <button
            type="button"
            className="btn btn-primary"
            onClick={handleNext}
            disabled={tourStep === TOUR_STOPS.length - 1}
          >
            {tourStep === TOUR_STOPS.length - 1 ? t('atlasTourFinish') : t('atlasTourNext')} <ArrowRight size={12} style={{ marginLeft: 6 }} />
          </button>
        </div>
      </div>
    </div>
  )
}
