import { writable } from 'svelte/store';

export interface ToastItem {
  id: string;
  message: string;
  type: 'info' | 'success' | 'error';
  duration: number;
}

export const toasts = writable<ToastItem[]>([]);

export function showToast(
  message: string,
  type: 'info' | 'success' | 'error' = 'info',
  duration: number = 3500
): string {
  const id = Math.random().toString(36).substring(2, 9);
  const item: ToastItem = { id, message, type, duration };

  toasts.update((current) => [...current, item]);

  if (duration > 0) {
    setTimeout(() => {
      removeToast(id);
    }, duration);
  }

  return id;
}

export function removeToast(id: string): void {
  toasts.update((current) => current.filter((t) => t.id !== id));
}
