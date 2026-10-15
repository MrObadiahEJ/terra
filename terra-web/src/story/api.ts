// Stories API client (RFC-014) — /api/v1/stories + /api/v1/exports.

import { API_BASE } from '../lib/constants'
import type { StoryDefinition, StoryExport, StoryRow, StoryWithDefinition, ExportAspect } from './types'

async function request<T>(method: string, path: string, body?: unknown): Promise<T> {
  const res = await fetch(`${API_BASE}${path}`, {
    method,
    headers: body !== undefined ? { 'content-type': 'application/json' } : undefined,
    body: body !== undefined ? JSON.stringify(body) : undefined,
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
