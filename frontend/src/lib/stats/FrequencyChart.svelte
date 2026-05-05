<script lang="ts">
  import { onMount } from 'svelte';
  import type { FighterBout } from '../api/types';

  interface Props {
    bouts: FighterBout[];
    rawVideos?: any[];
    selectedWeek?: string;
    onfilter?: (week: string) => void;
  }

  let { bouts, rawVideos = [], selectedWeek = '', onfilter }: Props = $props();

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
    // 1. Count unique videos that HAVE bouts (tagged/marked)
    const taggedVideoIdsPerWeek = new Map<string, Set<string>>();
    for (const b of bouts) {
      const week = getISOWeek(b.video_date);
      if (!taggedVideoIdsPerWeek.has(week)) taggedVideoIdsPerWeek.set(week, new Set());
      taggedVideoIdsPerWeek.get(week)!.add(b.video_id);
    }
    
    const taggedCountPerWeek = new Map<string, number>();
    for (const [week, ids] of taggedVideoIdsPerWeek.entries()) {
      taggedCountPerWeek.set(week, ids.size);
    }
    
    // 2. Count TOTAL unique videos from rawVideos
    const totalVideoIdsPerWeek = new Map<string, Set<string>>();
    for (const v of rawVideos) {
      if (v.date) {
        const week = getISOWeek(v.date);
        if (!totalVideoIdsPerWeek.has(week)) totalVideoIdsPerWeek.set(week, new Set());
        totalVideoIdsPerWeek.get(week)!.add(v.id || v.video_id);
      }
    }

    const untaggedCountPerWeek = new Map<string, number>();
    const allVideoWeeks = new Set<string>([...taggedCountPerWeek.keys(), ...totalVideoIdsPerWeek.keys()]);

    for (const week of allVideoWeeks) {
      const total = totalVideoIdsPerWeek.get(week)?.size ?? 0;
      const tagged = taggedCountPerWeek.get(week) ?? 0;
      // We take max(0, total - tagged) just in case data is inconsistent
      untaggedCountPerWeek.set(week, Math.max(0, total - tagged));
    }

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

    const taggedData = allWeeks.map(w => taggedCountPerWeek.get(w) ?? 0);
    const untaggedData = allWeeks.map(w => untaggedCountPerWeek.get(w) ?? 0);

    const yearBoundaries: number[] = [];
    for (let i = 1; i < allWeeks.length; i++) {
      if (allWeeks[i].split('-W')[0] !== allWeeks[i - 1].split('-W')[0]) {
        yearBoundaries.push(i);
      }
    }

    const labels = allWeeks.map((w, i) => {
      const [year, wStr] = w.split('-W');
      return parseInt(wStr); // Just the week number
    });

    return { labels, taggedData, untaggedData, yearBoundaries, allWeeks };
  }

  let isDestroyed = false;
  $effect(() => {
    if (!canvas) return;
    isDestroyed = false;
    const { labels, taggedData, untaggedData, yearBoundaries, allWeeks = [] } = buildData(bouts, rawVideos);

    const monthBoundaryPlugin = {
      id: 'monthBoundaries',
      beforeDraw(ch: any) {
        const { ctx, chartArea, scales } = ch;
        if (!chartArea || !scales?.x || allWeeks.length === 0) return;
        ctx.save();
        const meta = ch.getDatasetMeta(0);
        const monthNames = ['Янв', 'Фев', 'Мар', 'Апр', 'Май', 'Июн', 'Июл', 'Авг', 'Сен', 'Окт', 'Ноя', 'Дек'];
        
        ctx.strokeStyle = 'rgba(255,255,255,0.2)';
        ctx.fillStyle = 'rgba(255,255,255,0.4)';
        ctx.font = 'bold 10px Inter';
        ctx.textAlign = 'center';

        let monthStartIdx = 0;
        const getMonth = (w: string) => {
          const [year, wStr] = w.split('-W');
          const d = new Date(parseInt(year), 0, 1 + (parseInt(wStr) - 1) * 7);
          return d.getMonth();
        };

        for (let i = 0; i <= allWeeks.length; i++) {
          if (i === allWeeks.length || (i > 0 && getMonth(allWeeks[i]) !== getMonth(allWeeks[i-1]))) {
            const startX = meta.data[monthStartIdx].x;
            const endX = meta.data[i-1].x;
            const y = chartArea.bottom + 35; // Matches ResultsChart
            const monthIdx = getMonth(allWeeks[monthStartIdx]);

            // Draw bracket if month has more than 1 week or it's just a small dash
            ctx.beginPath();
            ctx.moveTo(startX, y - 5);
            ctx.lineTo(startX, y);
            ctx.lineTo(endX, y);
            ctx.lineTo(endX, y - 5);
            ctx.stroke();

            // Draw month name
            ctx.fillText(monthNames[monthIdx], (startX + endX) / 2, y + 15);
            
            monthStartIdx = i;
          }
        }
        ctx.restore();
      }
    };

    import('chart.js').then(({ Chart, registerables }) => {
      if (isDestroyed) return;
      Chart.register(...registerables);
      if (chart) { chart.destroy(); chart = null; }
      if (!canvas) return;

      chart = new Chart(canvas, {
        type: 'bar',
        plugins: [monthBoundaryPlugin],
        data: {
          labels: labels as string[],
          datasets: [
            {
              label: 'Размечено',
              data: taggedData,
              backgroundColor: (ctx: any) => {
                const week = allWeeks[ctx.dataIndex];
                if (!selectedWeek || week === selectedWeek) return '#f59e0b';
                return 'rgba(245, 158, 11, 0.2)';
              },
              borderRadius: 2,
              borderSkipped: false,
              barPercentage: 0.8,
              categoryPercentage: 0.8,
            },
            {
              label: 'Всего боёв',
              data: untaggedData,
              backgroundColor: (ctx: any) => {
                const week = allWeeks[ctx.dataIndex];
                if (!selectedWeek || week === selectedWeek) return '#334155';
                return 'rgba(51, 65, 85, 0.2)';
              },
              borderRadius: 2,
              borderSkipped: false,
              barPercentage: 0.8,
              categoryPercentage: 0.8,
            }
          ],
        },
        options: {
          responsive: true,
          maintainAspectRatio: false,
          onClick: (_e, elements) => {
            if (elements.length > 0 && onfilter) {
              const week = allWeeks[elements[0].index];
              if (week === selectedWeek) {
                onfilter('');
              } else {
                onfilter(week);
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
                  return allWeeks[idx] ?? '';
                },
                label: (ctx) => {
                  const idx = ctx.dataIndex;
                  const tagged = taggedData[idx];
                  const untagged = untaggedData[idx];
                  const total = tagged + untagged;
                  if (ctx.datasetIndex === 0) return `Размечено: ${tagged}`;
                  return `Всего: ${total}`;
                }
              },
            },
          },
          layout: {
            padding: { left: 10, right: 10, top: 10, bottom: 60 }
          },
          scales: {
            x: {
              stacked: true,
              ticks: { 
                color: '#6b7280', 
                font: { family: 'Inter', size: 9 }, 
                maxRotation: 0,
                autoSkip: false,
                callback: function(val, index) {
                  return labels[index];
                }
              },
              grid: { display: false },
              border: { display: false }
            },
            y: {
              stacked: true,
              beginAtZero: true,
              ticks: { display: true, color: '#6b7280', font: { size: 10 }, stepSize: 1 },
              grid: { display: true, color: 'rgba(255,255,255,0.03)' },
              border: { display: false }
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
  <div class="chart-title">Частота боёв</div>
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

  .chart-title {
    font-size: 1rem;
    font-weight: 700;
    color: var(--text-primary);
    margin: 0 0 20px 0;
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
