<script lang="ts">
  import type { Video } from '../api/types';
  import { resolveColor } from '../api/types';

  interface Props {
    video: Video;
    onopen?: (id: string) => void;
  }

  let { video, onopen }: Props = $props();

  let imgError = $state(false);

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

  let previewSrc = $derived(video.preview_url);
</script>

<button
  class="card"
  onclick={handleClick}
  onauxclick={handleAuxClick}
>
  <div class="preview">
    {#if !imgError}
      <img src={previewSrc} alt="" loading="lazy" onerror={handleImgError} />
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
</style>
