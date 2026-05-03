<script lang="ts">
  import { onMount } from 'svelte';
  import { getVideo } from '../lib/api/videos';
  import type { VideoFull, Bout, Comment } from '../lib/api/types';
  import VideoPlayer from '../lib/player/VideoPlayer.svelte';
  import JudgingPanel from '../lib/player/JudgingPanel.svelte';
  import Chat from '../lib/player/Chat.svelte';
  import Timeline from '../lib/player/Timeline.svelte';

  interface Props {
    videoId: string;
    initialTimeMs?: number;
  }
  let { videoId, initialTimeMs = 0 }: Props = $props();

  let video = $state<VideoFull | null>(null);
  let loading = $state(true);
  let loadError = $state<string | null>(null);

  // Playback state
  let currentTime = $state(0);
  let duration = $state(0);
  let playing = $state(false);
  let looping = $state(false);
  let speed = $state(1);
  let volume = $state(1);
  let fps = $state(25);

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  let player: any = $state(null);
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  let judgingPanel: any = $state(null);

  let layoutEl: HTMLElement | null = $state(null);

  async function toggleFullscreen() {
    if (document.fullscreenElement) {
      await document.exitFullscreen();
    } else {
      await layoutEl?.requestFullscreen();
    }
  }

  // Live data that Timeline reads (updated by JudgingPanel + Chat via WS)
  let liveBouts = $state<Bout[]>([]);
  let liveComments = $state<Comment[]>([]);

  // Panel visibility
  let showJudging = $state(true);
  let showChat = $state(true);

  onMount(async () => {
    try {
      video = await getVideo(videoId);
      if (video.duration_ms) duration = video.duration_ms / 1000;
      liveBouts = [...video.bouts];
      liveComments = [...video.comments];
    } catch (e) {
      loadError = e instanceof Error ? e.message : 'Ошибка загрузки видео';
    } finally {
      loading = false;
    }
  });

  // Seek to initial time after player is ready
  $effect(() => {
    if (player && initialTimeMs > 0 && !loading && video) {
      player.seekTo(initialTimeMs);
    }
  });

  function handleKeydown(e: KeyboardEvent) {
    const tag = (e.target as HTMLElement).tagName.toLowerCase();
    if (tag === 'input' || tag === 'textarea' || tag === 'select') return;

    if (e.code === 'Space') {
      e.preventDefault(); player?.togglePlay();
    } else if (e.code === 'KeyX') {
      e.preventDefault(); player?.stepForward();
    } else if (e.code === 'KeyZ') {
      e.preventDefault(); player?.stepBackward();
    } else if (e.code === 'ArrowLeft') {
      e.preventDefault(); player?.seekTo(Math.max(0, (currentTime - 2) * 1000));
    } else if (e.code === 'ArrowRight') {
      e.preventDefault(); player?.seekTo(Math.min(duration, currentTime + 2) * 1000);
    } else if (e.code === 'KeyC') {
      e.preventDefault(); judgingPanel?.triggerMark();
    } else if (e.code === 'KeyF') {
      e.preventDefault(); toggleFullscreen();
    } else if (e.code === 'KeyA') {
      e.preventDefault();
      const s = speed === 0.2 ? 1 : 0.2;
      speed = s; player?.setSpeed(s);
    } else if (e.code === 'KeyS') {
      e.preventDefault();
      const s = speed === 2 ? 1 : 2;
      speed = s; player?.setSpeed(s);
    } else if (e.code === 'KeyG') {
      e.preventDefault();
      showJudging = !showJudging;
      showChat = !showChat;
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if loading}
  <div class="state-msg">Загрузка…</div>
{:else if loadError}
  <div class="state-msg error">{loadError}</div>
{:else if video}
  <div class="layout" bind:this={layoutEl}>

    <div class="cols">

      <!-- Toggle pills -->
      <div class="panel-toggles">
        <button
          class="panel-toggle"
          class:active={showJudging}
          onclick={() => { showJudging = !showJudging; }}
          title="Панель судейства"
        >
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true">
            <rect x="3" y="3" width="7" height="18" rx="1"/>
            <rect x="14" y="3" width="7" height="8" rx="1"/>
            <rect x="14" y="15" width="7" height="6" rx="1"/>
          </svg>
          Судейство
        </button>
        <button
          class="panel-toggle"
          class:active={showChat}
          onclick={() => { showChat = !showChat; }}
          title="Чат"
        >
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true">
            <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/>
          </svg>
          Чат
        </button>
      </div>

      <!-- Left: Judging panel -->
      {#if showJudging}
        <div class="col col-left">
          <JudgingPanel
            bind:this={judgingPanel}
            {video}
            {currentTime}
            {playing}
            onboutschange={(b) => { liveBouts = b; }}
            onseekrequest={(ms, endMs) => { player?.setLoop(ms, endMs, playing); }}
          />
        </div>
      {/if}

      <!-- Center: Video player -->
      <div class="col col-center">
        <VideoPlayer
          bind:this={player}
          src={video.stream_url}
          {speed}
          {volume}
          ontimeupdate={(t) => { currentTime = t; }}
          ondurationchange={(d) => { duration = d; }}
          onplayingchange={(p) => { playing = p; }}
          onloopingchange={(l) => { looping = l; }}
          onfpschange={(f) => { fps = f; }}
        />
      </div>

      <!-- Right: Chat -->
      {#if showChat}
        <div class="col col-right">
          <Chat
            {videoId}
            comments={video.comments}
            {currentTime}
            onseek={(ms) => { player?.seekTo(ms); player?.pause(); }}
            oncommentschange={(c) => { liveComments = c; }}
          />
        </div>
      {/if}

    </div>

    <!-- Bottom: Timeline -->
    <div class="timeline-row">
      <Timeline
        {currentTime}
        {duration}
        bouts={liveBouts}
        comments={liveComments}
        fighterA={video.fighter_a}
        fighterB={video.fighter_b}
        {playing}
        {looping}
        {speed}
        {volume}
        {fps}
        onseek={(ms) => player?.seekTo(ms)}
        onloop={({ start, end }) => { player?.seekTo(start); player?.setLoop(start, end); }}
        onboutclick={(id) => { judgingPanel?.expandBout(id); }}
        onplay={() => player?.togglePlay()}
        onstepback={() => player?.stepBackward()}
        onstepforward={() => player?.stepForward()}
        onspeedchange={(s) => { speed = s; player?.setSpeed(s); }}
        onvolumechange={(v) => { volume = v; player?.setVolume(v); }}
        onlooptoggle={() => player?.toggleLoop()}
      />
    </div>

  </div>
{/if}

<style>
  .layout {
    display: flex;
    flex-direction: column;
    height: calc(100vh - 56px);
    overflow: hidden;
    background: #0d1b2a;
  }

  .cols {
    display: flex;
    flex: 1;
    min-height: 0;
    overflow: hidden;
    position: relative;
  }

  /* Toggle pills — float over top-left of video area */
  .panel-toggles {
    position: absolute;
    top: 8px;
    left: 50%;
    transform: translateX(-50%);
    display: flex;
    gap: 6px;
    z-index: 10;
  }

  .panel-toggle {
    display: flex;
    align-items: center;
    gap: 5px;
    padding: 4px 10px;
    border-radius: 20px;
    border: 1px solid #1a3050;
    background: rgba(8, 16, 31, 0.85);
    color: #4a6280;
    font-size: 0.72rem;
    font-weight: 600;
    cursor: pointer;
    transition: background 0.12s, color 0.12s, border-color 0.12s;
    backdrop-filter: blur(4px);
  }

  .panel-toggle:hover {
    color: #a0b4c8;
    border-color: #2a4f73;
  }

  .panel-toggle.active {
    color: #DB841F;
    border-color: rgba(219, 132, 31, 0.5);
    background: rgba(219, 132, 31, 0.1);
  }

  .col {
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .col-left {
    width: 300px;
    flex-shrink: 0;
    overflow-y: hidden;
  }

  .col-center {
    flex: 1;
    min-width: 0;
  }

  .col-right {
    width: 320px;
    flex-shrink: 0;
  }

  .timeline-row {
    flex-shrink: 0;
  }

  .state-msg {
    display: flex;
    align-items: center;
    justify-content: center;
    height: calc(100vh - 56px);
    font-size: 0.9rem;
    color: #4a6280;
  }

  .state-msg.error { color: #e05252; }
</style>
