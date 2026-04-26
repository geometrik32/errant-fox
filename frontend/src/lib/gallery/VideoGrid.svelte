<script lang="ts">
  import type { Video } from '../api/types';
  import VideoCard from './VideoCard.svelte';

  interface Props {
    videos: Video[];
    onopen?: (id: string) => void;
  }

  let { videos, onopen }: Props = $props();

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
</script>

{#if videos.length === 0}
  <p class="empty">Видео не найдено</p>
{:else}
  {#each groups as group (group.date)}
    <div class="date-group">
      <h3 class="date-label">{group.label}</h3>
      <div class="grid">
        {#each group.videos as video (video.id)}
          <VideoCard {video} {onopen} />
        {/each}
      </div>
    </div>
  {/each}
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

  .date-label {
    font-size: 0.72rem;
    font-weight: 600;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: #4a6280;
    margin: 0 0 12px;
    padding-bottom: 6px;
    border-bottom: 1px solid #1f3a57;
  }

  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(260px, 1fr));
    gap: 16px;
  }
</style>
