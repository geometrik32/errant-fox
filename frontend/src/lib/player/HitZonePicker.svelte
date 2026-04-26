<script lang="ts">
  export const HIT_ZONES = [
    'Голова', 'Шея', 'Плечи', 'Предплечья', 'Кисти',
    'Тело', 'Таз', 'Бедро', 'Голень',
  ] as const;

  export type HitZone = typeof HIT_ZONES[number];

  interface Props {
    value: string;
    onchange?: (zone: string) => void;
  }

  let { value, onchange }: Props = $props();

  function select(zone: string) {
    onchange?.(value === zone ? '' : zone);
  }

  function fill(zone: string) {
    return value === zone ? '#DB841F' : '#1a3a5c';
  }

  function stroke(zone: string) {
    return value === zone ? '#e8941f' : '#2a4f73';
  }
</script>

<!--
  Compact interactive silhouette. ViewBox 90 × 230.
  Zones (top → bottom):
    Голова (head)          — circle
    Шея (neck)             — rect center
    Плечи (shoulders)      — wide rect
    Предплечья (forearms)  — two side rects
    Кисти (hands)          — two small rects
    Тело (torso)           — center rect
    Таз (pelvis)           — short center rect
    Бедро (thighs)         — two rects
    Голень (shins)         — two rects
-->
<div class="picker">
  <svg viewBox="0 0 90 230" xmlns="http://www.w3.org/2000/svg" class="svg" aria-label="Зона поражения">

    <!-- Голова -->
    <circle cx="45" cy="14" r="12"
      fill={fill('Голова')} stroke={stroke('Голова')} stroke-width="1"
      class="zone" role="button" tabindex="0"
      onclick={() => select('Голова')}
      onkeydown={(e) => e.key === 'Enter' && select('Голова')}
      aria-label="Голова" aria-pressed={value === 'Голова'}
    >
      <title>Голова</title>
    </circle>

    <!-- Шея -->
    <rect x="38" y="27" width="14" height="10" rx="3"
      fill={fill('Шея')} stroke={stroke('Шея')} stroke-width="1"
      class="zone" role="button" tabindex="0"
      onclick={() => select('Шея')}
      onkeydown={(e) => e.key === 'Enter' && select('Шея')}
      aria-label="Шея" aria-pressed={value === 'Шея'}
    >
      <title>Шея</title>
    </rect>

    <!-- Плечи (wide band) -->
    <rect x="6" y="38" width="78" height="14" rx="5"
      fill={fill('Плечи')} stroke={stroke('Плечи')} stroke-width="1"
      class="zone" role="button" tabindex="0"
      onclick={() => select('Плечи')}
      onkeydown={(e) => e.key === 'Enter' && select('Плечи')}
      aria-label="Плечи" aria-pressed={value === 'Плечи'}
    >
      <title>Плечи</title>
    </rect>

    <!-- Тело -->
    <rect x="26" y="53" width="38" height="52" rx="4"
      fill={fill('Тело')} stroke={stroke('Тело')} stroke-width="1"
      class="zone" role="button" tabindex="0"
      onclick={() => select('Тело')}
      onkeydown={(e) => e.key === 'Enter' && select('Тело')}
      aria-label="Тело" aria-pressed={value === 'Тело'}
    >
      <title>Тело</title>
    </rect>

    <!-- Предплечья — левое (визуально левое = правая сторона фигуры) -->
    <rect x="6" y="53" width="18" height="36" rx="4"
      fill={fill('Предплечья')} stroke={stroke('Предплечья')} stroke-width="1"
      class="zone" role="button" tabindex="0"
      onclick={() => select('Предплечья')}
      onkeydown={(e) => e.key === 'Enter' && select('Предплечья')}
      aria-label="Предплечья" aria-pressed={value === 'Предплечья'}
    >
      <title>Предплечья</title>
    </rect>

    <!-- Предплечья — правое -->
    <rect x="66" y="53" width="18" height="36" rx="4"
      fill={fill('Предплечья')} stroke={stroke('Предплечья')} stroke-width="1"
      class="zone" role="button" tabindex="0"
      onclick={() => select('Предплечья')}
      onkeydown={(e) => e.key === 'Enter' && select('Предплечья')}
      aria-label="Предплечья" aria-pressed={value === 'Предплечья'}
    >
      <title>Предплечья</title>
    </rect>

    <!-- Кисти — левая -->
    <rect x="7" y="90" width="16" height="12" rx="4"
      fill={fill('Кисти')} stroke={stroke('Кисти')} stroke-width="1"
      class="zone" role="button" tabindex="0"
      onclick={() => select('Кисти')}
      onkeydown={(e) => e.key === 'Enter' && select('Кисти')}
      aria-label="Кисти" aria-pressed={value === 'Кисти'}
    >
      <title>Кисти</title>
    </rect>

    <!-- Кисти — правая -->
    <rect x="67" y="90" width="16" height="12" rx="4"
      fill={fill('Кисти')} stroke={stroke('Кисти')} stroke-width="1"
      class="zone" role="button" tabindex="0"
      onclick={() => select('Кисти')}
      onkeydown={(e) => e.key === 'Enter' && select('Кисти')}
      aria-label="Кисти" aria-pressed={value === 'Кисти'}
    >
      <title>Кисти</title>
    </rect>

    <!-- Таз -->
    <rect x="28" y="106" width="34" height="16" rx="3"
      fill={fill('Таз')} stroke={stroke('Таз')} stroke-width="1"
      class="zone" role="button" tabindex="0"
      onclick={() => select('Таз')}
      onkeydown={(e) => e.key === 'Enter' && select('Таз')}
      aria-label="Таз" aria-pressed={value === 'Таз'}
    >
      <title>Таз</title>
    </rect>

    <!-- Бедро — левое -->
    <rect x="28" y="124" width="15" height="44" rx="4"
      fill={fill('Бедро')} stroke={stroke('Бедро')} stroke-width="1"
      class="zone" role="button" tabindex="0"
      onclick={() => select('Бедро')}
      onkeydown={(e) => e.key === 'Enter' && select('Бедро')}
      aria-label="Бедро" aria-pressed={value === 'Бедро'}
    >
      <title>Бедро</title>
    </rect>

    <!-- Бедро — правое -->
    <rect x="47" y="124" width="15" height="44" rx="4"
      fill={fill('Бедро')} stroke={stroke('Бедро')} stroke-width="1"
      class="zone" role="button" tabindex="0"
      onclick={() => select('Бедро')}
      onkeydown={(e) => e.key === 'Enter' && select('Бедро')}
      aria-label="Бедро" aria-pressed={value === 'Бедро'}
    >
      <title>Бедро</title>
    </rect>

    <!-- Голень — левая -->
    <rect x="28" y="170" width="15" height="44" rx="4"
      fill={fill('Голень')} stroke={stroke('Голень')} stroke-width="1"
      class="zone" role="button" tabindex="0"
      onclick={() => select('Голень')}
      onkeydown={(e) => e.key === 'Enter' && select('Голень')}
      aria-label="Голень" aria-pressed={value === 'Голень'}
    >
      <title>Голень</title>
    </rect>

    <!-- Голень — правая -->
    <rect x="47" y="170" width="15" height="44" rx="4"
      fill={fill('Голень')} stroke={stroke('Голень')} stroke-width="1"
      class="zone" role="button" tabindex="0"
      onclick={() => select('Голень')}
      onkeydown={(e) => e.key === 'Enter' && select('Голень')}
      aria-label="Голень" aria-pressed={value === 'Голень'}
    >
      <title>Голень</title>
    </rect>

    <!-- Labels -->
    {#if value}
      <text x="45" y="221" text-anchor="middle" class="selected-label">{value}</text>
    {/if}

  </svg>
</div>

<style>
  .picker {
    display: flex;
    justify-content: center;
  }

  .svg {
    width: 72px;
    flex-shrink: 0;
    overflow: visible;
  }

  .zone {
    cursor: pointer;
    transition: filter 0.1s;
    outline: none;
  }

  .zone:hover {
    filter: brightness(1.35);
  }

  .selected-label {
    font-size: 7px;
    fill: #DB841F;
    font-weight: 600;
  }
</style>
