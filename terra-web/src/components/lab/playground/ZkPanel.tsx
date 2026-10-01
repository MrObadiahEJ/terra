import { useState } from 'react'
import { KeyRound, ShieldCheck } from 'lucide-react'
import { Broadcast, DemoBanner, Field, VerdictCard } from './shared'
import { err, pushGate, type Gate, type Verdict } from './helpers'

const MAX_PROOF = 1024 // zk/mod.rs MAX_ZK_PROOF_SIZE
const MAX_PURPOSE = 128
const MAX_DISCLOSURE = 2 // disclosure_type::MAX = COUNT

type ZAction = 'register_zone_set' | 'generate_ownership_root' | 'verify_ownership_proof' | 'invalidate_proof' | 'update_verification_key_hash'

interface Call {
  ix: ZAction
  ok: boolean
  summary: string
  verdict: Verdict
}

export default function ZkPanel() {
  const [zone, setZone] = useState<{ commitmentCount: number; rootVersion: number } | null>(null)
  const [role, setRole] = useState<'authority' | 'other'>('authority')
  const [commitments, setCommitments] = useState(64)
  const [cid, setCid] = useState(true)
  const [geomHash, setGeomHash] = useState(true)
  const [purpose, setPurpose] = useState('membership')
  const [disclosure, setDisclosure] = useState(0)
  const [presentedVersion, setPresentedVersion] = useState(1)
  const [nullifier, setNullifier] = useState<'valid' | 'zero'>('valid')
  const [proofSize, setProofSize] = useState<'valid' | 'empty' | 'oversize'>('valid')
  const [frame, setFrame] = useState<'ok' | 'bad_magic'>('ok')
  const [vkMatch, setVkMatch] = useState(true)
  const [pairing, setPairing] = useState(true)
  const [ed25519, setEd25519] = useState(true)
  const [staleVersion, setStaleVersion] = useState(0)
  const [used, setUsed] = useState<string[]>([])
  const [last, setLast] = useState<Call | null>(null)

  const run = (ix: ZAction) => {
    const gates: Gate[] = []
    let ok = true
    let summary = ''
    const fail = (name: string, detail: string) => {
      if (ok) {
        ok = false
        summary = `Rejected — ${err(name)} — ${detail}`
        pushGate(gates, name, false, detail)
      }
    }
    const pass = (label: string, detail: string) => {
      if (ok) pushGate(gates, label, true, detail)
    }

    switch (ix) {
      case 'register_zone_set': {
        if (!cid) fail('CidRequired', 'snapshot_cid empty — zone registration requires a content-addressed snapshot')
        else pass('snapshot_cid ≤ 128 chars', 'CID present')
        if (!geomHash) fail('EmptyGeometryHash', 'zone geometry_hash = [0u8; 32]')
        else pass('zone geometry_hash set', 'SHA-256 of zone geometry')
        if (role !== 'authority') fail('UnauthorizedZoneAuthority', `signer ≠ zone_set.authority`)
        else pass('signer == zone_set.authority', 'authority signs')
        if (zone !== null) pass('re-register allowed', 'zone_set PDA exists — registration refreshes metadata')
        if (ok) {
          setZone({ commitmentCount: zone?.commitmentCount ?? 0, rootVersion: zone?.rootVersion ?? 0 })
          summary = 'ZoneSetRegistered — no ownership root yet (root_version = 0); call generate_ownership_root'
        }
        break
      }
      case 'generate_ownership_root': {
        if (role !== 'authority') fail('UnauthorizedZoneAuthority', 'only the zone authority publishes roots')
        else pass('signer == zone_set.authority', 'authority signs')
        if (!geomHash) fail('EmptyGeometryHash', 'ownership root geometry_hash empty')
        else pass('geometry_hash set', 'present')
        if (!cid) fail('CidRequired', 'snapshot_cid required for ownership root publish')
        else pass('snapshot_cid present', 'CID pinned')
        if (commitments <= 0) fail('EmptyZoneSet', 'commitment_count = 0 — empty zone set cannot produce a root')
        else pass('commitment_count > 0', `${commitments} commitments`)
        if (zone === null) fail('InvalidStatus', 'register_zone_set first — no zone_set PDA')
        else pass('zone_set exists', `root_version = ${zone.rootVersion}`)
        if (ok && zone) {
          const v = zone.rootVersion + 1
          setZone({ commitmentCount: commitments, rootVersion: v })
          setPresentedVersion(v)
          summary = `OwnershipRootUpdated — version → ${v}, merkle_root over ${commitments} commitments, verification_key_hash pinned`
        }
        break
      }
      case 'verify_ownership_proof': {
        // zk/mod.rs verify_ownership_proof — strict first-fail order.
        if (role !== 'authority') fail('UnauthorizedZoneAuthority', 'prover/authority key ≠ zone_set.authority')
        else pass('authority gate', 'signer matches zone_set.authority')
        if (nullifier === 'zero') fail('EmptyGeometryHash', 'nullifier_hash = [0u8; 32] (all-zero rejected)')
        else pass('nullifier_hash ≠ [0;32]', 'non-zero nullifier')
        const size = proofSize === 'valid' ? 896 : proofSize === 'empty' ? 0 : 1025
        if (size === 0 || size > MAX_PROOF) fail('ProofTooLarge', `proof_data.len() = ${size}; must be 1..${MAX_PROOF} (framed v1 payload, RFC-011 §6.3)`)
        else pass(`0 < proof_data.len() ≤ ${MAX_PROOF}`, `${size} bytes`)
        if (purpose === '') fail('InvalidProofPurpose', 'proof_purpose empty')
        else if (purpose.length > MAX_PURPOSE) fail('InvalidProofPurpose', `len ${purpose.length} > ${MAX_PURPOSE}`)
        else pass(`proof_purpose ≤ ${MAX_PURPOSE} chars`, purpose)
        if (disclosure > MAX_DISCLOSURE) fail('InvalidDisclosureType', `disclosure_type = ${disclosure} > MAX = ${MAX_DISCLOSURE} (MEMBERSHIP/RANGE/COUNT)`)
        else pass('disclosure_type ≤ MAX', `${disclosure} ∈ 0..${MAX_DISCLOSURE}`)
        if (zone === null || zone.rootVersion === 0) fail('RootVersionMismatch', 'no ownership root published yet (root_version = 0)')
        else if (presentedVersion !== zone.rootVersion) fail('RootVersionMismatch', `presented root_version ${presentedVersion} ≠ current ${zone.rootVersion}`)
        else pass('root_version == zone.current_root_version', `version ${zone.rootVersion}`)
        if (zone?.commitmentCount === 0) fail('EmptyZoneSet', 'root.commitment_count = 0')
        else if (zone) pass('root.commitment_count > 0', `${zone.commitmentCount}`)
        if (frame === 'bad_magic') fail('InvalidProofData', 'frame parse: magic/version/len mismatch — every proof-layer failure maps to InvalidProofData (6087)')
        else pass('frame parse (magic 4B + version 1B + body)', 'framed v1 payload ok')
        if (!vkMatch) fail('InvalidProofData', 'hash(frame.verification_key) ≠ root.verification_key_hash — VK pin mismatch')
        else pass('verification_key_hash pin', 'hash(vk) == pinned')
        if (!ed25519) fail('InvalidProofData', 'Ed25519 statement signature invalid (precompiled sysvar check)')
        else pass('statement Ed25519 signature', 'covers zone, root, version, nullifier, prover, purpose, disclosure')
        if (!pairing) fail('InvalidProofData', 'Groth16 pairing equation failed over SHA-256(statement) mod r')
        else pass('Groth16 pairing check', 'POSEIDON_GROTH16 proof accepted')
        if (ok) {
          if (nullifier === 'valid' && !used.includes('n-7f3a')) setUsed([...used, 'n-7f3a'])
          summary = `OwnershipProofVerified — version ${presentedVersion}, purpose "${purpose}", disclosure ${disclosure}; nullifier recorded (PDA ["nullifier", hash] now occupied — replay collides at the account layer)`
        }
        break
      }
      case 'invalidate_proof': {
        if (role !== 'authority') fail('UnauthorizedZoneAuthority', 'only the zone authority invalidates versions')
        else pass('signer == zone_set.authority', 'authority signs')
        if (staleVersion <= 0) fail('InvalidDisputeStatus', `stale_version = ${staleVersion}; must be > 0 (code reuse — source maps this guard to InvalidDisputeStatus)`)
        else pass('stale_version > 0', `${staleVersion}`)
        if (zone === null) fail('InvalidStatus', 'register_zone_set first')
        else if (staleVersion >= zone.rootVersion) fail('RootVersionMismatch', `stale_version ${staleVersion} ≥ current ${zone.rootVersion} — only older versions can be invalidated`)
        else pass('stale_version < current_root_version', `${staleVersion} < ${zone.rootVersion}`)
        if (ok && zone) {
          summary = `ProofVersionInvalidated — stale_version ${staleVersion} marked, current stays ${zone.rootVersion}`
        }
        break
      }
      case 'update_verification_key_hash': {
        if (role !== 'authority') fail('UnauthorizedZoneAuthority', 'VK hash rotated by authority only')
        else pass('signer == zone_set.authority', 'authority signs')
        if (zone === null || zone.rootVersion === 0) fail('InvalidStatus', 'no published root to re-pin against')
        else pass('root exists', `version ${zone.rootVersion} re-pinned`)
        if (ok) summary = 'VerificationKeyHashUpdated — new VK pinned; old proofs now fail the pin check (InvalidProofData)'
        break
      }
    }
    setLast({ ix, ok, summary, verdict: { ok, headline: summary, gates } })
  }

  return (
    <div className="lab-body">
      <DemoBanner source="zk/mod.rs + zk/groth16.rs (P0-ZK-01 framing)" />

      <div className="lab-grid mt-3">
        <div className="lab-card space-y-2">
          <h3 className="text-[13px] font-semibold flex items-center gap-1.5">
            <ShieldCheck size={14} /> Zone + root setup
          </h3>
          <div className="flex gap-1 items-center flex-wrap">
            <span className={`lab-badge ${zone === null ? 'lab-badge-mut' : zone.rootVersion === 0 ? 'lab-badge-warn' : 'lab-badge-ok'}`}>
              {zone === null ? 'NO ZONE' : zone.rootVersion === 0 ? 'ZONE, NO ROOT' : `ROOT v${zone.rootVersion}`}
            </span>
            <span className={`lab-badge ${used.length ? 'lab-badge-info' : 'lab-badge-mut'}`}>
              nullifiers used: {used.length}
            </span>
          </div>
          <div className="pg-fields">
            <Field label="signer role">
              <select className="select-input" value={role} onChange={(e) => setRole(e.target.value as 'authority' | 'other')}>
                <option value="authority">zone authority</option>
                <option value="other">other signer</option>
              </select>
            </Field>
            <Field label="commitment_count">
              <input
                className="select-input"
                type="number"
                value={commitments}
                onChange={(e) => setCommitments(Number(e.target.value) || 0)}
              />
            </Field>
            <Field label="snapshot_cid">
              <select className="select-input" value={cid ? 'ok' : 'empty'} onChange={(e) => setCid(e.target.value === 'ok')}>
                <option value="ok">present (≤128)</option>
                <option value="empty">empty</option>
              </select>
            </Field>
            <Field label="zone geometry_hash">
              <select className="select-input" value={geomHash ? 'ok' : 'zero'} onChange={(e) => setGeomHash(e.target.value === 'ok')}>
                <option value="ok">set</option>
                <option value="zero">all-zero</option>
              </select>
            </Field>
            <Field label="invalidate stale_version">
              <select className="select-input" value={staleVersion} onChange={(e) => setStaleVersion(Number(e.target.value))}>
                {[0, 1, 2, 3].map((v) => (
                  <option key={v} value={v}>{v}</option>
                ))}
              </select>
            </Field>
          </div>
          <div className="pg-actions">
            <button className="btn btn-secondary px-2 py-1 text-[11px]" onClick={() => run('register_zone_set')}>
              register_zone_set
            </button>
            <button className="btn btn-secondary px-2 py-1 text-[11px]" onClick={() => run('generate_ownership_root')}>
              generate_ownership_root
            </button>
            <button className="btn btn-secondary px-2 py-1 text-[11px]" onClick={() => run('invalidate_proof')}>
              invalidate_proof
            </button>
            <button className="btn btn-secondary px-2 py-1 text-[11px]" onClick={() => run('update_verification_key_hash')}>
              update_verification_key_hash
            </button>
            <button
              className="btn btn-ghost px-2 py-1 text-[11px]"
              onClick={() => {
                setZone(null)
                setUsed([])
                setLast(null)
                setPresentedVersion(1)
              }}
            >
              reset
            </button>
          </div>
        </div>

        <div className="lab-card space-y-2">
          <h3 className="text-[13px] font-semibold flex items-center gap-1.5">
            <KeyRound size={14} /> Framed ownership proof input
          </h3>
          <div className="pg-fields">
            <Field label="proof_purpose">
              <select className="select-input" value={purpose} onChange={(e) => setPurpose(e.target.value)}>
                <option value="membership">membership</option>
                <option value="ownership">ownership</option>
                <option value="">(empty — invalid)</option>
                <option value={'x'.repeat(129)}>(129 chars — too long)</option>
              </select>
            </Field>
            <Field label="disclosure_type">
              <select className="select-input" value={disclosure} onChange={(e) => setDisclosure(Number(e.target.value))}>
                <option value={0}>0 — MEMBERSHIP</option>
                <option value={1}>1 — RANGE</option>
                <option value={2}>2 — COUNT</option>
                <option value={3}>3 — invalid (&gt; MAX)</option>
              </select>
            </Field>
            <Field label="presented root_version">
              <select className="select-input" value={presentedVersion} onChange={(e) => setPresentedVersion(Number(e.target.value))}>
                {[0, 1, 2, 3].map((v) => (
                  <option key={v} value={v}>{v}</option>
                ))}
              </select>
            </Field>
            <Field label="nullifier_hash">
              <select className="select-input" value={nullifier} onChange={(e) => setNullifier(e.target.value as 'valid' | 'zero')}>
                <option value="valid">non-zero (n-7f3a…)</option>
                <option value="zero">[0u8; 32] (all-zero)</option>
              </select>
            </Field>
            <Field label="proof_data frame">
              <select className="select-input" value={proofSize} onChange={(e) => setProofSize(e.target.value as 'valid' | 'empty' | 'oversize')}>
                <option value="valid">896 B framed v1</option>
                <option value="empty">0 B (empty)</option>
                <option value="oversize">1025 B (&gt; MAX)</option>
              </select>
            </Field>
            <Field label="frame parse">
              <select className="select-input" value={frame} onChange={(e) => setFrame(e.target.value as 'ok' | 'bad_magic')}>
                <option value="ok">magic + version ok</option>
                <option value="bad_magic">bad magic (corrupt)</option>
              </select>
            </Field>
            <Field label="verification_key pin">
              <select className="select-input" value={vkMatch ? 'ok' : 'bad'} onChange={(e) => setVkMatch(e.target.value === 'ok')}>
                <option value="ok">hash(vk) == pinned</option>
                <option value="bad">mismatch (rotated VK)</option>
              </select>
            </Field>
            <Field label="Ed25519 statement sig">
              <select className="select-input" value={ed25519 ? 'ok' : 'bad'} onChange={(e) => setEd25519(e.target.value === 'ok')}>
                <option value="ok">valid</option>
                <option value="bad">invalid</option>
              </select>
            </Field>
            <Field label="Groth16 pairing">
              <select className="select-input" value={pairing ? 'ok' : 'bad'} onChange={(e) => setPairing(e.target.value === 'ok')}>
                <option value="ok">passes</option>
                <option value="bad">fails</option>
              </select>
            </Field>
          </div>
          <button className="btn btn-secondary px-2 py-1 text-[11px] self-start" onClick={() => run('verify_ownership_proof')}>
            verify_ownership_proof
          </button>
          <p className="text-[10px] text-muted">
            Production note (honest gap): the Groth16 circuit + trusted setup is the release gate — this panel
            mirrors the on-chain guard chain and frame parsing (RFC-011 §6.3.1), with the pairing/Ed25519
            outcomes switchable. Framing proof wiring is tracked as experimental, not implemented.
          </p>
        </div>
      </div>

      <div className="mt-3">{last && <VerdictCard title={`${last.ix} — guard trace (first-fail order)`} verdict={last.verdict} />}</div>

      <Broadcast
        ix={last?.ix ?? 'verify_ownership_proof'}
        ok={last?.ok ?? false}
        summary={last?.summary ?? 'verify_ownership_proof not run — no root published yet'}
        note="Proof-layer failures all land as InvalidProofData (6087) after the earlier guards pass."
      />
    </div>
  )
}
