<script lang="ts">
  import { onDestroy, untrack } from 'svelte';
  import { extractFpsFromUrl } from './moov';

  interface Props {
    src: string;
    fps?: number | null;
    speed?: number;
    volume?: number;
    judgingOpen?: boolean;
    chatOpen?: boolean;
    markingActive?: boolean;
    markingFinishAnimationKey?: number;
    activeViewers?: any[];
    ontimeupdate?: (t: number) => void;
    ondurationchange?: (d: number) => void;
    onplayingchange?: (p: boolean) => void;
    onloopingchange?: (l: boolean) => void;
    ondetectedfps?: (fps: number) => void;
    ontogglejudging?: () => void;
    ontogglechat?: () => void;
  }

  let {
    src,
    fps = null,
    speed = 1,
    volume = 1,
    judgingOpen = true,
    chatOpen = true,
    markingActive = false,
    markingFinishAnimationKey = 0,
    activeViewers = [],
    ontimeupdate,
    ondurationchange,
    onplayingchange,
    onloopingchange,
    ondetectedfps,
    ontogglejudging,
    ontogglechat,
  }: Props = $props();

  let videoEl: HTMLVideoElement;

  /// Effective FPS (integer): API value → moov parser fallback → 30 as last resort.
  let effectiveFps = $state<number>(untrack(() => fps != null && fps > 0 ? Math.round(fps) : 0));

  // Re-sync if parent passes a new non-null fps (e.g. after API re-fetch)
  $effect(() => {
    if (fps != null && fps > 0) effectiveFps = Math.round(fps);
  });

  let loopRange = $state<{ start: number; end: number } | null>(null);
  let looping = $state(false);
  let finishOutlineActive = $state(false);
  let finishOutlineTimer: number | null = null;
  let lastFinishAnimationKey = untrack(() => markingFinishAnimationKey);
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

  let outlineMode = $derived<'marking' | 'finish' | 'loop' | null>(
    markingActive
      ? 'marking'
      : finishOutlineActive
        ? 'finish'
        : looping
          ? 'loop'
          : null
  );

  function clearFinishOutlineTimer(): void {
    if (finishOutlineTimer === null) return;
    window.clearTimeout(finishOutlineTimer);
    finishOutlineTimer = null;
  }

  $effect(() => {
    const key = markingFinishAnimationKey;
    if (key === lastFinishAnimationKey) return;
    lastFinishAnimationKey = key;
    if (key <= 0) return;

    clearFinishOutlineTimer();
    finishOutlineActive = true;
    finishOutlineTimer = window.setTimeout(() => {
      finishOutlineActive = false;
      finishOutlineTimer = null;
    }, 720);
  });

  onDestroy(() => {
    clearFinishOutlineTimer();
  });

  // ── Moov-based FPS detection (fallback when API provides none) ──────────

  let fpsFromMoov = $state<number | null>(null);
  let pendingSeekTimeMs = $state<number | null>(null);

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
    if (looping && loopRange) {
      if (ms < loopRange.start || ms > loopRange.end) {
        looping = false;
        loopRange = null;
        onloopingchange?.(false);
      }
    }
    if (videoEl && videoEl.readyState >= 1) {
      videoEl.currentTime = ms / 1000;
    } else {
      pendingSeekTimeMs = ms;
    }
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
    if (looping) {
      looping = false;
      loopRange = null;
      onloopingchange?.(false);
    }
  }

  // ── Event handlers ───────────────────────────────────────────────────────

  function handleTimeUpdate() {
    if (!videoEl) return;
    if (looping && loopRange && videoEl.currentTime * 1000 >= loopRange.end) {
      videoEl.currentTime = loopRange.start / 1000;
      stepFrameTarget = null;
      ontimeupdate?.(videoEl.currentTime);
      return;
    }
    if (stepFrameTarget === null) ontimeupdate?.(videoEl.currentTime);
  }

  function handleDurationChange() {
    if (videoEl) {
      ondurationchange?.(videoEl.duration);
      if (pendingSeekTimeMs !== null && videoEl.readyState >= 1) {
        videoEl.currentTime = pendingSeekTimeMs / 1000;
        pendingSeekTimeMs = null;
      }
    }
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
  {#if activeViewers.length > 0}
    <div class="viewers-bar">
      {#each activeViewers as viewer (viewer.id)}
        <div class="viewer-avatar" style="--user-color: {viewer.color}" title={viewer.display_name}>
          <svg class="avatar-icon" width="10" height="10" viewBox="0 0 24 24" fill="none">
            <circle cx="12" cy="8" r="4" stroke="currentColor" stroke-width="1.5"/>
            <path d="M4 20c0-4 3.6-7 8-7s8 3 8 7" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
          </svg>
          {#if viewer.avatar_url}
            <img src={viewer.avatar_url} alt="" onerror={(e) => { (e.target as HTMLImageElement).style.display = 'none'; }} />
          {/if}
        </div>
      {/each}
    </div>
  {/if}

  <!-- svelte-ignore a11y_media_has_caption -->
  <video
    bind:this={videoEl}
    {src}
    ontimeupdate={handleTimeUpdate}
    ondurationchange={handleDurationChange}
    onloadedmetadata={handleDurationChange}
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
    playsinline
  ></video>

  {#if outlineMode}
    <div
      class="video-outline"
      class:video-outline--loop={outlineMode === 'loop'}
      class:video-outline--marking={outlineMode === 'marking'}
      class:video-outline--finish={outlineMode === 'finish'}
      aria-hidden="true"
    ></div>
  {/if}

  <button
    class="panel-dot panel-dot-left"
    class:active={judgingOpen}
    type="button"
    aria-label={judgingOpen ? 'Скрыть панель действий' : 'Показать панель действий'}
    aria-pressed={judgingOpen}
    title={judgingOpen ? 'Скрыть панель действий' : 'Показать панель действий'}
    onclick={(e) => { e.stopPropagation(); ontogglejudging?.(); }}
  ></button>

  <button
    class="panel-dot panel-dot-right"
    class:active={chatOpen}
    type="button"
    aria-label={chatOpen ? 'Скрыть чат' : 'Показать чат'}
    aria-pressed={chatOpen}
    title={chatOpen ? 'Скрыть чат' : 'Показать чат'}
    onclick={(e) => { e.stopPropagation(); ontogglechat?.(); }}
  ></button>

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
    border-radius: inherit;
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

  .video-outline {
    position: absolute;
    inset: 0;
    pointer-events: none;
    border: 1px solid currentColor;
    border-radius: inherit;
    z-index: 4;
  }

  .video-outline--loop {
    color: var(--accent-yellow);
    box-shadow:
      0 0 6px rgba(245, 158, 11, 0.45),
      inset 0 0 5px rgba(245, 158, 11, 0.18);
  }

  .video-outline--marking {
    color: var(--accent-green);
    box-shadow:
      0 0 6px rgba(16, 185, 129, 0.45),
      inset 0 0 5px rgba(16, 185, 129, 0.18);
  }

  .video-outline--finish {
    color: var(--accent-red);
    box-shadow:
      0 0 8px rgba(224, 82, 82, 0.5),
      inset 0 0 5px rgba(224, 82, 82, 0.18);
    animation: finish-outline-fade 0.72s ease-out forwards;
  }

  @keyframes finish-outline-fade {
    to {
      opacity: 0;
    }
  }

  .panel-dot {
    position: absolute;
    top: 10px;
    width: 10px;
    height: 10px;
    padding: 0;
    border: none;
    border-radius: 50%;
    background: #6b7280;
    cursor: pointer;
    z-index: 6;
    transition: background-color 0.18s ease, opacity 0.18s ease;
  }

  .panel-dot:hover {
    opacity: 0.86;
  }

  .panel-dot:focus-visible {
    outline: 2px solid var(--text-primary);
    outline-offset: 2px;
  }

  .panel-dot.active {
    background: var(--accent-yellow);
  }

  .panel-dot-left {
    left: 10px;
  }

  .panel-dot-right {
    right: 10px;
  }

  .zoom-badge {
    position: absolute;
    top: 34px;
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
    z-index: 6;
  }

  .zoom-badge:hover {
    background: rgba(0, 0, 0, 0.85);
  }

  .viewers-bar {
    position: absolute;
    top: 14px;
    left: 50%;
    transform: translateX(-50%);
    display: flex;
    gap: 6px;
    background: rgba(15, 23, 42, 0.65);
    border: 1px solid rgba(255, 255, 255, 0.08);
    padding: 5px;
    border-radius: 20px;
    z-index: 10;
  }

  .viewer-avatar {
    width: 24px;
    height: 24px;
    border-radius: 50%;
    background: var(--user-color, #4a6280);
    border: 1px solid rgba(255, 255, 255, 0.15);
    overflow: hidden;
    display: flex;
    align-items: center;
    justify-content: center;
    color: #fff;
    flex-shrink: 0;
    position: relative;
    transition: transform 0.15s ease;
    cursor: pointer;
  }

  .viewer-avatar:hover {
    transform: scale(1.12);
    z-index: 15;
  }

  .viewer-avatar .avatar-icon {
    position: absolute;
  }

  .viewer-avatar img {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    object-fit: cover;
  }
</style>
