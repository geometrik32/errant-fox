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
  let avgBoutsPerFight = $derived(totalBattles > 0 ? (totalBouts / totalBattles).toFixed(1) : '—');
</script>

<div class="kpi-grid">
  <!-- Row 1: Winrates and Points with Badges -->
  <div class="kpi-card col-center">
    <div class="kpi-content">
      <div class="kpi-label">Винрейт по сходам</div>
      <div class="kpi-value">{boutWinRate}%</div>
    </div>
  </div>
  
  <div class="kpi-card col-center">
    <div class="kpi-content">
      <div class="kpi-label">Винрейт по боям</div>
      <div class="kpi-value">{battleWinRate}%</div>
    </div>
  </div>

  <div class="kpi-card col-center">
    <div class="kpi-badge badge-red">
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round">
        <path d="M18 6L6 18M6 6l12 12"/>
      </svg>
    </div>
    <div class="kpi-content">
      <div class="kpi-label">Пропущено очков</div>
      <div class="kpi-value">{pointsConceded}</div>
    </div>
  </div>

  <div class="kpi-card col-center">
    <div class="kpi-badge badge-green">
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round">
        <path d="M20 6L9 17L4 12"/>
      </svg>
    </div>
    <div class="kpi-content">
      <div class="kpi-label">Набрано очков</div>
      <div class="kpi-value">{pointsScored}</div>
    </div>
  </div>

  <!-- Row 2: Totals and Wide Battle card -->
  <div class="kpi-card col-center">
    <div class="kpi-content">
      <div class="kpi-label">Всего сходов</div>
      <div class="kpi-value">{totalBouts}</div>
    </div>
  </div>

  <div class="kpi-card col-center">
    <div class="kpi-content">
      <div class="kpi-label">Сходов за бой</div>
      <div class="kpi-value">{avgBoutsPerFight}</div>
    </div>
  </div>

  <div class="kpi-card kpi-wide row-center">
    <div class="kpi-icon-box" style="background: rgba(142, 68, 173, 0.15);">
      <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="var(--accent-purple)" stroke-width="2">
        <path d="M4 6h16M4 12h16M4 18h16"/>
      </svg>
    </div>
    <div class="kpi-content text-left">
      <div class="kpi-label">Всего боёв</div>
      <div class="kpi-value-inline">
        <span class="count-main">{totalBattles}</span>
        {#if totalVideos > 0}
          <span class="count-sep">/</span><span class="count-total">{totalVideos}</span>
        {/if}
      </div>
    </div>
  </div>
</div>

<style>
  .kpi-grid {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    grid-template-rows: 165px 165px;
    gap: 20px;
    height: 350px;
  }

  .kpi-wide { grid-column: span 2; }

  .kpi-card {
    background: var(--surface);
    backdrop-filter: blur(var(--blur-amount));
    -webkit-backdrop-filter: blur(var(--blur-amount));
    border: 1px solid var(--border-color);
    border-radius: var(--radius-2xl);
    box-shadow: var(--shadow-md);
    padding: 24px;
    position: relative;
    transition: var(--transition);
    min-width: 0;
  }

  .col-center {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    text-align: center;
  }

  .row-center {
    display: flex;
    flex-direction: row;
    align-items: center;
    justify-content: center;
    gap: 24px;
  }

  .kpi-card:hover {
    box-shadow: var(--shadow-lg);
    border-color: var(--border-strong);
  }

  .kpi-badge {
    position: absolute;
    top: 12px;
    width: 100px;
    height: 32px;
    border-radius: 99px;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .badge-red {
    background: rgba(224, 82, 82, 0.1);
    color: #ef4444;
  }

  .badge-green {
    background: rgba(16, 185, 129, 0.1);
    color: #10b981;
  }

  .kpi-icon-box {
    width: 52px;
    height: 52px;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }

  .kpi-content {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }

  .text-left { text-align: left; }

  .kpi-label {
    font-size: 12px;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.04em;
    font-weight: 600;
    line-height: 1.1;
    max-width: 140px;
  }

  .kpi-value {
    font-size: 28px;
    font-weight: 700;
    color: var(--text-primary);
    line-height: 1;
  }

  .kpi-value-inline {
    display: flex;
    align-items: baseline;
    gap: 4px;
  }

  .count-main {
    font-size: 28px;
    font-weight: 700;
    color: var(--text-primary);
    line-height: 1;
  }

  .count-sep {
    color: var(--text-secondary);
    font-weight: 300;
    font-size: 28px;
    margin: 0 4px;
  }

  .count-total {
    font-weight: 300;
    font-size: 28px;
    color: var(--text-secondary);
  }
</style>
