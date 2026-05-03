<script lang="ts">
  import type { FighterBout } from '../api/types';

  interface Props {
    bouts: FighterBout[];
    onfilter?: (tech: string) => void;
  }

  let { bouts, onfilter }: Props = $props();

  function getTopTechniques(bouts: FighterBout[], limit: number) {
    const counts = new Map<string, { count: number, success: number }>();
    for (const b of bouts) {
      if (b.my_technique_name) {
        const stats = counts.get(b.my_technique_name) || { count: 0, success: 0 };
        stats.count++;
        if (b.my_result === 'hit') stats.success++;
        counts.set(b.my_technique_name, stats);
      }
    }
    return [...counts.entries()]
      .sort((a, b) => b[1].count - a[1].count)
      .slice(0, limit)
      .map(([name, stats]) => ({
        name,
        count: stats.count,
        successRate: Math.round((stats.success / stats.count) * 100)
      }));
  }

  let topTechniques = $derived(getTopTechniques(bouts, 5));
</script>

<div class="top-techniques glass-card">
  <div class="card-header">
    <h3 class="card-title">Топ техник</h3>
    <span class="card-subtitle">по частоте использования</span>
  </div>
  <div class="tech-list">
    {#each topTechniques as t}
      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div class="tech-item" onclick={() => onfilter?.(t.name)}>
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
    backdrop-filter: var(--glass-blur);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-lg);
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

  .card-subtitle {
    font-size: 0.8rem;
    color: var(--text-secondary);
  }

  .tech-list {
    display: flex;
    flex-direction: column;
    gap: 12px;
    flex: 1;
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
</style>
