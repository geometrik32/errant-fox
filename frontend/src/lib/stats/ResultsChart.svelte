<script lang="ts">
  import { onMount } from 'svelte';
  import type { FighterBout } from '../api/types';

  interface Props {
    bouts: FighterBout[];
    videoLabels?: Map<string, string>;
    selectedVideoId?: string;
    selectedWeek?: string;
    onfilter?: (videoId: string) => void;
  }

  let { bouts, videoLabels = new Map(), selectedVideoId = '', selectedWeek = '', onfilter }: Props = $props();

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

  // aggregate per-video win/loss
  function computeVideoResults(bouts: FighterBout[]) {
    const videoMap = new Map<string, { video_id: string; date: string; week: string; opponent_name: string; my: number; opp: number }>();
    for (const b of bouts) {
      const v = videoMap.get(b.video_id);
      if (v) { v.my += b.my_score; v.opp += b.opponent_score; }
      else videoMap.set(b.video_id, { video_id: b.video_id, date: b.video_date, week: getISOWeek(b.video_date), opponent_name: b.opponent_name, my: b.my_score, opp: b.opponent_score });
    }
    return [...videoMap.values()]
      .sort((a, b) => a.date.localeCompare(b.date))
      .map(v => ({ video_id: v.video_id, date: v.date, week: v.week, opponent_name: v.opponent_name, result: v.my > v.opp ? 1 : v.my < v.opp ? -1 : 0 }));
  }

  let winRate = $derived.by(() => {
    const results = computeVideoResults(bouts);
    if (!results.length) return null;
    const wins = results.filter(r => r.result === 1).length;
    return { wins, total: results.length, pct: Math.round(wins / results.length * 100) };
  });

  let dates = $derived.by(() => {
    const results = computeVideoResults(bouts);
    if (!results.length) return { first: '', last: '' };
    const formatD = (d: string) => {
      const parts = d.split('-');
      if (parts.length === 3) return `${parts[0].slice(2)}.${parts[1]}.${parts[2]}`;
      return d;
    };
    return { 
      first: formatD(results[0].date), 
      last: formatD(results[results.length - 1].date) 
    };
  });

  let isDestroyed = false;
  $effect(() => {
    if (!canvas) return;
    isDestroyed = false;
    const results = computeVideoResults(bouts);
    const labels = results.map(r => r.opponent_name || r.date.slice(5));
    const data = results.map(r => r.result);

    import('chart.js').then(({ Chart, registerables }) => {
      if (isDestroyed) return;
      Chart.register(...registerables);
      if (chart) { chart.destroy(); chart = null; }
      if (!canvas) return;

      const dayBoundaryPlugin = {
        id: 'dayBoundaries',
        beforeDraw(ch: any) {
          const { ctx, chartArea, scales } = ch;
          if (!chartArea || !scales?.x || results.length === 0) return;
          ctx.save();
          const meta = ch.getDatasetMeta(0);
          
          // 1. Draw date brackets and text
          ctx.strokeStyle = 'rgba(255,255,255,0.2)';
          ctx.fillStyle = 'rgba(255,255,255,0.4)';
          ctx.font = 'bold 10px Inter';
          ctx.textAlign = 'center';
          
          let dayStartIdx = 0;
          for (let i = 0; i <= results.length; i++) {
            // If date changed or reached end
            if (i === results.length || (i > 0 && results[i].date !== results[i-1].date)) {
              const startX = meta.data[dayStartIdx].x;
              const endX = meta.data[i-1].x;
              const y = chartArea.bottom + 35;
              const dateStr = results[dayStartIdx].date;
              const ymd = dateStr.split('-');
              const shortDate = `${ymd[0].slice(2)}.${ymd[1]}.${ymd[2]}`;

              // Draw bracket
              ctx.beginPath();
              ctx.moveTo(startX, y - 5);
              ctx.lineTo(startX, y);
              ctx.lineTo(endX, y);
              ctx.lineTo(endX, y - 5);
              ctx.stroke();

              // Draw text
              ctx.fillText(shortDate, (startX + endX) / 2, y + 12);
              
              dayStartIdx = i;
            }
          }
          
          ctx.restore();
        }
      };

      const isFaint = selectedVideoId || selectedWeek;

      chart = new Chart(canvas, {
        type: 'line',
        plugins: [dayBoundaryPlugin],
        data: {
          labels,
          datasets: [{
            label: 'Результат',
            data,
            borderColor: isFaint ? 'rgba(107, 114, 128, 0.2)' : '#6b7280',
            pointBackgroundColor: data.map((v, i) => {
              const r = results[i];
              const color = v === 1 ? '#10b981' : v === -1 ? '#ef4444' : '#fbbf24';
              const active = (!selectedVideoId && !selectedWeek) || 
                             (selectedVideoId ? r.video_id === selectedVideoId : r.week === selectedWeek);
              return active ? color : color + '33'; // 20% opacity
            }),
            pointBorderColor: data.map((v, i) => {
              const r = results[i];
              const color = v === 1 ? '#10b981' : v === -1 ? '#ef4444' : '#fbbf24';
              const active = (!selectedVideoId && !selectedWeek) || 
                             (selectedVideoId ? r.video_id === selectedVideoId : r.week === selectedWeek);
              return active ? color : color + '33';
            }),
            pointBorderWidth: 2,
            pointRadius: (ctx: any) => {
              const r = results[ctx.dataIndex];
              const active = (selectedVideoId ? r.video_id === selectedVideoId : r.week === selectedWeek);
              return active ? 7 : 5;
            },
            pointHoverRadius: 8,
            tension: 0.4,
            cubicInterpolationMode: 'monotone',
            backgroundColor: (ctx: any) => {
              const canvas = ctx.chart.canvas;
              const chartArea = ctx.chart.chartArea;
              if (!chartArea) return 'transparent';
              const gradient = canvas.getContext('2d').createLinearGradient(0, chartArea.top, 0, chartArea.bottom);
              const alpha = isFaint ? '0.05' : '0.2';
              gradient.addColorStop(0, `rgba(16, 185, 129, ${alpha})`); // Green
              gradient.addColorStop(0.5, 'rgba(0, 0, 0, 0)');      // Transparent center
              gradient.addColorStop(1, `rgba(239, 68, 68, ${alpha})`);   // Red
              return gradient;
            },
            fill: 'origin',
          }],
        },
        options: {
          responsive: true,
          maintainAspectRatio: false,
          layout: {
            padding: { left: 10, right: 10, top: 10, bottom: 45 }
          },
          onClick: (e, elements) => {
            if (elements.length > 0 && onfilter) {
              const index = elements[0].index;
              const vid = results[index].video_id;
              if (vid === selectedVideoId) {
                onfilter('');
              } else {
                onfilter(vid);
              }
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
              callbacks: {
                title: (items) => {
                  const idx = items[0].dataIndex;
                  return `${results[idx].opponent_name} (${results[idx].date})`;
                },
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
              afterFit: (axis: any) => { axis.width = 40; },
              min: -1.2, // Small padding for the points
              max: 1.2,
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

    return () => { 
      isDestroyed = true;
      if (chart) { chart.destroy(); chart = null; }
    };
  });

  onMount(() => () => { chart?.destroy(); chart = null; });
</script>

<div class="chart-card">
  <div class="chart-header">
    <h3 class="chart-title">Динамика боёв</h3>
  </div>
  <div class="chart-body">
    <canvas bind:this={canvas}></canvas>
  </div>
</div>

<style>
  .chart-card {
    background: var(--surface);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-2xl);
    box-shadow: var(--shadow-sm);
    padding: 24px 24px 12px 24px;
    height: 100%;
    display: flex;
    flex-direction: column;
  }

  .chart-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 20px;
  }

  .chart-title {
    font-size: 1rem;
    font-weight: 700;
    color: var(--text-primary);
    margin: 0;
  }

  .chart-body {
    flex: 1;
    min-height: 0;
    position: relative;
  }

  canvas {
    display: block;
    width: 100% !important;
    height: 100% !important;
  }
</style>
