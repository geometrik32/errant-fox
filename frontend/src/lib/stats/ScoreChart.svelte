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

  function buildVideoScores(bouts: FighterBout[]) {
    const videoMap = new Map<string, { video_id: string; date: string; score: number }>();
    for (const b of bouts) {
      const v = videoMap.get(b.video_id);
      if (v) { v.score += b.my_score; }
      else videoMap.set(b.video_id, { video_id: b.video_id, date: b.video_date, score: b.my_score });
    }
    return [...videoMap.values()].sort((a, b) => a.date.localeCompare(b.date));
  }

  $effect(() => {
    if (!canvas) return;
    const sessions = buildVideoScores(bouts);
    const labels = sessions.map(s => videoLabels.get(s.video_id) ?? s.date);
    const data = sessions.map(s => s.score);

    import('chart.js').then(({ Chart, registerables }) => {
      Chart.register(...registerables);
      if (chart) { chart.destroy(); chart = null; }
      chart = new Chart(canvas!, {
        type: 'line',
        data: {
          labels,
          datasets: [{
            label: 'Очки',
            data,
            borderColor: '#DB841F',
            backgroundColor: 'rgba(219, 132, 31, 0.12)',
            pointBackgroundColor: '#DB841F',
            pointRadius: 4,
            pointHoverRadius: 6,
            tension: 0.3,
            fill: true,
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
            },
          },
          scales: {
            x: {
              ticks: { color: '#4a6280', font: { size: 11 }, maxRotation: 45 },
              grid: { color: '#142338' },
            },
            y: {
              beginAtZero: true,
              ticks: { color: '#4a6280', stepSize: 1 },
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
  <div class="chart-title">Прогресс по баллам</div>
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
    border-radius: var(--radius-md);
    box-shadow: var(--shadow-sm);
    padding: 20px;
  }

  .chart-title {
    font-size: 0.8rem;
    font-weight: 600;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--text-secondary);
    margin-bottom: 16px;
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
