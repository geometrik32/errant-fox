<script lang="ts">
  import { onMount } from 'svelte';
  import { getVideos } from '../lib/api/videos';
  import Sidebar from '../lib/gallery/Sidebar.svelte';
  import VideoGrid from '../lib/gallery/VideoGrid.svelte';
  import TagModal from '../lib/gallery/TagModal.svelte';
  import type { Video } from '../lib/api/types';

  let allVideos = $state<Video[]>([]);
  let filteredVideos = $state<Video[]>([]);
  let tagTarget = $state<Video | null>(null);
  let loading = $state(true);
  let errorMsg = $state('');

  let activeFilter = $state<{
    fighter_ids: string[];
    date_from: string;
    date_to: string;
  }>({ fighter_ids: [], date_from: '', date_to: '' });

  async function loadVideos() {
    loading = true;
    errorMsg = '';
    try {
      allVideos = await getVideos();
      applyFilter();
    } catch (e) {
      errorMsg = e instanceof Error ? e.message : 'Ошибка загрузки видео';
    } finally {
      loading = false;
    }
  }

  function applyFilter() {
    let result = allVideos;

    if (activeFilter.fighter_ids.length > 0) {
      result = result.filter(
        (v) =>
          (v.fighter_a && activeFilter.fighter_ids.includes(v.fighter_a.id)) ||
          (v.fighter_b && activeFilter.fighter_ids.includes(v.fighter_b.id))
      );
    }

    if (activeFilter.date_from) {
      result = result.filter((v) => v.date >= activeFilter.date_from);
    }

    if (activeFilter.date_to) {
      result = result.filter((v) => v.date <= activeFilter.date_to);
    }

    filteredVideos = result;
  }

  function handleFilter(filter: { fighter_ids: string[]; date_from: string; date_to: string }) {
    activeFilter = filter;
    applyFilter();
  }

  function handleOpen(id: string) {
    window.location.hash = '#/player/' + id;
  }

  function handleTag(video: Video) {
    tagTarget = video;
  }

  async function handleSaved() {
    tagTarget = null;
    await loadVideos();
  }

  // WebSocket: receives new_video events from the backend sync process
  let ws: WebSocket | null = null;

  function connectWS() {
    ws = new WebSocket('ws://localhost:8080/ws');

    ws.onopen = () => {
      const token = localStorage.getItem('ef_token');
      if (token) {
        ws!.send(JSON.stringify({ token }));
      }
    };

    ws.onmessage = (e) => {
      try {
        const msg = JSON.parse(e.data as string);
        if (msg.type === 'new_video') {
          const newVideo: Video = {
            id: msg.id,
            date: msg.date,
            fighter_a: null,
            fighter_b: null,
            is_tagged: false,
            preview_url: msg.preview_url ?? `/api/videos/${msg.id}/previews/0`,
            preview_count: 0,
          };
          allVideos = [newVideo, ...allVideos];
          applyFilter();
        }
      } catch {
        // ignore malformed messages
      }
    };

    ws.onclose = () => {
      // reconnect after 4s unless the component was destroyed
      setTimeout(() => {
        if (ws !== null) connectWS();
      }, 4000);
    };
  }

  onMount(() => {
    loadVideos();
    connectWS();

    return () => {
      const socket = ws;
      ws = null; // prevent reconnect
      socket?.close();
    };
  });
</script>

{#if loading}
  <div class="state loading">Загрузка видео…</div>
{:else if errorMsg}
  <div class="state error">{errorMsg}</div>
{:else}
  <div class="gallery">
    <Sidebar videos={allVideos} onfilter={handleFilter} />
    <div class="content">
      <VideoGrid videos={filteredVideos} onopen={handleOpen} ontag={handleTag} />
    </div>
  </div>
{/if}

{#if tagTarget}
  <TagModal video={tagTarget} onsaved={handleSaved} onclose={() => (tagTarget = null)} />
{/if}

<style>
  .gallery {
    display: flex;
    gap: 28px;
    align-items: flex-start;
  }

  .content {
    flex: 1;
    min-width: 0;
  }

  .state {
    text-align: center;
    padding: 64px;
    font-size: 0.9rem;
  }

  .loading {
    color: #4a6280;
  }

  .error {
    color: #e05252;
  }
</style>
