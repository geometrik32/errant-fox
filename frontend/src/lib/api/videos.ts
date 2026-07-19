import { apiFetch } from './client';
import type { Video, VideoFull } from './types';

export interface VideoFilters {
  fighter_id?: string;
  date_from?: string;
  date_to?: string;
}

export interface PatchVideoData {
  fighter_a_id?: string | null;
  fighter_b_id?: string | null;
}

export async function getVideos(filters?: VideoFilters): Promise<Video[]> {
  const params = new URLSearchParams();
  if (filters?.fighter_id) params.set('fighter_id', filters.fighter_id);
  if (filters?.date_from) params.set('date_from', filters.date_from);
  if (filters?.date_to) params.set('date_to', filters.date_to);
  const query = params.toString();
  return apiFetch<Video[]>(`/videos${query ? `?${query}` : ''}`);
}

export async function getVideo(id: string): Promise<VideoFull> {
  return apiFetch<VideoFull>(`/videos/${id}`);
}

export async function patchVideo(id: string, data: PatchVideoData): Promise<VideoFull> {
  return apiFetch<VideoFull>(`/videos/${id}`, {
    method: 'PATCH',
    body: JSON.stringify(data),
  });
}

export async function getStreamUrl(id: string): Promise<{ stream_url: string }> {
  return apiFetch<{ stream_url: string }>(`/videos/${id}/stream`);
}

export async function regeneratePreview(id: string): Promise<{ status: string }> {
  return apiFetch<{ status: string }>(`/videos/${id}/previews/regenerate`, {
    method: 'POST',
  });
}

export async function checkStaleVideos(): Promise<Video[]> {
  return apiFetch<Video[]>('/admin/videos/sync-check');
}

export async function cleanStaleVideos(ids: string[]): Promise<{ status: string; deleted_count: number }> {
  return apiFetch<{ status: string; deleted_count: number }>('/admin/videos/sync-clean', {
    method: 'POST',
    body: JSON.stringify({ ids }),
  });
}

export async function getSharedVideo(id: string, token: string): Promise<VideoFull> {
  return apiFetch<VideoFull>(`/shared/videos/${id}?token=${encodeURIComponent(token)}`);
}

export async function createShareToken(id: string, boutId?: number | null): Promise<{ token: string }> {
  return apiFetch<{ token: string }>(`/videos/${id}/share`, {
    method: 'POST',
    body: JSON.stringify({ bout_id: boutId ?? null }),
  });
}

export async function aiLabelVideo(id: string): Promise<import('./types').Bout[]> {
  return apiFetch<import('./types').Bout[]>(`/videos/${id}/ai-label`, {
    method: 'POST',
  });
}
