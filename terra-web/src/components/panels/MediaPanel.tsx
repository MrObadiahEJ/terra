import { useCallback, useEffect, useRef, useState } from 'react'
import { Film, FileAudio, FileImage, FileText, Upload, X } from 'lucide-react'
import { useLocale } from '../../lib/locale'
import { useWallet } from '../../lib/wallet'
import {
  evidenceFileUrl,
  listEvidence,
  uploadEvidence,
  type EvidenceUpload,
} from '../../lib/evidence'

interface Props {
  /** Parcel the gallery should filter to, or null for all media. */
  parcelId: string | null
}

function fmtBytes(n: number): string {
  if (n >= 1024 * 1024) return `${(n / (1024 * 1024)).toFixed(1)} MB`
  if (n >= 1024) return `${(n / 1024).toFixed(0)} KB`
  return `${n} B`
}

function kindOf(contentType: string): 'image' | 'video' | 'audio' | 'file' {
  if (contentType.startsWith('image/')) return 'image'
  if (contentType.startsWith('video/')) return 'video'
  if (contentType.startsWith('audio/')) return 'audio'
  return 'file'
}

function KindIcon({ contentType }: { contentType: string }) {
  const kind = kindOf(contentType)
  if (kind === 'image') return <FileImage size={16} />
  if (kind === 'video') return <Film size={16} />
  if (kind === 'audio') return <FileAudio size={16} />
  return <FileText size={16} />
}

export default function MediaPanel({ parcelId }: Props) {
  const { t } = useLocale()
  const { connected, publicKey, signMessage } = useWallet()
  const [items, setItems] = useState<EvidenceUpload[] | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [notice, setNotice] = useState<string | null>(null)
  const [active, setActive] = useState<EvidenceUpload | null>(null)
  const [uploading, setUploading] = useState(false)
  const fileInputRef = useRef<HTMLInputElement>(null)
  const tokenRef = useRef(0)

  const refresh = useCallback(async () => {
    const token = ++tokenRef.current
    try {
      const rows = await listEvidence(parcelId ? { parcelId } : {})
      if (token === tokenRef.current) {
        setItems(rows)
        setError(null)
      }
    } catch (err) {
      if (token === tokenRef.current) {
        setItems([])
        setError(err instanceof Error ? err.message : String(err))
      }
    }
  }, [parcelId])

  /* eslint-disable react-hooks/set-state-in-effect -- initial gallery load; all setStates run after await */
  useEffect(() => {
    refresh()
  }, [refresh])
  /* eslint-enable react-hooks/set-state-in-effect */

  const onPickFile = useCallback(() => fileInputRef.current?.click(), [])

  const onFileChosen = useCallback(
    async (file: File) => {
      if (!publicKey || !signMessage) return
      setUploading(true)
      setError(null)
      setNotice(null)
      try {
        await uploadEvidence(file, {
          parcelId: parcelId ?? undefined,
          wallet: publicKey.toBase58(),
          signMessage,
        })
        setNotice(t('mediaUploaded', { name: file.name }))
        await refresh()
      } catch (err) {
        setError(err instanceof Error ? err.message : String(err))
      } finally {
        setUploading(false)
      }
    },
    [publicKey, signMessage, parcelId, refresh, t],
  )

  return (
    <div className="p-3 text-sm space-y-3">
      <div className="flex items-center justify-between">
        <h3 className="font-semibold">{t('mediaGallery')}</h3>
        {parcelId && (
          <span className="text-[11px] text-muted">{t('mediaParcelFilter', { id: parcelId.slice(0, 8) })}</span>
        )}
      </div>

      <div className="flex gap-2">
        <button
          className="btn btn-secondary flex-1 justify-center"
          onClick={onPickFile}
          disabled={!connected || !signMessage || uploading}
          title={connected ? t('mediaUploadHint') : t('mediaNoWallet')}
        >
          <Upload size={14} />
          {uploading ? '…' : t('mediaUpload')}
        </button>
      </div>
      {!connected && (
        <p className="text-[11px] text-muted">{t('mediaNoWallet')}</p>
      )}
      {notice && <p className="text-[11px] text-emerald-700">{notice}</p>}
      {error && <p className="text-[11px] text-red-600">{error}</p>}

      <input
        ref={fileInputRef}
        type="file"
        accept="image/*,video/*,audio/*,.pdf,.las,.laz"
        className="hidden"
        onChange={(e) => {
          const file = e.target.files?.[0]
          if (file) onFileChosen(file)
          e.target.value = ''
        }}
      />

      {items === null ? (
        <p className="text-[12px] text-muted">…</p>
      ) : items.length === 0 ? (
        <p className="text-[12px] text-muted">{t('mediaEmpty')}</p>
      ) : (
        <div className="grid grid-cols-3 gap-1.5">
          {items.map((item) => {
            const kind = kindOf(item.content_type)
            const url = evidenceFileUrl(item.id)
            return (
              <button
                key={item.id}
                className="media-thumb"
                onClick={() => setActive(item)}
                title={`${item.filename} · ${fmtBytes(item.size_bytes)}`}
              >
                {kind === 'image' ? (
                  <img src={url} alt={item.filename} loading="lazy" />
                ) : kind === 'video' ? (
                  <video src={url} preload="metadata" muted />
                ) : (
                  <span className="media-thumb-file">
                    <KindIcon contentType={item.content_type} />
                    <span className="text-[10px] truncate max-w-full">{item.filename}</span>
                  </span>
                )}
              </button>
            )
          })}
        </div>
      )}

      {active && (
        <div className="media-lightbox" onClick={() => setActive(null)}>
          <div className="media-lightbox-body" onClick={(e) => e.stopPropagation()}>
            <button
              className="btn btn-ghost p-1 absolute top-2 right-2"
              onClick={() => setActive(null)}
              title={t('mediaClose')}
            >
              <X size={16} />
            </button>
            {kindOf(active.content_type) === 'image' && (
              <img src={evidenceFileUrl(active.id)} alt={active.filename} />
            )}
            {kindOf(active.content_type) === 'video' && (
              <video src={evidenceFileUrl(active.id)} controls autoPlay />
            )}
            {kindOf(active.content_type) === 'audio' && (
              <audio src={evidenceFileUrl(active.id)} controls autoPlay />
            )}
            {kindOf(active.content_type) === 'file' && (
              <a
                className="btn btn-secondary"
                href={evidenceFileUrl(active.id)}
                target="_blank"
                rel="noreferrer"
              >
                {active.filename}
              </a>
            )}
            <div className="text-[11px] text-muted mt-2 space-y-0.5">
              <div>{active.filename} · {fmtBytes(active.size_bytes)}</div>
              <div className="font-mono break-all">SHA-256 {active.content_hash.slice(0, 16)}…</div>
              {active.uploaded_by && (
                <div className="font-mono break-all">{active.uploaded_by}</div>
              )}
              {active.parcel_id && (
                <div>{t('mediaParcelFilter', { id: active.parcel_id.slice(0, 8) })}</div>
              )}
            </div>
          </div>
        </div>
      )}
    </div>
  )
}
