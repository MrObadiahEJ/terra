import { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import type { LonLat } from '../../lib/geo'
import { ELEVATION_SOURCES } from '../../lib/labStore'
import {
  cellHeights,
  fitProjector,
  normalizeRing,
  poly,
  projectIso,
  shade,
} from '../../lib/terrain'
import { RotateCcw, RotateCw, ZoomIn, ZoomOut, Maximize2 } from 'lucide-react'

// Interactive land viewer for the /lab/land detail page — WebGL-free SVG.
// Two navigation modes:
//   3D → isometric extruded columns (painter-sorted)
//   2D → top-down heightfield (hypsometric tint, like a relief map)
// Pan with drag, zoom with wheel/buttons, rotate 90° steps, reset view.
// Pan/zoom mutate a <g> transform directly (no React re-render per frame).

interface Props {
  ring: LonLat[]
  elevMinM: number
  elevMaxM: number
  elevationSource: number
  dimension: number
  seedHex: string
  verified?: boolean
}

const W = 680
const H = 400
const GRID = 14
const S = 2.1
const H0 = 0.14

type Mode = '3d' | '2d'
type Pt = { x: number; y: number }

interface Scene {
  outline: string
  grid: { a: string; b: string }[]
  cols: {
    top: string
    south: string
    east: string
    flat: string
    h: number
  }[]
  cellCount: number
  color: string
  centroid: Pt
  flat: boolean
}

function clamp(v: number, lo: number, hi: number) {
  return Math.max(lo, Math.min(hi, v))
}

export default function LandViewer({
  ring,
  elevMinM,
  elevMaxM,
  elevationSource,
  dimension,
  seedHex,
  verified = false,
}: Props) {
  const [mode, setMode] = useState<Mode>('3d')
  const [rot, setRot] = useState(0)
  const svgRef = useRef<SVGSVGElement>(null)
  const gRef = useRef<SVGGElement>(null)
  const zoomRef = useRef<HTMLSpanElement>(null)
  const view = useRef({ tx: 0, ty: 0, s: 1 })
  const drag = useRef<{ x0: number; y0: number; tx: number; ty: number; w: number } | null>(null)

  const applyView = useCallback(() => {
    const v = view.current
    gRef.current?.setAttribute(
      'transform',
      `translate(${v.tx.toFixed(2)} ${v.ty.toFixed(2)}) scale(${v.s.toFixed(4)})`,
    )
    if (zoomRef.current) zoomRef.current.textContent = `${Math.round(v.s * 100)}%`
  }, [])

  const resetView = useCallback(() => {
    view.current = { tx: 0, ty: 0, s: 1 }
    applyView()
  }, [applyView])

  const zoomAt = useCallback(
    (k: number, cx = W / 2, cy = H / 2) => {
      const v = view.current
      const s = clamp(v.s * k, 0.4, 10)
      const kk = s / v.s
      view.current = { s, tx: cx - kk * (cx - v.tx), ty: cy - kk * (cy - v.ty) }
      applyView()
    },
    [applyView],
  )

  // wheel zoom around cursor (non-passive so preventDefault works)
  useEffect(() => {
    const el = svgRef.current
    if (!el) return
    const onWheel = (e: WheelEvent) => {
      e.preventDefault()
      const rect = el.getBoundingClientRect()
      const cx = ((e.clientX - rect.left) / rect.width) * W
      const cy = ((e.clientY - rect.top) / rect.height) * H
      zoomAt(Math.exp(-e.deltaY * 0.0018), cx, cy)
    }
    el.addEventListener('wheel', onWheel, { passive: false })
    return () => el.removeEventListener('wheel', onWheel)
  }, [zoomAt])

  // scene geometry (recomputed on rot/mode/data change; view transform persists)
  const scene = useMemo<Scene | null>(() => {
    if (ring.length < 3) return null
    const unit = normalizeRing(ring, rot)
    const flat = dimension === 0 || elevationSource === 0
    const spanM = Math.max(0, elevMaxM - elevMinM)
    const cells = cellHeights(unit, seedHex + String(rot), flat, GRID)
    if (cells.length === 0) return null
    const H1 = 0.52 + Math.min(0.25, spanM / 2000)
    const color = ELEVATION_SOURCES[elevationSource]?.color ?? '#9ca3af'
    const p3 = (x: number, y: number, z: number): Pt =>
      mode === '3d' ? projectIso(x, y, z) : { x, y }
    const p2 = (x: number, y: number): Pt => p3(x, y, 0)

    const base = unit.map(([x, y]) => p2((x - 0.5) * S, (y - 0.5) * S))

    const cols = cells.map((c) => {
      const x0 = (c.gx / GRID - 0.5) * S
      const x1 = ((c.gx + 1) / GRID - 0.5) * S
      const y0 = (c.gy / GRID - 0.5) * S
      const y1 = ((c.gy + 1) / GRID - 0.5) * S
      const z = (H0 + (H1 - H0) * c.h) * (flat ? 0.45 : 1)
      return {
        h: c.h,
        top: [p3(x0, y0, z), p3(x1, y0, z), p3(x1, y1, z), p3(x0, y1, z)],
        south: [p3(x0, y1, z), p3(x1, y1, z), p3(x1, y1, 0), p3(x0, y1, 0)],
        east: [p3(x1, y0, z), p3(x1, y1, z), p3(x1, y1, 0), p3(x1, y0, 0)],
        flat: [p2(x0, y0), p2(x1, y0), p2(x1, y1), p2(x0, y1)],
      }
    })
    // painter's order: back-to-front by gx+gy (cols[i] ↔ cells[i])
    const ordered = cells
      .map((c, i) => ({ i, key: c.gx + c.gy }))
      .sort((a, b) => a.key - b.key)
      .map((o) => cols[o.i])

    // survey grid on the ground plane
    const grid: { a: string; b: string }[] = []
    const N = 7
    for (let i = 0; i <= N; i++) {
      const t = (i / N - 0.5) * S
      const lineH = poly([p2(-S / 2, t), p2(S / 2, t)])
      const lineV = poly([p2(t, -S / 2), p2(t, S / 2)])
      grid.push({ a: lineH, b: lineV })
    }

    const all = [
      ...base,
      ...ordered.flatMap((c) => (mode === '3d' ? [...c.top, ...c.south, ...c.east] : c.flat)),
    ]
    const fit = fitProjector(all, W, H, 24)
    const cp = p2(0, 0)
    return {
      outline: poly(base.map(fit)),
      grid,
      cols: ordered.map((c) => ({
        top: poly(c.top.map(fit)),
        south: poly(c.south.map(fit)),
        east: poly(c.east.map(fit)),
        flat: poly(c.flat.map(fit)),
        h: c.h,
      })),
      cellCount: ordered.length,
      color,
      centroid: fit({ x: cp.x, y: cp.y }),
      flat,
    }
  }, [ring, rot, mode, seedHex, elevationSource, dimension, elevMinM, elevMaxM])

  // re-apply transform after scene re-render (same <g> node persists)
  useEffect(() => {
    applyView()
  }, [scene, applyView])

  const prov = ELEVATION_SOURCES[elevationSource]
  const baseColor = verified ? '#10b981' : (scene?.color ?? '#9ca3af')

  const onPointerDown = (e: React.PointerEvent) => {
    const rect = svgRef.current?.getBoundingClientRect()
    if (!rect) return
    e.currentTarget.setPointerCapture(e.pointerId)
    drag.current = { x0: e.clientX, y0: e.clientY, tx: view.current.tx, ty: view.current.ty, w: rect.width }
  }
  const onPointerMove = (e: React.PointerEvent) => {
    const d = drag.current
    if (!d) return
    const k = W / d.w
    view.current.tx = d.tx + (e.clientX - d.x0) * k
    view.current.ty = d.ty + (e.clientY - d.y0) * k
    applyView()
  }
  const onPointerUp = () => {
    drag.current = null
  }

  if (!scene) {
    return <div className="iso-empty">No geometry yet — anchor a shape first.</div>
  }

  return (
    <div className="land-viewer">
      <div className="land-viewer-toolbar">
        <div className="land-viewer-seg">
          <button
            className={`land-viewer-segbtn ${mode === '2d' ? 'on' : ''}`}
            onClick={() => setMode('2d')}
            title="Top-down 2D relief view"
          >
            2D
          </button>
          <button
            className={`land-viewer-segbtn ${mode === '3d' ? 'on' : ''}`}
            onClick={() => setMode('3d')}
            title="Isometric 3D view"
          >
            3D
          </button>
        </div>
        <span className="flex-1" />
        <button className="btn btn-ghost p-1" title="Rotate left 90°" onClick={() => setRot((r) => (r + 3) % 4)}>
          <RotateCcw size={14} />
        </button>
        <button className="btn btn-ghost p-1" title="Rotate right 90°" onClick={() => setRot((r) => (r + 1) % 4)}>
          <RotateCw size={14} />
        </button>
        <button className="btn btn-ghost p-1" title="Zoom out" onClick={() => zoomAt(1 / 1.3)}>
          <ZoomOut size={14} />
        </button>
        <button className="btn btn-ghost p-1" title="Zoom in" onClick={() => zoomAt(1.3)}>
          <ZoomIn size={14} />
        </button>
        <button className="btn btn-ghost p-1" title="Fit view" onClick={resetView}>
          <Maximize2 size={14} />
        </button>
        <span ref={zoomRef} className="land-viewer-zoom font-mono">
          100%
        </span>
      </div>

      <svg
        ref={svgRef}
        viewBox={`0 0 ${W} ${H}`}
        width="100%"
        height={H}
        preserveAspectRatio="xMidYMid meet"
        role="img"
        aria-label="Interactive land view (SVG, no WebGL)"
        className="land-viewer-svg"
        onPointerDown={onPointerDown}
        onPointerMove={onPointerMove}
        onPointerUp={onPointerUp}
        onPointerCancel={onPointerUp}
      >
        <defs>
          <linearGradient id="lv-sky" x1="0" y1="0" x2="0" y2="1">
            <stop offset="0%" stopColor="#f8fafc" />
            <stop offset="100%" stopColor="#eef2f7" />
          </linearGradient>
        </defs>
        <rect x="0" y="0" width={W} height={H} rx="8" fill="url(#lv-sky)" />

        <g ref={gRef}>
          {/* survey grid */}
          {scene.grid.map((l, i) => (
            <g key={`g${i}`}>
              <polyline points={l.a} fill="none" stroke="#cbd5e1" strokeWidth="0.6" strokeDasharray="3 4" />
              {l.b && (
                <polyline points={l.b} fill="none" stroke="#cbd5e1" strokeWidth="0.6" strokeDasharray="3 4" />
              )}
            </g>
          ))}

          {/* footprint */}
          <polygon
            points={scene.outline}
            fill="rgba(15,23,42,0.04)"
            stroke={verified ? '#10b981' : '#94a3b8'}
            strokeWidth="1.4"
            strokeDasharray="5 3"
          />

          {mode === '3d'
            ? scene.cols.map((c, i) => (
                <g key={i} className="isot-col" style={{ animationDelay: `${(i % 28) * 12}ms` }}>
                  <polygon points={c.south} fill={shade(baseColor, 0.62)} stroke="rgba(15,23,42,0.18)" strokeWidth="0.4" />
                  <polygon points={c.east} fill={shade(baseColor, 0.44)} stroke="rgba(15,23,42,0.18)" strokeWidth="0.4" />
                  <polygon
                    points={c.top}
                    fill={shade(baseColor, 1.18 - 0.42 * c.h)}
                    stroke={verified ? '#10b981' : 'rgba(15,23,42,0.22)'}
                    strokeWidth={verified ? 0.7 : 0.45}
                  />
                </g>
              ))
            : scene.cols.map((c, i) => (
                <polygon
                  key={i}
                  className="isot-col"
                  style={{ animationDelay: `${(i % 28) * 12}ms` }}
                  points={c.flat}
                  fill={shade(scene.color, 0.5 + 0.55 * c.h)}
                  stroke="rgba(15,23,42,0.25)"
                  strokeWidth="0.5"
                />
              ))}
        </g>

        {/* static overlays */}
        <g className="land-viewer-north">
          <path d={`M ${W - 34} 40 L ${W - 28} 24 L ${W - 22} 40 Z`} fill="#475569" />
          <text x={W - 28} y={54} textAnchor="middle" className="land-viewer-north-t">
            N
          </text>
        </g>
        <text x={14} y={H - 14} className="land-viewer-hint">
          drag to pan · wheel to zoom · {rot * 90}° · {scene.cellCount} cells
        </text>
      </svg>

      <div className="land-viewer-legend">
        <span className="text-[10px] text-muted font-mono">{Math.round(elevMinM)} m</span>
        <span
          className="land-viewer-ramp"
          style={{
            background: `linear-gradient(90deg, ${shade(scene.color, 0.55)}, ${shade(scene.color, 1.25)})`,
          }}
        />
        <span className="text-[10px] text-muted font-mono">{Math.round(elevMaxM)} m</span>
        <span className="lab-badge" style={{ background: `${scene.color}22`, color: shade(scene.color, 0.7) }}>
          {prov?.id ?? 'NONE'}
        </span>
        <span className="lab-badge lab-badge-info">SVG · no WebGL</span>
      </div>
      {scene.flat && (
        <p className="text-[10px] text-muted mt-1">
          Flat plate — D2 / elevation_source NONE carries a zero envelope (6237). Anchor a D2.5 version with
          provenance for relief.
        </p>
      )}
    </div>
  )
}
