<script lang="ts">
  import { fighters } from '../../stores';
  import type { Video } from '../api/types';
  import { resolveColor } from '../api/types';

  interface FilterEvent {
    fighter_ids: string[];
    date_from: string;
    date_to: string;
  }

  interface Props {
    videos: Video[];
    onfilter?: (filter: FilterEvent) => void;
    initialFilter?: FilterEvent;
  }

  let { videos, onfilter, initialFilter }: Props = $props();

  let selectedIds = $state<Set<string>>(new Set(initialFilter?.fighter_ids ?? []));
  let dateFrom = $state(initialFilter?.date_from ?? '');
  let dateTo = $state(initialFilter?.date_to ?? '');
  let dateMode = $state<'year' | 'classic'>('year');
  let calYear = $state(new Date().getFullYear());
  let selWeekStart = $state('');

  const MONTHS_RU = ['Янв', 'Фев', 'Мар', 'Апр', 'Май', 'Июн',
                     'Июл', 'Авг', 'Сен', 'Окт', 'Ноя', 'Дек'];

  // Set of ISO Monday dates that have at least one video
  let videoWeekSet = $derived.by(() => {
    const s = new Set<string>();
    for (const v of videos) {
      if (v.date) s.add(isoWeekStart(v.date.slice(0, 10)));
    }
    return s;
  });

  // Years that have at least one video (for visual cue on year nav)
  let videoYears = $derived.by(() => {
    const s = new Set<number>();
    for (const v of videos) {
      const y = parseInt(v.date?.slice(0, 4) ?? '0');
      if (y > 0) s.add(y);
    }
    return s;
  });

  function isoWeekStart(dateStr: string): string {
    const d = new Date(dateStr + 'T12:00:00Z');
    const dow = d.getUTCDay();
    d.setUTCDate(d.getUTCDate() + (dow === 0 ? -6 : 1 - dow));
    return d.toISOString().slice(0, 10);
  }

  function addDays(dateStr: string, n: number): string {
    const d = new Date(dateStr + 'T12:00:00Z');
    d.setUTCDate(d.getUTCDate() + n);
    return d.toISOString().slice(0, 10);
  }

  // Returns ISO Monday dates of weeks whose Monday falls in the given month
  function monthWeeks(year: number, month: number): string[] {
    const result: string[] = [];
    const first = new Date(Date.UTC(year, month, 1));
    const last  = new Date(Date.UTC(year, month + 1, 0));
    let d = new Date(first);
    const dow = d.getUTCDay();
    if (dow !== 1) d.setUTCDate(d.getUTCDate() + (dow === 0 ? 1 : 8 - dow));
    while (d <= last) {
      result.push(d.toISOString().slice(0, 10));
      d.setUTCDate(d.getUTCDate() + 7);
    }
    return result;
  }

  function selectWeek(weekStart: string) {
    if (selWeekStart === weekStart) {
      selWeekStart = '';
      dateFrom = '';
      dateTo = '';
    } else {
      selWeekStart = weekStart;
      dateFrom = weekStart;
      dateTo = addDays(weekStart, 6);
    }
    emit();
  }

  function clearDate() {
    selWeekStart = '';
    dateFrom = '';
    dateTo = '';
    emit();
  }

  function countForFighter(id: string): number {
    return videos.filter(v => v.fighter_a?.id === id || v.fighter_b?.id === id).length;
  }

  function toggleFighter(id: string) {
    const next = new Set(selectedIds);
    if (next.has(id)) next.delete(id);
    else next.add(id);
    selectedIds = next;
    emit();
  }

  function emit() {
    onfilter?.({ fighter_ids: [...selectedIds], date_from: dateFrom, date_to: dateTo });
  }

  function formatWeekLabel(start: string): string {
    const end = addDays(start, 6);
    const s = new Date(start + 'T12:00:00Z');
    const e = new Date(end   + 'T12:00:00Z');
    const fmt = (d: Date) => `${d.getUTCDate()} ${MONTHS_RU[d.getUTCMonth()]}`;
    return `${fmt(s)} — ${fmt(e)}`;
  }
</script>

<aside class="sidebar">
    <!-- Fighters -->
    <section class="section">
      <h3 class="section-title">Бойцы</h3>
      {#each $fighters as fighter (fighter.id)}
        {@const count = countForFighter(fighter.id)}
        <label class="row">
          <input type="checkbox" checked={selectedIds.has(fighter.id)} onchange={() => toggleFighter(fighter.id)} />
          <div class="fighter-info">
            <div class="color-dot" style:background={resolveColor(fighter.id, fighter.color)}></div>
            <span class="fighter-name">{fighter.display_name}</span>
          </div>
          <span class="count">{count}</span>
        </label>
      {/each}
      {#if $fighters.length === 0}
        <p class="empty">Нет бойцов</p>
      {/if}
    </section>

    <!-- Date filter -->
    <section class="section">
      <div class="section-title-row">
        <h3 class="section-title">Дата</h3>
        <button class="mode-btn" onclick={() => {
          dateMode = dateMode === 'year' ? 'classic' : 'year';
        }}>
          {dateMode === 'year' ? 'Классический' : 'Год-вид'}
        </button>
      </div>

      {#if selWeekStart && dateMode === 'year'}
        <div class="active-week">
          <span>{formatWeekLabel(selWeekStart)}</span>
          <button class="clear-btn" onclick={clearDate} title="Сбросить">×</button>
        </div>
      {/if}

      {#if dateMode === 'year'}
        <!-- Year navigation -->
        <div class="cal-nav">
          <button class="cal-arrow" onclick={() => { calYear -= 1; }}>‹</button>
          <span class="cal-year" class:has-videos={videoYears.has(calYear)}>{calYear}</span>
          <button class="cal-arrow" onclick={() => { calYear += 1; }}>›</button>
        </div>

        <!-- Month grid -->
        <div class="cal-months">
          {#each Array.from({length: 12}, (_, i) => i) as month}
            {@const weeks = monthWeeks(calYear, month)}
            {#if weeks.length > 0}
              <div class="cal-month">
                <div class="cal-month-lbl">{MONTHS_RU[month]}</div>
                <div class="cal-weeks">
                  {#each weeks as week}
                    <button
                      class="cal-week"
                      class:has-video={videoWeekSet.has(week)}
                      class:selected={selWeekStart === week}
                      onclick={() => selectWeek(week)}
                      title={formatWeekLabel(week)}
                    ></button>
                  {/each}
                </div>
              </div>
            {/if}
          {/each}
        </div>
      {:else}
        <div class="date-range">
          <label class="date-row">
            <span class="date-label">С</span>
            <input type="date" class="date-input" bind:value={dateFrom} onchange={emit} />
          </label>
          <label class="date-row">
            <span class="date-label">По</span>
            <input type="date" class="date-input" bind:value={dateTo} onchange={emit} />
          </label>
        </div>
      {/if}
    </section>
  </aside>

<style>
  .sidebar-slim {
    display: none;
  }

  .slim-toggle {
    display: none;
    line-height: 1;
    transition: var(--transition);
  }

  .slim-toggle:hover {
    color: var(--text-primary);
    border-color: var(--accent-yellow);
  }

  .sidebar {
    width: 280px;
    flex-shrink: 0;
    display: flex;
    flex-direction: column;
    gap: 24px;
    padding: 20px;
  }

  .sidebar-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding-bottom: 8px;
    border-bottom: 1px solid var(--border-color);
  }

  .head-title {
    font-size: 0.75rem;
    font-weight: 700;
    letter-spacing: 0.1em;
    text-transform: uppercase;
    color: var(--text-secondary);
  }

  .head-toggle {
    background: none;
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm);
    color: var(--text-secondary);
    width: 28px;
    height: 28px;
    cursor: pointer;
    font-size: 1.2rem;
    line-height: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: var(--transition);
  }

  .head-toggle:hover {
    color: var(--text-primary);
    background: var(--surface-hover);
  }

  .section {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .section-title {
    font-size: 0.75rem;
    font-weight: 600;
    letter-spacing: 0.09em;
    text-transform: uppercase;
    color: var(--text-secondary);
    margin: 0 0 12px;
  }

  .section-title-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 8px;
  }

  .section-title-row .section-title {
    margin-bottom: 0;
  }

  .mode-btn {
    background: none;
    border: none;
    color: var(--text-secondary);
    font-size: 0.7rem;
    cursor: pointer;
    padding: 0;
    transition: var(--transition);
  }

  .mode-btn:hover {
    color: var(--text-primary);
  }

  /* Fighter rows */
  .row {
    display: flex;
    align-items: center;
    gap: 12px;
    cursor: pointer;
    padding: 8px 10px;
    border-radius: var(--radius-sm);
    transition: var(--transition);
  }

  .row:hover { background: var(--surface-hover); }

  .row input[type="checkbox"] {
    width: 14px;
    height: 14px;
    accent-color: #DB841F;
    flex-shrink: 0;
    cursor: pointer;
  }

  .fighter-info {
    display: flex;
    align-items: center;
    gap: 7px;
    flex: 1;
    min-width: 0;
  }

  .color-dot {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .fighter-name {
    font-size: 0.9rem;
    color: var(--text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .count {
    font-size: 0.8rem;
    color: var(--text-secondary);
    flex-shrink: 0;
  }

  .empty {
    font-size: 0.8rem;
    color: var(--text-secondary);
    padding: 4px 6px;
  }

  /* Active week badge */
  .active-week {
    display: flex;
    align-items: center;
    justify-content: space-between;
    background: rgba(219, 132, 31, 0.1);
    border: 1px solid rgba(219, 132, 31, 0.3);
    border-radius: 5px;
    padding: 4px 8px;
    font-size: 0.74rem;
    color: #DB841F;
    margin-bottom: 6px;
  }

  .clear-btn {
    background: none;
    border: none;
    color: #DB841F;
    cursor: pointer;
    font-size: 1rem;
    line-height: 1;
    padding: 0 0 0 6px;
    opacity: 0.7;
    transition: opacity 0.12s;
  }

  .clear-btn:hover { opacity: 1; }

  /* Year calendar */
  .cal-nav {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 10px;
  }

  .cal-arrow {
    background: none;
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm);
    color: var(--text-secondary);
    width: 22px;
    height: 22px;
    cursor: pointer;
    font-size: 0.9rem;
    line-height: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: var(--transition);
  }

  .cal-arrow:hover {
    color: var(--text-primary);
    border-color: var(--accent-yellow);
  }

  .cal-year {
    font-size: 0.85rem;
    font-weight: 700;
    color: var(--text-secondary);
    letter-spacing: 0.04em;
  }

  .cal-year.has-videos {
    color: var(--text-primary);
  }

  .cal-months {
    display: flex;
    flex-direction: column;
    gap: 5px;
  }

  .cal-month {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .cal-month-lbl {
    font-size: 0.66rem;
    color: var(--text-secondary);
    width: 26px;
    flex-shrink: 0;
    text-align: right;
  }

  .cal-weeks {
    display: flex;
    gap: 3px;
    flex-wrap: wrap;
  }

  .cal-week {
    width: 12px;
    height: 12px;
    border-radius: 2px;
    border: none;
    background: var(--surface-solid);
    cursor: pointer;
    padding: 0;
    transition: var(--transition);
    outline: 1px solid transparent;
  }

  .cal-week:hover {
    background: var(--surface-hover);
    outline-color: var(--accent-yellow);
  }

  .cal-week.has-video {
    background: rgba(219, 132, 31, 0.45);
  }

  .cal-week.has-video:hover {
    background: rgba(219, 132, 31, 0.65);
  }

  .cal-week.selected {
    background: #DB841F !important;
    outline-color: #DB841F;
  }

  /* Classic date pickers */
  .date-range {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .date-row {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .date-label {
    font-size: 0.78rem;
    color: var(--text-secondary);
    width: 20px;
    flex-shrink: 0;
  }

  .date-input {
    flex: 1;
    background: var(--surface-solid);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm);
    color: var(--text-primary);
    font-size: 0.78rem;
    padding: 5px 7px;
    outline: none;
    transition: var(--transition);
    min-width: 0;
  }

  .date-input:focus { border-color: var(--accent-yellow); }

  .date-input::-webkit-calendar-picker-indicator {
    filter: invert(0.5);
    cursor: pointer;
  }
</style>
