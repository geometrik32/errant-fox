<script lang="ts">
  import { currentUser, fighters } from '../../stores';
  import { patchMe, uploadMyAvatar } from '../api/auth';
  import { getFighters } from '../api/fighters';
  import { resolveColor } from '../api/types';

  interface Props {
    onclose: () => void;
  }

  let { onclose }: Props = $props();

  let username = $state($currentUser?.username ?? '');
  let displayName = $state($currentUser?.display_name ?? '');
  let color = $state($currentUser?.color ?? '');
  let newPassword = $state('');
  let confirmPassword = $state('');
  let saving = $state(false);
  let error = $state('');
  let success = $state('');
  let avatarFile = $state<File | null>(null);
  let avatarPreview = $state<string | null>(null);

  let effectiveColor = $derived(
    color || resolveColor($currentUser?.id ?? '', null)
  );

  function handleAvatarChange(e: Event) {
    const file = (e.target as HTMLInputElement).files?.[0] ?? null;
    avatarFile = file;
    if (file) {
      const reader = new FileReader();
      reader.onload = (ev) => { avatarPreview = ev.target?.result as string; };
      reader.readAsDataURL(file);
    } else {
      avatarPreview = null;
    }
  }

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
      const data: { username?: string; display_name?: string; password?: string; color?: string } = {};
      if (username !== $currentUser?.username) data.username = username;
      if (displayName !== $currentUser?.display_name) data.display_name = displayName;
      if (newPassword) data.password = newPassword;
      if (color && color !== $currentUser?.color) data.color = color;

      const colorChanged = !!data.color;

      if (Object.keys(data).length > 0) {
        const updated = await patchMe(data);
        currentUser.set(updated);
        if (colorChanged) {
          fighters.set(await getFighters());
        }
      }

      if (avatarFile) {
        await uploadMyAvatar(avatarFile);
        currentUser.update(u => u ? { ...u, avatar_url: u.avatar_url + '?t=' + Date.now() } : u);
      }

      newPassword = '';
      confirmPassword = '';
      avatarFile = null;
      avatarPreview = null;
      success = 'Сохранено';
    } catch (err) {
      error = err instanceof Error ? err.message : 'Ошибка сохранения';
    } finally {
      saving = false;
    }
  }

  // Close only if mousedown started on the backdrop itself
  let backdropMousedown = false;

  function handleBackdropMousedown(e: MouseEvent) {
    backdropMousedown = (e.target as HTMLElement).classList.contains('modal-backdrop');
  }

  function handleBackdropClick(e: MouseEvent) {
    if (backdropMousedown && (e.target as HTMLElement).classList.contains('modal-backdrop')) {
      onclose();
    }
    backdropMousedown = false;
  }
</script>

<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<div
  class="modal-backdrop"
  role="presentation"
  onmousedown={handleBackdropMousedown}
  onclick={handleBackdropClick}
>
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
      <!-- Avatar -->
      <label class="avatar-wrap" title="Нажмите для загрузки аватарки">
        <div class="avatar-preview" style:background={effectiveColor}>
          <svg class="avatar-fallback" width="32" height="32" viewBox="0 0 24 24" fill="none" aria-hidden="true">
            <circle cx="12" cy="8" r="4" stroke="#fff" stroke-width="1.5"/>
            <path d="M4 20c0-4 3.6-7 8-7s8 3 8 7" stroke="#fff" stroke-width="1.5" stroke-linecap="round"/>
          </svg>
          {#if avatarPreview}
            <img src={avatarPreview} alt="preview" />
          {:else if $currentUser?.avatar_url}
            <img src={$currentUser.avatar_url} alt={$currentUser.display_name} onerror={(e) => { (e.target as HTMLImageElement).style.display = 'none'; }} />
          {/if}
        </div>
        <span class="avatar-hint">Загрузить фото</span>
        <input type="file" accept="image/*" class="file-hidden" onchange={handleAvatarChange} />
      </label>

      <div class="field">
        <label for="username">Логин</label>
        <input id="username" type="text" bind:value={username} autocomplete="username" required />
      </div>

      <div class="field">
        <label for="display-name">Имя</label>
        <input id="display-name" type="text" bind:value={displayName} required />
      </div>

      <div class="field">
        <label for="color-pick">Цвет</label>
        <div class="color-row">
          <input id="color-pick" type="color" bind:value={color} class="color-input" />
          <span class="color-val">{color || effectiveColor}</span>
        </div>
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
    max-height: 80vh;
    overflow-y: auto;
  }

  /* Avatar upload */
  .avatar-wrap {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 6px;
    cursor: pointer;
    align-self: center;
  }

  .avatar-preview {
    width: 72px;
    height: 72px;
    border-radius: 50%;
    border: 2px solid #2a4f73;
    overflow: hidden;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: filter 0.15s;
    position: relative;
  }

  .avatar-wrap:hover .avatar-preview {
    filter: brightness(0.75);
  }

  .avatar-fallback {
    position: absolute;
  }

  .avatar-preview img {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .avatar-hint {
    font-size: 0.72rem;
    color: #4a6280;
  }

  .file-hidden {
    display: none;
  }

  /* Color row */
  .color-row {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .color-input {
    width: 40px;
    height: 32px;
    border: 1px solid #1f3a57;
    border-radius: 6px;
    background: #0d1b2a;
    cursor: pointer;
    padding: 2px;
  }

  .color-val {
    font-size: 0.8rem;
    color: #6b8aab;
    font-family: monospace;
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

  .field input[type="text"],
  .field input[type="password"] {
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
