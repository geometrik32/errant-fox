<script lang="ts">
  import type { FighterBout } from '../api/types';

  interface Props {
    bouts: FighterBout[];
    totalVideos?: number;
  }

  let { bouts, totalVideos = 0 }: Props = $props();

  let totalBouts = $derived(bouts.length);
  let boutWins = $derived(bouts.filter(b => b.my_score > b.opponent_score).length);
  let boutWinRate = $derived(totalBouts > 0 ? Math.round((boutWins / totalBouts) * 100) : 0);

  let videoResults = $derived.by(() => {
    const videoMap = new Map<string, { my: number; opp: number }>();
    for (const b of bouts) {
      const v = videoMap.get(b.video_id);
      if (v) { v.my += b.my_score; v.opp += b.opponent_score; }
      else videoMap.set(b.video_id, { my: b.my_score, opp: b.opponent_score });
    }
    return [...videoMap.values()].map(v => v.my > v.opp ? 1 : v.my < v.opp ? -1 : 0);
  });

  let totalBattles = $derived(videoResults.length);
  let battleWins = $derived(videoResults.filter(r => r === 1).length);
  let battleWinRate = $derived(totalBattles > 0 ? Math.round((battleWins / totalBattles) * 100) : 0);
  
  let pointsScored = $derived(bouts.reduce((sum, b) => sum + b.my_score, 0));
  let pointsConceded = $derived(bouts.reduce((sum, b) => sum + b.opponent_score, 0));
  let untaggedVideos = $derived(totalVideos > 0 ? totalVideos - totalBattles : 0);
  let avgBoutsPerFight = $derived(totalBattles > 0 ? (totalBouts / totalBattles).toFixed(1) : '—');

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

  let defenseStats = $derived.by(() => {
    const relevant = bouts.filter(b => b.opponent_result === 'hit' || b.opponent_result === 'miss' || b.opponent_result === 'blocked');
    if (relevant.length === 0) return 0;
    const defended = relevant.filter(b => b.opponent_result === 'miss' || b.opponent_result === 'blocked').length;
    return Math.round((defended / relevant.length) * 100);
  });
</script>

<div class="kpi-grid">
  <div class="kpi-card glass-card">
    <div class="kpi-icon" style="background: rgba(142, 68, 173, 0.1); color: #8e44ad;">
      <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M4 6h16M4 12h16M4 18h16"/>
      </svg>
    </div>
    <div class="kpi-info">
      <div class="kpi-label">Всего боёв</div>
      <div class="kpi-value">
        <span class="fights-tagged">{totalBattles}</span>
        {#if totalVideos > 0}
          <span class="fights-sep"> / </span><span class="fights-total">{totalVideos}</span>
        {/if}
      </div>
    </div>
  </div>

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
    <div class="kpi-icon" style="background: rgba(80, 160, 200, 0.1); color: #50a0c8;">
      <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M8 6h13M8 12h13M8 18h13M3 6h.01M3 12h.01M3 18h.01"/>
      </svg>
    </div>
    <div class="kpi-info">
      <div class="kpi-label">Сходов за бой</div>
      <div class="kpi-value">{avgBoutsPerFight}</div>
    </div>
  </div>

  <div class="kpi-card glass-card">
    <div class="kpi-icon" style="background: rgba(219, 132, 31, 0.1); color: var(--accent-yellow);">
      <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="12" cy="12" r="10"/><path d="M16 12l-4-4-4 4M12 8v8"/>
      </svg>
    </div>
    <div class="kpi-info">
      <div class="kpi-label">Винрейт по боям</div>
      <div class="kpi-value">{battleWinRate}%</div>
    </div>
  </div>

  <div class="kpi-card glass-card">
    <div class="kpi-icon" style="background: rgba(241, 196, 15, 0.1); color: #f1c40f;">
      <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M12 2L2 22h20L12 2z"/>
      </svg>
    </div>
    <div class="kpi-info">
      <div class="kpi-label">Винрейт по сходам</div>
      <div class="kpi-value">{boutWinRate}%</div>
    </div>
  </div>

  <div class="kpi-card glass-card">
    <div class="kpi-icon" style="background: rgba(46, 204, 113, 0.1); color: #2ecc71;">
      <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/>
      </svg>
    </div>
    <div class="kpi-info">
      <div class="kpi-label">Защита</div>
      <div class="kpi-value">{defenseStats}%</div>
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

  .kpi-sub {
    font-size: 0.72rem;
    margin-top: 3px;
  }

  .sub-green { color: #4caf82; }
  .sub-dim   { color: var(--text-secondary); }

  .fights-tagged { font-weight: 700; }
  .fights-sep    { color: var(--text-secondary); font-weight: 300; font-size: 1.4rem; margin: 0 1px; }
  .fights-total  { font-weight: 300; font-size: 1.4rem; color: var(--text-secondary); }

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
