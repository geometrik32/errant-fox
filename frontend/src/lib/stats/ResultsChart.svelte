<script lang="ts">
  import { onMount } from 'svelte';
  import type { FighterBout } from '../api/types';

  interface Props {
    bouts: FighterBout[];
    videoLabels?: Map<string, string>;
  }

  let { bouts, videoLabels = new Map() }: Props = $props();

  let canvas = $state<HTMLCanvasElement | undefined>(undefined);
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  let chart: any = null;
  // aggregate per-video win/loss
  function computeVideoResults(bouts: FighterBout[]) {
    const videoMap = new Map<string, { video_id: string; date: string; my: number; opp: number }>();
    for (const b of bouts) {
      const v = videoMap.get(b.video_id);
      if (v) { v.my += b.my_score; v.opp += b.opponent_score; }
      else videoMap.set(b.video_id, { video_id: b.video_id, date: b.video_date, my: b.my_score, opp: b.opponent_score });
    }
    return [...videoMap.values()]
      .sort((a, b) => a.date.localeCompare(b.date))
      .map(v => ({ video_id: v.video_id, date: v.date, result: v.my > v.opp ? 1 : v.my < v.opp ? -1 : 0 }));
  }

  let winRate = $derived.by(() => {
    const results = computeVideoResults(bouts);
    if (!results.length) return null;
    const wins = results.filter(r => r.result === 1).length;
    return { wins, total: results.length, pct: Math.round(wins / results.length * 100) };
  });

  $effect(() => {
    if (!canvas) return;
    const results = computeVideoResults(bouts);
    const labels = results.map(r => videoLabels.get(r.video_id) ?? r.date);
    const data = results.map(r => r.result);

    import('chart.js').then(({ Chart, registerables }) => {
      Chart.register(...registerables);
      if (chart) { chart.destroy(); chart = null; }
      chart = new Chart(canvas!, {
        type: 'line',
        data: {
          labels,
          datasets: [{
            label: 'Результат',
            data,
            borderColor: '#6b7280',
            backgroundColor: 'transparent',
            pointBackgroundColor: data.map(v => v === 1 ? '#10b981' : v === -1 ? '#ef4444' : '#fbbf24'),
            pointBorderColor: '#ffffff',
            pointBorderWidth: 2,
            pointRadius: 6,
            pointHoverRadius: 8,
            tension: 0.4,
            fill: false,
          }],
        },
        options: {
          responsive: true,
          maintainAspectRatio: false,
          plugins: {
            legend: { display: false },
            tooltip: {
              backgroundColor: '#0f2035',
              borderColor: '#1f3a57',
              borderWidth: 1,
              titleColor: '#a0b4c8',
              bodyColor: '#e8edf2',
              callbacks: {
                label: (ctx) => ctx.raw === 1 ? 'Победа' : ctx.raw === -1 ? 'Поражение' : 'Ничья',
              },
            },
          },
          scales: {
            x: {
              ticks: { color: '#6b7280', font: { family: 'Inter', size: 11 }, maxRotation: 0 },
              grid: { display: false },
              border: { display: false }
            },
            y: {
              beginAtZero: true,
              ticks: { display: false },
              grid: { 
                display: true, 
                color: (ctx) => ctx.tick.value === 0 ? 'rgba(255,255,255,0.15)' : 'transparent',
                drawTicks: false
              },
              border: { display: false },
              title: { display: false }
            },
          },
        },
      });
    });

    return () => { chart?.destroy(); chart = null; };
  });

  onMount(() => () => { chart?.destroy(); chart = null; });
</script>

<div class="chart-card">
  <div class="chart-header">
    <div class="left">
      <div class="chart-title">Динамика результатов</div>
      {#if winRate}
        <div class="win-rate">
          <span class="wins">{winRate.pct}% побед</span>
          <span class="sep">·</span>
          <span class="totals">{winRate.wins} из {winRate.total}</span>
        </div>
      {/if}
    </div>
  </div>
  <div class="chart-body">
    <canvas bind:this={canvas}></canvas>
  </div>
</div>

<style>
  .chart-card {
    background: var(--surface);
    backdrop-filter: var(--glass-blur);
    -webkit-backdrop-filter: var(--glass-blur);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-sm);
    padding: 24px;
    height: 100%;
    display: flex;
    flex-direction: column;
  }

  .chart-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 12px;
    margin-bottom: 12px;
  }

  .chart-title {
    font-size: 1rem;
    font-weight: 700;
    color: var(--text-primary);
    margin: 0 0 4px 0;
  }

  .win-rate {
    font-size: 0.85rem;
    color: var(--text-secondary);
  }

  .wins {
    color: #4caf82;
    font-weight: 600;
  }

  .sep {
    margin: 0 5px;
    color: var(--border-color);
  }

  .opp-select {
    background: #0d1b2a;
    border: 1px solid #1f3a57;
    border-radius: 5px;
    color: #a0b4c8;
    font-size: 0.78rem;
    padding: 4px 8px;
    outline: none;
    cursor: pointer;
    flex-shrink: 0;
  }

  .opp-select:focus {
    border-color: #2a4f73;
  }

  .chart-body {
    flex: 1;
    min-height: 240px;
    position: relative;
  }

  canvas {
    display: block;
    width: 100% !important;
    height: 100% !important;
  }
</style>
