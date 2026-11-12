#!/usr/bin/env node
// Bundle-discipline gate (RISK R-012).
//
// Cesium is ~4 MB of JavaScript and is only needed on the map/story routes.
// Losing route-level lazy loading would silently make every visitor download
// it. This script runs after `vite build` and fails the build when:
//
//   1. an initial (entry + modulepreload) chunk contains Cesium;
//   2. the initial JavaScript graph exceeds its gzip budget;
//   3. the Cesium chunk exceeds its own budget;
//   4. total shipped JavaScript grows without a deliberate budget update.
//
// Budgets are ceilings, not targets. Raising one is a decision: update the
// number here and record why in the Book of Intents (RISK R-012, sprint 010).

import { readdirSync, readFileSync, statSync } from 'node:fs'
import { join } from 'node:path'
import { gzipSync } from 'node:zlib'

const DIST = new URL('../dist/', import.meta.url).pathname
// Budgets in decimal kB (gzip), matching Vite's build report.
const KB = 1000

// Current measured values are in the comment.
const BUDGET = {
  entryGzipKb: 340, // index-*.js            ~295 kB
  initialGzipKb: 360, // entry + preloads      ~301 kB
  cesiumGzipKb: 1200, // Cesium chunk         ~1098 kB
  totalGzipKb: 1900, // every emitted JS     ~1678 kB
}

const CESIUM_MARKERS = ['CesiumJS', 'CesiumWidget']

function kb(bytes) {
  return bytes / KB
}

function gzipSize(path) {
  return gzipSync(readFileSync(path)).length
}

function fail(message) {
  console.error(`\nFAIL  ${message}`)
  process.exitCode = 1
}

if (!statSync(DIST, { throwIfNoEntry: false })) {
  console.error('FAIL  dist/ missing — run `vite build` first')
  process.exit(1)
}

const assetsDir = join(DIST, 'assets')
const assets = new Map(
  readdirSync(assetsDir)
    .filter((name) => name.endsWith('.js'))
    .map((name) => {
      const path = join(assetsDir, name)
      return [
        name,
        { path, raw: statSync(path).size, gzip: gzipSize(path), text: null },
      ]
    }),
)

const html = readFileSync(join(DIST, 'index.html'), 'utf8')
const initialNames = new Set()
for (const match of html.matchAll(/<(?:script|link)\b[^>]*?(?:src|href)="[^"]*\/assets\/([^"]+\.js)"/g)) {
  initialNames.add(match[1])
}
for (const match of html.matchAll(/<link\b[^>]*rel="modulepreload"[^>]*href="[^"]*\/assets\/([^"]+\.js)"/g)) {
  initialNames.add(match[1])
}

if (initialNames.size === 0) {
  console.error('FAIL  no initial JS chunks found in dist/index.html')
  process.exit(1)
}

const initial = [...initialNames].map((name) => {
  const asset = assets.get(name)
  if (!asset) {
    console.error(`FAIL  index.html references missing chunk ${name}`)
    process.exit(1)
  }
  return { name, ...asset }
})

const entry = initial.reduce((a, b) => (a.raw >= b.raw ? a : b))
const initialGzip = initial.reduce((sum, a) => sum + a.gzip, 0)
const totalGzip = [...assets.values()].reduce((sum, a) => sum + a.gzip, 0)

// The Cesium chunk is identified by content, not by filename (hashes change).
const cesium = [...assets.entries()]
  .map(([name, asset]) => {
    if (asset.text === null) asset.text = readFileSync(asset.path, 'utf8')
    return { name, ...asset, isCesium: CESIUM_MARKERS.some((m) => asset.text.includes(m)) }
  })
  .filter((asset) => asset.isCesium)
  .sort((a, b) => b.raw - a.raw)
const cesiumChunk = cesium[0]

console.log('initial chunks (entry + modulepreload):')
for (const asset of initial) {
  console.log(
    `  ${asset.name.padEnd(36)} ${kb(asset.raw).toFixed(1).padStart(8)} KB raw` +
      ` ${kb(asset.gzip).toFixed(1).padStart(7)} KB gz`,
  )
}
console.log(
  `  ${'initial total'.padEnd(36)} ${kb(initialGzip).toFixed(1).padStart(7)} KB gz` +
    `  (budget ${BUDGET.initialGzipKb})`,
)
if (cesiumChunk) {
  console.log(
    `lazy Cesium chunk: ${cesiumChunk.name} ` +
      `${kb(cesiumChunk.raw).toFixed(1)} KB raw / ${kb(cesiumChunk.gzip).toFixed(1)} KB gz` +
      `  (budget ${BUDGET.cesiumGzipKb})`,
  )
} else {
  console.log('lazy Cesium chunk: none emitted (map routes may be gone)')
}
console.log(
  `total emitted JS: ${kb(totalGzip).toFixed(1)} KB gz  (budget ${BUDGET.totalGzipKb})\n`,
)

// 1. Cesium must never be part of the initial graph.
for (const asset of initial) {
  if (asset.text === null) asset.text = readFileSync(asset.path, 'utf8')
  if (CESIUM_MARKERS.some((marker) => asset.text.includes(marker))) {
    fail(`${asset.name} is initial but contains Cesium — a route lost its lazy import`)
  }
}

// 2. Initial graph budget (entry chunk separately, so a single runaway file is
//    named even when the total happens to fit).
if (kb(entry.gzip) > BUDGET.entryGzipKb) {
  fail(
    `entry chunk ${entry.name} is ${kb(entry.gzip).toFixed(1)} KB gz ` +
      `(budget ${BUDGET.entryGzipKb} KB)`,
  )
}
if (kb(initialGzip) > BUDGET.initialGzipKb) {
  fail(
    `initial JS graph is ${kb(initialGzip).toFixed(1)} KB gz ` +
      `(budget ${BUDGET.initialGzipKb} KB)`,
  )
}

// 3. Cesium chunk budget.
if (cesiumChunk && kb(cesiumChunk.gzip) > BUDGET.cesiumGzipKb) {
  fail(
    `${cesiumChunk.name} is ${kb(cesiumChunk.gzip).toFixed(1)} KB gz ` +
      `(budget ${BUDGET.cesiumGzipKb} KB)`,
  )
}

// 4. Overall JavaScript budget.
if (kb(totalGzip) > BUDGET.totalGzipKb) {
  fail(`emitted JS is ${kb(totalGzip).toFixed(1)} KB gz (budget ${BUDGET.totalGzipKb} KB)`)
}

if (process.exitCode) {
  console.error(
    '\nIf the growth is intentional, raise the matching budget in ' +
      'terra-web/scripts/check-bundle.mjs and record the reason in the Book (R-012).',
  )
} else {
  console.log('PASS  bundle discipline holds')
}
