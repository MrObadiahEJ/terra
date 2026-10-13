// Verification-media API client: gallery queries + signed multipart upload.
// The gallery reads are plain GETs; uploads reuse the SignedRequest auth
// protocol (Authorization: Signature …, x-timestamp, x-body-hash) so the
// server can record the caller's wallet as uploaded_by.

import { API_BASE } from './constants'

export interface EvidenceUpload {
  id: string
  content_hash: string
  storage_ref: string
  storage_backend: string
  filename: string
  content_type: string
  size_bytes: number
  uploaded_by: string | null
  parcel_id: string | null
  created_at: string
}

export function evidenceFileUrl(id: string): string {
  return `${API_BASE}/evidence/${id}/file`
}

export async function listEvidence(
  opts: { parcelId?: string; contentHash?: string } = {},
): Promise<EvidenceUpload[]> {
  const params = new URLSearchParams()
  if (opts.parcelId) params.set('parcel_id', opts.parcelId)
  if (opts.contentHash) params.set('content_hash', opts.contentHash)
  const qs = params.toString()
  const res = await fetch(`${API_BASE}/evidence${qs ? `?${qs}` : ''}`)
  if (!res.ok) throw new Error(`evidence list failed: ${res.status}`)
  return (await res.json()) as EvidenceUpload[]
}

function sha256Hex(bytes: BufferSource): Promise<string> {
  return crypto.subtle.digest('SHA-256', bytes).then((digest) =>
    Array.from(new Uint8Array(digest), (b) => b.toString(16).padStart(2, '0')).join(''),
  )
}

/**
 * Upload one file as verification media, optionally linked to a parcel.
 * `signMessage` is the connected wallet's Ed25519 message signer; the
 * multipart body is assembled here (not via FormData) so its exact bytes
 * can be hashed and signed for the SignedRequest protocol.
 */
export async function uploadEvidence(
  file: File,
  opts: {
    parcelId?: string
    wallet: string
    signMessage: (message: Uint8Array) => Promise<Uint8Array>
  },
): Promise<EvidenceUpload> {
  const boundary = `----TerraUpload${Math.random().toString(36).slice(2)}`
  const enc = new TextEncoder()
  const parts: Uint8Array[] = []

  const pushPart = (header: string, body: Uint8Array | string) => {
    parts.push(enc.encode(`--${boundary}\r\n${header}\r\n`))
    parts.push(typeof body === 'string' ? enc.encode(body) : body)
    parts.push(enc.encode('\r\n'))
  }

  pushPart(
    `Content-Disposition: form-data; name="file"; filename="${file.name.replace(/"/g, '')}"\r\nContent-Type: ${file.type || 'application/octet-stream'}`,
    new Uint8Array(await file.arrayBuffer()),
  )
  if (opts.parcelId) {
    pushPart('Content-Disposition: form-data; name="parcel_id"', opts.parcelId)
  }
  parts.push(enc.encode(`--${boundary}--\r\n`))

  const body = new Uint8Array(parts.reduce((n, p) => n + p.length, 0))
  let offset = 0
  for (const part of parts) {
    body.set(part, offset)
    offset += part.length
  }

  const bodyHash = await sha256Hex(body)
  const timestamp = Math.floor(Date.now() / 1000)
  const payload = `POST\n${API_BASE}/evidence/upload\n${bodyHash}\n${timestamp}`
  const signature = await opts.signMessage(enc.encode(payload))
  const signatureBase64 = btoa(String.fromCharCode(...signature))

  const res = await fetch(`${API_BASE}/evidence/upload`, {
    method: 'POST',
    body,
    headers: {
      'Content-Type': `multipart/form-data; boundary=${boundary}`,
      Authorization: `Signature ${signatureBase64}`,
      'x-timestamp': String(timestamp),
      'x-body-hash': bodyHash,
      'x-wallet': opts.wallet,
    },
  })
  if (!res.ok) {
    let detail = res.statusText
    try {
      const err = await res.json()
      detail = err?.error ?? err?.message ?? detail
    } catch {
      /* non-JSON error body */
    }
    throw new Error(`upload failed (${res.status}): ${detail}`)
  }
  return (await res.json()) as EvidenceUpload
}
