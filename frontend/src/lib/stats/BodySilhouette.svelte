<script lang="ts">
  import type { FighterBout } from '../api/types';

  interface Props {
    bouts: FighterBout[];
    type: 'dealt' | 'received';
  }

  let { bouts, type }: Props = $props();

  const ZONES = ['Голова', 'Тело', 'Рука правая', 'Рука левая', 'Нога правая', 'Нога левая'] as const;

  let counts = $derived.by(() => {
    const map = new Map<string, number>(ZONES.map(z => [z, 0]));
    for (const b of bouts) {
      if (type === 'dealt' && b.my_result === 'hit' && b.my_hit_zone) {
        map.set(b.my_hit_zone, (map.get(b.my_hit_zone) ?? 0) + 1);
      }
      if (type === 'received' && b.opponent_result === 'hit' && b.opponent_hit_zone) {
        map.set(b.opponent_hit_zone, (map.get(b.opponent_hit_zone) ?? 0) + 1);
      }
    }
    return map;
  });

  let maxCount = $derived(Math.max(...[...counts.values()], 1));

  function opacity(zone: string): number {
    const c = counts.get(zone) ?? 0;
    return c / maxCount * 0.9 + 0.1;
  }

  function fill(zone: string): string {
    return `rgba(219, 132, 31, ${opacity(zone).toFixed(2)})`;
  }

  function countLabel(zone: string): number {
    return counts.get(zone) ?? 0;
  }
</script>

<div class="silhouette-card">
  <div class="silhouette-title">
    {type === 'dealt' ? 'Нанесённый урон' : 'Полученный урон'}
  </div>
  <div class="silhouette-wrap">
    <svg viewBox="0 0 160 290" xmlns="http://www.w3.org/2000/svg" class="svg">
      <!-- Decorative outline (neck connector) -->
      <line x1="80" y1="43" x2="80" y2="50" stroke="#1f3a57" stroke-width="8" />

      <!-- Head / Голова -->
      <circle cx="80" cy="24" r="19"
        fill={fill('Голова')}
        stroke="#2a4f73" stroke-width="1"
        class="zone"
      >
        <title>Голова: {countLabel('Голова')}</title>
      </circle>

      <!-- Body / Тело -->
      <rect x="52" y="50" width="56" height="78" rx="4"
        fill={fill('Тело')}
        stroke="#2a4f73" stroke-width="1"
        class="zone"
      >
        <title>Тело: {countLabel('Тело')}</title>
      </rect>

      <!-- Right arm / Рука правая (person's right = viewer's left) -->
      <rect x="4" y="56" width="48" height="20" rx="5"
        fill={fill('Рука правая')}
        stroke="#2a4f73" stroke-width="1"
        class="zone"
      >
        <title>Рука правая: {countLabel('Рука правая')}</title>
      </rect>

      <!-- Left arm / Рука левая (person's left = viewer's right) -->
      <rect x="108" y="56" width="48" height="20" rx="5"
        fill={fill('Рука левая')}
        stroke="#2a4f73" stroke-width="1"
        class="zone"
      >
        <title>Рука левая: {countLabel('Рука левая')}</title>
      </rect>

      <!-- Right leg / Нога правая -->
      <rect x="55" y="132" width="24" height="100" rx="5"
        fill={fill('Нога правая')}
        stroke="#2a4f73" stroke-width="1"
        class="zone"
      >
        <title>Нога правая: {countLabel('Нога правая')}</title>
      </rect>

      <!-- Left leg / Нога левая -->
      <rect x="81" y="132" width="24" height="100" rx="5"
        fill={fill('Нога левая')}
        stroke="#2a4f73" stroke-width="1"
        class="zone"
      >
        <title>Нога левая: {countLabel('Нога левая')}</title>
      </rect>

      <!-- Count labels -->
      <text x="80" y="28" text-anchor="middle" dominant-baseline="middle" class="label">{countLabel('Голова')}</text>
      <text x="80" y="89" text-anchor="middle" dominant-baseline="middle" class="label">{countLabel('Тело')}</text>
      <text x="28" y="67" text-anchor="middle" dominant-baseline="middle" class="label">{countLabel('Рука правая')}</text>
      <text x="132" y="67" text-anchor="middle" dominant-baseline="middle" class="label">{countLabel('Рука левая')}</text>
      <text x="67" y="182" text-anchor="middle" dominant-baseline="middle" class="label">{countLabel('Нога правая')}</text>
      <text x="93" y="182" text-anchor="middle" dominant-baseline="middle" class="label">{countLabel('Нога левая')}</text>
    </svg>

    <!-- Legend -->
    <div class="legend">
      {#each ZONES as zone}
        {@const c = countLabel(zone)}
        <div class="legend-row">
          <div class="legend-swatch" style:background={fill(zone)}></div>
          <span class="legend-zone">{zone}</span>
          <span class="legend-count">{c}</span>
        </div>
      {/each}
    </div>
  </div>
</div>

<style>
  .silhouette-card {
    background: #0f2035;
    border: 1px solid #1f3a57;
    border-radius: 8px;
    padding: 16px;
  }

  .silhouette-title {
    font-size: 0.72rem;
    font-weight: 600;
    letter-spacing: 0.07em;
    text-transform: uppercase;
    color: #4a6280;
    margin-bottom: 12px;
  }

  .silhouette-wrap {
    display: flex;
    align-items: flex-start;
    gap: 20px;
  }

  .svg {
    width: 140px;
    flex-shrink: 0;
  }

  .zone {
    cursor: default;
    transition: filter 0.1s;
  }

  .zone:hover {
    filter: brightness(1.25);
  }

  .label {
    font-size: 9px;
    fill: #e8edf2;
    font-weight: 600;
    pointer-events: none;
  }

  .legend {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 6px;
    justify-content: center;
  }

  .legend-row {
    display: flex;
    align-items: center;
    gap: 7px;
  }

  .legend-swatch {
    width: 12px;
    height: 12px;
    border-radius: 2px;
    flex-shrink: 0;
    border: 1px solid rgba(219, 132, 31, 0.3);
  }

  .legend-zone {
    flex: 1;
    font-size: 0.76rem;
    color: #6b8aab;
  }

  .legend-count {
    font-size: 0.76rem;
    font-weight: 600;
    color: #a0b4c8;
    min-width: 16px;
    text-align: right;
  }
</style>
