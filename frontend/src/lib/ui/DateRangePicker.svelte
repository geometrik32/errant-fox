<script lang="ts">
  interface Props {
    dateStart: string;
    dateEnd: string;
    fightDates: Set<string>;
    onchange: (start: string, end: string) => void;
  }

  let { dateStart = $bindable(), dateEnd = $bindable(), fightDates, onchange }: Props = $props();

  let isOpen = $state(false);
  let viewDate = $state(new Date());
  let container = $state<HTMLElement | null>(null);
  let viewMode = $state<'weekly' | 'classic'>('weekly');

  const MONTH_NAMES = [
    'Январь', 'Февраль', 'Март', 'Апрель', 'Май', 'Июнь',
    'Июль', 'Август', 'Сентябрь', 'Октябрь', 'Ноябрь', 'Декабрь'
  ];
  const WEEKDAYS = ['Пн', 'Вт', 'Ср', 'Чт', 'Пт', 'Сб', 'Вс'];

  function formatDate(d: Date): string {
    const y = d.getFullYear();
    const m = String(d.getMonth() + 1).padStart(2, '0');
    const day = String(d.getDate()).padStart(2, '0');
    return `${y}-${m}-${day}`;
  }

  // Generate calendar days
  let calendarDays = $derived.by(() => {
    const year = viewDate.getFullYear();
    const month = viewDate.getMonth();

    const firstDay = new Date(year, month, 1);
    let startDay = firstDay.getDay(); // 0 is Sunday
    startDay = startDay === 0 ? 6 : startDay - 1; // Convert to Monday-indexed (0 = Monday)

    const totalDays = new Date(year, month + 1, 0).getDate();
    const prevMonthDays = new Date(year, month, 0).getDate();

    const list: Array<{ date: Date; dateStr: string; isCurrentMonth: boolean; hasFight: boolean }> = [];

    // Prev month padding
    for (let i = startDay - 1; i >= 0; i--) {
      const d = new Date(year, month - 1, prevMonthDays - i);
      const str = formatDate(d);
      list.push({ date: d, dateStr: str, isCurrentMonth: false, hasFight: fightDates.has(str) });
    }

    // Current month
    for (let i = 1; i <= totalDays; i++) {
      const d = new Date(year, month, i);
      const str = formatDate(d);
      list.push({ date: d, dateStr: str, isCurrentMonth: true, hasFight: fightDates.has(str) });
    }

    // Next month padding
    const remaining = list.length % 7;
    if (remaining > 0) {
      const nextPadding = 7 - remaining;
      for (let i = 1; i <= nextPadding; i++) {
        const d = new Date(year, month + 1, i);
        const str = formatDate(d);
        list.push({ date: d, dateStr: str, isCurrentMonth: false, hasFight: fightDates.has(str) });
      }
    }

    return list;
  });

  // Generate weeks list for weekly mode
  let monthlyWeeks = $derived.by(() => {
    const year = viewDate.getFullYear();
    const month = viewDate.getMonth();
    const first = new Date(Date.UTC(year, month, 1));
    const last = new Date(Date.UTC(year, month + 1, 0));
    
    // Find the Monday of the week containing the first day of the month
    let d = new Date(first);
    const dow = d.getUTCDay();
    if (dow === 0) {
      d.setUTCDate(d.getUTCDate() - 6);
    } else if (dow > 1) {
      d.setUTCDate(d.getUTCDate() - (dow - 1));
    }
    
    const list: Array<{ start: string; end: string; label: string; hasFight: boolean }> = [];
    
    // Loop until we cover all weeks overlapping with this month
    while (d <= last || (d.getUTCMonth() === month && d <= last)) {
      const startStr = formatDate(d);
      const sunday = new Date(d);
      sunday.setUTCDate(sunday.getUTCDate() + 6);
      const endStr = formatDate(sunday);
      
      // Check if any day in this week has a fight
      let hasFight = false;
      const checkDay = new Date(d);
      for (let i = 0; i < 7; i++) {
        if (fightDates.has(formatDate(checkDay))) {
          hasFight = true;
          break;
        }
        checkDay.setUTCDate(checkDay.getUTCDate() + 1);
      }
      
      const formatLabel = (s: Date, e: Date) => {
        const monthsRu = ['Янв', 'Фев', 'Мар', 'Апр', 'Май', 'Июн', 'Июл', 'Авг', 'Сен', 'Окт', 'Ноя', 'Дек'];
        return `${s.getUTCDate()} ${monthsRu[s.getUTCMonth()]} — ${e.getUTCDate()} ${monthsRu[e.getUTCMonth()]}`;
      };
      
      list.push({
        start: startStr,
        end: endStr,
        label: formatLabel(new Date(d), new Date(sunday)),
        hasFight
      });
      
      d.setUTCDate(d.getUTCDate() + 7);
    }
    return list;
  });

  function prevMonth() {
    viewDate = new Date(viewDate.getFullYear(), viewDate.getMonth() - 1, 1);
  }

  // Set view date to selected start date when opening calendar
  $effect(() => {
    if (isOpen && dateStart) {
      const parts = dateStart.split('-');
      if (parts.length === 3) {
        viewDate = new Date(parseInt(parts[0]), parseInt(parts[1]) - 1, 1);
      }
    }
  });

  function nextMonth() {
    viewDate = new Date(viewDate.getFullYear(), viewDate.getMonth() + 1, 1);
  }

  function selectDate(dateStr: string) {
    if (!dateStart || (dateStart && dateEnd)) {
      dateStart = dateStr;
      dateEnd = '';
      onchange(dateStart, dateEnd);
    } else {
      if (dateStr < dateStart) {
        dateStart = dateStr;
        onchange(dateStart, dateEnd);
      } else {
        dateEnd = dateStr;
        onchange(dateStart, dateEnd);
        isOpen = false;
      }
    }
  }

  function clearFilter() {
    dateStart = '';
    dateEnd = '';
    onchange('', '');
    isOpen = false;
  }

  function displayRange(start: string, end: string) {
    if (!start && !end) return 'ДД.ММ.ГГГГ';
    const fmt = (s: string) => {
      const parts = s.split('-');
      if (parts.length !== 3) return s;
      return `${parts[2]}.${parts[1]}.${parts[0].slice(2)}`;
    };
    if (start && !end) return `${fmt(start)} - …`;
    if (start && end) return `${fmt(start)} - ${fmt(end)}`;
    return 'ДД.ММ.ГГГГ';
  }

  function handleWindowClick(e: MouseEvent) {
    if (isOpen && container && !container.contains(e.target as Node)) {
      isOpen = false;
    }
  }
</script>

<svelte:window onclick={handleWindowClick} />

<div class="datepicker-container" bind:this={container}>
  <!-- Trigger Button -->
  <button class="datepicker-trigger" class:active={dateStart} onclick={() => isOpen = !isOpen}>
    <span class="trigger-text">
      {displayRange(dateStart, dateEnd)}
    </span>
    <svg class="calendar-icon" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
      <rect x="3" y="4" width="18" height="18" rx="2" ry="2"></rect>
      <line x1="16" y1="2" x2="16" y2="6"></line>
      <line x1="8" y1="2" x2="8" y2="6"></line>
      <line x1="3" y1="10" x2="21" y2="10"></line>
    </svg>
  </button>

  <!-- Calendar Dropdown -->
  {#if isOpen}
    <div class="calendar-dropdown">
      <!-- Mode Toggle -->
      <div class="mode-selector">
        <button class="mode-btn" class:active={viewMode === 'weekly'} onclick={() => viewMode = 'weekly'}>
          По неделям
        </button>
        <button class="mode-btn" class:active={viewMode === 'classic'} onclick={() => viewMode = 'classic'}>
          По дням
        </button>
      </div>

      <!-- Header navigation -->
      <div class="calendar-header">
        <button class="nav-arrow" onclick={prevMonth} aria-label="Предыдущий месяц" title="Предыдущий месяц">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <polyline points="15 18 9 12 15 6"></polyline>
          </svg>
        </button>
        <span class="month-title">
          {MONTH_NAMES[viewDate.getMonth()]} {viewDate.getFullYear()}
        </span>
        <button class="nav-arrow" onclick={nextMonth} aria-label="Следующий месяц" title="Следующий месяц">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <polyline points="9 18 15 12 9 6"></polyline>
          </svg>
        </button>
      </div>

      <!-- Weekly mode template -->
      {#if viewMode === 'weekly'}
        <div class="weeks-list">
          {#each monthlyWeeks as week}
            <button
              type="button"
              class="week-row"
              class:selected={dateStart === week.start && dateEnd === week.end}
              class:has-fight={week.hasFight}
              onclick={() => {
                dateStart = week.start;
                dateEnd = week.end;
                onchange(dateStart, dateEnd);
                isOpen = false;
              }}
            >
              <span class="week-label">{week.label}</span>
              {#if week.hasFight}
                <span class="week-fight-dot"></span>
              {/if}
            </button>
          {/each}
        </div>
      {:else}
        <!-- Classic mode template -->
        <div class="weekdays-grid">
          {#each WEEKDAYS as wd}
            <div class="weekday-cell">{wd}</div>
          {/each}
        </div>

        <div class="days-grid">
          {#each calendarDays as day}
            <!-- svelte-ignore a11y_click_events_have_key_events -->
            <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
            <div 
              class="day-cell" 
              role="button"
              tabindex="0"
              class:muted={!day.isCurrentMonth}
              class:selected-start={day.dateStr === dateStart}
              class:selected-end={day.dateStr === dateEnd}
              class:in-range={dateStart && dateEnd && day.dateStr > dateStart && day.dateStr < dateEnd}
              class:has-fight={day.hasFight}
              onclick={() => selectDate(day.dateStr)}
            >
              <span class="day-number">{day.date.getDate()}</span>
              {#if day.hasFight}
                <span class="fight-dot"></span>
              {/if}
            </div>
          {/each}
        </div>
      {/if}

      <div class="calendar-actions">
        <button class="action-btn clear-btn" onclick={clearFilter}>Сбросить</button>
      </div>
    </div>
  {/if}
</div>

<style>
  .datepicker-container {
    position: relative;
    width: 100%;
    min-width: 130px;
    margin-bottom: 4px;
  }

  .datepicker-trigger {
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
    background: var(--surface-solid);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm);
    color: var(--text-primary);
    font-size: 0.75rem;
    padding: 5px 8px;
    outline: none;
    cursor: pointer;
    transition: var(--transition);
    text-align: left;
    box-sizing: border-box;
  }

  .datepicker-trigger:hover, .datepicker-trigger:focus {
    border-color: var(--accent-yellow);
  }

  .datepicker-trigger.active {
    border-color: var(--accent-yellow);
    color: var(--accent-yellow);
  }

  .trigger-text {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    margin-right: 4px;
    font-variant-numeric: tabular-nums;
  }

  .calendar-icon {
    opacity: 0.6;
    flex-shrink: 0;
  }

  .datepicker-trigger.active .calendar-icon {
    opacity: 1;
  }

  .calendar-dropdown {
    position: absolute;
    top: calc(100% + 6px);
    left: 0;
    width: 260px;
    background: #0f172a;
    border: 1px solid var(--border-color);
    border-radius: var(--radius-md);
    box-shadow: 0 10px 25px -5px rgba(0, 0, 0, 0.5), 0 8px 10px -6px rgba(0, 0, 0, 0.5);
    padding: 12px;
    z-index: 100;
    box-sizing: border-box;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .mode-selector {
    display: flex;
    background: var(--surface-hover);
    padding: 3px;
    border-radius: var(--radius-sm);
    gap: 2px;
    margin-bottom: 4px;
  }

  .mode-btn {
    flex: 1;
    background: transparent;
    border: none;
    color: var(--text-secondary);
    font-size: 0.72rem;
    font-weight: 600;
    padding: 6px;
    border-radius: calc(var(--radius-sm) - 2px);
    cursor: pointer;
    transition: var(--transition);
  }

  .mode-btn:hover {
    color: var(--text-primary);
  }

  .mode-btn.active {
    background: var(--surface-solid);
    color: var(--accent-yellow);
    box-shadow: 0 1px 3px rgba(0,0,0,0.2);
  }

  .calendar-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 4px;
  }

  .month-title {
    font-size: 0.8rem;
    font-weight: 600;
    color: var(--text-primary);
  }

  .nav-arrow {
    background: none;
    border: none;
    color: var(--text-secondary);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    border-radius: 50%;
    transition: var(--transition);
  }

  .nav-arrow:hover {
    background: var(--surface-hover);
    color: var(--text-primary);
  }

  /* Weekly Mode styles */
  .weeks-list {
    display: flex;
    flex-direction: column;
    gap: 4px;
    margin-bottom: 4px;
  }

  .week-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    background: var(--surface-hover);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm);
    padding: 8px 12px;
    color: var(--text-primary);
    font-family: inherit;
    font-size: 0.8rem;
    font-weight: 500;
    cursor: pointer;
    transition: var(--transition);
    text-align: left;
    width: 100%;
    box-sizing: border-box;
  }

  .week-row:hover {
    border-color: var(--accent-yellow);
    background: rgba(245, 158, 11, 0.05);
  }

  .week-row.selected {
    background: var(--accent-yellow) !important;
    color: #000 !important;
    border-color: var(--accent-yellow);
    font-weight: 600;
  }

  .week-row.has-fight:not(.selected) {
    border-color: rgba(245, 158, 11, 0.4);
    background: rgba(245, 158, 11, 0.1) !important;
  }
  .week-row.has-fight .week-label {
    color: var(--accent-yellow);
    font-weight: 600;
  }

  .week-row.selected .week-fight-dot {
    background: #000;
  }

  .week-fight-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--accent-yellow);
    box-shadow: 0 0 4px var(--accent-yellow);
    flex-shrink: 0;
  }

  /* Classic mode styles */
  .weekdays-grid {
    display: grid;
    grid-template-columns: repeat(7, 1fr);
    text-align: center;
    margin-bottom: 2px;
  }

  .weekday-cell {
    font-size: 0.7rem;
    font-weight: 500;
    color: var(--text-secondary);
    padding: 2px 0;
  }

  .days-grid {
    display: grid;
    grid-template-columns: repeat(7, 1fr);
    row-gap: 2px;
  }

  .day-cell {
    position: relative;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 30px;
    cursor: pointer;
    border-radius: var(--radius-sm);
    font-size: 0.75rem;
    color: var(--text-primary);
    user-select: none;
    box-sizing: border-box;
  }

  .day-cell:hover {
    background: var(--surface-hover);
  }

  .day-cell.muted {
    color: var(--text-muted);
    opacity: 0.4;
  }

  .day-cell.selected-start,
  .day-cell.selected-end {
    background: var(--accent-yellow) !important;
    color: #000 !important;
    font-weight: 600;
  }

  .day-cell.in-range {
    background: rgba(245, 158, 11, 0.12);
    border-radius: 0;
  }

  .day-cell.selected-start {
    border-top-left-radius: var(--radius-sm);
    border-bottom-left-radius: var(--radius-sm);
  }
  .day-cell.selected-end {
    border-top-right-radius: var(--radius-sm);
    border-bottom-right-radius: var(--radius-sm);
  }

  .day-cell.has-fight {
    color: var(--accent-yellow);
    font-weight: 600;
  }
  .day-cell.has-fight:not(.selected-start):not(.selected-end):not(.in-range) {
    background: rgba(245, 158, 11, 0.22) !important;
    border: 1px solid rgba(245, 158, 11, 0.45);
    border-radius: 6px;
    color: #fff;
  }

  .day-cell.has-fight .fight-dot {
    display: none;
  }

  .fight-dot {
    position: absolute;
    bottom: 3px;
    width: 4px;
    height: 4px;
    border-radius: 50%;
    background: var(--accent-yellow);
    box-shadow: 0 0 3px var(--accent-yellow);
  }

  .day-cell.selected-start .fight-dot,
  .day-cell.selected-end .fight-dot {
    background: #000;
  }

  .calendar-actions {
    display: flex;
    justify-content: flex-end;
    margin-top: 4px;
    border-top: 1px solid var(--border-color);
    padding-top: 8px;
  }

  .action-btn {
    background: none;
    border: none;
    font-size: 0.75rem;
    font-weight: 500;
    cursor: pointer;
    padding: 4px 8px;
    border-radius: var(--radius-sm);
    transition: var(--transition);
  }

  .clear-btn {
    color: var(--text-secondary);
  }

  .clear-btn:hover {
    background: var(--surface-hover);
    color: var(--text-primary);
  }
</style>
