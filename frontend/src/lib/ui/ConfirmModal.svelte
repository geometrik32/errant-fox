<script lang="ts">
  import { onMount } from 'svelte';

  interface Props {
    title?: string;
    message: string;
    confirmText?: string;
    cancelText?: string;
    danger?: boolean;
    onconfirm: () => void;
    oncancel: () => void;
  }

  let {
    title = 'Подтверждение',
    message,
    confirmText = 'Подтвердить',
    cancelText = 'Отмена',
    danger = true,
    onconfirm,
    oncancel
  }: Props = $props();

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      oncancel();
    } else if (e.key === 'Enter') {
      onconfirm();
    }
  }

  onMount(() => {
    window.addEventListener('keydown', handleKeydown);
    return () => window.removeEventListener('keydown', handleKeydown);
  });
</script>

<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_noninteractive_element_interactions a11y_no_static_element_interactions -->
<div class="backdrop" role="presentation" onclick={oncancel}>
  <div class="modal glass-card" role="dialog" aria-modal="true" tabindex="-1" onclick={(e) => e.stopPropagation()}>
    <div class="header-row">
      <h2>{title}</h2>
      <button class="close-btn" onclick={oncancel} aria-label="Закрыть">✕</button>
    </div>

    <p class="description">{message}</p>

    <div class="footer-row">
      <button class="btn btn-secondary" onclick={oncancel}>{cancelText}</button>
      <button class="btn" class:btn-danger={danger} class:btn-primary={!danger} onclick={onconfirm}>
        {confirmText}
      </button>
    </div>
  </div>
</div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.7);
    backdrop-filter: blur(4px);
    display: grid;
    place-items: center;
    z-index: 10000;
    animation: fadeIn 0.15s ease-out;
  }

  .modal {
    width: 90%;
    max-width: 440px;
    padding: 24px;
    display: flex;
    flex-direction: column;
    gap: 16px;
    background: rgba(15, 23, 42, 0.95) !important;
    border: 1px solid rgba(255, 255, 255, 0.12) !important;
    box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.5), 0 8px 10px -6px rgba(0, 0, 0, 0.5);
    border-radius: var(--radius-lg, 12px);
    animation: scaleIn 0.15s ease-out;
  }

  .header-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .header-row h2 {
    margin: 0;
    font-size: 1.25rem;
    font-weight: 700;
    color: var(--text-primary, #f9fafb);
  }

  .close-btn {
    background: none;
    border: none;
    color: var(--text-secondary, #9ca3af);
    font-size: 1.2rem;
    cursor: pointer;
    padding: 4px 8px;
    border-radius: 4px;
    transition: var(--transition, all 0.2s);
  }

  .close-btn:hover {
    color: var(--text-primary, #ffffff);
    background: rgba(255, 255, 255, 0.1);
  }

  .description {
    margin: 0;
    font-size: 0.95rem;
    color: var(--text-secondary, #d1d5db);
    line-height: 1.5;
    white-space: pre-wrap;
  }

  .footer-row {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 12px;
    margin-top: 8px;
  }

  .btn-secondary {
    background: rgba(255, 255, 255, 0.08);
    color: var(--text-primary, #f3f4f6);
    border: 1px solid rgba(255, 255, 255, 0.1);
  }

  .btn-secondary:hover {
    background: rgba(255, 255, 255, 0.15);
  }

  .btn-danger {
    background: var(--accent-red, #e05252);
    color: #ffffff;
  }

  .btn-danger:hover {
    background: #c83d3d;
  }

  @keyframes fadeIn {
    from { opacity: 0; }
    to { opacity: 1; }
  }

  @keyframes scaleIn {
    from { transform: scale(0.95); opacity: 0; }
    to { transform: scale(1); opacity: 1; }
  }
</style>
