<script lang="ts">
  import type { Video } from '../api/types';
  import VideoCard from './VideoCard.svelte';
  import { currentUser } from '../../stores';
  import { showToast } from '../stores/toast';
  import {
    batchTranscodeVideos,
    batchOptimizeVideos,
    batchAiLabelVideos,
  } from '../api/videos';

  interface Props {
    videos: Video[];
    videoWatchers?: Record<string, any[]>;
    onopen?: (id: string) => void;
    onreload?: () => void;
  }

  let { videos, videoWatchers = {}, onopen, onreload }: Props = $props();

  let selectedIds = $state<Set<string>>(new Set());
  let lastSelectedId = $state<string | null>(null);

  let batchMenuOpen = $state(false);
  let batchMenuPos = $state({ x: 0, y: 0 });

  let selectedCount = $derived(selectedIds.size);
  let hasSelection = $derived(selectedIds.size > 0);

  interface DateGroup {
    date: string;
    label: string;
    videos: Video[];
  }

  let groups = $derived.by<DateGroup[]>(() => {
    const map = new Map<string, Video[]>();
    for (const v of videos) {
      const list = map.get(v.date) ?? [];
      list.push(v);
      map.set(v.date, list);
    }
    return [...map.entries()]
      .sort(([a], [b]) => b.localeCompare(a))
      .map(([date, list]) => ({
        date,
        label: new Date(date + 'T00:00:00').toLocaleDateString('ru-RU', {
          day: 'numeric',
          month: 'long',
          year: 'numeric',
        }),
        videos: list,
      }));
  });

  function handleToggle(videoId: string, e: MouseEvent) {
    const next = new Set(selectedIds);

    if (e.shiftKey && lastSelectedId) {
      const ids = videos.map((v) => v.id);
      const fromIdx = ids.indexOf(lastSelectedId);
      const toIdx = ids.indexOf(videoId);

      if (fromIdx !== -1 && toIdx !== -1) {
        const start = Math.min(fromIdx, toIdx);
        const end = Math.max(fromIdx, toIdx);
        for (let i = start; i <= end; i++) {
          next.add(ids[i]);
        }
        selectedIds = next;
        lastSelectedId = videoId;
        return;
      }
    }

    if (next.has(videoId)) {
      next.delete(videoId);
      if (lastSelectedId === videoId) {
        lastSelectedId = next.size > 0 ? Array.from(next)[next.size - 1] : null;
      }
    } else {
      next.add(videoId);
      lastSelectedId = videoId;
    }
    selectedIds = next;
  }

  function toggleDateGroup(groupVideos: Video[]) {
    const next = new Set(selectedIds);
    const allSelected = groupVideos.length > 0 && groupVideos.every((v) => next.has(v.id));

    if (allSelected) {
      for (const v of groupVideos) next.delete(v.id);
    } else {
      for (const v of groupVideos) next.add(v.id);
      if (groupVideos.length > 0) {
        lastSelectedId = groupVideos[groupVideos.length - 1].id;
      }
    }
    selectedIds = next;
  }

  function clearSelection() {
    selectedIds = new Set();
    lastSelectedId = null;
    batchMenuOpen = false;
  }

  function handleBatchContextMenu(e: MouseEvent) {
    e.preventDefault();
    e.stopPropagation();

    window.dispatchEvent(new CustomEvent('ef-close-context-menus'));

    const menuWidth = 240;
    const menuHeight = 220;
    let x = e.clientX;
    let y = e.clientY;

    if (x + menuWidth > window.innerWidth) {
      x = window.innerWidth - menuWidth - 10;
    }
    if (y + menuHeight > window.innerHeight) {
      y = window.innerHeight - menuHeight - 10;
    }

    batchMenuPos = { x, y };
    batchMenuOpen = true;
  }

  function closeBatchMenu() {
    batchMenuOpen = false;
  }

  function handleWindowClick() {
    if (batchMenuOpen) closeBatchMenu();
  }

  function handleWindowKeyDown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      if (batchMenuOpen) {
        closeBatchMenu();
      } else if (hasSelection) {
        clearSelection();
      }
    }
  }

  async function handleBatchTranscode() {
    const ids = Array.from(selectedIds);
    if (ids.length === 0) return;
    closeBatchMenu();
    try {
      const res = await batchTranscodeVideos(ids);
      if (res.queued > 0 && res.already_ready > 0) {
        showToast(`H.264: ${res.queued} видео в очереди, ${res.already_ready} уже готово`, 'success');
      } else if (res.queued > 0) {
        showToast(`Конвертация в H.264 запущена для ${res.queued} видео`, 'success');
      } else {
        showToast(`Все выбранные видео (${res.already_ready}) уже сконвертированы в H.264`, 'info');
      }
      clearSelection();
    } catch (err) {
      showToast(err instanceof Error ? err.message : 'Ошибка запуска конвертации', 'error');
    }
  }

  async function handleBatchOptimize() {
    const ids = Array.from(selectedIds);
    if (ids.length === 0) return;
    closeBatchMenu();
    try {
      await batchOptimizeVideos(ids);
      showToast(`Оптимизация запущена для ${ids.length} видео`, 'success');
      clearSelection();
    } catch (err) {
      showToast(err instanceof Error ? err.message : 'Ошибка запуска оптимизации', 'error');
    }
  }

  async function handleBatchAiLabel() {
    const ids = Array.from(selectedIds);
    if (ids.length === 0) return;
    closeBatchMenu();
    try {
      await batchAiLabelVideos(ids);
      showToast(`ИИ-разметка запущена для ${ids.length} видео`, 'success');
      clearSelection();
    } catch (err) {
      showToast(err instanceof Error ? err.message : 'Ошибка запуска ИИ-разметки', 'error');
    }
  }
</script>

<svelte:window onclick={handleWindowClick} onkeydown={handleWindowKeyDown} />

{#if videos.length === 0}
  <p class="empty">Видео не найдено</p>
{:else}
  {#each groups as group (group.date)}
    {@const allGroupSelected = group.videos.length > 0 && group.videos.every((v) => selectedIds.has(v.id))}
    {@const someGroupSelected = group.videos.some((v) => selectedIds.has(v.id))}
    <div class="date-group">
      <div class="date-group-header">
        <h3 class="date-label">{group.label}</h3>
        <button
          type="button"
          class="group-select-btn"
          class:active={allGroupSelected || someGroupSelected}
          onclick={() => toggleDateGroup(group.videos)}
          title={allGroupSelected ? 'Снять выделение за день' : 'Выбрать все видео за день'}
        >
          <span class="group-select-checkbox" class:checked={allGroupSelected} class:indeterminate={!allGroupSelected && someGroupSelected}>
            {#if allGroupSelected}
              ✓
            {:else if someGroupSelected}
              –
            {/if}
          </span>
          <span>{allGroupSelected ? 'Выбрано всё' : 'Выбрать день'} ({group.videos.length})</span>
        </button>
      </div>

      <div class="grid">
        {#each group.videos as video (video.id)}
          <VideoCard
            {video}
            watchers={videoWatchers[video.id] ?? []}
            selected={selectedIds.has(video.id)}
            {hasSelection}
            ontoggle={handleToggle}
            onbatchmenu={handleBatchContextMenu}
            {onopen}
            {onreload}
          />
        {/each}
      </div>
    </div>
  {/each}
{/if}

{#if batchMenuOpen}
  <div
    class="context-menu batch-context-menu"
    style="left: {batchMenuPos.x}px; top: {batchMenuPos.y}px;"
    onclick={(e) => e.stopPropagation()}
    role="menu"
    tabindex="-1"
  >
    <div class="menu-header">
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="var(--accent-yellow, #eab308)" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
        <polyline points="20 6 9 17 4 12" />
      </svg>
      <span>Выбрано: <strong>{selectedCount}</strong> видео</span>
    </div>
    <div class="menu-divider"></div>

    {#if $currentUser?.is_admin}
      <button class="menu-item" onclick={handleBatchTranscode} title="Сконвертировать выбранные видео в H.264">
        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <rect x="2" y="2" width="20" height="20" rx="2.18" ry="2.18"/>
          <line x1="7" y1="2" x2="7" y2="22"/>
          <line x1="17" y1="2" x2="17" y2="22"/>
          <line x1="2" y1="12" x2="22" y2="12"/>
          <line x1="2" y1="7" x2="7" y2="7"/>
          <line x1="2" y1="17" x2="7" y2="17"/>
          <line x1="17" y1="17" x2="22" y2="17"/>
          <line x1="17" y1="7" x2="22" y2="7"/>
        </svg>
        <span>Конвертировать в H.264 ({selectedCount})</span>
      </button>

      <button class="menu-item menu-item-optimize" onclick={handleBatchOptimize} title="Сжать видео (VFR) с сохранением 100 fps">
        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2" />
        </svg>
        <span>Оптимизировать видео ({selectedCount})</span>
      </button>

      <button class="menu-item menu-item-ai" onclick={handleBatchAiLabel} title="Разметить сходы с помощью ИИ">
        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M12 2a10 10 0 1 0 10 10" />
          <path d="M12 6v6l4 2" />
          <circle cx="19" cy="5" r="3" fill="currentColor" stroke="none" />
        </svg>
        <span>Разметить сходы (ИИ) ({selectedCount})</span>
      </button>

      <div class="menu-divider"></div>
    {/if}

    <button class="menu-item" onclick={clearSelection}>
      <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <line x1="18" y1="6" x2="6" y2="18"/>
        <line x1="6" y1="6" x2="18" y2="18"/>
      </svg>
      <span>Снять выделение</span>
    </button>
  </div>
{/if}

{#if hasSelection}
  <div class="batch-bar" role="toolbar" aria-label="Пакетные действия">
    <div class="batch-bar-count">
      <span class="batch-bar-badge">{selectedCount}</span>
      <span class="batch-bar-label">выбрано</span>
    </div>

    {#if $currentUser?.is_admin}
      <button class="batch-bar-btn" onclick={handleBatchTranscode} title="Сконвертировать выбранные в H.264">
        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <rect x="2" y="2" width="20" height="20" rx="2.18" ry="2.18"/>
          <line x1="7" y1="2" x2="7" y2="22"/>
          <line x1="17" y1="2" x2="17" y2="22"/>
          <line x1="2" y1="12" x2="22" y2="12"/>
        </svg>
        <span>Конвертировать H.264</span>
      </button>

      <button class="batch-bar-btn" onclick={handleBatchOptimize} title="Оптимизировать видео (VFR-сжатие)">
        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2" />
        </svg>
        <span>Оптимизировать</span>
      </button>

      <button class="batch-bar-btn" onclick={handleBatchAiLabel} title="Разметить сходы нейросетью">
        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M12 2a10 10 0 1 0 10 10" />
          <path d="M12 6v6l4 2" />
          <circle cx="19" cy="5" r="3" fill="currentColor" stroke="none" />
        </svg>
        <span>Разметить (ИИ)</span>
      </button>
    {/if}

    <button class="batch-bar-close" onclick={clearSelection} title="Снять выделение (Esc)" aria-label="Снять выделение">
      ✕
    </button>
  </div>
{/if}

<style>
  .empty {
    color: #4a6280;
    text-align: center;
    padding: 64px 0;
    font-size: 0.9rem;
  }

  .date-group {
    margin-bottom: 32px;
  }

  .date-group-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin: 0 0 12px;
    padding-bottom: 6px;
    border-bottom: 1px solid #1f3a57;
  }

  .date-label {
    font-size: 0.72rem;
    font-weight: 600;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: #4a6280;
    margin: 0;
  }

  .group-select-btn {
    background: transparent;
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 6px;
    color: var(--text-secondary, #94a3b8);
    font-size: 0.76rem;
    padding: 3px 10px;
    display: inline-flex;
    align-items: center;
    gap: 6px;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .group-select-btn:hover {
    background: rgba(255, 255, 255, 0.06);
    color: var(--text-primary, #fff);
    border-color: rgba(255, 255, 255, 0.2);
  }

  .group-select-btn.active {
    background: rgba(37, 99, 235, 0.15);
    border-color: rgba(59, 130, 246, 0.4);
    color: #60a5fa;
  }

  .group-select-checkbox {
    width: 14px;
    height: 14px;
    border-radius: 3px;
    border: 1px solid rgba(255, 255, 255, 0.3);
    display: inline-flex;
    align-items: center;
    justify-content: center;
    font-size: 0.7rem;
    line-height: 1;
  }

  .group-select-checkbox.checked {
    background: #2563eb;
    border-color: #3b82f6;
    color: #fff;
  }

  .group-select-checkbox.indeterminate {
    background: #1e3a8a;
    border-color: #3b82f6;
    color: #93c5fd;
  }

  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(380px, 1fr));
    gap: 20px;
  }

  /* ── Batch Context Menu ────────────────────────────── */
  .batch-context-menu {
    position: fixed;
    background: rgba(30, 41, 59, 0.96);
    border: 1px solid rgba(255, 255, 255, 0.12);
    border-radius: var(--radius-sm, 6px);
    box-shadow: 0 12px 30px -5px rgba(0, 0, 0, 0.5), 0 8px 12px -6px rgba(0, 0, 0, 0.5);
    padding: 6px;
    z-index: 10000;
    display: flex;
    flex-direction: column;
    min-width: 220px;
    gap: 2px;
    backdrop-filter: blur(12px);
    -webkit-backdrop-filter: blur(12px);
  }

  .menu-header {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 10px;
    font-size: 0.8rem;
    color: var(--text-secondary, #94a3b8);
  }

  .menu-header strong {
    color: var(--text-primary, #fff);
  }

  .menu-divider {
    height: 1px;
    background: rgba(255, 255, 255, 0.08);
    margin: 4px 0;
  }

  .menu-item {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 8px 12px;
    background: transparent;
    border: none;
    color: var(--text-primary, #fff);
    font-family: inherit;
    font-size: 0.85rem;
    text-align: left;
    cursor: pointer;
    border-radius: 4px;
    transition: background 0.15s ease, color 0.15s ease;
    width: 100%;
  }

  .menu-item:hover:not(:disabled) {
    background: rgba(255, 255, 255, 0.08);
    color: var(--accent-yellow, #eab308);
  }

  .menu-item-ai:not(:disabled) {
    background: linear-gradient(90deg, rgba(124, 58, 237, 0.1), transparent);
    border-left: 2px solid #7c3aed;
  }

  .menu-item-ai:hover:not(:disabled) {
    background: rgba(124, 58, 237, 0.22);
    color: #c4b5fd;
  }

  .menu-item-optimize:not(:disabled) {
    background: linear-gradient(90deg, rgba(16, 185, 129, 0.12), transparent);
    border-left: 2px solid #10b981;
  }

  .menu-item-optimize:hover:not(:disabled) {
    background: rgba(16, 185, 129, 0.22);
    color: #34d399;
  }

  /* ── Floating Action Bar ────────────────────────────── */
  .batch-bar {
    position: fixed;
    bottom: 28px;
    left: 50%;
    transform: translateX(-50%);
    background: rgba(22, 32, 49, 0.94);
    border: 1px solid rgba(255, 255, 255, 0.16);
    box-shadow: 0 12px 35px rgba(0, 0, 0, 0.55), 0 0 0 1px rgba(0, 0, 0, 0.2);
    border-radius: 12px;
    padding: 8px 12px;
    display: flex;
    align-items: center;
    gap: 10px;
    z-index: 5000;
    backdrop-filter: blur(14px);
    -webkit-backdrop-filter: blur(14px);
    animation: batchBarSlideUp 0.22s ease-out;
  }

  .batch-bar-count {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 0 10px 0 6px;
    border-right: 1px solid rgba(255, 255, 255, 0.12);
  }

  .batch-bar-badge {
    background: #2563eb;
    color: #fff;
    font-weight: 700;
    font-size: 0.8rem;
    padding: 2px 8px;
    border-radius: 10px;
  }

  .batch-bar-label {
    font-size: 0.82rem;
    color: var(--text-secondary, #94a3b8);
  }

  .batch-bar-btn {
    display: flex;
    align-items: center;
    gap: 7px;
    padding: 7px 13px;
    background: rgba(255, 255, 255, 0.08);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 8px;
    color: var(--text-primary, #fff);
    font-family: inherit;
    font-size: 0.82rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .batch-bar-btn:hover {
    background: rgba(255, 255, 255, 0.16);
    border-color: rgba(255, 255, 255, 0.25);
    color: var(--accent-yellow, #eab308);
    transform: translateY(-1px);
  }

  .batch-bar-close {
    background: transparent;
    border: none;
    color: var(--text-secondary, #94a3b8);
    font-size: 0.95rem;
    cursor: pointer;
    padding: 6px 8px;
    border-radius: 6px;
    transition: all 0.15s ease;
  }

  .batch-bar-close:hover {
    background: rgba(255, 255, 255, 0.1);
    color: #fff;
  }

  @keyframes batchBarSlideUp {
    from {
      opacity: 0;
      transform: translate(-50%, 16px) scale(0.96);
    }
    to {
      opacity: 1;
      transform: translate(-50%, 0) scale(1);
    }
  }

  @media (max-width: 768px) {
    .grid {
      grid-template-columns: 1fr;
    }
    .batch-bar {
      bottom: 16px;
      left: 12px;
      right: 12px;
      transform: none;
      flex-wrap: wrap;
      justify-content: center;
    }
    @keyframes batchBarSlideUp {
      from {
        opacity: 0;
        transform: translateY(16px);
      }
      to {
        opacity: 1;
        transform: translateY(0);
      }
    }
  }
</style>
