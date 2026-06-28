<script lang="ts">
  import { techniques } from '../../stores';
  import type { FighterBout } from '../api/types';

  interface Props {
    bouts: FighterBout[];
    type?: 'my' | 'opponent';
    selectedTechnique?: string;
    onfilter?: (tech: string) => void;
  }

  let { bouts, type = 'my', selectedTechnique = '', onfilter }: Props = $props();

  let sortBy = $state<'successRate' | 'count' | 'success'>('successRate');

  function getTopTechniques(bouts: FighterBout[], currentSort: 'successRate' | 'count' | 'success') {
    const counts = new Map<string, { count: number, success: number }>();
    
    // Initialize with all techniques from store
    for (const t of $techniques) {
      counts.set(t.name, { count: 0, success: 0 });
    }

    for (const b of bouts) {
      const name = type === 'my' ? b.my_technique_name : b.opponent_technique_name;
      const res = type === 'my' ? b.my_result : b.opponent_result;
      
      if (name) {
        const stats = counts.get(name) || { count: 0, success: 0 };
        stats.count++;
        if (res === 'hit') stats.success++;
        counts.set(name, stats);
      }
    }

    let items = [...counts.entries()]
      .map(([name, stats]) => ({
        name,
        count: stats.count,
        success: stats.success,
        successRate: stats.count > 0 ? Math.round((stats.success / stats.count) * 100) : 0
      }));

    if (currentSort === 'successRate') {
      items.sort((a, b) => b.successRate - a.successRate || b.count - a.count);
    } else if (currentSort === 'count') {
      items.sort((a, b) => b.count - a.count || b.successRate - a.successRate);
    } else if (currentSort === 'success') {
      items.sort((a, b) => b.success - a.success || b.count - a.count);
    }

    return items;
  }

  let topTechniques = $derived(getTopTechniques(bouts, sortBy));
</script>

<div class="top-techniques glass-card">
  <div class="card-header">
    <h3 class="card-title">{type === 'my' ? 'Топ моих техник' : 'Топ техник оппонента'}</h3>
    <select class="sort-select" bind:value={sortBy}>
      <option value="successRate">% успеха</option>
      <option value="count">Частота</option>
      <option value="success">Попадания</option>
    </select>
  </div>
  <div class="tech-list">
    {#each topTechniques as t}
      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div class="tech-item" class:selected={t.name === selectedTechnique} onclick={() => onfilter?.(t.name)}>
        <div class="tech-info">
          <span class="tech-name">{t.name}</span>
          <span class="tech-stats">{t.count} раз ({t.successRate}% успех)</span>
        </div>
        <div class="tech-bar-bg">
          <div class="tech-bar-fill" style="width: {t.successRate}%;"></div>
        </div>
      </div>
    {:else}
      <div class="empty">Нет данных о техниках</div>
    {/each}
  </div>
</div>

<style>
  .top-techniques {
    background: var(--surface);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-2xl);
    padding: 24px;
    box-shadow: var(--shadow-sm);
    height: 100%;
    display: flex;
    flex-direction: column;
  }

  .card-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 24px;
  }

  .card-title {
    font-size: 1rem;
    font-weight: 700;
    color: var(--text-primary);
    margin: 0;
  }

  .tech-list {
    display: flex;
    flex-direction: column;
    gap: 12px;
    flex: 1;
    overflow-y: auto;
    min-height: 0;
    padding-right: 4px;
  }

  .tech-list::-webkit-scrollbar {
    width: 4px;
  }
  .tech-list::-webkit-scrollbar-thumb {
    background: var(--surface-solid);
    border-radius: 4px;
  }

  .tech-item {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 8px 12px;
    border-radius: var(--radius-sm);
    cursor: pointer;
    transition: background 0.2s;
  }

  .tech-item:hover {
    background: var(--surface-hover);
  }
 
  .tech-item.selected {
    background: rgba(219, 132, 31, 0.12);
    border-color: rgba(219, 132, 31, 0.4);
    box-shadow: 0 4px 12px rgba(219, 132, 31, 0.1);
  }

  .tech-info {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .tech-name {
    font-size: 0.9rem;
    font-weight: 600;
    color: var(--text-primary);
  }

  .tech-stats {
    font-size: 0.8rem;
    color: var(--text-secondary);
  }

  .tech-bar-bg {
    height: 8px;
    background: var(--surface-solid);
    border-radius: 4px;
    overflow: hidden;
  }

  .tech-bar-fill {
    height: 100%;
    background: var(--accent-yellow);
    border-radius: 4px;
    transition: width 0.5s ease-out;
  }

  .empty {
    font-size: 0.85rem;
    color: var(--text-secondary);
    text-align: center;
    padding: 20px 0;
  }

  .sort-select {
    background: rgba(0, 0, 0, 0.2);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm);
    padding: 4px 24px 4px 8px;
    font-size: 0.8rem;
    color: var(--text-secondary);
    outline: none;
    cursor: pointer;
    transition: var(--transition);
    appearance: none;
    -webkit-appearance: none;
    background-image: url("data:image/svg+xml;utf8,<svg fill='gray' height='24' viewBox='0 0 24 24' width='24' xmlns='http://www.w3.org/2000/svg'><path d='M7 10l5 5 5-5z'/><path d='M0 0h24v24H0z' fill='none'/></svg>");
    background-repeat: no-repeat;
    background-position: right 4px center;
    background-size: 16px;
  }

  .sort-select:hover {
    border-color: var(--border-strong);
    color: var(--text-primary);
  }

  .sort-select:focus {
    border-color: var(--accent-yellow);
    color: var(--text-primary);
  }

  .sort-select option {
    background: var(--surface-solid);
    color: var(--text-primary);
  }
</style>
