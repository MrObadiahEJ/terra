// Static asset URLs served from `public/`.
//
// Vite's `base` is `/terra/` on GitHub Pages and `/` locally, so every runtime
// fetch of a bundled asset must go through `import.meta.env.BASE_URL` instead
// of an absolute root path.
export function assetUrl(path: string): string {
  return `${import.meta.env.BASE_URL}${path.replace(/^\//, '')}`
}
