<script lang="ts">
  import { onMount, onDestroy, untrack } from 'svelte';
  import { currentUser } from '../stores';
  import { getVideo, getSharedVideo } from '../lib/api/videos';
  import type { VideoFull, Bout, Comment } from '../lib/api/types';
  import VideoPlayer from '../lib/player/VideoPlayer.svelte';
  import JudgingPanel from '../lib/player/JudgingPanel.svelte';
  import Chat from '../lib/player/Chat.svelte';
  import Timeline from '../lib/player/Timeline.svelte';

  interface Props {
    videoId: string;
    initialTimeMs?: number;
    shareToken?: string;
    sharedBoutId?: number | null;
  }
  let { videoId, initialTimeMs = 0, shareToken = '', sharedBoutId = null }: Props = $props();

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
  let myWsId = $state<string | null>(null);

  let sharedBout = $derived(
    sharedBoutId && video
      ? video.bouts.find(b => b.id === sharedBoutId)
      : null
  );

  let boutStart = $derived(sharedBout ? sharedBout.time_start_ms / 1000 : 0);
  let boutEnd = $derived(sharedBout ? sharedBout.time_end_ms / 1000 : 0);
  let timelineDuration = $derived(sharedBout ? boutEnd - boutStart : duration);
  let timelineCurrentTime = $derived(sharedBout ? Math.max(0, Math.min(timelineDuration, currentTime - boutStart)) : currentTime);

  let timelineComments = $derived(
    sharedBout
      ? liveComments
          .filter(c => c.timestamp_ms >= sharedBout.time_start_ms && c.timestamp_ms <= sharedBout.time_end_ms)
          .map(c => ({
            ...c,
            timestamp_ms: c.timestamp_ms - sharedBout.time_start_ms
          }))
      : liveComments
  );

  let activeViewers = $derived.by(() => {
    const currentViewers = onlineUsers.filter(u => u.watching === videoId);
    const seen = new Set<string>();
    return currentViewers.filter(u => {
      if (u.id === $currentUser?.id) return false;
      if (myWsId && u.id === myWsId) return false;
      if (seen.has(u.id)) return false;
      seen.add(u.id);
      return true;
    });
  });

  // Panel visibility
  let showJudging = $state(true);
  let showChat = $state(true);
  let chatComponent = $state<any>(null);

  // Marking state for video outline feedback
  let markingActive = $state(false);
  let markingFinishAnimationKey = $state(0);

  // Comment highlight driven by timeline marker click
  let highlightedCommentId = $state<number | null>(null);

  let ws: WebSocket | null = null;

  function connectWS() {
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    ws = new WebSocket(`${protocol}//${window.location.host}/ws`);

    ws.onopen = () => {
      const tok = shareToken || localStorage.getItem('ef_token');
      if (tok) ws!.send(JSON.stringify({ token: tok }));
      ws!.send(JSON.stringify({ watching: videoId }));
    };

    ws.onmessage = async (e) => {
      try {
        const msg = JSON.parse(e.data as string);
        if (msg.type === 'init') {
          myWsId = msg.user_id;
        } else if (msg.type === 'update_video_ai_labeled' && msg.video_id === videoId) {
          if (video) {
            video.is_ai_labeled = msg.is_ai_labeled;
            video.is_analyzing = msg.is_analyzing;
          }
          if (!msg.is_analyzing) {
            const reloaded = shareToken
              ? await getSharedVideo(videoId, shareToken)
              : await getVideo(videoId);
            video = reloaded;
            liveBouts = sharedBoutId ? reloaded.bouts.filter(b => b.id === sharedBoutId) : [...reloaded.bouts];
            liveComments = [...reloaded.comments];
          }
        } else if (msg.type === 'presence_update') {
          onlineUsers = msg.users as any[];
        }

        judgingPanel?.handleWsMessage(msg);
        chatComponent?.handleWsMessage(msg);
      } catch {
        // ignore
      }
    };

    ws.onclose = () => {
      setTimeout(() => { if (ws !== null) connectWS(); }, 4000);
    };
  }

  onMount(async () => {
    try {
      video = shareToken ? await getSharedVideo(videoId, shareToken) : await getVideo(videoId);
      if (video.duration_ms) duration = video.duration_ms / 1000;
      fps = video.fps ?? null;
      liveBouts = sharedBoutId ? video.bouts.filter(b => b.id === sharedBoutId) : [...video.bouts];
      liveComments = [...video.comments];
      connectWS();
    } catch (e) {
      loadError = e instanceof Error ? e.message : 'Ошибка загрузки видео';
    } finally {
      loading = false;
    }
  });

  onDestroy(() => {
    const w = ws;
    ws = null;
    w?.close();
  });

  // Seek to initial time or shared bout after player is ready (one-shot)
  let _initialSeekDone = false;
  $effect(() => {
    if (_initialSeekDone) return;
    if (!player || loading) return;
    // Use untrack for video to avoid re-runs on WS mutations
    const v = untrack(() => video);
    if (!v) return;
    const boutId = untrack(() => sharedBoutId);
    if (boutId != null) {
      const bout = v.bouts.find(b => b.id === boutId);
      if (bout) {
        _initialSeekDone = true;
        const targetMs = initialTimeMs > 0 ? initialTimeMs : bout.time_start_ms;
        player.seekTo(targetMs);
        player.setLoop(bout.time_start_ms, bout.time_end_ms);
      }
    } else if (initialTimeMs > 0) {
      _initialSeekDone = true;
      player.seekTo(initialTimeMs);
    } else {
      _initialSeekDone = true;
    }
  });

  // DEBUG: trace timeline values (remove after fix verified)
  $effect(() => {
    if (sharedBout) {
      console.log('[Player] currentTime:', currentTime.toFixed(3),
        'boutStart:', boutStart.toFixed(3),
        'timelineCurrentTime:', timelineCurrentTime.toFixed(3),
        'timelineDuration:', timelineDuration.toFixed(3));
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
      if (!shareToken) player?.toggleLoop();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if loading}
  <div class="state-msg">Загрузка…</div>
{:else if loadError}
  <div class="state-msg error">{loadError}</div>
{:else if video}
  <div class="layout" class:no-header={!!shareToken} bind:this={layoutEl}>

    <div class="cols">

      <!-- Left: Judging panel -->
      <div class="col col-left" class:hidden={!showJudging}>
        <JudgingPanel
          bind:this={judgingPanel}
          {video}
          {currentTime}
          {playing}
          {shareToken}
          {sharedBoutId}
          readonly={!!shareToken}
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
            bind:this={chatComponent}
            {videoId}
            comments={video.comments}
            {currentTime}
            highlightedId={highlightedCommentId}
            bouts={liveBouts}
            {shareToken}
            {sharedBoutId}
            onseek={(ms) => { player?.seekTo(ms); player?.pause(); }}
            oncommentschange={(c) => { liveComments = c; }}
          />
        </div>
      {/if}

    </div>

    <!-- Bottom: Timeline -->
    <div class="timeline-row">
      <Timeline
        currentTime={sharedBout ? timelineCurrentTime : currentTime}
        duration={sharedBout ? timelineDuration : duration}
        bouts={sharedBout ? [sharedBout] : liveBouts}
        comments={sharedBout ? timelineComments : liveComments}
        fighterA={video.fighter_a}
        fighterB={video.fighter_b}
        {playing}
        {looping}
        {speed}
        {volume}
        {fps}
        {startTime}
        {finishing}
        readonly={!!shareToken}
        onseek={(ms) => {
          const targetMs = sharedBout ? ms + sharedBout.time_start_ms : ms;
          player?.seekTo(targetMs);
        }}
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

  .layout.no-header {
    height: 100vh;
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
