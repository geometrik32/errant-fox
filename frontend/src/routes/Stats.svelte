<script lang="ts">
  import { onMount } from 'svelte';
  import { fighters, currentUser } from '../stores';
  import { getFighterBouts } from '../lib/api/fighters';
  import { resolveColor } from '../lib/api/types';
  import FighterSidebar from '../lib/stats/FighterSidebar.svelte';
  import HistoryTable from '../lib/stats/HistoryTable.svelte';
  import QuickStats from '../lib/stats/QuickStats.svelte';
  import FrequencyChart from '../lib/stats/FrequencyChart.svelte';
  import ResultsChart from '../lib/stats/ResultsChart.svelte';
  import ScoreChart from '../lib/stats/ScoreChart.svelte';
  import BodySilhouette from '../lib/stats/BodySilhouette.svelte';
  import TopTechniques from '../lib/stats/TopTechniques.svelte';
  import RecentOpponents from '../lib/stats/RecentOpponents.svelte';
  import type { Fighter, FighterBout } from '../lib/api/types';
  import { buildVideoLabels } from '../lib/api/types';
  import type { TableFilters } from '../lib/stats/HistoryTable.svelte';

  let selectedFighter = $state<Fighter | null>(null);
  let rawBouts = $state<FighterBout[]>([]);
  let loading = $state(false);
  let errorMsg = $state('');

  const defaultFilters: TableFilters = {
    date: '',
    opponent_id: '',
    opponent_name: '',
    my_technique: '',
    my_result: '',
    opponent_technique: '',
    opponent_result: '',
    sort_col: 'video_date',
    sort_dir: 'desc',
  };

  let tableFilters = $state<TableFilters>({ ...defaultFilters });
  let zoneFilter = $state('');

  // The single derived store — all dashboard components read from this
  let filteredBouts = $derived.by(() => {
    let result = rawBouts;

    if (zoneFilter) {
      result = result.filter(b => {
        const mz = (b.my_hit_zone ?? '').split(':')[0];
        const oz = (b.opponent_hit_zone ?? '').split(':')[0];
        return mz === zoneFilter || oz === zoneFilter;
      });
    }

    if (tableFilters.date)
      result = result.filter(b => b.video_date === tableFilters.date);
    if (tableFilters.opponent_id)
      result = result.filter(b => b.opponent_id === tableFilters.opponent_id);
    if (tableFilters.my_technique)
      result = result.filter(b =>
        (b.my_technique_name ?? '').toLowerCase().includes(tableFilters.my_technique.toLowerCase())
      );
    if (tableFilters.my_result)
      result = result.filter(b => b.my_result === tableFilters.my_result);
    if (tableFilters.opponent_technique)
      result = result.filter(b =>
        (b.opponent_technique_name ?? '').toLowerCase().includes(tableFilters.opponent_technique.toLowerCase())
      );
    if (tableFilters.opponent_result)
      result = result.filter(b => b.opponent_result === tableFilters.opponent_result);

    if (tableFilters.sort_col) {
      const col = tableFilters.sort_col as keyof FighterBout;
      const dir = tableFilters.sort_dir;
      result = [...result].sort((a, b) => {
        const va = a[col];
        const vb = b[col];
        if (va === null || va === undefined) return 1;
        if (vb === null || vb === undefined) return -1;
        if (typeof va === 'string' && typeof vb === 'string')
          return dir === 'asc' ? va.localeCompare(vb) : vb.localeCompare(va);
        return dir === 'asc'
          ? (va as number) - (vb as number)
          : (vb as number) - (va as number);
      });
    }

    return result;
  });

  // unique opponents from raw (unfiltered) bouts for dropdowns
  let opponents = $derived.by(() => {
    const map = new Map<string, string>();
    for (const b of rawBouts) map.set(b.opponent_id, b.opponent_name);
    return [...map.entries()].map(([id, name]) => ({ id, name }));
  });

  let videoLabels = $derived(
    selectedFighter ? buildVideoLabels(rawBouts, selectedFighter.display_name) : new Map<string, string>()
  );

  let firstBoutDate = $derived(
    rawBouts.length > 0
      ? rawBouts.reduce((min, b) => b.video_date < min ? b.video_date : min, rawBouts[0].video_date)
      : null
  );

  async function selectFighter(fighter: Fighter) {
    if (selectedFighter?.id === fighter.id) return;
    selectedFighter = fighter;
    rawBouts = [];
    tableFilters = { ...defaultFilters };
    zoneFilter = '';
    loading = true;
    errorMsg = '';
    try {
      rawBouts = await getFighterBouts(fighter.id);
    } catch (e) {
      errorMsg = e instanceof Error ? e.message : 'Ошибка загрузки данных';
    } finally {
      loading = false;
    }
  }

  // Auto-select the fighter matching the current user on mount
  onMount(() => {
    const me = $currentUser;
    if (me && !selectedFighter) {
      const myFighter = $fighters.find(f => f.id === me.id);
      if (myFighter) selectFighter(myFighter);
    }
  });

  function handleFilter(filters: TableFilters) {
    tableFilters = filters;
  }

  function handleNavigate(videoId: string, timeStartMs?: number) {
    const t = timeStartMs ? `?t=${timeStartMs}` : '';
    window.location.hash = '#/player/' + videoId + t;
  }

  function formatDate(d: string): string {
    if (!d) return '';
    return new Date(d).toLocaleDateString('ru-RU', { year: 'numeric', month: 'long', day: 'numeric' });
  }

  let showFighterDropdown = $state(false);
</script>

<div class="stats-layout">
  <div class="dashboard">
    {#if !selectedFighter}
      <div class="empty-state">
        <div class="empty-icon">👤</div>
        <div class="empty-text">Выберите бойца для просмотра статистики</div>
      </div>
    {:else if loading}
      <div class="loading">Загрузка данных…</div>
    {:else if errorMsg}
      <div class="error">{errorMsg}</div>
    {:else}
      <!-- Fighter header & Quick stats -->
      <div class="dashboard-top">
        <!-- svelte-ignore a11y_interactive_supports_focus -->
        <div class="fighter-hero glass-card"
          role="button" tabindex="0"
          onclick={() => showFighterDropdown = !showFighterDropdown}
          onkeydown={(e) => e.key === 'Enter' && (showFighterDropdown = !showFighterDropdown)}
        >
          <div class="avatar-wrap" style:background={resolveColor(selectedFighter.id, selectedFighter.color)}>
            <svg class="avatar-icon" width="40" height="40" viewBox="0 0 24 24" fill="none" aria-hidden="true">
              <circle cx="12" cy="8" r="4" stroke="#fff" stroke-width="1.5"/>
              <path d="M4 20c0-4 3.6-7 8-7s8 3 8 7" stroke="#fff" stroke-width="1.5" stroke-linecap="round"/>
            </svg>
            <img class="avatar-img" src={selectedFighter.avatar_url} alt={selectedFighter.display_name}
              onerror={(e) => { (e.target as HTMLImageElement).style.display = 'none'; }} />
          </div>
          <div class="fighter-info">
            <div class="greeting">Статистика бойца</div>
            <div class="fighter-name">
              {selectedFighter.display_name}
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true" style="vertical-align: middle; margin-left: 4px; transform: {showFighterDropdown ? 'rotate(180deg)' : 'none'}; transition: transform 0.2s;">
                <path d="M6 9l6 6 6-6"/>
              </svg>
            </div>
            {#if firstBoutDate}
              <div class="fighter-since">с {formatDate(firstBoutDate)}</div>
            {/if}
          </div>

          {#if showFighterDropdown}
            <!-- svelte-ignore a11y_click_events_have_key_events -->
            <!-- svelte-ignore a11y_no_static_element_interactions -->
            <div class="fighter-dropdown" onclick={(e) => e.stopPropagation()}>
              <div class="dropdown-header">Выберите бойца</div>
              <div class="dropdown-list">
                {#each $fighters as f (f.id)}
                  <button class="fighter-opt" class:selected={selectedFighter.id === f.id} onclick={() => { selectFighter(f); showFighterDropdown = false; }}>
                    <div class="opt-avatar" style:background={resolveColor(f.id, f.color)}></div>
                    <span class="opt-name">{f.display_name}</span>
                  </button>
                {/each}
              </div>
            </div>
          {/if}
        </div>

        <QuickStats bouts={filteredBouts} />
      </div>

      <!-- Main row -->
      <div class="main-row">
        <div class="main-chart-wrapper">
          <ScoreChart bouts={filteredBouts} {videoLabels} />
        </div>
        <div class="side-panel-wrapper">
          <RecentOpponents bouts={filteredBouts} limit={8} />
        </div>
      </div>

      <!-- Charts row -->
      <div class="charts-row">
        <FrequencyChart bouts={filteredBouts} />
        <ResultsChart bouts={filteredBouts} {videoLabels} />
        <TopTechniques bouts={filteredBouts} />
      </div>

      <!-- Body silhouettes -->
      <div class="silhouettes-row">
        <BodySilhouette bouts={filteredBouts} type="dealt" selectedZone={zoneFilter} onzoneclick={(z) => { zoneFilter = z; }} />
        <BodySilhouette bouts={filteredBouts} type="received" selectedZone={zoneFilter} onzoneclick={(z) => { zoneFilter = z; }} />
      </div>

      <!-- History table -->
      <div class="table-section">
        <HistoryTable
          bouts={filteredBouts}
          filters={tableFilters}
          {opponents}
          {videoLabels}
          onfilter={handleFilter}
          onnavigate={handleNavigate}
        />
      </div>
    {/if}
  </div>
</div>

<style>
  .stats-layout {
    display: flex;
    min-height: 0;
  }

  .dashboard {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 20px;
  }

  /* Empty / loading states */
  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 12px;
    padding: 80px 24px;
    color: #4a6280;
  }

  .empty-icon {
    font-size: 2.5rem;
    opacity: 0.4;
  }

  .empty-text {
    font-size: 0.9rem;
  }

  .loading, .error {
    text-align: center;
    padding: 64px;
    font-size: 0.9rem;
  }

  .loading { color: #4a6280; }
  .error   { color: #e05252; }

  /* Fighter header */
  .dashboard-top {
    display: flex;
    gap: 20px;
    align-items: stretch;
  }

  .fighter-hero {
    flex: 0 0 280px;
    padding: 30px 24px;
    background: var(--surface);
    backdrop-filter: var(--glass-blur);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-sm);
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    text-align: center;
    gap: 16px;
    cursor: pointer;
    position: relative;
    user-select: none;
    transition: var(--transition);
  }

  .fighter-hero:hover {
    border-color: var(--text-secondary);
  }

  .fighter-dropdown {
    position: absolute;
    top: calc(100% + 8px);
    left: 0;
    right: 0;
    background: var(--surface-solid);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-md);
    box-shadow: var(--shadow-lg);
    z-index: 50;
    display: flex;
    flex-direction: column;
    max-height: 400px;
    overflow: hidden;
    cursor: default;
  }

  .dropdown-header {
    padding: 12px 16px;
    font-size: 0.75rem;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--text-secondary);
    border-bottom: 1px solid var(--border-color);
    text-align: left;
  }

  .dropdown-list {
    overflow-y: auto;
    padding: 8px;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .fighter-opt {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 8px 12px;
    background: transparent;
    border: none;
    border-radius: var(--radius-sm);
    cursor: pointer;
    text-align: left;
    transition: var(--transition);
  }

  .fighter-opt:hover {
    background: var(--surface-hover);
  }

  .fighter-opt.selected {
    background: rgba(219, 132, 31, 0.12);
  }

  .opt-avatar {
    width: 24px;
    height: 24px;
    border-radius: 50%;
    flex-shrink: 0;
    border: 1px solid rgba(255,255,255,0.1);
  }

  .opt-name {
    font-size: 0.9rem;
    color: var(--text-primary);
    font-weight: 500;
  }

  .avatar-wrap {
    width: 90px;
    height: 90px;
    border-radius: 50%;
    border: 3px solid rgba(255,255,255,0.2);
    flex-shrink: 0;
    position: relative;
    display: flex;
    align-items: center;
    justify-content: center;
    overflow: hidden;
  }

  .avatar-icon {
    position: absolute;
    pointer-events: none;
    opacity: 0.6;
  }

  .avatar-img {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .greeting {
    font-size: 0.85rem;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    margin-bottom: 6px;
  }

  .fighter-name {
    font-size: 1.6rem;
    font-weight: 700;
    color: var(--text-primary);
    line-height: 1.2;
  }

  .fighter-since {
    font-size: 0.9rem;
    color: var(--text-secondary);
    margin-top: 4px;
  }

  /* Main Row */
  .main-row {
    display: flex;
    gap: 20px;
  }
  
  .main-chart-wrapper {
    flex: 2;
    min-width: 0;
  }
  
  .side-panel-wrapper {
    flex: 1;
    min-width: 0;
  }

  /* Charts 3-column grid */
  .charts-row {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 20px;
  }

  /* Silhouettes 2-column */
  .silhouettes-row {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 20px;
  }

  /* History table section */
  .table-section {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  @media (max-width: 1024px) {
    .stats-layout {
      flex-direction: column;
    }
    .dashboard-top {
      flex-direction: column;
    }
    .fighter-hero {
      flex: none;
      flex-direction: row;
      text-align: left;
      justify-content: flex-start;
      padding: 20px;
    }
    .avatar-wrap {
      width: 60px;
      height: 60px;
    }
    .charts-row, .silhouettes-row, .main-row {
      flex-direction: column;
      grid-template-columns: 1fr;
    }
  }
</style>
