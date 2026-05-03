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
        <input class="input-glass" id="username" type="text" bind:value={username} autocomplete="username" required />
      </div>

      <div class="field">
        <label for="display-name">Имя</label>
        <input class="input-glass" id="display-name" type="text" bind:value={displayName} required />
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
        <input class="input-glass" id="new-password" type="password" bind:value={newPassword} autocomplete="new-password" placeholder="Оставьте пустым, чтобы не менять" />
      </div>

      <div class="field">
        <label for="confirm-password">Подтверждение пароля</label>
        <input class="input-glass" id="confirm-password" type="password" bind:value={confirmPassword} autocomplete="new-password" />
      </div>

      {#if error}
        <p class="msg error">{error}</p>
      {/if}
      {#if success}
        <p class="msg success">{success}</p>
      {/if}

      <div class="actions">
        <button type="button" class="btn btn-outline" onclick={onclose}>Отмена</button>
        <button type="submit" class="btn btn-primary" disabled={saving}>
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
    padding: 16px;
  }

  .modal {
    background: var(--surface);
    backdrop-filter: var(--glass-blur);
    -webkit-backdrop-filter: var(--glass-blur);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-lg);
    width: 100%;
    max-width: 440px;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 24px 28px 16px;
    border-bottom: 1px solid var(--border-color);
  }

  .modal-header h2 {
    font-size: 1.2rem;
    font-weight: 700;
    color: var(--text-primary);
    margin: 0;
  }

  .close-btn {
    background: var(--surface-solid);
    border: 1px solid var(--border-color);
    box-shadow: var(--shadow-sm);
    color: var(--text-secondary);
    cursor: pointer;
    padding: 6px;
    border-radius: 50%;
    display: flex;
    transition: var(--transition);
  }

  .close-btn:hover {
    color: var(--text-primary);
    transform: scale(1.05);
  }

  .modal-body {
    padding: 24px 28px 28px;
    display: flex;
    flex-direction: column;
    gap: 20px;
    max-height: calc(100vh - 100px);
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
    width: 80px;
    height: 80px;
    border-radius: 50%;
    border: 2px solid var(--border-color);
    box-shadow: var(--shadow-md);
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
    font-size: 0.8rem;
    color: var(--text-secondary);
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
    width: 32px;
    height: 32px;
    border: 1px solid var(--border-color);
    border-radius: 50%;
    padding: 0;
    overflow: hidden;
    cursor: pointer;
    background: transparent;
  }

  .color-input::-webkit-color-swatch-wrapper {
    padding: 0;
  }

  .color-input::-webkit-color-swatch {
    border: none;
    border-radius: 50%;
  }

  .color-val {
    font-size: 0.9rem;
    color: var(--text-secondary);
    font-family: monospace;
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .field label {
    font-size: 0.85rem;
    font-weight: 500;
    color: var(--text-secondary);
  }

  .field input:disabled { color: var(--text-secondary); cursor: not-allowed; }

  .divider {
    height: 1px;
    background: var(--border-color);
    margin: 8px 0;
  }

  .msg {
    font-size: 0.9rem;
    padding: 10px 14px;
    border-radius: var(--radius-sm);
    margin: 0;
  }

  .error {
    color: #ef4444;
    background: rgba(239, 68, 68, 0.1);
    border: 1px solid rgba(239, 68, 68, 0.2);
  }

  .success {
    color: #10b981;
    background: rgba(16, 185, 129, 0.1);
    border: 1px solid rgba(16, 185, 129, 0.2);
  }

  .actions {
    display: flex;
    gap: 12px;
    justify-content: flex-end;
    margin-top: 8px;
  }

  @media (max-width: 768px) {
    .modal {
      border-radius: var(--radius-lg) var(--radius-lg) 0 0;
      margin-top: auto;
    }
    .modal-backdrop {
      padding: 0;
      align-items: flex-end;
    }
    .actions {
      flex-direction: column-reverse;
    }
    .actions button {
      width: 100%;
    }
  }
</style>
