<script lang="ts">
  import { onMount } from 'svelte';
  import type { FighterBout } from '../api/types';

  interface Props {
    bouts: FighterBout[];
    onfilter?: (week: string) => void;
  }

  let { bouts, onfilter }: Props = $props();

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
    const weekVideos = new Map<string, Set<string>>();
    for (const b of bouts) {
      const week = getISOWeek(b.video_date);
      if (!weekVideos.has(week)) weekVideos.set(week, new Set());
      weekVideos.get(week)!.add(b.video_id);
    }
    const sorted = [...weekVideos.entries()].sort((a, b) => a[0].localeCompare(b[0]));
    return {
      labels: sorted.map(([w]) => w),
      data: sorted.map(([, videos]) => videos.size),
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
            label: 'Сходы',
            data,
            backgroundColor: '#fbbf24',
            borderRadius: Number.MAX_VALUE,
            borderSkipped: false,
            barPercentage: 0.5,
          }],
        },
        options: {
          responsive: true,
          maintainAspectRatio: false,
          onClick: (e, elements) => {
            if (elements.length > 0 && onfilter) {
              const week = labels[elements[0].index];
              onfilter(week);
            }
          },
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
              ticks: { color: '#6b7280', font: { family: 'Inter', size: 11 }, maxRotation: 0 },
              grid: { display: false },
              border: { display: false }
            },
            y: {
              beginAtZero: true,
              ticks: { display: true, color: '#6b7280', font: { size: 10 }, stepSize: 1 },
              grid: { display: false },
              border: { display: false }
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

  .chart-title {
    font-size: 1rem;
    font-weight: 700;
    color: var(--text-primary);
    margin: 0 0 20px 0;
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
