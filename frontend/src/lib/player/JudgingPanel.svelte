<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { fighters } from '../../stores';
  import type { VideoFull, Bout, VideoFighter } from '../api/types';
  import { resolveColor } from '../api/types';
  import { createBout } from '../api/bouts';
  import { patchVideo } from '../api/videos';
  import BoutCard from './BoutCard.svelte';

  interface Props {
    video: VideoFull;
    currentTime: number;
    playing?: boolean;
    onboutschange?: (bouts: Bout[]) => void;
    onseekrequest?: (ms: number, endMs: number) => void;
  }

  let { video, currentTime, playing = false, onboutschange, onseekrequest }: Props = $props();

  // ── Local bouts state ────────────────────────────────────────────────────

  let bouts = $state<Bout[]>([...video.bouts]);

  // ── Fighter assignment ───────────────────────────────────────────────────

  let fighterAId = $state<string>(video.fighter_a?.id ?? '');
  let fighterBId = $state<string>(video.fighter_b?.id ?? '');

  let activeFighterA = $derived<VideoFighter | null>(
    $fighters.find(f => f.id === fighterAId) as VideoFighter | null ?? null
  );
  let activeFighterB = $derived<VideoFighter | null>(
    $fighters.find(f => f.id === fighterBId) as VideoFighter | null ?? null
  );

  // ── Fighter save ─────────────────────────────────────────────────────────

  async function saveFighters() {
    try {
      await patchVideo(video.id, {
        fighter_a_id: fighterAId || null,
        fighter_b_id: fighterBId || null,
      });
    } catch (e) {
      console.error('Failed to save fighters:', e);
    }
  }

  // ── START / FINISH ───────────────────────────────────────────────────────

  let startTime = $state<number | null>(null);
  let finishing = $state(false);
  let finishError = $state<string | null>(null);

  export function triggerMark(): void {
    if (startTime === null) {
      startTime = currentTime;
      finishError = null;
    } else {
      handleFinish();
    }
  }

  async function handleFinish() {
    if (startTime === null) return;
    finishing = true;
    finishError = null;
    try {
      const created = await createBout({
        video_id: video.id,
        time_start_ms: Math.round(startTime * 1000),
        time_end_ms: Math.round(currentTime * 1000),
      });
      bouts = [...bouts, created];
      onboutschange?.(bouts);
      startTime = null;
      expandedBoutId = created.id;
      requestAnimationFrame(() => {
        if (boutsListEl) boutsListEl.scrollTop = boutsListEl.scrollHeight;
      });
    } catch (e) {
      finishError = e instanceof Error ? e.message : 'Ошибка создания схода';
    } finally {
      finishing = false;
    }
  }

  // ── Expand / collapse ────────────────────────────────────────────────────

  let expandedBoutId = $state<number | null>(null);
  let dirtyBoutIds = $state(new Set<number>());

  function handleExpand(id: number) {
    if (expandedBoutId !== null && expandedBoutId !== id && dirtyBoutIds.has(expandedBoutId)) {
      if (!confirm('Есть несохранённые изменения. Свернуть текущую карточку?')) return;
    }
    expandedBoutId = id;
    const b = bouts.find(b => b.id === id);
    if (b) onseekrequest?.(b.time_start_ms, b.time_end_ms);
  }

  export function expandBout(id: number) {
    expandedBoutId = id;
    const el = boutsListEl?.querySelector(`[data-bout-id="${id}"]`);
    el?.scrollIntoView({ block: 'nearest', behavior: 'smooth' });
  }

  function handleCollapse() {
    expandedBoutId = null;
  }

  function handleMarkDirty(id: number, dirty: boolean) {
    const next = new Set(dirtyBoutIds);
    if (dirty) next.add(id); else next.delete(id);
    dirtyBoutIds = next;
  }

  function handleBoutUpdate(updated: Bout) {
    bouts = bouts.map(b => b.id === updated.id ? updated : b);
    onboutschange?.(bouts);
  }

  function handleBoutDelete(id: number) {
    bouts = bouts.filter(b => b.id !== id);
    if (expandedBoutId === id) expandedBoutId = null;
    onboutschange?.(bouts);
  }

  let boutsListEl: HTMLDivElement | null = $state(null);

  // ── Derived lists & scores ───────────────────────────────────────────────

  let sortedBouts = $derived([...bouts].sort((a, b) => a.time_start_ms - b.time_start_ms));
  let totalScoreA = $derived(bouts.reduce((s, b) => s + b.score_a, 0));
  let totalScoreB = $derived(bouts.reduce((s, b) => s + b.score_b, 0));

  // ── WebSocket ────────────────────────────────────────────────────────────

  let ws: WebSocket | null = null;

  function connectWS() {
    ws = new WebSocket('ws://localhost:8080/ws');

    ws.onopen = () => {
      const token = localStorage.getItem('ef_token');
      if (!token) return;
      ws!.send(JSON.stringify({ token }));
      ws!.send(JSON.stringify({ watching: video.id }));
    };

    ws.onmessage = (e) => {
      try {
        const msg = JSON.parse(e.data as string) as Record<string, unknown>;
        if (msg.type === 'update_bout') {
          const { type: _t, video_id: _v, ...fields } = msg;
          const id = fields.id as number;

          if (fields.deleted) {
            bouts = bouts.filter(b => b.id !== id);
            if (expandedBoutId === id) expandedBoutId = null;
          } else {
            const incoming = fields as unknown as Bout;
            const idx = bouts.findIndex(b => b.id === id);
            bouts = idx >= 0
              ? bouts.map((b, i) => i === idx ? incoming : b)
              : [...bouts, incoming];
          }
          onboutschange?.(bouts);
        }
      } catch { /* ignore malformed */ }
    };

    ws.onclose = () => {
      setTimeout(() => { if (ws !== null) connectWS(); }, 4000);
    };
  }

  onMount(connectWS);

  onDestroy(() => {
    const w = ws;
    ws = null;
    w?.close();
  });

  // ── Custom fighter dropdowns ─────────────────────────────────────────────

  let openDropdown = $state<'a' | 'b' | null>(null);

  function toggleDropdown(slot: 'a' | 'b') {
    openDropdown = openDropdown === slot ? null : slot;
  }

  function selectFighter(slot: 'a' | 'b', id: string) {
    if (slot === 'a') fighterAId = id;
    else fighterBId = id;
    openDropdown = null;
    saveFighters();
  }

  function handleDropdownKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') openDropdown = null;
  }

  function handleDropdownClickOutside(e: MouseEvent) {
    const target = e.target as HTMLElement;
    if (!target.closest('.fighter-slot')) openDropdown = null;
  }

  $effect(() => {
    if (openDropdown !== null) {
      document.addEventListener('click', handleDropdownClickOutside);
      document.addEventListener('keydown', handleDropdownKeydown);
      return () => {
        document.removeEventListener('click', handleDropdownClickOutside);
        document.removeEventListener('keydown', handleDropdownKeydown);
      };
    }
  });

  // ── Helpers ──────────────────────────────────────────────────────────────

  function fmtSec(sec: number): string {
    const h = Math.floor(sec / 3600);
    const m = Math.floor((sec % 3600) / 60);
    const s = Math.floor(sec % 60);
    return h > 0
      ? `${String(h).padStart(2, '0')}:${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}`
      : `${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}`;
  }
</script>

<div class="panel">

  <!-- ── Fighter selects ──────────────────────────────────────────────────── -->
  <div class="fighters-section">
    <div class="fighters-row">
      {#each (['a', 'b'] as const) as slot}
        {@const activeF = slot === 'a' ? activeFighterA : activeFighterB}
        {@const defaultColor = slot === 'a' ? '#6fa0e0' : '#e08080'}
        {@const dotColor = activeF ? resolveColor(activeF.id, activeF.color) : defaultColor}
        {@const label = slot === 'a' ? '— Боец A —' : '— Боец B —'}
        <div class="fighter-slot">
          <button
            class="fighter-btn"
            onclick={(e) => { e.stopPropagation(); toggleDropdown(slot); }}
            aria-label={label}
          >
            <span class="fighter-dot" style:background={dotColor}></span>
            <span class="fighter-btn-name">{activeF?.display_name ?? label}</span>
            <svg class="fighter-chevron" width="10" height="10" viewBox="0 0 24 24" fill="none">
              <path d="M6 9l6 6 6-6" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
            </svg>
          </button>
          {#if openDropdown === slot}
            <div class="fighter-dropdown">
              <button class="fighter-opt" onclick={() => selectFighter(slot, '')}>
                <span class="fighter-opt-dot" style:background="transparent; border: 1px solid #3a5470"></span>
                <span>— Не выбран —</span>
              </button>
              {#each $fighters as f (f.id)}
                <button
                  class="fighter-opt"
                  class:selected={slot === 'a' ? fighterAId === f.id : fighterBId === f.id}
                  onclick={() => selectFighter(slot, f.id)}
                >
                  <span class="fighter-opt-dot" style:background={resolveColor(f.id, f.color)}></span>
                  <span>{f.display_name}</span>
                </button>
              {/each}
            </div>
          {/if}
        </div>
      {/each}
    </div>
  </div>

  <!-- ── Controls ─────────────────────────────────────────────────────────── -->
  <div class="controls">
    <button
      class="btn-start"
      class:btn-start--active={startTime !== null}
      onclick={() => { startTime = currentTime; finishError = null; }}
      aria-label="Зафиксировать начало схода"
    >
      START
      {#if startTime !== null}
        <span class="start-hint">@{fmtSec(startTime)}</span>
      {/if}
    </button>

    <button
      class="btn-finish"
      disabled={startTime === null || finishing}
      onclick={handleFinish}
      aria-label="Зафиксировать конец схода"
    >
      {finishing ? '…' : 'FINISH'}
    </button>
  </div>

  {#if finishError}
    <div class="finish-error">{finishError}</div>
  {/if}

  <!-- ── Bouts list ────────────────────────────────────────────────────────── -->
  <div class="bouts-list" bind:this={boutsListEl}>
    {#each sortedBouts as bout, i (bout.id)}
      <div data-bout-id={bout.id}>
        <BoutCard
          {bout}
          boutIndex={i + 1}
          fighters={[activeFighterA, activeFighterB]}
          {currentTime}
          expanded={expandedBoutId === bout.id}
          onexpand={() => handleExpand(bout.id)}
          oncollapse={handleCollapse}
          onmarkdirty={(d) => handleMarkDirty(bout.id, d)}
          onupdate={handleBoutUpdate}
          ondelete={() => handleBoutDelete(bout.id)}
        />
      </div>
    {:else}
      <div class="empty">Нет сходов</div>
    {/each}
  </div>

  <!-- ── Footer ───────────────────────────────────────────────────────────── -->
  <div class="footer">
    <span class="footer-label">TOTAL SCORE</span>
    <span class="footer-score">{totalScoreA} : {totalScoreB}</span>
  </div>

</div>

<style>
  .panel {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: #08101f;
    border-right: 1px solid #1a3050;
    overflow: hidden;
  }

  /* ── Fighter selects ────────────────────────────────────────────────────── */
  .fighters-section {
    padding: 8px 10px;
    border-bottom: 1px solid #1a3050;
  }

  .fighters-row {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 6px;
  }

  .fighter-slot {
    position: relative;
    flex: 1;
    min-width: 0;
  }

  .fighter-dot {
    width: 9px;
    height: 9px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .fighter-btn {
    display: flex;
    align-items: center;
    gap: 5px;
    width: 100%;
    background: #0d1e35;
    border: 1px solid #1a3050;
    border-radius: 4px;
    color: #a0b4c8;
    font-size: 0.78rem;
    padding: 4px 6px;
    cursor: pointer;
    transition: border-color 0.1s;
    text-align: left;
  }

  .fighter-btn:hover { border-color: #2a4f73; color: #d0dde8; }

  .fighter-btn-name {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .fighter-chevron {
    flex-shrink: 0;
    color: #3a5470;
  }

  .fighter-dropdown {
    position: absolute;
    top: calc(100% + 3px);
    left: 0;
    right: 0;
    background: #0d1e35;
    border: 1px solid #2a4f73;
    border-radius: 5px;
    z-index: 50;
    overflow: hidden;
    box-shadow: 0 6px 18px rgba(0,0,0,0.4);
  }

  .fighter-opt {
    display: flex;
    align-items: center;
    gap: 6px;
    width: 100%;
    padding: 6px 8px;
    background: none;
    border: none;
    color: #a0b4c8;
    font-size: 0.78rem;
    cursor: pointer;
    text-align: left;
    transition: background 0.1s, color 0.1s;
  }

  .fighter-opt:hover { background: #1a3050; color: #d0dde8; }
  .fighter-opt.selected { background: rgba(219,132,31,0.12); color: #DB841F; }

  .fighter-opt-dot {
    width: 9px;
    height: 9px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  /* ── Controls ───────────────────────────────────────────────────────────── */
  .controls {
    display: flex;
    gap: 6px;
    padding: 8px 10px;
    border-bottom: 1px solid #1a3050;
  }

  .btn-start,
  .btn-finish {
    flex: 1;
    padding: 8px 6px;
    border-radius: 5px;
    font-size: 0.8rem;
    font-weight: 700;
    letter-spacing: 0.06em;
    cursor: pointer;
    border: none;
    transition: background 0.12s, opacity 0.12s;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 2px;
  }

  .btn-start {
    background: #1a6b35;
    color: #52d47a;
    border: 1px solid #27ae60;
  }

  .btn-start:hover {
    background: #1f7d3e;
  }

  .btn-start--active {
    background: #0f4020;
    border-color: #1a8040;
    color: #3bc266;
  }

  .start-hint {
    font-size: 0.65rem;
    font-weight: 400;
    color: #3bc266;
    letter-spacing: 0;
  }

  .btn-finish {
    background: #6b1a1a;
    color: #e05252;
    border: 1px solid #ae2727;
  }

  .btn-finish:hover:not(:disabled) {
    background: #7d1f1f;
  }

  .btn-finish:disabled {
    opacity: 0.35;
    cursor: default;
  }

  .finish-error {
    margin: 0 10px 6px;
    padding: 5px 8px;
    background: rgba(224, 82, 82, 0.1);
    border: 1px solid rgba(224, 82, 82, 0.3);
    border-radius: 4px;
    font-size: 0.72rem;
    color: #e08080;
  }

  /* ── Bouts list ─────────────────────────────────────────────────────────── */
  .bouts-list {
    flex: 1;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 4px;
    padding: 8px;
    scrollbar-width: thin;
    scrollbar-color: #1a3050 transparent;
  }

  .bouts-list::-webkit-scrollbar {
    width: 4px;
  }

  .bouts-list::-webkit-scrollbar-track {
    background: transparent;
  }

  .bouts-list::-webkit-scrollbar-thumb {
    background: #1a3050;
    border-radius: 2px;
  }

  .empty {
    font-size: 0.78rem;
    color: #3a5470;
    text-align: center;
    padding: 24px 0;
  }

  /* ── Footer ─────────────────────────────────────────────────────────────── */
  .footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 12px;
    border-top: 1px solid #1a3050;
    background: #060e1a;
    flex-shrink: 0;
  }

  .footer-label {
    font-size: 0.68rem;
    font-weight: 700;
    letter-spacing: 0.1em;
    color: #4a6280;
    text-transform: uppercase;
  }

  .footer-score {
    font-size: 1rem;
    font-weight: 700;
    color: #DB841F;
    font-variant-numeric: tabular-nums;
  }
</style>
