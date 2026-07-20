<script lang="ts">
  interface Props {
    onclose: () => void;
  }

  let { onclose }: Props = $props();

  function handleBackdrop(e: MouseEvent) {
    if ((e.target as HTMLElement).classList.contains('modal-backdrop')) {
      onclose();
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      onclose();
    }
  }

  const playerShortcuts = [
    { keys: ['Пробел'], label: 'Воспроизведение / Пауза' },
    { keys: ['Z', 'X'], label: 'Перемотка покадрово назад / вперед' },
    { keys: ['←', '→'], label: 'Перемотка на 2 секунды назад / вперед' },
    { keys: ['C'], label: 'Засечь сход / Отметить удар в судействе' },
    { keys: ['A'], label: 'Переключить замедление (0.2x / 1.0x)' },
    { keys: ['S'], label: 'Переключить ускорение (2.0x / 1.0x)' },
    { keys: ['D'], label: 'Включить / отключить зацикливание' },
    { keys: ['F'], label: 'Полноэкранный режим' },
    { keys: ['G'], label: 'Показать / скрыть панель судейства и чат' },
  ];

  const generalShortcuts = [
    { keys: ['Esc'], label: 'Закрыть модальное окно или выпадающее меню' },
    { keys: ['Enter'], label: 'Подтвердить ввод / Отправить сообщение' },
  ];
</script>

<svelte:window onkeydown={handleKeydown} />

<!-- svelte-ignore a11y_click_events_have_key_events -->
<div class="modal-backdrop" onclick={handleBackdrop} role="dialog" aria-modal="true" aria-label="Горячие клавиши" tabindex="-1">
  <div class="modal">
    <div class="modal-header">
      <div class="modal-title">
        <svg class="title-icon" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <rect x="2" y="4" width="20" height="16" rx="2" ry="2"/>
          <path d="M6 8h.01M10 8h.01M14 8h.01M18 8h.01M6 12h.01M10 12h.01M14 12h.01M18 12h.01M8 16h8"/>
        </svg>
        <h2>Горячие клавиши</h2>
      </div>
      <button class="close-btn" onclick={onclose} aria-label="Закрыть">
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none">
          <path d="M18 6L6 18M6 6l12 12" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
        </svg>
      </button>
    </div>

    <div class="modal-body">
      <section class="shortcut-section">
        <h3 class="section-title">Плеер и судейство</h3>
        <div class="shortcuts-grid">
          {#each playerShortcuts as item}
            <div class="shortcut-row">
              <div class="keys-group">
                {#each item.keys as k, i}
                  {#if i > 0}<span class="key-sep">/</span>{/if}
                  <kbd class="kbd">{k}</kbd>
                {/each}
              </div>
              <span class="shortcut-label">{item.label}</span>
            </div>
          {/each}
        </div>
      </section>

      <section class="shortcut-section">
        <h3 class="section-title">Общие</h3>
        <div class="shortcuts-grid">
          {#each generalShortcuts as item}
            <div class="shortcut-row">
              <div class="keys-group">
                {#each item.keys as k, i}
                  {#if i > 0}<span class="key-sep">/</span>{/if}
                  <kbd class="kbd">{k}</kbd>
                {/each}
              </div>
              <span class="shortcut-label">{item.label}</span>
            </div>
          {/each}
        </div>
      </section>
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
    max-width: 540px;
    max-height: 85vh;
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
    padding: 20px 24px;
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
    font-size: 1.2rem;
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
    padding: 24px;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 24px;
  }

  .shortcut-section {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .section-title {
    font-size: 0.85rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--accent-yellow, #F59E0B);
    margin: 0;
  }

  .shortcuts-grid {
    display: flex;
    flex-direction: column;
    gap: 8px;
    background: rgba(0, 0, 0, 0.2);
    border: 1px solid var(--border-color, rgba(255, 255, 255, 0.05));
    border-radius: var(--radius-lg, 16px);
    padding: 12px 16px;
  }

  .shortcut-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 6px 0;
    gap: 16px;
  }

  .shortcut-row:not(:last-child) {
    border-bottom: 1px solid rgba(255, 255, 255, 0.04);
  }

  .keys-group {
    display: flex;
    align-items: center;
    gap: 4px;
    flex-shrink: 0;
  }

  .key-sep {
    font-size: 0.85rem;
    color: var(--text-muted, #6B7280);
  }

  .kbd {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 28px;
    height: 28px;
    padding: 0 8px;
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
  }

  .shortcut-label {
    font-size: 0.9rem;
    color: var(--text-secondary, #9CA3AF);
    text-align: right;
  }
</style>
