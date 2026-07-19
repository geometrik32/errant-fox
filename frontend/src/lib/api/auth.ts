import { apiFetch } from './client';
import type { User } from './types';

export interface LoginResponse {
  token: string;
  user: User;
}

export async function login(username: string, password: string): Promise<LoginResponse> {
  return apiFetch<LoginResponse>('/auth/login', {
    method: 'POST',
    body: JSON.stringify({ username, password }),
  });
}

export async function getVkConfig(): Promise<{ client_id: string | null }> {
  return apiFetch<{ client_id: string | null }>('/auth/vk/config');
}

export async function loginWithVk(code: string, redirectUri: string, codeVerifier?: string, deviceId?: string): Promise<LoginResponse> {
  return apiFetch<LoginResponse>('/auth/vk', {
    method: 'POST',
    body: JSON.stringify({ code, redirect_uri: redirectUri, code_verifier: codeVerifier, device_id: deviceId }),
  });
}

export async function getMe(): Promise<User> {
  return apiFetch<User>('/users/me');
}

export async function patchMe(data: { username?: string; display_name?: string; password?: string; color?: string; vk_id?: string; role?: string }): Promise<User> {
  return apiFetch<User>('/users/me', {
    method: 'PATCH',
    body: JSON.stringify(data),
  });
}

export async function uploadMyAvatar(file: File): Promise<{ avatar_url: string }> {
  const token = localStorage.getItem('ef_token');
  const form = new FormData();
  form.append('avatar', file);
  const res = await fetch('/api/users/me/avatar', {
    method: 'POST',
    headers: token ? { Authorization: `Bearer ${token}` } : {},
    body: form,
  });
  if (!res.ok) throw new Error(`Ошибка ${res.status}: ${await res.text().catch(() => '')}`);
  return res.json();
}

export async function unlinkVk(): Promise<User> {
  return apiFetch<User>('/auth/vk/unlink', {
    method: 'POST',
  });
}
