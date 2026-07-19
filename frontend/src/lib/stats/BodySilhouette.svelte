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

  const ARM_ZONES_LEFT = ['Плечо пр.', 'Предплечье пр.', 'Кисть пр.'] as const;
  const LEG_ZONES_LEFT = ['Бедро пр.', 'Голень пр.', 'Стопа пр.'] as const;

  const ARM_ZONES_RIGHT = ['Плечо лев.', 'Предплечье лев.', 'Кисть лев.'] as const;
  const LEG_ZONES_RIGHT = ['Бедро лев.', 'Голень лев.', 'Стопа лев.'] as const;

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
    return selectedZone === zone ? 'var(--accent-yellow)' : 'rgba(255, 255, 255, 0.2)';
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
    <h3 class="card-title">{type === 'dealt' ? 'Нанесенный урон' : 'Полученный урон'}</h3>
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
    </div>
    <div class="header-group header-group--right">
      {#each TORSO_ZONES as zone}
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
  </div>

  <!-- Silhouette row: left legend | SVG | right legend -->
  <div class="silhouette-wrap">

    <!-- Left Legend: right-side limbs -->
    <div class="legend left-legend">
      <div class="legend-group">
        {#each ARM_ZONES_LEFT as zone}
          {@const c = cnt(zone)}
          <div class="legend-row" class:selected={selectedZone === zone}
            role="button" tabindex="0"
            onclick={() => handleZoneClick(zone)}
            onkeydown={(e) => e.key === 'Enter' && handleZoneClick(zone)}>
            <span class="legend-count">{c}</span>
            <span class="legend-zone">{zone}</span>
          </div>
        {/each}
      </div>
      <div class="legend-group">
        {#each LEG_ZONES_LEFT as zone}
          {@const c = cnt(zone)}
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

    <!-- SVG silhouette -->
    <svg viewBox="0 0 350 1055" xmlns="http://www.w3.org/2000/svg" class="svg">

      <!-- 1. Голова -->
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <rect x="113" y="0" width="125" height="125" rx="20" role="button" tabindex="0"
        fill={fill('Голова')} stroke={strokeColor('Голова')} stroke-width="2" class="zone"
        onclick={() => handleZoneClick('Голова')}
        onkeydown={(e) => e.key === 'Enter' && handleZoneClick('Голова')}><title>Голова: {cnt('Голова')}</title></rect>

      <!-- 2. Шея -->
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <rect x="130" y="130" width="90" height="45" rx="20" role="button" tabindex="0"
        fill={fill('Шея')} stroke={strokeColor('Шея')} stroke-width="2" class="zone"
        onclick={() => handleZoneClick('Шея')}
        onkeydown={(e) => e.key === 'Enter' && handleZoneClick('Шея')}><title>Шея: {cnt('Шея')}</title></rect>

      <!-- 3. Плечо пр. -->
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <rect x="0" y="180" width="70" height="195" rx="20" role="button" tabindex="0"
        fill={fill('Плечо пр.')} stroke={strokeColor('Плечо пр.')} stroke-width="2" class="zone"
        onclick={() => handleZoneClick('Плечо пр.')}
        onkeydown={(e) => e.key === 'Enter' && handleZoneClick('Плечо пр.')}><title>Плечо пр.: {cnt('Плечо пр.')}</title></rect>

      <!-- 4. Предплечье пр. -->
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <rect x="0" y="380" width="70" height="195" rx="20" role="button" tabindex="0"
        fill={fill('Предплечье пр.')} stroke={strokeColor('Предплечье пр.')} stroke-width="2" class="zone"
        onclick={() => handleZoneClick('Предплечье пр.')}
        onkeydown={(e) => e.key === 'Enter' && handleZoneClick('Предплечье пр.')}><title>Предплечье пр.: {cnt('Предплечье пр.')}</title></rect>

      <!-- 5. Кисть пр. -->
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <rect x="0" y="580" width="70" height="70" rx="20" role="button" tabindex="0"
        fill={fill('Кисть пр.')} stroke={strokeColor('Кисть пр.')} stroke-width="2" class="zone"
        onclick={() => handleZoneClick('Кисть пр.')}
        onkeydown={(e) => e.key === 'Enter' && handleZoneClick('Кисть пр.')}><title>Кисть пр.: {cnt('Кисть пр.')}</title></rect>

      <!-- 9. Тело -->
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <rect x="80" y="180" width="190" height="290" rx="20" role="button" tabindex="0"
        fill={fill('Тело')} stroke={strokeColor('Тело')} stroke-width="2" class="zone"
        onclick={() => handleZoneClick('Тело')}
        onkeydown={(e) => e.key === 'Enter' && handleZoneClick('Тело')}><title>Тело: {cnt('Тело')}</title></rect>

      <!-- 10. Таз -->
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <rect x="80" y="475" width="190" height="100" rx="20" role="button" tabindex="0"
        fill={fill('Таз')} stroke={strokeColor('Таз')} stroke-width="2" class="zone"
        onclick={() => handleZoneClick('Таз')}
        onkeydown={(e) => e.key === 'Enter' && handleZoneClick('Таз')}><title>Таз: {cnt('Таз')}</title></rect>

      <!-- 6. Плечо лев. -->
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <rect x="280" y="180" width="70" height="195" rx="20" role="button" tabindex="0"
        fill={fill('Плечо лев.')} stroke={strokeColor('Плечо лев.')} stroke-width="2" class="zone"
        onclick={() => handleZoneClick('Плечо лев.')}
        onkeydown={(e) => e.key === 'Enter' && handleZoneClick('Плечо лев.')}><title>Плечо лев.: {cnt('Плечо лев.')}</title></rect>

      <!-- 7. Предплечье лев. -->
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <rect x="280" y="380" width="70" height="195" rx="20" role="button" tabindex="0"
        fill={fill('Предплечье лев.')} stroke={strokeColor('Предплечье лев.')} stroke-width="2" class="zone"
        onclick={() => handleZoneClick('Предплечье лев.')}
        onkeydown={(e) => e.key === 'Enter' && handleZoneClick('Предплечье лев.')}><title>Предплечье лев.: {cnt('Предплечье лев.')}</title></rect>

      <!-- 8. Кисть лев. -->
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <rect x="280" y="580" width="70" height="70" rx="20" role="button" tabindex="0"
        fill={fill('Кисть лев.')} stroke={strokeColor('Кисть лев.')} stroke-width="2" class="zone"
        onclick={() => handleZoneClick('Кисть лев.')}
        onkeydown={(e) => e.key === 'Enter' && handleZoneClick('Кисть лев.')}><title>Кисть лев.: {cnt('Кисть лев.')}</title></rect>

      <!-- 11. Бедро пр. -->
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <rect x="80" y="580" width="90" height="210" rx="20" role="button" tabindex="0"
        fill={fill('Бедро пр.')} stroke={strokeColor('Бедро пр.')} stroke-width="2" class="zone"
        onclick={() => handleZoneClick('Бедро пр.')}
        onkeydown={(e) => e.key === 'Enter' && handleZoneClick('Бедро пр.')}><title>Бедро пр.: {cnt('Бедро пр.')}</title></rect>

      <!-- 12. Голень пр. -->
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <rect x="80" y="795" width="90" height="210" rx="20" role="button" tabindex="0"
        fill={fill('Голень пр.')} stroke={strokeColor('Голень пр.')} stroke-width="2" class="zone"
        onclick={() => handleZoneClick('Голень пр.')}
        onkeydown={(e) => e.key === 'Enter' && handleZoneClick('Голень пр.')}><title>Голень пр.: {cnt('Голень пр.')}</title></rect>

      <!-- 13. Стопа пр. -->
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <rect x="80" y="1010" width="90" height="45" rx="20" role="button" tabindex="0"
        fill={fill('Стопа пр.')} stroke={strokeColor('Стопа пр.')} stroke-width="2" class="zone"
        onclick={() => handleZoneClick('Стопа пр.')}
        onkeydown={(e) => e.key === 'Enter' && handleZoneClick('Стопа пр.')}><title>Стопа пр.: {cnt('Стопа пр.')}</title></rect>

      <!-- 14. Бедро лев. -->
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <rect x="180" y="580" width="90" height="210" rx="20" role="button" tabindex="0"
        fill={fill('Бедро лев.')} stroke={strokeColor('Бедро лев.')} stroke-width="2" class="zone"
        onclick={() => handleZoneClick('Бедро лев.')}
        onkeydown={(e) => e.key === 'Enter' && handleZoneClick('Бедро лев.')}><title>Бедро лев.: {cnt('Бедро лев.')}</title></rect>

      <!-- 15. Голень лев. -->
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <rect x="180" y="795" width="90" height="210" rx="20" role="button" tabindex="0"
        fill={fill('Голень лев.')} stroke={strokeColor('Голень лев.')} stroke-width="2" class="zone"
        onclick={() => handleZoneClick('Голень лев.')}
        onkeydown={(e) => e.key === 'Enter' && handleZoneClick('Голень лев.')}><title>Голень лев.: {cnt('Голень лев.')}</title></rect>

      <!-- 16. Стопа лев. -->
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <rect x="180" y="1010" width="90" height="45" rx="20" role="button" tabindex="0"
        fill={fill('Стопа лев.')} stroke={strokeColor('Стопа лев.')} stroke-width="2" class="zone"
        onclick={() => handleZoneClick('Стопа лев.')}
        onkeydown={(e) => e.key === 'Enter' && handleZoneClick('Стопа лев.')}><title>Стопа лев.: {cnt('Стопа лев.')}</title></rect>

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
      <div class="legend-group">
        {#each ARM_ZONES_RIGHT as zone}
          {@const c = cnt(zone)}
          <div class="legend-row" class:selected={selectedZone === zone}
            role="button" tabindex="0"
            onclick={() => handleZoneClick(zone)}
            onkeydown={(e) => e.key === 'Enter' && handleZoneClick(zone)}>
            <span class="legend-zone">{zone}</span>
            <span class="legend-count">{c}</span>
          </div>
        {/each}
      </div>
      <div class="legend-group">
        {#each LEG_ZONES_RIGHT as zone}
          {@const c = cnt(zone)}
          <div class="legend-row" class:selected={selectedZone === zone}
            role="button" tabindex="0"
            onclick={() => handleZoneClick(zone)}
            onkeydown={(e) => e.key === 'Enter' && handleZoneClick(zone)}>
            <span class="legend-zone">{zone}</span>
            <span class="legend-count">{c}</span>
          </div>
        {/each}
      </div>
    </div>

  </div>
</div>

<style>
  .silhouette-card {
    background: var(--surface);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-md);
    padding: 24px;
    height: 100%;
    box-sizing: border-box;
    display: flex;
    flex-direction: column;
  }

  .card-header {
    display: flex;
    justify-content: center;
    align-items: center;
    margin-bottom: 20px;
    position: relative;
  }

  .card-title {
    font-size: 0.9rem;
    font-weight: 500;
    color: var(--text-primary);
    margin: 0;
    text-align: center;
    text-transform: none;
    letter-spacing: 0.02em;
  }

  /* Header zones: head/neck | total | body/pelvis */
  .header-zones {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    margin-bottom: 30px;
    padding: 15px 0;
    background: #1e293b;
    border-radius: 12px;
    border: 1px solid rgba(255, 255, 255, 0.05);
    box-shadow: inset 0 1px 1px rgba(255, 255, 255, 0.05);
  }

  .header-group {
    display: flex;
    gap: 6px;
    flex: 1;
  }

  .header-group--left { justify-content: flex-start; padding-left: 40px; }
  .header-group--right { justify-content: flex-end; padding-right: 40px; }

  .header-zone {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 2px;
    padding: 6px 0;
    width: 80px;
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
    font-size: 0.85rem;
    color: var(--text-secondary);
    white-space: nowrap;
    font-weight: 500;
    text-transform: none;
  }

  .hz-count {
    font-size: 1.2rem;
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
    font-size: 2rem;
    font-weight: 700;
    color: var(--accent-yellow);
    line-height: 1;
  }

  /* Silhouette main row */
  .silhouette-wrap {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 40px;
    flex: 1;
    min-height: 0;
  }

  .svg {
    width: 220px;
    height: auto;
    max-height: 650px;
    margin: 0 auto;
    overflow: visible;
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
    min-width: 140px;
    height: 100%;
    padding: 30px 0 60px 0;
  }

  .left-legend { margin-left: 62px; }
  .right-legend { margin-right: 62px; }

  .legend-group {
    display: flex;
    flex-direction: column;
    gap: 20px;
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

  .left-legend .legend-row { justify-content: flex-start; }
  .right-legend .legend-row { justify-content: flex-end; }

  .legend-zone {
    font-size: 0.85rem;
    color: var(--text-secondary);
    white-space: nowrap;
  }

  .legend-count {
    font-size: 1.2rem;
    font-weight: 700;
    color: var(--text-primary);
    width: 36px;
    flex-shrink: 0;
  }

  .left-legend .legend-count { text-align: left; }
  .right-legend .legend-count { text-align: right; }

</style>
