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
    onboutschange?: (bouts: Bout[]) => void;
  }

  let { video, currentTime, onboutschange }: Props = $props();

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
  }

  function handleBoutDelete(id: number) {
    bouts = bouts.filter(b => b.id !== id);
    if (expandedBoutId === id) expandedBoutId = null;
    onboutschange?.(bouts);
  }

  // ── Derived lists & scores ───────────────────────────────────────────────

  let sortedBouts = $derived([...bouts].sort((a, b) => a.order_index - b.order_index));
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
      <div class="fighter-slot">
        <span
          class="fighter-dot"
          style:background={activeFighterA ? resolveColor(activeFighterA.id, activeFighterA.color) : '#6fa0e0'}
        ></span>
        <select class="fighter-sel" bind:value={fighterAId} onchange={saveFighters} aria-label="Боец A">
          <option value="">— Боец A —</option>
          {#each $fighters as f (f.id)}
            <option value={f.id}>{f.display_name}</option>
          {/each}
        </select>
      </div>
      <div class="fighter-slot">
        <span
          class="fighter-dot"
          style:background={activeFighterB ? resolveColor(activeFighterB.id, activeFighterB.color) : '#e08080'}
        ></span>
        <select class="fighter-sel" bind:value={fighterBId} onchange={saveFighters} aria-label="Боец B">
          <option value="">— Боец B —</option>
          {#each $fighters as f (f.id)}
            <option value={f.id}>{f.display_name}</option>
          {/each}
        </select>
      </div>
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
  <div class="bouts-list">
    {#each sortedBouts as bout (bout.id)}
      <BoutCard
        {bout}
        fighters={[activeFighterA, activeFighterB]}
        {currentTime}
        expanded={expandedBoutId === bout.id}
        onexpand={() => handleExpand(bout.id)}
        oncollapse={handleCollapse}
        onmarkdirty={(d) => handleMarkDirty(bout.id, d)}
        onupdate={handleBoutUpdate}
        ondelete={() => handleBoutDelete(bout.id)}
      />
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
    display: flex;
    align-items: center;
    gap: 5px;
    min-width: 0;
  }

  .fighter-dot {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .fighter-sel {
    flex: 1;
    background: #0d1e35;
    border: 1px solid #1a3050;
    border-radius: 4px;
    color: #a0b4c8;
    font-size: 0.78rem;
    padding: 4px 6px;
    outline: none;
    cursor: pointer;
    transition: border-color 0.1s;
  }

  .fighter-sel:hover,
  .fighter-sel:focus {
    border-color: #2a4f73;
    color: #d0dde8;
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
