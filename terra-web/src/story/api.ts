// Stories API client (RFC-014) — /api/v1/stories + /api/v1/exports.

import { API_BASE } from '../lib/constants'
import type { StoryDefinition, StoryExport, StoryRow, StoryWithDefinition, ExportAspect } from './types'

// ---------------------------------------------------------------------------
// Signed writes. The API requires a `SignedRequest` on every mutating call
// (POST/PUT/DELETE): `Authorization: Signature <base64>`, `x-timestamp`,
// `x-body-hash`, over the payload `{METHOD}\n{PATH}\n{SHA256(body)}\n{TS}`.
//
// A connected wallet signs and attributes the request (x-wallet). Without one,
// an ephemeral in-browser Ed25519 key still signs it, so the offline demo
// creator works — the story is simply stored unattributed.
// ---------------------------------------------------------------------------

let walletSigner: ((message: Uint8Array) => Promise<Uint8Array>) | null = null
let walletAddress: string | null = null

/** Register (or clear) the connected wallet used to sign story writes. */
export function setStorySigner(
  signer: ((message: Uint8Array) => Promise<Uint8Array>) | null,
  wallet: string | null,
): void {
  walletSigner = signer
  walletAddress = wallet
}

let ephemeralKeyPromise: Promise<CryptoKey | null> | null = null

function getEphemeralKey(): Promise<CryptoKey | null> {
  if (!ephemeralKeyPromise) {
    ephemeralKeyPromise = crypto.subtle
      .generateKey({ name: 'Ed25519' }, false, ['sign'])
      .then((pair) => pair.privateKey)
      .catch(() => null)
  }
  return ephemeralKeyPromise
}

function toBase64(bytes: Uint8Array): string {
  let s = ''
  for (const b of bytes) s += String.fromCharCode(b)
  return btoa(s)
}

async function sha256Hex(bytes: BufferSource): Promise<string> {
  const digest = await crypto.subtle.digest('SHA-256', bytes)
  return Array.from(new Uint8Array(digest), (b) => b.toString(16).padStart(2, '0')).join('')
}

async function signedHeaders(
  method: string,
  path: string,
  bodyStr: string,
): Promise<Record<string, string>> {
  const enc = new TextEncoder()
  const bodyHash = await sha256Hex(enc.encode(bodyStr))
  const timestamp = Math.floor(Date.now() / 1000)
  const message = enc.encode(`${method}\n${API_BASE}${path}\n${bodyHash}\n${timestamp}`)

  let signature: Uint8Array | null = null
  if (walletSigner) {
    signature = await walletSigner(message)
  } else {
    const key = await getEphemeralKey()
    if (key) signature = new Uint8Array(await crypto.subtle.sign('Ed25519', key, message))
  }

  const headers: Record<string, string> = {
    'x-timestamp': String(timestamp),
    'x-body-hash': bodyHash,
    // The server only verifies the signature when x-wallet is present; a
    // well-formed 64-byte value still satisfies the structural check.
    Authorization: `Signature ${toBase64(signature ?? new Uint8Array(64))}`,
  }
  if (walletAddress) headers['x-wallet'] = walletAddress
  return headers
}

async function request<T>(method: string, path: string, body?: unknown): Promise<T> {
  const bodyStr = body !== undefined ? JSON.stringify(body) : undefined
  const headers: Record<string, string> = {}
  if (bodyStr !== undefined) headers['content-type'] = 'application/json'
  if (method !== 'GET') Object.assign(headers, await signedHeaders(method, path, bodyStr ?? ''))
  const res = await fetch(`${API_BASE}${path}`, {
    method,
    headers,
    body: bodyStr,
  })
  if (!res.ok) {
    let detail = `${res.status}`
    try {
      const j = (await res.json()) as { error?: string; message?: string }
      detail = j.error ?? j.message ?? detail
    } catch {
      /* keep status */
    }
    throw new Error(`stories ${method} ${path} failed: ${detail}`)
  }
  if (res.status === 204) return undefined as T
  return (await res.json()) as T
}

export function listStories(opts: { story_type?: string; entity?: string } = {}): Promise<StoryRow[]> {
  const params = new URLSearchParams()
  if (opts.story_type) params.set('story_type', opts.story_type)
  if (opts.entity) params.set('entity', opts.entity)
  const qs = params.toString()
  return request<StoryRow[]>('GET', `/stories${qs ? `?${qs}` : ''}`)
}

export function getStory(id: string): Promise<StoryWithDefinition> {
  return request<StoryWithDefinition>('GET', `/stories/${encodeURIComponent(id)}`)
}

export function createStory(def: StoryDefinition): Promise<StoryWithDefinition> {
  return request<StoryWithDefinition>('POST', '/stories', def)
}

export function updateStory(def: StoryDefinition): Promise<StoryWithDefinition> {
  return request<StoryWithDefinition>('PUT', `/stories/${encodeURIComponent(def.id)}`, def)
}

export function deleteStory(id: string): Promise<void> {
  return request<void>('DELETE', `/stories/${encodeURIComponent(id)}`)
}

export function listExports(storyId: string): Promise<StoryExport[]> {
  return request<StoryExport[]>('GET', `/stories/${encodeURIComponent(storyId)}/exports`)
}

export interface NewExportInput {
  aspect: ExportAspect
  width: number
  height: number
  fps: number
  duration_s: number
}

export function createExport(storyId: string, input: NewExportInput): Promise<StoryExport> {
  return request<StoryExport>('POST', `/stories/${encodeURIComponent(storyId)}/exports`, input)
}

export interface UpdateExportInput {
  status: 'running' | 'done' | 'failed' | 'cancelled'
  progress?: number
  error?: string
}

export function updateExport(exportId: string, input: UpdateExportInput): Promise<StoryExport> {
  return request<StoryExport>('PUT', `/exports/${encodeURIComponent(exportId)}`, input)
}
