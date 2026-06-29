<script lang="ts">
  import { onMount } from 'svelte';
  
  interface Props {
    boutId: number;
    boutIndex: number;
    onclose: () => void;
  }

  let { boutId, boutIndex, onclose }: Props = $props();

  interface HistoryEntry {
    id: number;
    bout_id: number;
    user: {
      id: string;
      display_name: string;
      avatar_url: string;
      color: string | null;
    };
    action: string;
    details: string | null;
    created_at: string;
  }

  let history = $state<HistoryEntry[]>([]);
  let loading = $state(true);
  let error = $state('');

  onMount(async () => {
    try {
      const token = localStorage.getItem('ef_token');
      const headers: Record<string, string> = {};
      if (token) {
        headers['Authorization'] = `Bearer ${token}`;
      }
      const res = await fetch(`/api/bouts/${boutId}/history`, { headers });
      if (!res.ok) {
        throw new Error(`Ошибка загрузки: ${res.status}`);
      }
      history = await res.json();
    } catch (e) {
      error = e instanceof Error ? e.message : 'Неизвестная ошибка';
    } finally {
      loading = false;
    }
  });

  // Portal action to render modal on body level
  function portal(node: HTMLElement) {
    const target = document.fullscreenElement || document.body;
    target.appendChild(node);
    return {
      destroy() {
        if (node.parentNode) node.parentNode.removeChild(node);
      }
    };
  }

  function formatDateTime(isoString: string): string {
    try {
      const d = new Date(isoString);
      return d.toLocaleString('ru-RU', {
        day: '2-digit',
        month: '2-digit',
        year: 'numeric',
        hour: '2-digit',
        minute: '2-digit',
        second: '2-digit'
      });
    } catch {
      return isoString;
    }
  }

  let backdropMousedown = false;

  function handleBackdropMousedown(e: MouseEvent) {
    backdropMousedown = (e.target as HTMLElement).classList.contains('modal-backdrop');
  }

  function handleBackdropClick(e: MouseEvent) {
    if (backdropMousedown && (e.target as HTMLElement).classList.contains('modal-backdrop')) {
      onclose();
    }
    backdropMousedown = false;
  }
</script>

<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<div
  class="modal-backdrop"
  use:portal
  role="presentation"
  onmousedown={handleBackdropMousedown}
  onclick={handleBackdropClick}
>
  <div class="modal" role="dialog" aria-modal="true" aria-label="История изменений схода">
    <div class="modal-header">
      <h2>История изменений схода №{boutIndex}</h2>
      <button class="close-btn" onclick={onclose} aria-label="Закрыть">
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none">
          <path d="M18 6L6 18M6 6l12 12" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
        </svg>
      </button>
    </div>

    <div class="modal-body">
      {#if loading}
        <div class="loading-container">
          <span class="spinner" aria-hidden="true"></span>
          <span>Загрузка истории…</span>
        </div>
      {:else if error}
        <div class="error-container">
          <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
            <circle cx="12" cy="12" r="10" />
            <line x1="12" y1="8" x2="12" y2="12" />
            <line x1="12" y1="16" x2="12.01" y2="16" />
          </svg>
          <span>{error}</span>
        </div>
      {:else if history.length === 0}
        <div class="empty-container">
          <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
            <rect x="3" y="4" width="18" height="18" rx="2" ry="2"/>
            <line x1="16" y1="2" x2="16" y2="6"/>
            <line x1="8" y1="2" x2="8" y2="6"/>
            <line x1="3" y1="10" x2="21" y2="10"/>
          </svg>
          <span>История изменений пуста</span>
        </div>
      {:else}
        <div class="history-list">
          {#each history as entry (entry.id)}
            <div class="history-item">
              <div class="user-avatar" style:background={entry.user.color || '#6fa0e0'} style:border-color={entry.user.color || '#6fa0e0'}>
                <span class="avatar-fallback">{entry.user.display_name.charAt(0).toUpperCase()}</span>
                {#if entry.user.avatar_url}
                  <img src={entry.user.avatar_url} alt="" onerror={(e) => { (e.target as HTMLImageElement).style.display = 'none'; }} />
                {/if}
              </div>

              <div class="history-content">
                <div class="history-meta">
                  <span class="user-name">{entry.user.display_name}</span>
                  {#if entry.action === 'create'}
                    <span class="action-badge action-badge--create">Создал сход</span>
                  {:else}
                    <span class="action-badge action-badge--update">Изменил сход</span>
                  {/if}
                </div>
                <div class="history-time">{formatDateTime(entry.created_at)}</div>
                
                {#if entry.details}
                  <div class="history-details">
                    {#each entry.details.split('; ') as detail}
                      <div class="detail-line">• {detail}</div>
                    {/each}
                  </div>
                {/if}
              </div>
            </div>
          {/each}
        </div>
      {/if}
    </div>
  </div>
</div>

<style>
  .modal-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(15, 23, 42, 0.75);
    backdrop-filter: blur(4px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 9999;
    padding: 16px;
  }

  .modal {
    background: var(--surface-solid);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-lg);
    width: 100%;
    max-width: 500px;
    max-height: 80vh;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 16px 20px;
    border-bottom: 1px solid var(--border-color);
  }

  .modal-header h2 {
    font-size: 1.1rem;
    font-weight: 600;
    color: var(--text-primary);
    margin: 0;
  }

  .close-btn {
    background: transparent;
    border: 1px solid var(--border-color);
    color: var(--text-secondary);
    cursor: pointer;
    padding: 6px;
    border-radius: 50%;
    display: flex;
    transition: var(--transition);
  }

  .close-btn:hover {
    color: var(--text-primary);
    background: var(--surface-hover);
  }

  .modal-body {
    padding: 20px;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .history-list {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .history-item {
    display: flex;
    gap: 12px;
    align-items: flex-start;
    padding-bottom: 16px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.03);
  }

  .history-item:last-child {
    border-bottom: none;
    padding-bottom: 0;
  }

  .user-avatar {
    width: 32px;
    height: 32px;
    border-radius: 50%;
    flex-shrink: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    border: 1px solid rgba(255, 255, 255, 0.1);
    font-size: 0.85rem;
    font-weight: 600;
    color: #fff;
    overflow: hidden;
    position: relative;
  }

  .avatar-fallback {
    position: absolute;
    z-index: 1;
  }

  .user-avatar img {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    object-fit: cover;
    z-index: 2;
  }

  .history-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .history-meta {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
  }

  .user-name {
    font-weight: 500;
    font-size: 0.85rem;
    color: var(--text-primary);
  }

  .action-badge {
    font-size: 0.7rem;
    padding: 2px 6px;
    border-radius: 4px;
    font-weight: 500;
  }

  .action-badge--create {
    background: rgba(16, 185, 129, 0.1);
    color: var(--accent-green);
  }

  .action-badge--update {
    background: rgba(245, 158, 11, 0.1);
    color: var(--accent-yellow);
  }

  .history-time {
    font-size: 0.72rem;
    color: var(--text-muted);
  }

  .history-details {
    font-size: 0.78rem;
    color: var(--text-secondary);
    background: rgba(0, 0, 0, 0.15);
    padding: 6px 10px;
    border-radius: 6px;
    margin-top: 4px;
    line-height: 1.4;
  }

  .detail-line {
    margin: 2px 0;
  }

  .loading-container,
  .error-container,
  .empty-container {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 40px 0;
    color: var(--text-secondary);
    font-size: 0.85rem;
    gap: 8px;
  }

  .error-container {
    color: var(--accent-red);
  }

  .spinner {
    width: 24px;
    height: 24px;
    border: 2px solid rgba(255, 255, 255, 0.1);
    border-top-color: var(--accent-yellow);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }
</style>
