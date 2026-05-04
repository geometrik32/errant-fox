<script lang="ts">
  import { onMount } from 'svelte';
  import type { FighterBout } from '../api/types';

  interface Props {
    bouts: FighterBout[];
    rawVideos?: any[];
    onfilter?: (week: string) => void;
  }

  let { bouts, rawVideos = [], onfilter }: Props = $props();

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

  function addWeeks(isoWeek: string, n: number): string {
    const [yearStr, wStr] = isoWeek.split('-W');
    const year = parseInt(yearStr);
    const week = parseInt(wStr);
    const jan4 = new Date(Date.UTC(year, 0, 4));
    const startOfWeek1 = new Date(jan4);
    startOfWeek1.setUTCDate(jan4.getUTCDate() - (jan4.getUTCDay() || 7) + 1);
    const ms = startOfWeek1.getTime() + (week - 1 + n) * 7 * 86400000;
    return getISOWeek(new Date(ms).toISOString().slice(0, 10));
  }

  function buildData(bouts: FighterBout[], rawVideos: any[]) {
    const taggedWeeks = new Map<string, number>();
    for (const b of bouts) {
      const week = getISOWeek(b.video_date);
      taggedWeeks.set(week, (taggedWeeks.get(week) ?? 0) + 1);
    }
    
    const untaggedWeeks = new Map<string, number>();
    for (const v of rawVideos) {
      if (v.date && !v.is_tagged) {
        const week = getISOWeek(v.date);
        untaggedWeeks.set(week, (untaggedWeeks.get(week) ?? 0) + 1);
      }
    }

    const allVideoWeeks = new Set<string>([...taggedWeeks.keys(), ...untaggedWeeks.keys()]);

    if (allVideoWeeks.size === 0) return { labels: [], taggedData: [], untaggedData: [], yearBoundaries: [] };

    const sortedWeeks = [...allVideoWeeks].sort();
    const firstYear = parseInt(sortedWeeks[0].split('-W')[0]);
    const currentYear = new Date().getFullYear();
    
    const firstWeek = `${firstYear}-W01`;
    const lastWeek = `${currentYear}-W52`;

    const allWeeks: string[] = [];
    let cur = firstWeek;
    while (cur <= lastWeek) {
      allWeeks.push(cur);
      cur = addWeeks(cur, 1);
    }

    const taggedData = allWeeks.map(w => taggedWeeks.get(w) ?? 0);
    const untaggedData = allWeeks.map(w => untaggedWeeks.get(w) ?? 0);

    const yearBoundaries: number[] = [];
    for (let i = 1; i < allWeeks.length; i++) {
      if (allWeeks[i].split('-W')[0] !== allWeeks[i - 1].split('-W')[0]) {
        yearBoundaries.push(i);
      }
    }

    const monthNames = ['Янв', 'Фев', 'Мар', 'Апр', 'Май', 'Июн', 'Июл', 'Авг', 'Сен', 'Окт', 'Ноя', 'Дек'];
    const labels = allWeeks.map((w, i) => {
      const [year, wStr] = w.split('-W');
      const wNum = parseInt(wStr);
      const date = new Date(parseInt(year), 0, 1 + (wNum - 1) * 7);
      const month = date.getMonth();
      const prevDate = i > 0 ? new Date(parseInt(allWeeks[i-1].split('-W')[0]), 0, 1 + (parseInt(allWeeks[i-1].split('-W')[1]) - 1) * 7) : null;
      if (i === 0 || (prevDate && month !== prevDate.getMonth())) {
        return monthNames[month];
      }
      return '';
    });

    return { labels, taggedData, untaggedData, yearBoundaries, allWeeks };
  }

  $effect(() => {
    if (!canvas) return;
    const { labels, taggedData, untaggedData, yearBoundaries, allWeeks = [] } = buildData(bouts, rawVideos);

    const yearLinePlugin = {
      id: 'yearLines',
      beforeDraw(ch: any) {
        if (!yearBoundaries.length) return;
        const { ctx, chartArea, scales } = ch;
        if (!chartArea || !scales?.x) return;
        ctx.save();
        ctx.strokeStyle = 'rgba(180,190,200,0.25)';
        ctx.lineWidth = 1;
        ctx.setLineDash([4, 3]);
        for (const idx of yearBoundaries) {
          const meta = ch.getDatasetMeta(0);
          if (!meta.data[idx] || !meta.data[idx - 1]) continue;
          const x = (meta.data[idx - 1].x + meta.data[idx].x) / 2;
          ctx.beginPath();
          ctx.moveTo(x, chartArea.top);
          ctx.lineTo(x, chartArea.bottom);
          ctx.stroke();
          const year = '20' + (allWeeks[idx]?.split('-W')[0]?.slice(2) ?? '');
          ctx.setLineDash([]);
          ctx.fillStyle = 'rgba(160,180,200,0.7)';
          ctx.font = '10px Inter, sans-serif';
          ctx.textAlign = 'left';
          ctx.fillText(year, x + 3, chartArea.top + 12);
        }
        ctx.restore();
      }
    };

    import('chart.js').then(({ Chart, registerables }) => {
      Chart.register(...registerables);
      if (chart) { chart.destroy(); chart = null; }
      chart = new Chart(canvas!, {
        type: 'bar',
        plugins: [yearLinePlugin],
        data: {
          labels,
          datasets: [
            {
              label: 'Размечено',
              data: taggedData,
              backgroundColor: '#fbbf24',
              borderRadius: 2,
              borderSkipped: false,
              barPercentage: 0.7,
            },
            {
              label: 'Не размечено',
              data: untaggedData,
              backgroundColor: 'rgba(100,130,160,0.3)',
              borderRadius: 2,
              borderSkipped: false,
              barPercentage: 0.7,
            }
          ],
        },
        options: {
          responsive: true,
          maintainAspectRatio: false,
          onClick: (_e, elements) => {
            if (elements.length > 0 && onfilter) {
              const week = allWeeks[elements[0].index];
              if (week) onfilter(week);
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
                title: (items) => allWeeks[items[0].dataIndex] ?? '',
              },
            },
          },
          scales: {
            x: {
              stacked: true,
              ticks: { color: '#6b7280', font: { family: 'Inter', size: 10 }, maxRotation: 0 },
              grid: { display: false },
              border: { display: false }
            },
            y: {
              stacked: true,
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
