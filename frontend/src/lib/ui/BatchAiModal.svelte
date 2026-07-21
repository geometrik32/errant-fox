<script lang="ts">
  import { onMount } from 'svelte';
  import { getVideos, cancelAiLabelVideo, batchAiLabelVideos } from '../api/videos';
  import type { Video } from '../api/types';

  interface Props {
    mode?: 'new' | 'relabel';
    onclose: () => void;
  }
  let { mode = 'new', onclose }: Props = $props();

  let activeTab = $state<'new' | 'relabel'>('new');
  $effect(() => {
    activeTab = mode;
  });
  let loading = $state(true);
  let error = $state<string | null>(null);
  let unanalyzedVideos = $state<Video[]>([]);
  let missingTranscriptVideos = $state<Video[]>([]);
  let analyzingVideos = $state<Video[]>([]);
  let queuedVideos = $state<Video[]>([]);
  let starting = $state(false);
  let progressMessage = $state<string | null>(null);

  let activeOrQueuedCount = $derived(analyzingVideos.length + queuedVideos.length);
  let isCancelMode = $derived(activeOrQueuedCount > 0);
  let activeTargetList = $derived(activeTab === 'new' ? unanalyzedVideos : missingTranscriptVideos);

  onMount(async () => {
    try {
      const all = await getVideos();
      // Currently analyzing videos
      analyzingVideos = all.filter(v => v.is_analyzing);
      // Queued videos
      queuedVideos = all.filter(v => v.is_queued);
      
      // Unanalyzed = not human labeled (has_human_bouts or scores) AND not currently ai_labeled AND not currently analyzing AND not queued
      unanalyzedVideos = all.filter(v => {
        const isHuman = v.has_human_bouts || (v.total_score_a ?? 0) > 0 || (v.total_score_b ?? 0) > 0;
        return !isHuman && !v.is_ai_labeled && !v.is_analyzing && !v.is_queued;
      });

      // Missing transcript = AI labeled (or tagged) AND no transcript on server AND not human labeled AND not analyzing AND not queued
      missingTranscriptVideos = all.filter(v => {
        const isHuman = v.has_human_bouts || (v.total_score_a ?? 0) > 0 || (v.total_score_b ?? 0) > 0;
        return !isHuman && !v.has_transcript && (v.is_ai_labeled || v.is_tagged) && !v.is_analyzing && !v.is_queued;
      });
    } catch (e) {
      error = e instanceof Error ? e.message : 'Не удалось загрузить список видео';
    } finally {
      loading = false;
    }
  });

  async function handleBatchLabel() {
    if (starting || activeTargetList.length === 0) return;
    starting = true;
    error = null;
    try {
      progressMessage = `Запуск фоновой очереди на сервере для ${activeTargetList.length} видео...`;
      const ids = activeTargetList.map(v => v.id);
      await batchAiLabelVideos(ids);
      progressMessage = `Серверная очередь успешно запущена для ${activeTargetList.length} видео! Вы можете закрыть это окно.`;
      setTimeout(() => {
        onclose();
      }, 1500);
    } catch (e) {
      error = e instanceof Error ? e.message : 'Ошибка запуска разметки';
      starting = false;
    }
  }

  async function handleBatchCancel() {
    const cancelTargets = [...analyzingVideos, ...queuedVideos];
    if (starting || cancelTargets.length === 0) return;
    starting = true;
    error = null;
    let count = 0;
    try {
      for (const video of cancelTargets) {
        count++;
        progressMessage = `Отмена очереди для видео ${count} из ${cancelTargets.length}...`;
        await cancelAiLabelVideo(video.id).catch(() => {});
      }
      progressMessage = `Очередь ИИ успешно отменена для всех ${cancelTargets.length} видео.`;
      setTimeout(() => {
        onclose();
      }, 1500);
    } catch (e) {
      error = e instanceof Error ? e.message : 'Ошибка отмены очереди';
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
      <h2>{isCancelMode ? 'Отменить ИИ-разметку' : (activeTab === 'relabel' ? 'ИИ-переразметка (без расшифровки)' : 'Разметить при помощи ИИ')}</h2>
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
          Анализируется: <strong>{analyzingVideos.length} шт.</strong>, в очереди: <strong>{queuedVideos.length} шт.</strong><br/>
          Вы действительно хотите отменить ИИ-разметку для всей очереди?
        </p>

        <div class="list-container">
          <ul class="stale-list">
            {#each analyzingVideos as video}
              <li>
                <span class="video-date" style="color: #a78bfa;">[Анализ ИИ...]</span>
                <span class="video-path" title={video.seafile_path}>{video.seafile_path}</span>
              </li>
            {/each}
            {#each queuedVideos as video}
              <li>
                <span class="video-date" style="color: var(--accent-yellow);">[В очереди ИИ]</span>
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
            <span>Отменить всю очередь ({activeOrQueuedCount})</span>
          </button>
        </div>
      {:else}
        <!-- Tab selector -->
        <div class="tabs-bar">
          <button
            class="tab-btn"
            class:active={activeTab === 'new'}
            onclick={() => activeTab = 'new'}
            disabled={starting}
          >
            Новые неразмеченные ({unanalyzedVideos.length})
          </button>
          <button
            class="tab-btn"
            class:active={activeTab === 'relabel'}
            onclick={() => activeTab = 'relabel'}
            disabled={starting}
          >
            Без расшифровки ({missingTranscriptVideos.length})
          </button>
        </div>

        {#if activeTargetList.length === 0}
          <p class="msg-info">
            {activeTab === 'new' 
              ? 'Неразмеченных видео не найдено! Все видео уже размечены людьми или ИИ.' 
              : 'Видео без расшифровок не найдено! У всех ИИ-видео уже есть файлы расшифровки.'}
          </p>
          <div class="actions">
            <button class="btn btn-primary" onclick={onclose}>Отлично</button>
          </div>
        {:else}
          <p class="warning-text">
            {#if activeTab === 'new'}
              Найдено <strong>{unanalyzedVideos.length} шт.</strong> неразмеченных видео.
              Запустить автоматическую ИИ-разметку сходов для всех этих видео?
            {:else}
              Найдено <strong>{missingTranscriptVideos.length} шт.</strong> видео с отсутствующей расшифровкой.
              Запустить повторную ИИ-разметку для сбора расшифровок?
            {/if}
          </p>

          <div class="list-container">
            <ul class="stale-list">
              {#each activeTargetList as video}
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
              <span>{activeTab === 'new' ? 'Запустить анализ' : 'Запустить переразметку'} ({activeTargetList.length})</span>
            </button>
          </div>
        {/if}
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

  .tabs-bar {
    display: flex;
    gap: 8px;
    margin-bottom: 16px;
    border-bottom: 1px solid var(--border-color);
    padding-bottom: 10px;
  }

  .tab-btn {
    background: transparent;
    border: 1px solid var(--border-color);
    color: var(--text-muted);
    padding: 6px 12px;
    border-radius: var(--radius-md);
    font-size: 0.82rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .tab-btn:hover:not(:disabled) {
    color: var(--text-main);
    background: var(--surface-hover);
  }

  .tab-btn.active {
    background: #8b5cf6;
    border-color: #8b5cf6;
    color: #fff;
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
