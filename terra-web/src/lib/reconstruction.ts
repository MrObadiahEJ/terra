// 3D reconstruction client (RFC-013 Ph.C scaffolding): capture sessions,
// queued reconstruction jobs, and the resulting models. Model payloads live
// off-chain; `uri` points at a 3D Tiles tileset (or mesh) the globe can load.

import { API_BASE } from './constants'

export interface CaptureSession {
  id: string
  parcel_id: string | null
  title: string | null
  captured_by: string | null
  captured_at: string | null
  media_count: number
  evidence_refs: string[]
  created_at: string
}

export interface ReconstructionJob {
  id: string
  capture_session_id: string
  status: string
  progress: number
  error: string | null
  artifact_uri: string | null
  created_at: string
  updated_at: string
}

export interface ThreeDModel {
  id: string
  parcel_id: string | null
  reconstruction_job_id: string | null
  uri: string
  format: string | null
  content_hash: string | null
  created_at: string
}

export async function listModels(parcelId?: string): Promise<ThreeDModel[]> {
  const qs = parcelId ? `?parcel_id=${encodeURIComponent(parcelId)}` : ''
  const res = await fetch(`${API_BASE}/reconstruction/models${qs}`)
  if (!res.ok) throw new Error(`models list failed: ${res.status}`)
  return (await res.json()) as ThreeDModel[]
}

export async function listCaptureSessions(parcelId?: string): Promise<CaptureSession[]> {
  const qs = parcelId ? `?parcel_id=${encodeURIComponent(parcelId)}` : ''
  const res = await fetch(`${API_BASE}/reconstruction/sessions${qs}`)
  if (!res.ok) throw new Error(`sessions list failed: ${res.status}`)
  return (await res.json()) as CaptureSession[]
}

export async function getJob(id: string): Promise<ReconstructionJob> {
  const res = await fetch(`${API_BASE}/reconstruction/jobs/${id}`)
  if (!res.ok) throw new Error(`job fetch failed: ${res.status}`)
  return (await res.json()) as ReconstructionJob
}
