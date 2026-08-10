import { Capacitor } from '@capacitor/core';

export function getApiBaseUrl(): string {
  if (typeof window === 'undefined') return '/api';

  const devOverride = localStorage.getItem('DEV_API_URL');
  if (devOverride) {
    return devOverride;
  }

  if ((window as any).__CAPACITOR_SERVER_URL__) {
    return `${(window as any).__CAPACITOR_SERVER_URL__}/api`;
  }
  
  if ((window as any).Capacitor?.isNative) {
    return 'https://errantfox.aat-terra.ru/api';
  }

  if (Capacitor.isNativePlatform && Capacitor.isNativePlatform()) {
    return 'https://errantfox.aat-terra.ru/api';
  }

  // Capacitor Android uses https://localhost by default with androidScheme: 'https'
  if (window.location.origin === 'https://localhost' || window.location.origin === 'http://localhost') {
    if (window.location.port === '5173') {
      return '/api'; // Local dev server
    }
    return 'https://errantfox.aat-terra.ru/api'; // Native capacitor app
  }

  return '/api';
}

export function getOriginUrl(): string {
  const apiBase = getApiBaseUrl();
  if (apiBase.startsWith('http')) {
    return new URL(apiBase).origin;
  }
  return typeof window !== 'undefined' ? window.location.origin : '';
}

export function resolveMediaUrl(path: string | null | undefined): string {
  if (!path) return '';
  if (path.startsWith('http://') || path.startsWith('https://') || path.startsWith('data:')) {
    return path;
  }
  const origin = getOriginUrl();
  const cleanPath = path.startsWith('/') ? path : `/${path}`;
  return `${origin}${cleanPath}`;
}

export async function apiFetch<T>(path: string, options: RequestInit = {}): Promise<T> {
  const token = localStorage.getItem('ef_token');

  const headers: Record<string, string> = {
    'Content-Type': 'application/json',
    ...(options.headers as Record<string, string> ?? {}),
  };

  if (token) {
    headers['Authorization'] = `Bearer ${token}`;
  }

  const baseUrl = getApiBaseUrl();
  let response: Response;
  try {
    response = await fetch(`${baseUrl}${path}`, { ...options, headers });
  } catch {
    throw new Error('Не удалось подключиться к серверу. Проверьте соединение.');
  }

  if (response.status === 401) {
    if (typeof window !== 'undefined' && !window.location.hash.startsWith('#/share/')) {
      localStorage.removeItem('ef_token');
      window.location.href = '/auth';
    }
    throw new Error('Сессия истекла. Войдите снова.');
  }

  if (!response.ok) {
    const text = await response.text().catch(() => '');
    throw new Error(`Ошибка ${response.status}: ${text || response.statusText}`);
  }

  return response.json() as Promise<T>;
}

