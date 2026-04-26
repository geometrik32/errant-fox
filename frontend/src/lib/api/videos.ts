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
