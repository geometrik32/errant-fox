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
    onboutclick?: (boutId: number) => void;
    oncommentclick?: (commentId: number) => void;
    onplay?: () => void;
    onstepback?: () => void;
    onstepforward?: () => void;
    onspeedchange?: (speed: number) => void;
    onvolumechange?: (volume: number) => void;
    onlooptoggle?: () => void;
    fps?: number | null;
    startTime?: number | null;
    finishing?: boolean;
    onstartclick?: () => void;
    onfinishclick?: () => void;
    onshare?: () => void;
    readonly?: boolean;
  }

  let {
    currentTime = 0,
    duration = 0,
    bouts = [],
    comments = [],
    fighterA = null,
    fighterB = null,
    playing = false,
    looping = false,
    speed = 1,
    volume = 1,
    fps = 25,
    startTime = null,
    finishing = false,
    readonly = false,
    onseek,
    onloop,
    onboutclick,
    oncommentclick,
    onplay,
    onstepback,
    onstepforward,
    onspeedchange,
    onvolumechange,
    onlooptoggle,
    onstartclick,
    onfinishclick,
    onshare,
  }: Props = $props();

  const SPEEDS = [0.15, 0.2, 0.25, 0.5, 0.75, 1.0, 1.25, 1.5, 2.0, 2.5];

  let preMuteVolume = $state(1);

  function toggleMute() {
    if (volume > 0.01) {
      preMuteVolume = volume;
      onvolumechange?.(0);
    } else {
      onvolumechange?.(preMuteVolume > 0.01 ? preMuteVolume : 1);
    }
  }

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

  function fmtSec(sec: number): string {
    const h = Math.floor(sec / 3600);
    const m = Math.floor((sec % 3600) / 60);
    const s = Math.floor(sec % 60);
    return h > 0
      ? `${String(h).padStart(2, '0')}:${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}`
      : `${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}`;
  }

  /// Format current time with 1-based frame number: MM:SS:FFF
  /// Frames run from 1 to fps, then the next second starts.
  /// The integer frame rate, rounded from potentially fractional fps (e.g. NTSC 29.97 → 30).
  const fpsInt = $derived(fps ? Math.round(fps) : 0);

  function fmtWithFrame(sec: number): string {
    if (!fpsInt) return fmt(sec);
    const totalFrames = Math.round(sec * fpsInt);
    const framesInSec = totalFrames % fpsInt;           // 0-based within second
    const totalSecs = Math.floor(totalFrames / fpsInt);
    const m = Math.floor(totalSecs / 60);
    const s = totalSecs % 60;
    const padLen = String(fpsInt).length < 3 ? 3 : String(fpsInt).length;
    // 1-based: frame 0 → display "001", frame (fpsInt-1) → display fpsInt
    return `${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}:${String(framesInSec + 1).padStart(padLen, '0')}`;
  }

  /// Format duration showing the last valid frame (1-based within its second).
  function fmtDuration(sec: number): string {
    if (!fpsInt) return fmt(sec);
    const totalFrames = Math.round(sec * fpsInt);
    if (totalFrames === 0) return fmt(sec);
    const lastFrameIdx = totalFrames - 1;
    const totalSecs = Math.floor(lastFrameIdx / fpsInt);
    const framesInSec = lastFrameIdx % fpsInt;
    const m = Math.floor(totalSecs / 60);
    const s = totalSecs % 60;
    const padLen = String(fpsInt).length < 3 ? 3 : String(fpsInt).length;
    return `${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}:${String(framesInSec + 1).padStart(padLen, '0')}`;
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

  let pct = $derived.by(() => {
    if (!duration || duration <= 0) return 0;
    const val = (currentTime / duration) * 100;
    return Math.max(0, Math.min(100, val));
  });

  const isInsideBout = $derived(
    bouts.some(b => {
      const currentMs = currentTime * 1000;
      return currentMs >= b.time_start_ms && currentMs <= b.time_end_ms;
    })
  );

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
        onclick={() => { onseek?.(c.timestamp_ms); oncommentclick?.(c.id); }}
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
  {#if !readonly}
    <div class="track track--bouts">
      {#each bouts as b, i (b.id)}
        <button
          class="bout-seg"
          class:is-ai={b.is_ai}
          style="left: {boutL(b)}%; width: {boutW(b)}%; --color: {boutColor(b)}"
          onclick={() => { onloop?.({ start: b.time_start_ms, end: b.time_end_ms }); onboutclick?.(b.id); }}
          aria-label="Сход {i + 1} — зациклить"
          title="Сход {i + 1}"
        ></button>
      {/each}
    </div>
  {/if}

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
        disabled={!looping}
        aria-pressed={looping}
        aria-label="Повтор (D)"
        title={looping ? "Отключить повтор (D)" : "Повтор (D)"}
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

    <div class="ctrl-group ctrl-group--center">
      {#if !readonly}
        <button
          class="ctrl-btn start-btn"
          class:start-btn--active={startTime !== null}
          onclick={onstartclick}
          disabled={isInsideBout}
          aria-label="Зафиксировать начало схода"
          style="background: {startTime !== null ? '#0f4020' : '#1a6b35'}; border-color: {startTime !== null ? '#1a8040' : '#27ae60'}; color: {startTime !== null ? '#3bc266' : '#52d47a'};"
        >
          {#if startTime !== null}
            {fmtSec(startTime)}
          {:else}
            START
          {/if}
        </button>

        <button
          class="ctrl-btn finish-btn"
          disabled={startTime === null || finishing}
          onclick={onfinishclick}
          aria-label="Зафиксировать конец схода"
          style="color: #e05252; border-color: #ae2727; background: rgba(174, 39, 39, 0.1);"
        >
          {finishing ? '…' : 'FINISH'}
        </button>
      {/if}
    </div>

    <div class="ctrl-group ctrl-group--right">

      <div class="vol-wrap">
        <button class="vol-icon-btn" onclick={toggleMute} aria-label={volume > 0.01 ? 'Выключить звук' : 'Включить звук'}>
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true">
            <polygon points="11,5 6,9 2,9 2,15 6,15 11,19" fill="currentColor" stroke="none"/>
            {#if volume > 0.01}
              <path d="M15.54 8.46a5 5 0 0 1 0 7.07"/>
            {/if}
            {#if volume > 0.5}
              <path d="M19.07 4.93a10 10 0 0 1 0 14.14"/>
            {/if}
          </svg>
        </button>
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
      </div>

      {#if onshare}
        <button class="ctrl-btn" onclick={onshare} aria-label="Поделиться видео" title="Поделиться видео">
          <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
            <circle cx="18" cy="5" r="3" />
            <circle cx="6" cy="12" r="3" />
            <circle cx="18" cy="19" r="3" />
            <line x1="8.59" y1="13.51" x2="15.42" y2="17.49" />
            <line x1="15.41" y1="6.51" x2="8.59" y2="10.49" />
          </svg>
        </button>
      {/if}

      <time class="time-disp" datetime="PT{Math.round(currentTime)}S">
        {fmtWithFrame(currentTime)} / {fmtDuration(duration)}
      </time>

    </div>
  </div>

</div>

<style>
  .timeline {
    display: flex;
    flex-direction: column;
    width: 100%;
    background: transparent;
    user-select: none;
    padding: 0 20px;
    box-sizing: border-box;
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
    background: transparent;
    border-bottom: 1px solid var(--border-color);
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
    height: 8px;
    background: var(--surface-hover);
    cursor: pointer;
  }

  .track--progress:hover .prog-thumb {
    transform: translate(-50%, -50%) scale(1.25);
    box-shadow: 0 0 10px rgba(245, 158, 11, 0.9), 0 2px 5px rgba(0, 0, 0, 0.6);
  }

  .prog-fill {
    position: absolute;
    left: 0;
    top: 0;
    height: 100%;
    background: var(--accent-yellow);
    pointer-events: none;
  }

  .prog-thumb {
    position: absolute;
    top: 50%;
    width: 14px;
    height: 14px;
    border-radius: 50%;
    background: #ffffff;
    border: 2px solid var(--accent-yellow);
    box-shadow: 0 0 6px rgba(245, 158, 11, 0.6), 0 1px 3px rgba(0, 0, 0, 0.5);
    transform: translate(-50%, -50%) scale(1);
    opacity: 1;
    pointer-events: none;
    transition: transform 0.1s ease, box-shadow 0.1s ease;
    z-index: 5;
  }

  /* ── Row 3: Bout track ── */
  .track--bouts {
    height: 24px;
    background: transparent;
    border-top: 1px solid var(--border-color);
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

  .bout-seg.is-ai {
    background-color: #8b5cf6 !important;
    animation: ai-timeline-pulse 4s ease-in-out infinite alternate;
    opacity: 0.9 !important;
    box-shadow: 0 0 6px rgba(139, 92, 246, 0.4);
  }

  @keyframes ai-timeline-pulse {
    0% {
      background-color: #8b5cf6 !important;
      box-shadow: 0 0 6px rgba(139, 92, 246, 0.4);
    }
    100% {
      background-color: #10b981 !important;
      box-shadow: 0 0 6px rgba(16, 185, 129, 0.4);
    }
  }

  /* ── Row 4: Controls ── */
  .controls {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: 48px;
    padding: 0 16px;
    background: transparent;
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
    width: 36px;
    height: 36px;
    border: none;
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--text-secondary);
    cursor: pointer;
    transition: var(--transition);
    flex-shrink: 0;
  }

  .ctrl-btn:hover {
    background: var(--surface-hover);
    color: var(--text-primary);
  }

  .loop-btn {
    width: auto;
    padding: 0 12px;
    font-size: 0.8rem;
    font-weight: 700;
    letter-spacing: 0.08em;
    color: var(--text-secondary);
    border: 1px solid var(--border-color);
  }

  .loop-btn.on {
    color: var(--accent-yellow);
    border-color: rgba(219, 132, 31, 0.5);
    background: rgba(219, 132, 31, 0.1);
  }

  .loop-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
    background: transparent;
    color: var(--text-secondary);
    border-color: var(--border-color);
    pointer-events: auto;
  }

  .ctrl-group--center {
    gap: 8px;
  }

  .start-btn,
  .finish-btn {
    width: 90px;
    padding: 0;
    font-size: 0.8rem;
    font-weight: 700;
    letter-spacing: 0.08em;
    border: 1px solid transparent;
    height: 28px;
    border-radius: var(--radius-sm);
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .start-btn {
    gap: 6px;
    transition: background 0.15s, border-color 0.15s, color 0.15s;
  }

  .start-btn:disabled {
    opacity: 0.35;
    cursor: not-allowed;
    background: rgba(26, 107, 53, 0.04) !important;
    border-color: rgba(26, 107, 53, 0.2) !important;
    color: rgba(82, 212, 122, 0.5) !important;
  }



  .finish-btn {
    transition: background 0.15s, border-color 0.15s, color 0.15s;
  }

  .finish-btn:disabled {
    opacity: 0.35;
    cursor: not-allowed;
    background: rgba(174, 39, 39, 0.04) !important;
    border-color: rgba(174, 39, 39, 0.2) !important;
    color: rgba(224, 82, 82, 0.5) !important;
  }

  .speed-sel {
    background: transparent;
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm);
    color: var(--text-secondary);
    font-size: 0.85rem;
    padding: 6px 10px;
    cursor: pointer;
    outline: none;
    transition: var(--transition);
    margin-left: 8px;
  }

  .speed-sel:hover,
  .speed-sel:focus {
    border-color: var(--text-primary);
    color: var(--text-primary);
  }

  .speed-sel option {
    background: var(--surface-solid);
    color: var(--text-primary);
  }

  /* Volume */
  .vol-wrap {
    display: flex;
    align-items: center;
    gap: 8px;
    color: var(--text-secondary);
  }

  .vol-icon-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    background: none;
    border: none;
    color: var(--text-secondary);
    cursor: pointer;
    padding: 4px;
    border-radius: var(--radius-sm);
    transition: var(--transition);
    flex-shrink: 0;
  }

  .vol-icon-btn:hover {
    color: var(--text-primary);
    background: var(--surface-hover);
  }

  .vol-slider {
    -webkit-appearance: none;
    appearance: none;
    width: 80px;
    height: 6px;
    background: var(--surface-hover);
    border: 1px solid var(--border-color);
    border-radius: 3px;
    outline: none;
    cursor: pointer;
  }

  .vol-slider::-webkit-slider-thumb {
    -webkit-appearance: none;
    width: 14px;
    height: 14px;
    border-radius: 50%;
    background: var(--accent-yellow);
    cursor: pointer;
  }

  .vol-slider::-moz-range-thumb {
    width: 14px;
    height: 14px;
    border-radius: 50%;
    background: var(--accent-yellow);
    cursor: pointer;
    border: none;
  }

  /* Time display */
  .time-disp {
    font-size: 0.85rem;
    font-variant-numeric: tabular-nums;
    color: var(--text-secondary);
    white-space: nowrap;
    min-width: 130px;
    text-align: right;
  }
</style>
