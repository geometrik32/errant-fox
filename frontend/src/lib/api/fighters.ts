import { apiFetch } from './client';
import type { Fighter, FighterBout } from './types';

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
}): Promise<Fighter> {
  return apiFetch<Fighter>('/admin/users', {
    method: 'POST',
    body: JSON.stringify(data),
  });
}
