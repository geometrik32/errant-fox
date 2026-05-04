<script lang="ts">
  import { onMount } from 'svelte';
  import type { FighterBout } from '../api/types';

  interface Props {
    bouts: FighterBout[];
    videoLabels?: Map<string, string>;
    onfilter?: (date: string) => void;
  }

  let { bouts, videoLabels = new Map(), onfilter }: Props = $props();

  let canvas = $state<HTMLCanvasElement | undefined>(undefined);
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  let chart: any = null;

  function buildChartData(bouts: FighterBout[]) {
    const videoMap = new Map<string, { video_id: string; date: string; opponent_name: string; my: number; opp: number }>();
    for (const b of bouts) {
      const v = videoMap.get(b.video_id);
      if (v) { v.my += b.my_score; v.opp += b.opponent_score; }
      else videoMap.set(b.video_id, { video_id: b.video_id, date: b.video_date, opponent_name: b.opponent_name, my: b.my_score, opp: b.opponent_score });
    }
    return [...videoMap.values()].sort((a, b) => a.date.localeCompare(b.date));
  }

  $effect(() => {
    if (!canvas) return;
    const sessions = buildChartData(bouts);
    const labels = sessions.map(s => s.opponent_name || s.date.slice(5));
    const myData = sessions.map(s => s.my);
    const oppData = sessions.map(s => s.opp);

    import('chart.js').then(({ Chart, registerables }) => {
      Chart.register(...registerables);
      if (chart) { chart.destroy(); chart = null; }
      chart = new Chart(canvas!, {
        type: 'line',
        data: {
          labels,
          datasets: [
            {
              label: 'Мои баллы',
              data: myData,
              borderColor: '#fbbf24',
              backgroundColor: 'rgba(251, 191, 36, 0.1)',
              pointBackgroundColor: '#ffffff',
              pointBorderColor: '#fbbf24',
              pointBorderWidth: 2,
              pointRadius: 4,
              tension: 0.4,
              fill: true,
            },
            {
              label: 'Баллы оппонента',
              data: oppData,
              borderColor: '#60a5fa',
              backgroundColor: 'transparent',
              pointBackgroundColor: '#ffffff',
              pointBorderColor: '#60a5fa',
              pointBorderWidth: 2,
              pointRadius: 4,
              tension: 0.4,
              fill: false,
            }
          ],
        },
        options: {
          responsive: true,
          maintainAspectRatio: false,
          onClick: (e, elements) => {
            if (elements.length > 0 && onfilter) {
              const index = elements[0].index;
              const date = sessions[index].date;
              if (date) onfilter(date);
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
              ticks: { display: true, color: '#6b7280', font: { size: 10 } },
              grid: { display: false },
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
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-sm);
    padding: 24px;
    height: 100%;
    display: flex;
    flex-direction: column;
  }

  .chart-title {
    display: none;
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
