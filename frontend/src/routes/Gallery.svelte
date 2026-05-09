<script lang="ts">
  import { onMount } from 'svelte';
  import { getVideos } from '../lib/api/videos';
  import Sidebar from '../lib/gallery/Sidebar.svelte';
  import VideoGrid from '../lib/gallery/VideoGrid.svelte';
  import type { Video } from '../lib/api/types';

  let allVideos = $state<Video[]>([]);
  let filteredVideos = $state<Video[]>([]);
  let loading = $state(true);
  let errorMsg = $state('');

  const FILTER_KEY = 'ef_gallery_filter';
  const SCROLL_KEY = 'ef_gallery_scroll';

  function loadSavedFilter() {
    try {
      const raw = sessionStorage.getItem(FILTER_KEY);
      if (raw) return JSON.parse(raw) as { fighter_ids: string[]; date_from: string; date_to: string };
    } catch { /* ignore */ }
    return { fighter_ids: [], date_from: '', date_to: '' };
  }

  let activeFilter = $state<{
    fighter_ids: string[];
    date_from: string;
    date_to: string;
  }>(loadSavedFilter());

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
      result = result.filter((v) =>
        activeFilter.fighter_ids.every(
          (id) => v.fighter_a?.id === id || v.fighter_b?.id === id
        )
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
    sessionStorage.setItem(FILTER_KEY, JSON.stringify(filter));
    applyFilter();
  }

  function handleOpen(id: string) {
    sessionStorage.setItem(SCROLL_KEY, String(window.scrollY));
    window.location.hash = '#/player/' + id;
  }

  let ws: WebSocket | null = null;

  function connectWS() {
    ws = new WebSocket('ws://localhost:8080/ws');

    ws.onopen = () => {
      const token = localStorage.getItem('ef_token');
      if (token) ws!.send(JSON.stringify({ token }));
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
        } else if (msg.type === 'video_removed') {
          allVideos = allVideos.filter(v => v.id !== msg.id);
          applyFilter();
        }
      } catch {
        // ignore malformed messages
      }
    };

    ws.onclose = () => {
      setTimeout(() => { if (ws !== null) connectWS(); }, 4000);
    };
  }

  onMount(() => {
    loadVideos().then(() => {
      const saved = sessionStorage.getItem(SCROLL_KEY);
      if (saved) {
        requestAnimationFrame(() => { window.scrollTo(0, parseInt(saved)); });
        sessionStorage.removeItem(SCROLL_KEY);
      }
    });
    connectWS();
    return () => {
      const socket = ws;
      ws = null;
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
    <div class="sidebar-sticky">
      <Sidebar videos={allVideos} onfilter={handleFilter} initialFilter={activeFilter} />
    </div>
    <div class="content">
      <VideoGrid videos={filteredVideos} onopen={handleOpen} />
    </div>
  </div>
{/if}

<style>
  .gallery {
    display: flex;
    gap: 24px;
    align-items: flex-start;
    min-height: calc(100vh - 64px);
    margin-top: -24px;
  }

  .sidebar-sticky {
    position: sticky;
    top: 0;
    max-height: calc(100vh - 64px);
    overflow-y: auto;
    flex-shrink: 0;
    align-self: flex-start;
    width: 280px;
  }

  .content {
    flex: 1;
    min-width: 0;
    padding-top: 24px; /* Restore padding only for video content */
  }

  @media (max-width: 768px) {
    .gallery {
      flex-direction: column;
    }
    .sidebar-sticky {
      position: static;
      width: 100%;
      max-height: none;
    }
  }

  .state {
    text-align: center;
    padding: 64px;
    font-size: 0.9rem;
  }

  .loading { color: #4a6280; }
  .error { color: #e05252; }
</style>
