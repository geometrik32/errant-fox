<script lang="ts">
  import { untrack } from 'svelte';
  import { techniques } from '../../stores';
  import type { Bout, VideoFighter } from '../api/types';
  import { resolveColor } from '../api/types';
  import { updateBout } from '../api/bouts';
  import HitZonePicker from './HitZonePicker.svelte';
  import { deleteBout } from '../api/bouts';

  type ResultType = 'hit' | 'miss' | 'blocked' | 'late' | 'no_strike' | 'disqualification' | 'afterblow';

  interface Props {
    bout: Bout;
    boutIndex: number;
    fighters: [VideoFighter | null, VideoFighter | null];
    expanded: boolean;
    currentTime: number;
    onexpand: () => void;
    oncollapse: () => void;
    onmarkdirty: (dirty: boolean) => void;
    onupdate: (updated: Bout) => void;
    ondelete?: () => void;
  }

  let { bout, boutIndex, fighters, expanded, currentTime, onexpand, oncollapse, onmarkdirty, onupdate, ondelete }: Props = $props();

  let winnerColor = $derived(
    bout.score_a > bout.score_b
      ? resolveColor(fighters[0]?.id ?? 'a', fighters[0]?.color ?? null)
      : bout.score_b > bout.score_a
        ? resolveColor(fighters[1]?.id ?? 'b', fighters[1]?.color ?? null)
        : null
  );

  // ── Form state ──────────────────────────────────────────────────────────────

  let timeStartMs = $state(bout.time_start_ms);
  let timeEndMs   = $state(bout.time_end_ms);
  let scoreA   = $state(bout.score_a);
  let scoreB   = $state(bout.score_b);
  let techAId  = $state<number | null>(bout.technique_a_id);
  let techBId  = $state<number | null>(bout.technique_b_id);
  let zoneA    = $state<string>(bout.hit_zone_a ?? '');
  let zoneB    = $state<string>(bout.hit_zone_b ?? '');
  let resA     = $state<ResultType>((bout.result_a as ResultType | null) ?? 'hit');
  let resB     = $state<ResultType>((bout.result_b as ResultType | null) ?? 'hit');

  // ── Committed snapshot (last saved) ─────────────────────────────────────────

  let cTimeStart = $state(bout.time_start_ms);
  let cTimeEnd   = $state(bout.time_end_ms);
  let cScoreA  = $state(bout.score_a);
  let cScoreB  = $state(bout.score_b);
  let cTechAId = $state<number | null>(bout.technique_a_id);
  let cTechBId = $state<number | null>(bout.technique_b_id);
  let cZoneA   = $state<string>(bout.hit_zone_a ?? '');
  let cZoneB   = $state<string>(bout.hit_zone_b ?? '');
  let cResA    = $state<ResultType>((bout.result_a as ResultType | null) ?? 'hit');
  let cResB    = $state<ResultType>((bout.result_b as ResultType | null) ?? 'hit');

  let dirty = $derived(
    timeStartMs !== cTimeStart || timeEndMs !== cTimeEnd ||
    scoreA !== cScoreA || scoreB !== cScoreB ||
    techAId !== cTechAId || techBId !== cTechBId ||
    zoneA !== cZoneA || zoneB !== cZoneB ||
    resA !== cResA || resB !== cResB
  );

  // Notify parent when dirty changes
  $effect(() => {
    const d = dirty;
    untrack(() => onmarkdirty?.(d));
  });

  // Sync committed + form when bout prop changes from outside (WS / save)
  $effect(() => {
    const b = bout;
    const isOpen = expanded;

    cTimeStart = b.time_start_ms;
    cTimeEnd   = b.time_end_ms;
    cScoreA  = b.score_a;
    cScoreB  = b.score_b;
    cTechAId = b.technique_a_id;
    cTechBId = b.technique_b_id;
    cZoneA   = b.hit_zone_a ?? '';
    cZoneB   = b.hit_zone_b ?? '';
    cResA    = (b.result_a as ResultType | null) ?? 'hit';
    cResB    = (b.result_b as ResultType | null) ?? 'hit';

    if (!isOpen) {
      timeStartMs = b.time_start_ms;
      timeEndMs   = b.time_end_ms;
      scoreA  = b.score_a;
      scoreB  = b.score_b;
      techAId = b.technique_a_id;
      techBId = b.technique_b_id;
      zoneA   = b.hit_zone_a ?? '';
      zoneB   = b.hit_zone_b ?? '';
      resA    = (b.result_a as ResultType | null) ?? 'hit';
      resB    = (b.result_b as ResultType | null) ?? 'hit';
    }
  });

  // ── Actions ─────────────────────────────────────────────────────────────────

  let saving = $state(false);
  let saveError = $state<string | null>(null);

  async function handleSave() {
    saving = true;
    saveError = null;
    try {
      const updated = await updateBout(bout.id, {
        time_start_ms: timeStartMs,
        time_end_ms: timeEndMs,
        score_a: scoreA,
        score_b: scoreB,
        technique_a_id: techAId,
        technique_b_id: techBId,
        hit_zone_a: zoneA || null,
        hit_zone_b: zoneB || null,
        result_a: resA,
        result_b: resB,
      });
      cTimeStart = timeStartMs;
      cTimeEnd   = timeEndMs;
      cScoreA  = scoreA;
      cScoreB  = scoreB;
      cTechAId = techAId;
      cTechBId = techBId;
      cZoneA   = zoneA;
      cZoneB   = zoneB;
      cResA    = resA;
      cResB    = resB;
      untrack(() => onupdate?.(updated));
    } catch (e) {
      saveError = e instanceof Error ? e.message : 'Ошибка сохранения';
    } finally {
      saving = false;
    }
  }

  function handleCollapse() {
    if (dirty && !confirm('Есть несохранённые изменения. Свернуть карточку?')) return;
    oncollapse?.();
  }

  let deleting = $state(false);

  async function handleDelete() {
    if (!confirm(`Удалить Сход ${boutIndex}?`)) return;
    deleting = true;
    try {
      await deleteBout(bout.id);
      ondelete?.();
    } catch (e) {
      alert(e instanceof Error ? e.message : 'Ошибка удаления');
    } finally {
      deleting = false;
    }
  }

  // ── Technique tooltip ────────────────────────────────────────────────────────

  let tooltipTimer: ReturnType<typeof setTimeout> | null = null;
  let showTooltip = $state<'a' | 'b' | null>(null);

  let techADescription = $derived($techniques.find(t => t.id === techAId)?.description ?? null);
  let techBDescription = $derived($techniques.find(t => t.id === techBId)?.description ?? null);

  function startTooltip(side: 'a' | 'b') {
    if (tooltipTimer) clearTimeout(tooltipTimer);
    tooltipTimer = setTimeout(() => { showTooltip = side; }, 1000);
  }

  function stopTooltip() {
    if (tooltipTimer) { clearTimeout(tooltipTimer); tooltipTimer = null; }
    showTooltip = null;
  }

  // ── Helpers ─────────────────────────────────────────────────────────────────

  function fmtMs(ms: number): string {
    const t = Math.floor(ms / 1000);
    const h = Math.floor(t / 3600);
    const m = Math.floor((t % 3600) / 60);
    const s = t % 60;
    return h > 0
      ? `${String(h).padStart(2, '0')}:${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}`
      : `${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}`;
  }
</script>

{#if !expanded}
  <!-- ── Collapsed ── -->
  <button
    class="card card--collapsed"
    style={winnerColor ? `border-left: 3px solid ${winnerColor}; background: color-mix(in srgb, ${winnerColor} 10%, var(--surface-hover))` : ''}
    onclick={onexpand}
  >
    <span class="card-label">
      Сход {boutIndex}
      <span class="time">({fmtMs(bout.time_start_ms)} — {fmtMs(bout.time_end_ms)})</span>
    </span>
    <span class="card-score">{bout.score_a} : {bout.score_b}</span>
    {#if dirty}<span class="dirty-dot" title="Несохранённые изменения"></span>{/if}
  </button>

{:else}
  <!-- ── Expanded ── -->
  <div class="card card--expanded">

    <!-- Header (click to collapse) -->
    <!-- svelte-ignore a11y_interactive_supports_focus -->
    <div
      class="card-header"
      role="button"
      tabindex="0"
      style={winnerColor ? `background: color-mix(in srgb, ${winnerColor} 15%, var(--surface-solid)); border-bottom-color: color-mix(in srgb, ${winnerColor} 30%, var(--border-color))` : ''}
      onclick={handleCollapse}
      onkeydown={(e) => e.key === 'Enter' && handleCollapse()}
    >
      <span class="card-label">
        Сход {boutIndex}
        <span class="time">({fmtMs(bout.time_start_ms)} — {fmtMs(bout.time_end_ms)})</span>
      </span>
      <span class="card-score">{scoreA} : {scoreB}</span>
      {#if dirty}<span class="dirty-dot" title="Несохранённые изменения"></span>{/if}
    </div>

    <!-- Time range row -->
    <div class="time-row">
      <button class="time-cap-btn" onclick={() => { timeStartMs = Math.round(currentTime * 1000); }} aria-label="Захватить начало схода">
        <span class="time-cap-label">Начало</span>
        <span class="time-cap-value">{fmtMs(timeStartMs)}</span>
      </button>
      <button class="time-cap-btn" onclick={() => { timeEndMs = Math.round(currentTime * 1000); }} aria-label="Захватить конец схода">
        <span class="time-cap-label">Конец</span>
        <span class="time-cap-value">{fmtMs(timeEndMs)}</span>
      </button>
    </div>

    <!-- Two-column fighter form -->
    <div class="fighters-grid">

      <!-- Fighter A -->
      <div class="fighter-col">
        <div class="fighter-name">{fighters[0]?.display_name ?? 'Боец A'}</div>

        <div class="score-row">
          <button class="adj" onclick={() => { scoreA = Math.max(0, scoreA - 1); }} aria-label="−">−</button>
          <input class="score-inp" type="number" min="0" bind:value={scoreA} aria-label="Очки A" />
          <button class="adj" onclick={() => { scoreA += 1; }} aria-label="+">+</button>
        </div>

        <div class="field">
          <span class="field-lbl">Техника</span>
          <div class="tech-wrap"
            onmouseenter={() => startTooltip('a')}
            onmouseleave={stopTooltip}
          >
            <select class="field-sel" bind:value={techAId}>
              <option value={null}>—</option>
              {#each $techniques as t (t.id)}
                <option value={t.id}>{t.name}</option>
              {/each}
            </select>
            {#if showTooltip === 'a' && techADescription}
              <div class="tech-tooltip">
                <!-- eslint-disable-next-line svelte/no-at-html-tags -->
                {@html techADescription}
              </div>
            {/if}
          </div>
        </div>

        <div class="field">
          <span class="field-lbl">Зона поражения</span>
          <HitZonePicker value={zoneA} onchange={(z) => { zoneA = z; }} />
        </div>

        <div class="field">
          <span class="field-lbl">Результат</span>
          <select class="field-sel" bind:value={resA}>
            <option value="hit">Попал</option>
            <option value="miss">Промахнулся</option>
            <option value="blocked">Заблокировали</option>
            <option value="late">Опоздал</option>
            <option value="no_strike">Не бил</option>
            <option value="disqualification">Неквалификация</option>
            <option value="afterblow">Афтерблоу</option>
          </select>
        </div>
      </div>

      <div class="col-divider"></div>

      <!-- Fighter B -->
      <div class="fighter-col">
        <div class="fighter-name">{fighters[1]?.display_name ?? 'Боец B'}</div>

        <div class="score-row">
          <button class="adj" onclick={() => { scoreB = Math.max(0, scoreB - 1); }} aria-label="−">−</button>
          <input class="score-inp" type="number" min="0" bind:value={scoreB} aria-label="Очки B" />
          <button class="adj" onclick={() => { scoreB += 1; }} aria-label="+">+</button>
        </div>

        <div class="field">
          <span class="field-lbl">Техника</span>
          <div class="tech-wrap"
            onmouseenter={() => startTooltip('b')}
            onmouseleave={stopTooltip}
          >
            <select class="field-sel" bind:value={techBId}>
              <option value={null}>—</option>
              {#each $techniques as t (t.id)}
                <option value={t.id}>{t.name}</option>
              {/each}
            </select>
            {#if showTooltip === 'b' && techBDescription}
              <div class="tech-tooltip">
                <!-- eslint-disable-next-line svelte/no-at-html-tags -->
                {@html techBDescription}
              </div>
            {/if}
          </div>
        </div>

        <div class="field">
          <span class="field-lbl">Зона поражения</span>
          <HitZonePicker value={zoneB} onchange={(z) => { zoneB = z; }} />
        </div>

        <div class="field">
          <span class="field-lbl">Результат</span>
          <select class="field-sel" bind:value={resB}>
            <option value="hit">Попал</option>
            <option value="miss">Промахнулся</option>
            <option value="blocked">Заблокировали</option>
            <option value="late">Опоздал</option>
            <option value="no_strike">Не бил</option>
            <option value="disqualification">Неквалификация</option>
            <option value="afterblow">Афтерблоу</option>
          </select>
        </div>
      </div>

    </div>

    {#if saveError}
      <div class="save-error">{saveError}</div>
    {/if}

    <div class="card-actions">
      <button class="btn btn-primary btn-sm" onclick={handleSave} disabled={saving || !dirty}>
        {saving ? 'Сохранение…' : 'Сохранить'}
      </button>
      <button class="btn btn-outline btn-sm" onclick={handleCollapse}>Свернуть</button>
      <button class="btn-delete" onclick={handleDelete} disabled={deleting} aria-label="Удалить сход">
        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true">
          <polyline points="3,6 5,6 21,6"/><path d="M19,6l-1,14H6L5,6"/><path d="M10,11v6"/><path d="M14,11v6"/><path d="M9,6V4h6v2"/>
        </svg>
      </button>
    </div>

  </div>
{/if}

<style>
  /* ── Shared card base ───────────────────────────────────────────────────── */
  .card {
    width: 100%;
    border-radius: 5px;
    font-size: 0.8rem;
    color: var(--text-primary);
  }

  /* ── Collapsed ──────────────────────────────────────────────────────────── */
  .card--collapsed {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 12px;
    background: var(--surface-hover);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm);
    cursor: pointer;
    text-align: left;
    transition: var(--transition);
    position: relative;
  }

  .card--collapsed:hover {
    background: var(--surface-solid);
    border-color: var(--text-secondary);
  }

  /* ── Expanded ───────────────────────────────────────────────────────────── */
  .card--expanded {
    background: var(--surface-hover);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm);
    display: flex;
    flex-direction: column;
    gap: 0;
  }

  .card-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 12px;
    cursor: pointer;
    border-bottom: 1px solid var(--border-color);
    border-radius: var(--radius-sm) var(--radius-sm) 0 0;
    transition: var(--transition);
    user-select: none;
  }

  .card-header:hover {
    background: var(--surface-solid);
  }

  /* ── Labels & score ─────────────────────────────────────────────────────── */
  .card-label {
    font-size: 0.78rem;
    color: #a0b4c8;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .time {
    color: #5a7a96;
    font-size: 0.72rem;
    margin-left: 4px;
  }

  .card-score {
    font-size: 0.88rem;
    font-weight: 700;
    color: #DB841F;
    white-space: nowrap;
    margin-left: 12px;
    flex-shrink: 0;
  }

  .dirty-dot {
    display: inline-block;
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: #DB841F;
    margin-left: 6px;
    flex-shrink: 0;
    vertical-align: middle;
  }

  /* ── Time row ───────────────────────────────────────────────────────────── */
  .time-row {
    display: flex;
    gap: 6px;
    padding: 8px 10px 0;
  }

  .time-cap-btn {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
    padding: 6px 8px;
    background: var(--surface-solid);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm);
    cursor: pointer;
    transition: var(--transition);
  }

  .time-cap-btn:hover {
    border-color: var(--accent-yellow);
    background: rgba(219, 132, 31, 0.08);
  }

  .time-cap-label {
    font-size: 0.62rem;
    color: #4a6280;
    text-transform: uppercase;
    letter-spacing: 0.06em;
  }

  .time-cap-value {
    font-size: 0.78rem;
    font-variant-numeric: tabular-nums;
    color: #a0b4c8;
  }

  /* ── Fighter grid ───────────────────────────────────────────────────────── */
  .fighters-grid {
    display: grid;
    grid-template-columns: 1fr 1px 1fr;
    gap: 0;
    padding: 10px;
  }

  .col-divider {
    background: var(--border-color);
    margin: 0 10px;
  }

  .fighter-col {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .fighter-name {
    font-size: 0.8rem;
    font-weight: 600;
    color: var(--text-primary);
    letter-spacing: 0.04em;
    text-transform: uppercase;
    padding-bottom: 4px;
    border-bottom: 1px solid var(--border-color);
  }

  /* ── Score row ──────────────────────────────────────────────────────────── */
  .score-row {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .adj {
    width: 28px;
    height: 28px;
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm);
    background: var(--surface-solid);
    color: var(--text-secondary);
    font-size: 1rem;
    line-height: 1;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    transition: var(--transition);
  }

  .adj:hover {
    background: var(--surface-hover);
    border-color: var(--text-primary);
    color: var(--text-primary);
  }

  .score-inp {
    width: 48px;
    text-align: center;
    background: var(--surface-solid);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm);
    color: var(--accent-yellow);
    font-size: 0.95rem;
    font-weight: 700;
    padding: 4px 6px;
    outline: none;
    -moz-appearance: textfield;
  }

  .score-inp::-webkit-inner-spin-button,
  .score-inp::-webkit-outer-spin-button {
    -webkit-appearance: none;
  }

  .score-inp:focus {
    border-color: var(--text-primary);
  }

  /* ── Field group ────────────────────────────────────────────────────────── */
  .field {
    display: flex;
    flex-direction: column;
    gap: 3px;
  }

  .field-lbl {
    font-size: 0.75rem;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.06em;
  }

  .field-sel {
    width: 100%;
    padding: 6px 8px;
    background: var(--surface-solid);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm);
    color: var(--text-primary);
    outline: none;
    transition: var(--transition);
  }

  .field-sel:focus {
    border-color: var(--accent-yellow);
  }

  /* ── Technique tooltip ──────────────────────────────────────────────────── */
  .tech-wrap {
    position: relative;
  }

  .tech-tooltip {
    position: absolute;
    bottom: calc(100% + 6px);
    left: 0;
    z-index: 300;
    background: var(--surface-solid);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-md);
    padding: 12px 16px;
    width: 280px;
    max-height: 240px;
    overflow-y: auto;
    font-size: 0.85rem;
    color: var(--text-primary);
    line-height: 1.55;
    box-shadow: var(--shadow-lg);
    pointer-events: none;
  }

  .tech-tooltip :global(img) {
    max-width: 100%;
    border-radius: 3px;
    margin: 4px 0;
  }

  .tech-tooltip :global(iframe) {
    max-width: 100%;
    border-radius: 3px;
    margin: 4px 0;
  }

  .tech-tooltip :global(p) {
    margin: 0 0 6px;
  }

  /* ── Error ──────────────────────────────────────────────────────────────── */
  .save-error {
    margin: 0 12px;
    padding: 8px 12px;
    background: rgba(239, 68, 68, 0.1);
    border: 1px solid rgba(239, 68, 68, 0.2);
    border-radius: var(--radius-sm);
    font-size: 0.85rem;
    color: #ef4444;
  }

  .card-actions {
    display: flex;
    gap: 8px;
    padding: 12px;
    border-top: 1px solid var(--border-color);
  }

  .btn-delete {
    flex-shrink: 0;
    width: 32px;
    height: 32px;
    padding: 0;
    margin-left: auto;
    border-radius: var(--radius-sm);
    background: rgba(239, 68, 68, 0.08);
    border: 1px solid rgba(239, 68, 68, 0.2);
    color: #dc2626;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: var(--transition);
  }

  .btn-delete:hover:not(:disabled) {
    background: rgba(239, 68, 68, 0.2);
    border-color: rgba(239, 68, 68, 0.4);
    color: #ef4444;
  }

  .btn-delete:disabled {
    opacity: 0.4;
    cursor: default;
  }
</style>
