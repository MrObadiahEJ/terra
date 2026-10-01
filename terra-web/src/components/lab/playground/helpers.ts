import { reportTx } from '../../../lib/txStore'
import { terraRegistry } from '../../../idl/terraRegistry'

const ERR_CODE = new Map<string, number>()
for (const e of terraRegistry.errors ?? []) ERR_CODE.set(e.name, e.code)

/** Exact `TerraError` variant → `Name (6000+index)` string, IDL-verified. */
export function err(name: string): string {
  const code = ERR_CODE.get(name)
  if (code === undefined) throw new Error(`playground: TerraError::${name} not in IDL`)
  return `${name} (${code})`
}

export function tx(ix: string, ok: boolean, summary: string): void {
  reportTx(ix, ok, summary)
}

export type GateState = 'pass' | 'fail' | 'skip'
export interface Gate {
  label: string
  state: GateState
  detail: string
}
export interface Verdict {
  ok: boolean
  headline: string
  gates: Gate[]
}

export function pushGate(gates: Gate[], label: string, pass: boolean, detail: string): boolean {
  gates.push({ label, state: pass ? 'pass' : 'fail', detail })
  return pass
}

/** Frozen simulated clock base — module const (react-hooks/purity). */
export const SIM_START = 1_735_689_600 // 2025-01-01T00:00:00Z
export const DAY = 86_400
export const HOUR = 3_600

export function fmtSim(ts: number): string {
  return `${new Date(ts * 1000).toISOString().slice(0, 16).replace('T', ' ')}Z`
}
