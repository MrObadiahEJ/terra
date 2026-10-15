import { defineConfig } from 'vitest/config'

// Story-engine unit tests run in a Node environment (pure logic — no DOM, no
// Cesium, no API). Keep this separate from vite.config.ts so the Cesium static
// asset plugin does not run during tests.
export default defineConfig({
  test: {
    environment: 'node',
    include: ['src/**/*.test.ts'],
  },
})
