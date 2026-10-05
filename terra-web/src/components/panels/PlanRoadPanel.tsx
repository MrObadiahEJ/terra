import { useMemo } from 'react'
import { Check, Grid3x3, Magnet, Route, Square, Trash2, Undo2 } from 'lucide-react'
import { lineLengthM, type LonLat } from '../../lib/geo'
import type { PlannedRoad } from '../../lib/atlasLayers'
import type { DrawVertex } from '../map/TerraGlobe'

interface Props {
  drawing: boolean
  drawKind: 'parcel' | 'road'
  drawVertices: DrawVertex[]
  plannedRoads: PlannedRoad[]
  onToggleDrawing: () => void
  onClearDrawing: () => void
  onUndoVertex: () => void
  onFinishDrawing: () => void
  onRemoveRoad: (id: string) => void
  snapGeom: boolean
  snapGrid: boolean
  onToggleSnap: (key: 'geom' | 'grid') => void
}

function fmtDist(m: number): string {
  return m >= 1000 ? `${(m / 1000).toFixed(2)} km` : `${m.toFixed(1)} m`
}

export default function PlanRoadPanel({
  drawing,
  drawKind,
  drawVertices,
  plannedRoads,
  onToggleDrawing,
  onClearDrawing,
  onUndoVertex,
  onFinishDrawing,
  onRemoveRoad,
  snapGeom,
  snapGrid,
  onToggleSnap,
}: Props) {
  const active = drawing && drawKind === 'road'
  const n = drawVertices.length
  const lengthM = useMemo(() => {
    if (n < 2) return 0
    return lineLengthM(drawVertices.map((v) => [v.lon, v.lat] as LonLat))
  }, [drawVertices, n])

  const totalLength = useMemo(
    () => plannedRoads.reduce((sum, road) => sum + road.lengthM, 0),
    [plannedRoads],
  )

  return (
    <div className="p-3 text-sm space-y-3">
      <div className="flex items-center justify-between">
        <h3 className="font-semibold">Road planning</h3>
        <span className="text-[11px] text-muted">
          {plannedRoads.length} planned · {fmtDist(totalLength)}
        </span>
      </div>

      <div className="flex gap-2">
        <button className="btn btn-secondary flex-1 justify-center" onClick={onToggleDrawing}>
          {active ? <Square size={14} /> : <Route size={14} />}
          {active ? 'Stop drawing' : 'Draw road'}
        </button>
        <button
          className="btn btn-ghost"
          onClick={onClearDrawing}
          disabled={n === 0}
        >
          Clear
        </button>
      </div>

      {active && (
        <p className="text-[12px] text-muted">
          Click the map to drop points · double-click (or click the first point) to finish ·
          Backspace undoes the last point. Roads stay open — they are not closed rings.
        </p>
      )}

      {n >= 2 && (
        <div className="text-[12px] text-muted flex gap-3">
          <span>
            Length <b>{fmtDist(lengthM)}</b>
          </span>
          <span>
            Points <b>{n}</b>
          </span>
        </div>
      )}

      {n > 0 && (
        <div className="flex gap-2">
          <button className="btn btn-secondary flex-1 justify-center" onClick={onUndoVertex}>
            <Undo2 size={14} />
            Undo
          </button>
          <button
            className="btn btn-secondary flex-1 justify-center"
            onClick={onFinishDrawing}
            disabled={!active || n < 2}
          >
            <Check size={14} />
            Finish road
          </button>
        </div>
      )}

      {active && (
        <div className="flex gap-2 text-[11px]">
          <button
            className={`btn btn-ghost px-2 py-1 gap-1 ${snapGeom ? 'text-emerald-700' : 'text-muted'}`}
            onClick={() => onToggleSnap('geom')}
            title="Snap points to parcel boundaries"
          >
            <Magnet size={12} />
            Snap
          </button>
          <button
            className={`btn btn-ghost px-2 py-1 gap-1 ${snapGrid ? 'text-emerald-700' : 'text-muted'}`}
            onClick={() => onToggleSnap('grid')}
            title="Round free picks to a 1e-6° grid"
          >
            <Grid3x3 size={12} />
            Grid
          </button>
        </div>
      )}

      {plannedRoads.length > 0 && (
        <div className="space-y-1.5">
          <h4 className="text-[12px] font-semibold">Planned alignments</h4>
          {plannedRoads.map((road) => (
            <div key={road.id} className="plan-row">
              <div className="min-w-0">
                <div className="font-medium truncate">{road.name}</div>
                <div className="plan-meta">
                  {fmtDist(road.lengthM)} · {road.vertices.length} points
                </div>
              </div>
              <button
                className="btn btn-ghost p-1 shrink-0"
                onClick={() => onRemoveRoad(road.id)}
                title="Remove road"
              >
                <Trash2 size={13} />
              </button>
            </div>
          ))}
        </div>
      )}
    </div>
  )
}
