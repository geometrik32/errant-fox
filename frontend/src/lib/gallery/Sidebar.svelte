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
  }

  let { videos, onfilter }: Props = $props();

  let selectedIds = $state<Set<string>>(new Set());
  let dateFrom = $state('');
  let dateTo = $state('');

  function countForFighter(id: string): number {
    return videos.filter(
      (v) => v.fighter_a?.id === id || v.fighter_b?.id === id
    ).length;
  }

  function toggleFighter(id: string) {
    const next = new Set(selectedIds);
    if (next.has(id)) {
      next.delete(id);
    } else {
      next.add(id);
    }
    selectedIds = next;
    emit();
  }

  function emit() {
    onfilter?.({
      fighter_ids: [...selectedIds],
      date_from: dateFrom,
      date_to: dateTo,
    });
  }
</script>

<aside class="sidebar">
  <section class="section">
    <h3 class="section-title">Бойцы</h3>

    {#each $fighters as fighter (fighter.id)}
      {@const count = countForFighter(fighter.id)}
      <label class="row">
        <input
          type="checkbox"
          checked={selectedIds.has(fighter.id)}
          onchange={() => toggleFighter(fighter.id)}
        />
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

  <section class="section">
    <h3 class="section-title">Дата</h3>
    <div class="date-range">
      <label class="date-row">
        <span class="date-label">С</span>
        <input
          type="date"
          class="date-input"
          bind:value={dateFrom}
          onchange={emit}
        />
      </label>
      <label class="date-row">
        <span class="date-label">По</span>
        <input
          type="date"
          class="date-input"
          bind:value={dateTo}
          onchange={emit}
        />
      </label>
    </div>
  </section>
</aside>

<style>
  .sidebar {
    width: 216px;
    flex-shrink: 0;
    display: flex;
    flex-direction: column;
    gap: 28px;
  }

  .section {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .section-title {
    font-size: 0.68rem;
    font-weight: 600;
    letter-spacing: 0.09em;
    text-transform: uppercase;
    color: #4a6280;
    margin-bottom: 8px;
  }

  .row {
    display: flex;
    align-items: center;
    gap: 8px;
    cursor: pointer;
    padding: 5px 6px;
    border-radius: 6px;
    transition: background 0.12s;
  }

  .row:hover {
    background: #1a3050;
  }

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
    font-size: 0.83rem;
    color: #a0b4c8;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .count {
    font-size: 0.72rem;
    color: #4a6280;
    flex-shrink: 0;
  }

  .empty {
    font-size: 0.8rem;
    color: #4a6280;
    padding: 4px 6px;
  }

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
    color: #6b8aab;
    width: 20px;
    flex-shrink: 0;
  }

  .date-input {
    flex: 1;
    background: #0f2035;
    border: 1px solid #1f3a57;
    border-radius: 6px;
    color: #a0b4c8;
    font-size: 0.78rem;
    padding: 5px 7px;
    outline: none;
    transition: border-color 0.12s;
    min-width: 0;
  }

  .date-input:focus {
    border-color: #2a4f73;
  }

  .date-input::-webkit-calendar-picker-indicator {
    filter: invert(0.5);
    cursor: pointer;
  }
</style>
