import { useState } from 'react'
import { Cpu } from 'lucide-react'
import { Broadcast, ClockBar, DemoBanner, Field, VerdictCard } from './shared'
import { SIM_START, err, pushGate, type Gate, type Verdict } from './helpers'

const MAX_METADATA = 128 // device_identity.rs MAX_DEVICE_METADATA_LEN
const D_STATUS = ['ACTIVE', 'SUSPENDED', 'REVOKED'] as const

type Role = 'owner' | 'other'
type DAction =
  | 'register_device'
  | 'update_device'
  | 'rotate_device_key'
  | 'set_device_status'
  | 'set_device_calibration'
  | 'verify_device'

interface Call {
  ix: DAction
  ok: boolean
  summary: string
  verdict: Verdict
}

interface Device {
  deviceKey: string
  owner: string
  status: number
  metadataRef: string
  calibrationHash: string
  calibratedAt: number
}

export default function DevicePanel() {
  const [device, setDevice] = useState<Device | null>(null)
  const [keyInput, setKeyInput] = useState('DemoDeviceKey9xK2pQ7mN4wR8tV1yZ6uJ3hF5sL0aB')
  const [zeroKey, setZeroKey] = useState(false)
  const [metadata, setMetadata] = useState('sensor://demo/imu-v2')
  const [owner, setOwner] = useState('DemoOwner111111111111111111111111111111111')
  const [role, setRole] = useState<Role>('owner')
  const [newStatus, setNewStatus] = useState(0)
  const [calib, setCalib] = useState('sha256:9f2c4e…demo')
  const [zeroCalib, setZeroCalib] = useState(false)
  const [now, setNow] = useState(SIM_START)
  const [last, setLast] = useState<Call | null>(null)

  const run = (ix: DAction) => {
    const gates: Gate[] = []
    let ok = true
    let summary = ''
    const fail = (name: string, detail: string) => {
      if (ok) {
        ok = false
        summary = `Rejected — ${err(name)} — ${detail}`
      }
      pushGate(gates, name, false, detail)
    }
    const pass = (label: string, detail: string) => pushGate(gates, label, true, detail)

    switch (ix) {
      case 'register_device': {
        if (zeroKey || keyInput === '') fail('InvalidDeviceKey', 'device_key = Pubkey::default() / empty')
        else pass('device_key ≠ default', keyInput.slice(0, 18) + '…')
        if (metadata.length > MAX_METADATA) fail('DeviceMetadataTooLong', `metadata_ref len ${metadata.length} > ${MAX_METADATA}`)
        else pass(`metadata_ref.len() ≤ ${MAX_METADATA}`, `${metadata.length} chars`)
        if (newStatus > 2) fail('InvalidDeviceStatus', `status = ${newStatus} > device_status::MAX = 2`)
        else pass('status ≤ device_status::MAX', `${newStatus} = ${D_STATUS[newStatus]}`)
        if (role !== 'owner') fail('NotDeviceOwner', `signer ${role} ≠ device owner`)
        else pass('signer == owner', 'owner signs')
        if (device !== null) fail('InvalidStatus', 'device record already registered for this PDA')
        else pass('device PDA free', 'no record yet')
        if (ok) {
          setDevice({ deviceKey: keyInput, owner, status: newStatus, metadataRef: metadata, calibrationHash: '', calibratedAt: 0 })
          summary = `DeviceRegistered — status = ${D_STATUS[newStatus]}, owner = ${owner.slice(0, 12)}…`
        }
        break
      }
      case 'update_device': {
        if (!device) fail('InvalidStatus', 'register_device first')
        else pass('device exists', `status = ${D_STATUS[device.status]}`)
        if (device?.status === 2) fail('DeviceRevoked', 'REVOKED is terminal — no further writes accepted')
        else if (device) pass('status ≠ REVOKED', D_STATUS[device.status])
        if (role === 'other') fail('NotDeviceOwner', 'only the owner writes metadata')
        else pass('signer == owner', 'owner signs')
        if (metadata.length > MAX_METADATA) fail('DeviceMetadataTooLong', `len ${metadata.length} > ${MAX_METADATA}`)
        else pass(`len ≤ ${MAX_METADATA}`, `${metadata.length} chars`)
        if (ok && device) {
          setDevice({ ...device, metadataRef: metadata })
          summary = `DeviceUpdated — metadata_ref → ${metadata.slice(0, 32)}`
        }
        break
      }
      case 'rotate_device_key': {
        if (!device) fail('InvalidStatus', 'register_device first')
        else pass('device exists', `status = ${D_STATUS[device.status]}`)
        if (device?.status === 2) fail('DeviceRevoked', 'REVOKED is terminal')
        else if (device) pass('status ≠ REVOKED', D_STATUS[device.status])
        if (role === 'other') fail('NotDeviceOwner', 'only the owner rotates the device key')
        else pass('signer == owner', 'owner signs')
        if (zeroKey || keyInput === '') fail('InvalidDeviceKey', 'new_device_key = default / empty')
        else pass('new_device_key ≠ default', keyInput.slice(0, 18) + '…')
        if (ok && device) {
          setDevice({ ...device, deviceKey: keyInput })
          summary = `DeviceKeyRotated — device_key → ${keyInput.slice(0, 18)}…`
        }
        break
      }
      case 'set_device_status': {
        if (!device) fail('InvalidStatus', 'register_device first')
        else pass('device exists', `status = ${D_STATUS[device.status]}`)
        if (newStatus > 2) fail('InvalidDeviceStatus', `status = ${newStatus} > MAX = 2`)
        else pass('status ≤ MAX', `${newStatus} = ${D_STATUS[newStatus]}`)
        if (device?.status === 2) fail('DeviceRevoked', 'REVOKED is terminal — status can never change again')
        else if (device) pass('not terminal', D_STATUS[device.status])
        if (role === 'other') fail('NotDeviceOwner', 'only the owner changes device status')
        else pass('signer == owner', 'owner signs')
        if (ok && device) {
          setDevice({ ...device, status: newStatus })
          summary = `DeviceStatusChanged — → ${D_STATUS[newStatus]}${newStatus === 2 ? ' (terminal)' : ''}`
        }
        break
      }
      case 'set_device_calibration': {
        if (!device) fail('InvalidStatus', 'register_device first')
        else pass('device exists', `status = ${D_STATUS[device.status]}`)
        if (device?.status === 2) fail('DeviceRevoked', 'REVOKED is terminal')
        else if (device) pass('status ≠ REVOKED', D_STATUS[device.status])
        if (role === 'other') fail('NotDeviceOwner', 'only the owner records calibration')
        else pass('signer == owner', 'owner signs')
        if (zeroCalib || calib === '') fail('EmptyContentHash', 'calibration_hash = [0u8; 32] — all-zero means never calibrated')
        else pass('calibration_hash ≠ [0;32]', calib.slice(0, 24))
        if (ok && device) {
          setDevice({ ...device, calibrationHash: calib, calibratedAt: now })
          summary = `DeviceCalibrationSet — calibrated_at = ${new Date(now * 1000).toISOString().slice(0, 16).replace('T', ' ')}Z`
        }
        break
      }
      case 'verify_device': {
        if (!device) fail('InvalidStatus', 'no device record')
        else pass('device exists', `status = ${D_STATUS[device.status]}`)
        if (device?.status === 2) fail('DeviceRevoked', 'REVOKED device cannot be verified')
        else if (device?.status === 1) pass('status SUSPENDED (resumable)', 'suspended — verification flagged, not fatal')
        else if (device) pass('status ACTIVE', 'ACTIVE')
        if (device && device.calibrationHash === '') pass('calibration recorded?', 'never calibrated — verification succeeds but report notes "never" (informational)')
        else if (device) pass('calibration fresh', `calibrated ${new Date(device.calibratedAt * 1000).toISOString().slice(0, 10)}`)
        if (ok && device) {
          summary = `DeviceVerified — status ${D_STATUS[device.status]}; hardware attestation is checked off-chain (MVP gap: attestation layer not wired into the program)`
        }
        break
      }
    }
    setLast({ ix, ok, summary, verdict: { ok, headline: summary, gates } })
  }

  return (
    <div className="lab-body">
      <DemoBanner source="device_identity.rs" />

      <div className="lab-card mt-3 space-y-2">
        <div className="flex items-center justify-between flex-wrap gap-2">
          <h3 className="text-[13px] font-semibold flex items-center gap-1.5">
            <Cpu size={14} /> Device lifecycle: register → calibrate → rotate → suspend/revoke
          </h3>
          <span className={`lab-badge ${device === null ? 'lab-badge-mut' : device.status === 2 ? 'lab-badge-err' : device.status === 1 ? 'lab-badge-warn' : 'lab-badge-ok'}`}>
            {device === null ? 'NO DEVICE' : D_STATUS[device.status]}
          </span>
        </div>
        <div className="pg-steps">
          {['REGISTERED', 'CALIBRATED', 'VERIFIED', 'REVOKED'].map((s, i) => {
            const st =
              device === null
                ? 'pending'
                : i === 0
                  ? 'done'
                  : i === 1
                    ? device.calibrationHash ? 'done' : 'pending'
                    : i === 2
                      ? 'pending'
                      : device.status === 2 ? 'failed' : 'pending'
            return (
              <span key={s} className={`pg-step pg-step-${st}`}>
                {s}
              </span>
            )
          })}
        </div>
        <div className="pg-fields">
          <Field label="device_key">
            <input className="select-input" value={keyInput} onChange={(e) => setKeyInput(e.target.value)} />
          </Field>
          <Field label="device_key validity">
            <select className="select-input" value={zeroKey ? 'zero' : 'ok'} onChange={(e) => setZeroKey(e.target.value === 'zero')}>
              <option value="ok">valid pubkey</option>
              <option value="zero">Pubkey::default() (zero)</option>
            </select>
          </Field>
          <Field label={`metadata_ref (≤ ${MAX_METADATA} chars)`}>
            <input className="select-input" value={metadata} onChange={(e) => setMetadata(e.target.value)} />
          </Field>
          <Field label="owner">
            <input className="select-input" value={owner} onChange={(e) => setOwner(e.target.value)} />
          </Field>
          <Field label="signer role">
            <select className="select-input" value={role} onChange={(e) => setRole(e.target.value as Role)}>
              <option value="owner">device owner</option>
              <option value="other">unrelated signer</option>
            </select>
          </Field>
          <Field label="target status (register / set_device_status)">
            <select className="select-input" value={newStatus} onChange={(e) => setNewStatus(Number(e.target.value))}>
              <option value={0}>0 — ACTIVE</option>
              <option value={1}>1 — SUSPENDED</option>
              <option value={2}>2 — REVOKED (terminal)</option>
              <option value={3}>3 — invalid (&gt; MAX)</option>
            </select>
          </Field>
          <Field label="calibration_hash">
            <input className="select-input" value={calib} onChange={(e) => setCalib(e.target.value)} />
          </Field>
          <Field label="calibration value">
            <select className="select-input" value={zeroCalib ? 'zero' : 'ok'} onChange={(e) => setZeroCalib(e.target.value === 'zero')}>
              <option value="ok">SHA-256 cert</option>
              <option value="zero">[0u8; 32] (never)</option>
            </select>
          </Field>
        </div>
        <div className="pg-actions">
          {(['register_device', 'update_device', 'rotate_device_key', 'set_device_status', 'set_device_calibration', 'verify_device'] as const).map((ix) => (
            <button key={ix} className="btn btn-secondary px-2 py-1 text-[11px]" onClick={() => run(ix)}>
              {ix}
            </button>
          ))}
          <button
            className="btn btn-ghost px-2 py-1 text-[11px]"
            onClick={() => {
              setDevice(null)
              setLast(null)
              setMetadata('sensor://demo/imu-v2')
              setZeroKey(false)
              setZeroCalib(false)
              setNewStatus(0)
            }}
          >
            reset
          </button>
        </div>
        <p className="text-[10px] text-muted">
          {device
            ? `key = ${device.deviceKey.slice(0, 20)}… · metadata = ${device.metadataRef} · ${
                device.calibrationHash ? `calibrated ${new Date(device.calibratedAt * 1000).toISOString().slice(0, 10)}` : 'never calibrated'
              }`
            : 'No device record yet — register_device creates the PDA (owner-signed).'}
        </p>
      </div>

      <div className="mt-3">{last && <VerdictCard title={`${last.ix} — guard trace`} verdict={last.verdict} />}</div>

      <Broadcast
        ix={last?.ix ?? 'register_device'}
        ok={last?.ok ?? false}
        summary={last?.summary ?? 'register_device not run yet — no device PDA'}
        note="REVOKED is terminal: every later call fails with DeviceRevoked (6223)."
      />

      <ClockBar now={now} onAdvance={(s) => setNow(now + s)} onReset={() => setNow(SIM_START)} />
    </div>
  )
}
