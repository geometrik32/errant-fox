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
  let showInfo = $state(false);

  $effect(() => {
    if (!canvas) return;
    
    const T = bouts.length;
    let hit = 0, afterblow = 0, late = 0, disqualification = 0, no_strike = 0, miss = 0, blocked = 0;

    if (T > 0) {
      for (const b of bouts) {
        switch(b.my_result) {
          case 'hit': hit++; break;
          case 'afterblow': afterblow++; break;
          case 'late': late++; break;
          case 'disqualification': disqualification++; break;
          case 'no_strike': no_strike++; break;
          case 'miss': miss++; break;
          case 'blocked': blocked++; break;
        }
      }
    }

    const data = T === 0 ? [0, 0, 0, 0, 0, 0, 0] : [
      (hit / T) * 100,
      100 - (afterblow / T * 100),
      100 - (late / T * 100),
      100 - (disqualification / T * 100),
      100 - (no_strike / T * 100),
      100 - (miss / T * 100),
      100 - (blocked / T * 100)
    ];

    import('chart.js').then(({ Chart, registerables }) => {
      Chart.register(...registerables);
      if (chart) { chart.destroy(); chart = null; }
      chart = new Chart(canvas!, {
        type: 'radar',
        data: {
          labels: ['Результативность', '!Афтерблоу!', '!Опоздал!', 'Техника', 'Оценка риска', 'Меткость', '!Заблок.!'],
          datasets: [{
            label: 'Конвертация',
            data,
            backgroundColor: (ctx: any) => {
              const canvas = ctx.chart.canvas;
              const chartArea = ctx.chart.chartArea;
              if (!chartArea) return 'transparent';
              const cx = (chartArea.left + chartArea.right) / 2;
              const cy = (chartArea.top + chartArea.bottom) / 2;
              const radius = Math.min(chartArea.right - chartArea.left, chartArea.bottom - chartArea.top) / 2;
              const gradient = canvas.getContext('2d').createRadialGradient(cx, cy, 0, cx, cy, radius);
              gradient.addColorStop(0, 'rgba(219, 132, 31, 0.05)');
              gradient.addColorStop(1, 'rgba(219, 132, 31, 0.4)');
              return gradient;
            },
            borderColor: '#db841f',
            pointBackgroundColor: '#fff',
            pointBorderColor: '#db841f',
            pointBorderWidth: 2,
            pointRadius: 3,
            pointHoverRadius: 5,
            borderWidth: 2,
          }]
        },
        options: {
          responsive: true,
          maintainAspectRatio: false,
          scales: {
            r: {
              angleLines: { color: 'rgba(255, 255, 255, 0.1)' },
              grid: { color: 'rgba(255, 255, 255, 0.1)' },
              pointLabels: {
                color: '#a0b4c8',
                font: { family: 'Inter', size: 12 }
              },
              min: 0,
              max: 100,
              ticks: { display: false }
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
                label: (ctx) => `${ctx.formattedValue}%`
              }
            }
          },
          layout: {
            padding: 20
          }
        }
      });
    });

    return () => { chart?.destroy(); chart = null; };
  });

  onMount(() => () => { chart?.destroy(); chart = null; });
</script>

<div class="radar-container glass-card">
  <button class="help-btn" onclick={() => showInfo = !showInfo} title="Как считаются метрики?">
    <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
      <circle cx="12" cy="12" r="10"/><path d="M9.09 9a3 3 0 0 1 5.83 1c0 2-3 3-3 3"/><line x1="12" y1="17" x2="12.01" y2="17"/>
    </svg>
  </button>

  {#if showInfo}
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="info-overlay" onclick={() => showInfo = false}>
      <div class="info-card" onclick={(e) => e.stopPropagation()}>
        <div class="info-header">
          <span>Методология расчёта</span>
          <button class="close-btn" onclick={() => showInfo = false}>&times;</button>
        </div>
        <table class="info-table">
          <thead>
            <tr>
              <th>Показатель</th>
              <th>Исход</th>
              <th>Формула (T — сумма сходов)</th>
            </tr>
          </thead>
          <tbody>
            <tr>
              <td>Результативность</td>
              <td>Попал</td>
              <td>(Попал / T) × 100</td>
            </tr>
            <tr>
              <td>!Афтерблоу!</td>
              <td>Афтерблоу</td>
              <td>100 – (Афтерблоу / T × 100)</td>
            </tr>
            <tr>
              <td>!Опоздал!</td>
              <td>Опоздал</td>
              <td>100 – (Опоздал / T × 100)</td>
            </tr>
            <tr>
              <td>Техника</td>
              <td>Неквалиф.</td>
              <td>100 – (Неквалиф. / T × 100)</td>
            </tr>
            <tr>
              <td>Оценка риска</td>
              <td>Не бил</td>
              <td>100 – (Не бил / T × 100)</td>
            </tr>
            <tr>
              <td>Меткость</td>
              <td>Промах</td>
              <td>100 – (Промах / T × 100)</td>
            </tr>
            <tr>
              <td>!Заблок.!</td>
              <td>Заблок.</td>
              <td>100 – (Заблок. / T × 100)</td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>
  {/if}

  <canvas bind:this={canvas}></canvas>
</div>

<style>
  .radar-container {
    width: 100%;
    height: 100%;
    background: var(--surface);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-lg);
    padding: 16px;
    display: flex;
    align-items: center;
    justify-content: center;
    position: relative;
  }
  
  canvas {
    display: block;
    width: 100% !important;
    height: 100% !important;
  }

  .help-btn {
    position: absolute;
    top: 16px;
    right: 16px;
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid var(--border-color);
    color: var(--text-secondary);
    width: 36px;
    height: 36px;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition: var(--transition);
    z-index: 10;
  }

  .help-btn:hover {
    background: rgba(255, 255, 255, 0.1);
    color: var(--text-primary);
    border-color: var(--border-strong);
  }

  .info-overlay {
    position: absolute;
    inset: 0;
    background: rgba(15, 23, 42, 0.85);
    z-index: 100;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 20px;
    border-radius: var(--radius-lg);
  }

  .info-card {
    background: #1e293b;
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 16px;
    padding: 24px;
    width: 100%;
    max-width: 500px;
    box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.5);
  }

  .info-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 20px;
    font-weight: 700;
    font-size: 1rem;
    color: var(--text-primary);
  }

  .close-btn {
    background: transparent;
    border: none;
    color: var(--text-muted);
    font-size: 1.5rem;
    cursor: pointer;
    line-height: 1;
  }

  .info-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 0.85rem;
  }

  .info-table th {
    text-align: left;
    padding: 8px;
    color: var(--text-secondary);
    border-bottom: 1px solid rgba(255, 255, 255, 0.1);
    font-weight: 600;
  }

  .info-table td {
    padding: 10px 8px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.05);
    color: var(--text-primary);
  }

  .info-table tr:last-child td {
    border-bottom: none;
  }
</style>
