<script lang="ts">
  import { type Snippet } from 'svelte';

  interface Props {
    left?: Snippet;
    center?: Snippet;
    right?: Snippet;
    activePanel?: number; // 0: left, 1: center, 2: right
    locked?: boolean;
    onpanelchange?: (panel: number) => void;
  }

  let {
    left,
    center,
    right,
    activePanel = $bindable(1),
    locked = false,
    onpanelchange,
  }: Props = $props();

  let touchStartX = 0;
  let touchStartY = 0;
  let currentTouchX = 0;
  let isSwiping = $state(false);
  let dragOffset = $state(0);
  let showEdgeHints = $state(true);

  let hasCenter = $derived(center !== undefined);
  let maxPanel = $derived(hasCenter ? 2 : 1);

  function handleTouchStart(e: TouchEvent) {
    if (locked || e.touches.length > 1) return;
    const target = e.target as HTMLElement;
    if (target.closest('.track--progress, .drawing-canvas, .drawing-toolbar, input, textarea, select')) {
      return;
    }

    touchStartX = e.touches[0].clientX;
    touchStartY = e.touches[0].clientY;
    currentTouchX = touchStartX;
    isSwiping = true;
    dragOffset = 0;

    window.addEventListener('touchmove', handleWindowTouchMove, { passive: false });
    window.addEventListener('touchend', handleWindowTouchEnd);
    window.addEventListener('touchcancel', handleWindowTouchEnd);
  }

  function handleWindowTouchMove(e: TouchEvent) {
    if (!isSwiping || locked || e.touches.length > 1) return;

    const currentX = e.touches[0].clientX;
    const currentY = e.touches[0].clientY;
    const deltaX = currentX - touchStartX;
    const deltaY = currentY - touchStartY;

    // Horizontal check
    if (Math.abs(deltaX) > Math.abs(deltaY) && Math.abs(deltaX) > 10) {
      if (e.cancelable) e.preventDefault();
      // Apply drag offset with resistance at edges
      if ((activePanel === 0 && deltaX > 0) || (activePanel === maxPanel && deltaX < 0)) {
        dragOffset = deltaX * 0.3;
      } else {
        dragOffset = deltaX;
      }
      currentTouchX = currentX;
    }
  }

  function handleWindowTouchEnd() {
    window.removeEventListener('touchmove', handleWindowTouchMove);
    window.removeEventListener('touchend', handleWindowTouchEnd);
    window.removeEventListener('touchcancel', handleWindowTouchEnd);

    if (!isSwiping) return;
    isSwiping = false;

    const threshold = window.innerWidth * 0.2; // 20% width to trigger page change
    if (dragOffset < -threshold && activePanel < maxPanel) {
      activePanel += 1;
      onpanelchange?.(activePanel);
      showEdgeHints = false;
    } else if (dragOffset > threshold && activePanel > 0) {
      activePanel -= 1;
      onpanelchange?.(activePanel);
      showEdgeHints = false;
    }

    dragOffset = 0;
  }

  function goToPanel(idx: number) {
    activePanel = idx;
    onpanelchange?.(idx);
    showEdgeHints = false;
  }
</script>

<div
  class="swipe-container"
  ontouchstart={handleTouchStart}
>
  <div
    class="swipe-track"
    class:animating={!isSwiping}
    style="transform: translate3d(calc(-{activePanel * 100}vw + {dragOffset}px), 0, 0); width: {hasCenter ? '300%' : '200%'};"
  >
    <div class="swipe-panel">
      {#if left}{@render left()}{:else}<div class="panel-placeholder">Судейство</div>{/if}
    </div>

    {#if center}
      <div class="swipe-panel">
        {@render center()}
      </div>
    {/if}

    <div class="swipe-panel">
      {#if right}{@render right()}{:else}<div class="panel-placeholder">Чат</div>{/if}
    </div>
  </div>

  <!-- Indicator bar -->
  <div class="swipe-indicators" ontouchstart={(e) => e.stopPropagation()}>
    <button class="nav-arrow-btn" class:disabled={activePanel === 0} onclick={(e) => { e.stopPropagation(); goToPanel(activePanel - 1); }}>
      ◄
    </button>

    <div class="dots">
      <button
        class="dot"
        class:active={activePanel === 0}
        onclick={(e) => { e.stopPropagation(); goToPanel(0); }}
        aria-label="Судейство"
      >
        <span class="dot-label">Судейство</span>
      </button>
      {#if hasCenter}
        <button
          class="dot"
          class:active={activePanel === 1}
          onclick={(e) => { e.stopPropagation(); goToPanel(1); }}
          aria-label="Плеер"
        >
          <span class="dot-label">Плеер</span>
        </button>
      {/if}
      <button
        class="dot"
        class:active={activePanel === maxPanel}
        onclick={(e) => { e.stopPropagation(); goToPanel(maxPanel); }}
        aria-label="Чат"
      >
        <span class="dot-label">Чат</span>
      </button>
    </div>

    <button class="nav-arrow-btn" class:disabled={activePanel === maxPanel} onclick={(e) => { e.stopPropagation(); goToPanel(activePanel + 1); }}>
      ►
    </button>
  </div>

  <!-- Edge Hints when on central panel -->
  {#if activePanel === 1 && showEdgeHints}
    <div class="edge-hint edge-hint-left">
      <span>◄ Судейство</span>
    </div>
    <div class="edge-hint edge-hint-right">
      <span>Чат ►</span>
    </div>
  {/if}
</div>

<style>
  .swipe-container {
    position: relative;
    width: 100vw;
    height: 100%;
    overflow: hidden;
    display: flex;
    flex-direction: column;
    background: var(--bg-base);
    touch-action: pan-y;
  }

  .swipe-track {
    display: flex;
    height: calc(100% - 40px);
    will-change: transform;
  }

  .swipe-track.animating {
    transition: transform 0.3s cubic-bezier(0.25, 1, 0.5, 1);
  }

  .swipe-panel {
    width: 100vw;
    height: 100%;
    flex-shrink: 0;
    overflow-y: auto;
    -webkit-overflow-scrolling: touch;
    scrollbar-width: none;
    -ms-overflow-style: none;
  }

  .swipe-panel::-webkit-scrollbar {
    display: none;
    width: 0;
    height: 0;
  }

  .panel-placeholder {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: var(--text-muted);
  }

  /* Bottom indicator bar */
  .swipe-indicators {
    height: 40px;
    background: rgba(15, 23, 42, 0.95);
    border-top: 1px solid var(--border-color);
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 12px;
    z-index: 20;
    flex-shrink: 0;
  }

  .nav-arrow-btn {
    background: transparent;
    border: none;
    color: var(--accent-yellow);
    font-size: 0.9rem;
    padding: 6px 12px;
    cursor: pointer;
    min-width: 44px;
    min-height: 44px;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .nav-arrow-btn.disabled {
    opacity: 0.2;
    pointer-events: none;
  }

  .dots {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .dot {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 4px 10px;
    border: 1px solid var(--border-color);
    border-radius: var(--radius-pill);
    background: rgba(255, 255, 255, 0.05);
    color: var(--text-secondary);
    font-size: 0.75rem;
    cursor: pointer;
    transition: var(--transition);
    min-height: 32px;
  }

  .dot.active {
    background: var(--accent-yellow);
    color: #000;
    font-weight: 600;
    border-color: var(--accent-yellow);
  }

  .dot-label {
    white-space: nowrap;
  }

  /* Edge Hints */
  .edge-hint {
    position: absolute;
    top: 30%;
    transform: translateY(-50%);
    background: rgba(31, 41, 55, 0.85);
    border: 1px solid var(--border-strong);
    color: var(--text-primary);
    padding: 6px 10px;
    font-size: 0.75rem;
    border-radius: var(--radius-pill);
    pointer-events: none;
    z-index: 15;
    animation: pulseHint 2s infinite ease-in-out;
  }

  .edge-hint-left {
    left: 8px;
  }

  .edge-hint-right {
    right: 8px;
  }

  @keyframes pulseHint {
    0%, 100% { opacity: 0.4; }
    50% { opacity: 0.85; }
  }
</style>
