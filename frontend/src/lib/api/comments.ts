import { apiFetch } from './client';
import type { Comment } from './types';

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
