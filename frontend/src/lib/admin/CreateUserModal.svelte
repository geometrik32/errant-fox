<script lang="ts">
  import { fighters } from '../../stores';
  import { createUser, patchUser, deleteUser, uploadUserAvatar } from '../api/fighters';
  import { resolveColor } from '../api/types';
  import type { Fighter } from '../api/types';

  interface Props {
    onclose?: () => void;
  }

  let { onclose }: Props = $props();

  // ── CREATE form ──────────────────────────────────────────────────────────────

  let newUsername    = $state('');
  let newDisplayName = $state('');
  let newPassword    = $state('');
  let newColor       = $state('#DB841F');
  let newIsAdmin     = $state(false);
  let newAvatarFile  = $state<File | null>(null);
  let newAvatarPrev  = $state<string | null>(null);
  let creating       = $state(false);
  let createError    = $state('');

  let canCreate = $derived(
    newUsername.trim().length > 0 &&
    newDisplayName.trim().length > 0 &&
    newPassword.length > 0 &&
    !creating
  );

  function handleNewAvatar(e: Event) {
    const file = (e.target as HTMLInputElement).files?.[0] ?? null;
    newAvatarFile = file;
    if (file) {
      const r = new FileReader();
      r.onload = (ev) => { newAvatarPrev = ev.target?.result as string; };
      r.readAsDataURL(file);
    } else {
      newAvatarPrev = null;
    }
  }

  async function handleCreate() {
    if (!canCreate) return;
    creating = true;
    createError = '';
    try {
      const created = await createUser({
        username: newUsername.trim(),
        display_name: newDisplayName.trim(),
        password: newPassword,
        is_admin: newIsAdmin,
        color: newColor,
      });
      if (newAvatarFile) {
        await uploadUserAvatar(created.id, newAvatarFile);
      }
      fighters.update(list => [...list, { ...created, is_admin: newIsAdmin }]);
      newUsername = ''; newDisplayName = ''; newPassword = '';
      newColor = '#DB841F'; newIsAdmin = false;
      newAvatarFile = null; newAvatarPrev = null;
    } catch (e) {
      createError = e instanceof Error ? e.message : 'Ошибка при создании';
    } finally {
      creating = false;
    }
  }

  // ── EDIT inline ──────────────────────────────────────────────────────────────

  let editingId      = $state<string | null>(null);
  let editName       = $state('');
  let editColor      = $state('');
  let editPassword   = $state('');
  let editIsAdmin    = $state(false);
  let editAvatarFile = $state<File | null>(null);
  let editAvatarPrev = $state<string | null>(null);
  let saving         = $state(false);
  let editError      = $state('');

  function startEdit(f: Fighter) {
    editingId      = f.id;
    editName       = f.display_name;
    editColor      = f.color ?? resolveColor(f.id, null);
    editPassword   = '';
    editIsAdmin    = f.is_admin;
    editAvatarFile = null;
    editAvatarPrev = null;
    editError      = '';
  }

  function cancelEdit() {
    editingId = null;
  }

  function handleEditAvatar(e: Event) {
    const file = (e.target as HTMLInputElement).files?.[0] ?? null;
    editAvatarFile = file;
    if (file) {
      const r = new FileReader();
      r.onload = (ev) => { editAvatarPrev = ev.target?.result as string; };
      r.readAsDataURL(file);
    } else {
      editAvatarPrev = null;
    }
  }

  async function handleSaveEdit(f: Fighter) {
    saving = true;
    editError = '';
    try {
      const data: { display_name?: string; password?: string; color?: string; is_admin?: boolean } = {};
      if (editName !== f.display_name) data.display_name = editName;
      if (editPassword) data.password = editPassword;
      if (editColor !== (f.color ?? '')) data.color = editColor;
      if (editIsAdmin !== f.is_admin) data.is_admin = editIsAdmin;

      let updated = f;
      if (Object.keys(data).length > 0) {
        const result = await patchUser(f.id, data);
        updated = { ...f, ...result };
      }
      if (editAvatarFile) {
        await uploadUserAvatar(f.id, editAvatarFile);
        updated = { ...updated, avatar_url: updated.avatar_url + '?t=' + Date.now() };
      }
      fighters.update(list => list.map(u => u.id === f.id ? updated : u));
      editingId = null;
    } catch (e) {
      editError = e instanceof Error ? e.message : 'Ошибка при сохранении';
    } finally {
      saving = false;
    }
  }

  async function handleDelete(f: Fighter) {
    if (!confirm(`Удалить бойца «${f.display_name}»?`)) return;
    try {
      await deleteUser(f.id);
      fighters.update(list => list.filter(u => u.id !== f.id));
      if (editingId === f.id) editingId = null;
    } catch (e) {
      alert(e instanceof Error ? e.message : 'Ошибка при удалении');
    }
  }

  // ── Backdrop close ──────────────────────────────────────────────────────────
  let backdropDown = false;

  function onBackdropMousedown(e: MouseEvent) {
    backdropDown = (e.target as HTMLElement).classList.contains('backdrop');
  }

  function onBackdropClick(e: MouseEvent) {
    if (backdropDown && (e.target as HTMLElement).classList.contains('backdrop')) {
      onclose?.();
    }
    backdropDown = false;
  }
</script>

<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<div
  class="backdrop"
  role="presentation"
  onmousedown={onBackdropMousedown}
  onclick={onBackdropClick}
>
  <div class="modal" role="dialog" aria-modal="true" aria-label="Управление бойцами">

    <div class="modal-header">
      <h2>Управление бойцами</h2>
      <button class="close-btn" onclick={onclose} aria-label="Закрыть">
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none">
          <path d="M18 6L6 18M6 6l12 12" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
        </svg>
      </button>
    </div>

    <div class="modal-body">

      <!-- ── Create section ─────────────────────────────────────────────── -->
      <section class="section">
        <h3 class="section-title">Создать бойца</h3>

        <!-- Avatar preview -->
        <label class="avatar-pick" title="Загрузить аватар">
          <div class="avatar-circle" style:background={newColor}>
            {#if newAvatarPrev}
              <img src={newAvatarPrev} alt="preview" />
            {:else}
              <svg width="24" height="24" viewBox="0 0 24 24" fill="none">
                <circle cx="12" cy="8" r="4" stroke="#fff" stroke-width="1.5"/>
                <path d="M4 20c0-4 3.6-7 8-7s8 3 8 7" stroke="#fff" stroke-width="1.5" stroke-linecap="round"/>
              </svg>
            {/if}
          </div>
          <input type="file" accept="image/*" class="file-hidden" onchange={handleNewAvatar} />
        </label>

        <div class="form-grid">
          <div class="field">
            <label class="label" for="new-username">Логин</label>
            <input id="new-username" class="input-glass" type="text" bind:value={newUsername} placeholder="ivan" autocomplete="off" />
          </div>

          <div class="field">
            <label class="label" for="new-display">Имя</label>
            <input id="new-display" class="input-glass" type="text" bind:value={newDisplayName} placeholder="Иван" autocomplete="off" />
          </div>

          <div class="field">
            <label class="label" for="new-pw">Пароль</label>
            <input id="new-pw" class="input-glass" type="password" bind:value={newPassword} autocomplete="new-password" />
          </div>

          <div class="field">
            <label class="label" for="new-color">Цвет</label>
            <div class="color-row">
              <input id="new-color" type="color" class="color-input" bind:value={newColor} />
              <span class="color-val">{newColor}</span>
            </div>
          </div>
        </div>

        <label class="checkbox-row">
          <input type="checkbox" bind:checked={newIsAdmin} />
          <span>Администратор</span>
        </label>

        {#if createError}
          <p class="error">{createError}</p>
        {/if}

        <div class="row-end">
          <button class="btn btn-primary" onclick={handleCreate} disabled={!canCreate}>
            {creating ? 'Создание…' : 'Создать'}
          </button>
        </div>
      </section>

      <div class="divider"></div>

      <!-- ── Existing users ────────────────────────────────────────────── -->
      <section class="section">
        <h3 class="section-title">Все бойцы</h3>

        {#each $fighters as f (f.id)}
          <div class="user-row">

            {#if editingId === f.id}
              <!-- Inline edit form -->
              <div class="edit-form">
                <label class="avatar-pick small" title="Загрузить аватар">
                  <div class="avatar-circle small" style:background={editColor}>
                    {#if editAvatarPrev}
                      <img src={editAvatarPrev} alt="preview" />
                    {:else if f.avatar_url}
                      <img src={f.avatar_url} alt={f.display_name} />
                    {:else}
                      <svg width="16" height="16" viewBox="0 0 24 24" fill="none">
                        <circle cx="12" cy="8" r="4" stroke="#fff" stroke-width="1.5"/>
                        <path d="M4 20c0-4 3.6-7 8-7s8 3 8 7" stroke="#fff" stroke-width="1.5" stroke-linecap="round"/>
                      </svg>
                    {/if}
                  </div>
                  <input type="file" accept="image/*" class="file-hidden" onchange={handleEditAvatar} />
                </label>

                <div class="edit-fields">
                  <input class="input-glass" type="text" bind:value={editName} placeholder="Имя" />
                  <input class="input-glass" type="password" bind:value={editPassword} placeholder="Новый пароль (необязательно)" autocomplete="new-password" />
                  <div class="color-row">
                    <input type="color" class="color-input" bind:value={editColor} />
                    <span class="color-val">{editColor}</span>
                    <label class="checkbox-row" style="margin-left: auto">
                      <input type="checkbox" bind:checked={editIsAdmin} />
                      <span>Админ</span>
                    </label>
                  </div>
                  {#if editError}
                    <p class="error">{editError}</p>
                  {/if}
                  <div class="edit-actions">
                    <button class="btn btn-primary btn-sm" onclick={() => handleSaveEdit(f)} disabled={saving}>
                      {saving ? '…' : 'Сохранить'}
                    </button>
                    <button class="btn btn-outline btn-sm" onclick={cancelEdit}>Отмена</button>
                  </div>
                </div>
              </div>

            {:else}
              <!-- Collapsed row -->
              <div class="user-info">
                <div class="user-dot" style:background={resolveColor(f.id, f.color)}></div>
                <div class="user-avatar-wrap">
                  <svg class="user-avatar-icon" width="14" height="14" viewBox="0 0 24 24" fill="none" aria-hidden="true">
                    <circle cx="12" cy="8" r="4" stroke="#fff" stroke-width="1.5"/>
                    <path d="M4 20c0-4 3.6-7 8-7s8 3 8 7" stroke="#fff" stroke-width="1.5" stroke-linecap="round"/>
                  </svg>
                  <img class="user-avatar-img" src={f.avatar_url} alt={f.display_name} onerror={(e) => { (e.target as HTMLImageElement).style.display = 'none'; }} />
                </div>
                <div class="user-names">
                  <span class="user-display">{f.display_name}</span>
                  <span class="user-login">@{f.username}{f.is_admin ? ' · Адм' : ''}</span>
                </div>
              </div>
              <div class="user-btns">
                <button class="btn-icon" onclick={() => startEdit(f)} title="Редактировать">
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none">
                    <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
                    <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
                  </svg>
                </button>
                <button class="btn-icon danger" onclick={() => handleDelete(f)} title="Удалить">
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none">
                    <polyline points="3 6 5 6 21 6" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
                    <path d="M19 6l-1 14H6L5 6" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
                    <path d="M10 11v6M14 11v6" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
                    <path d="M9 6V4h6v2" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
                  </svg>
                </button>
              </div>
            {/if}

          </div>
        {/each}

        {#if $fighters.length === 0}
          <p class="empty">Нет бойцов</p>
        {/if}
      </section>

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
    background: var(--surface);
    backdrop-filter: var(--glass-blur);
    -webkit-backdrop-filter: var(--glass-blur);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-lg);
    width: 480px;
    max-width: calc(100vw - 24px);
    max-height: 88vh;
    display: flex;
    flex-direction: column;
    box-shadow: var(--shadow-lg);
    overflow: hidden;
  }

  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 24px 28px 16px;
    border-bottom: 1px solid var(--border-color);
    flex-shrink: 0;
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
    padding: 24px 28px;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 0;
  }

  .section {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .section-title {
    font-size: 0.8rem;
    font-weight: 600;
    letter-spacing: 0.09em;
    text-transform: uppercase;
    color: var(--text-secondary);
    margin: 0;
  }

  .divider {
    height: 1px;
    background: var(--border-color);
    margin: 24px 0;
  }

  /* Avatar pick */
  .avatar-pick {
    cursor: pointer;
    align-self: center;
  }

  .avatar-circle {
    width: 64px;
    height: 64px;
    border-radius: 50%;
    border: 2px solid var(--border-color);
    overflow: hidden;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: filter 0.15s;
    box-shadow: var(--shadow-md);
  }

  .avatar-pick:hover .avatar-circle { filter: brightness(0.75); }

  .avatar-circle.small {
    width: 36px;
    height: 36px;
    flex-shrink: 0;
  }

  .avatar-circle img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .avatar-pick.small { align-self: flex-start; }

  .file-hidden { display: none; }

  /* Form grid */
  .form-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 10px;
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .label {
    font-size: 0.75rem;
    color: var(--text-secondary);
    font-weight: 500;
  }

  .input {
    background: var(--surface-solid);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm);
    color: var(--text-primary);
    font-size: 0.875rem;
    padding: 7px 10px;
    outline: none;
    width: 100%;
    transition: var(--transition);
    box-sizing: border-box;
  }

  .input:focus { border-color: var(--accent-yellow); }

  .color-row {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .color-input {
    width: 40px;
    height: 32px;
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm);
    background: transparent;
    cursor: pointer;
    padding: 2px;
    flex-shrink: 0;
  }

  .color-val {
    font-size: 0.85rem;
    color: var(--text-secondary);
    font-family: monospace;
  }

  .checkbox-row {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 0.83rem;
    color: var(--text-secondary);
    cursor: pointer;
  }

  .checkbox-row input[type='checkbox'] {
    accent-color: #DB841F;
    cursor: pointer;
  }

  .row-end { display: flex; justify-content: flex-end; }

  .error {
    font-size: 0.8rem;
    color: #e05252;
    margin: 0;
  }

  .empty {
    font-size: 0.83rem;
    color: var(--text-secondary);
  }

  .user-row {
    display: flex;
    align-items: center;
    gap: 0;
    background: var(--surface-hover);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-md);
    overflow: hidden;
  }

  .user-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .user-avatar-wrap {
    width: 28px;
    height: 28px;
    border-radius: 50%;
    background: var(--surface-solid);
    border: 1px solid var(--border-color);
    display: flex;
    align-items: center;
    justify-content: center;
    position: relative;
    flex-shrink: 0;
    overflow: hidden;
  }

  .user-avatar-icon {
    position: absolute;
    pointer-events: none;
  }

  .user-avatar-img {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .user-names {
    display: flex;
    flex-direction: column;
    min-width: 0;
  }

  .user-display {
    font-size: 0.83rem;
    color: var(--text-primary);
    font-weight: 500;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .user-login {
    font-size: 0.72rem;
    color: var(--text-secondary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .user-btns {
    display: flex;
    align-items: center;
    gap: 2px;
    padding: 0 8px;
    flex-shrink: 0;
  }

  .btn-icon {
    width: 28px;
    height: 28px;
    border: none;
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--text-secondary);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: var(--transition);
  }

  .btn-icon:hover { background: var(--surface-hover); color: var(--text-primary); }
  .btn-icon.danger:hover { background: rgba(239,68,68,0.12); color: #ef4444; }

  /* Edit form */
  .edit-form {
    display: flex;
    align-items: flex-start;
    gap: 10px;
    padding: 10px;
    width: 100%;
  }

  .edit-fields {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 8px;
    min-width: 0;
  }

  .edit-actions {
    display: flex;
    gap: 6px;
  }

  /* Buttons */
  .btn-sm { padding: 6px 12px; font-size: 0.85rem; }

  @media (max-width: 768px) {
    .modal {
      border-radius: var(--radius-lg) var(--radius-lg) 0 0;
      margin-top: auto;
      max-width: 100vw;
    }
    .form-grid {
      grid-template-columns: 1fr;
    }
  }
</style>
