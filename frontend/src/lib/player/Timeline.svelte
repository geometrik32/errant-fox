<script lang="ts">
  import type { Bout, Comment, VideoFighter } from '../api/types';
  import { resolveColor } from '../api/types';

  interface Props {
    currentTime: number;
    duration: number;
    bouts?: Bout[];
    comments?: Comment[];
    fighterA?: VideoFighter | null;
    fighterB?: VideoFighter | null;
    playing?: boolean;
    looping?: boolean;
    speed?: number;
    volume?: number;
    onseek?: (timestamp_ms: number) => void;
    onloop?: (range: { start: number; end: number }) => void;
    onplay?: () => void;
    onstepback?: () => void;
    onstepforward?: () => void;
    onspeedchange?: (speed: number) => void;
    onvolumechange?: (volume: number) => void;
    onlooptoggle?: () => void;
  }

  let {
    currentTime,
    duration,
    bouts = [],
    comments = [],
    fighterA = null,
    fighterB = null,
    playing = false,
    looping = false,
    speed = 1,
    volume = 1,
    onseek,
    onloop,
    onplay,
    onstepback,
    onstepforward,
    onspeedchange,
    onvolumechange,
    onlooptoggle,
  }: Props = $props();

  const SPEEDS = [0.15, 0.2, 0.25, 0.5, 0.75, 1.0, 1.25, 1.5, 2.0, 2.5];

  function boutColor(b: Bout): string {
    if (b.score_a > b.score_b) return resolveColor(fighterA?.id ?? 'a', fighterA?.color ?? null);
    if (b.score_b > b.score_a) return resolveColor(fighterB?.id ?? 'b', fighterB?.color ?? null);
    return '#c8d8e8';
  }

  function commentColor(c: Comment): string {
    return resolveColor(c.author.id, (c.author as { color?: string | null }).color ?? null);
  }

  function fmt(sec: number): string {
    const m = Math.floor(sec / 60);
    const s = Math.floor(sec % 60);
    return `${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}`;
  }

  let progressEl: HTMLDivElement | null = $state(null);

  function seekAt(e: MouseEvent, el: HTMLElement) {
    const r = el.getBoundingClientRect();
    const pct = Math.max(0, Math.min(1, (e.clientX - r.left) / r.width));
    onseek?.(pct * duration * 1000);
  }

  function startDrag(e: MouseEvent) {
    if (!progressEl) return;
    seekAt(e, progressEl);
    const el = progressEl;
    const move = (ev: MouseEvent) => seekAt(ev, el);
    const up = () => {
      window.removeEventListener('mousemove', move);
      window.removeEventListener('mouseup', up);
    };
    window.addEventListener('mousemove', move);
    window.addEventListener('mouseup', up);
    e.preventDefault();
  }

  let pct = $derived(duration > 0 ? (currentTime / duration) * 100 : 0);

  const commentPos = (c: Comment) =>
    duration > 0 ? (c.timestamp_ms / 1000 / duration) * 100 : 0;
  const boutL = (b: Bout) =>
    duration > 0 ? (b.time_start_ms / (duration * 1000)) * 100 : 0;
  const boutW = (b: Bout) =>
    duration > 0 ? ((b.time_end_ms - b.time_start_ms) / (duration * 1000)) * 100 : 0;
</script>

<div class="timeline">

  <!-- Row 1: Comment markers -->
  <div class="track track--comments">
    {#each comments as c (c.id)}
      <button
        class="c-dot"
        style="left: {commentPos(c)}%; background: {commentColor(c)}"
        onclick={() => onseek?.(c.timestamp_ms)}
        title={c.text}
        aria-label="Комментарий: {c.text}"
      ></button>
    {/each}
  </div>

  <!-- Row 2: Progress bar -->
  <!-- svelte-ignore a11y_interactive_supports_focus -->
  <div
    class="track track--progress"
    bind:this={progressEl}
    onmousedown={startDrag}
    role="slider"
    aria-valuenow={Math.round(currentTime)}
    aria-valuemin={0}
    aria-valuemax={Math.round(duration)}
    aria-label="Прогресс"
    tabindex="0"
  >
    <div class="prog-fill" style="width: {pct}%"></div>
    <div class="prog-thumb" style="left: {pct}%"></div>
  </div>

  <!-- Row 3: Bout track -->
  <div class="track track--bouts">
    {#each bouts as b, i (b.id)}
      <button
        class="bout-seg"
        style="left: {boutL(b)}%; width: {boutW(b)}%; --color: {boutColor(b)}"
        onclick={() => onloop?.({ start: b.time_start_ms, end: b.time_end_ms })}
        aria-label="Сход {i + 1} — зациклить"
        title="Сход {i + 1}"
      ></button>
    {/each}
  </div>

  <!-- Row 4: Controls -->
  <div class="controls">
    <div class="ctrl-group">

      <button class="ctrl-btn" onclick={onplay} aria-label={playing ? 'Пауза' : 'Воспроизвести'}>
        {#if playing}
          <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
            <rect x="6" y="4" width="4" height="16" rx="1"/>
            <rect x="14" y="4" width="4" height="16" rx="1"/>
          </svg>
        {:else}
          <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
            <polygon points="5,3 19,12 5,21"/>
          </svg>
        {/if}
      </button>

      <button class="ctrl-btn" onclick={onstepback} aria-label="Шаг назад (Z)">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
          <polygon points="18,5 8,12 18,19"/>
          <rect x="5" y="5" width="3" height="14" rx="1"/>
        </svg>
      </button>

      <button class="ctrl-btn" onclick={onstepforward} aria-label="Шаг вперёд (X)">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
          <polygon points="6,5 16,12 6,19"/>
          <rect x="16" y="5" width="3" height="14" rx="1"/>
        </svg>
      </button>

      <button
        class="ctrl-btn loop-btn"
        class:on={looping}
        onclick={onlooptoggle}
        aria-pressed={looping}
        aria-label="Повтор"
      >LOOP</button>

      <select
        class="speed-sel"
        value={SPEEDS.includes(speed) ? speed : 1}
        onchange={(e) => onspeedchange?.(parseFloat((e.target as HTMLSelectElement).value))}
        aria-label="Скорость воспроизведения"
      >
        {#each SPEEDS as s}
          <option value={s}>{s}×</option>
        {/each}
      </select>

    </div>

    <div class="ctrl-group ctrl-group--right">

      <!-- svelte-ignore a11y_no_noninteractive_element_to_interactive_role -->
      <label class="vol-wrap">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true">
          <polygon points="11,5 6,9 2,9 2,15 6,15 11,19" fill="currentColor" stroke="none"/>
          {#if volume > 0.01}
            <path d="M15.54 8.46a5 5 0 0 1 0 7.07"/>
          {/if}
          {#if volume > 0.5}
            <path d="M19.07 4.93a10 10 0 0 1 0 14.14"/>
          {/if}
        </svg>
        <input
          type="range"
          class="vol-slider"
          min="0"
          max="1"
          step="0.02"
          value={volume}
          oninput={(e) => onvolumechange?.(parseFloat((e.target as HTMLInputElement).value))}
          aria-label="Громкость"
        />
      </label>

      <time class="time-disp" datetime="PT{Math.round(currentTime)}S">
        {fmt(currentTime)} / {fmt(duration)}
      </time>

    </div>
  </div>

</div>

<style>
  .timeline {
    display: flex;
    flex-direction: column;
    width: 100%;
    background: #060e1a;
    border-top: 1px solid #1a3050;
    user-select: none;
  }

  /* ── Shared track base ── */
  .track {
    position: relative;
    width: 100%;
    flex-shrink: 0;
  }

  /* ── Row 1: Comment markers ── */
  .track--comments {
    height: 18px;
    background: #08101f;
    border-bottom: 1px solid #0d1e35;
  }

  .c-dot {
    position: absolute;
    top: 50%;
    transform: translate(-50%, -50%);
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: #52b4e0;
    border: none;
    cursor: pointer;
    padding: 0;
    transition: transform 0.1s, background 0.1s;
  }

  .c-dot:hover {
    transform: translate(-50%, -50%) scale(1.7);
    background: #7dd0f5;
  }

  /* ── Row 2: Progress bar ── */
  .track--progress {
    height: 7px;
    background: #0d1e35;
    cursor: pointer;
  }

  .track--progress:hover .prog-thumb {
    opacity: 1;
    transform: translate(-50%, -50%) scale(1);
  }

  .prog-fill {
    position: absolute;
    inset: 0;
    height: 100%;
    background: #DB841F;
    pointer-events: none;
  }

  .prog-thumb {
    position: absolute;
    top: 50%;
    width: 14px;
    height: 14px;
    border-radius: 50%;
    background: #fff;
    border: 2px solid #DB841F;
    transform: translate(-50%, -50%) scale(0.6);
    opacity: 0.6;
    pointer-events: none;
    transition: transform 0.1s, opacity 0.1s;
  }

  /* ── Row 3: Bout track ── */
  .track--bouts {
    height: 22px;
    background: #08101f;
    border-top: 1px solid #0d1e35;
    border-bottom: 1px solid #0d1e35;
  }

  .bout-seg {
    position: absolute;
    top: 3px;
    height: calc(100% - 6px);
    min-width: 3px;
    background-color: var(--color);
    border: none;
    border-radius: 2px;
    cursor: pointer;
    padding: 0;
    opacity: 0.7;
    transition: opacity 0.12s;
  }

  .bout-seg:hover {
    opacity: 1;
  }

  /* ── Row 4: Controls ── */
  .controls {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: 44px;
    padding: 0 10px;
    background: #060e1a;
  }

  .ctrl-group {
    display: flex;
    align-items: center;
    gap: 2px;
  }

  .ctrl-group--right {
    gap: 12px;
  }

  .ctrl-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 30px;
    height: 30px;
    border: none;
    border-radius: 5px;
    background: transparent;
    color: #5a7a96;
    cursor: pointer;
    transition: background 0.12s, color 0.12s;
    flex-shrink: 0;
  }

  .ctrl-btn:hover {
    background: #0f2035;
    color: #d0dde8;
  }

  .loop-btn {
    width: auto;
    padding: 0 10px;
    font-size: 0.68rem;
    font-weight: 700;
    letter-spacing: 0.08em;
    color: #3a5470;
    border: 1px solid #152840;
  }

  .loop-btn.on {
    color: #DB841F;
    border-color: rgba(219, 132, 31, 0.5);
    background: rgba(219, 132, 31, 0.1);
  }

  .speed-sel {
    background: #060e1a;
    border: 1px solid #152840;
    border-radius: 5px;
    color: #7090a8;
    font-size: 0.78rem;
    padding: 4px 6px;
    cursor: pointer;
    outline: none;
    transition: border-color 0.12s, color 0.12s;
    margin-left: 4px;
  }

  .speed-sel:hover,
  .speed-sel:focus {
    border-color: #2a4f73;
    color: #d0dde8;
  }

  /* Volume */
  .vol-wrap {
    display: flex;
    align-items: center;
    gap: 6px;
    color: #5a7a96;
    cursor: pointer;
  }

  .vol-slider {
    -webkit-appearance: none;
    appearance: none;
    width: 72px;
    height: 4px;
    background: #152840;
    border-radius: 2px;
    outline: none;
    cursor: pointer;
  }

  .vol-slider::-webkit-slider-thumb {
    -webkit-appearance: none;
    width: 12px;
    height: 12px;
    border-radius: 50%;
    background: #DB841F;
    cursor: pointer;
  }

  .vol-slider::-moz-range-thumb {
    width: 12px;
    height: 12px;
    border-radius: 50%;
    background: #DB841F;
    cursor: pointer;
    border: none;
  }

  /* Time display */
  .time-disp {
    font-size: 0.78rem;
    font-variant-numeric: tabular-nums;
    color: #5a7a96;
    white-space: nowrap;
    min-width: 96px;
    text-align: right;
  }
</style>
