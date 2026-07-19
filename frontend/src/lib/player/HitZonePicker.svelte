<script module lang="ts">
  export const HIT_ZONES = [
    'Голова', 'Шея',
    'Плечо пр.', 'Предплечье пр.', 'Кисть пр.',
    'Плечо лев.', 'Предплечье лев.', 'Кисть лев.',
    'Тело', 'Таз',
    'Бедро пр.', 'Голень пр.', 'Стопа пр.',
    'Бедро лев.', 'Голень лев.', 'Стопа лев.',
  ] as const;

  export type HitZone = typeof HIT_ZONES[number];
</script>

<script lang="ts">
  interface Props {
    value: string;
    readonly?: boolean;
    onchange?: (value: string) => void;
  }

  let { value, readonly = false, onchange }: Props = $props();

  let svgEl: SVGElement;

  // value format: "ZoneName" or "ZoneName:x:y" or ""
  function currentZone(): string {
    return value.split(':')[0];
  }

  function parseCoords(): { x: number; y: number } | null {
    const parts = value.split(':');
    if (parts.length < 3) return null;
    const x = parseFloat(parts[1]);
    const y = parseFloat(parts[2]);
    if (isNaN(x) || isNaN(y)) return null;
    return { x, y };
  }

  function handleZoneClick(e: MouseEvent, zone: string) {
    if (readonly) return;
    if (currentZone() === zone) {
      onchange?.('');
      return;
    }
    const rect = svgEl.getBoundingClientRect();
    const x = ((e.clientX - rect.left) / rect.width).toFixed(3);
    const y = ((e.clientY - rect.top) / rect.height).toFixed(3);
    onchange?.(`${zone}:${x}:${y}`);
  }

  function fill(zone: string) {
    return currentZone() === zone ? 'rgba(219,132,31,0.3)' : 'var(--surface-solid)';
  }

  function stroke(zone: string) {
    return currentZone() === zone ? 'var(--accent-yellow)' : 'var(--text-secondary)';
  }
</script>

<div class="picker">
  <svg bind:this={svgEl} viewBox="0 0 350 1055" xmlns="http://www.w3.org/2000/svg" class="svg" class:readonly={readonly} aria-label="Зона поражения">

    <!-- 1. Голова -->
    <rect x="113" y="0" width="125" height="125" rx="20"
      fill={fill('Голова')} stroke={stroke('Голова')} stroke-width="2"
      class="zone" role="button" tabindex="0"
      onclick={(e) => handleZoneClick(e, 'Голова')}
      onkeydown={(e) => e.key === 'Enter' && handleZoneClick(e as unknown as MouseEvent, 'Голова')}
      aria-label="Голова" aria-pressed={currentZone() === 'Голова'}
    ><title>Голова</title></rect>

    <!-- 2. Шея -->
    <rect x="130" y="130" width="90" height="45" rx="20"
      fill={fill('Шея')} stroke={stroke('Шея')} stroke-width="2"
      class="zone" role="button" tabindex="0"
      onclick={(e) => handleZoneClick(e, 'Шея')}
      onkeydown={(e) => e.key === 'Enter' && handleZoneClick(e as unknown as MouseEvent, 'Шея')}
      aria-label="Шея" aria-pressed={currentZone() === 'Шея'}
    ><title>Шея</title></rect>

    <!-- 3. Плечо пр. (viewer left = fighter right) -->
    <rect x="0" y="180" width="70" height="195" rx="20"
      fill={fill('Плечо пр.')} stroke={stroke('Плечо пр.')} stroke-width="2"
      class="zone" role="button" tabindex="0"
      onclick={(e) => handleZoneClick(e, 'Плечо пр.')}
      onkeydown={(e) => e.key === 'Enter' && handleZoneClick(e as unknown as MouseEvent, 'Плечо пр.')}
      aria-label="Плечо пр." aria-pressed={currentZone() === 'Плечо пр.'}
    ><title>Плечо пр.</title></rect>

    <!-- 4. Предплечье пр. -->
    <rect x="0" y="380" width="70" height="195" rx="20"
      fill={fill('Предплечье пр.')} stroke={stroke('Предплечье пр.')} stroke-width="2"
      class="zone" role="button" tabindex="0"
      onclick={(e) => handleZoneClick(e, 'Предплечье пр.')}
      onkeydown={(e) => e.key === 'Enter' && handleZoneClick(e as unknown as MouseEvent, 'Предплечье пр.')}
      aria-label="Предплечье пр." aria-pressed={currentZone() === 'Предплечье пр.'}
    ><title>Предплечье пр.</title></rect>

    <!-- 5. Кисть пр. -->
    <rect x="0" y="580" width="70" height="70" rx="20"
      fill={fill('Кисть пр.')} stroke={stroke('Кисть пр.')} stroke-width="2"
      class="zone" role="button" tabindex="0"
      onclick={(e) => handleZoneClick(e, 'Кисть пр.')}
      onkeydown={(e) => e.key === 'Enter' && handleZoneClick(e as unknown as MouseEvent, 'Кисть пр.')}
      aria-label="Кисть пр." aria-pressed={currentZone() === 'Кисть пр.'}
    ><title>Кисть пр.</title></rect>

    <!-- 9. Тело -->
    <rect x="80" y="180" width="190" height="290" rx="20"
      fill={fill('Тело')} stroke={stroke('Тело')} stroke-width="2"
      class="zone" role="button" tabindex="0"
      onclick={(e) => handleZoneClick(e, 'Тело')}
      onkeydown={(e) => e.key === 'Enter' && handleZoneClick(e as unknown as MouseEvent, 'Тело')}
      aria-label="Тело" aria-pressed={currentZone() === 'Тело'}
    ><title>Тело</title></rect>

    <!-- 10. Таз -->
    <rect x="80" y="475" width="190" height="100" rx="20"
      fill={fill('Таз')} stroke={stroke('Таз')} stroke-width="2"
      class="zone" role="button" tabindex="0"
      onclick={(e) => handleZoneClick(e, 'Таз')}
      onkeydown={(e) => e.key === 'Enter' && handleZoneClick(e as unknown as MouseEvent, 'Таз')}
      aria-label="Таз" aria-pressed={currentZone() === 'Таз'}
    ><title>Таз</title></rect>

    <!-- 6. Плечо лев. (viewer right = fighter left) -->
    <rect x="280" y="180" width="70" height="195" rx="20"
      fill={fill('Плечо лев.')} stroke={stroke('Плечо лев.')} stroke-width="2"
      class="zone" role="button" tabindex="0"
      onclick={(e) => handleZoneClick(e, 'Плечо лев.')}
      onkeydown={(e) => e.key === 'Enter' && handleZoneClick(e as unknown as MouseEvent, 'Плечо лев.')}
      aria-label="Плечо лев." aria-pressed={currentZone() === 'Плечо лев.'}
    ><title>Плечо лев.</title></rect>

    <!-- 7. Предплечье лев. -->
    <rect x="280" y="380" width="70" height="195" rx="20"
      fill={fill('Предплечье лев.')} stroke={stroke('Предплечье лев.')} stroke-width="2"
      class="zone" role="button" tabindex="0"
      onclick={(e) => handleZoneClick(e, 'Предплечье лев.')}
      onkeydown={(e) => e.key === 'Enter' && handleZoneClick(e as unknown as MouseEvent, 'Предплечье лев.')}
      aria-label="Предплечье лев." aria-pressed={currentZone() === 'Предплечье лев.'}
    ><title>Предплечье лев.</title></rect>

    <!-- 8. Кисть лев. -->
    <rect x="280" y="580" width="70" height="70" rx="20"
      fill={fill('Кисть лев.')} stroke={stroke('Кисть лев.')} stroke-width="2"
      class="zone" role="button" tabindex="0"
      onclick={(e) => handleZoneClick(e, 'Кисть лев.')}
      onkeydown={(e) => e.key === 'Enter' && handleZoneClick(e as unknown as MouseEvent, 'Кисть лев.')}
      aria-label="Кисть лев." aria-pressed={currentZone() === 'Кисть лев.'}
    ><title>Кисть лев.</title></rect>

    <!-- 11. Бедро пр. -->
    <rect x="80" y="580" width="90" height="210" rx="20"
      fill={fill('Бедро пр.')} stroke={stroke('Бедро пр.')} stroke-width="2"
      class="zone" role="button" tabindex="0"
      onclick={(e) => handleZoneClick(e, 'Бедро пр.')}
      onkeydown={(e) => e.key === 'Enter' && handleZoneClick(e as unknown as MouseEvent, 'Бедро пр.')}
      aria-label="Бедро пр." aria-pressed={currentZone() === 'Бедро пр.'}
    ><title>Бедро пр.</title></rect>

    <!-- 12. Голень пр. -->
    <rect x="80" y="795" width="90" height="210" rx="20"
      fill={fill('Голень пр.')} stroke={stroke('Голень пр.')} stroke-width="2"
      class="zone" role="button" tabindex="0"
      onclick={(e) => handleZoneClick(e, 'Голень пр.')}
      onkeydown={(e) => e.key === 'Enter' && handleZoneClick(e as unknown as MouseEvent, 'Голень пр.')}
      aria-label="Голень пр." aria-pressed={currentZone() === 'Голень пр.'}
    ><title>Голень пр.</title></rect>

    <!-- 13. Стопа пр. -->
    <rect x="80" y="1010" width="90" height="45" rx="20"
      fill={fill('Стопа пр.')} stroke={stroke('Стопа пр.')} stroke-width="2"
      class="zone" role="button" tabindex="0"
      onclick={(e) => handleZoneClick(e, 'Стопа пр.')}
      onkeydown={(e) => e.key === 'Enter' && handleZoneClick(e as unknown as MouseEvent, 'Стопа пр.')}
      aria-label="Стопа пр." aria-pressed={currentZone() === 'Стопа пр.'}
    ><title>Стопа пр.</title></rect>

    <!-- 14. Бедро лев. -->
    <rect x="180" y="580" width="90" height="210" rx="20"
      fill={fill('Бедро лев.')} stroke={stroke('Бедро лев.')} stroke-width="2"
      class="zone" role="button" tabindex="0"
      onclick={(e) => handleZoneClick(e, 'Бедро лев.')}
      onkeydown={(e) => e.key === 'Enter' && handleZoneClick(e as unknown as MouseEvent, 'Бедро лев.')}
      aria-label="Бедро лев." aria-pressed={currentZone() === 'Бедро лев.'}
    ><title>Бедро лев.</title></rect>

    <!-- 15. Голень лев. -->
    <rect x="180" y="795" width="90" height="210" rx="20"
      fill={fill('Голень лев.')} stroke={stroke('Голень лев.')} stroke-width="2"
      class="zone" role="button" tabindex="0"
      onclick={(e) => handleZoneClick(e, 'Голень лев.')}
      onkeydown={(e) => e.key === 'Enter' && handleZoneClick(e as unknown as MouseEvent, 'Голень лев.')}
      aria-label="Голень лев." aria-pressed={currentZone() === 'Голень лев.'}
    ><title>Голень лев.</title></rect>

    <!-- 16. Стопа лев. -->
    <rect x="180" y="1010" width="90" height="45" rx="20"
      fill={fill('Стопа лев.')} stroke={stroke('Стопа лев.')} stroke-width="2"
      class="zone" role="button" tabindex="0"
      onclick={(e) => handleZoneClick(e, 'Стопа лев.')}
      onkeydown={(e) => e.key === 'Enter' && handleZoneClick(e as unknown as MouseEvent, 'Стопа лев.')}
      aria-label="Стопа лев." aria-pressed={currentZone() === 'Стопа лев.'}
    ><title>Стопа лев.</title></rect>

    <!-- Hit dot at click position -->
    {#if currentZone()}
      {@const dot = parseCoords()}
      {#if dot}
        <circle cx={dot.x * 350} cy={dot.y * 1055} r="12" fill="#e02020" pointer-events="none" />
      {/if}
    {/if}

  </svg>
</div>

<style>
  .picker {
    display: flex;
    justify-content: center;
  }

  .svg {
    width: 120px;
    flex-shrink: 0;
    overflow: visible;
  }

  .svg.readonly .zone {
    cursor: default;
    pointer-events: none;
  }

  .svg.readonly .zone:hover {
    filter: none;
  }

  .zone {
    cursor: pointer;
    transition: filter 0.1s;
    outline: none;
  }

  .zone:hover {
    filter: brightness(1.4);
  }
</style>
