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
            backgroundColor: 'rgba(219, 132, 31, 0.2)',
            borderColor: '#db841f',
            pointBackgroundColor: '#db841f',
            pointBorderColor: '#fff',
            pointHoverBackgroundColor: '#fff',
            pointHoverBorderColor: '#db841f',
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
                font: { family: 'Inter', size: 10 }
              },
              ticks: { display: false, min: 0, max: 100 }
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
          }
        }
      });
    });

    return () => { chart?.destroy(); chart = null; };
  });

  onMount(() => () => { chart?.destroy(); chart = null; });
</script>

<div class="radar-container glass-card">
  <canvas bind:this={canvas}></canvas>
</div>

<style>
  .radar-container {
    width: 100%;
    height: 100%;
    min-height: 200px;
    background: var(--surface);
    backdrop-filter: var(--glass-blur);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-lg);
    padding: 16px;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  
  canvas {
    display: block;
    width: 100% !important;
    height: 100% !important;
  }
</style>
