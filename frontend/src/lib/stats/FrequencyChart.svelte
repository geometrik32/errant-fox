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

  function getISOWeek(dateStr: string): string {
    const date = new Date(dateStr);
    const d = new Date(Date.UTC(date.getFullYear(), date.getMonth(), date.getDate()));
    const day = d.getUTCDay() || 7;
    d.setUTCDate(d.getUTCDate() + 4 - day);
    const yearStart = new Date(Date.UTC(d.getUTCFullYear(), 0, 1));
    const week = Math.ceil((((d.getTime() - yearStart.getTime()) / 86400000) + 1) / 7);
    return `${d.getUTCFullYear()}-W${String(week).padStart(2, '0')}`;
  }

  function buildData(bouts: FighterBout[]) {
    const counts = new Map<string, number>();
    for (const b of bouts) {
      const week = getISOWeek(b.video_date);
      counts.set(week, (counts.get(week) ?? 0) + 1);
    }
    const sorted = [...counts.entries()].sort((a, b) => a[0].localeCompare(b[0]));
    return {
      labels: sorted.map(([w]) => w),
      data: sorted.map(([, n]) => n),
    };
  }

  $effect(() => {
    if (!canvas) return;
    const { labels, data } = buildData(bouts);

    import('chart.js').then(({ Chart, registerables }) => {
      Chart.register(...registerables);
      if (chart) { chart.destroy(); chart = null; }
      chart = new Chart(canvas!, {
        type: 'bar',
        data: {
          labels,
          datasets: [{
            label: 'Боёв',
            data,
            backgroundColor: 'rgba(219, 132, 31, 0.6)',
            borderColor: '#DB841F',
            borderWidth: 1,
            borderRadius: 3,
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
  <div class="chart-title">Частота поединков</div>
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
