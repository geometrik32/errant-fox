<script lang="ts">
  import type { Video } from '../api/types';
  import { resolveColor } from '../api/types';
  import { regeneratePreview, aiLabelVideo } from '../api/videos';
  import { currentUser } from '../../stores';
  import ShareModal from '../ui/ShareModal.svelte';

  interface Props {
    video: Video;
    watchers?: any[];
    onopen?: (id: string) => void;
    onreload?: () => void;
  }

  let { video, watchers = [], onopen, onreload }: Props = $props();

  let imgError = $state(false);
  let isRegenerating = $state(false);
  let isAiLabeling = $state(false);
  let menuOpen = $state(false);
  let showShare = $state(false);
  let showInfo = $state(false);
  let menuPos = $state({ x: 0, y: 0 });
  let previewVersion = $state(0);

  // Visual state derived from video data
  // 0 = untagged, 1 = fighters-only, 2 = human-labeled, 3 = ai-labeled
  let cardState = $derived((): 0 | 1 | 2 | 3 => {
    const hasFighters = !!video.fighter_a && !!video.fighter_b;
    if (!hasFighters || !video.is_tagged) return 0;
    const scoreA = video.total_score_a;
    const scoreB = video.total_score_b;
    const hasScores = scoreA !== undefined && scoreB !== undefined;
    const hasBouts = hasScores && ((scoreA ?? 0) > 0 || (scoreB ?? 0) > 0);
    if (!hasBouts) return 0;
    if (video.is_ai_labeled) return 3;
    return 2;
  });

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

  $effect(() => {
    video.is_analyzing;
    video.preview_url;
    imgError = false;
  });

  let previewSrc = $derived(previewVersion ? `${video.preview_url}?v=${previewVersion}` : video.preview_url);

  function handleContextMenu(e: MouseEvent) {
    e.preventDefault();
    e.stopPropagation(); // Prevent immediate closing from window contextmenu handler
    
    const menuWidth = 190;
    const menuHeight = 150;
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

  function handleOpenTranscript() {
    const token = localStorage.getItem('ef_token') || '';
    const transcriptUrl = `/api/videos/${video.id}/transcript?token=${encodeURIComponent(token)}`;
    window.open(transcriptUrl, '_blank');
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

<!-- AI-border wrapper -->
<div class="card-wrapper" 
     class:ai-labeled={video.is_ai_labeled && !video.is_analyzing}
     class:analyzing={video.is_analyzing || isAiLabeling}>
<button
  class="card"
  class:state-untagged={cardState() === 0}
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
    {#if isAiLabeling || video.is_analyzing}
      <div class="spinner-container">
        <div class="spinner ai-spinner"></div>
        <span class="spinner-text">Анализ ИИ...</span>
      </div>
    {:else if isRegenerating}
      <div class="spinner-container">
        <div class="spinner"></div>
        <span class="spinner-text">Обновление...</span>
      </div>
    {:else if !imgError}
      <img src={previewSrc} alt="" loading="lazy" onerror={handleImgError} oncontextmenu={handleContextMenu} />
    {/if}
    <!-- State overlay for untagged / no bouts -->
    {#if cardState() === 0}
      <div class="state-overlay" style="--overlay-opacity: 0.55;"></div>
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
</div>

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
    <button 
      class="menu-item" 
      onclick={(e) => { e.stopPropagation(); closeMenu(); showShare = true; }}
    >
      <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <circle cx="18" cy="5" r="3" />
        <circle cx="6" cy="12" r="3" />
        <circle cx="18" cy="19" r="3" />
        <line x1="8.59" y1="13.51" x2="15.42" y2="17.49" />
        <line x1="15.41" y1="6.51" x2="8.59" y2="10.49" />
      </svg>
      <span>Поделиться</span>
    </button>
    {#if $currentUser?.is_admin}
      {#if video.is_ai_labeled}
        <button 
          class="menu-item"
          onclick={(e) => {
            e.stopPropagation();
            closeMenu();
            handleOpenTranscript();
          }}
        >
          <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
            <polyline points="14 2 14 8 20 8" />
            <line x1="16" y1="13" x2="8" y2="13" />
            <line x1="16" y1="17" x2="8" y2="17" />
          </svg>
          <span>Расшифровка ИИ</span>
        </button>
      {/if}

      {@const isHumanLabeled = cardState() === 2}
      <button 
        class="menu-item menu-item-ai" 
        onclick={async (e) => { 
          if (isHumanLabeled) return;
          e.stopPropagation(); 
          closeMenu(); 
          video.is_analyzing = true;
          isAiLabeling = true; 
          try { 
            await aiLabelVideo(video.id);
          } catch (err) { 
            video.is_analyzing = false;
            alert(err instanceof Error ? err.message : 'Ошибка анализа ИИ'); 
          } finally {
            isAiLabeling = false; 
          }
        }}
        disabled={isAiLabeling || isHumanLabeled}
        title={isHumanLabeled ? 'Нельзя запускать ИИ-разметку для видео, размеченного человеком' : video.is_ai_labeled ? 'Переразметить сходы с помощью ИИ' : 'Разметить сходы (ИИ)'}
      >
        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M12 2a10 10 0 1 0 10 10" />
          <path d="M12 6v6l4 2" />
          <circle cx="19" cy="5" r="3" fill="currentColor" stroke="none" />
        </svg>
        <span>{video.is_ai_labeled ? 'Переразметить сходы (ИИ)' : 'Разметить сходы (ИИ)'}</span>
      </button>

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

      <button 
        class="menu-item" 
        onclick={(e) => { 
          e.stopPropagation(); 
          closeMenu(); 
          showInfo = true;
        }}
      >
        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <circle cx="12" cy="12" r="10" />
          <line x1="12" y1="16" x2="12" y2="12" />
          <line x1="12" y1="8" x2="12.01" y2="8" />
        </svg>
        <span>Сведения о видео</span>
      </button>
    {/if}
  </div>
{/if}

{#if showShare}
  <ShareModal videoId={video.id} onclose={() => showShare = false} />
{/if}

{#if showInfo}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_noninteractive_element_to_interactive_role -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="modal-overlay" onclick={() => showInfo = false} role="presentation">
    <div class="modal-card info-modal" onclick={(e) => e.stopPropagation()} role="dialog" aria-modal="true" aria-label="Сведения о видео" tabindex="-1">
      <div class="modal-header">
        <h3>Сведения о видео</h3>
        <button class="close-btn" onclick={() => showInfo = false} aria-label="Закрыть">✕</button>
      </div>
      <div class="modal-body">
        <table class="info-table">
          <tbody>
            <tr>
              <td><strong>Имя файла:</strong></td>
              <td>{video.seafile_path ? video.seafile_path.split('/').pop() : 'Неизвестно'}</td>
            </tr>
            <tr>
              <td><strong>Путь на сервере:</strong></td>
              <td><code>{video.seafile_path || 'Неизвестно'}</code></td>
            </tr>
            <tr>
              <td><strong>Ссылка в Seafile:</strong></td>
              <td>
                {#if video.seafile_web_url}
                  <a href={video.seafile_web_url} target="_blank" rel="noopener noreferrer" class="link-styled">
                    Открыть в Seafile
                  </a>
                {:else}
                  <span>Недоступно</span>
                {/if}
              </td>
            </tr>
            <tr>
              <td><strong>Ссылка на скачивание:</strong></td>
              <td>
                <a href="/api/videos/{video.id}/download?token={encodeURIComponent(localStorage.getItem('ef_token') || '')}" target="_blank" rel="noopener noreferrer" class="link-styled">
                  Скачать файл
                </a>
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>
  </div>
{/if}

<style>
  /* ── AI-border wrapper ─────────────────────────────── */
  .card-wrapper {
    position: relative;
    border-radius: calc(var(--radius-md) + 3px);
    padding: 0;
    transition: padding 0.2s ease, background 0.2s ease, box-shadow 0.2s ease;
  }

  @property --ai-angle {
    syntax: '<angle>';
    initial-value: 0deg;
    inherits: false;
  }

  @keyframes ai-spin {
    to { --ai-angle: 360deg; }
  }

  /* Active processing state: animated flowing rainbow border */
  .card-wrapper.analyzing {
    padding: 2px;
    background: conic-gradient(
      from var(--ai-angle),
      #7c3aed 0%,
      #2563eb 25%,
      #06b6d4 50%,
      #7c3aed 75%,
      #7c3aed 100%
    );
    animation: ai-spin 3s linear infinite;
    box-shadow: 0 0 16px 3px rgba(124, 58, 237, 0.5);
  }

  /* Completed AI labeling state: static glowing border */
  .card-wrapper.ai-labeled {
    padding: 2px;
    background: linear-gradient(
      135deg,
      #7c3aed 0%,
      #2563eb 50%,
      #06b6d4 100%
    );
    box-shadow: 0 0 10px 1px rgba(124, 58, 237, 0.25);
  }

  .card-wrapper.analyzing .card,
  .card-wrapper.ai-labeled .card {
    border-color: transparent;
  }

  /* ── Card base ─────────────────────────────────────── */
  .card {
    background: var(--surface);
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

  /* ── State overlays ───────────────────────────────── */
  .state-overlay {
    position: absolute;
    inset: 0;
    background: rgba(30, 30, 45, var(--overlay-opacity, 0.5));
    pointer-events: none;
  }

  /* ── AI spinner ───────────────────────────────────── */
  .ai-spinner {
    border-top-color: #7c3aed !important;
    border-right-color: #2563eb !important;
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

  .menu-item-ai:not(:disabled) {
    background: linear-gradient(90deg, rgba(124,58,237,0.08), transparent);
    border-left: 2px solid #7c3aed;
  }

  .menu-item-ai:hover:not(:disabled) {
    background: rgba(124, 58, 237, 0.18);
    color: #a78bfa;
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

  .modal-overlay {
    position: fixed;
    top: 0;
    left: 0;
    width: 100vw;
    height: 100vh;
    background: rgba(0, 0, 0, 0.6);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .info-modal {
    background: var(--surface);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-lg);
    width: 90%;
    max-width: 600px;
    padding: 24px;
    box-shadow: var(--shadow-lg);
    color: var(--text-primary);
  }

  .modal-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 20px;
  }

  .modal-header h3 {
    margin: 0;
    font-size: 1.25rem;
    font-weight: 600;
  }

  .close-btn {
    background: none;
    border: none;
    color: var(--text-secondary);
    font-size: 1.25rem;
    cursor: pointer;
    padding: 4px;
    transition: var(--transition);
  }

  .close-btn:hover {
    color: var(--text-primary);
  }

  .info-table {
    width: 100%;
    border-collapse: collapse;
  }

  .info-table td {
    padding: 10px;
    border-bottom: 1px solid var(--border-color);
    font-size: 0.9rem;
    vertical-align: top;
    word-break: break-all;
  }

  .info-table tr:last-child td {
    border-bottom: none;
  }

  .link-styled {
    color: var(--accent-yellow, #ffd700);
    text-decoration: underline;
  }

  .link-styled:hover {
    color: #fff;
  }
</style>
