<script lang="ts">
  import type { FighterBout } from '../api/types';

  interface Props {
    bouts: FighterBout[];
    type: 'dealt' | 'received';
    selectedZone?: string;
    onzoneclick?: (zone: string) => void;
  }

  let { bouts, type, selectedZone = '', onzoneclick }: Props = $props();

  const ZONES = [
    'Голова', 'Шея',
    'Плечо пр.', 'Предплечье пр.', 'Кисть пр.',
    'Плечо лев.', 'Предплечье лев.', 'Кисть лев.',
    'Тело', 'Таз',
    'Бедро пр.', 'Голень пр.', 'Стопа пр.',
    'Бедро лев.', 'Голень лев.', 'Стопа лев.',
  ] as const;

  // Parse "ZoneName" or "ZoneName:x:y" — returns zone name only
  function parseZone(s: string | null): string {
    if (!s) return '';
    return s.split(':')[0];
  }

  // Parse coordinates from "ZoneName:x:y" — returns null if no coords stored
  function parseCoords(s: string | null): { x: number; y: number } | null {
    if (!s) return null;
    const parts = s.split(':');
    if (parts.length < 3) return null;
    const x = parseFloat(parts[1]);
    const y = parseFloat(parts[2]);
    if (isNaN(x) || isNaN(y)) return null;
    return { x, y };
  }

  let counts = $derived.by(() => {
    const map = new Map<string, number>(ZONES.map(z => [z, 0]));
    for (const b of bouts) {
      const raw = type === 'dealt' ? b.my_hit_zone : b.opponent_hit_zone;
      const result = type === 'dealt' ? b.my_result : b.opponent_result;
      if (result === 'hit') {
        const zone = parseZone(raw);
        if (map.has(zone)) map.set(zone, (map.get(zone) ?? 0) + 1);
      }
    }
    return map;
  });

  // Hit dots (for bouts that have coordinate data)
  let dots = $derived.by(() => {
    const result: Array<{ x: number; y: number; zone: string }> = [];
    for (const b of bouts) {
      const raw = type === 'dealt' ? b.my_hit_zone : b.opponent_hit_zone;
      const hitResult = type === 'dealt' ? b.my_result : b.opponent_result;
      if (hitResult !== 'hit') continue;
      const coords = parseCoords(raw);
      if (coords) result.push({ ...coords, zone: parseZone(raw) });
    }
    return result;
  });

  let maxCount = $derived(Math.max(...[...counts.values()], 1));

  // SVG viewBox is 0 0 90 222, dot coords are 0-1 fractions of that
  const VW = 90;
  const VH = 222;

  function opacity(zone: string): number {
    const c = counts.get(zone) ?? 0;
    return c === 0 ? 0.08 : c / maxCount * 0.75 + 0.15;
  }

  function fill(zone: string): string {
    if (selectedZone && selectedZone === zone) return 'rgba(219,132,31,0.6)';
    return `rgba(219, 132, 31, ${opacity(zone).toFixed(2)})`;
  }

  function strokeColor(zone: string): string {
    return selectedZone === zone ? '#e8941f' : '#2a4f73';
  }

  function cnt(zone: string): number {
    return counts.get(zone) ?? 0;
  }

  function handleZoneClick(zone: string) {
    onzoneclick?.(selectedZone === zone ? '' : zone);
  }
</script>

<div class="silhouette-card">
  <div class="silhouette-title">
    {type === 'dealt' ? 'Нанесённый урон' : 'Полученный урон'}
  </div>
  <div class="silhouette-wrap">

    <svg viewBox="0 0 90 222" xmlns="http://www.w3.org/2000/svg" class="svg">

      <!-- Голова -->
      <rect x="29" y="1" width="32" height="28" rx="6"
        fill={fill('Голова')} stroke={strokeColor('Голова')} stroke-width="1" class="zone"
        onclick={() => handleZoneClick('Голова')}><title>Голова: {cnt('Голова')}</title></rect>

      <!-- Шея -->
      <rect x="36" y="30" width="18" height="10" rx="3"
        fill={fill('Шея')} stroke={strokeColor('Шея')} stroke-width="1" class="zone"
        onclick={() => handleZoneClick('Шея')}><title>Шея: {cnt('Шея')}</title></rect>

      <!-- Плечо пр. -->
      <rect x="4" y="41" width="23" height="28" rx="4"
        fill={fill('Плечо пр.')} stroke={strokeColor('Плечо пр.')} stroke-width="1" class="zone"
        onclick={() => handleZoneClick('Плечо пр.')}><title>Плечо пр.: {cnt('Плечо пр.')}</title></rect>

      <!-- Предплечье пр. -->
      <rect x="4" y="70" width="18" height="30" rx="4"
        fill={fill('Предплечье пр.')} stroke={strokeColor('Предплечье пр.')} stroke-width="1" class="zone"
        onclick={() => handleZoneClick('Предплечье пр.')}><title>Предплечье пр.: {cnt('Предплечье пр.')}</title></rect>

      <!-- Кисть пр. -->
      <rect x="5" y="101" width="15" height="12" rx="3"
        fill={fill('Кисть пр.')} stroke={strokeColor('Кисть пр.')} stroke-width="1" class="zone"
        onclick={() => handleZoneClick('Кисть пр.')}><title>Кисть пр.: {cnt('Кисть пр.')}</title></rect>

      <!-- Тело -->
      <rect x="28" y="41" width="34" height="54" rx="4"
        fill={fill('Тело')} stroke={strokeColor('Тело')} stroke-width="1" class="zone"
        onclick={() => handleZoneClick('Тело')}><title>Тело: {cnt('Тело')}</title></rect>

      <!-- Таз -->
      <rect x="28" y="96" width="34" height="18" rx="3"
        fill={fill('Таз')} stroke={strokeColor('Таз')} stroke-width="1" class="zone"
        onclick={() => handleZoneClick('Таз')}><title>Таз: {cnt('Таз')}</title></rect>

      <!-- Плечо лев. -->
      <rect x="63" y="41" width="23" height="28" rx="4"
        fill={fill('Плечо лев.')} stroke={strokeColor('Плечо лев.')} stroke-width="1" class="zone"
        onclick={() => handleZoneClick('Плечо лев.')}><title>Плечо лев.: {cnt('Плечо лев.')}</title></rect>

      <!-- Предплечье лев. -->
      <rect x="68" y="70" width="18" height="30" rx="4"
        fill={fill('Предплечье лев.')} stroke={strokeColor('Предплечье лев.')} stroke-width="1" class="zone"
        onclick={() => handleZoneClick('Предплечье лев.')}><title>Предплечье лев.: {cnt('Предплечье лев.')}</title></rect>

      <!-- Кисть лев. -->
      <rect x="70" y="101" width="15" height="12" rx="3"
        fill={fill('Кисть лев.')} stroke={strokeColor('Кисть лев.')} stroke-width="1" class="zone"
        onclick={() => handleZoneClick('Кисть лев.')}><title>Кисть лев.: {cnt('Кисть лев.')}</title></rect>

      <!-- Бедро пр. -->
      <rect x="27" y="115" width="16" height="42" rx="4"
        fill={fill('Бедро пр.')} stroke={strokeColor('Бедро пр.')} stroke-width="1" class="zone"
        onclick={() => handleZoneClick('Бедро пр.')}><title>Бедро пр.: {cnt('Бедро пр.')}</title></rect>

      <!-- Голень пр. -->
      <rect x="27" y="158" width="16" height="40" rx="4"
        fill={fill('Голень пр.')} stroke={strokeColor('Голень пр.')} stroke-width="1" class="zone"
        onclick={() => handleZoneClick('Голень пр.')}><title>Голень пр.: {cnt('Голень пр.')}</title></rect>

      <!-- Стопа пр. -->
      <rect x="27" y="199" width="16" height="12" rx="3"
        fill={fill('Стопа пр.')} stroke={strokeColor('Стопа пр.')} stroke-width="1" class="zone"
        onclick={() => handleZoneClick('Стопа пр.')}><title>Стопа пр.: {cnt('Стопа пр.')}</title></rect>

      <!-- Бедро лев. -->
      <rect x="47" y="115" width="16" height="42" rx="4"
        fill={fill('Бедро лев.')} stroke={strokeColor('Бедро лев.')} stroke-width="1" class="zone"
        onclick={() => handleZoneClick('Бедро лев.')}><title>Бедро лев.: {cnt('Бедро лев.')}</title></rect>

      <!-- Голень лев. -->
      <rect x="47" y="158" width="16" height="40" rx="4"
        fill={fill('Голень лев.')} stroke={strokeColor('Голень лев.')} stroke-width="1" class="zone"
        onclick={() => handleZoneClick('Голень лев.')}><title>Голень лев.: {cnt('Голень лев.')}</title></rect>

      <!-- Стопа лев. -->
      <rect x="47" y="199" width="16" height="12" rx="3"
        fill={fill('Стопа лев.')} stroke={strokeColor('Стопа лев.')} stroke-width="1" class="zone"
        onclick={() => handleZoneClick('Стопа лев.')}><title>Стопа лев.: {cnt('Стопа лев.')}</title></rect>

      <!-- Hit dots at stored coordinates -->
      {#each dots as dot}
        <circle
          cx={dot.x * VW}
          cy={dot.y * VH}
          r="2.5"
          fill="#DB841F"
          opacity="0.7"
          pointer-events="none"
        />
      {/each}

    </svg>

    <!-- Legend -->
    <div class="legend">
      {#each ZONES as zone}
        {@const c = cnt(zone)}
        <!-- svelte-ignore a11y_interactive_supports_focus -->
        <div
          class="legend-row"
          class:selected={selectedZone === zone}
          role="button"
          tabindex="0"
          onclick={() => handleZoneClick(zone)}
          onkeydown={(e) => e.key === 'Enter' && handleZoneClick(zone)}
        >
          <div class="legend-swatch" style:background={fill(zone)}></div>
          <span class="legend-zone">{zone}</span>
          <span class="legend-count">{c}</span>
        </div>
      {/each}
    </div>

  </div>
  {#if selectedZone}
    <div class="zone-filter-badge">
      Фильтр: {selectedZone}
      <button class="clear-filter" onclick={() => onzoneclick?.('')}>✕</button>
    </div>
  {/if}
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
    cursor: pointer;
    transition: filter 0.1s;
  }

  .zone:hover { filter: brightness(1.3); }

  .legend {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 3px;
    justify-content: center;
  }

  .legend-row {
    display: flex;
    align-items: center;
    gap: 7px;
    padding: 2px 5px;
    border-radius: 4px;
    cursor: pointer;
    transition: background 0.1s;
    outline: none;
  }

  .legend-row:hover {
    background: #1a3050;
  }

  .legend-row.selected {
    background: rgba(219, 132, 31, 0.15);
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
    font-size: 0.72rem;
    color: #6b8aab;
  }

  .legend-count {
    font-size: 0.72rem;
    font-weight: 600;
    color: #a0b4c8;
    min-width: 14px;
    text-align: right;
  }

  .zone-filter-badge {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-top: 10px;
    padding: 4px 8px;
    background: rgba(219, 132, 31, 0.12);
    border: 1px solid rgba(219, 132, 31, 0.3);
    border-radius: 5px;
    font-size: 0.75rem;
    color: #DB841F;
  }

  .clear-filter {
    background: none;
    border: none;
    color: #DB841F;
    cursor: pointer;
    font-size: 0.75rem;
    padding: 0;
    line-height: 1;
  }
</style>
