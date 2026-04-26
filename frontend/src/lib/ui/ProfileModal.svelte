<script lang="ts">
  import { currentUser } from '../../stores';
  import { patchMe } from '../api/auth';

  interface Props {
    onclose: () => void;
  }

  let { onclose }: Props = $props();

  let displayName = $state($currentUser?.display_name ?? '');
  let newPassword = $state('');
  let confirmPassword = $state('');
  let saving = $state(false);
  let error = $state('');
  let success = $state('');

  async function handleSave(e: Event) {
    e.preventDefault();
    error = '';
    success = '';

    if (newPassword && newPassword !== confirmPassword) {
      error = 'Пароли не совпадают';
      return;
    }

    saving = true;
    try {
      const data: { display_name?: string; password?: string } = {};
      if (displayName !== $currentUser?.display_name) data.display_name = displayName;
      if (newPassword) data.password = newPassword;

      if (Object.keys(data).length === 0) {
        success = 'Нет изменений';
        return;
      }

      const updated = await patchMe(data);
      currentUser.set(updated);
      newPassword = '';
      confirmPassword = '';
      success = 'Сохранено';
    } catch (e) {
      error = e instanceof Error ? e.message : 'Ошибка сохранения';
    } finally {
      saving = false;
    }
  }

  function handleBackdrop(e: MouseEvent) {
    if ((e.target as HTMLElement).classList.contains('modal-backdrop')) {
      onclose();
    }
  }
</script>

<div class="modal-backdrop" role="presentation" onclick={handleBackdrop}>
  <div class="modal" role="dialog" aria-modal="true" aria-label="Профиль">
    <div class="modal-header">
      <h2>Профиль</h2>
      <button class="close-btn" onclick={onclose} aria-label="Закрыть">
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none">
          <path d="M18 6L6 18M6 6l12 12" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
        </svg>
      </button>
    </div>

    <form class="modal-body" onsubmit={handleSave}>
      <div class="avatar-preview" style:background={$currentUser?.color ?? '#1f3a57'}>
        {#if $currentUser?.avatar_url}
          <img src={$currentUser.avatar_url} alt={$currentUser.display_name} />
        {:else}
          <span>{$currentUser?.display_name?.charAt(0).toUpperCase() ?? '?'}</span>
        {/if}
      </div>

      <div class="field">
        <label for="username">Логин</label>
        <input id="username" type="text" value={$currentUser?.username ?? ''} disabled />
      </div>

      <div class="field">
        <label for="display-name">Имя</label>
        <input id="display-name" type="text" bind:value={displayName} required />
      </div>

      <div class="divider"></div>

      <div class="field">
        <label for="new-password">Новый пароль</label>
        <input id="new-password" type="password" bind:value={newPassword} autocomplete="new-password" placeholder="Оставьте пустым, чтобы не менять" />
      </div>

      <div class="field">
        <label for="confirm-password">Подтверждение пароля</label>
        <input id="confirm-password" type="password" bind:value={confirmPassword} autocomplete="new-password" />
      </div>

      {#if error}
        <p class="msg error">{error}</p>
      {/if}
      {#if success}
        <p class="msg success">{success}</p>
      {/if}

      <div class="actions">
        <button type="button" class="btn-cancel" onclick={onclose}>Отмена</button>
        <button type="submit" class="btn-save" disabled={saving}>
          {saving ? 'Сохранение…' : 'Сохранить'}
        </button>
      </div>
    </form>
  </div>
</div>

<style>
  .modal-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.55);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 500;
  }

  .modal {
    background: #0f2035;
    border: 1px solid #1f3a57;
    border-radius: 12px;
    width: 100%;
    max-width: 420px;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 20px 24px 16px;
    border-bottom: 1px solid #1f3a57;
  }

  .modal-header h2 {
    font-size: 1rem;
    font-weight: 600;
    color: #e8edf2;
    margin: 0;
  }

  .close-btn {
    background: none;
    border: none;
    color: #6b8aab;
    cursor: pointer;
    padding: 4px;
    border-radius: 4px;
    display: flex;
    transition: color 0.15s;
  }

  .close-btn:hover { color: #e8edf2; }

  .modal-body {
    padding: 24px;
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .avatar-preview {
    width: 64px;
    height: 64px;
    border-radius: 50%;
    border: 2px solid #2a4f73;
    overflow: hidden;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 1.5rem;
    font-weight: 700;
    color: #fff;
    align-self: center;
    margin-bottom: 4px;
  }

  .avatar-preview img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .field label {
    font-size: 0.8rem;
    font-weight: 500;
    color: #6b8aab;
  }

  .field input {
    background: #0d1b2a;
    border: 1px solid #1f3a57;
    border-radius: 6px;
    color: #e8edf2;
    padding: 9px 12px;
    font-size: 0.9rem;
    outline: none;
    transition: border-color 0.2s;
  }

  .field input:focus { border-color: #DB841F; }
  .field input:disabled { color: #4a6280; cursor: not-allowed; }

  .divider {
    height: 1px;
    background: #1f3a57;
    margin: 4px 0;
  }

  .msg {
    font-size: 0.85rem;
    padding: 8px 12px;
    border-radius: 6px;
    margin: 0;
  }

  .error {
    color: #e05252;
    background: rgba(224, 82, 82, 0.1);
    border: 1px solid rgba(224, 82, 82, 0.2);
  }

  .success {
    color: #6aaa5e;
    background: rgba(106, 170, 94, 0.1);
    border: 1px solid rgba(106, 170, 94, 0.2);
  }

  .actions {
    display: flex;
    gap: 10px;
    justify-content: flex-end;
    margin-top: 4px;
  }

  .btn-cancel {
    background: none;
    border: 1px solid #1f3a57;
    color: #6b8aab;
    border-radius: 6px;
    padding: 8px 18px;
    font-size: 0.875rem;
    cursor: pointer;
    transition: border-color 0.15s, color 0.15s;
  }

  .btn-cancel:hover {
    border-color: #4a6280;
    color: #e8edf2;
  }

  .btn-save {
    background: #DB841F;
    border: none;
    color: #fff;
    border-radius: 6px;
    padding: 8px 18px;
    font-size: 0.875rem;
    font-weight: 600;
    cursor: pointer;
    transition: background 0.15s;
  }

  .btn-save:hover:not(:disabled) { background: #c4741a; }
  .btn-save:disabled { opacity: 0.6; cursor: not-allowed; }
</style>
