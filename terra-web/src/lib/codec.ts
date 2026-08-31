// Browser-safe byte/hex helpers to avoid depending on Node's Buffer.

export function bytesToHex(bytes: Uint8Array | number[]): string {
  return Array.from(bytes)
    .map((b) => b.toString(16).padStart(2, '0'))
    .join('')
}

export function hexToBytes(hex: string): number[] {
  const clean = hex.replace(/^0x/, '')
  const out: number[] = []
  for (let i = 0; i < clean.length; i += 2) {
    out.push(parseInt(clean.slice(i, i + 2), 16))
  }
  return out
}

export function strToBytes(s: string): Uint8Array {
  return new TextEncoder().encode(s)
}
