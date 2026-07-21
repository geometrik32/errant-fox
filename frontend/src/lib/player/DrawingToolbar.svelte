<script lang="ts">
  import { onMount } from 'svelte';
  import type { VideoFighter } from '../api/types';

  export interface SlotPreset {
    hex: string;
    avatar_url?: string;
  }

  interface Props {
    color?: string;
    width?: number;
    userColor?: string | null;
    fighterA?: VideoFighter | null;
    fighterB?: VideoFighter | null;
    currentUser?: any;
    hasStrokes?: boolean;
    activeTool?: 'brush' | 'eraser';
    oncolorchange?: (color: string) => void;
    onwidthchange?: (width: number) => void;
    ontoolchange?: (tool: 'brush' | 'eraser') => void;
    onundo?: () => void;
    onclear?: () => void;
  }

  let {
    color = $bindable('#ef4444'),
    width = $bindable(4),
    userColor = null,
    fighterA = null,
    fighterB = null,
    currentUser = null,
    hasStrokes = false,
    activeTool = $bindable('brush'),
    oncolorchange,
    onwidthchange,
    ontoolchange,
    onundo,
    onclear,
  }: Props = $props();

  const BASE_COLORS = [
    { hex: '#ef4444', label: 'Красный' },
    { hex: '#f59e0b', label: 'Жёлтый' },
    { hex: '#10b981', label: 'Зелёный' },
    { hex: '#3b82f6', label: 'Синий' },
    { hex: '#ffffff', label: 'Белый' },
  ];

  // 3 Preset Color Slots
  let presets = $state<SlotPreset[]>([
    { hex: '#ef4444' },
    { hex: '#f59e0b' },
    { hex: '#10b981' },
  ]);
  let activeSlot = $state<number>(0);
  let openPopover = $state<boolean>(false);

  onMount(() => {
    try {
      const saved = localStorage.getItem('ef_drawing_color_presets_v2');
      if (saved) {
        const parsed = JSON.parse(saved);
        if (Array.isArray(parsed) && parsed.length >= 3) {
          presets = parsed.map((p: any) =>
            typeof p === 'string' ? { hex: p } : { hex: p.hex, avatar_url: p.avatar_url }
          );
        }
      } else {
        const myCol = userColor || currentUser?.color;
        const myAv = currentUser?.avatar_url;
        if (myCol) {
          presets[0] = { hex: myCol, avatar_url: myAv };
        }
      }
    } catch (e) {}
    color = presets[activeSlot]?.hex || '#ef4444';
  });

  function savePresets(newPresets: SlotPreset[]) {
    presets = newPresets;
    try {
      localStorage.setItem('ef_drawing_color_presets_v2', JSON.stringify(newPresets));
    } catch (e) {}
  }

  function selectSlot(index: number) {
    if (activeSlot === index) {
      openPopover = !openPopover;
    } else {
      activeSlot = index;
      openPopover = false;
    }
    color = presets[index]?.hex || '#ef4444';
    activeTool = 'brush';
    oncolorchange?.(color);
  }

  function chooseColorForCurrentSlot(hex: string, avatar_url?: string) {
    const next = [...presets];
    next[activeSlot] = { hex, avatar_url };
    savePresets(next);
    color = hex;
    activeTool = 'brush';
    openPopover = false;
    oncolorchange?.(hex);
  }

  function handleWidthInput(e: Event) {
    const val = Number((e.target as HTMLInputElement).value);
    width = val;
    onwidthchange?.(val);
  }

  function setTool(t: 'brush' | 'eraser') {
    activeTool = t;
    openPopover = false;
    ontoolchange?.(t);
  }

  let availableUsers = $derived.by(() => {
    const list: Array<{ id: string; display_name: string; color: string; avatar_url?: string }> = [];
    const seen = new Set<string>();

    const checkAndAdd = (u?: any, defaultLabel?: string) => {
      if (!u) return;
      const col = u.color || (typeof u === 'string' ? u : null);
      if (!col) return;
      const key = (u.id || defaultLabel || col).toString();
      if (seen.has(key)) return;
      seen.add(key);
      list.push({
        id: key,
        display_name: u.display_name || defaultLabel || 'Пользователь',
        color: col,
        avatar_url: u.avatar_url,
      });
    };

    checkAndAdd(currentUser, 'Мой цвет');
    checkAndAdd(fighterA, 'Боец А');
    checkAndAdd(fighterB, 'Боец Б');

    if (userColor && !seen.has('userColor') && (!currentUser || currentUser.color !== userColor)) {
      list.push({
        id: 'userColor',
        display_name: 'Мой цвет',
        color: userColor,
      });
    }

    return list;
  });

  function handlePopoverBlur(e: FocusEvent) {
    const target = e.relatedTarget as HTMLElement;
    if (!target || !target.closest('.color-popover-container')) {
      openPopover = false;
    }
  }
</script>

<!-- svelte-ignore a11y_interactive_supports_focus -->
<div
  class="drawing-toolbar"
  role="toolbar"
  aria-label="Панель инструментов рисования"
  tabindex="0"
  onclick={(e) => e.stopPropagation()}
  onkeydown={(e) => e.stopPropagation()}
>
  <!-- Tool selector (Brush vs Eraser) -->
  <div class="toolbar-section tools-section">
    <button
      type="button"
      class="btn-tool mode-btn"
      class:active={activeTool === 'brush'}
      onclick={() => setTool('brush')}
      title="Кисть (произвольное рисование)"
    >
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M12 20h9" />
        <path d="M16.5 3.5a2.121 2.121 0 0 1 3 3L7 19l-4 1 1-4L16.5 3.5z" />
      </svg>
    </button>

    <button
      type="button"
      class="btn-tool mode-btn"
      class:active={activeTool === 'eraser'}
      onclick={() => setTool('eraser')}
      title="Стёрка / Ластик (стирает штрихи)"
    >
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M20 20H7L3 16C2 15 2 13.5 3 12.5L13 2.5C14 1.5 15.5 1.5 16.5 2.5L21.5 7.5C22.5 8.5 22.5 10 21.5 11L14.5 18" />
        <path d="M18 11l-5-5" />
      </svg>
    </button>
  </div>

  <div class="toolbar-divider"></div>

  <!-- 3 Color Presets Slots + Popover Palette -->
  <div class="toolbar-section color-popover-container" onfocusout={handlePopoverBlur}>
    <div class="preset-slots">
      {#each presets as slot, i}
        <button
          type="button"
          class="slot-swatch"
          class:active={activeSlot === i && activeTool === 'brush'}
          style="background-color: {slot.hex}; border-color: {slot.hex};"
          title="Слот цвета {i + 1} (нажмите для выбора цвета)"
          aria-label="Слот цвета {i + 1}"
          onclick={() => selectSlot(i)}
        >
          {#if slot.avatar_url}
            <img src={slot.avatar_url} alt="" class="slot-avatar-img" onerror={(e) => { (e.target as HTMLElement).style.display = 'none'; }} />
          {/if}
          {#if activeSlot === i && openPopover}
            <span class="popover-caret">▲</span>
          {/if}
        </button>
      {/each}
    </div>

    <!-- Dropdown popover menu for selecting color of current slot -->
    {#if openPopover}
      <div class="color-popover" role="dialog" aria-label="Выбор цвета">
        <div class="popover-section-title">Базовые цвета</div>
        <div class="popover-row">
          {#each BASE_COLORS as c}
            <button
              type="button"
              class="popover-swatch"
              class:selected={presets[activeSlot]?.hex.toLowerCase() === c.hex.toLowerCase() && !presets[activeSlot]?.avatar_url}
              style="background-color: {c.hex};"
              title={c.label}
              aria-label={c.label}
              onclick={() => chooseColorForCurrentSlot(c.hex, undefined)}
            ></button>
          {/each}
        </div>

        {#if availableUsers.length > 0}
          <div class="popover-section-title margin-top">Цвета пользователей</div>
          <div class="popover-row">
            {#each availableUsers as u}
              <button
                type="button"
                class="popover-avatar-swatch"
                class:selected={presets[activeSlot]?.hex.toLowerCase() === u.color.toLowerCase() && presets[activeSlot]?.avatar_url === u.avatar_url}
                style="--border-color: {u.color};"
                title="{u.display_name} ({u.color})"
                aria-label="Цвет {u.display_name}"
                onclick={() => chooseColorForCurrentSlot(u.color, u.avatar_url)}
              >
                {#if u.avatar_url}
                  <img src={u.avatar_url} alt="" onerror={(e) => { (e.target as HTMLElement).style.display = 'none'; }} />
                {/if}
                <span class="avatar-initial">{u.display_name.slice(0, 1).toUpperCase()}</span>
              </button>
            {/each}
          </div>
        {/if}
      </div>
    {/if}
  </div>

  <div class="toolbar-divider"></div>

  <!-- Size slider (min 1px) -->
  <div class="toolbar-section size-section">
    <span
      class="size-icon"
      style="width: {Math.max(3, Math.min(18, width))}px; height: {Math.max(3, Math.min(18, width))}px; background: {activeTool === 'eraser' ? '#ef4444' : color}"
    ></span>
    <input
      type="range"
      class="size-slider"
      min="1"
      max="24"
      step="1"
      value={width}
      oninput={handleWidthInput}
      title="Размер {activeTool === 'eraser' ? 'стёрки' : 'кисти'}: {width}px"
      aria-label="Размер инструмента"
    />
    <span class="size-val">{width}px</span>
  </div>

  <div class="toolbar-divider"></div>

  <!-- Action buttons (Undo & Clear) -->
  <div class="toolbar-section actions-section">
    <button
      type="button"
      class="btn-tool"
      disabled={!hasStrokes}
      onclick={onundo}
      title="Отменить последний штрих (Ctrl+Z)"
    >
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M3 7v6h6" />
        <path d="M21 17a9 9 0 0 0-9-9 9 9 0 0 0-6 2.3L3 13" />
      </svg>
    </button>

    <button
      type="button"
      class="btn-tool danger"
      disabled={!hasStrokes}
      onclick={onclear}
      title="Очистить всё"
    >
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M3 6h18" />
        <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" />
      </svg>
    </button>
  </div>
</div>

<style>
  .drawing-toolbar {
    position: absolute;
    bottom: 16px;
    left: 50%;
    transform: translate3d(-50%, 0, 0);
    z-index: 30;
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 6px 14px;
    background: #0f172a;
    border: 1px solid rgba(255, 255, 255, 0.2);
    border-radius: 9999px;
    box-shadow: 0 10px 25px -5px rgba(0, 0, 0, 0.6), 0 8px 10px -6px rgba(0, 0, 0, 0.4);
    color: #f8fafc;
    user-select: none;
    max-width: 98%;
    flex-wrap: nowrap;
    white-space: nowrap;
    -webkit-font-smoothing: antialiased;
    -moz-osx-font-smoothing: grayscale;
  }

  .toolbar-section {
    display: flex;
    align-items: center;
    gap: 6px;
    flex-shrink: 0;
    position: relative;
  }

  .toolbar-divider {
    width: 1px;
    height: 18px;
    background: rgba(255, 255, 255, 0.18);
    flex-shrink: 0;
  }

  .preset-slots {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .slot-swatch {
    width: 22px;
    height: 22px;
    border-radius: 50%;
    border: 2px solid rgba(255, 255, 255, 0.35);
    cursor: pointer;
    padding: 0;
    transition: transform 0.15s ease, border-color 0.15s ease, box-shadow 0.15s ease;
    position: relative;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    overflow: hidden;
  }

  .slot-swatch:hover {
    transform: scale(1.15);
    border-color: rgba(255, 255, 255, 0.85);
  }

  .slot-swatch.active {
    transform: scale(1.2);
    border-color: #ffffff;
    box-shadow: 0 0 10px currentColor;
  }

  .slot-avatar-img {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    object-fit: cover;
    border-radius: 50%;
  }

  .popover-caret {
    font-size: 8px;
    color: #ffffff;
    text-shadow: 0 0 2px rgba(0, 0, 0, 0.9);
    position: absolute;
    top: -12px;
    z-index: 10;
  }

  .color-popover {
    position: absolute;
    bottom: 38px;
    left: 50%;
    transform: translate3d(-50%, 0, 0);
    background: #0f172a;
    border: 1px solid rgba(255, 255, 255, 0.25);
    border-radius: 12px;
    padding: 10px 14px;
    box-shadow: 0 16px 36px rgba(0, 0, 0, 0.85);
    z-index: 50;
    display: flex;
    flex-direction: column;
    gap: 8px;
    white-space: nowrap;
    -webkit-font-smoothing: antialiased;
    -moz-osx-font-smoothing: grayscale;
  }

  .popover-section-title {
    font-size: 0.68rem;
    font-weight: 700;
    color: #94a3b8;
    text-transform: uppercase;
    letter-spacing: 0.8px;
    line-height: 1;
  }

  .popover-section-title.margin-top {
    margin-top: 4px;
  }

  .popover-row {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: nowrap;
  }

  .popover-swatch {
    width: 24px;
    height: 24px;
    border-radius: 50%;
    border: 2px solid rgba(255, 255, 255, 0.35);
    cursor: pointer;
    padding: 0;
    flex-shrink: 0;
    transition: transform 0.15s ease, border-color 0.15s ease;
  }

  .popover-swatch:hover {
    transform: scale(1.18);
    border-color: #ffffff;
  }

  .popover-swatch.selected {
    border-color: #ffffff;
    box-shadow: 0 0 8px currentColor;
    transform: scale(1.15);
  }

  .popover-avatar-swatch {
    width: 24px;
    height: 24px;
    border-radius: 50%;
    border: 2px solid var(--border-color, #f59e0b);
    background: #1e293b;
    cursor: pointer;
    padding: 0;
    flex-shrink: 0;
    position: relative;
    overflow: hidden;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: transform 0.15s ease;
  }

  .popover-avatar-swatch:hover {
    transform: scale(1.18);
  }

  .popover-avatar-swatch.selected {
    box-shadow: 0 0 10px var(--border-color, #f59e0b);
    transform: scale(1.15);
  }

  .popover-avatar-swatch img {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .avatar-initial {
    font-size: 10px;
    font-weight: 700;
    color: #ffffff;
  }

  .size-section {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .size-icon {
    border-radius: 50%;
    display: inline-block;
    flex-shrink: 0;
    transition: all 0.1s ease;
  }

  .size-slider {
    width: 70px;
    accent-color: #f59e0b;
    cursor: pointer;
  }

  .size-val {
    font-size: 0.72rem;
    color: #94a3b8;
    min-width: 24px;
  }

  .btn-tool {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    padding: 4px 8px;
    font-size: 0.75rem;
    font-weight: 500;
    border-radius: 6px;
    border: 1px solid rgba(255, 255, 255, 0.15);
    background: rgba(255, 255, 255, 0.08);
    color: #f8fafc;
    cursor: pointer;
    transition: background 0.15s ease, opacity 0.15s ease, color 0.15s ease;
  }

  .btn-tool:hover:not(:disabled) {
    background: rgba(255, 255, 255, 0.2);
  }

  .btn-tool:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .btn-tool.mode-btn.active {
    background: #f59e0b;
    color: #0f172a;
    border-color: #f59e0b;
  }
</style>
