<script lang="ts">
  import { fighters } from '../stores';
  import { getFighterBouts } from '../lib/api/fighters';
  import FighterSidebar from '../lib/stats/FighterSidebar.svelte';
  import HistoryTable from '../lib/stats/HistoryTable.svelte';
  import QuickStats from '../lib/stats/QuickStats.svelte';
  import FrequencyChart from '../lib/stats/FrequencyChart.svelte';
  import ResultsChart from '../lib/stats/ResultsChart.svelte';
  import ScoreChart from '../lib/stats/ScoreChart.svelte';
  import BodySilhouette from '../lib/stats/BodySilhouette.svelte';
  import type { Fighter, FighterBout } from '../lib/api/types';
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

  // The single derived store — all dashboard components read from this
  let filteredBouts = $derived.by(() => {
    let result = rawBouts;

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

  function handleFilter(filters: TableFilters) {
    tableFilters = filters;
  }

  function handleNavigate(videoId: string) {
    window.location.hash = '#/player/' + videoId;
  }

  function formatDate(d: string): string {
    if (!d) return '';
    return new Date(d).toLocaleDateString('ru-RU', { year: 'numeric', month: 'long', day: 'numeric' });
  }
</script>

<div class="stats-layout">
  <FighterSidebar
    fighters={$fighters}
    selectedId={selectedFighter?.id ?? null}
    onselect={selectFighter}
  />

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
      <!-- Fighter header -->
      <div class="fighter-header">
        {#if selectedFighter.avatar_url}
          <img class="avatar" src={selectedFighter.avatar_url} alt={selectedFighter.display_name} />
        {:else}
          <div class="avatar-fallback">{selectedFighter.display_name.charAt(0).toUpperCase()}</div>
        {/if}
        <div class="fighter-info">
          <div class="fighter-name">{selectedFighter.display_name}</div>
          {#if firstBoutDate}
            <div class="fighter-since">с {formatDate(firstBoutDate)}</div>
          {/if}
        </div>
      </div>

      <!-- Quick stats -->
      <QuickStats bouts={filteredBouts} />

      <!-- Charts row -->
      <div class="charts-row">
        <FrequencyChart bouts={filteredBouts} />
        <ResultsChart bouts={filteredBouts} />
        <ScoreChart bouts={filteredBouts} />
      </div>

      <!-- Body silhouettes -->
      <div class="silhouettes-row">
        <BodySilhouette bouts={filteredBouts} type="dealt" />
        <BodySilhouette bouts={filteredBouts} type="received" />
      </div>

      <!-- History table -->
      <div class="table-section">
        <div class="section-title">История боёв</div>
        <HistoryTable
          bouts={filteredBouts}
          filters={tableFilters}
          {opponents}
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
    gap: 28px;
    align-items: flex-start;
    min-height: 0;
  }

  .dashboard {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 18px;
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
  .fighter-header {
    display: flex;
    align-items: center;
    gap: 14px;
    padding: 16px 20px;
    background: #0f2035;
    border: 1px solid #1f3a57;
    border-radius: 10px;
  }

  .avatar {
    width: 54px;
    height: 54px;
    border-radius: 50%;
    object-fit: cover;
    border: 2px solid #2a4f73;
    flex-shrink: 0;
  }

  .avatar-fallback {
    width: 54px;
    height: 54px;
    border-radius: 50%;
    background: #1a3050;
    border: 2px solid #2a4f73;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 1.4rem;
    font-weight: 700;
    color: #a0b4c8;
    flex-shrink: 0;
  }

  .fighter-name {
    font-size: 1.2rem;
    font-weight: 700;
    color: #e8edf2;
  }

  .fighter-since {
    font-size: 0.8rem;
    color: #4a6280;
    margin-top: 2px;
  }

  /* Charts 3-column grid */
  .charts-row {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 14px;
  }

  /* Silhouettes 2-column */
  .silhouettes-row {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 14px;
  }

  /* History table section */
  .table-section {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .section-title {
    font-size: 0.72rem;
    font-weight: 600;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: #4a6280;
  }
</style>
