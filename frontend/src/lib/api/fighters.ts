import { apiFetch } from './client';
import type { Fighter, FighterBout, User } from './types';

export async function getFighters(): Promise<Fighter[]> {
  return apiFetch<Fighter[]>('/fighters');
}

export async function getFighterBouts(id: string): Promise<FighterBout[]> {
  return apiFetch<FighterBout[]>(`/fighters/${id}/bouts`);
}

export async function createUser(data: {
  username: string;
  display_name: string;
  password: string;
  is_admin: boolean;
  color?: string;
}): Promise<User> {
  return apiFetch<User>('/admin/users', {
    method: 'POST',
    body: JSON.stringify(data),
  });
}

export async function patchUser(id: string, data: {
  display_name?: string;
  password?: string;
  color?: string;
  is_admin?: boolean;
}): Promise<User> {
  return apiFetch<User>(`/admin/users/${id}`, {
    method: 'PATCH',
    body: JSON.stringify(data),
  });
}

export async function deleteUser(id: string): Promise<void> {
  await apiFetch<unknown>(`/admin/users/${id}`, { method: 'DELETE' });
}

export async function uploadUserAvatar(userId: string, file: File): Promise<{ avatar_url: string }> {
  const token = localStorage.getItem('ef_token');
  const form = new FormData();
  form.append('avatar', file);
  const res = await fetch(`/api/admin/users/${userId}/avatar`, {
    method: 'POST',
    headers: token ? { Authorization: `Bearer ${token}` } : {},
    body: form,
  });
  if (!res.ok) throw new Error(`Ошибка ${res.status}: ${await res.text().catch(() => '')}`);
  return res.json();
}
