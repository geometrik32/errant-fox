<script lang="ts">
  import { onMount } from 'svelte';
  import type { FighterBout } from '../api/types';

  interface Props {
    bouts: FighterBout[];
  }

  let { bouts }: Props = $props();

  let canvas = $state<HTMLCanvasElement | undefined>(undefined);
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  let chart: any = null;
  let opponentFilter = $state('');

  // unique opponents in current bouts
  let opponents = $derived.by(() => {
    const map = new Map<string, string>();
    for (const b of bouts) map.set(b.opponent_id, b.opponent_name);
    return [...map.entries()].map(([id, name]) => ({ id, name }));
  });

  // aggregate per-video win/loss, filtered by opponent
  function computeVideoResults(bouts: FighterBout[], opponentId: string) {
    const filtered = opponentId ? bouts.filter(b => b.opponent_id === opponentId) : bouts;
    const videoMap = new Map<string, { date: string; my: number; opp: number }>();
    for (const b of filtered) {
      const v = videoMap.get(b.video_id);
      if (v) { v.my += b.my_score; v.opp += b.opponent_score; }
      else videoMap.set(b.video_id, { date: b.video_date, my: b.my_score, opp: b.opponent_score });
    }
    return [...videoMap.values()]
      .sort((a, b) => a.date.localeCompare(b.date))
      .map(v => ({ date: v.date, result: v.my > v.opp ? 1 : v.my < v.opp ? -1 : 0 }));
  }

  let winRate = $derived.by(() => {
    const results = computeVideoResults(bouts, opponentFilter);
    if (!results.length) return null;
    const wins = results.filter(r => r.result === 1).length;
    return { wins, total: results.length, pct: Math.round(wins / results.length * 100) };
  });

  $effect(() => {
    if (!canvas) return;
    const results = computeVideoResults(bouts, opponentFilter);
    const labels = results.map(r => r.date);
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
            borderColor: '#DB841F',
            backgroundColor: 'rgba(219, 132, 31, 0.08)',
            pointBackgroundColor: data.map(v => v === 1 ? '#4caf82' : v === -1 ? '#e05252' : '#4a6280'),
            pointRadius: 5,
            pointHoverRadius: 7,
            tension: 0.2,
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
              ticks: { color: '#4a6280', font: { size: 11 }, maxRotation: 45 },
              grid: { color: '#142338' },
            },
            y: {
              min: -1.5,
              max: 1.5,
              ticks: {
                color: '#4a6280',
                stepSize: 1,
                callback: (v) => v === 1 ? 'Победа' : v === -1 ? 'Поражение' : '',
              },
              grid: { color: '#142338' },
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
    <select class="opp-select" bind:value={opponentFilter}>
      <option value="">Все оппоненты</option>
      {#each opponents as opp}
        <option value={opp.id}>{opp.name}</option>
      {/each}
    </select>
  </div>
  <div class="chart-body">
    <canvas bind:this={canvas}></canvas>
  </div>
</div>

<style>
  .chart-card {
    background: #0f2035;
    border: 1px solid #1f3a57;
    border-radius: 8px;
    padding: 16px;
  }

  .chart-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 12px;
    margin-bottom: 12px;
  }

  .chart-title {
    font-size: 0.72rem;
    font-weight: 600;
    letter-spacing: 0.07em;
    text-transform: uppercase;
    color: #4a6280;
    margin-bottom: 2px;
  }

  .win-rate {
    font-size: 0.8rem;
    color: #6b8aab;
  }

  .wins {
    color: #4caf82;
    font-weight: 600;
  }

  .sep {
    margin: 0 5px;
    color: #2a4f73;
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
    height: 180px;
    position: relative;
  }

  canvas {
    display: block;
    width: 100% !important;
    height: 100% !important;
  }
</style>
