<script lang="ts">
  import type { FighterBout } from '../api/types';

  interface Props {
    bouts: FighterBout[];
  }

  let { bouts }: Props = $props();

  let totalBouts = $derived(bouts.length);
  let wins = $derived(bouts.filter(b => b.my_score > b.opponent_score).length);
  let winRate = $derived(totalBouts > 0 ? Math.round((wins / totalBouts) * 100) : 0);
  
  let pointsScored = $derived(bouts.reduce((sum, b) => sum + b.my_score, 0));
  let pointsConceded = $derived(bouts.reduce((sum, b) => sum + b.opponent_score, 0));

  // Text Stats
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

<div class="kpi-grid">
  <div class="kpi-card glass-card">
    <div class="kpi-icon" style="background: rgba(111, 160, 224, 0.1); color: #6fa0e0;">
      <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5"/>
      </svg>
    </div>
    <div class="kpi-info">
      <div class="kpi-label">Всего сходов</div>
      <div class="kpi-value">{totalBouts}</div>
    </div>
  </div>

  <div class="kpi-card glass-card">
    <div class="kpi-icon" style="background: rgba(219, 132, 31, 0.1); color: var(--accent-yellow);">
      <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M12 2L2 22h20L12 2z"/>
      </svg>
    </div>
    <div class="kpi-info">
      <div class="kpi-label">Винрейт</div>
      <div class="kpi-value">{winRate}%</div>
    </div>
  </div>

  <div class="kpi-card glass-card">
    <div class="kpi-icon" style="background: rgba(39, 174, 96, 0.1); color: #27ae60;">
      <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M5 12l5 5L20 7"/>
      </svg>
    </div>
    <div class="kpi-info">
      <div class="kpi-label">Набрано очков</div>
      <div class="kpi-value">{pointsScored}</div>
    </div>
  </div>

  <div class="kpi-card glass-card">
    <div class="kpi-icon" style="background: rgba(239, 68, 68, 0.1); color: #ef4444;">
      <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M18 6L6 18M6 6l12 12"/>
      </svg>
    </div>
    <div class="kpi-info">
      <div class="kpi-label">Пропущено очков</div>
      <div class="kpi-value">{pointsConceded}</div>
    </div>
  </div>

  <div class="kpi-card glass-card text-kpi">
    <div class="kpi-info">
      <div class="kpi-label">Попадаю чаще</div>
      <div class="kpi-value text-val">{mostUsed}</div>
    </div>
  </div>

  <div class="kpi-card glass-card text-kpi">
    <div class="kpi-info">
      <div class="kpi-label">Промахиваюсь</div>
      <div class="kpi-value text-val">{mostMissed}</div>
    </div>
  </div>

  <div class="kpi-card glass-card text-kpi">
    <div class="kpi-info">
      <div class="kpi-label">Пропускаю чаще</div>
      <div class="kpi-value text-val">{mostReceived}</div>
    </div>
  </div>
</div>

<style>
  .kpi-grid {
    display: flex;
    flex-wrap: wrap;
    gap: 20px;
    flex: 1;
  }

  .kpi-card {
    flex: 1 1 180px;
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 24px;
    background: var(--surface);
    backdrop-filter: var(--glass-blur);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-lg);
    box-shadow: none;
  }

  .kpi-icon {
    width: 48px;
    height: 48px;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }

  .kpi-info {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .kpi-label {
    font-size: 0.8rem;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    font-weight: 600;
  }

  .kpi-value {
    font-size: 1.8rem;
    font-weight: 700;
    color: var(--text-primary);
    line-height: 1;
  }

  .text-kpi {
    padding: 16px 24px;
    justify-content: center;
  }

  .text-val {
    font-size: 1.2rem;
    color: var(--accent-yellow);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 100%;
  }

  @media (max-width: 1024px) {
    .kpi-card {
      flex: 1 1 45%;
    }
  }

  @media (max-width: 640px) {
    .kpi-card {
      flex: 1 1 100%;
    }
  }
</style>
