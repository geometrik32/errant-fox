<script lang="ts">
  import type { FighterBout } from '../api/types';

  interface Props {
    bouts: FighterBout[];
    type: 'dealt' | 'received';
  }

  let { bouts, type }: Props = $props();

  const ZONES = [
    'Голова', 'Шея', 'Плечи', 'Предплечья', 'Кисти',
    'Тело', 'Таз', 'Бедро', 'Голень',
  ] as const;

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
    return c / maxCount * 0.85 + 0.12;
  }

  function fill(zone: string): string {
    return `rgba(219, 132, 31, ${opacity(zone).toFixed(2)})`;
  }

  function cnt(zone: string): number {
    return counts.get(zone) ?? 0;
  }
</script>

<div class="silhouette-card">
  <div class="silhouette-title">
    {type === 'dealt' ? 'Нанесённый урон' : 'Полученный урон'}
  </div>
  <div class="silhouette-wrap">

    <svg viewBox="0 0 90 230" xmlns="http://www.w3.org/2000/svg" class="svg">

      <!-- Голова -->
      <circle cx="45" cy="14" r="12" fill={fill('Голова')} stroke="#2a4f73" stroke-width="1" class="zone">
        <title>Голова: {cnt('Голова')}</title>
      </circle>

      <!-- Шея -->
      <rect x="38" y="27" width="14" height="10" rx="3" fill={fill('Шея')} stroke="#2a4f73" stroke-width="1" class="zone">
        <title>Шея: {cnt('Шея')}</title>
      </rect>

      <!-- Плечи -->
      <rect x="6" y="38" width="78" height="14" rx="5" fill={fill('Плечи')} stroke="#2a4f73" stroke-width="1" class="zone">
        <title>Плечи: {cnt('Плечи')}</title>
      </rect>

      <!-- Тело -->
      <rect x="26" y="53" width="38" height="52" rx="4" fill={fill('Тело')} stroke="#2a4f73" stroke-width="1" class="zone">
        <title>Тело: {cnt('Тело')}</title>
      </rect>

      <!-- Предплечья -->
      <rect x="6" y="53" width="18" height="36" rx="4" fill={fill('Предплечья')} stroke="#2a4f73" stroke-width="1" class="zone">
        <title>Предплечья: {cnt('Предплечья')}</title>
      </rect>
      <rect x="66" y="53" width="18" height="36" rx="4" fill={fill('Предплечья')} stroke="#2a4f73" stroke-width="1" class="zone">
        <title>Предплечья: {cnt('Предплечья')}</title>
      </rect>

      <!-- Кисти -->
      <rect x="7" y="90" width="16" height="12" rx="4" fill={fill('Кисти')} stroke="#2a4f73" stroke-width="1" class="zone">
        <title>Кисти: {cnt('Кисти')}</title>
      </rect>
      <rect x="67" y="90" width="16" height="12" rx="4" fill={fill('Кисти')} stroke="#2a4f73" stroke-width="1" class="zone">
        <title>Кисти: {cnt('Кисти')}</title>
      </rect>

      <!-- Таз -->
      <rect x="28" y="106" width="34" height="16" rx="3" fill={fill('Таз')} stroke="#2a4f73" stroke-width="1" class="zone">
        <title>Таз: {cnt('Таз')}</title>
      </rect>

      <!-- Бедро -->
      <rect x="28" y="124" width="15" height="44" rx="4" fill={fill('Бедро')} stroke="#2a4f73" stroke-width="1" class="zone">
        <title>Бедро: {cnt('Бедро')}</title>
      </rect>
      <rect x="47" y="124" width="15" height="44" rx="4" fill={fill('Бедро')} stroke="#2a4f73" stroke-width="1" class="zone">
        <title>Бедро: {cnt('Бедро')}</title>
      </rect>

      <!-- Голень -->
      <rect x="28" y="170" width="15" height="44" rx="4" fill={fill('Голень')} stroke="#2a4f73" stroke-width="1" class="zone">
        <title>Голень: {cnt('Голень')}</title>
      </rect>
      <rect x="47" y="170" width="15" height="44" rx="4" fill={fill('Голень')} stroke="#2a4f73" stroke-width="1" class="zone">
        <title>Голень: {cnt('Голень')}</title>
      </rect>

      <!-- Labels on key zones -->
      <text x="45" y="18" text-anchor="middle" dominant-baseline="middle" class="lbl">{cnt('Голова')}</text>
      <text x="45" y="79" text-anchor="middle" dominant-baseline="middle" class="lbl">{cnt('Тело')}</text>
      <text x="45" y="114" text-anchor="middle" dominant-baseline="middle" class="lbl">{cnt('Таз')}</text>
      <text x="35" y="146" text-anchor="middle" dominant-baseline="middle" class="lbl">{cnt('Бедро')}</text>
      <text x="35" y="192" text-anchor="middle" dominant-baseline="middle" class="lbl">{cnt('Голень')}</text>

    </svg>

    <!-- Legend -->
    <div class="legend">
      {#each ZONES as zone}
        {@const c = cnt(zone)}
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
    width: 110px;
    flex-shrink: 0;
  }

  .zone {
    cursor: default;
    transition: filter 0.1s;
  }

  .zone:hover { filter: brightness(1.25); }

  .lbl {
    font-size: 8px;
    fill: #e8edf2;
    font-weight: 600;
    pointer-events: none;
  }

  .legend {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 5px;
    justify-content: center;
  }

  .legend-row {
    display: flex;
    align-items: center;
    gap: 7px;
  }

  .legend-swatch {
    width: 10px;
    height: 10px;
    border-radius: 2px;
    flex-shrink: 0;
    border: 1px solid rgba(219, 132, 31, 0.3);
  }

  .legend-zone {
    flex: 1;
    font-size: 0.74rem;
    color: #6b8aab;
  }

  .legend-count {
    font-size: 0.74rem;
    font-weight: 600;
    color: #a0b4c8;
    min-width: 14px;
    text-align: right;
  }
</style>
