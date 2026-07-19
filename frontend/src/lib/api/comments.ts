import { apiFetch } from './client';
import type { Comment, SearchResult } from './types';

export interface CreateCommentData {
  video_id: string;
  timestamp_ms: number;
  text: string;
  reply_to_id?: number | null;
}

export async function createComment(data: CreateCommentData): Promise<Comment> {
  return apiFetch<Comment>('/comments', {
    method: 'POST',
    body: JSON.stringify(data),
  });
}

export async function updateComment(id: number, text: string): Promise<Comment> {
  return apiFetch<Comment>(`/comments/${id}`, {
    method: 'PATCH',
    body: JSON.stringify({ text }),
  });
}

export async function deleteComment(id: number): Promise<{ ok: boolean }> {
  return apiFetch<{ ok: boolean }>(`/comments/${id}`, {
    method: 'DELETE',
  });
}

export async function reactComment(id: number, kind: 'like' | 'dislike'): Promise<void> {
  await apiFetch<{ ok: boolean }>(`/comments/${id}/react`, {
    method: 'POST',
    body: JSON.stringify({ kind }),
  });
}

export async function deleteReact(id: number): Promise<void> {
  await apiFetch<{ ok: boolean }>(`/comments/${id}/react`, {
    method: 'DELETE',
  });
}

export async function searchComments(q: string): Promise<SearchResult[]> {
  return apiFetch<SearchResult[]>(`/comments/search?q=${encodeURIComponent(q)}`);
}

export interface CreateSharedCommentData {
  videoId: string;
  token: string;
  nickname: string;
  text: string;
  timestamp_ms: number;
  reply_to_id?: number | null;
  bout_id?: number | null;
}

export async function createSharedComment(data: CreateSharedCommentData): Promise<Comment> {
  const url = `/shared/videos/${data.videoId}/comments?token=${encodeURIComponent(data.token)}`;
  return apiFetch<Comment>(url, {
    method: 'POST',
    body: JSON.stringify({
      nickname: data.nickname,
      text: data.text,
      reply_to_id: data.reply_to_id,
      timestamp_ms: data.timestamp_ms,
      bout_id: data.bout_id,
    }),
  });
}
