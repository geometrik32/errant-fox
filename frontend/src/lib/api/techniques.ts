import { apiFetch } from './client';
import type { Technique } from './types';

export async function getTechniques(): Promise<Technique[]> {
  return apiFetch<Technique[]>('/techniques');
}

export async function createTechnique(name: string): Promise<Technique> {
  return apiFetch<Technique>('/admin/techniques', {
    method: 'POST',
    body: JSON.stringify({ name }),
  });
}

export async function renameTechnique(id: number, name: string): Promise<Technique> {
  return apiFetch<Technique>(`/admin/techniques/${id}`, {
    method: 'PATCH',
    body: JSON.stringify({ name }),
  });
}

export async function deleteTechnique(id: number): Promise<{ ok: boolean }> {
  return apiFetch<{ ok: boolean }>(`/admin/techniques/${id}`, {
    method: 'DELETE',
  });
}
