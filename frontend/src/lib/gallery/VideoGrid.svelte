<script lang="ts">
  import type { Video } from '../api/types';
  import VideoCard from './VideoCard.svelte';

  interface Props {
    videos: Video[];
    onopen?: (id: string) => void;
    ontag?: (video: Video) => void;
  }

  let { videos, onopen, ontag }: Props = $props();
</script>

<div class="grid">
  {#each videos as video (video.id)}
    <VideoCard {video} {onopen} {ontag} />
  {/each}

  {#if videos.length === 0}
    <p class="empty">Видео не найдено</p>
  {/if}
</div>

<style>
  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(260px, 1fr));
    gap: 16px;
  }

  .empty {
    grid-column: 1 / -1;
    color: #4a6280;
    text-align: center;
    padding: 64px 0;
    font-size: 0.9rem;
  }
</style>
