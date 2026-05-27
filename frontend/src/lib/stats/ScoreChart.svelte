<script lang="ts">
  import { onMount } from 'svelte';
  import type { FighterBout } from '../api/types';

  interface Props {
    bouts: FighterBout[];
    selectedVideoId?: string;
    selectedWeeks?: string[];
    xAxisMode?: 'overview' | 'detail';
    scrollRatio?: number;
    onfilter?: (videoId: string) => void;
    onmodechange?: (mode: 'overview' | 'detail') => void;
    onscrollsync?: (ratio: number) => void;
  }

  let {
    bouts,
    selectedVideoId = '',
    selectedWeeks = [],
    xAxisMode = 'overview',
    scrollRatio = 0,
    onfilter,
    onmodechange,
    onscrollsync
  }: Props = $props();

  let canvas = $state<HTMLCanvasElement | undefined>(undefined);
  let scrollEl = $state<HTMLDivElement | undefined>(undefined);
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  let chart: any = null;
  const AXIS_TOGGLE_THRESHOLD = 11;

  function getISOWeek(dateStr: string): string {
    const date = new Date(dateStr);
    const d = new Date(Date.UTC(date.getFullYear(), date.getMonth(), date.getDate()));
    const day = d.getUTCDay() || 7;
    d.setUTCDate(d.getUTCDate() + 4 - day);
    const yearStart = new Date(Date.UTC(d.getUTCFullYear(), 0, 1));
    const week = Math.ceil((((d.getTime() - yearStart.getTime()) / 86400000) + 1) / 7);
    return `${d.getUTCFullYear()}-W${String(week).padStart(2, '0')}`;
  }

  function buildChartData(bouts: FighterBout[]) {
    const videoMap = new Map<string, { video_id: string; date: string; week: string; opponent_name: string; my: number; opp: number }>();
    for (const b of bouts) {
      const v = videoMap.get(b.video_id);
      if (v) { v.my += b.my_score; v.opp += b.opponent_score; }
      else videoMap.set(b.video_id, { video_id: b.video_id, date: b.video_date, week: getISOWeek(b.video_date), opponent_name: b.opponent_name, my: b.my_score, opp: b.opponent_score });
    }
    return [...videoMap.values()].sort((a, b) => a.date.localeCompare(b.date));
  }

  let dates = $derived.by(() => {
    const results = buildChartData(bouts);
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

  let canToggleAxisMode = $derived(buildChartData(bouts).length > AXIS_TOGGLE_THRESHOLD);
  let effectiveXAxisMode = $derived(canToggleAxisMode ? xAxisMode : 'detail');

  let chartWidth = $derived.by(() => {
    const sessions = buildChartData(bouts);
    if (!canToggleAxisMode || effectiveXAxisMode === 'overview') return '100%';
    if (!sessions.length) return '100%';
    const longestLabel = Math.max(...sessions.map(s => (s.opponent_name || s.date.slice(5)).length));
    const pointWidth = Math.max(60, Math.min(120, longestLabel * 8 + 20));
    return `${Math.max(720, sessions.length * pointWidth)}px`;
  });

  function formatShortDate(dateStr: string) {
    const ymd = dateStr.split('-');
    if (ymd.length !== 3) return dateStr;
    return `${ymd[0].slice(2)}.${ymd[1]}.${ymd[2]}`;
  }

  function dateRangeGroups(items: { date: string }[]) {
    const groupCount = Math.min(6, items.length);
    const groupSize = Math.ceil(items.length / groupCount);
    const groups: { start: number; end: number; label: string }[] = [];
    for (let start = 0; start < items.length; start += groupSize) {
      const end = Math.min(items.length - 1, start + groupSize - 1);
      const startLabel = formatShortDate(items[start].date);
      const endLabel = formatShortDate(items[end].date);
      groups.push({ start, end, label: startLabel === endLabel ? startLabel : `${startLabel}-${endLabel}` });
    }
    return groups;
  }

  function isSessionActive(session: { video_id: string; week: string }): boolean {
    if (selectedVideoId) return session.video_id === selectedVideoId;
    if (selectedWeeks.length > 0) return selectedWeeks.includes(session.week);
    return true;
  }

  function isSessionSelected(session: { video_id: string; week: string }): boolean {
    if (selectedVideoId) return session.video_id === selectedVideoId;
    if (selectedWeeks.length > 0) return selectedWeeks.includes(session.week);
    return false;
  }

  function handleScroll() {
    if (!scrollEl || !canToggleAxisMode || effectiveXAxisMode !== 'detail') return;
    const maxScroll = scrollEl.scrollWidth - scrollEl.clientWidth;
    if (maxScroll <= 0) return;
    onscrollsync?.(scrollEl.scrollLeft / maxScroll);
  }

  $effect(() => {
    if (!scrollEl || !canToggleAxisMode || effectiveXAxisMode !== 'detail') return;
    const maxScroll = scrollEl.scrollWidth - scrollEl.clientWidth;
    if (maxScroll <= 0) return;
    const target = maxScroll * scrollRatio;
    if (Math.abs(scrollEl.scrollLeft - target) > 1) scrollEl.scrollLeft = target;
  });

  $effect(() => {
    if (!canvas) return;
    let effectActive = true;
    const sessions = buildChartData(bouts);
    
    if (sessions.length === 0) {
      if (chart) { chart.destroy(); chart = null; }
      return;
    }

    const labels = effectiveXAxisMode === 'detail'
      ? sessions.map(s => s.opponent_name || s.date.slice(5))
      : sessions.map(() => '');
    const myData = sessions.map(s => s.my);
    const oppData = sessions.map(s => s.opp);

    import('chart.js').then(({ Chart, registerables }) => {
      if (!effectActive) return;
      Chart.register(...registerables);
      if (chart) { chart.destroy(); chart = null; }
      if (!canvas) return;

      const dayBoundaryPlugin = {
        id: 'dayBoundaries',
        beforeDraw(ch: any) {
          const { ctx, chartArea, scales } = ch;
          if (!chartArea || !scales?.x || sessions.length === 0) return;
          ctx.save();
          const meta = ch.getDatasetMeta(0);
          
          ctx.strokeStyle = 'rgba(255,255,255,0.2)';
          ctx.fillStyle = 'rgba(255,255,255,0.4)';
          ctx.font = 'bold 10px Inter';
          ctx.textAlign = 'center';
          
          let dayStartIdx = 0;
          for (let i = 0; i <= sessions.length; i++) {
            if (i === sessions.length || (i > 0 && sessions[i].date !== sessions[i-1].date)) {
              if (!meta.data[dayStartIdx] || !meta.data[i-1]) continue;
              const startX = meta.data[dayStartIdx].x;
              const endX = meta.data[i-1].x;
              const y = chartArea.bottom + 35;
              const dateStr = sessions[dayStartIdx].date;
              const ymd = dateStr.split('-');
              const shortDate = `${ymd[0].slice(2)}.${ymd[1]}.${ymd[2]}`;

              ctx.beginPath();
              ctx.moveTo(startX, y - 5);
              ctx.lineTo(startX, y);
              ctx.lineTo(endX, y);
              ctx.lineTo(endX, y - 5);
              ctx.stroke();

              ctx.fillText(shortDate, (startX + endX) / 2, y + 12);
              dayStartIdx = i;
            }
          }
          ctx.restore();
        }
      };

      const rangeBoundaryPlugin = {
        id: 'rangeBoundaries',
        beforeDraw(ch: any) {
          const { ctx, chartArea } = ch;
          if (!chartArea || sessions.length === 0) return;
          const meta = ch.getDatasetMeta(0);
          const groups = dateRangeGroups(sessions);
          ctx.save();
          ctx.strokeStyle = 'rgba(255,255,255,0.22)';
          ctx.fillStyle = 'rgba(255,255,255,0.42)';
          ctx.font = 'bold 10px Inter';
          ctx.textAlign = 'center';

          for (const group of groups) {
            if (!meta.data[group.start] || !meta.data[group.end]) continue;
            const startX = meta.data[group.start].x;
            const endX = meta.data[group.end].x;
            const y = chartArea.bottom + 20;

            ctx.beginPath();
            ctx.moveTo(startX, y - 5);
            ctx.lineTo(startX, y);
            ctx.lineTo(endX, y);
            ctx.lineTo(endX, y - 5);
            ctx.stroke();
            ctx.fillText(group.label, (startX + endX) / 2, y + 13);
          }
          ctx.restore();
        }
      };

      const isFaint = selectedVideoId || selectedWeeks.length > 0;

      chart = new Chart(canvas!, {
        type: 'line',
        plugins: effectiveXAxisMode === 'detail' ? [dayBoundaryPlugin] : [rangeBoundaryPlugin],
        data: {
          labels,
          datasets: [
            {
              label: 'Мои баллы',
              data: myData,
              borderColor: isFaint ? 'rgba(245, 158, 11, 0.2)' : '#f59e0b',
              backgroundColor: (ctx: any) => {
                const canvas = ctx.chart.canvas;
                const chartArea = ctx.chart.chartArea;
                if (!chartArea) return 'transparent';
                const gradient = canvas.getContext('2d').createLinearGradient(0, chartArea.top, 0, chartArea.bottom);
                const alpha = isFaint ? '0.05' : '0.2';
                gradient.addColorStop(0, `rgba(245, 158, 11, ${alpha})`);
                gradient.addColorStop(1, `rgba(245, 158, 11, 0)`);
                return gradient;
              },
              pointBackgroundColor: (ctx: any) => {
                const s = sessions[ctx.dataIndex];
                return isSessionActive(s) ? '#f59e0b' : 'rgba(245, 158, 11, 0.2)';
              },
              pointBorderColor: (ctx: any) => {
                const s = sessions[ctx.dataIndex];
                return isSessionActive(s) ? '#f59e0b' : 'rgba(245, 158, 11, 0.2)';
              },
              pointBorderWidth: 2,
              pointRadius: (ctx: any) => {
                const s = sessions[ctx.dataIndex];
                return isSessionSelected(s) ? 6 : 4;
              },
              tension: 0.4,
              cubicInterpolationMode: 'monotone',
              fill: true,
            },
            {
              label: 'Баллы оппонента',
              data: oppData,
              borderColor: isFaint ? 'rgba(239, 68, 68, 0.2)' : '#ef4444',
              backgroundColor: (ctx: any) => {
                const canvas = ctx.chart.canvas;
                const chartArea = ctx.chart.chartArea;
                if (!chartArea) return 'transparent';
                const gradient = canvas.getContext('2d').createLinearGradient(0, chartArea.top, 0, chartArea.bottom);
                const alpha = isFaint ? '0.05' : '0.2';
                gradient.addColorStop(0, `rgba(239, 68, 68, ${alpha})`);
                gradient.addColorStop(1, `rgba(239, 68, 68, 0)`);
                return gradient;
              },
              pointBackgroundColor: (ctx: any) => {
                const s = sessions[ctx.dataIndex];
                return isSessionActive(s) ? '#ef4444' : 'rgba(239, 68, 68, 0.2)';
              },
              pointBorderColor: (ctx: any) => {
                const s = sessions[ctx.dataIndex];
                return isSessionActive(s) ? '#ef4444' : 'rgba(239, 68, 68, 0.2)';
              },
              pointBorderWidth: 2,
              pointRadius: (ctx: any) => {
                const s = sessions[ctx.dataIndex];
                return isSessionSelected(s) ? 6 : 4;
              },
              tension: 0.4,
              cubicInterpolationMode: 'monotone',
              fill: true,
            }
          ],
        },
        options: {
          responsive: true,
          maintainAspectRatio: false,
          layout: {
            padding: { left: 10, right: 10, top: 10, bottom: 48 }
          },
          onClick: (e, elements) => {
            if (elements.length > 0 && onfilter) {
              const index = elements[0].index;
              const vid = sessions[index].video_id;
              if (vid === selectedVideoId) onfilter('');
              else onfilter(vid);
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
              ticks: { display: effectiveXAxisMode === 'detail', autoSkip: false, color: '#6b7280', font: { family: 'Inter', size: 10 }, maxRotation: 0 },
              grid: { display: false },
              border: { display: false }
            },
            y: {
              afterFit: (axis: any) => { axis.width = 40; },
              beginAtZero: true,
              ticks: { display: true, color: '#6b7280', font: { size: 10 } },
              grid: { display: false },
              border: { display: false },
            },
          },
        },
      });
    });

    return () => { 
      effectActive = false;
      if (chart) { chart.destroy(); chart = null; }
    };
  });

  onMount(() => () => { chart?.destroy(); chart = null; });
</script>

<div class="chart-card">
  <div class="card-header">
    <h3 class="chart-title">Динамика очков</h3>
    {#if canToggleAxisMode}
      <button
        class="axis-switch"
        type="button"
        aria-pressed={xAxisMode === 'detail'}
        aria-label={xAxisMode === 'detail' ? 'Показать обзор с диапазонами дат' : 'Показать подробную ось со скроллом'}
        title={xAxisMode === 'detail' ? 'Показать обзор с диапазонами дат' : 'Показать подробную ось со скроллом'}
        onclick={() => onmodechange?.(xAxisMode === 'detail' ? 'overview' : 'detail')}
      >
        <span class="switch-thumb"></span>
      </button>
    {/if}
  </div>
  <div class="chart-body">
    {#if buildChartData(bouts).length === 0}
      <div class="no-data">Нет данных для отображения</div>
    {/if}
    <div
      bind:this={scrollEl}
      class={`chart-scroll ${canToggleAxisMode && effectiveXAxisMode === 'detail' ? 'chart-scroll--detail' : ''}`}
      onscroll={handleScroll}
    >
      <div class="chart-canvas" style={`width: ${chartWidth};`}>
        <canvas bind:this={canvas}></canvas>
      </div>
    </div>
  </div>
  {#if dates.first}
    <div class="chart-footer">
      <span>{dates.first}</span>
      <span>{dates.last}</span>
    </div>
  {/if}
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
    position: relative;
    min-width: 0;
    overflow: hidden;
  }

  .card-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 20px;
  }

  .chart-title {
    font-size: 1rem;
    font-weight: 700;
    color: var(--text-primary);
    margin: 0;
  }

  .axis-switch {
    position: relative;
    flex: 0 0 auto;
    width: 46px;
    height: 24px;
    border: 1px solid rgba(148, 163, 184, 0.18);
    border-radius: 999px;
    background: rgba(15, 23, 42, 0.35);
    padding: 0;
    cursor: pointer;
    overflow: hidden;
    transition: background-color 180ms ease, border-color 180ms ease;
  }

  .switch-thumb {
    position: absolute;
    top: 50%;
    left: 2px;
    width: 18px;
    height: 18px;
    border-radius: 999px;
    background: rgba(148, 163, 184, 0.88);
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.22);
    transform: translateY(-50%);
    transition: transform 180ms ease, background-color 180ms ease;
  }

  .axis-switch[aria-pressed="true"] {
    background: rgba(245, 158, 11, 0.16);
    border-color: rgba(245, 158, 11, 0.4);
  }

  .axis-switch[aria-pressed="true"] .switch-thumb {
    transform: translate(22px, -50%);
    background: #f59e0b;
  }

  .chart-body {
    flex: 1;
    min-height: 0;
    min-width: 0;
    position: relative;
    overflow: hidden;
  }

  .chart-scroll {
    width: 100%;
    max-width: 100%;
    height: 100%;
    min-width: 0;
    overflow-x: hidden;
    overflow-y: hidden;
    scrollbar-width: thin;
    scrollbar-color: rgba(148, 163, 184, 0.35) transparent;
  }

  .chart-scroll--detail {
    overflow-x: auto;
  }

  .chart-scroll::-webkit-scrollbar {
    height: 6px;
  }

  .chart-scroll::-webkit-scrollbar-track {
    background: transparent;
  }

  .chart-scroll::-webkit-scrollbar-thumb {
    background: rgba(148, 163, 184, 0.35);
    border-radius: 999px;
  }

  .chart-canvas {
    min-width: 100%;
    height: 100%;
    position: relative;
  }

  .no-data {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-muted);
    font-size: 0.85rem;
    background: rgba(15, 23, 42, 0.4);
    backdrop-filter: blur(4px);
    border-radius: var(--radius-lg);
    z-index: 5;
  }

  .chart-footer {
    display: flex;
    justify-content: space-between;
    padding-top: 8px;
    font-size: 0.7rem;
    color: var(--text-muted);
    font-family: 'Inter', sans-serif;
  }

  canvas {
    display: block;
    width: 100% !important;
    height: 100% !important;
  }
</style>
