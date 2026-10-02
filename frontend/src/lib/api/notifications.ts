import { apiFetch } from './client';
import type { NotificationsResponse } from './types';

export interface ListNotificationsParams {
  limit?: number;
  offset?: number;
  is_read?: boolean;
  event_type?: string;
}

export async function fetchNotifications(params?: ListNotificationsParams): Promise<NotificationsResponse> {
  const query = new URLSearchParams();
  if (params?.limit !== undefined) query.set('limit', String(params.limit));
  if (params?.offset !== undefined) query.set('offset', String(params.offset));
  if (params?.is_read !== undefined) query.set('is_read', String(params.is_read));
  if (params?.event_type !== undefined) query.set('event_type', params.event_type);

  const qs = query.toString();
  return apiFetch<NotificationsResponse>(`/notifications${qs ? `?${qs}` : ''}`);
}

export async function fetchUnreadCount(): Promise<number> {
  const res = await apiFetch<{ unread_count: number }>('/notifications/unread-count');
  return res.unread_count;
}

export async function markReadByVideo(videoId: string): Promise<void> {
  await apiFetch<void>('/notifications/read-by-video', {
    method: 'POST',
    body: JSON.stringify({ video_id: videoId }),
  });
}

export async function markReadBout(boutId: number): Promise<void> {
  await apiFetch<void>('/notifications/read-bout', {
    method: 'POST',
    body: JSON.stringify({ bout_id: boutId }),
  });
}

export async function markReadComment(commentId: number): Promise<void> {
  await apiFetch<void>('/notifications/read-comment', {
    method: 'POST',
    body: JSON.stringify({ comment_id: commentId }),
  });
}

export async function markReadAll(): Promise<void> {
  await apiFetch<void>('/notifications/read-all', {
    method: 'POST',
  });
}
