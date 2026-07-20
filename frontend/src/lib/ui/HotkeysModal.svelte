<script lang="ts">
  import { hotkeysStore, HOTKEY_ACTIONS, formatEventKey } from '../stores/hotkeys';

  interface Props {
    onclose: () => void;
  }

  let { onclose }: Props = $props();

  let recordingActionId = $state<string | null>(null);

  function handleBackdrop(e: MouseEvent) {
    if ((e.target as HTMLElement).classList.contains('modal-backdrop')) {
      onclose();
    }
  }

  function handleWindowKeydown(e: KeyboardEvent) {
    if (recordingActionId) {
      e.preventDefault();
      e.stopPropagation();

      // Ignore lone modifier keys
      if (['Control', 'Shift', 'Alt', 'Meta', 'CapsLock'].includes(e.key)) {
        return;
      }

      const formatted = formatEventKey(e);
      hotkeysStore.setKey(recordingActionId, formatted.code, formatted.displayKey);
      recordingActionId = null;
      return;
    }

    if (e.key === 'Escape') {
      onclose();
    }
  }

  function startRecording(actionId: string) {
    recordingActionId = actionId;
  }

  function cancelRecording() {
    recordingActionId = null;
  }

  const seekOptions = [0.5, 1, 2, 3, 5, 10];
  const slowOptions = [0.05, 0.1, 0.15, 0.2, 0.25, 0.3, 0.5];
  const fastOptions = [1.25, 1.5, 1.75, 2.0, 2.5, 3.0];
</script>

<svelte:window onkeydown={handleWindowKeydown} />

<!-- svelte-ignore a11y_click_events_have_key_events -->
<div class="modal-backdrop" onclick={handleBackdrop} role="dialog" aria-modal="true" aria-label="Настройка горячих клавиш" tabindex="-1">
  <div class="modal">
    <div class="modal-header">
      <div class="modal-title">
        <svg class="title-icon" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <rect x="2" y="4" width="20" height="16" rx="2" ry="2"/>
          <path d="M6 8h.01M10 8h.01M14 8h.01M18 8h.01M6 12h.01M10 12h.01M14 12h.01M18 12h.01M18 12h.01M8 16h8"/>
        </svg>
        <h2>Горячие клавиши и плеер</h2>
      </div>
      <button class="close-btn" onclick={onclose} aria-label="Закрыть">
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none">
          <path d="M18 6L6 18M6 6l12 12" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
        </svg>
      </button>
    </div>

    <div class="modal-body">
      <!-- Player Parameters Settings -->
      <section class="shortcut-section">
        <h3 class="section-title">Параметры плеера</h3>
        <div class="settings-card">
          <div class="setting-row">
            <div class="setting-info">
              <span class="setting-name">Шаг перемотки (стрелки ← / →)</span>
              <span class="setting-desc">Сколько секунд проматывать за одно нажатие</span>
            </div>
            <select
              class="select-glass"
              value={$hotkeysStore.seekStepSeconds}
              onchange={(e) => hotkeysStore.updateSettings({ seekStepSeconds: parseFloat((e.target as HTMLSelectElement).value) })}
            >
              {#each seekOptions as opt}
                <option value={opt}>{opt} сек</option>
              {/each}
            </select>
          </div>

          <div class="setting-row">
            <div class="setting-info">
              <span class="setting-name">Скорость замедления (клавиша A)</span>
              <span class="setting-desc">Скорость воспроизведения при нажатии A</span>
            </div>
            <select
              class="select-glass"
              value={$hotkeysStore.slowSpeed}
              onchange={(e) => hotkeysStore.updateSettings({ slowSpeed: parseFloat((e.target as HTMLSelectElement).value) })}
            >
              {#each slowOptions as opt}
                <option value={opt}>{opt}x</option>
              {/each}
            </select>
          </div>

          <div class="setting-row">
            <div class="setting-info">
              <span class="setting-name">Скорость ускорения (клавиша S)</span>
              <span class="setting-desc">Скорость воспроизведения при нажатии S</span>
            </div>
            <select
              class="select-glass"
              value={$hotkeysStore.fastSpeed}
              onchange={(e) => hotkeysStore.updateSettings({ fastSpeed: parseFloat((e.target as HTMLSelectElement).value) })}
            >
              {#each fastOptions as opt}
                <option value={opt}>{opt}x</option>
              {/each}
            </select>
          </div>
        </div>
      </section>

      <!-- Keybindings Section -->
      <section class="shortcut-section">
        <div class="section-header-row">
          <h3 class="section-title">Клавиши управления</h3>
          <span class="hint-text">Нажмите на клавишу для переназначения</span>
        </div>
        <div class="shortcuts-grid">
          {#each HOTKEY_ACTIONS as act}
            {@const binding = $hotkeysStore.keys[act.id]}
            {@const isRecording = recordingActionId === act.id}
            <div class="shortcut-row" class:is-recording={isRecording}>
              <span class="shortcut-label">{act.label}</span>
              <div class="key-container">
                {#if isRecording}
                  <button class="kbd-btn recording" onclick={cancelRecording} title="Нажмите Esc для отмены">
                    Нажмите клавишу…
                  </button>
                {:else}
                  <button class="kbd-btn" onclick={() => startRecording(act.id)} title="Нажмите, чтобы изменить">
                    <kbd class="kbd">{binding?.displayKey || '—'}</kbd>
                  </button>
                {/if}
              </div>
            </div>
          {/each}
        </div>
      </section>
    </div>

    <div class="modal-footer">
      <button class="btn-reset" onclick={() => hotkeysStore.resetDefaults()}>
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"/>
          <path d="M3 3v5h5"/>
        </svg>
        Сбросить настройки
      </button>
      <button class="btn-close" onclick={onclose}>Готово</button>
    </div>
  </div>
</div>

<style>
  .modal-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(10, 10, 12, 0.75);
    backdrop-filter: blur(8px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
    padding: 16px;
  }

  .modal {
    background: var(--surface-solid, #1F2937);
    border: 1px solid var(--border-color, rgba(255, 255, 255, 0.1));
    border-radius: var(--radius-xl, 20px);
    width: 100%;
    max-width: 580px;
    max-height: 88vh;
    display: flex;
    flex-direction: column;
    box-shadow: 0 20px 40px rgba(0, 0, 0, 0.5);
    overflow: hidden;
    animation: modalIn 0.2s cubic-bezier(0.16, 1, 0.3, 1);
  }

  @keyframes modalIn {
    from {
      opacity: 0;
      transform: scale(0.96) translateY(8px);
    }
    to {
      opacity: 1;
      transform: scale(1) translateY(0);
    }
  }

  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 18px 24px;
    border-bottom: 1px solid var(--border-color, rgba(255, 255, 255, 0.08));
  }

  .modal-title {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .title-icon {
    color: var(--accent-yellow, #F59E0B);
  }

  .modal-header h2 {
    margin: 0;
    font-size: 1.15rem;
    font-weight: 600;
    color: var(--text-primary, #F9FAFB);
  }

  .close-btn {
    background: none;
    border: none;
    color: var(--text-muted, #6B7280);
    cursor: pointer;
    padding: 6px;
    border-radius: var(--radius-md, 8px);
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.15s ease;
  }

  .close-btn:hover {
    color: var(--text-primary, #F9FAFB);
    background: rgba(255, 255, 255, 0.08);
  }

  .modal-body {
    padding: 20px 24px;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 20px;
  }

  .shortcut-section {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .section-header-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .section-title {
    font-size: 0.8rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--accent-yellow, #F59E0B);
    margin: 0;
  }

  .hint-text {
    font-size: 0.75rem;
    color: var(--text-muted, #6B7280);
  }

  .settings-card {
    display: flex;
    flex-direction: column;
    gap: 10px;
    background: rgba(0, 0, 0, 0.2);
    border: 1px solid var(--border-color, rgba(255, 255, 255, 0.05));
    border-radius: var(--radius-lg, 16px);
    padding: 12px 16px;
  }

  .setting-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
  }

  .setting-row:not(:last-child) {
    border-bottom: 1px solid rgba(255, 255, 255, 0.04);
    padding-bottom: 10px;
  }

  .setting-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .setting-name {
    font-size: 0.88rem;
    font-weight: 500;
    color: var(--text-primary, #F9FAFB);
  }

  .setting-desc {
    font-size: 0.78rem;
    color: var(--text-muted, #6B7280);
  }

  .select-glass {
    background: var(--surface-solid, #1F2937);
    color: var(--text-primary, #F9FAFB);
    border: 1px solid var(--border-color, rgba(255, 255, 255, 0.15));
    border-radius: var(--radius-md, 8px);
    padding: 6px 12px;
    font-size: 0.85rem;
    outline: none;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .select-glass:hover, .select-glass:focus {
    border-color: var(--accent-yellow, #F59E0B);
  }

  .shortcuts-grid {
    display: flex;
    flex-direction: column;
    gap: 6px;
    background: rgba(0, 0, 0, 0.2);
    border: 1px solid var(--border-color, rgba(255, 255, 255, 0.05));
    border-radius: var(--radius-lg, 16px);
    padding: 10px 16px;
  }

  .shortcut-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 5px 0;
    gap: 16px;
    transition: background 0.15s;
  }

  .shortcut-row:not(:last-child) {
    border-bottom: 1px solid rgba(255, 255, 255, 0.04);
  }

  .shortcut-label {
    font-size: 0.88rem;
    color: var(--text-secondary, #9CA3AF);
  }

  .key-container {
    display: flex;
    align-items: center;
  }

  .kbd-btn {
    background: none;
    border: none;
    padding: 0;
    cursor: pointer;
    display: inline-flex;
    align-items: center;
  }

  .kbd-btn.recording {
    background: rgba(245, 158, 11, 0.15);
    border: 1px solid var(--accent-yellow, #F59E0B);
    color: var(--accent-yellow, #F59E0B);
    padding: 4px 12px;
    border-radius: 6px;
    font-size: 0.8rem;
    font-weight: 600;
    animation: pulse 1.5s infinite;
  }

  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.6; }
  }

  .kbd {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 32px;
    height: 28px;
    padding: 0 10px;
    font-family: inherit;
    font-size: 0.82rem;
    font-weight: 600;
    color: var(--text-primary, #F9FAFB);
    background: linear-gradient(180deg, rgba(255, 255, 255, 0.12) 0%, rgba(255, 255, 255, 0.04) 100%);
    border: 1px solid rgba(255, 255, 255, 0.15);
    border-bottom: 2px solid rgba(0, 0, 0, 0.4);
    border-radius: 6px;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
    box-sizing: border-box;
    transition: all 0.15s ease;
  }

  .kbd-btn:hover .kbd {
    border-color: var(--accent-yellow, #F59E0B);
    background: linear-gradient(180deg, rgba(245, 158, 11, 0.2) 0%, rgba(245, 158, 11, 0.05) 100%);
  }

  .modal-footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 14px 24px;
    border-top: 1px solid var(--border-color, rgba(255, 255, 255, 0.08));
    background: rgba(0, 0, 0, 0.15);
  }

  .btn-reset {
    display: flex;
    align-items: center;
    gap: 6px;
    background: none;
    border: none;
    color: var(--text-muted, #6B7280);
    font-size: 0.82rem;
    cursor: pointer;
    transition: color 0.15s ease;
  }

  .btn-reset:hover {
    color: var(--accent-red, #E05252);
  }

  .btn-close {
    background: var(--accent-yellow, #F59E0B);
    color: #000;
    font-weight: 600;
    font-size: 0.85rem;
    padding: 6px 18px;
    border-radius: var(--radius-md, 8px);
    border: none;
    cursor: pointer;
    transition: background 0.15s ease;
  }

  .btn-close:hover {
    background: var(--accent-yellow-hover, #D97706);
  }
</style>
