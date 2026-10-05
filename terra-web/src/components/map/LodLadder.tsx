import { LOD_LEVELS } from '../../lib/lod'

interface LodLadderProps {
  lod: number
  onSelect?: (lod: number) => void
}

export default function LodLadder({ lod, onSelect }: LodLadderProps) {
  const active = LOD_LEVELS[lod] ?? LOD_LEVELS[0]
  return (
    <div className={`lod-ladder ${onSelect ? 'lod-ladder--live' : ''}`} aria-label="Level of detail">
      <div className="lod-head">
        <span>Zoom detail</span>
        <b>LOD {active.id}</b>
      </div>
      <ol className="lod-rungs">
        {LOD_LEVELS.map((level) => (
          <li
            key={level.id}
            className={`lod-rung ${level.id === lod ? 'is-active' : level.id < lod ? 'is-past' : ''}`}
          >
            {onSelect ? (
              <button type="button" onClick={() => onSelect(level.id)} title={`Jump to ${level.name}`}>
                <span className="lod-dot" />
                <span className="lod-name">{level.name}</span>
              </button>
            ) : (
              <>
                <span className="lod-dot" />
                <span className="lod-name">{level.name}</span>
              </>
            )}
          </li>
        ))}
      </ol>
      <div className="lod-reveals">{active.reveals}</div>
    </div>
  )
}
