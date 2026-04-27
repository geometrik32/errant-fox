<script lang="ts">
  interface Props {
    src: string;
    speed?: number;
    volume?: number;
    ontimeupdate?: (t: number) => void;
    ondurationchange?: (d: number) => void;
    onplayingchange?: (p: boolean) => void;
    onloopingchange?: (l: boolean) => void;
    onfpschange?: (fps: number) => void;
  }

  let {
    src,
    speed = 1,
    volume = 1,
    ontimeupdate,
    ondurationchange,
    onplayingchange,
    onloopingchange,
    onfpschange,
  }: Props = $props();

  let videoEl: HTMLVideoElement;
  let loopRange: { start: number; end: number } | null = null;
  let looping = false;
  let zoom = $state(1);
  let originX = $state(50);
  let originY = $state(50);

  // Pan state (middle-mouse drag)
  let panX = $state(0);
  let panY = $state(0);
  let panning = $state(false);
  let panStartX = 0;
  let panStartY = 0;
  let panStartPanX = 0;
  let panStartPanY = 0;

  $effect(() => { if (videoEl) videoEl.playbackRate = speed; });
  $effect(() => { if (videoEl) videoEl.volume = volume; });

  export function seekTo(ms: number): void {
    if (videoEl) videoEl.currentTime = ms / 1000;
  }

  export function pause(): void { videoEl?.pause(); }
  export function play(): void { videoEl?.play(); }

  export function togglePlay(): void {
    if (!videoEl) return;
    videoEl.paused ? videoEl.play() : videoEl.pause();
  }

  export function stepForward(): void {
    if (!videoEl) return;
    videoEl.currentTime = Math.min(videoEl.duration || 0, videoEl.currentTime + 1 / 30);
  }

  export function stepBackward(): void {
    if (!videoEl) return;
    videoEl.currentTime = Math.max(0, videoEl.currentTime - 1 / 30);
  }

  export function setLoop(start: number, end: number): void {
    loopRange = { start, end };
    looping = true;
    onloopingchange?.(true);
    seekTo(start);
    play();
  }

  export function setSpeed(s: number): void { if (videoEl) videoEl.playbackRate = s; }
  export function setVolume(v: number): void { if (videoEl) videoEl.volume = v; }

  export function toggleLoop(): void {
    looping = !looping;
    if (!looping) loopRange = null;
    onloopingchange?.(looping);
  }

  function handleTimeUpdate() {
    if (!videoEl) return;
    ontimeupdate?.(videoEl.currentTime);
    if (looping && loopRange && videoEl.currentTime * 1000 >= loopRange.end) {
      videoEl.currentTime = loopRange.start / 1000;
    }
  }

  function handleDurationChange() {
    if (videoEl) ondurationchange?.(videoEl.duration);
  }

  function handlePause() { onplayingchange?.(false); }

  // FPS detection — measure first 16 frame intervals after play
  let fpsSamples: number[] = [];
  let lastMediaTime = -1;

  function fpsCallback(_now: number, meta: { mediaTime: number }): void {
    if (lastMediaTime >= 0) {
      const diff = meta.mediaTime - lastMediaTime;
      if (diff > 0 && diff < 0.5) fpsSamples.push(diff);
    }
    lastMediaTime = meta.mediaTime;

    if (fpsSamples.length < 16) {
      videoEl?.requestVideoFrameCallback(fpsCallback);
    } else {
      const avg = fpsSamples.reduce((a, b) => a + b) / fpsSamples.length;
      const fps = Math.round(1 / avg);
      onfpschange?.(fps);
      fpsSamples = [];
      lastMediaTime = -1;
    }
  }

  function handlePlayForFps() {
    onplayingchange?.(true);
    if (fpsSamples.length === 0 && videoEl) {
      videoEl.requestVideoFrameCallback(fpsCallback);
    }
  }

  function handleWheel(e: WheelEvent) {
    e.preventDefault();
    const prevZoom = zoom;
    zoom = Math.max(1, Math.min(4, zoom + (e.deltaY < 0 ? 0.1 : -0.1)));

    if (videoEl) {
      const r = videoEl.getBoundingClientRect();
      originX = ((e.clientX - r.left) / r.width) * 100;
      originY = ((e.clientY - r.top) / r.height) * 100;
    }

    // Reset pan when fully zoomed out
    if (zoom === 1) { panX = 0; panY = 0; }

    void prevZoom;
  }

  function handleMousedown(e: MouseEvent) {
    // Middle mouse button = button 1
    if (e.button !== 1) return;
    e.preventDefault();
    panning = true;
    panStartX = e.clientX;
    panStartY = e.clientY;
    panStartPanX = panX;
    panStartPanY = panY;

    function onMove(ev: MouseEvent) {
      if (!panning) return;
      panX = panStartPanX + ev.clientX - panStartX;
      panY = panStartPanY + ev.clientY - panStartY;
    }

    function onUp() {
      panning = false;
      window.removeEventListener('mousemove', onMove);
      window.removeEventListener('mouseup', onUp);
    }

    window.addEventListener('mousemove', onMove);
    window.addEventListener('mouseup', onUp);
  }

  function handleClick(e: MouseEvent) {
    if (e.button !== 0) return;
    togglePlay();
  }

  function resetZoom() {
    zoom = 1;
    panX = 0;
    panY = 0;
  }
</script>

<div class="vp-wrap">
  <!-- svelte-ignore a11y_media_has_caption -->
  <video
    bind:this={videoEl}
    {src}
    ontimeupdate={handleTimeUpdate}
    ondurationchange={handleDurationChange}
    onplay={handlePlayForFps}
    onpause={handlePause}
    onclick={handleClick}
    ondblclick={resetZoom}
    onmousedown={handleMousedown}
    onwheel={handleWheel}
    class="video"
    style:transform="translate({panX}px, {panY}px) scale({zoom})"
    style:transform-origin="{originX}% {originY}%"
    style:cursor={panning ? 'grabbing' : zoom > 1.05 ? 'grab' : 'pointer'}
    preload="metadata"
  ></video>

  {#if zoom > 1.05}
    <button class="zoom-badge" onclick={resetZoom}>{zoom.toFixed(1)}×</button>
  {/if}
</div>

<style>
  .vp-wrap {
    position: relative;
    width: 100%;
    height: 100%;
    background: #000;
    overflow: hidden;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .video {
    width: 100%;
    height: 100%;
    object-fit: contain;
    display: block;
    will-change: transform;
  }

  .zoom-badge {
    position: absolute;
    top: 10px;
    right: 10px;
    background: rgba(0, 0, 0, 0.6);
    color: #DB841F;
    font-size: 0.78rem;
    font-weight: 700;
    padding: 3px 8px;
    border-radius: 4px;
    border: none;
    cursor: pointer;
    font-variant-numeric: tabular-nums;
    transition: background 0.12s;
  }

  .zoom-badge:hover {
    background: rgba(0, 0, 0, 0.85);
  }
</style>
