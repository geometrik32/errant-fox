<script lang="ts">
  import { onMount, onDestroy, tick, untrack } from 'svelte';
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
    onpauserequest?: () => void;
    onmarkingchange?: (active: boolean) => void;
    onmarkingfinish?: () => void;
    onboutdelete?: () => void;
    startTime?: number | null;
    finishing?: boolean;
  }

  let {
    video,
    currentTime,
    playing = false,
    onboutschange,
    onseekrequest,
    onpauserequest,
    onmarkingchange,
    onmarkingfinish,
    onboutdelete,
    startTime = $bindable(null),
    finishing = $bindable(false),
  }: Props = $props();

  // ── Local bouts state ────────────────────────────────────────────────────

  let bouts = $state<Bout[]>([...untrack(() => video.bouts)]);

  // ── Fighter assignment ───────────────────────────────────────────────────

  let fighterAId = $state<string>(untrack(() => video.fighter_a?.id ?? ''));
  let fighterBId = $state<string>(untrack(() => video.fighter_b?.id ?? ''));

  let activeFighterA = $derived<VideoFighter | null>(
    $fighters.find(f => f.id === fighterAId) as VideoFighter | null ?? null
  );
  let activeFighterB = $derived<VideoFighter | null>(
    $fighters.find(f => f.id === fighterBId) as VideoFighter | null ?? null
  );
  let footerFighterA = $derived(activeFighterA ?? video.fighter_a);
  let footerFighterB = $derived(activeFighterB ?? video.fighter_b);
  let footerFighterAColor = $derived(
    footerFighterA ? resolveColor(footerFighterA.id, footerFighterA.color) : '#6fa0e0'
  );
  let footerFighterBColor = $derived(
    footerFighterB ? resolveColor(footerFighterB.id, footerFighterB.color) : '#e08080'
  );
  let footerFighterALabel = $derived(footerFighterA?.display_name ?? 'Боец A');
  let footerFighterBLabel = $derived(footerFighterB?.display_name ?? 'Боец B');

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

  let finishError = $state<string | null>(null);

  export function handleStart() {
    if (startTime !== null) {
      startTime = null;
      onmarkingchange?.(false);
    } else {
      startTime = currentTime;
      finishError = null;
      onmarkingchange?.(true);
    }
  }

  export function triggerMark(): void {
    if (startTime === null) {
      handleStart();
    } else {
      void handleFinish();
    }
  }

  export async function handleFinish() {
    if (startTime === null) return;
    onpauserequest?.();
    finishing = true;
    finishError = null;
    try {
      const created = await createBout({
        video_id: video.id,
        time_start_ms: Math.round(startTime * 1000),
        time_end_ms: Math.round(currentTime * 1000),
      });
      const exists = bouts.some(b => b.id === created.id);
      if (!exists) {
        bouts = [...bouts, created];
        onboutschange?.(bouts);
      }
      startTime = null;
      onmarkingchange?.(false);
      onmarkingfinish?.();
      expandedBoutId = created.id;
      await tick();
      requestAnimationFrame(() => scrollBoutToTop(created.id, 'smooth'));

      setTimeout(() => {
        onseekrequest?.(created.time_start_ms, created.time_end_ms);
      }, 300);
    } catch (e) {
      finishError = e instanceof Error ? e.message : 'Ошибка создания схода';
    } finally {
      finishing = false;
    }
  }

  // ── Expand / collapse ────────────────────────────────────────────────────

  let expandedBoutId = $state<number | null>(null);
  let dirtyBoutIds = $state(new Set<number>());

  async function handleExpand(id: number) {
    if (expandedBoutId !== null && expandedBoutId !== id && dirtyBoutIds.has(expandedBoutId)) {
      if (!confirm('Есть несохранённые изменения. Свернуть текущую карточку?')) return;
    }
    expandedBoutId = id;
    const b = bouts.find(b => b.id === id);
    if (b) onseekrequest?.(b.time_start_ms, b.time_end_ms);
    await tick();
    requestAnimationFrame(() => scrollBoutToTop(id, 'smooth'));
  }

  export async function expandBout(id: number) {
    expandedBoutId = id;
    await tick();
    requestAnimationFrame(() => scrollBoutToTop(id, 'smooth'));
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
    onboutdelete?.();
  }

  let boutsListEl: HTMLDivElement | null = $state(null);

  function scrollBoutToTop(id: number, behavior: ScrollBehavior = 'smooth') {
    const list = boutsListEl;
    if (!list) return;

    const el = list.querySelector<HTMLElement>(`[data-bout-id="${id}"]`);
    if (!el) return;

    const listRect = list.getBoundingClientRect();
    const elRect = el.getBoundingClientRect();
    const elementTop = list.scrollTop + elRect.top - listRect.top;

    const nextScrollTop = elementTop - 8;
    const maxScrollTop = Math.max(0, list.scrollHeight - list.clientHeight);
    list.scrollTo({
      top: Math.min(maxScrollTop, Math.max(0, nextScrollTop)),
      behavior,
    });
  }

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
            onboutdelete?.();
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
    if (!target.closest('.footer-fighter-select')) openDropdown = null;
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
    <div class="footer-fighter-select">
      <button
        class="footer-avatar"
        style:background={footerFighterAColor}
        style:border-color={footerFighterAColor}
        title={footerFighterALabel}
        aria-label={`Выбрать ${footerFighterALabel}`}
        aria-expanded={openDropdown === 'a'}
        onclick={(e) => { e.stopPropagation(); toggleDropdown('a'); }}
      >
        <svg class="footer-avatar-icon" width="16" height="16" viewBox="0 0 24 24" fill="none" aria-hidden="true">
          <circle cx="12" cy="8" r="4" stroke="currentColor" stroke-width="1.5" />
          <path d="M4 20c1.5-4 4.5-6 8-6s6.5 2 8 6" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
        </svg>
        {#if footerFighterA?.avatar_url}
          <img src={footerFighterA.avatar_url} alt="" onerror={(e) => { (e.target as HTMLImageElement).style.display = 'none'; }} />
        {/if}
      </button>

      {#if openDropdown === 'a'}
        <div class="fighter-dropdown">
          <button class="fighter-opt" onclick={() => selectFighter('a', '')}>
            <span class="fighter-opt-avatar unselected"></span>
            <span class="fighter-opt-name">Не выбран</span>
          </button>
          {#each $fighters as f (f.id)}
            {@const optColor = resolveColor(f.id, f.color)}
            <button
              class="fighter-opt"
              class:selected={fighterAId === f.id}
              onclick={() => selectFighter('a', f.id)}
            >
              <span class="fighter-opt-avatar" style:background={optColor} style:border-color={optColor}>
                <svg class="fighter-opt-icon" width="13" height="13" viewBox="0 0 24 24" fill="none" aria-hidden="true">
                  <circle cx="12" cy="8" r="4" stroke="currentColor" stroke-width="1.5" />
                  <path d="M4 20c1.5-4 4.5-6 8-6s6.5 2 8 6" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
                </svg>
                {#if f.avatar_url}
                  <img src={f.avatar_url} alt="" onerror={(e) => { (e.target as HTMLImageElement).style.display = 'none'; }} />
                {/if}
              </span>
              <span class="fighter-opt-name">{f.display_name}</span>
            </button>
          {/each}
        </div>
      {/if}
    </div>

    <div class="footer-score-wrap" aria-label="Total score">
      <span class="footer-label">TOTAL SCORE</span>
      <span class="footer-score">{totalScoreA} : {totalScoreB}</span>
    </div>

    <div class="footer-fighter-select align-right">
      <button
        class="footer-avatar"
        style:background={footerFighterBColor}
        style:border-color={footerFighterBColor}
        title={footerFighterBLabel}
        aria-label={`Выбрать ${footerFighterBLabel}`}
        aria-expanded={openDropdown === 'b'}
        onclick={(e) => { e.stopPropagation(); toggleDropdown('b'); }}
      >
        <svg class="footer-avatar-icon" width="16" height="16" viewBox="0 0 24 24" fill="none" aria-hidden="true">
          <circle cx="12" cy="8" r="4" stroke="currentColor" stroke-width="1.5" />
          <path d="M4 20c1.5-4 4.5-6 8-6s6.5 2 8 6" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
        </svg>
        {#if footerFighterB?.avatar_url}
          <img src={footerFighterB.avatar_url} alt="" onerror={(e) => { (e.target as HTMLImageElement).style.display = 'none'; }} />
        {/if}
      </button>

      {#if openDropdown === 'b'}
        <div class="fighter-dropdown align-right">
          <button class="fighter-opt" onclick={() => selectFighter('b', '')}>
            <span class="fighter-opt-avatar unselected"></span>
            <span class="fighter-opt-name">Не выбран</span>
          </button>
          {#each $fighters as f (f.id)}
            {@const optColor = resolveColor(f.id, f.color)}
            <button
              class="fighter-opt"
              class:selected={fighterBId === f.id}
              onclick={() => selectFighter('b', f.id)}
            >
              <span class="fighter-opt-avatar" style:background={optColor} style:border-color={optColor}>
                <svg class="fighter-opt-icon" width="13" height="13" viewBox="0 0 24 24" fill="none" aria-hidden="true">
                  <circle cx="12" cy="8" r="4" stroke="currentColor" stroke-width="1.5" />
                  <path d="M4 20c1.5-4 4.5-6 8-6s6.5 2 8 6" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
                </svg>
                {#if f.avatar_url}
                  <img src={f.avatar_url} alt="" onerror={(e) => { (e.target as HTMLImageElement).style.display = 'none'; }} />
                {/if}
              </span>
              <span class="fighter-opt-name">{f.display_name}</span>
            </button>
          {/each}
        </div>
      {/if}
    </div>
  </div>

</div>

<style>
  .panel {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: transparent;
    overflow: hidden;
  }

  .finish-error {
    margin: 0 12px 8px;
    padding: 8px 12px;
    background: rgba(239, 68, 68, 0.1);
    border: 1px solid rgba(239, 68, 68, 0.2);
    border-radius: var(--radius-sm);
    font-size: 0.8rem;
    color: #ef4444;
  }

  /* ── Bouts list ─────────────────────────────────────────────────────────── */
  .bouts-list {
    flex: 1;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 12px;
  }

  .empty {
    font-size: 0.78rem;
    color: #3a5470;
    text-align: center;
    padding: 24px 0;
  }

  /* ── Footer ─────────────────────────────────────────────────────────────── */
  .footer {
    display: grid;
    grid-template-columns: 38px 1fr 38px;
    align-items: center;
    gap: 12px;
    padding: 10px 16px;
    border-top: 1px solid var(--border-color);
    background: transparent;
    flex-shrink: 0;
  }

  .footer-fighter-select {
    position: relative;
    width: 38px;
    height: 38px;
  }

  .footer-fighter-select.align-right {
    justify-self: end;
  }

  .footer-avatar {
    position: relative;
    display: grid;
    place-items: center;
    width: 38px;
    height: 38px;
    border-radius: 50%;
    border: 1px solid currentColor;
    overflow: hidden;
    padding: 0;
    color: rgba(255, 255, 255, 0.7);
    cursor: pointer;
    transition: transform 0.15s ease, filter 0.15s ease;
  }

  .footer-avatar:hover { filter: brightness(1.12); transform: translateY(-1px); }

  .footer-avatar-icon {
    position: absolute;
    opacity: 0.7;
    pointer-events: none;
  }

  .footer-avatar img {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .fighter-dropdown {
    position: absolute;
    bottom: calc(100% + 8px);
    left: 0;
    width: 220px;
    max-height: 280px;
    overflow-y: auto;
    padding: 4px;
    background: var(--surface-solid);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm);
    z-index: 50;
    box-shadow: var(--shadow-md);
  }

  .fighter-dropdown.align-right {
    left: auto;
    right: 0;
  }

  .fighter-opt {
    display: flex;
    align-items: center;
    gap: 10px;
    width: 100%;
    min-width: 0;
    padding: 7px 8px;
    background: transparent;
    border: none;
    border-radius: 4px;
    color: var(--text-secondary);
    font-size: 0.84rem;
    cursor: pointer;
    text-align: left;
    transition: var(--transition);
  }

  .fighter-opt:hover { background: var(--surface-hover); color: var(--text-primary); }
  .fighter-opt.selected { background: rgba(219, 132, 31, 0.12); color: var(--accent-yellow); }

  .fighter-opt-avatar {
    position: relative;
    display: grid;
    place-items: center;
    width: 26px;
    height: 26px;
    border: 1px solid currentColor;
    border-radius: 50%;
    overflow: hidden;
    flex-shrink: 0;
    color: rgba(255, 255, 255, 0.68);
  }

  .fighter-opt-avatar.unselected {
    background: transparent;
    border-color: #3a5470;
  }

  .fighter-opt-icon {
    position: absolute;
    opacity: 0.7;
    pointer-events: none;
  }

  .fighter-opt-avatar img {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .fighter-opt-name {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .footer-score-wrap {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    min-width: 0;
  }

  .footer-label {
    font-size: 0.75rem;
    font-weight: 700;
    letter-spacing: 0.1em;
    color: var(--text-secondary);
    text-transform: uppercase;
  }

  .footer-score {
    font-size: 1.2rem;
    font-weight: 700;
    color: var(--accent-yellow);
    font-variant-numeric: tabular-nums;
    line-height: 1.2;
  }
</style>
