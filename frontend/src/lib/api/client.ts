const BASE_URL = 'http://localhost:8080/api';

export async function apiFetch<T>(path: string, options: RequestInit = {}): Promise<T> {
  const token = localStorage.getItem('ef_token');

  const headers: Record<string, string> = {
    'Content-Type': 'application/json',
    ...(options.headers as Record<string, string> ?? {}),
  };

  if (token) {
    headers['Authorization'] = `Bearer ${token}`;
  }

  let response: Response;
  try {
    response = await fetch(`${BASE_URL}${path}`, { ...options, headers });
  } catch {
    throw new Error('Не удалось подключиться к серверу. Проверьте соединение.');
  }

  if (response.status === 401) {
    localStorage.removeItem('ef_token');
    window.location.href = '/auth';
    throw new Error('Сессия истекла. Войдите снова.');
  }

  if (!response.ok) {
    const text = await response.text().catch(() => '');
    throw new Error(`Ошибка ${response.status}: ${text || response.statusText}`);
  }

  return response.json() as Promise<T>;
}
