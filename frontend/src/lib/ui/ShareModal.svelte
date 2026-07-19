<script lang="ts">
  import { onMount } from 'svelte';
  import { createShareToken } from '../api/videos';

  interface Props {
    videoId: string;
    boutId?: number | null;
    initialTimeMs?: number;
    onclose: () => void;
  }

  let { videoId, boutId = null, initialTimeMs = 0, onclose }: Props = $props();

  let mode = $state<'guest' | 'member'>('guest');
  let token = $state<string | null>(null);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let copied = $state(false);

  // Link generation
  let memberLink = $derived.by(() => {
    const origin = window.location.origin;
    let link = `${origin}/#/player/${videoId}`;
    const params: string[] = [];
    if (boutId !== null) params.push(`bout_id=${boutId}`);
    if (initialTimeMs > 0) params.push(`t=${initialTimeMs}`);
    if (params.length > 0) link += `?${params.join('&')}`;
    return link;
  });

  let guestLink = $derived.by(() => {
    if (!token) return '';
    const origin = window.location.origin;
    let link = `${origin}/#/share/video/${videoId}?token=${encodeURIComponent(token)}`;
    if (boutId !== null) link += `&bout_id=${boutId}`;
    if (initialTimeMs > 0) link += `&t=${initialTimeMs}`;
    return link;
  });

  let currentLink = $derived(mode === 'guest' ? guestLink : memberLink);

  onMount(async () => {
    try {
      const res = await createShareToken(videoId, boutId);
      token = res.token;
    } catch (err) {
      error = err instanceof Error ? err.message : 'Не удалось создать гостевой токен';
    } finally {
      loading = false;
    }
  });

  async function handleCopy() {
    if (!currentLink) return;
    try {
      await navigator.clipboard.writeText(currentLink);
      copied = true;
      setTimeout(() => { copied = false; }, 2000);
    } catch {
      alert('Не удалось скопировать в буфер обмена');
    }
  }
</script>

<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_noninteractive_element_interactions a11y_no_static_element_interactions a11y_interactive_supports_focus -->
<div class="backdrop" role="presentation" onclick={onclose}>
  <div class="modal glass-card" role="dialog" aria-modal="true" tabindex="-1" onclick={(e) => e.stopPropagation()}>
    <div class="header-row">
      <h2>Поделиться {boutId !== null ? 'сходом' : 'видео'}</h2>
      <button class="close-btn" onclick={onclose} aria-label="Закрыть">✕</button>
    </div>

    {#if error}
      <div class="error-msg">{error}</div>
    {/if}

    <div class="tabs">
      <button
        class="tab"
        class:active={mode === 'guest'}
        onclick={() => mode = 'guest'}
        disabled={loading || error !== null}
      >
        Гостевая ссылка
      </button>
      <button
        class="tab"
        class:active={mode === 'member'}
        onclick={() => mode = 'member'}
      >
        Для пользователей
      </button>
    </div>

    <div class="tab-content">
      {#if mode === 'guest'}
        <p class="description">
          Любой человек сможет посмотреть это {boutId !== null ? 'сход' : 'видео'} без регистрации и входа в аккаунт.
        </p>
      {:else}
        <p class="description">
          Ссылка для авторизованных участников. Потребуется ввести логин и пароль.
        </p>
      {/if}

      <div class="link-field">
        {#if loading && mode === 'guest'}
          <div class="loading-placeholder">Генерация ссылки…</div>
        {:else}
          <input
            type="text"
            class="input-glass link-input"
            value={currentLink}
            readonly
            onclick={(e) => (e.target as HTMLInputElement).select()}
          />
          <button class="btn btn-primary copy-btn" onclick={handleCopy} disabled={!currentLink}>
            {#if copied}
              Скопировано!
            {:else}
              Копировать
            {/if}
          </button>
        {/if}
      </div>
    </div>
  </div>
</div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.6);
    display: grid;
    place-items: center;
    z-index: 1000;
  }

  .modal {
    width: 90%;
    max-width: 500px;
    padding: 32px;
    display: flex;
    flex-direction: column;
    gap: 20px;
    background: rgba(15, 23, 42, 0.8) !important;
    border-color: rgba(255, 255, 255, 0.1) !important;
  }

  .header-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .header-row h2 {
    margin: 0;
    font-size: 1.4rem;
    font-weight: 700;
    color: var(--text-primary);
  }

  .close-btn {
    background: none;
    border: none;
    color: var(--text-secondary);
    font-size: 1.2rem;
    cursor: pointer;
    padding: 4px 8px;
    border-radius: 4px;
    transition: var(--transition);
  }

  .close-btn:hover {
    color: var(--text-primary);
    background: var(--surface-hover);
  }

  .error-msg {
    color: var(--accent-red);
    background: rgba(224, 82, 82, 0.1);
    padding: 10px;
    border-radius: var(--radius-sm);
    font-size: 0.85rem;
  }

  .tabs {
    display: flex;
    gap: 8px;
    border-bottom: 1px solid var(--border-color);
    padding-bottom: 8px;
  }

  .tab {
    background: none;
    border: none;
    padding: 8px 16px;
    color: var(--text-secondary);
    font-size: 0.9rem;
    font-weight: 500;
    cursor: pointer;
    border-radius: var(--radius-sm);
    transition: var(--transition);
  }

  .tab:hover:not(:disabled) {
    color: var(--text-primary);
    background: var(--surface-hover);
  }

  .tab.active {
    color: var(--accent-yellow);
    background: rgba(245, 158, 11, 0.1);
  }

  .tab:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .tab-content {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .description {
    font-size: 0.85rem;
    color: var(--text-secondary);
    line-height: 1.4;
  }

  .link-field {
    display: flex;
    gap: 10px;
    align-items: center;
  }

  .link-input {
    flex: 1;
    font-family: monospace;
    font-size: 0.8rem;
    background: rgba(0, 0, 0, 0.4) !important;
  }

  .copy-btn {
    flex-shrink: 0;
    min-width: 120px;
  }

  .loading-placeholder {
    font-size: 0.9rem;
    color: var(--text-muted);
    padding: 10px;
  }
</style>
