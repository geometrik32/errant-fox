<script lang="ts">
  import { fighters } from '../../stores';
  import { createUser } from '../api/fighters';

  interface Props {
    onclose?: () => void;
  }

  let { onclose }: Props = $props();

  let username = $state('');
  let displayName = $state('');
  let password = $state('');
  let isAdmin = $state(false);
  let saving = $state(false);
  let errorMsg = $state('');

  let canSave = $derived(
    username.trim().length > 0 &&
    displayName.trim().length > 0 &&
    password.length > 0 &&
    !saving
  );

  async function save() {
    if (!canSave) return;
    saving = true;
    errorMsg = '';
    try {
      const created = await createUser({
        username: username.trim(),
        display_name: displayName.trim(),
        password,
        is_admin: isAdmin,
      });
      fighters.update((list) => [...list, created]);
      onclose?.();
    } catch (e) {
      errorMsg = e instanceof Error ? e.message : 'Ошибка при создании';
    } finally {
      saving = false;
    }
  }

  function handleBackdrop(e: MouseEvent) {
    if ((e.target as HTMLElement).classList.contains('backdrop')) {
      onclose?.();
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') onclose?.();
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="backdrop" onclick={handleBackdrop} role="dialog" aria-modal="true" aria-label="Создать бойца">
  <div class="modal">
    <h2 class="title">Создать бойца</h2>

    <div class="field">
      <label class="label" for="new-username">Логин</label>
      <input
        id="new-username"
        class="input"
        type="text"
        bind:value={username}
        placeholder="ivan"
        autocomplete="off"
      />
    </div>

    <div class="field">
      <label class="label" for="new-display-name">Отображаемое имя</label>
      <input
        id="new-display-name"
        class="input"
        type="text"
        bind:value={displayName}
        placeholder="Иван"
        autocomplete="off"
      />
    </div>

    <div class="field">
      <label class="label" for="new-password">Пароль</label>
      <input
        id="new-password"
        class="input"
        type="password"
        bind:value={password}
        autocomplete="new-password"
      />
    </div>

    <label class="checkbox-row">
      <input type="checkbox" bind:checked={isAdmin} />
      <span class="checkbox-label">Администратор</span>
    </label>

    {#if errorMsg}
      <p class="error">{errorMsg}</p>
    {/if}

    <div class="actions">
      <button class="btn-cancel" onclick={onclose} disabled={saving}>Отмена</button>
      <button class="btn-save" onclick={save} disabled={!canSave}>
        {saving ? 'Создание…' : 'Создать'}
      </button>
    </div>
  </div>
</div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.6);
    backdrop-filter: blur(3px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 500;
  }

  .modal {
    background: #0f2035;
    border: 1px solid #1f3a57;
    border-radius: 12px;
    padding: 28px;
    width: 380px;
    max-width: calc(100vw - 32px);
    display: flex;
    flex-direction: column;
    gap: 16px;
    box-shadow: 0 16px 48px rgba(0, 0, 0, 0.5);
  }

  .title {
    font-size: 1.05rem;
    font-weight: 600;
    color: #e8edf2;
    margin: 0;
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .label {
    font-size: 0.78rem;
    color: #6b8aab;
    font-weight: 500;
  }

  .input {
    background: #060e18;
    border: 1px solid #1f3a57;
    border-radius: 6px;
    color: #e8edf2;
    font-size: 0.9rem;
    padding: 8px 10px;
    outline: none;
    width: 100%;
    transition: border-color 0.12s;
    box-sizing: border-box;
  }

  .input:focus {
    border-color: #2a4f73;
  }

  .checkbox-row {
    display: flex;
    align-items: center;
    gap: 10px;
    cursor: pointer;
  }

  .checkbox-row input[type='checkbox'] {
    width: 16px;
    height: 16px;
    accent-color: #DB841F;
    cursor: pointer;
    flex-shrink: 0;
  }

  .checkbox-label {
    font-size: 0.88rem;
    color: #a0b4c8;
  }

  .error {
    font-size: 0.83rem;
    color: #e05252;
    margin: 0;
  }

  .actions {
    display: flex;
    gap: 10px;
    justify-content: flex-end;
    margin-top: 4px;
  }

  .btn-cancel,
  .btn-save {
    border: none;
    border-radius: 6px;
    font-size: 0.88rem;
    font-weight: 500;
    padding: 8px 18px;
    cursor: pointer;
    transition: background 0.12s;
  }

  .btn-cancel {
    background: #1a3050;
    color: #a0b4c8;
  }

  .btn-cancel:hover:not(:disabled) {
    background: #1f3a57;
  }

  .btn-save {
    background: #DB841F;
    color: #fff;
  }

  .btn-save:hover:not(:disabled) {
    background: #c4731a;
  }

  .btn-cancel:disabled,
  .btn-save:disabled {
    opacity: 0.45;
    cursor: not-allowed;
  }
</style>
