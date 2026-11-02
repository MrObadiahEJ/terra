import { describe, it, expect } from 'vitest'
import { bytesToHex, hexToBytes, strToBytes } from './codec'

describe('codec — byte/hex helpers', () => {
  it('bytesToHex zero-pads each byte to two hex digits', () => {
    expect(bytesToHex([0, 1, 255])).toBe('0001ff')
  })

  it('bytesToHex accepts Uint8Array input', () => {
    expect(bytesToHex(new Uint8Array([0xab]))).toBe('ab')
  })

  it('hexToBytes decodes hex pairs and ignores an optional 0x prefix', () => {
    expect(hexToBytes('deadbeef')).toEqual([0xde, 0xad, 0xbe, 0xef])
    expect(hexToBytes('0xdeadbeef')).toEqual([0xde, 0xad, 0xbe, 0xef])
  })

  it('strToBytes encodes a string as utf-8 bytes', () => {
    expect(Array.from(strToBytes('hi'))).toEqual([104, 105])
  })
})
