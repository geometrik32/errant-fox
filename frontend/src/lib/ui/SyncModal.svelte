<script lang="ts">
  import { onMount } from 'svelte';
  import { checkStaleVideos, cleanStaleVideos } from '../api/videos';
  import type { Video } from '../api/types';

  interface Props {
    onclose: () => void;
  }
  let { onclose }: Props = $props();

  let loading = $state(true);
  let error = $state<string | null>(null);
  let staleVideos = $state<Video[]>([]);
  let cleaning = $state(false);
  let successMessage = $state<string | null>(null);

  onMount(async () => {
    try {
      staleVideos = await checkStaleVideos();
    } catch (e) {
      error = e instanceof Error ? e.message : 'Не удалось проверить базу данных';
    } finally {
      loading = false;
    }
  });

  async function handleClean() {
    if (cleaning || staleVideos.length === 0) return;
    cleaning = true;
    error = null;
    try {
      const ids = staleVideos.map(v => v.id);
      const res = await cleanStaleVideos(ids);
      successMessage = `Успешно удалено видео: ${res.deleted_count}`;
      setTimeout(() => {
        onclose();
      }, 1500);
    } catch (e) {
      error = e instanceof Error ? e.message : 'Не удалось очистить базу данных';
      cleaning = false;
    }
  }

  function handleBackdropMousedown(e: MouseEvent) {
    if (e.target === e.currentTarget) {
      // Prevent drag select closing
    }
  }

  function handleBackdropClick(e: MouseEvent) {
    if (e.target === e.currentTarget && !cleaning) {
      onclose();
    }
  }
</script>

<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<div
  class="modal-backdrop"
  role="presentation"
  onmousedown={handleBackdropMousedown}
  onclick={handleBackdropClick}
>
  <div class="modal" role="dialog" aria-modal="true" aria-label="Актуализация базы данных">
    <div class="modal-header">
      <h2>Актуализация базы данных</h2>
      <button class="close-btn" onclick={onclose} aria-label="Закрыть" disabled={cleaning}>
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none">
          <path d="M18 6L6 18M6 6l12 12" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
        </svg>
      </button>
    </div>

    <div class="modal-body">
      {#if loading}
        <div class="state-msg">Сверка с Seafile…</div>
      {:else if error}
        <p class="msg error">{error}</p>
        <div class="actions">
          <button class="btn btn-outline" onclick={onclose}>Закрыть</button>
        </div>
      {:else if successMessage}
        <p class="msg success">{successMessage}</p>
      {:else if staleVideos.length === 0}
        <p class="msg-info">Расхождений с Seafile не обнаружено. Все файлы видео на месте!</p>
        <div class="actions">
          <button class="btn btn-primary" onclick={onclose}>Отлично</button>
        </div>
      {:else}
        <p class="warning-text">
          В базе данных найдены записи видео ({staleVideos.length} шт.), файлы которых отсутствуют в Seafile.
          Они будут каскадно удалены из базы со всеми комментариями и сходами.
        </p>

        <div class="list-container">
          <ul class="stale-list">
            {#each staleVideos as video}
              <li>
                <span class="video-date">[{video.date}]</span>
                <span class="video-path" title={video.seafile_path}>{video.seafile_path}</span>
              </li>
            {/each}
          </ul>
        </div>

        <div class="actions">
          <button class="btn btn-outline" onclick={onclose} disabled={cleaning}>Отмена</button>
          <button class="btn btn-danger" onclick={handleClean} disabled={cleaning}>
            {cleaning ? 'Удаление…' : 'Удалить из базы данных'}
          </button>
        </div>
      {/if}
    </div>
  </div>
</div>

<style>
  .modal-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.55);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 500;
    padding: 16px;
  }

  .modal {
    background: var(--surface-solid);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-lg);
    width: 100%;
    max-width: 500px;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 24px 28px 16px;
    border-bottom: 1px solid var(--border-color);
  }

  .modal-header h2 {
    font-size: 1.2rem;
    font-weight: 700;
    color: var(--text-primary);
    margin: 0;
  }

  .close-btn {
    background: var(--surface-solid);
    border: 1px solid var(--border-color);
    box-shadow: var(--shadow-sm);
    color: var(--text-secondary);
    cursor: pointer;
    padding: 6px;
    border-radius: 50%;
    display: flex;
    transition: var(--transition);
  }

  .close-btn:hover:not(:disabled) {
    color: var(--text-primary);
    transform: scale(1.05);
  }

  .close-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .modal-body {
    padding: 24px 28px 28px;
    display: flex;
    flex-direction: column;
    gap: 20px;
    max-height: calc(100vh - 100px);
    overflow-y: auto;
  }

  .state-msg {
    text-align: center;
    color: var(--text-secondary);
    padding: 20px 0;
  }

  .msg-info {
    color: var(--text-primary);
    line-height: 1.5;
    text-align: center;
    padding: 10px 0;
  }

  .warning-text {
    color: #ef4444;
    font-size: 0.9rem;
    line-height: 1.5;
  }

  .list-container {
    border: 1px solid var(--border-color);
    border-radius: var(--radius-md);
    background: rgba(0, 0, 0, 0.2);
    max-height: 200px;
    overflow-y: auto;
    padding: 8px 12px;
  }

  .stale-list {
    list-style: none;
    padding: 0;
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: 8px;
    font-family: monospace;
    font-size: 0.85rem;
  }

  .stale-list li {
    display: flex;
    gap: 8px;
    color: var(--text-secondary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .video-date {
    color: var(--text-primary);
    flex-shrink: 0;
  }

  .video-path {
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .msg {
    padding: 12px;
    border-radius: var(--radius-md);
    font-size: 0.9rem;
    line-height: 1.4;
    text-align: center;
  }

  .msg.error {
    background: rgba(239, 68, 68, 0.1);
    color: #ef4444;
    border: 1px solid rgba(239, 68, 68, 0.2);
  }

  .msg.success {
    background: rgba(34, 197, 94, 0.1);
    color: #22c55e;
    border: 1px solid rgba(34, 197, 94, 0.2);
  }

  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 12px;
    margin-top: 8px;
  }

  .btn {
    padding: 8px 16px;
    font-size: 0.9rem;
    font-weight: 500;
    border-radius: var(--radius-md);
    cursor: pointer;
    transition: var(--transition);
    display: inline-flex;
    align-items: center;
    justify-content: center;
  }

  .btn-outline {
    background: transparent;
    border: 1px solid var(--border-color);
    color: var(--text-primary);
  }

  .btn-outline:hover:not(:disabled) {
    background: rgba(255, 255, 255, 0.05);
  }

  .btn-primary {
    background: var(--primary);
    border: 1px solid var(--primary);
    color: #fff;
  }

  .btn-primary:hover:not(:disabled) {
    opacity: 0.9;
  }

  .btn-danger {
    background: #ef4444;
    border: 1px solid #ef4444;
    color: #fff;
  }

  .btn-danger:hover:not(:disabled) {
    background: #dc2626;
    border-color: #dc2626;
  }

  .btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
