<script lang="ts">
  import { toasts, removeToast } from '../stores/toast';
</script>

{#if $toasts.length > 0}
  <div class="toast-container" aria-live="polite">
    {#each $toasts as t (t.id)}
      <div class="toast-card toast-{t.type}">
        <div class="toast-icon">
          {#if t.type === 'success'}
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="#34d399" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14" />
              <polyline points="22 4 12 14.01 9 11.01" />
            </svg>
          {:else if t.type === 'error'}
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="#f87171" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <circle cx="12" cy="12" r="10" />
              <line x1="15" y1="9" x2="9" y2="15" />
              <line x1="9" y1="9" x2="15" y2="15" />
            </svg>
          {:else}
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="#60a5fa" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <circle cx="12" cy="12" r="10" />
              <line x1="12" y1="16" x2="12" y2="12" />
              <line x1="12" y1="8" x2="12.01" y2="8" />
            </svg>
          {/if}
        </div>
        <div class="toast-message">{t.message}</div>
        <button class="toast-close" onclick={() => removeToast(t.id)} aria-label="Закрыть">✕</button>
      </div>
    {/each}
  </div>
{/if}

<style>
  .toast-container {
    position: fixed;
    bottom: 24px;
    right: 24px;
    z-index: 100000;
    display: flex;
    flex-direction: column;
    gap: 10px;
    max-width: 420px;
    pointer-events: none;
  }

  .toast-card {
    pointer-events: auto;
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px 16px;
    background: rgba(26, 36, 52, 0.95);
    backdrop-filter: blur(12px);
    -webkit-backdrop-filter: blur(12px);
    border: 1px solid rgba(255, 255, 255, 0.12);
    border-radius: var(--radius-md, 8px);
    box-shadow: 0 10px 25px -5px rgba(0, 0, 0, 0.5), 0 8px 10px -6px rgba(0, 0, 0, 0.5);
    color: var(--text-primary, #fff);
    font-size: 0.88rem;
    animation: toastSlideIn 0.22s ease-out;
  }

  .toast-card.toast-success {
    border-left: 3px solid #10b981;
  }

  .toast-card.toast-error {
    border-left: 3px solid #ef4444;
  }

  .toast-card.toast-info {
    border-left: 3px solid #3b82f6;
  }

  .toast-icon {
    display: flex;
    align-items: center;
    flex-shrink: 0;
  }

  .toast-message {
    flex-grow: 1;
    line-height: 1.4;
  }

  .toast-close {
    background: transparent;
    border: none;
    color: var(--text-secondary, #94a3b8);
    font-size: 0.85rem;
    cursor: pointer;
    padding: 4px;
    border-radius: 4px;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: color 0.15s ease;
    flex-shrink: 0;
  }

  .toast-close:hover {
    color: #fff;
  }

  @keyframes toastSlideIn {
    from {
      opacity: 0;
      transform: translateY(12px) scale(0.96);
    }
    to {
      opacity: 1;
      transform: translateY(0) scale(1);
    }
  }

  @media (max-width: 768px) {
    .toast-container {
      bottom: 16px;
      left: 16px;
      right: 16px;
      max-width: none;
    }
  }
</style>
