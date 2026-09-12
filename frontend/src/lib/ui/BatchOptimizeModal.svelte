<script lang="ts">
  import { onMount } from 'svelte';
  import { getOptimizationCandidates, batchOptimizeVideos } from '../api/videos';
  import type { OptimizationCandidate } from '../api/types';

  interface Props {
    onclose: () => void;
  }
  let { onclose }: Props = $props();

  let selectedPeriod = $state<number | undefined>(undefined); // undefined = all, 7 = >7 days, etc.
  let loading = $state(true);
  let error = $state<string | null>(null);
  let candidates = $state<OptimizationCandidate[]>([]);
  let starting = $state(false);
  let progressMessage = $state<string | null>(null);

  async function loadCandidates(days?: number) {
    loading = true;
    error = null;
    try {
      candidates = await getOptimizationCandidates(days);
    } catch (e) {
      error = e instanceof Error ? e.message : 'Не удалось загрузить список видео для оптимизации';
    } finally {
      loading = false;
    }
  }

  onMount(() => {
    loadCandidates(selectedPeriod);
  });

  function handlePeriodChange(days?: number) {
    selectedPeriod = days;
    loadCandidates(days);
  }

  // Оценка экономии: в среднем ~300 МБ на видео (с 360 МБ до 55 МБ)
  let estimatedSavingsMb = $derived(candidates.length * 300);
  let estimatedSavingsGb = $derived((estimatedSavingsMb / 1024).toFixed(1));

  async function handleBatchOptimize() {
    if (starting || candidates.length === 0) return;
    starting = true;
    error = null;
    try {
      progressMessage = `Запуск фоновой очереди оптимизации для ${candidates.length} видео...`;
      const ids = candidates.map(v => v.id);
      await batchOptimizeVideos(ids);
      progressMessage = `Фоновая оптимизация успешно запущена для ${candidates.length} видео! Обработка выполняется на сервере.`;
      setTimeout(() => {
        onclose();
      }, 1800);
    } catch (e) {
      error = e instanceof Error ? e.message : 'Ошибка запуска оптимизации';
      starting = false;
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
  onclick={handleBackdropClick}
>
  <div class="modal" role="dialog" aria-modal="true" aria-label="Оптимизация видео (VFR)">
    <div class="modal-header">
      <div class="title-with-icon">
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="var(--accent-yellow)" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2" />
        </svg>
        <h2>Адаптивная VFR-оптимизация видео</h2>
      </div>
      <button class="close-btn" onclick={onclose} aria-label="Закрыть" disabled={starting}>
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none">
          <path d="M18 6L6 18M6 6l12 12" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
        </svg>
      </button>
    </div>

    <div class="modal-body">
      <!-- Period filter tabs -->
      <div class="tabs-bar">
        <button
          class="tab-btn"
          class:active={selectedPeriod === undefined}
          onclick={() => handlePeriodChange(undefined)}
          disabled={loading || starting}
        >
          Все размеченные
        </button>
        <button
          class="tab-btn"
          class:active={selectedPeriod === 7}
          onclick={() => handlePeriodChange(7)}
          disabled={loading || starting}
        >
          Старше 7 дней
        </button>
        <button
          class="tab-btn"
          class:active={selectedPeriod === 14}
          onclick={() => handlePeriodChange(14)}
          disabled={loading || starting}
        >
          Старше 14 дней
        </button>
        <button
          class="tab-btn"
          class:active={selectedPeriod === 30}
          onclick={() => handlePeriodChange(30)}
          disabled={loading || starting}
        >
          Старше 30 дней
        </button>
      </div>

      {#if loading}
        <div class="state-msg">
          <div class="spinner"></div>
          <span>Поиск полностью размеченных видео…</span>
        </div>
      {:else if error}
        <p class="msg error">{error}</p>
        <div class="actions">
          <button class="btn btn-outline" onclick={onclose}>Закрыть</button>
        </div>
      {:else if progressMessage}
        <p class="msg success">{progressMessage}</p>
      {:else if candidates.length === 0}
        <div class="empty-state">
          <svg width="40" height="40" viewBox="0 0 24 24" fill="none" stroke="var(--text-muted)" stroke-width="1.5">
            <circle cx="12" cy="12" r="10" />
            <path d="M9 12l2 2 4-4" stroke-linecap="round" stroke-linejoin="round" />
          </svg>
          <p class="msg-info">
            Подходящих видео для оптимизации не найдено.<br />
            <span style="font-size: 0.85rem; color: var(--text-muted);">
              Видео должно быть полностью проверено человеком (без оставшихся меток ИИ) и не должно быть уже оптимизировано.
            </span>
          </p>
        </div>
        <div class="actions">
          <button class="btn btn-primary" onclick={onclose}>Понятно</button>
        </div>
      {:else}
        <div class="summary-card">
          <div class="summary-item">
            <span class="summary-label">Готово к оптимизации:</span>
            <span class="summary-val">{candidates.length} шт.</span>
          </div>
          <div class="summary-item">
            <span class="summary-label">Ожидаемая экономия диска:</span>
            <span class="summary-val highlight">~{estimatedSavingsGb} ГБ ({estimatedSavingsMb} МБ)</span>
          </div>
        </div>

        <p class="desc-text">
          Сходы будут сохранены в исходном качестве и частоте кадров (100 fps). Паузы между сходами будут прорежены до 25 fps с сохранением четкого звука обсуждений.
        </p>

        <div class="list-container">
          <ul class="candidate-list">
            {#each candidates as video}
              <li>
                <div class="cand-info">
                  <span class="video-date">[{video.date}]</span>
                  <span class="video-fighters">
                    {video.fighter_a_name || 'Боец 1'} vs {video.fighter_b_name || 'Боец 2'}
                  </span>
                  <span class="video-bouts">({video.bouts_count} сходов)</span>
                </div>
                <span class="video-path" title={video.seafile_path}>{video.seafile_path.split('/').pop()}</span>
              </li>
            {/each}
          </ul>
        </div>

        <div class="actions">
          <button class="btn btn-outline" onclick={onclose} disabled={starting}>Отмена</button>
          <button class="btn btn-optimize" onclick={handleBatchOptimize} disabled={starting}>
            <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2" />
            </svg>
            <span>Оптимизировать {candidates.length} видео</span>
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
    background: rgba(10, 16, 26, 0.8);
    backdrop-filter: blur(4px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
    padding: 16px;
  }

  .modal {
    background: var(--surface);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-lg);
    width: 100%;
    max-width: 580px;
    box-shadow: 0 16px 40px rgba(0,0,0,0.6);
    overflow: hidden;
  }

  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 16px 20px;
    border-bottom: 1px solid var(--border-color);
  }

  .title-with-icon {
    display: flex;
    align-items: center;
    gap: 10px;
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
    padding-bottom: 12px;
    overflow-x: auto;
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
    white-space: nowrap;
    transition: all 0.2s ease;
  }

  .tab-btn:hover:not(:disabled) {
    color: var(--text-main);
    background: var(--surface-hover);
  }

  .tab-btn.active {
    background: var(--accent-yellow);
    border-color: var(--accent-yellow);
    color: #000;
    font-weight: 600;
  }

  .state-msg {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 12px;
    padding: 36px 0;
    color: var(--text-muted);
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

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 24px 0;
    text-align: center;
  }

  .msg-info {
    color: var(--text-main);
    line-height: 1.5;
    margin-top: 12px;
  }

  .summary-card {
    display: flex;
    justify-content: space-between;
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-md);
    padding: 12px 16px;
    margin-bottom: 12px;
  }

  .summary-item {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .summary-label {
    font-size: 0.78rem;
    color: var(--text-muted);
  }

  .summary-val {
    font-size: 1rem;
    font-weight: 600;
    color: var(--text-main);
  }

  .summary-val.highlight {
    color: #22c55e;
  }

  .desc-text {
    font-size: 0.82rem;
    color: var(--text-muted);
    line-height: 1.4;
    margin-bottom: 14px;
  }

  .list-container {
    max-height: 200px;
    overflow-y: auto;
    background: rgba(0, 0, 0, 0.25);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-md);
    padding: 8px 12px;
    margin-bottom: 20px;
  }

  .candidate-list {
    list-style: none;
    padding: 0;
    margin: 0;
  }

  .candidate-list li {
    font-size: 0.82rem;
    padding: 6px 0;
    display: flex;
    justify-content: space-between;
    align-items: center;
    border-bottom: 1px solid rgba(255, 255, 255, 0.05);
    gap: 8px;
  }

  .candidate-list li:last-child {
    border-bottom: none;
  }

  .cand-info {
    display: flex;
    align-items: center;
    gap: 8px;
    overflow: hidden;
  }

  .video-date {
    color: var(--accent-yellow);
    font-family: monospace;
    font-size: 0.78rem;
    flex-shrink: 0;
  }

  .video-fighters {
    font-weight: 500;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .video-bouts {
    color: var(--text-muted);
    font-size: 0.75rem;
    flex-shrink: 0;
  }

  .video-path {
    color: var(--text-muted);
    font-family: monospace;
    font-size: 0.75rem;
    flex-shrink: 0;
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
    font-weight: 500;
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
    transition: all 0.15s;
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

  .btn-optimize {
    background: #10b981;
    color: #fff;
    font-weight: 600;
  }

  .btn-optimize:hover:not(:disabled) {
    background: #059669;
  }

  .btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
