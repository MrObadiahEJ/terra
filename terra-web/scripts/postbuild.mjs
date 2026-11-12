#!/usr/bin/env node
// GitHub Pages post-build step.
//
// Pages is a static host with no SPA rewrite. Copying the built index.html to
// 404.html lets deep links such as /terra/atlas load the app instead of a bare
// "404 File not found" page; BrowserRouter then resolves the route from the
// unchanged address bar. `.nojekyll` keeps Pages from running Jekyll over the
// Vite asset directory.

import { copyFileSync, writeFileSync, existsSync } from 'node:fs'
import { join } from 'node:path'
import { fileURLToPath } from 'node:url'

const dist = fileURLToPath(new URL('../dist/', import.meta.url))
const index = join(dist, 'index.html')

if (!existsSync(index)) {
  console.error(`FAIL  ${index} missing — run \`vite build\` first`)
  process.exit(1)
}

copyFileSync(index, join(dist, '404.html'))
writeFileSync(join(dist, '.nojekyll'), '')
console.log('postbuild: wrote dist/404.html (SPA fallback) and dist/.nojekyll')
