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

  // SVG viewBox is 0 0 350 1055, dot coords are 0-1 fractions of that
  const VW = 350;
  const VH = 1055;

  function fill(zone: string): string {
    if (selectedZone && selectedZone === zone) return 'rgba(219,132,31,0.4)';
    return 'var(--surface-solid)';
  }

  function strokeColor(zone: string): string {
    return selectedZone === zone ? 'var(--accent-yellow)' : 'var(--text-secondary)';
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

    <svg viewBox="0 0 350 1055" xmlns="http://www.w3.org/2000/svg" class="svg">

      <!-- Голова -->
      <rect x="113" y="0" width="125" height="125" rx="20"
        fill={fill('Голова')} stroke={strokeColor('Голова')} stroke-width="2" class="zone"
        onclick={() => handleZoneClick('Голова')}><title>Голова: {cnt('Голова')}</title></rect>

      <!-- Шея -->
      <rect x="130" y="130" width="90" height="45" rx="20"
        fill={fill('Шея')} stroke={strokeColor('Шея')} stroke-width="2" class="zone"
        onclick={() => handleZoneClick('Шея')}><title>Шея: {cnt('Шея')}</title></rect>

      <!-- Тело -->
      <rect x="80" y="180" width="190" height="290" rx="20"
        fill={fill('Тело')} stroke={strokeColor('Тело')} stroke-width="2" class="zone"
        onclick={() => handleZoneClick('Тело')}><title>Тело: {cnt('Тело')}</title></rect>

      <!-- Таз -->
      <rect x="80" y="475" width="190" height="100" rx="20"
        fill={fill('Таз')} stroke={strokeColor('Таз')} stroke-width="2" class="zone"
        onclick={() => handleZoneClick('Таз')}><title>Таз: {cnt('Таз')}</title></rect>

      <!-- Плечо пр. -->
      <rect x="0" y="180" width="70" height="195" rx="20"
        fill={fill('Плечо пр.')} stroke={strokeColor('Плечо пр.')} stroke-width="2" class="zone"
        onclick={() => handleZoneClick('Плечо пр.')}><title>Плечо пр.: {cnt('Плечо пр.')}</title></rect>

      <!-- Предплечье пр. -->
      <rect x="0" y="380" width="70" height="195" rx="20"
        fill={fill('Предплечье пр.')} stroke={strokeColor('Предплечье пр.')} stroke-width="2" class="zone"
        onclick={() => handleZoneClick('Предплечье пр.')}><title>Предплечье пр.: {cnt('Предплечье пр.')}</title></rect>

      <!-- Кисть пр. -->
      <rect x="0" y="580" width="70" height="70" rx="20"
        fill={fill('Кисть пр.')} stroke={strokeColor('Кисть пр.')} stroke-width="2" class="zone"
        onclick={() => handleZoneClick('Кисть пр.')}><title>Кисть пр.: {cnt('Кисть пр.')}</title></rect>

      <!-- Плечо лев. -->
      <rect x="280" y="180" width="70" height="195" rx="20"
        fill={fill('Плечо лев.')} stroke={strokeColor('Плечо лев.')} stroke-width="2" class="zone"
        onclick={() => handleZoneClick('Плечо лев.')}><title>Плечо лев.: {cnt('Плечо лев.')}</title></rect>

      <!-- Предплечье лев. -->
      <rect x="280" y="380" width="70" height="195" rx="20"
        fill={fill('Предплечье лев.')} stroke={strokeColor('Предплечье лев.')} stroke-width="2" class="zone"
        onclick={() => handleZoneClick('Предплечье лев.')}><title>Предплечье лев.: {cnt('Предплечье лев.')}</title></rect>

      <!-- Кисть лев. -->
      <rect x="280" y="580" width="70" height="70" rx="20"
        fill={fill('Кисть лев.')} stroke={strokeColor('Кисть лев.')} stroke-width="2" class="zone"
        onclick={() => handleZoneClick('Кисть лев.')}><title>Кисть лев.: {cnt('Кисть лев.')}</title></rect>

      <!-- Бедро пр. -->
      <rect x="80" y="580" width="90" height="210" rx="20"
        fill={fill('Бедро пр.')} stroke={strokeColor('Бедро пр.')} stroke-width="2" class="zone"
        onclick={() => handleZoneClick('Бедро пр.')}><title>Бедро пр.: {cnt('Бедро пр.')}</title></rect>

      <!-- Голень пр. -->
      <rect x="80" y="795" width="90" height="210" rx="20"
        fill={fill('Голень пр.')} stroke={strokeColor('Голень пр.')} stroke-width="2" class="zone"
        onclick={() => handleZoneClick('Голень пр.')}><title>Голень пр.: {cnt('Голень пр.')}</title></rect>

      <!-- Стопа пр. -->
      <rect x="80" y="1010" width="90" height="45" rx="20"
        fill={fill('Стопа пр.')} stroke={strokeColor('Стопа пр.')} stroke-width="2" class="zone"
        onclick={() => handleZoneClick('Стопа пр.')}><title>Стопа пр.: {cnt('Стопа пр.')}</title></rect>

      <!-- Бедро лев. -->
      <rect x="180" y="580" width="90" height="210" rx="20"
        fill={fill('Бедро лев.')} stroke={strokeColor('Бедро лев.')} stroke-width="2" class="zone"
        onclick={() => handleZoneClick('Бедро лев.')}><title>Бедро лев.: {cnt('Бедро лев.')}</title></rect>

      <!-- Голень лев. -->
      <rect x="180" y="795" width="90" height="210" rx="20"
        fill={fill('Голень лев.')} stroke={strokeColor('Голень лев.')} stroke-width="2" class="zone"
        onclick={() => handleZoneClick('Голень лев.')}><title>Голень лев.: {cnt('Голень лев.')}</title></rect>

      <!-- Стопа лев. -->
      <rect x="180" y="1010" width="90" height="45" rx="20"
        fill={fill('Стопа лев.')} stroke={strokeColor('Стопа лев.')} stroke-width="2" class="zone"
        onclick={() => handleZoneClick('Стопа лев.')}><title>Стопа лев.: {cnt('Стопа лев')}</title></rect>

      <!-- Hit dots at stored coordinates -->
      {#each dots as dot}
        <circle
          cx={dot.x * VW}
          cy={dot.y * VH}
          r="8"
          fill="#e02020"
          opacity="0.6"
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
    background: var(--surface);
    backdrop-filter: var(--glass-blur);
    -webkit-backdrop-filter: var(--glass-blur);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-md);
    box-shadow: var(--shadow-sm);
    padding: 20px;
  }

  .silhouette-title {
    font-size: 0.8rem;
    font-weight: 600;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--text-secondary);
    margin-bottom: 16px;
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
    background: var(--surface-hover);
  }

  .legend-row.selected {
    background: rgba(219, 132, 31, 0.15);
  }

  .legend-swatch {
    width: 12px;
    height: 12px;
    border-radius: 3px;
    flex-shrink: 0;
    border: 1px solid var(--text-secondary);
  }

  .legend-zone {
    flex: 1;
    font-size: 0.8rem;
    color: var(--text-secondary);
  }

  .legend-count {
    font-size: 0.85rem;
    font-weight: 600;
    color: var(--text-primary);
    min-width: 14px;
    text-align: right;
  }

  .zone-filter-badge {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-top: 12px;
    padding: 6px 12px;
    background: rgba(219, 132, 31, 0.12);
    border: 1px solid rgba(219, 132, 31, 0.3);
    border-radius: var(--radius-sm);
    font-size: 0.8rem;
    color: var(--accent-yellow);
  }

  .clear-filter {
    background: none;
    border: none;
    color: var(--accent-yellow);
    cursor: pointer;
    font-size: 0.85rem;
    padding: 0;
    line-height: 1;
  }
</style>
