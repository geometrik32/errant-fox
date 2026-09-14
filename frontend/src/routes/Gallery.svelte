<script lang="ts">
  import { onMount } from 'svelte';
  import { getVideos, getVideo } from '../lib/api/videos';
  import { gallerySidebarOpen } from '../stores';
  import Sidebar from '../lib/gallery/Sidebar.svelte';
  import VideoGrid from '../lib/gallery/VideoGrid.svelte';
  import type { Video, VideoFull } from '../lib/api/types';

  let allVideos = $state<Video[]>([]);
  let filteredVideos = $state<Video[]>([]);
  let loading = $state(true);
  let errorMsg = $state('');
  let onlineUsers = $state<any[]>([]);

  let videoWatchers = $derived.by(() => {
    const obj: Record<string, any[]> = {};
    for (const u of onlineUsers) {
      if (u.watching) {
        if (!obj[u.watching]) {
          obj[u.watching] = [];
        }
        const list = obj[u.watching];
        if (!list.some(existing => existing.id === u.id)) {
          list.push(u);
        }
      }
    }
    return obj;
  });

  const FILTER_KEY = 'ef_gallery_filter';
  const SCROLL_KEY = 'ef_gallery_scroll';

  type EventTypeFilter = 'all' | 'training' | 'tournament';

  interface GalleryFilter {
    fighter_ids: string[];
    date_from: string;
    date_to: string;
    event_type: EventTypeFilter;
  }

  function loadSavedFilter(): GalleryFilter {
    try {
      const raw = sessionStorage.getItem(FILTER_KEY);
      if (raw) {
        const parsed = JSON.parse(raw);
        return {
          fighter_ids: parsed.fighter_ids || [],
          date_from: parsed.date_from || '',
          date_to: parsed.date_to || '',
          event_type: parsed.event_type || 'all',
        };
      }
    } catch { /* ignore */ }
    return { fighter_ids: [], date_from: '', date_to: '', event_type: 'all' };
  }

  let activeFilter = $state<GalleryFilter>(loadSavedFilter());

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

    if (activeFilter.event_type === 'training') {
      result = result.filter((v) => !v.is_tournament);
    } else if (activeFilter.event_type === 'tournament') {
      result = result.filter((v) => v.is_tournament);
    }

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

  function handleFilter(filter: GalleryFilter) {
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
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    ws = new WebSocket(`${protocol}//${window.location.host}/ws`);

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
            is_ai_labeled: false,
            is_analyzing: false,
            is_tournament: !!msg.is_tournament,
            tournament_name: msg.tournament_name || null,
            preview_url: msg.preview_url ?? `/api/videos/${msg.id}/previews/0`,
            preview_count: 0,
          };
          allVideos = [newVideo, ...allVideos];
          applyFilter();
        } else if (msg.type === 'video_removed') {
          allVideos = allVideos.filter(v => v.id !== msg.id);
          applyFilter();
        } else if (msg.type === 'presence_update') {
          console.log('Gallery presence update:', msg.users);
          onlineUsers = msg.users;
        } else if (msg.type === 'update_video_score') {
          allVideos = allVideos.map(v => {
            if (v.id === msg.video_id) {
              return { ...v, total_score_a: msg.total_score_a, total_score_b: msg.total_score_b };
            }
            return v;
          });
          applyFilter();
        } else if (msg.type === 'update_video_fighters') {
          allVideos = allVideos.map(v => {
            if (v.id === msg.video_id) {
              return { ...v, fighter_a: msg.fighter_a, fighter_b: msg.fighter_b, is_tagged: !!(msg.fighter_a && msg.fighter_b) };
            }
            return v;
          });
          applyFilter();
        } else if (msg.type === 'update_video_preview') {
          allVideos = allVideos.map(v => {
            if (v.id === msg.video_id) {
              return { ...v, preview_url: msg.preview_url ?? v.preview_url, preview_count: 1 };
            }
            return v;
          });
          applyFilter();
        } else if (msg.type === 'update_video_ai_labeled') {
          allVideos = allVideos.map(v => {
            if (v.id === msg.video_id) {
              const has_transcript = msg.has_transcript !== undefined ? msg.has_transcript : v.has_transcript;
              return { ...v, is_ai_labeled: msg.is_ai_labeled, is_analyzing: msg.is_analyzing, is_queued: msg.is_queued, has_transcript };
            }
            return v;
          });
          applyFilter();
          if (!msg.is_analyzing && !msg.is_queued) {
            getVideo(msg.video_id).then((updated: VideoFull) => {
              allVideos = allVideos.map(v => v.id === msg.video_id ? { ...v, ...updated, is_analyzing: false, is_queued: false } : v);
              applyFilter();
            }).catch(() => {});
          }
        } else if (msg.type === 'update_video_optimized') {
          allVideos = allVideos.map(v => {
            if (v.id === msg.video_id) {
              return { ...v, is_optimized: msg.is_optimized, is_optimizing: msg.is_optimizing };
            }
            return v;
          });
          applyFilter();
        } else if (msg.type === 'update_video_transcode') {
          allVideos = allVideos.map(v => {
            if (v.id === msg.video_id) {
              return { ...v, has_h264: msg.has_h264, is_transcoding: msg.is_transcoding };
            }
            return v;
          });
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
  <div class="gallery" class:gallery--no-sidebar={!$gallerySidebarOpen}>
    <!-- Sidebar — grows to fit content, no scrollbars -->
    {#if $gallerySidebarOpen}
      <div class="sidebar-island">
        <Sidebar videos={allVideos} onfilter={handleFilter} initialFilter={activeFilter} {onlineUsers} />
      </div>
    {/if}

    <div class="content">
      <VideoGrid videos={filteredVideos} {videoWatchers} onopen={handleOpen} onreload={loadVideos} />
    </div>
  </div>
{/if}

<style>
  .gallery {
    display: grid;
    grid-template-columns: 280px 1fr;
    gap: 24px;
    align-items: flex-start;
    min-height: calc(100vh - 64px);
    margin-top: -24px;
  }

  .gallery.gallery--no-sidebar {
    grid-template-columns: 1fr;
  }

  .sidebar-island {
    position: sticky;
    top: 24px;
    width: 280px;
    flex-shrink: 0;
    background: var(--surface);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-lg);
    box-sizing: border-box;
    overflow: visible;
  }

  .content {
    min-width: 0;
    padding-top: 24px;
  }

  @media (max-width: 768px) {
    .gallery {
      grid-template-columns: 1fr !important;
      gap: 16px;
      margin-top: 0;
    }
    .sidebar-island {
      position: fixed;
      top: 64px;
      left: 0;
      bottom: 0;
      width: 85vw;
      max-width: 320px;
      z-index: 500;
      border-radius: 0 var(--radius-lg) var(--radius-lg) 0;
      box-shadow: var(--shadow-lg);
      background: var(--surface-solid);
      overflow-y: auto;
      animation: slideInDrawer 0.25s ease-out;
    }
  }

  @keyframes slideInDrawer {
    from { transform: translateX(-100%); }
    to { transform: translateX(0); }
  }

  .state {
    text-align: center;
    padding: 64px;
    font-size: 0.9rem;
  }

  .loading { color: #4a6280; }
  .error { color: #e05252; }
</style>
