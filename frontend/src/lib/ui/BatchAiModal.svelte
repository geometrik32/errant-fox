<script lang="ts">
  import { onMount } from 'svelte';
  import { getVideos, aiLabelVideo, cancelAiLabelVideo, batchAiLabelVideos } from '../api/videos';
  import type { Video } from '../api/types';

  interface Props {
    onclose: () => void;
  }
  let { onclose }: Props = $props();

  let loading = $state(true);
  let error = $state<string | null>(null);
  let unanalyzedVideos = $state<Video[]>([]);
  let analyzingVideos = $state<Video[]>([]);
  let starting = $state(false);
  let progressMessage = $state<string | null>(null);

  let isCancelMode = $derived(analyzingVideos.length > 0);

  onMount(async () => {
    try {
      const all = await getVideos();
      // Currently analyzing videos
      analyzingVideos = all.filter(v => v.is_analyzing);
      
      // Unanalyzed = not human labeled (total_score <= 0 or not tagged) AND not currently ai_labeled AND not currently analyzing
      unanalyzedVideos = all.filter(v => {
        const isHuman = (v.total_score_a ?? 0) > 0 || (v.total_score_b ?? 0) > 0;
        return !isHuman && !v.is_ai_labeled && !v.is_analyzing;
      });
    } catch (e) {
      error = e instanceof Error ? e.message : 'Не удалось загрузить список видео';
    } finally {
      loading = false;
    }
  });

  async function handleBatchLabel() {
    if (starting || unanalyzedVideos.length === 0) return;
    starting = true;
    error = null;
    try {
      progressMessage = `Запуск фонового анализа на сервере для ${unanalyzedVideos.length} видео...`;
      const ids = unanalyzedVideos.map(v => v.id);
      await batchAiLabelVideos(ids);
      progressMessage = `Серверная очередь успешно запущена для ${unanalyzedVideos.length} видео! Вы можете закрыть это окно.`;
      setTimeout(() => {
        onclose();
      }, 1500);
    } catch (e) {
      error = e instanceof Error ? e.message : 'Ошибка запуска разметки';
      starting = false;
    }
  }

  async function handleBatchCancel() {
    if (starting || analyzingVideos.length === 0) return;
    starting = true;
    error = null;
    let count = 0;
    try {
      for (const video of analyzingVideos) {
        count++;
        progressMessage = `Отмена анализа для видео ${count} из ${analyzingVideos.length}...`;
        await cancelAiLabelVideo(video.id).catch(() => {});
      }
      progressMessage = `Анализ успешно отменен для всех ${analyzingVideos.length} видео.`;
      setTimeout(() => {
        onclose();
      }, 1800);
    } catch (e) {
      error = e instanceof Error ? e.message : 'Ошибка отмены разметки';
      starting = false;
    }
  }

  function handleBackdropMousedown(e: MouseEvent) {
    if (e.target === e.currentTarget) {
      // Prevent closing
    }
  }

  function handleBackdropClick(e: MouseEvent) {
    if (e.target === e.currentTarget && !starting) {
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
  <div class="modal" role="dialog" aria-modal="true" aria-label="ИИ-разметка видео">
    <div class="modal-header">
      <h2>{isCancelMode ? 'Отменить ИИ-разметку' : 'Разметить при помощи ИИ'}</h2>
      <button class="close-btn" onclick={onclose} aria-label="Закрыть" disabled={starting}>
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none">
          <path d="M18 6L6 18M6 6l12 12" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
        </svg>
      </button>
    </div>

    <div class="modal-body">
      {#if loading}
        <div class="state-msg">Проверка статуса видео…</div>
      {:else if error}
        <p class="msg error">{error}</p>
        <div class="actions">
          <button class="btn btn-outline" onclick={onclose}>Закрыть</button>
        </div>
      {:else if progressMessage}
        <p class="msg success">{progressMessage}</p>
      {:else if isCancelMode}
        <p class="warning-text">
          В данный момент обрабатывается <strong>{analyzingVideos.length} шт.</strong> видео.
          Вы действительно хотите отменить ИИ-разметку для всех текущих процессов?
        </p>

        <div class="list-container">
          <ul class="stale-list">
            {#each analyzingVideos as video}
              <li>
                <span class="video-date">[{video.date}]</span>
                <span class="video-path" title={video.seafile_path}>{video.seafile_path}</span>
              </li>
            {/each}
          </ul>
        </div>

        <div class="actions">
          <button class="btn btn-outline" onclick={onclose} disabled={starting}>Назад</button>
          <button class="btn btn-danger" onclick={handleBatchCancel} disabled={starting}>
            <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <circle cx="12" cy="12" r="10" />
              <line x1="15" y1="9" x2="9" y2="15" />
              <line x1="9" y1="9" x2="15" y2="15" />
            </svg>
            <span>Отменить все ({analyzingVideos.length})</span>
          </button>
        </div>
      {:else if unanalyzedVideos.length === 0}
        <p class="msg-info">Неразмеченных видео не найдено! Все видео уже размечены людьми или ИИ.</p>
        <div class="actions">
          <button class="btn btn-primary" onclick={onclose}>Отлично</button>
        </div>
      {:else}
        <p class="warning-text">
          Найдено <strong>{unanalyzedVideos.length} шт.</strong> неразмеченных видео.
          Запустить автоматическую ИИ-разметку сходов для всех этих видео?
        </p>

        <div class="list-container">
          <ul class="stale-list">
            {#each unanalyzedVideos as video}
              <li>
                <span class="video-date">[{video.date}]</span>
                <span class="video-path" title={video.seafile_path}>{video.seafile_path}</span>
              </li>
            {/each}
          </ul>
        </div>

        <div class="actions">
          <button class="btn btn-outline" onclick={onclose} disabled={starting}>Отмена</button>
          <button class="btn btn-ai" onclick={handleBatchLabel} disabled={starting}>
            <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M12 2a10 10 0 1 0 10 10" />
              <path d="M12 6v6l4 2" />
              <circle cx="19" cy="5" r="3" fill="currentColor" stroke="none" />
            </svg>
            <span>Запустить анализ ({unanalyzedVideos.length})</span>
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
    background: rgba(10, 16, 26, 0.75);
    backdrop-filter: blur(4px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .modal {
    background: var(--surface);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-lg);
    width: 100%;
    max-width: 520px;
    box-shadow: 0 16px 40px rgba(0,0,0,0.5);
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
    margin: 0;
  }

  .close-btn {
    background: none;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
    padding: 4px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 4px;
  }

  .close-btn:hover:not(:disabled) {
    color: var(--text-main);
    background: var(--surface-hover);
  }

  .modal-body {
    padding: 20px;
  }

  .state-msg, .msg-info {
    text-align: center;
    color: var(--text-muted);
    padding: 20px 0;
  }

  .msg.error {
    color: #ef4444;
  }

  .msg.success {
    color: #10b981;
    text-align: center;
    font-weight: 500;
    padding: 16px 0;
  }

  .warning-text {
    font-size: 0.9rem;
    line-height: 1.5;
    margin-top: 0;
    margin-bottom: 14px;
  }

  .list-container {
    max-height: 200px;
    overflow-y: auto;
    background: rgba(0, 0, 0, 0.2);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-md);
    padding: 8px 12px;
    margin-bottom: 20px;
  }

  .stale-list {
    list-style: none;
    padding: 0;
    margin: 0;
  }

  .stale-list li {
    font-family: monospace;
    font-size: 0.8rem;
    padding: 4px 0;
    display: flex;
    gap: 8px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.05);
  }

  .stale-list li:last-child {
    border-bottom: none;
  }

  .video-date {
    color: var(--accent-yellow);
    flex-shrink: 0;
  }

  .video-path {
    color: var(--text-muted);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 12px;
  }

  .btn {
    padding: 8px 16px;
    border-radius: var(--radius-md);
    font-size: 0.88rem;
    font-weight: 500;
    cursor: pointer;
    border: none;
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .btn-outline {
    background: transparent;
    border: 1px solid var(--border-color);
    color: var(--text-main);
  }

  .btn-outline:hover:not(:disabled) {
    background: var(--surface-hover);
  }

  .btn-primary {
    background: var(--accent-yellow);
    color: #000;
  }

  .btn-ai {
    background: #8b5cf6;
    color: #fff;
  }

  .btn-ai:hover:not(:disabled) {
    background: #7c3aed;
  }

  .btn-danger {
    background: #ef4444;
    color: #fff;
  }

  .btn-danger:hover:not(:disabled) {
    background: #dc2626;
  }

  .btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
