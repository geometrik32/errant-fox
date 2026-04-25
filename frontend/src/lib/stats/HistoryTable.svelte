<script lang="ts">
  import type { FighterBout } from '../api/types';

  export interface TableFilters {
    date: string;
    opponent_id: string;
    opponent_name: string;
    my_technique: string;
    my_result: string;
    opponent_technique: string;
    opponent_result: string;
    sort_col: string;
    sort_dir: 'asc' | 'desc';
  }

  interface Props {
    bouts: FighterBout[];
    filters: TableFilters;
    opponents: Array<{ id: string; name: string }>;
    onfilter: (f: TableFilters) => void;
    onnavigate: (videoId: string) => void;
  }

  let { bouts, filters, opponents, onfilter, onnavigate }: Props = $props();

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
</script>

<div class="table-wrap">
  <table class="table">
    <thead>
      <tr class="header-row">
        <th class="th sortable" onclick={() => toggleSort('video_date')}>
          <span>Дата <span class="sort-icon">{sortIcon('video_date')}</span></span>
          <input
            class="filter-input"
            type="date"
            value={filters.date}
            oninput={(e) => setFilter('date', (e.target as HTMLInputElement).value)}
            onclick={(e) => e.stopPropagation()}
          />
        </th>
        <th class="th sortable" onclick={() => toggleSort('opponent_name')}>
          <span>Оппонент <span class="sort-icon">{sortIcon('opponent_name')}</span></span>
          <select
            class="filter-input"
            value={filters.opponent_id}
            onchange={(e) => setFilter('opponent_id', (e.target as HTMLSelectElement).value)}
            onclick={(e) => e.stopPropagation()}
          >
            <option value="">Все</option>
            {#each opponents as opp}
              <option value={opp.id}>{opp.name}</option>
            {/each}
          </select>
        </th>
        <th class="th sortable" onclick={() => toggleSort('my_score')}>
          <span>Счёт <span class="sort-icon">{sortIcon('my_score')}</span></span>
          <div class="filter-spacer"></div>
        </th>
        <th class="th sortable" onclick={() => toggleSort('my_technique_name')}>
          <span>Мой приём <span class="sort-icon">{sortIcon('my_technique_name')}</span></span>
          <input
            class="filter-input"
            type="text"
            placeholder="Фильтр…"
            value={filters.my_technique}
            oninput={(e) => setFilter('my_technique', (e.target as HTMLInputElement).value)}
            onclick={(e) => e.stopPropagation()}
          />
        </th>
        <th class="th sortable" onclick={() => toggleSort('my_result')}>
          <span>Мой результат <span class="sort-icon">{sortIcon('my_result')}</span></span>
          <select
            class="filter-input"
            value={filters.my_result}
            onchange={(e) => setFilter('my_result', (e.target as HTMLSelectElement).value)}
            onclick={(e) => e.stopPropagation()}
          >
            <option value="">Все</option>
            <option value="hit">Попал</option>
            <option value="miss">Промах</option>
          </select>
        </th>
        <th class="th sortable" onclick={() => toggleSort('opponent_technique_name')}>
          <span>Приём оппонента <span class="sort-icon">{sortIcon('opponent_technique_name')}</span></span>
          <input
            class="filter-input"
            type="text"
            placeholder="Фильтр…"
            value={filters.opponent_technique}
            oninput={(e) => setFilter('opponent_technique', (e.target as HTMLInputElement).value)}
            onclick={(e) => e.stopPropagation()}
          />
        </th>
        <th class="th sortable" onclick={() => toggleSort('opponent_result')}>
          <span>Результат оппонента <span class="sort-icon">{sortIcon('opponent_result')}</span></span>
          <select
            class="filter-input"
            value={filters.opponent_result}
            onchange={(e) => setFilter('opponent_result', (e.target as HTMLSelectElement).value)}
            onclick={(e) => e.stopPropagation()}
          >
            <option value="">Все</option>
            <option value="hit">Попал</option>
            <option value="miss">Промах</option>
          </select>
        </th>
        <th class="th th--nav"></th>
      </tr>
    </thead>
    <tbody>
      {#each bouts as bout (bout.id)}
        <tr class="body-row">
          <td class="td">{formatDate(bout.video_date)}</td>
          <td class="td">{bout.opponent_name}</td>
          <td class="td score">
            <span class:win={bout.my_score > bout.opponent_score}
                  class:loss={bout.my_score < bout.opponent_score}>
              {bout.my_score}
            </span>
            <span class="sep">:</span>
            <span class:win={bout.opponent_score > bout.my_score}
                  class:loss={bout.opponent_score < bout.my_score}>
              {bout.opponent_score}
            </span>
          </td>
          <td class="td">{bout.my_technique_name ?? '—'}</td>
          <td class="td">
            <span class="result-badge" class:hit={bout.my_result === 'hit'} class:miss={bout.my_result === 'miss'}>
              {bout.my_result === 'hit' ? 'Попал' : 'Промах'}
            </span>
          </td>
          <td class="td">{bout.opponent_technique_name ?? '—'}</td>
          <td class="td">
            <span class="result-badge" class:hit={bout.opponent_result === 'hit'} class:miss={bout.opponent_result === 'miss'}>
              {bout.opponent_result === 'hit' ? 'Попал' : 'Промах'}
            </span>
          </td>
          <td class="td td--nav">
            <button class="nav-btn" onclick={() => onnavigate(bout.video_id)} title="Открыть видео">→</button>
          </td>
        </tr>
      {/each}
      {#if bouts.length === 0}
        <tr>
          <td colspan="8" class="td empty">Нет данных</td>
        </tr>
      {/if}
    </tbody>
  </table>
</div>

<style>
  .table-wrap {
    overflow-x: auto;
    border: 1px solid #1f3a57;
    border-radius: 8px;
  }

  .table {
    width: 100%;
    border-collapse: collapse;
    font-size: 0.82rem;
  }

  .th {
    background: #0d1e30;
    color: #4a6280;
    font-weight: 600;
    font-size: 0.72rem;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    padding: 8px 10px 4px;
    text-align: left;
    border-bottom: 1px solid #1f3a57;
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
    color: #6b8aab;
  }

  .th span {
    display: block;
    margin-bottom: 5px;
  }

  .sort-icon {
    color: #2a4f73;
    font-size: 0.7rem;
  }

  .th--nav {
    width: 40px;
    cursor: default;
  }

  .filter-input {
    display: block;
    width: 100%;
    background: #0f2035;
    border: 1px solid #1f3a57;
    border-radius: 4px;
    color: #a0b4c8;
    font-size: 0.75rem;
    padding: 3px 6px;
    outline: none;
    min-width: 0;
    transition: border-color 0.12s;
    margin-bottom: 4px;
  }

  .filter-input:focus {
    border-color: #2a4f73;
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
    border-bottom: 1px solid #142338;
    transition: background 0.1s;
  }

  .body-row:hover {
    background: #0f2035;
  }

  .td {
    padding: 8px 10px;
    color: #a0b4c8;
    vertical-align: middle;
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
    color: #4a6280;
    margin: 0 3px;
  }

  .win {
    color: #4caf82;
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

  .td--nav {
    text-align: center;
  }

  .nav-btn {
    background: none;
    border: 1px solid #2a4f73;
    color: #6b8aab;
    border-radius: 4px;
    width: 28px;
    height: 24px;
    cursor: pointer;
    font-size: 0.9rem;
    transition: background 0.12s, color 0.12s;
    display: flex;
    align-items: center;
    justify-content: center;
    margin: 0 auto;
  }

  .nav-btn:hover {
    background: #1a3050;
    color: #DB841F;
    border-color: #DB841F;
  }
</style>
