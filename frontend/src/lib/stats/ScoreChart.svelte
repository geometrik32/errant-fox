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

  function buildVideoScores(bouts: FighterBout[]) {
    const videoMap = new Map<string, { date: string; score: number }>();
    for (const b of bouts) {
      const v = videoMap.get(b.video_id);
      if (v) { v.score += b.my_score; }
      else videoMap.set(b.video_id, { date: b.video_date, score: b.my_score });
    }
    return [...videoMap.values()].sort((a, b) => a.date.localeCompare(b.date));
  }

  $effect(() => {
    if (!canvas) return;
    const sessions = buildVideoScores(bouts);
    const labels = sessions.map(s => s.date);
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
    background: #0f2035;
    border: 1px solid #1f3a57;
    border-radius: 8px;
    padding: 16px;
  }

  .chart-title {
    font-size: 0.72rem;
    font-weight: 600;
    letter-spacing: 0.07em;
    text-transform: uppercase;
    color: #4a6280;
    margin-bottom: 12px;
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
