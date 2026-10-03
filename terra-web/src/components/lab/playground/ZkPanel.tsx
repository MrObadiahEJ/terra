import { useEffect, useState } from 'react'
import { KeyRound, ShieldCheck } from 'lucide-react'
import { ed25519 as nobleEd25519 } from '@noble/curves/ed25519'
import { PublicKey } from '@solana/web3.js'
import { api, type NullifierRecord, type OwnershipRoot, type ZkServiceStatus, type ZoneSet } from '../../../lib/api'
import { Broadcast, DemoBanner, Field, VerdictCard } from './shared'
import { err, pushGate, type Gate, type Verdict } from './helpers'

const MAX_PROOF = 1024 // zk/mod.rs MAX_ZK_PROOF_SIZE
const MAX_PURPOSE = 128
const MAX_DISCLOSURE = 2 // disclosure_type::MAX = COUNT
const HEX_32 = /^[0-9a-f]{64}$/i

type ZAction = 'register_zone_set' | 'generate_ownership_root' | 'verify_ownership_proof' | 'invalidate_proof' | 'update_verification_key_hash'

interface Call {
  ix: ZAction
  ok: boolean
  summary: string
  verdict: Verdict
}

function parseFieldHex(value: string, label: string): string {
  const normalized = value.trim().replace(/^0x/i, '')
  if (!HEX_32.test(normalized)) throw new Error(`${label} must be exactly 32 bytes of hex`)
  return normalized.toLowerCase()
}

function parseStatementHex(value: string): Uint8Array {
  const normalized = value.trim().replace(/^0x/i, '')
  if (normalized.length % 2 !== 0 || !/^[0-9a-f]*$/i.test(normalized)) {
    throw new Error('Backend returned a malformed ownership statement')
  }
  return Uint8Array.from(normalized.match(/.{2}/g) ?? [], (byte) => Number.parseInt(byte, 16))
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
  const [sigValid, setSigValid] = useState(true)
  const [staleVersion, setStaleVersion] = useState(0)
  const [used, setUsed] = useState<string[]>([])
  const [last, setLast] = useState<Call | null>(null)
  const [service, setService] = useState<ZkServiceStatus | null>(null)
  const [serviceError, setServiceError] = useState<string | null>(null)
  const [zoneSets, setZoneSets] = useState<ZoneSet[]>([])
  const [selectedZoneId, setSelectedZoneId] = useState('')
  const [ownershipRoots, setOwnershipRoots] = useState<OwnershipRoot[]>([])
  const [rootVersion, setRootVersion] = useState(0)
  const [secret, setSecret] = useState('')
  const [siblingsText, setSiblingsText] = useState('')
  const [pathIndicesText, setPathIndicesText] = useState('')
  const [liveBusy, setLiveBusy] = useState(false)
  const [liveError, setLiveError] = useState<string | null>(null)
  const [verifiedProof, setVerifiedProof] = useState<NullifierRecord | null>(null)

  useEffect(() => {
    let active = true
    api.zkStatus()
      .then((result) => { if (active) setService(result) })
      .catch((error: unknown) => { if (active) setServiceError(error instanceof Error ? error.message : String(error)) })
    api.listZoneSets()
      .then((result) => {
        if (!active) return
        setZoneSets(result)
        setSelectedZoneId((current) => current || result[0]?.id || '')
      })
      .catch((error: unknown) => { if (active) setServiceError(error instanceof Error ? error.message : String(error)) })
    return () => { active = false }
  }, [])

  useEffect(() => {
    let active = true
    if (!selectedZoneId) return () => { active = false }
    api.listOwnershipRoots(selectedZoneId)
      .then((result) => {
        if (!active) return
        setOwnershipRoots(result)
        const current = zoneSets.find((zoneSet) => zoneSet.id === selectedZoneId)?.current_root_version
        setRootVersion(current ?? result.at(-1)?.version ?? 0)
      })
      .catch((error: unknown) => { if (active) setLiveError(error instanceof Error ? error.message : String(error)) })
    return () => { active = false }
  }, [selectedZoneId, zoneSets])

  const runLiveProof = async () => {
    if (!service?.development_proving_enabled) {
      setLiveError('Live proving is unavailable: configure the API in development mode with a matching proving key.')
      return
    }
    if (!selectedZoneId || rootVersion < 1) {
      setLiveError('Choose a zone set with a published ownership root.')
      return
    }
    setLiveBusy(true)
    setLiveError(null)
    setVerifiedProof(null)
    let ephemeralSecret: Uint8Array | null = null
    try {
      const secretHex = parseFieldHex(secret, 'Private membership secret')
      const siblings = siblingsText.trim().split(/[\s,]+/).filter(Boolean)
        .map((value, index) => parseFieldHex(value, `Sibling ${index + 1}`))
      const pathParts = pathIndicesText.trim().split(/[\s,]+/).filter(Boolean)
      if (siblings.length !== service.tree_depth) {
        throw new Error(`Enter exactly ${service.tree_depth} sibling hashes (leaf-to-root order).`)
      }
      if (pathParts.length !== service.tree_depth || pathParts.some((value) => value !== '0' && value !== '1')) {
        throw new Error(`Enter exactly ${service.tree_depth} path bits (0 or 1, leaf-to-root order).`)
      }

      ephemeralSecret = nobleEd25519.utils.randomSecretKey()
      const presenter = new PublicKey(nobleEd25519.getPublicKey(ephemeralSecret)).toBase58()
      const proofContext = {
        root_version: rootVersion,
        presenter,
        proof_purpose: 'membership',
        disclosure_type: 0,
        secret: secretHex,
      }
      const prepared = await api.prepareDevProof(selectedZoneId, proofContext)
      const statement = parseStatementHex(prepared.statement)
      const signature = nobleEd25519.sign(statement, ephemeralSecret)
      const generated = await api.generateDevProof(selectedZoneId, {
        ...proofContext,
        siblings,
        path_indices: pathParts.map((value) => value === '1'),
        statement_signature: Array.from(signature as Uint8Array, (byte: number) => byte.toString(16).padStart(2, '0')).join(''),
      })
      const record = await api.verifyOwnershipProof(selectedZoneId, {
        nullifier_hash: prepared.nullifier_hash,
        root_version: rootVersion,
        presenter,
        proof_purpose: 'membership',
        disclosure_type: 0,
        proof_data: generated.proof_data,
      })
      setVerifiedProof(record)
      setSecret('')
      setSiblingsText('')
      setPathIndicesText('')
    } catch (error) {
      setLiveError(error instanceof Error ? error.message : String(error))
    } finally {
      ephemeralSecret?.fill(0)
      setLiveBusy(false)
    }
  }

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
        if (!sigValid) fail('InvalidProofData', 'Ed25519 statement signature invalid (precompiled sysvar check)')
        else pass('statement Ed25519 signature', 'covers zone, root, version, nullifier, prover, purpose, disclosure')
        if (!pairing) fail('InvalidProofData', 'Groth16 pairing equation failed for the 248-bit SHA-256 statement digest')
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
      <div className="lab-card space-y-3">
        <div className="flex items-start justify-between gap-3 flex-wrap">
          <div>
            <h3 className="text-[13px] font-semibold flex items-center gap-1.5">
              <ShieldCheck size={14} /> Live Groth16 membership proof
            </h3>
            <p className="text-[11px] text-muted mt-1">
              Generates a fresh one-time presenter key, proves membership against the selected root,
              then submits the proof for server-side verification and nullifier recording.
            </p>
          </div>
          <span className={`lab-badge ${service?.verification_enabled ? 'lab-badge-ok' : 'lab-badge-warn'}`}>
            {service?.mode ?? (serviceError ? 'API OFFLINE' : 'CHECKING')} {service?.mode === 'development' ? 'PROVER' : ''}
          </span>
        </div>
        {service && (
          <p className="text-[10px] text-muted break-all">
            {service.circuit} · depth {service.tree_depth} · VK {service.verification_key_hash ?? 'not configured'}
          </p>
        )}
        {serviceError && <p role="alert" className="text-[11px] text-red-700">{serviceError}</p>}
        <div className="pg-fields">
          <Field label="zone set">
            <select
              className="select-input"
              value={selectedZoneId}
              onChange={(event) => {
                const next = event.target.value
                setSelectedZoneId(next)
                if (!next) {
                  setOwnershipRoots([])
                  setRootVersion(0)
                }
              }}
            >
              <option value="">Select a zone set</option>
              {zoneSets.map((zoneSet) => (
                <option key={zoneSet.id} value={zoneSet.id}>
                  {zoneSet.zone_id} · root v{zoneSet.current_root_version}
                </option>
              ))}
            </select>
          </Field>
          <Field label="current root version">
            <select className="select-input" value={rootVersion} onChange={(event) => setRootVersion(Number(event.target.value))}>
              {ownershipRoots.map((root) => (
                <option key={root.id} value={root.version}>
                  v{root.version} · {root.commitment_count} commitments
                </option>
              ))}
            </select>
          </Field>
          <Field label="private membership secret (hex, 32 bytes)">
            <input
              className="select-input font-mono"
              type="password"
              autoComplete="off"
              spellCheck={false}
              value={secret}
              onChange={(event) => setSecret(event.target.value)}
              placeholder="32-byte private witness"
            />
          </Field>
          <Field label={`Merkle siblings (${service?.tree_depth ?? 20}, leaf-to-root; one hex hash per line)`}>
            <textarea
              className="select-input font-mono min-h-24"
              spellCheck={false}
              value={siblingsText}
              onChange={(event) => setSiblingsText(event.target.value)}
              placeholder={'One 32-byte sibling hash per line'}
            />
          </Field>
          <Field label={`Merkle path bits (${service?.tree_depth ?? 20}, leaf-to-root)`}>
            <input
              className="select-input font-mono"
              spellCheck={false}
              value={pathIndicesText}
              onChange={(event) => setPathIndicesText(event.target.value)}
              placeholder={`20 bits, e.g. ${'0'.repeat(service?.tree_depth ?? 20)}`}
            />
          </Field>
        </div>
        <p className="text-[10px] text-amber-800">
          Development proving sends the secret and Merkle witness to this loopback API. Do not expose a development
          prover through a public reverse proxy. The witness stays in this page only until submission.
        </p>
        {liveError && <p role="alert" className="text-[11px] text-red-700">{liveError}</p>}
        {verifiedProof && (
          <div className="lab-card" role="status">
            <p className="text-[12px] font-semibold text-emerald-700">Groth16 proof verified · nullifier recorded</p>
            <p className="text-[10px] font-mono break-all mt-1">{verifiedProof.nullifier_hash}</p>
            <p className="text-[10px] text-muted mt-1">One-time presenter: {verifiedProof.prover}</p>
          </div>
        )}
        <button
          className="btn btn-primary px-3 py-2 text-[12px] self-start"
          type="button"
          onClick={() => void runLiveProof()}
          disabled={liveBusy || !service?.development_proving_enabled || !ownershipRoots.length}
        >
          {liveBusy ? 'Building and verifying Groth16 proof…' : 'Generate & verify private membership proof'}
        </button>
      </div>

      <DemoBanner source="zk/mod.rs + zk/groth16.rs (guard-chain simulator)" />

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
                <option value="valid">901 B TG16 v1</option>
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
              <select className="select-input" value={sigValid ? 'ok' : 'bad'} onChange={(e) => setSigValid(e.target.value === 'ok')}>
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
            This lower panel is a guard-chain simulator only; use the live proof flow above to generate and verify a real proof.
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
