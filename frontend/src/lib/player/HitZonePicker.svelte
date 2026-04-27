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
    onchange?: (value: string) => void;
  }

  let { value, onchange }: Props = $props();

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
    return currentZone() === zone ? 'rgba(219,132,31,0.22)' : '#1a3a5c';
  }

  function stroke(zone: string) {
    return currentZone() === zone ? '#e8941f' : '#2a4f73';
  }
</script>

<div class="picker">
  <svg bind:this={svgEl} viewBox="0 0 90 222" xmlns="http://www.w3.org/2000/svg" class="svg" aria-label="Зона поражения">

    <!-- 1. Голова -->
    <rect x="29" y="1" width="32" height="28" rx="6"
      fill={fill('Голова')} stroke={stroke('Голова')} stroke-width="1"
      class="zone" role="button" tabindex="0"
      onclick={(e) => handleZoneClick(e, 'Голова')}
      onkeydown={(e) => e.key === 'Enter' && handleZoneClick(e as unknown as MouseEvent, 'Голова')}
      aria-label="Голова" aria-pressed={currentZone() === 'Голова'}
    ><title>Голова</title></rect>

    <!-- 2. Шея -->
    <rect x="36" y="30" width="18" height="10" rx="3"
      fill={fill('Шея')} stroke={stroke('Шея')} stroke-width="1"
      class="zone" role="button" tabindex="0"
      onclick={(e) => handleZoneClick(e, 'Шея')}
      onkeydown={(e) => e.key === 'Enter' && handleZoneClick(e as unknown as MouseEvent, 'Шея')}
      aria-label="Шея" aria-pressed={currentZone() === 'Шея'}
    ><title>Шея</title></rect>

    <!-- 3. Плечо пр. (viewer left = fighter right) -->
    <rect x="4" y="41" width="23" height="28" rx="4"
      fill={fill('Плечо пр.')} stroke={stroke('Плечо пр.')} stroke-width="1"
      class="zone" role="button" tabindex="0"
      onclick={(e) => handleZoneClick(e, 'Плечо пр.')}
      onkeydown={(e) => e.key === 'Enter' && handleZoneClick(e as unknown as MouseEvent, 'Плечо пр.')}
      aria-label="Плечо пр." aria-pressed={currentZone() === 'Плечо пр.'}
    ><title>Плечо пр.</title></rect>

    <!-- 4. Предплечье пр. -->
    <rect x="4" y="70" width="18" height="30" rx="4"
      fill={fill('Предплечье пр.')} stroke={stroke('Предплечье пр.')} stroke-width="1"
      class="zone" role="button" tabindex="0"
      onclick={(e) => handleZoneClick(e, 'Предплечье пр.')}
      onkeydown={(e) => e.key === 'Enter' && handleZoneClick(e as unknown as MouseEvent, 'Предплечье пр.')}
      aria-label="Предплечье пр." aria-pressed={currentZone() === 'Предплечье пр.'}
    ><title>Предплечье пр.</title></rect>

    <!-- 5. Кисть пр. -->
    <rect x="5" y="101" width="15" height="12" rx="3"
      fill={fill('Кисть пр.')} stroke={stroke('Кисть пр.')} stroke-width="1"
      class="zone" role="button" tabindex="0"
      onclick={(e) => handleZoneClick(e, 'Кисть пр.')}
      onkeydown={(e) => e.key === 'Enter' && handleZoneClick(e as unknown as MouseEvent, 'Кисть пр.')}
      aria-label="Кисть пр." aria-pressed={currentZone() === 'Кисть пр.'}
    ><title>Кисть пр.</title></rect>

    <!-- 9. Тело -->
    <rect x="28" y="41" width="34" height="54" rx="4"
      fill={fill('Тело')} stroke={stroke('Тело')} stroke-width="1"
      class="zone" role="button" tabindex="0"
      onclick={(e) => handleZoneClick(e, 'Тело')}
      onkeydown={(e) => e.key === 'Enter' && handleZoneClick(e as unknown as MouseEvent, 'Тело')}
      aria-label="Тело" aria-pressed={currentZone() === 'Тело'}
    ><title>Тело</title></rect>

    <!-- 10. Таз -->
    <rect x="28" y="96" width="34" height="18" rx="3"
      fill={fill('Таз')} stroke={stroke('Таз')} stroke-width="1"
      class="zone" role="button" tabindex="0"
      onclick={(e) => handleZoneClick(e, 'Таз')}
      onkeydown={(e) => e.key === 'Enter' && handleZoneClick(e as unknown as MouseEvent, 'Таз')}
      aria-label="Таз" aria-pressed={currentZone() === 'Таз'}
    ><title>Таз</title></rect>

    <!-- 6. Плечо лев. (viewer right = fighter left) -->
    <rect x="63" y="41" width="23" height="28" rx="4"
      fill={fill('Плечо лев.')} stroke={stroke('Плечо лев.')} stroke-width="1"
      class="zone" role="button" tabindex="0"
      onclick={(e) => handleZoneClick(e, 'Плечо лев.')}
      onkeydown={(e) => e.key === 'Enter' && handleZoneClick(e as unknown as MouseEvent, 'Плечо лев.')}
      aria-label="Плечо лев." aria-pressed={currentZone() === 'Плечо лев.'}
    ><title>Плечо лев.</title></rect>

    <!-- 7. Предплечье лев. -->
    <rect x="68" y="70" width="18" height="30" rx="4"
      fill={fill('Предплечье лев.')} stroke={stroke('Предплечье лев.')} stroke-width="1"
      class="zone" role="button" tabindex="0"
      onclick={(e) => handleZoneClick(e, 'Предплечье лев.')}
      onkeydown={(e) => e.key === 'Enter' && handleZoneClick(e as unknown as MouseEvent, 'Предплечье лев.')}
      aria-label="Предплечье лев." aria-pressed={currentZone() === 'Предплечье лев.'}
    ><title>Предплечье лев.</title></rect>

    <!-- 8. Кисть лев. -->
    <rect x="70" y="101" width="15" height="12" rx="3"
      fill={fill('Кисть лев.')} stroke={stroke('Кисть лев.')} stroke-width="1"
      class="zone" role="button" tabindex="0"
      onclick={(e) => handleZoneClick(e, 'Кисть лев.')}
      onkeydown={(e) => e.key === 'Enter' && handleZoneClick(e as unknown as MouseEvent, 'Кисть лев.')}
      aria-label="Кисть лев." aria-pressed={currentZone() === 'Кисть лев.'}
    ><title>Кисть лев.</title></rect>

    <!-- 11. Бедро пр. -->
    <rect x="27" y="115" width="16" height="42" rx="4"
      fill={fill('Бедро пр.')} stroke={stroke('Бедро пр.')} stroke-width="1"
      class="zone" role="button" tabindex="0"
      onclick={(e) => handleZoneClick(e, 'Бедро пр.')}
      onkeydown={(e) => e.key === 'Enter' && handleZoneClick(e as unknown as MouseEvent, 'Бедро пр.')}
      aria-label="Бедро пр." aria-pressed={currentZone() === 'Бедро пр.'}
    ><title>Бедро пр.</title></rect>

    <!-- 12. Голень пр. -->
    <rect x="27" y="158" width="16" height="40" rx="4"
      fill={fill('Голень пр.')} stroke={stroke('Голень пр.')} stroke-width="1"
      class="zone" role="button" tabindex="0"
      onclick={(e) => handleZoneClick(e, 'Голень пр.')}
      onkeydown={(e) => e.key === 'Enter' && handleZoneClick(e as unknown as MouseEvent, 'Голень пр.')}
      aria-label="Голень пр." aria-pressed={currentZone() === 'Голень пр.'}
    ><title>Голень пр.</title></rect>

    <!-- 13. Стопа пр. -->
    <rect x="27" y="199" width="16" height="12" rx="3"
      fill={fill('Стопа пр.')} stroke={stroke('Стопа пр.')} stroke-width="1"
      class="zone" role="button" tabindex="0"
      onclick={(e) => handleZoneClick(e, 'Стопа пр.')}
      onkeydown={(e) => e.key === 'Enter' && handleZoneClick(e as unknown as MouseEvent, 'Стопа пр.')}
      aria-label="Стопа пр." aria-pressed={currentZone() === 'Стопа пр.'}
    ><title>Стопа пр.</title></rect>

    <!-- 14. Бедро лев. -->
    <rect x="47" y="115" width="16" height="42" rx="4"
      fill={fill('Бедро лев.')} stroke={stroke('Бедро лев.')} stroke-width="1"
      class="zone" role="button" tabindex="0"
      onclick={(e) => handleZoneClick(e, 'Бедро лев.')}
      onkeydown={(e) => e.key === 'Enter' && handleZoneClick(e as unknown as MouseEvent, 'Бедро лев.')}
      aria-label="Бедро лев." aria-pressed={currentZone() === 'Бедро лев.'}
    ><title>Бедро лев.</title></rect>

    <!-- 15. Голень лев. -->
    <rect x="47" y="158" width="16" height="40" rx="4"
      fill={fill('Голень лев.')} stroke={stroke('Голень лев.')} stroke-width="1"
      class="zone" role="button" tabindex="0"
      onclick={(e) => handleZoneClick(e, 'Голень лев.')}
      onkeydown={(e) => e.key === 'Enter' && handleZoneClick(e as unknown as MouseEvent, 'Голень лев.')}
      aria-label="Голень лев." aria-pressed={currentZone() === 'Голень лев.'}
    ><title>Голень лев.</title></rect>

    <!-- 16. Стопа лев. -->
    <rect x="47" y="199" width="16" height="12" rx="3"
      fill={fill('Стопа лев.')} stroke={stroke('Стопа лев.')} stroke-width="1"
      class="zone" role="button" tabindex="0"
      onclick={(e) => handleZoneClick(e, 'Стопа лев.')}
      onkeydown={(e) => e.key === 'Enter' && handleZoneClick(e as unknown as MouseEvent, 'Стопа лев.')}
      aria-label="Стопа лев." aria-pressed={currentZone() === 'Стопа лев.'}
    ><title>Стопа лев.</title></rect>

    <!-- Hit dot at click position -->
    {#if currentZone()}
      {@const dot = parseCoords()}
      {#if dot}
        <circle cx={dot.x * 90} cy={dot.y * 222} r="3.5" fill="#e02020" pointer-events="none" />
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

  .zone {
    cursor: pointer;
    transition: filter 0.1s;
    outline: none;
  }

  .zone:hover {
    filter: brightness(1.4);
  }
</style>
