<script lang="ts">
  import { untrack } from 'svelte';
  import { techniques } from '../../stores';
  import type { Bout, VideoFighter } from '../api/types';
  import { updateBout } from '../api/bouts';

  const HIT_ZONES = [
    'Голова', 'Тело', 'Рука левая', 'Рука правая', 'Нога левая', 'Нога правая',
  ] as const;

  type ResultType = 'hit' | 'miss' | 'blocked';

  interface Props {
    bout: Bout;
    fighters: [VideoFighter | null, VideoFighter | null];
    expanded: boolean;
    onexpand: () => void;
    oncollapse: () => void;
    onmarkdirty: (dirty: boolean) => void;
    onupdate: (updated: Bout) => void;
  }

  let { bout, fighters, expanded, onexpand, oncollapse, onmarkdirty, onupdate }: Props = $props();

  // ── Form state ──────────────────────────────────────────────────────────────

  let scoreA   = $state(bout.score_a);
  let scoreB   = $state(bout.score_b);
  let techAId  = $state<number | null>(bout.technique_a_id);
  let techBId  = $state<number | null>(bout.technique_b_id);
  let zoneA    = $state<string>(bout.hit_zone_a ?? '');
  let zoneB    = $state<string>(bout.hit_zone_b ?? '');
  let resA     = $state<ResultType>((bout.result_a as ResultType | null) ?? 'hit');
  let resB     = $state<ResultType>((bout.result_b as ResultType | null) ?? 'hit');

  // ── Committed snapshot (last saved) ─────────────────────────────────────────

  let cScoreA  = $state(bout.score_a);
  let cScoreB  = $state(bout.score_b);
  let cTechAId = $state<number | null>(bout.technique_a_id);
  let cTechBId = $state<number | null>(bout.technique_b_id);
  let cZoneA   = $state<string>(bout.hit_zone_a ?? '');
  let cZoneB   = $state<string>(bout.hit_zone_b ?? '');
  let cResA    = $state<ResultType>((bout.result_a as ResultType | null) ?? 'hit');
  let cResB    = $state<ResultType>((bout.result_b as ResultType | null) ?? 'hit');

  let dirty = $derived(
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

    cScoreA  = b.score_a;
    cScoreB  = b.score_b;
    cTechAId = b.technique_a_id;
    cTechBId = b.technique_b_id;
    cZoneA   = b.hit_zone_a ?? '';
    cZoneB   = b.hit_zone_b ?? '';
    cResA    = (b.result_a as ResultType | null) ?? 'hit';
    cResB    = (b.result_b as ResultType | null) ?? 'hit';

    if (!isOpen) {
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
        score_a: scoreA,
        score_b: scoreB,
        technique_a_id: techAId,
        technique_b_id: techBId,
        hit_zone_a: zoneA || null,
        hit_zone_b: zoneB || null,
        result_a: resA,
        result_b: resB,
      });
      // Advance committed to current form values
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
  <button class="card card--collapsed" onclick={onexpand}>
    <span class="card-label">
      Сход {bout.order_index}
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
      onclick={handleCollapse}
      onkeydown={(e) => e.key === 'Enter' && handleCollapse()}
    >
      <span class="card-label">
        Сход {bout.order_index}
        <span class="time">({fmtMs(bout.time_start_ms)} — {fmtMs(bout.time_end_ms)})</span>
      </span>
      <span class="card-score">{scoreA} : {scoreB}</span>
      {#if dirty}<span class="dirty-dot" title="Несохранённые изменения"></span>{/if}
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
          <select class="field-sel" bind:value={techAId}>
            <option value={null}>—</option>
            {#each $techniques as t (t.id)}
              <option value={t.id}>{t.name}</option>
            {/each}
          </select>
        </div>

        <div class="field">
          <span class="field-lbl">Зона поражения</span>
          <select class="field-sel" bind:value={zoneA}>
            <option value="">—</option>
            {#each HIT_ZONES as z}
              <option value={z}>{z}</option>
            {/each}
          </select>
        </div>

        <div class="field">
          <span class="field-lbl">Результат</span>
          <div class="radios">
            <label><input type="radio" bind:group={resA} value="hit" /> Попал</label>
            <label><input type="radio" bind:group={resA} value="miss" /> Промахнулся</label>
            <label><input type="radio" bind:group={resA} value="blocked" /> Заблокировали</label>
          </div>
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
          <select class="field-sel" bind:value={techBId}>
            <option value={null}>—</option>
            {#each $techniques as t (t.id)}
              <option value={t.id}>{t.name}</option>
            {/each}
          </select>
        </div>

        <div class="field">
          <span class="field-lbl">Зона поражения</span>
          <select class="field-sel" bind:value={zoneB}>
            <option value="">—</option>
            {#each HIT_ZONES as z}
              <option value={z}>{z}</option>
            {/each}
          </select>
        </div>

        <div class="field">
          <span class="field-lbl">Результат</span>
          <div class="radios">
            <label><input type="radio" bind:group={resB} value="hit" /> Попал</label>
            <label><input type="radio" bind:group={resB} value="miss" /> Промахнулся</label>
            <label><input type="radio" bind:group={resB} value="blocked" /> Заблокировали</label>
          </div>
        </div>
      </div>

    </div>

    {#if saveError}
      <div class="save-error">{saveError}</div>
    {/if}

    <div class="card-actions">
      <button class="btn-save" onclick={handleSave} disabled={saving || !dirty}>
        {saving ? 'Сохранение…' : 'Сохранить'}
      </button>
      <button class="btn-collapse" onclick={handleCollapse}>Свернуть</button>
    </div>

  </div>
{/if}

<style>
  /* ── Shared card base ───────────────────────────────────────────────────── */
  .card {
    width: 100%;
    border-radius: 5px;
    font-size: 0.8rem;
    color: #a0b4c8;
  }

  /* ── Collapsed ──────────────────────────────────────────────────────────── */
  .card--collapsed {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 7px 10px;
    background: #0d1e35;
    border: 1px solid #1a3050;
    cursor: pointer;
    text-align: left;
    transition: background 0.12s, border-color 0.12s;
    position: relative;
  }

  .card--collapsed:hover {
    background: #0f2035;
    border-color: #2a4f73;
    color: #d0dde8;
  }

  /* ── Expanded ───────────────────────────────────────────────────────────── */
  .card--expanded {
    background: #0d1e35;
    border: 1px solid #2a4f73;
    display: flex;
    flex-direction: column;
    gap: 0;
  }

  .card-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 7px 10px;
    cursor: pointer;
    border-bottom: 1px solid #1a3050;
    transition: background 0.12s;
    user-select: none;
  }

  .card-header:hover {
    background: #0f2035;
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

  /* ── Fighter grid ───────────────────────────────────────────────────────── */
  .fighters-grid {
    display: grid;
    grid-template-columns: 1fr 1px 1fr;
    gap: 0;
    padding: 10px;
  }

  .col-divider {
    background: #1a3050;
    margin: 0 10px;
  }

  .fighter-col {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .fighter-name {
    font-size: 0.75rem;
    font-weight: 600;
    color: #d0dde8;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    padding-bottom: 4px;
    border-bottom: 1px solid #1a3050;
  }

  /* ── Score row ──────────────────────────────────────────────────────────── */
  .score-row {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .adj {
    width: 26px;
    height: 26px;
    border: 1px solid #1a3050;
    border-radius: 4px;
    background: #0a1628;
    color: #a0b4c8;
    font-size: 1rem;
    line-height: 1;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    transition: background 0.1s, border-color 0.1s;
  }

  .adj:hover {
    background: #0f2035;
    border-color: #2a4f73;
    color: #d0dde8;
  }

  .score-inp {
    width: 44px;
    text-align: center;
    background: #0a1628;
    border: 1px solid #1a3050;
    border-radius: 4px;
    color: #DB841F;
    font-size: 0.9rem;
    font-weight: 700;
    padding: 3px 4px;
    outline: none;
    -moz-appearance: textfield;
  }

  .score-inp::-webkit-inner-spin-button,
  .score-inp::-webkit-outer-spin-button {
    -webkit-appearance: none;
  }

  .score-inp:focus {
    border-color: #2a4f73;
  }

  /* ── Field group ────────────────────────────────────────────────────────── */
  .field {
    display: flex;
    flex-direction: column;
    gap: 3px;
  }

  .field-lbl {
    font-size: 0.68rem;
    color: #4a6280;
    text-transform: uppercase;
    letter-spacing: 0.06em;
  }

  .field-sel {
    background: #0a1628;
    border: 1px solid #1a3050;
    border-radius: 4px;
    color: #a0b4c8;
    font-size: 0.78rem;
    padding: 4px 6px;
    outline: none;
    cursor: pointer;
    transition: border-color 0.1s;
  }

  .field-sel:hover,
  .field-sel:focus {
    border-color: #2a4f73;
    color: #d0dde8;
  }

  /* ── Radios ─────────────────────────────────────────────────────────────── */
  .radios {
    display: flex;
    flex-direction: column;
    gap: 3px;
  }

  .radios label {
    display: flex;
    align-items: center;
    gap: 5px;
    font-size: 0.76rem;
    color: #8aa0b8;
    cursor: pointer;
  }

  .radios input[type="radio"] {
    accent-color: #DB841F;
    cursor: pointer;
  }

  /* ── Error ──────────────────────────────────────────────────────────────── */
  .save-error {
    margin: 0 10px;
    padding: 5px 8px;
    background: rgba(224, 82, 82, 0.12);
    border: 1px solid rgba(224, 82, 82, 0.3);
    border-radius: 4px;
    font-size: 0.75rem;
    color: #e08080;
  }

  /* ── Actions ────────────────────────────────────────────────────────────── */
  .card-actions {
    display: flex;
    gap: 6px;
    padding: 8px 10px;
    border-top: 1px solid #1a3050;
  }

  .btn-save,
  .btn-collapse {
    flex: 1;
    padding: 6px 0;
    border-radius: 4px;
    font-size: 0.78rem;
    font-weight: 600;
    cursor: pointer;
    transition: background 0.12s, opacity 0.12s;
    border: none;
  }

  .btn-save {
    background: #DB841F;
    color: #fff;
  }

  .btn-save:hover:not(:disabled) {
    background: #e8941f;
  }

  .btn-save:disabled {
    opacity: 0.4;
    cursor: default;
  }

  .btn-collapse {
    background: #0a1628;
    border: 1px solid #1a3050;
    color: #7090a8;
  }

  .btn-collapse:hover {
    background: #0f2035;
    border-color: #2a4f73;
    color: #d0dde8;
  }
</style>
