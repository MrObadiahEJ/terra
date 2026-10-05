import { LOD_LEVELS } from '../../lib/lod'

export default function LodLadder({ lod }: { lod: number }) {
  const active = LOD_LEVELS[lod] ?? LOD_LEVELS[0]
  return (
    <div className="lod-ladder" aria-label="Level of detail">
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
            <span className="lod-dot" />
            <span className="lod-name">{level.name}</span>
          </li>
        ))}
      </ol>
      <div className="lod-reveals">{active.reveals}</div>
    </div>
  )
}
