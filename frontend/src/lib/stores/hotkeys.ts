import { writable } from 'svelte/store';

export interface HotkeyActionDef {
  id: string;
  label: string;
  category: 'player' | 'general';
  defaultCode: string;
  defaultDisplayKey: string;
}

export const HOTKEY_ACTIONS: HotkeyActionDef[] = [
  { id: 'playPause', label: 'Воспроизведение / Пауза', category: 'player', defaultCode: 'Space', defaultDisplayKey: 'Пробел' },
  { id: 'stepBackward', label: 'Покадрово назад', category: 'player', defaultCode: 'KeyZ', defaultDisplayKey: 'Z' },
  { id: 'stepForward', label: 'Покадрово вперед', category: 'player', defaultCode: 'KeyX', defaultDisplayKey: 'X' },
  { id: 'seekBackward', label: 'Перемотка назад', category: 'player', defaultCode: 'ArrowLeft', defaultDisplayKey: '←' },
  { id: 'seekForward', label: 'Перемотка вперед', category: 'player', defaultCode: 'ArrowRight', defaultDisplayKey: '→' },
  { id: 'triggerMark', label: 'Засечь сход / Отметить удар', category: 'player', defaultCode: 'KeyC', defaultDisplayKey: 'C' },
  { id: 'toggleSlow', label: 'Переключить замедление', category: 'player', defaultCode: 'KeyA', defaultDisplayKey: 'A' },
  { id: 'toggleFast', label: 'Переключить ускорение', category: 'player', defaultCode: 'KeyS', defaultDisplayKey: 'S' },
  { id: 'toggleLoop', label: 'Зацикливание схода', category: 'player', defaultCode: 'KeyD', defaultDisplayKey: 'D' },
  { id: 'toggleFullscreen', label: 'Полноэкранный режим', category: 'player', defaultCode: 'KeyF', defaultDisplayKey: 'F' },
  { id: 'togglePanels', label: 'Скрыть/показать панели', category: 'player', defaultCode: 'KeyG', defaultDisplayKey: 'G' },
  { id: 'closeModal', label: 'Закрыть окно / меню', category: 'general', defaultCode: 'Escape', defaultDisplayKey: 'Esc' },
  { id: 'confirmAction', label: 'Подтвердить / Отправить', category: 'general', defaultCode: 'Enter', defaultDisplayKey: 'Enter' },
];

export interface HotkeysConfig {
  seekStepSeconds: number;
  slowSpeed: number;
  fastSpeed: number;
  keys: Record<string, { code: string; displayKey: string }>;
}

function getDefaultConfig(): HotkeysConfig {
  const keys: Record<string, { code: string; displayKey: string }> = {};
  for (const act of HOTKEY_ACTIONS) {
    keys[act.id] = { code: act.defaultCode, displayKey: act.defaultDisplayKey };
  }
  return {
    seekStepSeconds: 2,
    slowSpeed: 0.1, // Default set to 0.1x per user request
    fastSpeed: 2.0,
    keys,
  };
}

const STORAGE_KEY = 'ef_hotkeys_config';

function loadConfig(): HotkeysConfig {
  if (typeof localStorage === 'undefined') return getDefaultConfig();
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return getDefaultConfig();
    const parsed = JSON.parse(raw);
    const defaults = getDefaultConfig();
    return {
      seekStepSeconds: typeof parsed.seekStepSeconds === 'number' ? parsed.seekStepSeconds : defaults.seekStepSeconds,
      slowSpeed: typeof parsed.slowSpeed === 'number' ? parsed.slowSpeed : defaults.slowSpeed,
      fastSpeed: typeof parsed.fastSpeed === 'number' ? parsed.fastSpeed : defaults.fastSpeed,
      keys: { ...defaults.keys, ...(parsed.keys || {}) },
    };
  } catch {
    return getDefaultConfig();
  }
}

function createHotkeysStore() {
  const { subscribe, set, update } = writable<HotkeysConfig>(loadConfig());

  function save(cfg: HotkeysConfig) {
    if (typeof localStorage !== 'undefined') {
      try {
        localStorage.setItem(STORAGE_KEY, JSON.stringify(cfg));
      } catch {
        // ignore
      }
    }
  }

  return {
    subscribe,
    setKey: (actionId: string, code: string, displayKey: string) => {
      update((cfg) => {
        const newKeys = { ...cfg.keys };
        // Check if code is already used by another action, swap or clear it
        for (const [otherId, binding] of Object.entries(newKeys)) {
          if (binding.code === code && otherId !== actionId) {
            newKeys[otherId] = { code: '', displayKey: '—' };
          }
        }
        newKeys[actionId] = { code, displayKey };
        const updated = { ...cfg, keys: newKeys };
        save(updated);
        return updated;
      });
    },
    updateSettings: (settings: Partial<Pick<HotkeysConfig, 'seekStepSeconds' | 'slowSpeed' | 'fastSpeed'>>) => {
      update((cfg) => {
        const updated = { ...cfg, ...settings };
        save(updated);
        return updated;
      });
    },
    resetDefaults: () => {
      const def = getDefaultConfig();
      save(def);
      set(def);
    },
  };
}

export const hotkeysStore = createHotkeysStore();

export function formatEventKey(e: KeyboardEvent): { code: string; displayKey: string } {
  const code = e.code;
  let displayKey = e.key;

  if (code === 'Space') displayKey = 'Пробел';
  else if (code === 'ArrowLeft') displayKey = '←';
  else if (code === 'ArrowRight') displayKey = '→';
  else if (code === 'ArrowUp') displayKey = '↑';
  else if (code === 'ArrowDown') displayKey = '↓';
  else if (code === 'Escape') displayKey = 'Esc';
  else if (code === 'Enter') displayKey = 'Enter';
  else if (code === 'Tab') displayKey = 'Tab';
  else if (code.startsWith('Key')) displayKey = code.slice(3).toUpperCase();
  else if (code.startsWith('Digit')) displayKey = code.slice(5);

  return { code, displayKey };
}
