import { LOD_LEVELS } from '../../lib/lod'
import { useLocale } from '../../lib/locale'

interface LodLadderProps {
  lod: number
  onSelect?: (lod: number) => void
}

export default function LodLadder({ lod, onSelect }: LodLadderProps) {
  const { t } = useLocale()
  const active = LOD_LEVELS[lod] ?? LOD_LEVELS[0]
  return (
    <div className={`lod-ladder ${onSelect ? 'lod-ladder--live' : ''}`} aria-label={t('lodAria')}>
      <div className="lod-head">
        <span>{t('zoomDetail')}</span>
        <b>LOD {active.id}</b>
      </div>
      <ol className="lod-rungs">
        {LOD_LEVELS.map((level) => (
          <li
            key={level.id}
            className={`lod-rung ${level.id === lod ? 'is-active' : level.id < lod ? 'is-past' : ''}`}
          >
            {onSelect ? (
              <button
                type="button"
                onClick={() => onSelect(level.id)}
                title={t('jumpToLod', { name: t(level.nameKey) })}
              >
                <span className="lod-dot" />
                <span className="lod-name">{t(level.nameKey)}</span>
              </button>
            ) : (
              <>
                <span className="lod-dot" />
                <span className="lod-name">{t(level.nameKey)}</span>
              </>
            )}
          </li>
        ))}
      </ol>
      <div className="lod-reveals">{t(active.revealsKey)}</div>
    </div>
  )
}
