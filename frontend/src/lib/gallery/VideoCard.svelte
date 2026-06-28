<script lang="ts">
  import type { Video } from '../api/types';
  import { resolveColor } from '../api/types';
  import { regeneratePreview } from '../api/videos';
  import { currentUser } from '../../stores';

  interface Props {
    video: Video;
    watchers?: any[];
    onopen?: (id: string) => void;
  }

  let { video, watchers = [], onopen }: Props = $props();

  let imgError = $state(false);
  let isRegenerating = $state(false);
  let menuOpen = $state(false);
  let menuPos = $state({ x: 0, y: 0 });
  let previewVersion = $state(0);

  function handleClick() {
    onopen?.(video.id);
  }

  function handleAuxClick(e: MouseEvent) {
    if (e.button === 1) {
      e.preventDefault();
      window.open(`#/player/${video.id}`, '_blank');
    }
  }

  function handleImgError() {
    imgError = true;
  }

  let previewSrc = $derived(previewVersion ? `${video.preview_url}?v=${previewVersion}` : video.preview_url);

  function handleContextMenu(e: MouseEvent) {
    e.preventDefault();
    e.stopPropagation(); // Prevent immediate closing from window contextmenu handler
    
    const menuWidth = 190;
    const menuHeight = 90;
    let x = e.clientX;
    let y = e.clientY;
    
    if (x + menuWidth > window.innerWidth) {
      x = window.innerWidth - menuWidth - 10;
    }
    if (y + menuHeight > window.innerHeight) {
      y = window.innerHeight - menuHeight - 10;
    }
    
    menuPos = { x, y };
    menuOpen = true;
  }

  function closeMenu() {
    menuOpen = false;
  }

  function handleWindowClick() {
    if (menuOpen) closeMenu();
  }

  function handleWindowContextMenu() {
    if (menuOpen) closeMenu();
  }

  function handleWindowKeyDown(e: KeyboardEvent) {
    if (menuOpen && e.key === 'Escape') closeMenu();
  }

  function handleDownload() {
    const token = localStorage.getItem('ef_token') || '';
    const downloadUrl = `/api/videos/${video.id}/download?token=${encodeURIComponent(token)}`;
    const a = document.createElement('a');
    a.href = downloadUrl;
    a.click();
  }

  function pollPreview() {
    const url = video.preview_url;
    let attempts = 0;
    const interval = setInterval(async () => {
      attempts++;
      if (attempts > 20) {
        clearInterval(interval);
        isRegenerating = false;
        alert("Превышено время ожидания генерации превью");
        return;
      }
      try {
        const res = await fetch(url);
        if (res.status === 200) {
          clearInterval(interval);
          previewVersion = Date.now();
          imgError = false;
          isRegenerating = false;
        }
      } catch (e) {
        console.error("Error polling preview:", e);
      }
    }, 1500);
  }
</script>

<svelte:window 
  onclick={handleWindowClick} 
  oncontextmenu={handleWindowContextMenu} 
  onkeydown={handleWindowKeyDown} 
/>

<button
  class="card"
  onclick={handleClick}
  onauxclick={handleAuxClick}
  oncontextmenu={handleContextMenu}
>
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="preview" oncontextmenu={handleContextMenu}>
    {#if watchers.length > 0}
      <div class="watchers-column">
        {#each watchers as watcher (watcher.id)}
          <div class="watcher-avatar" style="--user-color: {watcher.color}" title={watcher.display_name}>
            <svg class="avatar-icon" width="16" height="16" viewBox="0 0 24 24" fill="none">
              <circle cx="12" cy="8" r="4" stroke="currentColor" stroke-width="1.5"/>
              <path d="M4 20c0-4 3.6-7 8-7s8 3 8 7" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
            </svg>
            {#if watcher.avatar_url}
              <img src={watcher.avatar_url} alt="" onerror={(e) => { (e.target as HTMLImageElement).style.display = 'none'; }} />
            {/if}
          </div>
        {/each}
      </div>
    {/if}
    {#if isRegenerating}
      <div class="spinner-container">
        <div class="spinner"></div>
        <span class="spinner-text">Обновление...</span>
      </div>
    {:else if !imgError}
      <img src={previewSrc} alt="" loading="lazy" onerror={handleImgError} oncontextmenu={handleContextMenu} />
    {/if}
  </div>

  <div class="info">
    {#if video.is_tagged}
      <div class="fighters">
        <div class="fighter">
          <div class="dot" style:background={resolveColor(video.fighter_a?.id ?? '', video.fighter_a?.color ?? null)}></div>
          <span class="name">{video.fighter_a?.display_name}</span>
        </div>

        <div class="score">{video.total_score_a ?? 0} : {video.total_score_b ?? 0}</div>

        <div class="fighter right">
          <span class="name">{video.fighter_b?.display_name}</span>
          <div class="dot" style:background={resolveColor(video.fighter_b?.id ?? '', video.fighter_b?.color ?? null)}></div>
        </div>
      </div>
    {:else}
      <div class="untagged">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" aria-hidden="true">
          <circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="1.5" />
          <path d="M12 8v4m0 4h.01" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
        </svg>
        <span>Не размечено</span>
      </div>
    {/if}
  </div>
</button>

{#if menuOpen}
  <div 
    class="context-menu" 
    style="left: {menuPos.x}px; top: {menuPos.y}px;"
  >
    <button 
      class="menu-item" 
      onclick={(e) => { e.stopPropagation(); closeMenu(); handleDownload(); }}
    >
      <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
        <polyline points="7 10 12 15 17 10" />
        <line x1="12" y1="15" x2="12" y2="3" />
      </svg>
      <span>Скачать видео</span>
    </button>
    {#if $currentUser?.is_admin}
      <button 
        class="menu-item" 
        onclick={async (e) => { 
          e.stopPropagation(); 
          closeMenu(); 
          isRegenerating = true; 
          try { 
            await regeneratePreview(video.id); 
            pollPreview(); 
          } catch (err) { 
            alert(err instanceof Error ? err.message : 'Ошибка при обновлении превью'); 
            isRegenerating = false; 
          } 
        }}
        disabled={isRegenerating}
      >
        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M21.5 2v6h-6M21.34 15.57a10 10 0 1 1-.57-8.38l5.67-5.67" />
        </svg>
        <span>Обновить превью</span>
      </button>
    {/if}
  </div>
{/if}

<style>
  .card {
    background: var(--surface);
    backdrop-filter: var(--glass-blur);
    -webkit-backdrop-filter: var(--glass-blur);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-md);
    box-shadow: var(--shadow-sm);
    overflow: hidden;
    cursor: pointer;
    text-align: left;
    padding: 0;
    width: 100%;
    display: flex;
    flex-direction: column;
    transition: var(--transition);
  }

  .card:hover {
    box-shadow: var(--shadow-lg);
    transform: translateY(-4px);
    border-color: var(--accent-yellow);
  }

  .preview {
    width: 100%;
    aspect-ratio: 16 / 9;
    background: var(--surface-hover);
    overflow: hidden;
    flex-shrink: 0;
    position: relative;
  }

  .preview img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    display: block;
  }

  .info {
    padding: 12px 16px;
    display: flex;
    flex-direction: column;
  }

  .fighters {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .fighter {
    display: flex;
    align-items: center;
    gap: 5px;
    flex: 1;
    min-width: 0;
  }

  .fighter.right {
    justify-content: flex-end;
  }

  .dot {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .name {
    font-size: 0.85rem;
    font-weight: 500;
    color: var(--text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .score {
    font-size: 0.95rem;
    font-weight: 700;
    color: var(--accent-yellow);
    white-space: nowrap;
    flex-shrink: 0;
  }

  .untagged {
    display: flex;
    align-items: center;
    gap: 6px;
    color: var(--text-secondary);
    font-size: 0.85rem;
  }

  .context-menu {
    position: fixed;
    background: rgba(30, 41, 59, 0.95);
    backdrop-filter: blur(12px);
    -webkit-backdrop-filter: blur(12px);
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: var(--radius-sm);
    box-shadow: 0 10px 25px -5px rgba(0, 0, 0, 0.4), 0 8px 10px -6px rgba(0, 0, 0, 0.4);
    padding: 6px;
    z-index: 10000;
    display: flex;
    flex-direction: column;
    min-width: 180px;
    gap: 2px;
  }

  .menu-item {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 8px 12px;
    background: transparent;
    border: none;
    color: var(--text-primary);
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
    color: var(--accent-yellow);
  }

  .menu-item:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .spinner-container {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    width: 100%;
    height: 100%;
    gap: 8px;
    background: rgba(15, 23, 42, 0.6);
  }

  .spinner {
    width: 24px;
    height: 24px;
    border: 2px solid rgba(255, 255, 255, 0.1);
    border-top-color: var(--accent-yellow);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  .spinner-text {
    font-size: 0.75rem;
    color: var(--text-secondary);
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  .watchers-column {
    position: absolute;
    top: 10px;
    right: 10px;
    display: flex;
    flex-direction: column;
    gap: 6px;
    z-index: 10;
  }

  .watcher-avatar {
    width: 36px;
    height: 36px;
    border-radius: 50%;
    background: var(--user-color, #4a6280);
    border: 1.5px solid rgba(255, 255, 255, 0.2);
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.3);
    overflow: hidden;
    display: flex;
    align-items: center;
    justify-content: center;
    color: #fff;
    flex-shrink: 0;
    position: relative;
  }

  .watcher-avatar .avatar-icon {
    position: absolute;
  }

  .watcher-avatar img {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    object-fit: cover;
  }
</style>
