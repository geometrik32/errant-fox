<script lang="ts">
  import { untrack } from 'svelte';
  import { extractFpsFromUrl } from './moov';

  interface Props {
    src: string;
    fps?: number | null;
    speed?: number;
    volume?: number;
    ontimeupdate?: (t: number) => void;
    ondurationchange?: (d: number) => void;
    onplayingchange?: (p: boolean) => void;
    onloopingchange?: (l: boolean) => void;
    ondetectedfps?: (fps: number) => void;
  }

  let {
    src,
    fps = null,
    speed = 1,
    volume = 1,
    ontimeupdate,
    ondurationchange,
    onplayingchange,
    onloopingchange,
    ondetectedfps,
  }: Props = $props();

  let videoEl: HTMLVideoElement;

  /// Effective FPS (integer): API value → moov parser fallback → 30 as last resort.
  let effectiveFps = $state<number>(untrack(() => fps != null && fps > 0 ? Math.round(fps) : 0));

  // Re-sync if parent passes a new non-null fps (e.g. after API re-fetch)
  $effect(() => {
    if (fps != null && fps > 0) effectiveFps = Math.round(fps);
  });

  let loopRange: { start: number; end: number } | null = null;
  let looping = false;
  let zoom = $state(1);
  let panX = $state(0);
  let panY = $state(0);
  let panning = $state(false);
  let panStartX = 0;
  let panStartY = 0;
  let panStartPanX = 0;
  let panStartPanY = 0;

  // Integer frame target during stepping
  let stepFrameTarget: number | null = null;
  let stepRetries = 0;

  // Track middle-click for double-click reset
  let lastMiddleClickTime = 0;

  $effect(() => { if (videoEl) videoEl.playbackRate = speed; });
  $effect(() => { if (videoEl) videoEl.volume = volume; });

  // ── Moov-based FPS detection (fallback when API provides none) ──────────

  let fpsFromMoov = $state<number | null>(null);

  // Kick off FPS extraction from the moov atom on mount (when API has no fps).
  // This parses the first 1 MB of the video file to find the real frame rate.
  function detectFpsFromMoov(): void {
    if (fps != null) return;  // already have FPS from API
    if (fpsFromMoov !== null) return; // already tried
    extractFpsFromUrl(src).then((detected) => {
      if (detected != null && detected > 0) {
        fpsFromMoov = detected;
        effectiveFps = detected;
        ondetectedfps?.(detected);
      } else {
        // Last resort — 30 fps is a safe universal fallback
        fpsFromMoov = 30;
        effectiveFps = 30;
        ondetectedfps?.(30);
      }
    });
  }

  // Run detection when src is set
  $effect(() => {
    if (src) detectFpsFromMoov();
  });

  // ── Exports ──────────────────────────────────────────────────────────────

  export function seekTo(ms: number): void {
    stepFrameTarget = null;
    if (videoEl) videoEl.currentTime = ms / 1000;
  }

  export function pause(): void { videoEl?.pause(); }
  export function play(): void { videoEl?.play(); }

  export function togglePlay(): void {
    if (!videoEl) return;
    videoEl.paused ? videoEl.play() : videoEl.pause();
  }

  export function stepForward(): void {
    if (!videoEl || effectiveFps === 0) return;
    videoEl.pause();
    if (stepFrameTarget === null)
      stepFrameTarget = Math.round(videoEl.currentTime * effectiveFps);
    stepFrameTarget++;
    const t = Math.min(videoEl.duration || 0, stepFrameTarget / effectiveFps);
    videoEl.currentTime = t;
    ontimeupdate?.(t);
    stepRetries = 0;
    correctFrameStep();
  }

  export function stepBackward(): void {
    if (!videoEl || effectiveFps === 0) return;
    videoEl.pause();
    if (stepFrameTarget === null)
      stepFrameTarget = Math.round(videoEl.currentTime * effectiveFps);
    stepFrameTarget = Math.max(0, stepFrameTarget - 1);
    const t = stepFrameTarget / effectiveFps;
    videoEl.currentTime = t;
    ontimeupdate?.(t);
    stepRetries = 0;
    correctFrameStep();
  }

  function correctFrameStep(): void {
    if (!videoEl || stepFrameTarget === null || effectiveFps === 0) return;
    if (!('requestVideoFrameCallback' in videoEl)) return;

    const target = stepFrameTarget;
    videoEl.requestVideoFrameCallback((_now, meta) => {
      if (stepFrameTarget !== target) return;
      const actualFrame = Math.round(meta.mediaTime * effectiveFps);
      if (actualFrame !== target && stepRetries < 1) {
        stepRetries++;
        const t = target / effectiveFps;
        videoEl!.currentTime = t;
        ontimeupdate?.(t);
        correctFrameStep();
      }
    });
  }

  export function setLoop(start: number, end: number, autoPlay = true): void {
    loopRange = { start, end };
    looping = true;
    onloopingchange?.(true);
    seekTo(start);
    if (autoPlay) play();
  }

  export function setSpeed(s: number): void { if (videoEl) videoEl.playbackRate = s; }
  export function setVolume(v: number): void { if (videoEl) videoEl.volume = v; }

  export function toggleLoop(): void {
    looping = !looping;
    if (!looping) loopRange = null;
    onloopingchange?.(looping);
  }

  // ── Event handlers ───────────────────────────────────────────────────────

  function handleTimeUpdate() {
    if (!videoEl) return;
    if (stepFrameTarget === null) ontimeupdate?.(videoEl.currentTime);
    if (looping && loopRange && videoEl.currentTime * 1000 >= loopRange.end) {
      videoEl.currentTime = loopRange.start / 1000;
      stepFrameTarget = null;
    }
  }

  function handleDurationChange() {
    if (videoEl) ondurationchange?.(videoEl.duration);
  }

  function handlePause() { onplayingchange?.(false); }

  function handlePlay() {
    stepFrameTarget = null;
    onplayingchange?.(true);
  }

  let wrapEl: HTMLDivElement | null = $state(null);

  function handleWheel(e: WheelEvent) {
    e.preventDefault();
    const oldZoom = zoom;
    const newZoom = Math.max(1, Math.min(4, zoom + (e.deltaY < 0 ? 0.15 : -0.15)));
    if (newZoom === oldZoom) return;

    if (wrapEl) {
      const r = wrapEl.getBoundingClientRect();
      const cx = e.clientX - r.left;
      const cy = e.clientY - r.top;
      panX = cx - (cx - panX) * newZoom / oldZoom;
      panY = cy - (cy - panY) * newZoom / oldZoom;
    }

    zoom = newZoom;
    if (zoom === 1) { panX = 0; panY = 0; }
  }

  function handleMousedown(e: MouseEvent) {
    if (e.button !== 1) return;
    e.preventDefault();

    const now = Date.now();
    if (now - lastMiddleClickTime < 400) {
      resetZoom();
      lastMiddleClickTime = 0;
      return;
    }
    lastMiddleClickTime = now;

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

<div class="vp-wrap" bind:this={wrapEl}>
  <!-- svelte-ignore a11y_media_has_caption -->
  <video
    bind:this={videoEl}
    {src}
    ontimeupdate={handleTimeUpdate}
    ondurationchange={handleDurationChange}
    onplay={handlePlay}
    onpause={handlePause}
    onclick={handleClick}
    onmousedown={handleMousedown}
    onwheel={handleWheel}
    class="video"
    style:transform="translate({panX}px, {panY}px) scale({zoom})"
    style:transform-origin="0 0"
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
    color: var(--accent-yellow);
    font-size: 0.8rem;
    font-weight: 700;
    padding: 4px 8px;
    border-radius: var(--radius-sm);
    border: none;
    cursor: pointer;
    font-variant-numeric: tabular-nums;
    transition: var(--transition);
  }

  .zoom-badge:hover {
    background: rgba(0, 0, 0, 0.85);
  }
</style>
