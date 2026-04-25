import { apiFetch } from './client';
import type { Bout } from './types';

export interface CreateBoutData {
  video_id: string;
  time_start_ms: number;
  time_end_ms: number;
}

export interface UpdateBoutData {
  time_start_ms?: number;
  time_end_ms?: number;
  score_a?: number;
  score_b?: number;
  technique_a_id?: number | null;
  hit_zone_a?: string | null;
  result_a?: 'hit' | 'miss' | 'blocked' | null;
  technique_b_id?: number | null;
  hit_zone_b?: string | null;
  result_b?: 'hit' | 'miss' | 'blocked' | null;
}

export async function createBout(data: CreateBoutData): Promise<Bout> {
  return apiFetch<Bout>('/bouts', {
    method: 'POST',
    body: JSON.stringify(data),
  });
}

export async function updateBout(id: number, data: UpdateBoutData): Promise<Bout> {
  return apiFetch<Bout>(`/bouts/${id}`, {
    method: 'PATCH',
    body: JSON.stringify(data),
  });
}

export async function deleteBout(id: number): Promise<{ ok: boolean }> {
  return apiFetch<{ ok: boolean }>(`/bouts/${id}`, {
    method: 'DELETE',
  });
}
