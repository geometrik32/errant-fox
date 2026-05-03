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
</script>

<div class="quick-stats">
  <div class="stat-block glass-card">
    <div class="stat-label">Использую чаще всего</div>
    <div class="stat-value">{mostUsed}</div>
  </div>
  <div class="stat-block glass-card">
    <div class="stat-label">Промахиваюсь чаще всего с</div>
    <div class="stat-value">{mostMissed}</div>
  </div>
  <div class="stat-block glass-card">
    <div class="stat-label">Получаю урон чаще всего от</div>
    <div class="stat-value">{mostReceived}</div>
  </div>
</div>

<style>
  .quick-stats {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 14px;
  }

  @media (max-width: 768px) {
    .quick-stats {
      grid-template-columns: 1fr;
    }
  }

  .stat-block {
    padding: 16px 20px;
    background: var(--surface);
    backdrop-filter: var(--glass-blur);
    -webkit-backdrop-filter: var(--glass-blur);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-md);
    box-shadow: var(--shadow-sm);
  }

  .stat-label {
    font-size: 0.8rem;
    font-weight: 600;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--text-secondary);
    margin-bottom: 8px;
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
