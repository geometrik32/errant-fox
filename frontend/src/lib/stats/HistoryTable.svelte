<script lang="ts">
  import type { FighterBout } from '../api/types';
  import { resolveColor } from '../api/types';
  import { techniques, fighters } from '../../stores';
  import { HIT_ZONES } from '../player/HitZonePicker.svelte';

  import DateRangePicker from '../ui/DateRangePicker.svelte';

  export interface TableFilters {
    date_start: string;
    date_end: string;
    video_id: string;
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
    sort_col: string;
    sort_dir: 'asc' | 'desc';
  }

  interface Props {
    bouts: FighterBout[];
    filters: TableFilters;
    opponents: Array<{ id: string; name: string }>;
    fightDates?: Set<string>;
    onfilter: (f: TableFilters) => void;
    onnavigate: (videoId: string, timeStartMs?: number) => void;
  }

  let { bouts, filters, opponents, fightDates = new Set(), onfilter, onnavigate }: Props = $props();

  // Visible column config
  type ColKey = 'date' | 'opponent' | 'score' | 'my_tech' | 'my_result' | 'my_zone' | 'opp_tech' | 'opp_result' | 'opp_zone';
  const COL_LABELS: Record<ColKey, string> = {
    date: 'Дата', opponent: 'Оппонент', score: 'Счёт',
    my_tech: 'Мой приём', my_result: 'Мой рез.', my_zone: 'Моя зона',
    opp_tech: 'Приём опп.', opp_result: 'Рез. опп.', opp_zone: 'Зона опп.',
  };
  // 'date' hidden by default (available in column picker); 'video' shown
  let visibleCols = $state<Set<ColKey>>(new Set(
    (Object.keys(COL_LABELS) as ColKey[])
  ));

  let isGrouped = $state(true); // Grouped by default
  let expandedFights = $state<Set<string>>(new Set());

  function toggleFight(videoId: string, isUnmarked?: boolean) {
    if (isUnmarked) return;
    const next = new Set(expandedFights);
    if (next.has(videoId)) {
      next.delete(videoId);
    } else {
      next.add(videoId);
    }
    expandedFights = next;
  }

  function getOpponentInfo(opponentId: string) {
    const f = $fighters.find(x => x.id === opponentId);
    return {
      avatar_url: f?.avatar_url || '',
      color: f?.color || null
    };
  }

  interface FightGroup {
    video_id: string;
    video_date: string;
    opponent_id: string;
    opponent_name: string;
    my_score: number;
    opponent_score: number;
    bouts: FighterBout[];
    is_unmarked?: boolean;
  }

  let groups = $derived.by(() => {
    const list: FightGroup[] = [];
    const map = new Map<string, FightGroup>();
    for (const b of bouts) {
      let g = map.get(b.video_id);
      if (!g) {
        g = {
          video_id: b.video_id,
          video_date: b.video_date,
          opponent_id: b.opponent_id,
          opponent_name: b.opponent_name,
          my_score: 0,
          opponent_score: 0,
          bouts: [],
          is_unmarked: b.is_unmarked
        };
        map.set(b.video_id, g);
        list.push(g);
      }
      if (!b.is_unmarked) {
        g.my_score += b.my_score;
        g.opponent_score += b.opponent_score;
        g.bouts.push(b);
      }
    }
    return list;
  });

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
  <table class="table">
    <thead>
      <tr class="header-row">

        {#if visibleCols.has('date')}
        <th class="th sortable" onclick={() => toggleSort('video_date')}>
          <span>ДАТА <span class="sort-icon">{sortIcon('video_date')}</span></span>
          <!-- svelte-ignore a11y_click_events_have_key_events -->
          <!-- svelte-ignore a11y_no_static_element_interactions -->
          <div onclick={(e) => e.stopPropagation()}>
            <DateRangePicker
              bind:dateStart={filters.date_start}
              bind:dateEnd={filters.date_end}
              {fightDates}
              onchange={(start, end) => {
                onfilter({ ...filters, date_start: start, date_end: end });
              }}
            />
          </div>
        </th>
        {/if}
        {#if visibleCols.has('opponent')}
        <th class="th sortable" onclick={() => toggleSort('opponent_name')}>
          <span>ОППОНЕНТ <span class="sort-icon">{sortIcon('opponent_name')}</span></span>
          <select class="filter-input" value={filters.opponent_id}
            onchange={(e) => setFilter('opponent_id', (e.target as HTMLSelectElement).value)}
            onclick={(e) => e.stopPropagation()}>
            <option value="">ВСЕ</option>
            {#each opponents as opp}
              <option value={opp.id}>{opp.name}</option>
            {/each}
          </select>
        </th>
        {/if}
        {#if visibleCols.has('score')}
        <th class="th sortable score-th" onclick={() => toggleSort('my_score')}>
          <span>СЧЁТ <span class="sort-icon">{sortIcon('my_score')}</span></span>
          <input class="filter-input" type="text" placeholder="Фильтр…" value={filters.score}
            oninput={(e) => setFilter('score', (e.target as HTMLInputElement).value)}
            onclick={(e) => e.stopPropagation()} />
        </th>
        {/if}
        {#if visibleCols.has('my_tech')}
        <th class="th sortable" onclick={() => toggleSort('my_technique_name')}>
          <span>МОЙ ПРИЁМ <span class="sort-icon">{sortIcon('my_technique_name')}</span></span>
          <select class="filter-input" value={filters.my_technique}
            onchange={(e) => setFilter('my_technique', (e.target as HTMLSelectElement).value)}
            onclick={(e) => e.stopPropagation()}>
            <option value="">ВСЕ</option>
            {#each $techniques as t (t.id)}
              <option value={t.name}>{t.name}</option>
            {/each}
          </select>
        </th>
        {/if}
        {#if visibleCols.has('my_result')}
        <th class="th sortable" onclick={() => toggleSort('my_result')}>
          <span>МОЙ РЕЗ. <span class="sort-icon">{sortIcon('my_result')}</span></span>
          <select class="filter-input" value={filters.my_result}
            onchange={(e) => setFilter('my_result', (e.target as HTMLSelectElement).value)}
            onclick={(e) => e.stopPropagation()}>
            <option value="">ВСЕ</option>
            <option value="hit">ПОПАЛ</option>
            <option value="miss">ПРОМАХ</option>
            <option value="blocked">ЗАБЛОКИРОВАЛИ</option>
            <option value="late">ОПОЗДАЛ</option>
            <option value="no_strike">НЕ БИЛ</option>
            <option value="disqualification">НЕКВАЛИФИКАЦИЯ</option>
            <option value="afterblow">АФТЕРБЛОУ</option>
          </select>
        </th>
        {/if}
        {#if visibleCols.has('my_zone')}
        <th class="th sortable" onclick={() => toggleSort('my_hit_zone')}>
          <span>МОЯ ЗОНА <span class="sort-icon">{sortIcon('my_hit_zone')}</span></span>
          <select class="filter-input" value={filters.my_zone}
            onchange={(e) => setFilter('my_zone', (e.target as HTMLSelectElement).value)}
            onclick={(e) => e.stopPropagation()}>
            <option value="">ВСЕ</option>
            {#each HIT_ZONES as zone}
              <option value={zone}>{zone}</option>
            {/each}
          </select>
        </th>
        {/if}
        {#if visibleCols.has('opp_tech')}
        <th class="th sortable" onclick={() => toggleSort('opponent_technique_name')}>
          <span>ПРИЁМ ОПП. <span class="sort-icon">{sortIcon('opponent_technique_name')}</span></span>
          <select class="filter-input" value={filters.opponent_technique}
            onchange={(e) => setFilter('opponent_technique', (e.target as HTMLSelectElement).value)}
            onclick={(e) => e.stopPropagation()}>
            <option value="">ВСЕ</option>
            {#each $techniques as t (t.id)}
              <option value={t.name}>{t.name}</option>
            {/each}
          </select>
        </th>
        {/if}
        {#if visibleCols.has('opp_result')}
        <th class="th sortable" onclick={() => toggleSort('opponent_result')}>
          <span>РЕЗ. ОПП. <span class="sort-icon">{sortIcon('opponent_result')}</span></span>
          <select class="filter-input" value={filters.opponent_result}
            onchange={(e) => setFilter('opponent_result', (e.target as HTMLSelectElement).value)}
            onclick={(e) => e.stopPropagation()}>
            <option value="">ВСЕ</option>
            <option value="hit">ПОПАЛ</option>
            <option value="miss">ПРОМАХ</option>
            <option value="blocked">ЗАБЛОКИРОВАЛИ</option>
            <option value="late">ОПОЗДАЛ</option>
            <option value="no_strike">НЕ БИЛ</option>
            <option value="disqualification">НЕКВАЛИФИКАЦИЯ</option>
            <option value="afterblow">АФТЕРБЛОУ</option>
          </select>
        </th>
        {/if}
        {#if visibleCols.has('opp_zone')}
        <th class="th sortable" onclick={() => toggleSort('opponent_hit_zone')}>
          <span>ЗОНА ОПП. <span class="sort-icon">{sortIcon('opponent_hit_zone')}</span></span>
          <select class="filter-input" value={filters.opponent_zone}
            onchange={(e) => setFilter('opponent_zone', (e.target as HTMLSelectElement).value)}
            onclick={(e) => e.stopPropagation()}>
            <option value="">ВСЕ</option>
            {#each HIT_ZONES as zone}
              <option value={zone}>{zone}</option>
            {/each}
          </select>
        </th>
        {/if}
        <th class="th th--nav">
          <button class="toggle-mode-btn" onclick={() => { isGrouped = !isGrouped; }} title={isGrouped ? "Показать списком" : "Сгруппировать по боям"}>
            {#if isGrouped}
              <!-- Grouped mode icon -->
              <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                <path d="M4 6h16M4 12h16M8 18h12"/>
                <circle cx="2" cy="6" r="1" fill="currentColor"/>
                <circle cx="2" cy="12" r="1" fill="currentColor"/>
                <circle cx="6" cy="18" r="1" fill="currentColor"/>
              </svg>
            {:else}
              <!-- Flat list icon -->
              <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                <line x1="8" y1="6" x2="21" y2="6"></line>
                <line x1="8" y1="12" x2="21" y2="12"></line>
                <line x1="8" y1="18" x2="21" y2="18"></line>
                <circle cx="4" cy="6" r="1.5"></circle>
                <circle cx="4" cy="12" r="1.5"></circle>
                <circle cx="4" cy="18" r="1.5"></circle>
              </svg>
            {/if}
          </button>
        </th>
      </tr>
    </thead>
    <tbody>
      {#if isGrouped}
        {#each groups as group (group.video_id)}
          <!-- Group header row -->
          <!-- svelte-ignore a11y_click_events_have_key_events -->
          <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
          <tr class="body-row group-header-row" class:unclickable={group.is_unmarked} onclick={() => toggleFight(group.video_id, group.is_unmarked)}>
            {#if visibleCols.has('date')}
              <td class="td date-cell group-video-cell">
                <span class="chevron-icon" class:expanded={expandedFights.has(group.video_id)} style:visibility={group.is_unmarked ? 'hidden' : 'visible'}>
                  <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round">
                    <path d="M9 18l6-6-6-6"/>
                  </svg>
                </span>
                {formatDate(group.video_date)}
              </td>
            {/if}
            {#if visibleCols.has('opponent')}
              <td class="td">
                <div class="opponent-cell-content">
                  <div class="opp-avatar" style:background={resolveColor(group.opponent_id, getOpponentInfo(group.opponent_id).color)}>
                    <svg class="opp-avatar-icon" width="12" height="12" viewBox="0 0 24 24" fill="none" aria-hidden="true">
                      <circle cx="12" cy="8" r="4" stroke="#fff" stroke-width="1.5" opacity="0.6"/>
                      <path d="M4 20c0-4 3.6-7 8-7s8 3 8 7" stroke="#fff" stroke-width="1.5" stroke-linecap="round" opacity="0.6"/>
                    </svg>
                    {#if getOpponentInfo(group.opponent_id).avatar_url}
                      <img class="opp-avatar-img" src={getOpponentInfo(group.opponent_id).avatar_url} alt={group.opponent_name}
                        onerror={(e) => { (e.target as HTMLImageElement).style.display = 'none'; }} />
                    {/if}
                  </div>
                  <span class="opp-name">{group.opponent_name}</span>
                </div>
              </td>
            {/if}
            {#if visibleCols.has('score')}
              <td class="td score">
                {#if group.is_unmarked}
                  —
                {:else}
                  <span class="group-score">
                    <span class:win={group.my_score > group.opponent_score} class:loss={group.my_score < group.opponent_score}>{group.my_score}</span>
                    <span class="sep">:</span>
                    <span class:win={group.my_score > group.opponent_score} class:loss={group.my_score < group.opponent_score}>{group.opponent_score}</span>
                  </span>
                {/if}
              </td>
            {/if}
            {#if visibleCols.has('my_tech')}<td class="td">—</td>{/if}
            {#if visibleCols.has('my_result')}
              <td class="td">
                {#if group.is_unmarked}
                  <span class="group-result-badge unmarked">Неразмечено</span>
                {:else}
                  <span class="group-result-badge" class:win={group.my_score > group.opponent_score} class:loss={group.my_score < group.opponent_score} class:draw={group.my_score === group.opponent_score}>
                    {group.my_score > group.opponent_score ? 'Победа' : group.my_score < group.opponent_score ? 'Поражение' : 'Ничья'}
                  </span>
                {/if}
              </td>
            {/if}
            {#if visibleCols.has('my_zone')}<td class="td">—</td>{/if}
            {#if visibleCols.has('opp_tech')}<td class="td">—</td>{/if}
            {#if visibleCols.has('opp_result')}<td class="td">—</td>{/if}
            {#if visibleCols.has('opp_zone')}<td class="td">—</td>{/if}
            <td class="td td--nav">
              <button class="nav-btn" onclick={(e) => { e.stopPropagation(); onnavigate(group.video_id); }} title="Открыть видео">→</button>
            </td>
          </tr>
          {#if expandedFights.has(group.video_id)}
            {#each group.bouts as bout (bout.id)}
              <tr class="body-row bout-sub-row">

                {#if visibleCols.has('date')}<td class="td date-cell sub-cell">{formatDate(bout.video_date)}</td>{/if}
                {#if visibleCols.has('opponent')}<td class="td sub-cell">{bout.opponent_name}</td>{/if}
                {#if visibleCols.has('score')}
                  <td class="td score sub-cell">
                    <span class:win={bout.my_score > bout.opponent_score} class:loss={bout.my_score < bout.opponent_score}>{bout.my_score}</span>
                    <span class="sep">:</span>
                    <span class:win={bout.my_score > bout.opponent_score} class:loss={bout.my_score < bout.opponent_score}>{bout.opponent_score}</span>
                  </td>
                {/if}
                {#if visibleCols.has('my_tech')}<td class="td sub-cell">{bout.my_technique_name ?? '—'}</td>{/if}
                {#if visibleCols.has('my_result')}
                  <td class="td sub-cell">
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
                {#if visibleCols.has('my_zone')}<td class="td zone-cell sub-cell">{(bout.my_hit_zone ?? '').split(':')[0] || '—'}</td>{/if}
                {#if visibleCols.has('opp_tech')}<td class="td sub-cell">{bout.opponent_technique_name ?? '—'}</td>{/if}
                {#if visibleCols.has('opp_result')}
                  <td class="td sub-cell">
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
                {#if visibleCols.has('opp_zone')}<td class="td zone-cell sub-cell">{(bout.opponent_hit_zone ?? '').split(':')[0] || '—'}</td>{/if}
                <td class="td td--nav sub-cell">
                  <button class="nav-btn" onclick={() => onnavigate(bout.video_id, bout.time_start_ms)} title="Открыть видео">→</button>
                </td>
              </tr>
            {/each}
          {/if}
        {/each}
      {:else}
        {#each bouts as bout (bout.id)}
          <tr class="body-row">

            {#if visibleCols.has('date')}<td class="td date-cell">{formatDate(bout.video_date)}</td>{/if}
            {#if visibleCols.has('opponent')}<td class="td">{bout.opponent_name}</td>{/if}
            {#if visibleCols.has('score')}
              <td class="td score">
                {#if bout.is_unmarked}
                  —
                {:else}
                  <span class:win={bout.my_score > bout.opponent_score} class:loss={bout.my_score < bout.opponent_score}>{bout.my_score}</span>
                  <span class="sep">:</span>
                  <span class:win={bout.my_score > bout.opponent_score} class:loss={bout.my_score < bout.opponent_score}>{bout.opponent_score}</span>
                {/if}
              </td>
            {/if}
            {#if visibleCols.has('my_tech')}<td class="td">{bout.my_technique_name ?? '—'}</td>{/if}
            {#if visibleCols.has('my_result')}
              <td class="td">
                {#if bout.is_unmarked}
                  <span class="result-badge unmarked">Неразмечено</span>
                {:else}
                  <span class="result-badge"
                    class:hit={bout.my_result === 'hit'}
                    class:miss={bout.my_result === 'miss'}
                    class:blocked={bout.my_result === 'blocked'}
                    class:late={bout.my_result === 'late'}
                    class:no-strike={bout.my_result === 'no_strike'}
                    class:disqualification={bout.my_result === 'disqualification'}
                    class:afterblow={bout.my_result === 'afterblow'}
                  >{resultLabel(bout.my_result)}</span>
                {/if}
              </td>
            {/if}
            {#if visibleCols.has('my_zone')}<td class="td zone-cell">{(bout.my_hit_zone ?? '').split(':')[0] || '—'}</td>{/if}
            {#if visibleCols.has('opp_tech')}<td class="td">{bout.opponent_technique_name ?? '—'}</td>{/if}
            {#if visibleCols.has('opp_result')}
              <td class="td">
                {#if bout.is_unmarked}
                  —
                {:else}
                  <span class="result-badge"
                    class:hit={bout.opponent_result === 'hit'}
                    class:miss={bout.opponent_result === 'miss'}
                    class:blocked={bout.opponent_result === 'blocked'}
                    class:late={bout.opponent_result === 'late'}
                    class:no-strike={bout.opponent_result === 'no_strike'}
                    class:disqualification={bout.opponent_result === 'disqualification'}
                    class:afterblow={bout.opponent_result === 'afterblow'}
                  >{resultLabel(bout.opponent_result)}</span>
                {/if}
              </td>
            {/if}
            {#if visibleCols.has('opp_zone')}<td class="td zone-cell">{(bout.opponent_hit_zone ?? '').split(':')[0] || '—'}</td>{/if}
            <td class="td td--nav">
              <button class="nav-btn" onclick={() => onnavigate(bout.video_id, bout.time_start_ms)} title="Открыть видео">→</button>
            </td>
          </tr>
        {/each}
      {/if}
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
    overflow-y: auto;
    background: var(--surface-solid);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-lg);
    box-shadow: none;
    height: 677px;
    box-sizing: border-box;
  }

  .table {
    width: 100%;
    border-collapse: collapse;
    font-size: 0.82rem;
  }

  .th {
    background: #0f172a;
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
    width: 50px;
    cursor: default;
    padding: 14px 4px 6px;
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

  .score-th {
    text-align: center;
  }

  .score-th span {
    justify-content: center;
  }

  .score {
    font-variant-numeric: tabular-nums;
    white-space: nowrap;
    text-align: center;
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



  .zone-cell {
    font-size: 0.8rem;
    color: var(--text-secondary);
  }



  .date-cell {
    font-size: 0.78rem;
    color: var(--text-secondary);
    white-space: nowrap;
    font-variant-numeric: tabular-nums;
  }

  /* Toggle mode button */
  .toggle-mode-btn {
    background: none;
    border: none;
    border-radius: var(--radius-sm);
    color: var(--text-secondary);
    width: 24px;
    height: 24px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: var(--transition);
    margin: 0 auto;
  }

  .toggle-mode-btn:hover {
    color: var(--text-primary);
    background: var(--surface-hover);
  }

  /* Chevron animation */
  .chevron-icon {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    transition: transform 0.2s ease;
    margin-right: 8px;
    color: var(--text-secondary);
    vertical-align: middle;
  }
  .chevron-icon.expanded {
    transform: rotate(90deg);
    color: var(--accent-yellow);
  }

  /* Group header rows */
  .group-header-row {
    cursor: pointer;
    background: rgba(255, 255, 255, 0.015);
  }
  .group-header-row:hover {
    background: var(--surface-hover);
  }

  /* Bout sub-rows nested within groups */
  .bout-sub-row {
    background: rgba(0, 0, 0, 0.2);
    border-left: 2px solid var(--accent-yellow);
  }
  .bout-sub-row:hover {
    background: rgba(255, 255, 255, 0.03) !important;
  }

  .sub-cell {
    opacity: 0.85;
  }

  /* Group Result badges */
  .group-result-badge {
    display: inline-block;
    padding: 2px 8px;
    border-radius: 4px;
    font-size: 0.72rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    text-align: center;
    min-width: 80px;
  }
  .group-result-badge.win {
    background: rgba(16, 185, 129, 0.15);
    color: #10b981;
  }
  .group-result-badge.loss {
    background: rgba(239, 68, 68, 0.15);
    color: #ef4444;
  }
  .group-result-badge.draw {
    background: rgba(156, 163, 175, 0.15);
    color: #9ca3af;
  }

  /* Opponent cell style */
  .opponent-cell-content {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .opp-avatar {
    width: 24px;
    height: 24px;
    border-radius: 50%;
    border: 1px solid rgba(255, 255, 255, 0.15);
    position: relative;
    display: flex;
    align-items: center;
    justify-content: center;
    overflow: hidden;
    flex-shrink: 0;
  }

  .opp-avatar-icon {
    position: absolute;
    pointer-events: none;
  }

  .opp-avatar-img {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .opp-name {
    font-weight: 500;
  }

  .unclickable {
    cursor: default !important;
  }

  .group-result-badge.unmarked,
  .result-badge.unmarked {
    background: rgba(156, 163, 175, 0.15) !important;
    color: #9ca3af !important;
  }
</style>
