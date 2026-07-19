<script lang="ts">
  import { untrack } from 'svelte';
  import { fighters } from '../../stores';
  import type { Video } from '../api/types';
  import { resolveColor } from '../api/types';
  import DateRangePicker from '../ui/DateRangePicker.svelte';

  interface FilterEvent {
    fighter_ids: string[];
    date_from: string;
    date_to: string;
  }

  interface Props {
    videos: Video[];
    onfilter?: (filter: FilterEvent) => void;
    initialFilter?: FilterEvent;
    onlineUsers?: any[];
  }

  let { videos, onfilter, initialFilter, onlineUsers = [] }: Props = $props();

  let selectedIds = $state<Set<string>>(new Set(untrack(() => initialFilter?.fighter_ids ?? [])));
  let dateFrom = $state(untrack(() => initialFilter?.date_from ?? ''));
  let dateTo = $state(untrack(() => initialFilter?.date_to ?? ''));

  let onlineFighterIds = $derived(new Set(onlineUsers.map(u => u.id)));

  // YYYY-MM-DD Dates with spars matching selected fighters
  let videoDatesSet = $derived.by(() => {
    const s = new Set<string>();
    for (const v of videos) {
      if (selectedIds.size > 0) {
        const match = [...selectedIds].every(
          (id) => v.fighter_a?.id === id || v.fighter_b?.id === id
        );
        if (!match) continue;
      }
      if (v.date) s.add(v.date.slice(0, 10));
    }
    return s;
  });

  function countForFighter(id: string): number {
    if (selectedIds.size === 1) {
      const [selectedId] = selectedIds;
      if (selectedId === id) {
        return videos.filter(v => v.fighter_a?.id === id || v.fighter_b?.id === id).length;
      }
      return videos.filter(v =>
        (v.fighter_a?.id === selectedId || v.fighter_b?.id === selectedId) &&
        (v.fighter_a?.id === id || v.fighter_b?.id === id)
      ).length;
    }
    return videos.filter(v => v.fighter_a?.id === id || v.fighter_b?.id === id).length;
  }

  function isDisabled(id: string): boolean {
    return selectedIds.size >= 2 && !selectedIds.has(id);
  }

  function toggleFighter(id: string) {
    if (isDisabled(id)) return;
    const next = new Set(selectedIds);
    if (next.has(id)) next.delete(id);
    else next.add(id);
    selectedIds = next;
    emit();
  }

  function handleDateChange(start: string, end: string) {
    dateFrom = start;
    dateTo = end;
    emit();
  }

  function emit() {
    onfilter?.({ fighter_ids: [...selectedIds], date_from: dateFrom, date_to: dateTo });
  }
</script>

<aside class="sidebar">
  <!-- Fighters -->
  <section class="section">
    <h3 class="section-title">Бойцы</h3>
    <div class="fighters-list">
      {#each $fighters as fighter (fighter.id)}
        {@const count = countForFighter(fighter.id)}
        {@const disabled = isDisabled(fighter.id)}
        {@const isOnline = onlineFighterIds.has(fighter.id)}
        <!-- svelte-ignore a11y_label_has_associated_control -->
        <label class="row" class:row--disabled={disabled}>
          <input type="checkbox" checked={selectedIds.has(fighter.id)} disabled={disabled} tabindex="-1" onchange={() => toggleFighter(fighter.id)} />
          <div class="fighter-info">
            <div class="avatar-container">
              <div class="avatar" style="--fighter-color: {resolveColor(fighter.id, fighter.color)}">
                <svg class="avatar-icon" width="14" height="14" viewBox="0 0 24 24" fill="none" aria-hidden="true">
                  <circle cx="12" cy="8" r="4" stroke="#fff" stroke-width="1.5" opacity="0.6"/>
                  <path d="M4 20c0-4 3.6-7 8-7s8 3 8 7" stroke="#fff" stroke-width="1.5" stroke-linecap="round" opacity="0.6"/>
                </svg>
                {#if fighter.avatar_url}
                  <img src={fighter.avatar_url} alt="" onerror={(e) => { (e.target as HTMLImageElement).style.display = 'none'; }} />
                {/if}
              </div>
              <span class="status-dot" class:online={isOnline}></span>
            </div>
            <span class="fighter-name">{fighter.display_name}</span>
          </div>
          <span class="count">{count}</span>
        </label>
      {/each}
    </div>
    {#if $fighters.length === 0}
      <p class="empty">Нет бойцов</p>
    {/if}
  </section>

  <!-- Date filter -->
  <section class="section">
    <h3 class="section-title">Дата</h3>
    <DateRangePicker
      dateStart={dateFrom}
      dateEnd={dateTo}
      fightDates={videoDatesSet}
      onchange={handleDateChange}
    />
  </section>
</aside>

<style>
  .sidebar {
    display: flex;
    flex-direction: column;
    gap: 24px;
    padding: 24px 20px 20px;
    box-sizing: border-box;
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

  /* Fighters list container */
  .fighters-list {
    display: flex;
    flex-direction: column;
    gap: 4px;
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
    appearance: none;
    -webkit-appearance: none;
    width: 14px;
    height: 14px;
    border: 1.5px solid rgba(255, 255, 255, 0.5);
    border-radius: 50%;
    background: transparent;
    cursor: pointer;
    position: relative;
    transition: var(--transition);
    flex-shrink: 0;
  }
  
  .row input[type="checkbox"]:checked {
    background: #DB841F;
    border-color: #DB841F;
  }

  .fighter-info {
    display: flex;
    align-items: center;
    gap: 10px;
    flex: 1;
    min-width: 0;
  }

  .avatar-container {
    position: relative;
    width: 32px;
    height: 32px;
    flex-shrink: 0;
  }

  .avatar {
    width: 32px;
    height: 32px;
    border-radius: 50%;
    border: 2px solid var(--fighter-color, #4a6280);
    background: var(--surface-hover);
    overflow: hidden;
    display: flex;
    align-items: center;
    justify-content: center;
    position: relative;
  }

  .avatar-icon {
    position: absolute;
  }

  .avatar img {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .status-dot {
    position: absolute;
    bottom: -1px;
    right: -1px;
    width: 10px;
    height: 10px;
    border-radius: 50%;
    background: #6b7280;
    border: 2px solid #1e293b;
    transition: var(--transition);
  }

  .status-dot.online {
    background: var(--accent-green);
    box-shadow: 0 0 4px var(--accent-green);
  }

  .fighter-name {
    font-size: 0.9rem;
    font-weight: 500;
    color: var(--text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .count {
    font-size: 0.8rem;
    color: var(--text-secondary);
    flex-shrink: 0;
    min-width: 2ch;
    text-align: right;
    font-variant-numeric: tabular-nums;
  }

  .row--disabled {
    opacity: 0.35;
    cursor: not-allowed;
    pointer-events: none;
  }

  .empty {
    font-size: 0.8rem;
    color: var(--text-secondary);
    padding: 4px 6px;
  }
</style>
