import { describe, expect, it } from 'vitest'
import {
  hasEmbeddedCredential,
  isLoopbackEndpoint,
  resolvePublicEndpoint,
} from './constants'

const dev = { allowLoopback: true, allowCredentials: true }
const prod = { allowLoopback: false, allowCredentials: false }

describe('public endpoint resolution', () => {
  it('keeps a configured endpoint in development', () => {
    expect(
      resolvePublicEndpoint('http://localhost:8080/api/v1', '/api/v1', dev),
    ).toBe('http://localhost:8080/api/v1')
    expect(
      resolvePublicEndpoint(
        'https://devnet.helius-rpc.com/?api-key=public-demo-key',
        'https://api.devnet.solana.com',
        dev,
      ),
    ).toContain('api-key=')
  })

  it('strips loopback API endpoints from production builds', () => {
    expect(
      resolvePublicEndpoint('http://localhost:8080/api/v1', '/api/v1', prod),
    ).toBe('/api/v1')
    expect(
      resolvePublicEndpoint('http://127.0.0.1:8080/api/v1', '/api/v1', prod),
    ).toBe('/api/v1')
    expect(
      resolvePublicEndpoint('https://terra.example.com/api/v1', '/api/v1', prod),
    ).toBe('https://terra.example.com/api/v1')
  })

  it('strips embedded credentials from production RPC URLs unless opted in', () => {
    const keyed = 'https://devnet.helius-rpc.com/?api-key=secret-key'
    const fallback = 'https://api.devnet.solana.com'
    expect(resolvePublicEndpoint(keyed, fallback, prod)).toBe(fallback)
    expect(
      resolvePublicEndpoint(keyed, fallback, { ...prod, allowCredentials: true }),
    ).toBe(keyed)
  })

  it('falls back for missing or blank configuration', () => {
    expect(resolvePublicEndpoint(undefined, '/api/v1', prod)).toBe('/api/v1')
    expect(resolvePublicEndpoint('  ', '/api/v1', prod)).toBe('/api/v1')
  })

  it('recognises loopback and credential patterns without false positives', () => {
    expect(isLoopbackEndpoint('http://localhost:8080/api/v1')).toBe(true)
    expect(isLoopbackEndpoint('https://127.0.0.1/')).toBe(true)
    expect(isLoopbackEndpoint('https://api.devnet.solana.com')).toBe(false)
    expect(hasEmbeddedCredential('https://rpc.example/?api-key=abc123')).toBe(true)
    expect(hasEmbeddedCredential('https://rpc.example/?cluster=devnet')).toBe(false)
  })
})
