<script lang="ts">
  import type { FighterBout } from '../api/types';

  interface Props {
    bouts: FighterBout[];
  }

  let { bouts }: Props = $props();

  function topBy(bouts: FighterBout[], getName: (b: FighterBout) => string | null): string {
    const counts = new Map<string, number>();
    for (const b of bouts) {
      const name = getName(b);
      if (name) counts.set(name, (counts.get(name) ?? 0) + 1);
    }
    let max = 0, top = '—';
    for (const [name, count] of counts) {
      if (count > max) { max = count; top = name; }
    }
    return top;
  }

  let mostUsed = $derived(topBy(bouts, (b) => b.my_technique_name));
  let mostMissed = $derived(topBy(bouts.filter(b => b.my_result === 'miss'), (b) => b.my_technique_name));
  let mostReceived = $derived(topBy(bouts.filter(b => b.opponent_result === 'hit'), (b) => b.opponent_technique_name));

  let totalBouts = $derived(bouts.length);
  let wins = $derived(bouts.filter(b => b.my_score > b.opponent_score).length);
  let winRate = $derived(totalBouts > 0 ? Math.round((wins / totalBouts) * 100) : 0);
</script>

<div class="quick-stats-container">
  <div class="numbers-row">
    <div class="num-card glass-card">
      <div class="num-val">{totalBouts}</div>
      <div class="num-lbl">Всего сходов</div>
    </div>
    <div class="num-card glass-card">
      <div class="num-val">{wins}</div>
      <div class="num-lbl">Побед</div>
    </div>
    <div class="num-card glass-card">
      <div class="num-val">{winRate}%</div>
      <div class="num-lbl">Винрейт</div>
    </div>
  </div>

  <div class="text-stats-row">
    <div class="stat-block glass-card">
      <div class="stat-label">Использую чаще всего</div>
      <div class="stat-value">{mostUsed}</div>
    </div>
    <div class="stat-block glass-card">
      <div class="stat-label">Промахиваюсь с</div>
      <div class="stat-value">{mostMissed}</div>
    </div>
    <div class="stat-block glass-card">
      <div class="stat-label">Урон чаще всего от</div>
      <div class="stat-value">{mostReceived}</div>
    </div>
  </div>
</div>

<style>
  .quick-stats-container {
    display: flex;
    flex-direction: column;
    gap: 14px;
    flex: 1;
  }

  .numbers-row {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 14px;
  }

  .text-stats-row {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 14px;
  }

  @media (max-width: 768px) {
    .numbers-row, .text-stats-row {
      grid-template-columns: 1fr;
    }
  }

  .num-card {
    padding: 24px 20px;
    background: var(--surface);
    backdrop-filter: var(--glass-blur);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-sm);
    display: flex;
    flex-direction: column;
    justify-content: center;
    align-items: center;
    text-align: center;
  }

  .num-val {
    font-size: 3rem;
    font-weight: 300;
    color: var(--text-primary);
    line-height: 1;
    margin-bottom: 6px;
  }

  .num-lbl {
    font-size: 0.85rem;
    color: var(--text-secondary);
    font-weight: 500;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .stat-block {
    padding: 18px 20px;
    background: var(--surface-solid);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-md);
    box-shadow: var(--shadow-sm);
    display: flex;
    flex-direction: column;
    justify-content: center;
  }

  .stat-label {
    font-size: 0.75rem;
    font-weight: 600;
    letter-spacing: 0.05em;
    text-transform: uppercase;
    color: var(--text-secondary);
    margin-bottom: 6px;
  }

  .stat-value {
    font-size: 1.1rem;
    font-weight: 700;
    color: var(--accent-yellow);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
</style>
