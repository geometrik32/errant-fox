<script lang="ts">
  import type { Video } from '../api/types';

  interface Props {
    video: Video;
    onopen?: (id: string) => void;
    ontag?: (video: Video) => void;
  }

  let { video, onopen, ontag }: Props = $props();

  let frame = $state(0);

  function handleMouseMove(e: MouseEvent) {
    if (video.preview_count <= 1) return;
    const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
    const x = e.clientX - rect.left;
    frame = Math.min(
      video.preview_count - 1,
      Math.floor((x / rect.width) * video.preview_count)
    );
  }

  function handleMouseLeave() {
    frame = 0;
  }

  function handleClick() {
    if (video.is_tagged) {
      onopen?.(video.id);
    } else {
      ontag?.(video);
    }
  }

  let previewSrc = $derived(
    video.preview_url.replace(/\/previews\/\d+$/, `/previews/${frame}`)
  );

  let formattedDate = $derived(
    new Date(video.date + 'T00:00:00').toLocaleDateString('ru-RU', {
      day: '2-digit',
      month: '2-digit',
      year: 'numeric',
    })
  );
</script>

<button
  class="card"
  onclick={handleClick}
  onmousemove={handleMouseMove}
  onmouseleave={handleMouseLeave}
>
  <div class="preview">
    <img src={previewSrc} alt="" loading="lazy" />
  </div>

  <div class="info">
    {#if video.is_tagged}
      <div class="fighters">
        <div class="fighter">
          {#if video.fighter_a?.avatar_url}
            <img class="avatar" src={video.fighter_a.avatar_url} alt={video.fighter_a.display_name} />
          {:else}
            <div class="avatar-fallback">{video.fighter_a?.display_name?.charAt(0) ?? '?'}</div>
          {/if}
          <span class="name">{video.fighter_a?.display_name}</span>
        </div>

        <div class="score">{video.total_score_a ?? 0} : {video.total_score_b ?? 0}</div>

        <div class="fighter right">
          <span class="name">{video.fighter_b?.display_name}</span>
          {#if video.fighter_b?.avatar_url}
            <img class="avatar" src={video.fighter_b.avatar_url} alt={video.fighter_b.display_name} />
          {:else}
            <div class="avatar-fallback">{video.fighter_b?.display_name?.charAt(0) ?? '?'}</div>
          {/if}
        </div>
      </div>
      <div class="date">{formattedDate}</div>
    {:else}
      <div class="untagged">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" aria-hidden="true">
          <circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="1.5" />
          <path d="M12 8v4m0 4h.01" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
        </svg>
        <span>Заполните данные</span>
      </div>
      <div class="date">{formattedDate}</div>
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
    padding: 10px 12px;
    display: flex;
    flex-direction: column;
    gap: 4px;
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

  .avatar {
    width: 22px;
    height: 22px;
    border-radius: 50%;
    object-fit: cover;
    flex-shrink: 0;
    border: 1px solid #2a4f73;
  }

  .avatar-fallback {
    width: 22px;
    height: 22px;
    border-radius: 50%;
    background: #1a3050;
    border: 1px solid #2a4f73;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 0.65rem;
    font-weight: 600;
    color: #a0b4c8;
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

  .date {
    font-size: 0.72rem;
    color: #4a6280;
  }

  .untagged {
    display: flex;
    align-items: center;
    gap: 6px;
    color: #4a6280;
    font-size: 0.78rem;
  }
</style>
