<script lang="ts">
  import type { FighterBout } from '../api/types';

  interface Props {
    bouts: FighterBout[];
    type: 'dealt' | 'received';
    selectedZone?: string;
    onzoneclick?: (zone: string) => void;
  }

  let { bouts, type, selectedZone = '', onzoneclick }: Props = $props();

  // Central body zones go in the header row
  const HEAD_ZONES   = ['Голова', 'Шея'] as const;
  const TORSO_ZONES  = ['Тело', 'Таз']  as const;

  // Limb zones go in side legends (right-side arm/leg first)
  const LEFT_LIMB_ZONES  = ['Плечо пр.', 'Предплечье пр.', 'Кисть пр.', 'Бедро пр.', 'Голень пр.', 'Стопа пр.'] as const;
  const RIGHT_LIMB_ZONES = ['Плечо лев.', 'Предплечье лев.', 'Кисть лев.', 'Бедро лев.', 'Голень лев.', 'Стопа лев.'] as const;

  const ALL_ZONES = [...HEAD_ZONES, ...TORSO_ZONES, ...LEFT_LIMB_ZONES, ...RIGHT_LIMB_ZONES];

  function parseZone(s: string | null): string {
    if (!s) return '';
    return s.split(':')[0];
  }

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
    const map = new Map<string, number>(ALL_ZONES.map(z => [z, 0]));
    for (const b of bouts) {
      const raw    = type === 'dealt' ? b.my_hit_zone : b.opponent_hit_zone;
      const result = type === 'dealt' ? b.my_result   : b.opponent_result;
      if (result === 'hit') {
        const zone = parseZone(raw);
        if (map.has(zone)) map.set(zone, (map.get(zone) ?? 0) + 1);
      }
    }
    return map;
  });

  let totalHits = $derived([...counts.values()].reduce((a, b) => a + b, 0));

  let dots = $derived.by(() => {
    const result: Array<{ x: number; y: number; zone: string }> = [];
    for (const b of bouts) {
      const raw       = type === 'dealt' ? b.my_hit_zone : b.opponent_hit_zone;
      const hitResult = type === 'dealt' ? b.my_result   : b.opponent_result;
      if (hitResult !== 'hit') continue;
      const coords = parseCoords(raw);
      if (coords) result.push({ ...coords, zone: parseZone(raw) });
    }
    return result;
  });

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

  <!-- Title + filter badge -->
  <div class="card-header">
    <h3 class="card-title">{type === 'dealt' ? 'Нанесённый урон' : 'Полученный урон'}</h3>
    {#if selectedZone}
      <div class="zone-filter-badge">
        {selectedZone}
        <button class="clear-filter" onclick={() => onzoneclick?.('')}>✕</button>
      </div>
    {/if}
  </div>

  <!-- Header zones row: head/neck | total | body/pelvis -->
  <div class="header-zones">
    <div class="header-group header-group--left">
      {#each HEAD_ZONES as zone}
        {@const c = cnt(zone)}
        <!-- svelte-ignore a11y_interactive_supports_focus -->
        <div class="header-zone" class:selected={selectedZone === zone}
          role="button" tabindex="0"
          onclick={() => handleZoneClick(zone)}
          onkeydown={(e) => e.key === 'Enter' && handleZoneClick(zone)}>
          <span class="hz-name">{zone}</span>
          <span class="hz-count">{c}</span>
        </div>
      {/each}
    </div>
    <div class="total-badge">
      <span class="total-num">{totalHits}</span>
      <span class="total-label">всего</span>
    </div>
    <div class="header-group header-group--right">
      {#each TORSO_ZONES as zone}
        {@const c = cnt(zone)}
        <!-- svelte-ignore a11y_interactive_supports_focus -->
        <div class="header-zone" class:selected={selectedZone === zone}
          role="button" tabindex="0"
          onclick={() => handleZoneClick(zone)}
          onkeydown={(e) => e.key === 'Enter' && handleZoneClick(zone)}>
          <span class="hz-count">{c}</span>
          <span class="hz-name">{zone}</span>
        </div>
      {/each}
    </div>
  </div>

  <!-- Silhouette row: left legend | SVG | right legend -->
  <div class="silhouette-wrap">

    <!-- Left Legend: right-side limbs -->
    <div class="legend left-legend">
      {#each LEFT_LIMB_ZONES as zone}
        {@const c = cnt(zone)}
        <!-- svelte-ignore a11y_interactive_supports_focus -->
        <div class="legend-row" class:selected={selectedZone === zone}
          role="button" tabindex="0"
          onclick={() => handleZoneClick(zone)}
          onkeydown={(e) => e.key === 'Enter' && handleZoneClick(zone)}>
          <span class="legend-zone">{zone}</span>
          <span class="legend-count">{c}</span>
        </div>
      {/each}
    </div>

    <!-- SVG silhouette -->
    <svg viewBox="0 0 350 1055" xmlns="http://www.w3.org/2000/svg" class="svg">

      <!-- Голова -->
      <rect x="112.5" y="0" width="125" height="125" rx="20"
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
        onclick={() => handleZoneClick('Стопа лев.')}><title>Стопа лев.: {cnt('Стопа лев.')}</title></rect>

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

    <!-- Right Legend: left-side limbs -->
    <div class="legend right-legend">
      {#each RIGHT_LIMB_ZONES as zone}
        {@const c = cnt(zone)}
        <!-- svelte-ignore a11y_interactive_supports_focus -->
        <div class="legend-row" class:selected={selectedZone === zone}
          role="button" tabindex="0"
          onclick={() => handleZoneClick(zone)}
          onkeydown={(e) => e.key === 'Enter' && handleZoneClick(zone)}>
          <span class="legend-count">{c}</span>
          <span class="legend-zone">{zone}</span>
        </div>
      {/each}
    </div>

  </div>
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

  .card-header {
    display: flex;
    justify-content: center;
    align-items: center;
    margin-bottom: 16px;
    position: relative;
  }

  .card-title {
    font-size: 1rem;
    font-weight: 700;
    color: var(--text-primary);
    margin: 0;
    text-align: center;
  }

  /* Header zones: head/neck | total | body/pelvis */
  .header-zones {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    margin-bottom: 16px;
    padding: 10px 14px;
    background: var(--surface-hover);
    border-radius: var(--radius-sm);
    border: 1px solid var(--border-color);
  }

  .header-group {
    display: flex;
    gap: 6px;
    flex: 1;
  }

  .header-group--left { justify-content: flex-start; }
  .header-group--right { justify-content: flex-end; }

  .header-zone {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 2px;
    padding: 6px 10px;
    border-radius: var(--radius-sm);
    cursor: pointer;
    transition: background 0.1s;
    border: 1px solid transparent;
  }

  .header-zone:hover {
    background: var(--surface-solid);
    border-color: var(--border-color);
  }

  .header-zone.selected {
    background: rgba(219, 132, 31, 0.15);
    border-color: rgba(219, 132, 31, 0.4);
  }

  .hz-name {
    font-size: 0.7rem;
    color: var(--text-secondary);
    white-space: nowrap;
  }

  .hz-count {
    font-size: 1rem;
    font-weight: 700;
    color: var(--text-primary);
    line-height: 1;
  }

  .total-badge {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 2px;
    flex-shrink: 0;
  }

  .total-num {
    font-size: 1.6rem;
    font-weight: 700;
    color: var(--accent-yellow);
    line-height: 1;
  }

  .total-label {
    font-size: 0.65rem;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.06em;
  }

  /* Silhouette main row */
  .silhouette-wrap {
    display: flex;
    align-items: stretch;
    justify-content: center;
    gap: 12px;
  }

  .svg {
    width: 120px;
    flex-shrink: 0;
  }

  .zone {
    cursor: pointer;
    transition: filter 0.1s;
  }

  .zone:hover { filter: brightness(1.3); }

  .legend {
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    align-items: stretch;
    flex: 1;
    min-width: 90px;
  }

  .legend-row {
    display: flex;
    align-items: center;
    gap: 6px;
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

  .left-legend .legend-row { justify-content: flex-end; }
  .right-legend .legend-row { justify-content: flex-start; }

  .legend-zone {
    font-size: 0.75rem;
    color: var(--text-secondary);
    white-space: nowrap;
  }

  .legend-count {
    font-size: 0.82rem;
    font-weight: 600;
    color: var(--text-primary);
    min-width: 16px;
    text-align: center;
  }

  .zone-filter-badge {
    position: absolute;
    right: 0;
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 3px 10px;
    background: rgba(219, 132, 31, 0.12);
    border: 1px solid rgba(219, 132, 31, 0.3);
    border-radius: var(--radius-sm);
    font-size: 0.75rem;
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
