// Asset Inspector — renders a demo asset as a Gaussian Splat-style scene
// using three.js Points (no SharedArrayBuffer, no cross-origin isolation
// required). Falls back to a static preview on low tier.
//
// Demo splat is synthetic (sampled from the bundled GLB), labeled honestly.
import { useEffect, useRef, useState } from 'react'
import * as THREE from 'three'
import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls.js'
import { Sparkles, Image, AlertTriangle } from 'lucide-react'
import { assetUrl } from '../../lib/assets'
import { AtlasHonestyLabel } from './AtlasPage'

// Small synthetic splat sampled from the bundled house_scene.glb.
// Header: "SPLAT" magic + uint32 LE point count, then per-point:
//   x,y,z,nx,ny,nz, f_dc_r,f_dc_g,f_dc_b, opacity, scale_x,scale_y,scale_z, rot_x,rot_y,rot_z,rot_w (all Float32)
const SPLAT_URL = assetUrl('models/demo-house.splat')

interface SplatPoint {
  x: number; y: number; z: number
  nx: number; ny: number; nz: number
  r: number; g: number; b: number
  opacity: number
  sx: number; sy: number; sz: number
  rx: number; ry: number; rz: number; rw: number
}

function loadSplat(url: string): Promise<SplatPoint[]> {
  return fetch(url)
    .then((r) => {
      if (!r.ok) throw new Error(`splat fetch ${r.status}`)
      return r.arrayBuffer()
    })
    .then((ab) => {
      const view = new DataView(ab)
      const magic = String.fromCharCode(...new Uint8Array(ab.slice(0, 5)))
      if (magic !== 'SPLAT') throw new Error('bad splat magic')
      const count = view.getUint32(5, true)
      const points: SplatPoint[] = []
      let off = 9
      const stride = 17 * 4
      for (let i = 0; i < count; i++) {
        points.push({
          x: view.getFloat32(off, true), y: view.getFloat32(off + 4, true), z: view.getFloat32(off + 8, true),
          nx: view.getFloat32(off + 12, true), ny: view.getFloat32(off + 16, true), nz: view.getFloat32(off + 20, true),
          r: view.getFloat32(off + 24, true), g: view.getFloat32(off + 28, true), b: view.getFloat32(off + 32, true),
          opacity: view.getFloat32(off + 36, true),
          sx: view.getFloat32(off + 40, true), sy: view.getFloat32(off + 44, true), sz: view.getFloat32(off + 48, true),
          rx: view.getFloat32(off + 52, true), ry: view.getFloat32(off + 56, true), rz: view.getFloat32(off + 60, true), rw: view.getFloat32(off + 64, true),
        })
        off += stride
      }
      return points
    })
}

export default function AssetInspector() {
  const [mode, setMode] = useState<'loading' | 'splat' | 'static' | 'error'>('loading')
  const [error, setError] = useState<string | null>(null)
  const hostRef = useRef<HTMLDivElement>(null)
  const ctxRef = useRef<{
    renderer: THREE.WebGLRenderer
    scene: THREE.Scene
    camera: THREE.PerspectiveCamera
    controls: OrbitControls
    points: THREE.Points
  } | null>(null)
  const cleanupRef = useRef<(() => void) | null>(null)

  // Try to load the synthetic splat; fall back to static on error.
  useEffect(() => {
    let cancelled = false
    loadSplat(SPLAT_URL)
      .then((points) => {
        if (cancelled) return
        const cleanup = initSplat(points, hostRef, ctxRef, setMode, setError)
        if (!cancelled) cleanupRef.current = cleanup
      })
      .catch((err) => {
        if (cancelled) return
        console.warn('splat load failed, showing static:', err.message)
        setMode('static')
        setError(err.message)
      })
    return () => {
      cancelled = true
      if (cleanupRef.current) cleanupRef.current()
      cleanupRef.current = null
      ctxRef.current = null
    }
  }, [])

function initSplat(
  points: SplatPoint[],
  hostRef: React.RefObject<HTMLDivElement | null>,
  ctxRef: React.RefObject<{
    renderer: THREE.WebGLRenderer
    scene: THREE.Scene
    camera: THREE.PerspectiveCamera
    controls: OrbitControls
    points: THREE.Points
  } | null>,
  setMode: (m: 'loading' | 'splat' | 'static' | 'error') => void,
  setError: (e: string | null) => void,
): () => void {
  const host = hostRef.current
  if (!host) return () => {}
  try {
    const renderer = new THREE.WebGLRenderer({ antialias: true, alpha: true, powerPreference: 'high-performance' })
    renderer.setPixelRatio(Math.min(window.devicePixelRatio, 1.5))
    renderer.setSize(host.clientWidth, host.clientHeight, false)
    host.appendChild(renderer.domElement)
    renderer.domElement.className = 'atlas-inspector-canvas'

    const scene = new THREE.Scene()
    scene.background = new THREE.Color(0x0a0a0f)
    const camera = new THREE.PerspectiveCamera(45, host.clientWidth / host.clientHeight, 0.1, 100)
    camera.position.set(0, 0, 5)

    const controls = new OrbitControls(camera, renderer.domElement)
    controls.enableDamping = true
    controls.minDistance = 1
    controls.maxDistance = 40

    const positions = new Float32Array(points.length * 3)
    const colors = new Float32Array(points.length * 3)
    const sizes = new Float32Array(points.length)
    const opacities = new Float32Array(points.length)

    points.forEach((p, i) => {
      positions[i * 3] = p.x
      positions[i * 3 + 1] = p.y
      positions[i * 3 + 2] = p.z
      colors[i * 3] = p.r
      colors[i * 3 + 1] = p.g
      colors[i * 3 + 2] = p.b
      sizes[i] = p.sx * 40
      opacities[i] = p.opacity
    })

    const geometry = new THREE.BufferGeometry()
    geometry.setAttribute('position', new THREE.BufferAttribute(positions, 3))
    geometry.setAttribute('color', new THREE.BufferAttribute(colors, 3))
    geometry.setAttribute('size', new THREE.BufferAttribute(sizes, 1))
    geometry.setAttribute('opacity', new THREE.BufferAttribute(opacities, 1))

    const material = new THREE.ShaderMaterial({
      vertexShader: `
        varying vec3 vColor;
        varying float vOpacity;
        uniform float pointScale;
        void main() {
          vColor = color;
          vOpacity = opacity;
          vec4 mvPosition = modelViewMatrix * vec4(position, 1.0);
          gl_PointSize = size * pointScale / -mvPosition.z;
          gl_Position = projectionMatrix * mvPosition;
        }
      `,
      fragmentShader: `
        varying vec3 vColor;
        varying float vOpacity;
        void main() {
          float r = length(gl_PointCoord - vec2(0.5));
          if (r > 0.5) discard;
          float alpha = vOpacity * (1.0 - smoothstep(0.0, 0.5, r * 2.0));
          gl_FragColor = vec4(vColor, alpha);
        }
      `,
      transparent: true,
      depthWrite: false,
      vertexColors: true,
      uniforms: { pointScale: { value: window.innerHeight } },
    })

    const pts = new THREE.Points(geometry, material)
    scene.add(pts)

    const light = new THREE.PointLight(0xAB9FF2, 2, 100)
    light.position.set(5, 5, 5)
    scene.add(light)
    scene.add(new THREE.AmbientLight(0x404040, 1))

    let frame = 0
    const render = (now: number) => {
      controls.update()
      const elapsed = now * 0.001
      pts.rotation.y = Math.sin(elapsed * 0.5) * 0.3
      pts.rotation.x = Math.cos(elapsed * 0.3) * 0.2
      renderer.render(scene, camera)
      frame = requestAnimationFrame(render)
    }
    render(0)

    ctxRef.current = { renderer, scene, camera, controls, points: pts }

    function onResize() {
      const ctx = ctxRef.current
      const h = hostRef.current
      if (!ctx || !h) return
      ctx.camera.aspect = h.clientWidth / h.clientHeight
      ctx.camera.updateProjectionMatrix()
      ctx.renderer.setSize(h.clientWidth, h.clientHeight, false)
    }

    window.addEventListener('resize', onResize)

    return () => {
      window.removeEventListener('resize', onResize)
      cancelAnimationFrame(frame)
      controls.dispose()
      geometry.dispose()
      material.dispose()
      renderer.dispose()
      renderer.domElement.remove()
      ctxRef.current = null
    }
  } catch (err) {
    console.warn('splat init failed:', err)
    setMode('static')
    setError(err instanceof Error ? err.message : String(err))
    return () => {}
  }
}

  return (
    <div className="atlas-window-body">
      <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 10 }}>
        <Sparkles size={14} className="text-muted" />
        <span style={{ fontWeight: 600, fontSize: 13, color: 'var(--ink)' }}>Asset Inspector</span>
        <AtlasHonestyLabel mode="simulation" />
      </div>

      <div className="atlas-inspector-stage" ref={hostRef}>
        {mode === 'loading' && (
          <div className="atlas-inspector-placeholder">
            <Sparkles size={20} className="text-muted animate-spin" />
            <div>loading splat…</div>
            <AtlasHonestyLabel mode="simulation" />
          </div>
        )}
        {mode === 'error' && error && (
          <div className="atlas-inspector-placeholder">
            <AlertTriangle size={20} className="text-muted" />
            <div>splat unavailable</div>
            <div style={{ fontSize: 10, marginTop: 4 }}>{error}</div>
          </div>
        )}
        {mode === 'static' && (
          <div className="atlas-inspector-placeholder">
            <Image size={32} className="text-muted" />
            <div>static preview</div>
            <AtlasHonestyLabel mode="prototype" />
          </div>
        )}
      </div>

      <div style={{ marginTop: 10, display: 'flex', gap: 8, flexWrap: 'wrap' }}>
        <button
          type="button"
          className="btn btn-ghost btn-p-1"
          onClick={() => {
            setMode('loading')
            setError(null)
            if (cleanupRef.current) cleanupRef.current()
            void loadSplat(SPLAT_URL).then((points) => {
              const cleanup = initSplat(points, hostRef, ctxRef, setMode, setError)
              cleanupRef.current = cleanup
            }).catch((err) => setError(err.message))
          }}
          disabled={mode === 'loading'}
        >
          <Sparkles size={12} /> reload splat
        </button>
      </div>

      <div style={{ marginTop: 10, fontSize: 11, color: 'var(--muted)', fontFamily: 'var(--font-mono)' }}>
        3D Gaussian Splat preview — synthetic sample, not a survey capture.
      </div>
    </div>
  )
}

// Minimal spin animation for the loading indicator.
const style = document.createElement('style')
style.textContent = `
  @keyframes animate-spin { to { transform: rotate(360deg); } }
  .animate-spin { animation: animate-spin 1s linear infinite; }
`
if (typeof document !== 'undefined') document.head.appendChild(style)
