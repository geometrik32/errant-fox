<script lang="ts">
  import { onMount } from 'svelte';
  import { currentUser } from '../stores';
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
  let fps = $state<number | null>(null);

  // Judging state
  let startTime = $state<number | null>(null);
  let finishing = $state(false);

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
  let onlineUsers = $state<any[]>([]);
  let activeViewers = $derived.by(() => {
    const currentViewers = onlineUsers.filter(u => u.watching === videoId);
    const seen = new Set<string>();
    return currentViewers.filter(u => {
      if (u.id === $currentUser?.id) return false;
      if (seen.has(u.id)) return false;
      seen.add(u.id);
      return true;
    });
  });

  // Panel visibility
  let showJudging = $state(true);
  let showChat = $state(true);

  // Marking state for video outline feedback
  let markingActive = $state(false);
  let markingFinishAnimationKey = $state(0);

  // Comment highlight driven by timeline marker click
  let highlightedCommentId = $state<number | null>(null);

  onMount(async () => {
    try {
      video = await getVideo(videoId);
      if (video.duration_ms) duration = video.duration_ms / 1000;
      fps = video.fps ?? null;
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
    } else if (e.code === 'KeyD') {
      e.preventDefault();
      player?.toggleLoop();
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

      <!-- Left: Judging panel -->
      <div class="col col-left" class:hidden={!showJudging}>
        <JudgingPanel
          bind:this={judgingPanel}
          {video}
          {currentTime}
          {playing}
          bind:startTime={startTime}
          bind:finishing={finishing}
          onboutschange={(b) => { liveBouts = b; }}
          onseekrequest={(ms, endMs) => { player?.setLoop(ms, endMs, playing); }}
          onpauserequest={() => { player?.pause(); }}
          onmarkingchange={(active) => { markingActive = active; }}
          onmarkingfinish={() => {
            markingActive = false;
            markingFinishAnimationKey += 1;
          }}
          onboutdelete={() => { player?.toggleLoop(); }}
          onpresenceupdate={(users) => { onlineUsers = users; }}
        />
      </div>

      <!-- Center: Video player -->
      <div class="col col-center">
        <VideoPlayer
          bind:this={player}
          src={video.stream_url}
          {speed}
          {volume}
          fps={video.fps ?? null}
          judgingOpen={showJudging}
          chatOpen={showChat}
          {markingActive}
          {markingFinishAnimationKey}
          activeViewers={activeViewers}
          ontimeupdate={(t) => { currentTime = t; }}
          ondurationchange={(d) => { duration = d; }}
          onplayingchange={(p) => { playing = p; }}
          onloopingchange={(l) => { looping = l; }}
          ondetectedfps={(f) => { if (fps == null) fps = f; }}
          ontogglejudging={() => { showJudging = !showJudging; }}
          ontogglechat={() => { showChat = !showChat; }}
        />
      </div>

      <!-- Right: Chat -->
      {#if showChat}
        <div class="col col-right">
          <Chat
            {videoId}
            comments={video.comments}
            {currentTime}
            highlightedId={highlightedCommentId}
            bouts={liveBouts}
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
        {startTime}
        {finishing}
        onseek={(ms) => player?.seekTo(ms)}
        onloop={({ start, end }) => { player?.seekTo(start); player?.setLoop(start, end); }}
        onboutclick={(id) => { judgingPanel?.expandBout(id); }}
        oncommentclick={(id) => { highlightedCommentId = id; if (!showChat) showChat = true; }}
        onplay={() => player?.togglePlay()}
        onstepback={() => player?.stepBackward()}
        onstepforward={() => player?.stepForward()}
        onspeedchange={(s) => { speed = s; player?.setSpeed(s); }}
        onvolumechange={(v) => { volume = v; player?.setVolume(v); }}
        onlooptoggle={() => player?.toggleLoop()}
        onstartclick={() => judgingPanel?.handleStart()}
        onfinishclick={() => judgingPanel?.handleFinish()}
      />
    </div>

  </div>
{/if}

<style>
  .layout {
    display: flex;
    flex-direction: column;
    height: calc(100vh - 64px);
    overflow: hidden;
    background: transparent;
    padding: 16px;
    gap: 16px;
  }

  .cols {
    display: flex;
    flex: 1;
    min-height: 0;
    overflow: hidden;
    position: relative;
    gap: 16px;
  }

  .col {
    display: flex;
    flex-direction: column;
    overflow: hidden;
    background: var(--surface);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-lg);
    box-shadow: none;
  }

  .col-left {
    width: 300px;
    flex-shrink: 0;
    overflow-y: hidden;
  }

  .hidden {
    display: none !important;
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
    background: var(--surface);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-lg);
    box-shadow: none;
    overflow: hidden;
  }

  .state-msg {
    display: flex;
    align-items: center;
    justify-content: center;
    height: calc(100vh - 64px);
    font-size: 0.9rem;
    color: var(--text-secondary);
  }

  .state-msg.error { color: #ef4444; }

  @media (max-width: 1024px) {
    .cols {
      flex-direction: column;
      overflow-y: auto;
    }
    .col-left, .col-right {
      width: 100%;
      height: 400px;
      flex-shrink: 0;
    }
    .col-center {
      flex: none;
      height: auto;
      aspect-ratio: 16 / 9;
    }
  }
</style>
