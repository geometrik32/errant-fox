<script lang="ts">
  import type { FighterBout } from '../api/types';
  import { techniques } from '../../stores';
  import { HIT_ZONES } from '../player/HitZonePicker.svelte';

  export interface TableFilters {
    date: string;
    opponent_id: string;
    opponent_name: string;
    my_technique: string;
    my_result: string;
    my_zone: string;
    opponent_technique: string;
    opponent_result: string;
    opponent_zone: string;
    score: string;
    date_week: string;
    video: string;
    sort_col: string;
    sort_dir: 'asc' | 'desc';
  }

  interface Props {
    bouts: FighterBout[];
    filters: TableFilters;
    opponents: Array<{ id: string; name: string }>;
    videoLabels?: Map<string, string>;
    onfilter: (f: TableFilters) => void;
    onnavigate: (videoId: string, timeStartMs?: number) => void;
  }

  let { bouts, filters, opponents, videoLabels = new Map(), onfilter, onnavigate }: Props = $props();

  // Visible column config
  type ColKey = 'video' | 'date' | 'opponent' | 'score' | 'my_tech' | 'my_result' | 'my_zone' | 'opp_tech' | 'opp_result' | 'opp_zone';
  const COL_LABELS: Record<ColKey, string> = {
    video: 'Видео', date: 'Дата', opponent: 'Оппонент', score: 'Счёт',
    my_tech: 'Мой приём', my_result: 'Мой рез.', my_zone: 'Моя зона',
    opp_tech: 'Приём опп.', opp_result: 'Рез. опп.', opp_zone: 'Зона опп.',
  };
  // 'date' hidden by default (available in column picker); 'video' shown
  let visibleCols = $state<Set<ColKey>>(new Set(
    (Object.keys(COL_LABELS) as ColKey[])
  ));
  let showColPicker = $state(false);

  function toggleCol(col: ColKey) {
    const next = new Set(visibleCols);
    if (next.has(col)) { if (next.size > 1) next.delete(col); }
    else next.add(col);
    visibleCols = next;
  }

  function toggleSort(col: string) {
    if (filters.sort_col === col) {
      onfilter({ ...filters, sort_dir: filters.sort_dir === 'asc' ? 'desc' : 'asc' });
    } else {
      onfilter({ ...filters, sort_col: col, sort_dir: 'asc' });
    }
  }

  function setFilter(key: keyof TableFilters, value: string) {
    onfilter({ ...filters, [key]: value });
  }

  function sortIcon(col: string) {
    if (filters.sort_col !== col) return '↕';
    return filters.sort_dir === 'asc' ? '↑' : '↓';
  }

  function formatDate(d: string) {
    return d ? d.slice(0, 10) : '';
  }

  function fmtMs(ms: number): string {
    const t = Math.floor(ms / 1000);
    const m = Math.floor(t / 60);
    const s = t % 60;
    return `${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}`;
  }

  const RESULT_LABELS: Record<string, string> = {
    hit: 'Попал', miss: 'Промах', blocked: 'Заблокировали',
    late: 'Опоздал', no_strike: 'Не бил',
    disqualification: 'Неквалификация', afterblow: 'Афтерблоу',
  };
  function resultLabel(v: string | null): string {
    return v ? (RESULT_LABELS[v] ?? v) : '—';
  }
</script>

<div class="table-wrap">
  <div class="table-header-bar">
    <h3 class="table-title">История сходов</h3>
    <div class="col-picker-wrap">
      <button class="eye-btn" onclick={() => { showColPicker = !showColPicker; }} title="Выбрать столбцы" aria-expanded={showColPicker}>
        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true">
          <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"/>
          <circle cx="12" cy="12" r="3"/>
        </svg>
      </button>
      {#if showColPicker}
        <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
        <div class="col-picker" role="menu" onclick={(e) => e.stopPropagation()}>
          {#each Object.entries(COL_LABELS) as [key, label]}
            <label class="col-option">
              <input type="checkbox" checked={visibleCols.has(key as ColKey)} onchange={() => toggleCol(key as ColKey)} />
              {label}
            </label>
          {/each}
        </div>
      {/if}
    </div>
  </div>
  <table class="table">
    <thead>
      <tr class="header-row">
        {#if visibleCols.has('video')}
        <th class="th sortable" onclick={() => toggleSort('video_date')}>
          <span>Видео <span class="sort-icon">{sortIcon('video_date')}</span></span>
          <input class="filter-input" type="text" placeholder="Поиск…" value={filters.video ?? ''}
            oninput={(e) => setFilter('video', (e.target as HTMLInputElement).value)}
            onclick={(e) => e.stopPropagation()} />
        </th>
        {/if}
        {#if visibleCols.has('date')}
        <th class="th sortable" onclick={() => toggleSort('video_date')}>
          <span>Дата <span class="sort-icon">{sortIcon('video_date')}</span></span>
          <input class="filter-input" type="date" value={filters.date}
            oninput={(e) => setFilter('date', (e.target as HTMLInputElement).value)}
            onclick={(e) => e.stopPropagation()} />
        </th>
        {/if}
        {#if visibleCols.has('opponent')}
        <th class="th sortable" onclick={() => toggleSort('opponent_name')}>
          <span>Оппонент <span class="sort-icon">{sortIcon('opponent_name')}</span></span>
          <select class="filter-input" value={filters.opponent_id}
            onchange={(e) => setFilter('opponent_id', (e.target as HTMLSelectElement).value)}
            onclick={(e) => e.stopPropagation()}>
            <option value="">Все</option>
            {#each opponents as opp}
              <option value={opp.id}>{opp.name}</option>
            {/each}
          </select>
        </th>
        {/if}
        {#if visibleCols.has('score')}
        <th class="th sortable" onclick={() => toggleSort('my_score')}>
          <span>Счёт <span class="sort-icon">{sortIcon('my_score')}</span></span>
          <input class="filter-input" type="text" placeholder="Фильтр…" value={filters.score}
            oninput={(e) => setFilter('score', (e.target as HTMLInputElement).value)}
            onclick={(e) => e.stopPropagation()} />
        </th>
        {/if}
        {#if visibleCols.has('my_tech')}
        <th class="th sortable" onclick={() => toggleSort('my_technique_name')}>
          <span>Мой приём <span class="sort-icon">{sortIcon('my_technique_name')}</span></span>
          <select class="filter-input" value={filters.my_technique}
            onchange={(e) => setFilter('my_technique', (e.target as HTMLSelectElement).value)}
            onclick={(e) => e.stopPropagation()}>
            <option value="">Все</option>
            {#each $techniques as t (t.id)}
              <option value={t.name}>{t.name}</option>
            {/each}
          </select>
        </th>
        {/if}
        {#if visibleCols.has('my_result')}
        <th class="th sortable" onclick={() => toggleSort('my_result')}>
          <span>Мой рез. <span class="sort-icon">{sortIcon('my_result')}</span></span>
          <select class="filter-input" value={filters.my_result}
            onchange={(e) => setFilter('my_result', (e.target as HTMLSelectElement).value)}
            onclick={(e) => e.stopPropagation()}>
            <option value="">Все</option>
            <option value="hit">Попал</option>
            <option value="miss">Промах</option>
            <option value="blocked">Заблокировали</option>
            <option value="late">Опоздал</option>
            <option value="no_strike">Не бил</option>
            <option value="disqualification">Неквалификация</option>
            <option value="afterblow">Афтерблоу</option>
          </select>
        </th>
        {/if}
        {#if visibleCols.has('my_zone')}
        <th class="th sortable" onclick={() => toggleSort('my_hit_zone')}>
          <span>Моя зона <span class="sort-icon">{sortIcon('my_hit_zone')}</span></span>
          <select class="filter-input" value={filters.my_zone}
            onchange={(e) => setFilter('my_zone', (e.target as HTMLSelectElement).value)}
            onclick={(e) => e.stopPropagation()}>
            <option value="">Все</option>
            {#each HIT_ZONES as zone}
              <option value={zone}>{zone}</option>
            {/each}
          </select>
        </th>
        {/if}
        {#if visibleCols.has('opp_tech')}
        <th class="th sortable" onclick={() => toggleSort('opponent_technique_name')}>
          <span>Приём опп. <span class="sort-icon">{sortIcon('opponent_technique_name')}</span></span>
          <select class="filter-input" value={filters.opponent_technique}
            onchange={(e) => setFilter('opponent_technique', (e.target as HTMLSelectElement).value)}
            onclick={(e) => e.stopPropagation()}>
            <option value="">Все</option>
            {#each $techniques as t (t.id)}
              <option value={t.name}>{t.name}</option>
            {/each}
          </select>
        </th>
        {/if}
        {#if visibleCols.has('opp_result')}
        <th class="th sortable" onclick={() => toggleSort('opponent_result')}>
          <span>Рез. опп. <span class="sort-icon">{sortIcon('opponent_result')}</span></span>
          <select class="filter-input" value={filters.opponent_result}
            onchange={(e) => setFilter('opponent_result', (e.target as HTMLSelectElement).value)}
            onclick={(e) => e.stopPropagation()}>
            <option value="">Все</option>
            <option value="hit">Попал</option>
            <option value="miss">Промах</option>
            <option value="blocked">Заблокировали</option>
            <option value="late">Опоздал</option>
            <option value="no_strike">Не бил</option>
            <option value="disqualification">Неквалификация</option>
            <option value="afterblow">Афтерблоу</option>
          </select>
        </th>
        {/if}
        {#if visibleCols.has('opp_zone')}
        <th class="th sortable" onclick={() => toggleSort('opponent_hit_zone')}>
          <span>Зона опп. <span class="sort-icon">{sortIcon('opponent_hit_zone')}</span></span>
          <select class="filter-input" value={filters.opponent_zone}
            onchange={(e) => setFilter('opponent_zone', (e.target as HTMLSelectElement).value)}
            onclick={(e) => e.stopPropagation()}>
            <option value="">Все</option>
            {#each HIT_ZONES as zone}
              <option value={zone}>{zone}</option>
            {/each}
          </select>
        </th>
        {/if}
        <th class="th th--nav"></th>
      </tr>
    </thead>
    <tbody>
      {#each bouts as bout (bout.id)}
        <tr class="body-row">
          {#if visibleCols.has('video')}<td class="td video-label">{videoLabels.get(bout.video_id) ?? bout.video_id.slice(0, 8)}</td>{/if}
          {#if visibleCols.has('date')}<td class="td date-cell">{formatDate(bout.video_date)}</td>{/if}
          {#if visibleCols.has('opponent')}<td class="td">{bout.opponent_name}</td>{/if}
          {#if visibleCols.has('score')}
          <td class="td score">
            <span class:win={bout.my_score > bout.opponent_score} class:loss={bout.my_score < bout.opponent_score}>{bout.my_score}</span>
            <span class="sep">:</span>
            <span class:win={bout.my_score > bout.opponent_score} class:loss={bout.my_score < bout.opponent_score}>{bout.opponent_score}</span>
          </td>
          {/if}
          {#if visibleCols.has('my_tech')}<td class="td">{bout.my_technique_name ?? '—'}</td>{/if}
          {#if visibleCols.has('my_result')}
          <td class="td">
            <span class="result-badge"
              class:hit={bout.my_result === 'hit'}
              class:miss={bout.my_result === 'miss'}
              class:blocked={bout.my_result === 'blocked'}
              class:late={bout.my_result === 'late'}
              class:no-strike={bout.my_result === 'no_strike'}
              class:disqualification={bout.my_result === 'disqualification'}
              class:afterblow={bout.my_result === 'afterblow'}
            >{resultLabel(bout.my_result)}</span>
          </td>
          {/if}
          {#if visibleCols.has('my_zone')}<td class="td zone-cell">{(bout.my_hit_zone ?? '').split(':')[0] || '—'}</td>{/if}
          {#if visibleCols.has('opp_tech')}<td class="td">{bout.opponent_technique_name ?? '—'}</td>{/if}
          {#if visibleCols.has('opp_result')}
          <td class="td">
            <span class="result-badge"
              class:hit={bout.opponent_result === 'hit'}
              class:miss={bout.opponent_result === 'miss'}
              class:blocked={bout.opponent_result === 'blocked'}
              class:late={bout.opponent_result === 'late'}
              class:no-strike={bout.opponent_result === 'no_strike'}
              class:disqualification={bout.opponent_result === 'disqualification'}
              class:afterblow={bout.opponent_result === 'afterblow'}
            >{resultLabel(bout.opponent_result)}</span>
          </td>
          {/if}
          {#if visibleCols.has('opp_zone')}<td class="td zone-cell">{(bout.opponent_hit_zone ?? '').split(':')[0] || '—'}</td>{/if}
          <td class="td td--nav">
            <button class="nav-btn" onclick={() => onnavigate(bout.video_id, bout.time_start_ms)} title="Открыть видео">→</button>
          </td>
        </tr>
      {/each}
      {#if bouts.length === 0}
        <tr>
          <td colspan="10" class="td empty">Нет данных</td>
        </tr>
      {/if}
    </tbody>
  </table>
</div>

<style>
  .table-wrap {
    overflow-x: auto;
    background: var(--surface);
    backdrop-filter: var(--glass-blur);
    -webkit-backdrop-filter: var(--glass-blur);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-md);
    box-shadow: var(--shadow-sm);
  }

  .table {
    width: 100%;
    border-collapse: collapse;
    font-size: 0.82rem;
  }

  .th {
    background: var(--surface-hover);
    color: var(--text-secondary);
    font-weight: 600;
    font-size: 0.72rem;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    padding: 10px 12px 6px;
    text-align: left;
    border-bottom: 1px solid var(--border-color);
    white-space: nowrap;
    vertical-align: top;
    position: sticky;
    top: 0;
    z-index: 2;
  }

  .th.sortable {
    cursor: pointer;
    user-select: none;
  }

  .th.sortable:hover {
    color: var(--text-primary);
  }

  .th span {
    display: flex;
    align-items: center;
    gap: 4px;
    margin-bottom: 5px;
  }

  .sort-icon {
    color: var(--accent-yellow);
    font-size: 0.7rem;
  }

  .th--nav {
    width: 40px;
    cursor: default;
  }

  .filter-input {
    display: block;
    width: 100%;
    background: var(--surface-solid);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm);
    color: var(--text-primary);
    font-size: 0.75rem;
    padding: 4px 8px;
    outline: none;
    min-width: 0;
    transition: var(--transition);
    margin-bottom: 4px;
  }

  .filter-input:focus {
    border-color: var(--accent-yellow);
  }

  .filter-input::-webkit-calendar-picker-indicator {
    filter: invert(0.5);
    cursor: pointer;
  }

  .filter-spacer {
    height: 26px;
    margin-bottom: 4px;
  }

  .body-row {
    border-bottom: 1px solid var(--border-color);
    transition: var(--transition);
  }

  .body-row:hover {
    background: var(--surface-hover);
  }

  .td {
    padding: 10px 12px;
    color: var(--text-primary);
    vertical-align: middle;
    text-align: left;
  }

  .td.empty {
    text-align: center;
    color: #4a6280;
    padding: 24px;
  }

  .score {
    font-variant-numeric: tabular-nums;
    white-space: nowrap;
  }

  .sep {
    color: var(--text-secondary);
    margin: 0 3px;
  }

  .win {
    color: #10b981;
    font-weight: 600;
  }

  .loss {
    color: #e05252;
  }

  .result-badge {
    display: inline-block;
    padding: 2px 7px;
    border-radius: 4px;
    font-size: 0.75rem;
    font-weight: 500;
  }

  .result-badge.hit {
    background: rgba(76, 175, 130, 0.12);
    color: #4caf82;
  }

  .result-badge.miss {
    background: rgba(224, 82, 82, 0.1);
    color: #e05252;
  }

  .result-badge.blocked {
    background: rgba(140, 100, 200, 0.12);
    color: #a07cd8;
  }

  .result-badge.late {
    background: rgba(200, 160, 40, 0.12);
    color: #c8a028;
  }

  .result-badge.no-strike {
    background: rgba(90, 120, 150, 0.12);
    color: #6b8aab;
  }

  .result-badge.disqualification {
    background: rgba(200, 80, 200, 0.1);
    color: #c070c0;
  }

  .result-badge.afterblow {
    background: rgba(80, 160, 200, 0.1);
    color: #50a0c8;
  }

  .td--nav {
    text-align: center;
  }

  .nav-btn {
    background: none;
    border: 1px solid var(--border-color);
    color: var(--text-secondary);
    border-radius: var(--radius-sm);
    width: 28px;
    height: 24px;
    cursor: pointer;
    font-size: 0.9rem;
    transition: var(--transition);
    display: flex;
    align-items: center;
    justify-content: center;
    margin: 0 auto;
  }

  .nav-btn:hover {
    background: var(--surface-solid);
    color: var(--accent-yellow);
    border-color: var(--accent-yellow);
  }

  /* Column picker */
  .table-header-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 16px 20px;
    border-bottom: 1px solid var(--border-color);
  }

  .table-title {
    font-size: 0.9rem;
    font-weight: 700;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--text-primary);
    margin: 0;
  }

  .col-picker-wrap {
    position: relative;
  }

  .eye-btn {
    background: none;
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm);
    color: var(--text-secondary);
    width: 28px;
    height: 28px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: var(--transition);
  }

  .eye-btn:hover {
    color: var(--text-primary);
    background: var(--surface-hover);
  }

  .col-picker {
    position: absolute;
    right: 0;
    top: calc(100% + 8px);
    background: var(--surface-solid);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-md);
    padding: 12px 16px;
    z-index: 20;
    min-width: 150px;
    display: flex;
    flex-direction: column;
    gap: 8px;
    box-shadow: var(--shadow-lg);
  }

  .col-option {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 0.85rem;
    color: var(--text-primary);
    cursor: pointer;
    user-select: none;
    white-space: nowrap;
  }

  .col-option input[type="checkbox"] {
    accent-color: var(--accent-yellow);
    cursor: pointer;
  }

  .zone-cell {
    font-size: 0.8rem;
    color: var(--text-secondary);
  }

  .video-label {
    font-size: 0.8rem;
    color: var(--text-primary);
    white-space: nowrap;
    max-width: 160px;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .date-cell {
    font-size: 0.78rem;
    color: var(--text-secondary);
    white-space: nowrap;
    font-variant-numeric: tabular-nums;
  }
</style>
