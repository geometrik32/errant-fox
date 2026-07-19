<script lang="ts">
  import { currentUser, fighters } from '../../stores';
  import { patchMe, uploadMyAvatar, getVkConfig, unlinkVk } from '../api/auth';
  import { getFighters } from '../api/fighters';
  import { resolveColor } from '../api/types';
  import { generateCodeVerifier, generateCodeChallenge } from '$lib/utils/pkce';

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
  let vkAppId = $state<string | null>(null);

  $effect(() => {
    getVkConfig().then(cfg => {
      vkAppId = cfg.client_id;
    }).catch(err => {
      console.error('Не удалось загрузить конфигурацию VK в профиле:', err);
    });
  });

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

  async function handleLinkVk() {
    if (!vkAppId) {
      error = 'Конфигурация VK ID не загружена';
      return;
    }
    const redirectUri = window.location.origin;
    const codeVerifier = generateCodeVerifier();
    sessionStorage.setItem('vk_code_verifier', codeVerifier);
    
    try {
      const codeChallenge = await generateCodeChallenge(codeVerifier);
      const state = Math.random().toString(36).substring(2, 15);
      sessionStorage.setItem('vk_state', state);
      
      window.location.href = `https://id.vk.com/authorize?response_type=code&client_id=${vkAppId}&redirect_uri=${encodeURIComponent(redirectUri)}&code_challenge=${codeChallenge}&code_challenge_method=S256&state=${state}`;
    } catch (e) {
      console.error('Ошибка подготовки PKCE для VK ID привязке:', e);
      error = 'Не удалось запустить авторизацию VK ID';
    }
  }

  async function handleUnlinkVk() {
    if (!confirm('Вы уверены, что хотите отвязать аккаунт ВКонтакте?')) return;
    try {
      saving = true;
      error = '';
      success = '';
      const updated = await unlinkVk();
      currentUser.set(updated);
      success = 'Аккаунт ВКонтакте успешно отвязан';
    } catch (err) {
      error = err instanceof Error ? err.message : 'Ошибка при отвязке аккаунта';
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
      <!-- Horizontal Identity Section -->
      <div class="profile-top-row">
        <!-- Left: Avatar -->
        <label class="avatar-wrap-modern" title="Нажмите для загрузки аватарки">
          <div class="avatar-preview-modern" style:background={effectiveColor}>
            <svg class="avatar-fallback-modern" width="24" height="24" viewBox="0 0 24 24" fill="none" aria-hidden="true">
              <circle cx="12" cy="8" r="4" stroke="#fff" stroke-width="1.5"/>
              <path d="M4 20c0-4 3.6-7 8-7s8 3 8 7" stroke="#fff" stroke-width="1.5" stroke-linecap="round"/>
            </svg>
            {#if avatarPreview}
              <img src={avatarPreview} alt="preview" />
            {:else if $currentUser?.avatar_url}
              <img src={$currentUser.avatar_url} alt={$currentUser.display_name} onerror={(e) => { (e.target as HTMLImageElement).style.display = 'none'; }} />
            {/if}
            <div class="avatar-overlay">
              <span>Изм.</span>
            </div>
          </div>
          <input type="file" accept="image/*" class="file-hidden" onchange={handleAvatarChange} />
        </label>

        <!-- Right: Identity fields -->
        <div class="identity-fields">
          <div class="field">
            <label class="label-sm" for="display-name">Имя</label>
            <input class="input-glass" id="display-name" type="text" bind:value={displayName} required />
          </div>
          
          <div class="row-fields">
            <div class="field flex-grow">
              <label class="label-sm" for="username">Логин</label>
              <input class="input-glass" id="username" type="text" bind:value={username} autocomplete="username" required />
            </div>
            
            {#if $currentUser?.role === 'fighter'}
              <div class="field color-field">
                <label class="label-sm" for="color-pick">Цвет</label>
                <div class="color-picker-wrapper">
                  <input id="color-pick" type="color" bind:value={color} class="color-input-modern" />
                  <span class="color-preview-dot" style:background={color || '#fff'}></span>
                </div>
              </div>
            {/if}
          </div>
        </div>
      </div>

      <!-- Integrations Section -->
      <div class="field">
        <label class="label-sm">Интеграция</label>
        {#if $currentUser?.vk_id}
          <div class="vk-badge-row">
            <div class="vk-badge-left">
              <svg class="vk-icon-modern" viewBox="0 0 24 24" width="18" height="18" fill="currentColor">
                <path d="M15.067 2H8.933C3.511 2 2 3.511 2 8.933v6.134C2 20.489 3.511 22 8.933 22h6.134C20.489 22 22 20.489 22 15.067V8.933C22 3.511 20.489 2 15.067 2zm2.748 13.882c.162.33-.067.618-.465.618h-1.543c-.334 0-.488-.178-.57-.354 0 0-.785-1.916-1.895-3.158-.36-.36-.519-.467-.714-.467-.1 0-.246.147-.246.568v2.793c0 .334-.1.488-.382.488h-2.427c-1.543 0-3.178-.813-4.225-2.29-1.537-2.176-2.164-5.367-2.164-5.367 0-.2.08-.488.413-.488h1.544c.298 0 .408.136.48.354 0 0 .788 2.052 1.848 3.517.333.333.483.438.618.438.067 0 .167-.105.167-.406v-3.344c0-.334-.1-.488-.38-.488h-.795c-.135 0-.197-.083-.135-.205.258-.54 1.134-1.045 2.502-1.045h1.16c.438 0 .568.147.568.568v4.542c0 .248.045.334.148.334.067 0 .198-.086.398-.288 1.055-1.196 1.802-3.415 1.802-3.415.074-.222.18-.354.482-.354h1.542c.334 0 .426.166.334.488 0 0-.825 1.93-2.11 3.424-.36.422-.48.553-.398.665.067.1.3.33.9 1.196 1.1 1.574 1.905 2.87 1.905 2.87z"/>
              </svg>
              <span class="vk-connected-label">ВКонтакте подключен</span>
            </div>
            <button type="button" class="vk-unlink-btn" onclick={handleUnlinkVk} disabled={saving}>
              Отвязать
            </button>
          </div>
          <span class="hint-vk">Уведомления включены. Чтобы отключить, напишите боту в ЛС.</span>
        {:else}
          <button type="button" class="btn-vk-link" onclick={handleLinkVk} disabled={saving}>
            <svg class="vk-icon-modern" viewBox="0 0 24 24" width="16" height="16" fill="currentColor">
              <path d="M15.067 2H8.933C3.511 2 2 3.511 2 8.933v6.134C2 20.489 3.511 22 8.933 22h6.134C20.489 22 22 20.489 22 15.067V8.933C22 3.511 20.489 2 15.067 2zm2.748 13.882c.162.33-.067.618-.465.618h-1.543c-.334 0-.488-.178-.57-.354 0 0-.785-1.916-1.895-3.158-.36-.36-.519-.467-.714-.467-.1 0-.246.147-.246.568v2.793c0 .334-.1.488-.382.488h-2.427c-1.543 0-3.178-.813-4.225-2.29-1.537-2.176-2.164-5.367-2.164-5.367 0-.2.08-.488.413-.488h1.544c.298 0 .408.136.48.354 0 0 .788 2.052 1.848 3.517.333.333.483.438.618.438.067 0 .167-.105.167-.406v-3.344c0-.334-.1-.488-.38-.488h-.795c-.135 0-.197-.083-.135-.205.258-.54 1.134-1.045 2.502-1.045h1.16c.438 0 .568.147.568.568v4.542c0 .248.045.334.148.334.067 0 .198-.086.398-.288 1.055-1.196 1.802-3.415 1.802-3.415.074-.222.18-.354.482-.354h1.542c.334 0 .426.166.334.488 0 0-.825 1.93-2.11 3.424-.36.422-.48.553-.398.665.067.1.3.33.9 1.196 1.1 1.574 1.905 2.87 1.905 2.87z"/>
            </svg>
            Подключить ВКонтакте
          </button>
        {/if}
      </div>

      <div class="divider"></div>

      <!-- Security / Password row (horizontal) -->
      <div class="row-fields">
        <div class="field flex-grow">
          <label class="label-sm" for="new-password">Новый пароль</label>
          <input class="input-glass" id="new-password" type="password" bind:value={newPassword} autocomplete="new-password" placeholder="Оставьте пустым" />
        </div>

        <div class="field flex-grow">
          <label class="label-sm" for="confirm-password">Подтверждение</label>
          <input class="input-glass" id="confirm-password" type="password" bind:value={confirmPassword} autocomplete="new-password" placeholder="Повторите" />
        </div>
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
    background: var(--surface-solid);
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
    padding: 20px 24px 14px;
    border-bottom: 1px solid var(--border-color);
  }

  .modal-header h2 {
    font-size: 1.15rem;
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
    padding: 20px 24px 24px;
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  /* Identity top row */
  .profile-top-row {
    display: flex;
    gap: 20px;
    align-items: center;
  }

  .identity-fields {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  /* Modern Avatar Pick */
  .avatar-wrap-modern {
    cursor: pointer;
    position: relative;
    flex-shrink: 0;
  }

  .avatar-preview-modern {
    width: 80px;
    height: 80px;
    border-radius: 50%;
    border: 2px solid var(--border-color);
    overflow: hidden;
    display: flex;
    align-items: center;
    justify-content: center;
    position: relative;
    box-shadow: var(--shadow-md);
  }

  .avatar-fallback-modern {
    opacity: 0.85;
  }

  .avatar-preview-modern img {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .avatar-overlay {
    position: absolute;
    inset: 0;
    background: rgba(0, 0, 0, 0.45);
    color: white;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 0.72rem;
    font-weight: 600;
    opacity: 0;
    transition: opacity 0.2s;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .avatar-wrap-modern:hover .avatar-overlay {
    opacity: 1;
  }

  .file-hidden {
    display: none;
  }

  /* Row layouts */
  .row-fields {
    display: flex;
    gap: 12px;
    width: 100%;
  }

  .flex-grow {
    flex: 1;
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 4px;
    min-width: 0;
  }

  .label-sm {
    font-size: 0.72rem;
    color: var(--text-secondary);
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  /* Color Pick Modern styling */
  .color-field {
    flex-shrink: 0;
    width: 60px;
  }

  .color-picker-wrapper {
    position: relative;
    width: 38px;
    height: 38px;
    cursor: pointer;
  }

  .color-input-modern {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    opacity: 0;
    cursor: pointer;
    z-index: 2;
  }

  .color-preview-dot {
    position: absolute;
    inset: 0;
    border-radius: 50%;
    border: 2px solid var(--border-color);
    box-shadow: var(--shadow-sm);
    pointer-events: none;
    z-index: 1;
    transition: transform 0.15s;
  }

  .color-picker-wrapper:hover .color-preview-dot {
    transform: scale(1.08);
  }

  /* VK Integration Modern badge */
  .vk-badge-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    background: rgba(255, 255, 255, 0.02);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-md);
    padding: 10px 14px;
    gap: 12px;
  }

  .vk-badge-left {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .vk-icon-modern {
    color: #0077FF;
    flex-shrink: 0;
  }

  .vk-connected-label {
    font-size: 0.88rem;
    font-weight: 500;
    color: var(--text-primary);
  }

  .vk-unlink-btn {
    background: transparent;
    border: none;
    font-size: 0.8rem;
    font-weight: 600;
    color: #ef4444;
    cursor: pointer;
    padding: 4px 8px;
    transition: opacity 0.2s;
  }
  .vk-unlink-btn:hover {
    opacity: 0.8;
  }

  .btn-vk-link {
    background: #0077FF;
    color: white !important;
    border: none;
    border-radius: var(--radius-md);
    padding: 10px 16px;
    font-size: 0.85rem;
    font-weight: 600;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    width: 100%;
    cursor: pointer;
    box-shadow: var(--shadow-sm);
    transition: opacity 0.15s;
  }
  .btn-vk-link:hover {
    opacity: 0.9;
  }

  .hint-vk {
    font-size: 0.72rem;
    color: var(--text-secondary);
    opacity: 0.75;
    line-height: 1.35;
    margin-top: -2px;
  }

  .divider {
    height: 1px;
    background: var(--border-color);
    margin: 4px 0;
  }

  .msg {
    font-size: 0.86rem;
    padding: 10px 14px;
    border-radius: var(--radius-sm);
    margin: 0;
  }

  .error {
    color: #ef4444;
    background: rgba(239, 68, 68, 0.08);
    border: 1px solid rgba(239, 68, 68, 0.15);
  }

  .success {
    color: #10b981;
    background: rgba(16, 185, 129, 0.08);
    border: 1px solid rgba(16, 185, 129, 0.15);
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
    .row-fields {
      flex-direction: column;
      gap: 12px;
    }
    .color-field {
      width: 100%;
    }
  }
</style>
