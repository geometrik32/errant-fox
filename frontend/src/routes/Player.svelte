<script lang="ts">
  import { onMount } from 'svelte';
  import { getVideo } from '../lib/api/videos';
  import type { VideoFull } from '../lib/api/types';
  import VideoPlayer from '../lib/player/VideoPlayer.svelte';
  import JudgingPanel from '../lib/player/JudgingPanel.svelte';
  import Chat from '../lib/player/Chat.svelte';
  import Timeline from '../lib/player/Timeline.svelte';

  interface Props {
    videoId: string;
  }
  let { videoId }: Props = $props();

  let video = $state<VideoFull | null>(null);
  let loading = $state(true);
  let loadError = $state<string | null>(null);

  // Playback state reflected from VideoPlayer events
  let currentTime = $state(0);
  let duration = $state(0);
  let playing = $state(false);
  let looping = $state(false);
  let speed = $state(1);
  let volume = $state(1);

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  let player: any = $state(null);

  onMount(async () => {
    try {
      video = await getVideo(videoId);
      if (video.duration_ms) duration = video.duration_ms / 1000;
    } catch (e) {
      loadError = e instanceof Error ? e.message : 'Ошибка загрузки видео';
    } finally {
      loading = false;
    }
  });

  function handleKeydown(e: KeyboardEvent) {
    const tag = (e.target as HTMLElement).tagName.toLowerCase();
    if (tag === 'input' || tag === 'textarea' || tag === 'select') return;
    if (e.code === 'Space') { e.preventDefault(); player?.togglePlay(); }
    else if (e.code === 'KeyX') { e.preventDefault(); player?.stepForward(); }
    else if (e.code === 'KeyZ') { e.preventDefault(); player?.stepBackward(); }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if loading}
  <div class="state-msg">Загрузка…</div>
{:else if loadError}
  <div class="state-msg error">{loadError}</div>
{:else if video}
  <div class="layout">

    <div class="cols">

      <!-- Left: Judging panel — 300px -->
      <div class="col col-left">
        <JudgingPanel {video} {currentTime} />
      </div>

      <!-- Center: Video player — remaining space -->
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
        />
      </div>

      <!-- Right: Chat — 320px -->
      <div class="col col-right">
        <Chat
          {videoId}
          comments={video.comments}
          {currentTime}
          onseek={(ms) => { player?.seekTo(ms); player?.pause(); }}
        />
      </div>

    </div>

    <!-- Bottom: Timeline — full width -->
    <div class="timeline-row">
      <Timeline
        {currentTime}
        {duration}
        bouts={video.bouts}
        comments={video.comments}
        {playing}
        {looping}
        {speed}
        {volume}
        onseek={(ms) => player?.seekTo(ms)}
        onloop={({ start, end }) => player?.setLoop(start, end)}
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
