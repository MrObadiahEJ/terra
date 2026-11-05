import { useEffect, useRef } from 'react'
import * as THREE from 'three'
import { GLTFLoader } from 'three/examples/jsm/loaders/GLTFLoader.js'
import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls.js'
import { DEMO_BUILDINGS } from '../../lib/demoData'
import { parseGeoJSON, type OffChainParcel, type OsmBuildingFootprint } from '../../lib/api'
import type { FocusTarget, ViewCommand } from './TerraGlobe'
import { buildingHeightM } from '../../lib/atlasLayers'

interface ThreeAtlasFallbackProps {
  focus?: FocusTarget | null
  viewCommand?: ViewCommand | null
  highlightParcelId?: string | null
  highlightBuildingId?: number | null
  onCameraMoved?: (cam: { longitude: number; latitude: number; height: number }) => void
  offChainParcels?: OffChainParcel[]
  buildings?: OsmBuildingFootprint[]
  onParcelClick?: (id: string) => void
  onBuildingClick?: (osmId: number) => void
}

function project(lon: number, lat: number): THREE.Vector3 {
  return new THREE.Vector3((lon - 11.501) * 720000, 0, -(lat - 3.847) * 720000)
}

function attachHouse(group: THREE.Group, instance: THREE.Object3D, index: number) {
  const size = new THREE.Box3().setFromObject(instance).getSize(new THREE.Vector3())
  const base = size.y > 0 ? 7 / size.y : 1
  instance.scale.setScalar(base * (index % 3 === 0 ? 0.82 : index % 3 === 1 ? 1 : 1.18))
  instance.rotation.y = (index % 4) * (Math.PI / 2)
  group.clear()
  group.add(instance)
}

type SceneCtx = {
  renderer: THREE.WebGLRenderer
  scene: THREE.Scene
  camera: THREE.PerspectiveCamera
  controls: OrbitControls
}

export default function ThreeAtlasFallback({
  focus,
  viewCommand,
  offChainParcels,
  buildings,
  highlightParcelId,
  highlightBuildingId,
  onParcelClick,
  onBuildingClick,
  onCameraMoved,
}: ThreeAtlasFallbackProps) {
  const hostRef = useRef<HTMLDivElement>(null)
  const ctxRef = useRef<SceneCtx | null>(null)
  const contentRef = useRef<THREE.Group | null>(null)
  const onCameraMovedRef = useRef(onCameraMoved)
  const onParcelClickRef = useRef(onParcelClick)
  const onBuildingClickRef = useRef(onBuildingClick)
  useEffect(() => {
    onCameraMovedRef.current = onCameraMoved
    onParcelClickRef.current = onParcelClick
    onBuildingClickRef.current = onBuildingClick
  })

  useEffect(() => {
    const host = hostRef.current
    if (!host) return

    let renderer: THREE.WebGLRenderer
    try {
      renderer = new THREE.WebGLRenderer({ antialias: true, alpha: false, powerPreference: 'low-power' })
    } catch {
      return
    }

    renderer.setPixelRatio(Math.min(devicePixelRatio, 1.25))
    renderer.setSize(host.clientWidth, host.clientHeight, false)
    renderer.setClearColor(0x07111d, 1)
    host.appendChild(renderer.domElement)
    renderer.domElement.className = 'three-atlas-canvas'

    const scene = new THREE.Scene()
    scene.fog = new THREE.Fog(0xcbd0d1, 700, 2600)
    const camera = new THREE.PerspectiveCamera(42, host.clientWidth / host.clientHeight, 0.1, 4000)
    camera.position.set(430, 360, 520)

    const controls = new OrbitControls(camera, renderer.domElement)
    controls.enableDamping = true
    controls.minDistance = 40
    controls.maxDistance = 2400
    controls.maxPolarAngle = Math.PI * 0.48

    scene.add(new THREE.HemisphereLight(0xe5ebed, 0x657079, 2.2))
    const sun = new THREE.DirectionalLight(0xfff4d6, 2)
    sun.position.set(80, 120, 40)
    scene.add(sun)

    const ground = new THREE.Mesh(
      new THREE.PlaneGeometry(1600, 1600),
      new THREE.MeshStandardMaterial({ color: 0x6f7779, roughness: 1 }),
    )
    ground.rotation.x = -Math.PI / 2
    scene.add(ground)

    const content = new THREE.Group()
    scene.add(content)
    contentRef.current = content
    ctxRef.current = { renderer, scene, camera, controls }

    // Click (not drag) picks parcels and buildings — Cesium parity.
    let sx = -99
    let sy = -99
    const dn = (e: PointerEvent) => { sx = e.offsetX; sy = e.offsetY }
    const pick = (e: MouseEvent) => {
      if (Math.abs(e.offsetX - sx) + Math.abs(e.offsetY - sy) > 6) return
      const el = renderer.domElement
      const p = new THREE.Vector2((e.offsetX / el.clientWidth) * 2 - 1, -(e.offsetY / el.clientHeight) * 2 + 1)
      const ray = new THREE.Raycaster()
      ray.setFromCamera(p, camera)
      const u = ray.intersectObjects(content.children, true)[0]?.object.userData ?? {}
      if (u.parcelId) onParcelClickRef.current?.(u.parcelId)
      else if (u.buildingId) onBuildingClickRef.current?.(u.buildingId)
    }
    renderer.domElement.addEventListener('pointerdown', dn)
    renderer.domElement.addEventListener('click', pick)

    let frame = 0
    let lastReport = 0
    let last = [-1e9, -1e9, -1e9]
    const render = () => {
      controls.update()
      renderer.render(scene, camera)
      const now = performance.now()
      if (now - lastReport > 220) {
        lastReport = now
        const c = [camera.position.x / 720000 + 11.501, -camera.position.z / 720000 + 3.847, camera.position.y]
        if (c.some((v, i) => Math.abs(v - last[i]) > (i < 2 ? 1e-6 : 0.5))) {
          last = c
          onCameraMovedRef.current?.({ longitude: c[0], latitude: c[1], height: c[2] })
        }
      }
      frame = window.requestAnimationFrame(render)
    }
    render()

    const onResize = () => {
      camera.aspect = host.clientWidth / host.clientHeight
      camera.updateProjectionMatrix()
      renderer.setSize(host.clientWidth, host.clientHeight, false)
      renderer.render(scene, camera)
    }
    window.addEventListener('resize', onResize)

    return () => {
      window.cancelAnimationFrame(frame)
      ctxRef.current = null
      contentRef.current = null
      window.removeEventListener('resize', onResize)
      renderer.domElement.removeEventListener('pointerdown', dn)
      renderer.domElement.removeEventListener('click', pick)
      controls.dispose()
      ground.geometry.dispose()
      ground.material.dispose()
      renderer.dispose()
      renderer.domElement.remove()
    }
  }, [])

  // ---- content: parcels + buildings + progressive GLB houses --------------
  useEffect(() => {
    const ctx = ctxRef.current
    const content = contentRef.current
    if (!ctx || !content) return
    let cancelled = false
    const { camera } = ctx

    let gate = 0
    // Parcels: transparent fill, brighter selected outline (Cesium parity).
    for (const parcel of offChainParcels ?? []) {
      const poly = parseGeoJSON<{ type: string; coordinates: number[][][] }>(parcel.geometry)
      const ring = poly?.type === 'Polygon' ? poly.coordinates[0] : []
      if (ring.length < 3) continue
      const sel = parcel.id === highlightParcelId
      const pts = ring.map(([lon, lat]) => project(lon, lat))
      const shape = new THREE.Shape(pts.map((p) => new THREE.Vector2(p.x, p.z)))
      const fill = new THREE.Mesh(new THREE.ShapeGeometry(shape), new THREE.MeshBasicMaterial({
        color: sel ? 0x34d399 : 0xb7c0c5, transparent: true, opacity: sel ? 0.14 : 0.04, side: THREE.DoubleSide, depthWrite: false,
      }))
      fill.rotation.x = -Math.PI / 2
      fill.position.y = 0.08
      fill.userData.parcelId = parcel.id
      content.add(fill)
      const edge = pts.map((p) => new THREE.Vector3(p.x, 0.12, p.z))
      edge.push(edge[0].clone())
      const outline = new THREE.Line(new THREE.BufferGeometry().setFromPoints(edge), new THREE.LineBasicMaterial({
        color: sel ? 0x34d399 : 0xd6c98f, transparent: true, opacity: sel ? 0.95 : 0.58,
      }))
      outline.userData.parcelId = parcel.id
      content.add(outline)
    }
    // Extruded footprints (~6.48 scene units per metre).
    const buildingList = (buildings?.length ? buildings : DEMO_BUILDINGS).slice(0, 90)
    for (const b of buildingList) {
      const ring = b.geometry?.coordinates?.[0] ?? []
      if (ring.length < 3) continue
      const sel = b.osm_id === highlightBuildingId
      const shape = new THREE.Shape(ring.map(([lon, lat]) => { const p = project(lon, lat); return new THREE.Vector2(p.x, p.z) }))
      const mesh = new THREE.Mesh(
        new THREE.ExtrudeGeometry(shape, { depth: Math.max(3, buildingHeightM(b) * 6.48), bevelEnabled: false }),
        new THREE.MeshStandardMaterial({ color: sel ? 0x5eead4 : 0x9fb6c9, transparent: true, opacity: sel ? 0.85 : 0.72, roughness: 0.85 }),
      )
      mesh.rotation.x = -Math.PI / 2
      mesh.position.y = 0.1
      mesh.userData.buildingId = b.osm_id
      content.add(mesh)
    }
    // Progressive GLB houses: staggered single-flight loads, cancellable.
    const modelUrls = ['/models/house_scene.glb', '/models/modern-house.glb']
    const protos = new Map<string, THREE.Object3D>()
    const houses: { g: THREE.Group; c: THREE.Vector3 }[] = []
    let chain: Promise<void> = Promise.resolve()
    buildingList.slice(0, 8).forEach((b, i) => {
      const ring = b.geometry?.coordinates?.[0] ?? []
      if (ring.length < 3) return
      const c = ring.reduce((a, [lon, lat]) => a.add(project(lon, lat)), new THREE.Vector3()).divideScalar(ring.length)
      const g = new THREE.Group()
      g.position.set(c.x, 0.12, c.z)
      g.visible = false
      g.userData.buildingId = b.osm_id
      content.add(g)
      houses.push({ g, c })
      const url = modelUrls[i % 2]
      chain = chain.then(() => new Promise<void>((res) => window.setTimeout(() => {
        if (cancelled) return res()
        const done = (o: THREE.Object3D) => { attachHouse(g, o, i); res() }
        const proto = protos.get(url)
        if (proto) return done(proto.clone(true))
        new GLTFLoader().load(url, (gl) => {
          if (cancelled) return res()
          protos.set(url, gl.scene)
          done(gl.scene.clone(true))
        }, undefined, () => res())
      }, 350)))
    })
    gate = window.setInterval(() => {
      for (const { g, c } of houses) g.visible = camera.position.distanceTo(c) < 420
    }, 300)
    return () => {
      cancelled = true
      window.clearInterval(gate)
      for (const child of [...content.children]) {
        content.remove(child)
        child.traverse((o) => {
          if (o instanceof THREE.Mesh || o instanceof THREE.Line) {
            o.geometry.dispose()
            ;(Array.isArray(o.material) ? o.material : [o.material]).forEach((m) => m.dispose())
          }
        })
      }
    }
  }, [offChainParcels, buildings, highlightParcelId, highlightBuildingId])

  // Focus fly-to + one-shot front/street/reset view commands (Cesium parity).
  const lastViewNonce = useRef(0)
  useEffect(() => {
    const ctx = ctxRef.current
    if (viewCommand && viewCommand.nonce !== lastViewNonce.current) {
      lastViewNonce.current = viewCommand.nonce
      const t = viewCommand.longitude != null && viewCommand.latitude != null
        ? project(viewCommand.longitude, viewCommand.latitude)
        : null
      if (t && ctx) {
        t.y = 3
        const s = viewCommand.kind === 'street'
        ctx.camera.position.copy(t).add(new THREE.Vector3(s ? 12 : 40, s ? 3 : 14, s ? -8 : -24))
        ctx.controls.target.copy(t)
        ctx.controls.update()
        ctx.renderer.render(ctx.scene, ctx.camera)
        return
      }
    }
    if (!focus || !ctx) return
    const t = project(focus.longitude, focus.latitude)
    const d = Math.max(40, Math.min(2400, focus.height * 0.06 + 40))
    ctx.camera.position.copy(t).add(new THREE.Vector3(d * 0.5, d * 0.45, d * 0.6))
    ctx.controls.target.copy(t)
    ctx.controls.update()
  }, [focus, viewCommand])

  return (
    <div ref={hostRef} className="three-atlas-fallback" />
  )
}
