<script lang="ts">
  import { fighters, currentUser } from '../stores';
  import { getFighterBouts } from '../lib/api/fighters';
  import { getVideos } from '../lib/api/videos';
  import { resolveColor } from '../lib/api/types';
  import QuickStats from '../lib/stats/QuickStats.svelte';
  import FrequencyChart from '../lib/stats/FrequencyChart.svelte';
  import ResultsChart from '../lib/stats/ResultsChart.svelte';
  import ScoreChart from '../lib/stats/ScoreChart.svelte';
  import BodySilhouette from '../lib/stats/BodySilhouette.svelte';
  import TopTechniques from '../lib/stats/TopTechniques.svelte';
  import RecentOpponents from '../lib/stats/RecentOpponents.svelte';
  import HistoryTable from '../lib/stats/HistoryTable.svelte';
  import RadarChart from '../lib/stats/RadarChart.svelte';
  import type { TableFilters } from '../lib/stats/HistoryTable.svelte';
  import type { Fighter, FighterBout } from '../lib/api/types';

  let selectedFighter = $state<Fighter | null>(null);
  let rawBouts = $state<FighterBout[]>([]);
  let rawVideos = $state<any[]>([]);
  let loading = $state(false);
  let errorMsg = $state('');

  let defaultFilters: TableFilters = {
    date_start: '', date_end: '', video_id: '', opponent_id: '', opponent_name: '',
    my_technique: '', my_result: '', my_zone: '',
    opponent_technique: '', opponent_result: '', opponent_zone: '',
    score: '', date_week: '',
    sort_col: 'video_date', sort_dir: 'desc'
  };

  let tableFilters = $state<TableFilters>({ ...defaultFilters });
  let zoneFilter = $state('');
  let chartXAxisMode = $state<'overview' | 'detail'>('overview');
  let chartScrollRatio = $state(0);

  function getISOWeek(dateStr: string): string {
    if (!dateStr) return '';
    const date = new Date(dateStr);
    const d = new Date(Date.UTC(date.getFullYear(), date.getMonth(), date.getDate()));
    const day = d.getUTCDay() || 7;
    d.setUTCDate(d.getUTCDate() + 4 - day);
    const yearStart = new Date(Date.UTC(d.getUTCFullYear(), 0, 1));
    const week = Math.ceil((((d.getTime() - yearStart.getTime()) / 86400000) + 1) / 7);
    return `${d.getUTCFullYear()}-W${String(week).padStart(2, '0')}`;
  }

  function parseWeeks(value: string): string[] {
    return value.split(',').map(w => w.trim()).filter(Boolean);
  }

  function matchesWeek(dateStr: string, weekFilter: string): boolean {
    const weeks = parseWeeks(weekFilter);
    return weeks.length === 0 || weeks.includes(getISOWeek(dateStr));
  }

  function toggleWeekSelection(current: string, week: string, additive: boolean): string {
    if (!week) return '';
    if (!additive) return current === week ? '' : week;

    const weeks = new Set(parseWeeks(current));
    if (weeks.has(week)) weeks.delete(week);
    else weeks.add(week);
    return [...weeks].sort().join(',');
  }

  let filteredBouts = $derived.by(() => {
    let result = [...rawBouts];
    if (zoneFilter) {
      result = result.filter(b => {
        const mz = (b.my_hit_zone ?? '').split(':')[0];
        const oz = (b.opponent_hit_zone ?? '').split(':')[0];
        return mz === zoneFilter || oz === zoneFilter;
      });
    }
    if (tableFilters.date_start)
      result = result.filter(b => b.video_date >= tableFilters.date_start);
    if (tableFilters.date_end)
      result = result.filter(b => b.video_date <= tableFilters.date_end);
    if (tableFilters.video_id)
      result = result.filter(b => b.video_id === tableFilters.video_id);
    if (tableFilters.opponent_id)
      result = result.filter(b => b.opponent_id === tableFilters.opponent_id);
    if (tableFilters.my_technique)
      result = result.filter(b => b.my_technique_name === tableFilters.my_technique);
    if (tableFilters.my_result)
      result = result.filter(b => b.my_result === tableFilters.my_result);
    if (tableFilters.my_zone)
      result = result.filter(b => (b.my_hit_zone ?? '').split(':')[0] === tableFilters.my_zone);
    if (tableFilters.opponent_technique)
      result = result.filter(b => b.opponent_technique_name === tableFilters.opponent_technique);
    if (tableFilters.opponent_result)
      result = result.filter(b => b.opponent_result === tableFilters.opponent_result);
    if (tableFilters.opponent_zone)
      result = result.filter(b => (b.opponent_hit_zone ?? '').split(':')[0] === tableFilters.opponent_zone);
    if (tableFilters.score)
      result = result.filter(b => `${b.my_score}:${b.opponent_score}`.includes(tableFilters.score));
    if (tableFilters.date_week)
      result = result.filter(b => matchesWeek(b.video_date, tableFilters.date_week));

    if (tableFilters.sort_col) {
      const col = tableFilters.sort_col as keyof FighterBout;
      const dir = tableFilters.sort_dir;
      result = [...result].sort((a, b) => {
        const va = a[col]; const vb = b[col];
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

  let tableBouts = $derived.by(() => {
    let result = [...filteredBouts];

    const hasBoutFilters = !!(
      tableFilters.my_technique ||
      tableFilters.my_result ||
      tableFilters.my_zone ||
      tableFilters.opponent_technique ||
      tableFilters.opponent_result ||
      tableFilters.opponent_zone ||
      tableFilters.score
    );

    if (!hasBoutFilters && selectedFighter) {
      const taggedVideoIds = new Set(rawBouts.map(b => b.video_id));
      for (const vid of filteredVideos) {
        if (!taggedVideoIds.has(vid.id)) {
          const am_a = vid.fighter_a?.id === selectedFighter.id;
          const opp = am_a ? vid.fighter_b : vid.fighter_a;

          result.push({
            id: -Math.floor(Math.random() * 1000000) - 1,
            video_id: vid.id,
            video_date: vid.date,
            opponent_id: opp?.id || '',
            opponent_name: opp?.display_name || '—',
            order_index: 0,
            time_start_ms: 0,
            time_end_ms: 0,
            my_score: 0,
            opponent_score: 0,
            my_technique_id: null,
            my_technique_name: null,
            my_hit_zone: null,
            my_result: null,
            opponent_technique_id: null,
            opponent_technique_name: null,
            opponent_hit_zone: null,
            opponent_result: null,
            is_unmarked: true
          } as any);
        }
      }
    }

    if (tableFilters.sort_col) {
      const col = tableFilters.sort_col as keyof FighterBout;
      const dir = tableFilters.sort_dir;
      result.sort((a, b) => {
        const va = a[col]; const vb = b[col];
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

  let opponents = $derived.by(() => {
    const map = new Map<string, string>();
    for (const b of rawBouts) {
      if (b.opponent_id) map.set(b.opponent_id, b.opponent_name);
    }
    if (selectedFighter) {
      for (const vid of rawVideos) {
        const am_a = vid.fighter_a?.id === selectedFighter.id;
        const opp = am_a ? vid.fighter_b : vid.fighter_a;
        if (opp && opp.id) {
          map.set(opp.id, opp.display_name);
        }
      }
    }
    return [...map.entries()].map(([id, name]) => ({ id, name }));
  });



  let firstBoutDate = $derived(
    rawBouts.length > 0
      ? rawBouts.reduce((min, b) => b.video_date < min ? b.video_date : min, rawBouts[0].video_date)
      : null
  );

  // Bouts for technique lists (we don't want them to filter themselves out)
  let boutsForMyTechniques = $derived.by(() => {
    let result = [...rawBouts];
    if (tableFilters.opponent_id) result = result.filter(b => b.opponent_id === tableFilters.opponent_id);
    if (zoneFilter) result = result.filter(b => (b.my_hit_zone ?? '').split(':')[0] === zoneFilter || (b.opponent_hit_zone ?? '').split(':')[0] === zoneFilter);
    if (tableFilters.opponent_technique) result = result.filter(b => b.opponent_technique_name === tableFilters.opponent_technique);
    if (tableFilters.video_id) result = result.filter(b => b.video_id === tableFilters.video_id);
    if (tableFilters.date_start) result = result.filter(b => b.video_date >= tableFilters.date_start);
    if (tableFilters.date_end) result = result.filter(b => b.video_date <= tableFilters.date_end);
    if (tableFilters.date_week) result = result.filter(b => matchesWeek(b.video_date, tableFilters.date_week));
    return result;
  });

  let boutsForOpponentTechniques = $derived.by(() => {
    let result = [...rawBouts];
    if (tableFilters.opponent_id) result = result.filter(b => b.opponent_id === tableFilters.opponent_id);
    if (zoneFilter) result = result.filter(b => (b.my_hit_zone ?? '').split(':')[0] === zoneFilter || (b.opponent_hit_zone ?? '').split(':')[0] === zoneFilter);
    if (tableFilters.my_technique) result = result.filter(b => b.my_technique_name === tableFilters.my_technique);
    if (tableFilters.video_id) result = result.filter(b => b.video_id === tableFilters.video_id);
    if (tableFilters.date_start) result = result.filter(b => b.video_date >= tableFilters.date_start);
    if (tableFilters.date_end) result = result.filter(b => b.video_date <= tableFilters.date_end);
    if (tableFilters.date_week) result = result.filter(b => matchesWeek(b.video_date, tableFilters.date_week));
    return result;
  });

  // Charts should respect technique filters
  let boutsForCharts = $derived.by(() => {
    let result = [...rawBouts];
    if (tableFilters.opponent_id)
      result = result.filter(b => b.opponent_id === tableFilters.opponent_id);
    if (zoneFilter) {
      result = result.filter(b => {
        const mz = (b.my_hit_zone ?? '').split(':')[0];
        const oz = (b.opponent_hit_zone ?? '').split(':')[0];
        return mz === zoneFilter || oz === zoneFilter;
      });
    }
    if (tableFilters.my_technique)
      result = result.filter(b => b.my_technique_name === tableFilters.my_technique);
    if (tableFilters.opponent_technique)
      result = result.filter(b => b.opponent_technique_name === tableFilters.opponent_technique);
    return result;
  });

  let boutsForTimelineCharts = $derived.by(() => {
    let result = [...boutsForCharts];
    if (tableFilters.date_week)
      result = result.filter(b => matchesWeek(b.video_date, tableFilters.date_week));
    return result;
  });

  let activeWeeks = $derived.by(() => {
    if (tableFilters.date_week) return parseWeeks(tableFilters.date_week);
    if (tableFilters.video_id) {
      const bout = rawBouts.find(b => b.video_id === tableFilters.video_id);
      if (bout) return [getISOWeek(bout.video_date)];
    }
    return [];
  });

  let filteredVideos = $derived.by(() => {
    let result = rawVideos;
    if (tableFilters.opponent_id) {
      result = result.filter(v => 
        v.fighter_a?.id === tableFilters.opponent_id || 
        v.fighter_b?.id === tableFilters.opponent_id
      );
    }
    if (tableFilters.date_start) {
      result = result.filter(v => v.date >= tableFilters.date_start);
    }
    if (tableFilters.date_end) {
      result = result.filter(v => v.date <= tableFilters.date_end);
    }
    if (tableFilters.date_week) {
      result = result.filter(v => matchesWeek(v.date, tableFilters.date_week));
    }
    if (tableFilters.video_id) {
      result = result.filter(v => v.id === tableFilters.video_id);
    }
    return result;
  });

  let videosForFrequencyChart = $derived.by(() => {
    let result = rawVideos;
    if (tableFilters.opponent_id) {
      result = result.filter(v =>
        v.fighter_a?.id === tableFilters.opponent_id ||
        v.fighter_b?.id === tableFilters.opponent_id
      );
    }
    if (tableFilters.date_start) {
      result = result.filter(v => v.date >= tableFilters.date_start);
    }
    if (tableFilters.date_end) {
      result = result.filter(v => v.date <= tableFilters.date_end);
    }
    return result;
  });

  let totalVideos = $derived(filteredVideos.length);

  async function selectFighter(fighter: Fighter) {
    if (selectedFighter?.id === fighter.id) return;
    selectedFighter = fighter;
    rawBouts = [];
    tableFilters = { ...defaultFilters };
    zoneFilter = '';
    loading = true;
    errorMsg = '';
    try {
      const [bouts, vids] = await Promise.all([
        getFighterBouts(fighter.id),
        getVideos({ fighter_id: fighter.id }),
      ]);
      rawBouts = bouts;
      rawVideos = vids;
    } catch (e) {
      errorMsg = e instanceof Error ? e.message : 'Ошибка загрузки данных';
    } finally {
      loading = false;
    }
  }

  $effect(() => {
    if ($fighters.length > 0 && !selectedFighter) {
      const me = $currentUser;
      if (me) {
        const myFighter = $fighters.find(f => f.id === me.id);
        if (myFighter) selectFighter(myFighter);
      }
    }
  });

  function handleFilter(filters: TableFilters) { tableFilters = filters; }
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
      <!-- ===== MAIN TWO COLUMNS ===== -->
      <div class="main-cols">

        <!-- LEFT COLUMN -->
        <div class="left-col">

          <!-- Top area: Hero card + KPI grid -->
          <div class="top-area">
            <!-- Hero Card -->
            <!-- svelte-ignore a11y_interactive_supports_focus -->
            <div class="fighter-hero glass-card"
              role="button" tabindex="0"
              onclick={() => showFighterDropdown = !showFighterDropdown}
              onkeydown={(e) => e.key === 'Enter' && (showFighterDropdown = !showFighterDropdown)}
            >
              <div class="avatar-wrap" style:background={resolveColor(selectedFighter.id, selectedFighter.color)}>
                <svg class="avatar-icon" width="40" height="40" viewBox="0 0 24 24" fill="none" aria-hidden="true">
                  <circle cx="12" cy="8" r="4" stroke="#fff" stroke-width="2.5" opacity="0.6"/>
                  <path d="M4 20c0-4 3.6-7 8-7s8 3 8 7" stroke="#fff" stroke-width="2.5" stroke-linecap="round" opacity="0.6"/>
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
                        <div class="opt-avatar" style:background={resolveColor(f.id, f.color)}>
                          <svg class="opt-icon" width="14" height="14" viewBox="0 0 24 24" fill="none" aria-hidden="true">
                            <circle cx="12" cy="8" r="4" stroke="#fff" stroke-width="1.5"/>
                            <path d="M4 20c0-4 3.6-7 8-7s8 3 8 7" stroke="#fff" stroke-width="1.5" stroke-linecap="round"/>
                          </svg>
                          {#if f.avatar_url}
                            <img class="opt-img" src={f.avatar_url} alt={f.display_name}
                              onerror={(e) => { (e.target as HTMLImageElement).style.display = 'none'; }} />
                          {/if}
                        </div>
                        <span class="opt-name">{f.display_name}</span>
                      </button>
                    {/each}
                  </div>
                </div>
              {/if}
            </div>

            <!-- KPI Grid -->
            <div class="kpis-area">
              <QuickStats bouts={filteredBouts} {totalVideos} />
            </div>
          </div>

          <!-- Chart slots -->
          <div class="chart-slot">
            <FrequencyChart 
              bouts={boutsForCharts} 
              rawVideos={videosForFrequencyChart} 
              selectedWeeks={activeWeeks} 
              isDrillDown={!!(tableFilters.my_technique || tableFilters.opponent_technique)}
              onfilter={(week, additive) => {
                const nextWeek = toggleWeekSelection(tableFilters.date_week, week, additive);
                chartScrollRatio = 0;
                if (nextWeek) chartXAxisMode = 'detail';
                handleFilter({...tableFilters, date_week: nextWeek, video_id: ''});
              }} 
            />
          </div>
          <div class="chart-slot">
            <ResultsChart
              bouts={boutsForTimelineCharts}
              selectedVideoId={tableFilters.video_id}
              selectedWeeks={activeWeeks}
              xAxisMode={chartXAxisMode}
              scrollRatio={chartScrollRatio}
              onmodechange={(mode) => { chartXAxisMode = mode; chartScrollRatio = 0; }}
              onscrollsync={(ratio) => chartScrollRatio = ratio}
              onfilter={(vid) => handleFilter({...tableFilters, video_id: vid, date_week: ''})}
            />
          </div>
          <div class="chart-slot">
            <ScoreChart
              bouts={boutsForTimelineCharts}
              selectedVideoId={tableFilters.video_id}
              selectedWeeks={activeWeeks}
              xAxisMode={chartXAxisMode}
              scrollRatio={chartScrollRatio}
              onmodechange={(mode) => { chartXAxisMode = mode; chartScrollRatio = 0; }}
              onscrollsync={(ratio) => chartScrollRatio = ratio}
              onfilter={(vid) => handleFilter({...tableFilters, video_id: vid, date_week: ''})}
            />
          </div>
        </div>

        <!-- RIGHT COLUMN -->
        <div class="right-col">
          <!-- Large top card: Radar -->
          <div class="right-large">
            <RadarChart bouts={filteredBouts} />
          </div>

          <!-- Two narrow cards: Techniques -->
          <div class="right-narrow-row">
            <div class="right-narrow">
              <TopTechniques 
                bouts={boutsForMyTechniques} 
                type="my" 
                selectedTechnique={tableFilters.my_technique}
                onfilter={(tech) => handleFilter({...tableFilters, my_technique: tech === tableFilters.my_technique ? '' : tech})} 
              />
            </div>
            <div class="right-narrow">
              <TopTechniques 
                bouts={boutsForOpponentTechniques} 
                type="opponent" 
                selectedTechnique={tableFilters.opponent_technique}
                onfilter={(tech) => handleFilter({...tableFilters, opponent_technique: tech === tableFilters.opponent_technique ? '' : tech})} 
              />
            </div>
          </div>

          <!-- Wide right card: Opponents -->
          <div class="right-wide">
            <RecentOpponents bouts={filteredBouts} currentFighterId={selectedFighter.id} selectedOpponentId={tableFilters.opponent_id || ''} onfilter={(oppId) => handleFilter({...tableFilters, opponent_id: oppId})} />
          </div>
        </div>
      </div>

      <!-- ===== BOTTOM: Silhouettes ===== -->
      <div class="bottom-cols">
        <div class="bottom-card">
          <BodySilhouette bouts={filteredBouts} type="dealt" selectedZone={zoneFilter} onzoneclick={(z) => { zoneFilter = z; }} />
        </div>
        <div class="bottom-card">
          <BodySilhouette bouts={filteredBouts} type="received" selectedZone={zoneFilter} onzoneclick={(z) => { zoneFilter = z; }} />
        </div>
      </div>

      <div class="table-slot">
        <HistoryTable
          bouts={tableBouts}
          filters={tableFilters}
          {opponents}
          fightDates={new Set(tableBouts.map(b => b.video_date.slice(0, 10)))}
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
    min-width: 0;
    overflow-x: hidden;
  }

  .dashboard {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 20px;
    padding: 20px;
    width: 100%;
    max-width: 1800px; /* Optional cap for extremely wide screens */
    margin: 0 auto;
    box-sizing: border-box;
    min-width: 0;
    overflow-x: hidden;
  }

  /* Empty / loading */
  .empty-state { display: flex; flex-direction: column; align-items: center; gap: 12px; padding: 80px 24px; color: var(--text-muted); }
  .empty-icon { font-size: 2.5rem; opacity: 0.4; }
  .empty-text { font-size: 0.9rem; }
  .loading, .error { text-align: center; padding: 64px; font-size: 0.9rem; }
  .loading { color: var(--text-muted); }
  .error { color: var(--accent-red); }

  /* ===== MAIN TWO COLUMNS ===== */
  .main-cols {
    display: grid;
    grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
    gap: 20px;
    align-items: start;
    min-width: 0;
  }

  .left-col, .right-col {
    display: flex;
    flex-direction: column;
    gap: 20px;
    min-width: 0;
    max-width: 100%;
  }

  /* Top area: Hero + KPIs */
  .top-area {
    display: grid;
    grid-template-columns: 252px minmax(0, 1fr);
    gap: 20px;
    height: 350px;
  }

  .kpis-area { min-width: 0; height: 350px; }

  /* Chart slots (left col) */
  .chart-slot {
    height: 450px;
    min-width: 0;
    max-width: 100%;
    overflow: hidden;
  }

  /* Right column slots */
  .right-large { height: 575px; }

  .right-narrow-row {
    display: grid;
    grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
    gap: 20px;
    height: 695px;
    min-width: 0;
  }

  .right-narrow { height: 695px; min-width: 0; }

  .right-large,
  .right-wide {
    min-width: 0;
    max-width: 100%;
    overflow: hidden;
  }

  .right-wide { height: 450px; }

  /* ===== BOTTOM SECTION ===== */
  .bottom-cols {
    display: grid;
    grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
    gap: 20px;
    min-width: 0;
  }

  .bottom-card {
    height: 848px;
    min-width: 0;
    overflow: hidden;
  }

  /* ===== HISTORY TABLE ===== */
  .table-slot {
    height: auto;
    min-width: 0;
    overflow: hidden;
  }

  /* Fighter Hero */
  .fighter-hero {
    padding: 30px 24px;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    text-align: center;
    gap: 16px;
    cursor: pointer;
    position: relative;
    z-index: 10;
    user-select: none;
    height: 100%;
  }

  .fighter-hero:hover { border-color: var(--text-secondary); }

  .fighter-dropdown {
    position: absolute;
    top: calc(100% + 8px);
    left: 0; right: 0;
    background: var(--surface-solid);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-md);
    box-shadow: var(--shadow-lg);
    z-index: 50;
    display: flex; flex-direction: column;
    cursor: default;
  }

  .dropdown-header {
    padding: 12px 16px; font-size: 0.75rem; font-weight: 700;
    text-transform: uppercase; letter-spacing: 0.08em;
    color: var(--text-secondary); border-bottom: 1px solid var(--border-color); text-align: left;
  }

  .dropdown-list { padding: 8px; display: flex; flex-direction: column; gap: 4px; }

  .fighter-opt {
    display: flex; align-items: center; gap: 12px; padding: 8px 12px;
    background: transparent; border: none; border-radius: var(--radius-sm);
    cursor: pointer; text-align: left; transition: var(--transition);
  }
  .fighter-opt:hover { background: var(--surface-hover); }
  .fighter-opt.selected { background: rgba(245, 158, 11, 0.12); }

  .opt-avatar {
    width: 24px; height: 24px; border-radius: 50%; flex-shrink: 0;
    border: 1px solid rgba(255,255,255,0.1); display: flex;
    align-items: center; justify-content: center; overflow: hidden; position: relative;
  }
  .opt-icon { opacity: 0.6; }
  .opt-img { position: absolute; inset: 0; width: 100%; height: 100%; object-fit: cover; }
  .opt-name { font-size: 0.9rem; color: var(--text-primary); font-weight: 500; }

  .avatar-wrap {
    width: 90px; height: 90px; border-radius: 50%;
    border: 3px solid rgba(255,255,255,0.2); flex-shrink: 0;
    position: relative; display: flex; align-items: center; justify-content: center; overflow: hidden;
  }
  .avatar-icon { position: absolute; pointer-events: none; }
  .avatar-img { position: absolute; inset: 0; width: 100%; height: 100%; object-fit: cover; }

  .greeting {
    font-size: 0.85rem; color: var(--text-secondary);
    text-transform: uppercase; letter-spacing: 0.05em; margin-bottom: 6px;
  }
  .fighter-name { font-size: 1.6rem; font-weight: 700; color: var(--text-primary); line-height: 1.2; }
  .fighter-since { font-size: 0.9rem; color: var(--text-secondary); margin-top: 4px; }
  
  /* RESPONSIVE */
  @media (max-width: 1400px) {
    .dashboard { padding: 16px; }
  }

  @media (max-width: 1100px) {
    .main-cols, .bottom-cols {
      grid-template-columns: 1fr;
    }
    .top-area {
      grid-template-columns: 1fr;
      height: auto;
    }
    .kpis-area { height: auto; }
    .right-narrow-row { height: auto; grid-template-columns: 1fr; }
    .right-narrow { height: 450px; }
  }



</style>
