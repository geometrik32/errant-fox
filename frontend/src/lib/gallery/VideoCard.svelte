<script lang="ts">
  import type { Video } from '../api/types';
  import { resolveColor } from '../api/types';

  interface Props {
    video: Video;
    onopen?: (id: string) => void;
  }

  let { video, onopen }: Props = $props();

  let frame = $state(0);
  let imgError = $state(false);

  function handleMouseMove(e: MouseEvent) {
    if (video.preview_count <= 1) return;
    const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
    const x = e.clientX - rect.left;
    frame = Math.min(video.preview_count - 1, Math.floor((x / rect.width) * video.preview_count));
  }

  function handleMouseLeave() {
    frame = 0;
  }

  function handleClick() {
    onopen?.(video.id);
  }

  function handleImgError() {
    imgError = true;
    setTimeout(() => { imgError = false; }, 3000);
  }

  let previewSrc = $derived(
    video.preview_url.replace(/\/previews\/\d+$/, `/previews/${frame}`)
  );
</script>

<button
  class="card"
  onclick={handleClick}
  onmousemove={handleMouseMove}
  onmouseleave={handleMouseLeave}
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
    background: #0f2035;
    border: 1px solid #1f3a57;
    border-radius: 8px;
    overflow: hidden;
    cursor: pointer;
    text-align: left;
    padding: 0;
    width: 100%;
    display: flex;
    flex-direction: column;
    transition: border-color 0.15s, transform 0.15s;
  }

  .card:hover {
    border-color: #2a4f73;
    transform: translateY(-1px);
  }

  .preview {
    width: 100%;
    aspect-ratio: 16 / 9;
    background: #060e18;
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
    padding: 8px 12px;
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
    font-size: 0.78rem;
    color: #a0b4c8;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .score {
    font-size: 0.88rem;
    font-weight: 700;
    color: #DB841F;
    white-space: nowrap;
    flex-shrink: 0;
  }

  .untagged {
    display: flex;
    align-items: center;
    gap: 6px;
    color: #4a6280;
    font-size: 0.78rem;
  }
</style>
