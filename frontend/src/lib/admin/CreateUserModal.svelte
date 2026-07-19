<script lang="ts">
  import { fighters } from '../../stores';
  import { createUser, patchUser, deleteUser, uploadUserAvatar, getAdminUsers, getFighters } from '../api/fighters';
  import { resolveColor } from '../api/types';
  import type { Fighter } from '../api/types';

  interface Props {
    onclose?: () => void;
  }

  let { onclose }: Props = $props();

  // ── LOAD USERS ──────────────────────────────────────────────────────────────
  let allUsers = $state<Fighter[]>([]);
  let loadingUsers = $state(false);

  async function loadUsers() {
    loadingUsers = true;
    try {
      allUsers = await getAdminUsers();
    } catch (e) {
      console.error('Ошибка загрузки пользователей:', e);
    } finally {
      loadingUsers = false;
    }
  }

  async function refreshFightersStore() {
    try {
      const list = await getFighters();
      fighters.set(list);
    } catch (e) {
      console.error('Ошибка обновления списка бойцов:', e);
    }
  }

  $effect(() => {
    loadUsers();
  });

  // ── TABS ──
  let activeTab = $state<'fighters' | 'guests'>('fighters');
  let showCreateForm = $state(false);

  let fightersList = $derived(allUsers.filter(u => u.role === 'fighter' || u.role === 'retired'));
  let guestsList = $derived(allUsers.filter(u => u.role === 'guest'));

  // ── CREATE form ──────────────────────────────────────────────────────────────
  let newUsername    = $state('');
  let newDisplayName = $state('');
  let newPassword    = $state('');
  let newColor       = $state('#DB841F');
  let newIsAdmin     = $state(false);
  let newRole        = $state('fighter'); // 'fighter' or 'guest'
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
        is_admin: newRole === 'fighter' ? newIsAdmin : false,
        color: newRole === 'fighter' ? newColor : '#9E9E9E',
        role: newRole,
      });
      if (newRole === 'fighter' && newAvatarFile) {
        await uploadUserAvatar(created.id, newAvatarFile);
      }
      await loadUsers();
      await refreshFightersStore();
      
      newUsername = ''; newDisplayName = ''; newPassword = '';
      newColor = '#DB841F'; newIsAdmin = false;
      newAvatarFile = null; newAvatarPrev = null;
      showCreateForm = false;
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
  let editVkId       = $state('');
  let editPassword   = $state('');
  let editIsAdmin    = $state(false);
  let editRole       = $state('fighter');
  let editAvatarFile = $state<File | null>(null);
  let editAvatarPrev = $state<string | null>(null);
  let saving         = $state(false);
  let editError      = $state('');

  function startEdit(f: Fighter) {
    editingId      = f.id;
    editName       = f.display_name;
    editColor      = f.color ?? resolveColor(f.id, null);
    editVkId       = f.vk_id ?? '';
    editPassword   = '';
    editIsAdmin    = f.is_admin;
    editRole       = f.role;
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
      const data: { display_name?: string; password?: string; color?: string; is_admin?: boolean; vk_id?: string; role?: string } = {};
      if (editName !== f.display_name) data.display_name = editName;
      if (editPassword) data.password = editPassword;
      if (editRole !== f.role) data.role = editRole;

      // Save color/avatar/admin if user is fighter or retired
      if (editRole === 'fighter' || editRole === 'retired') {
        if (editColor !== (f.color ?? '')) data.color = editColor;
        if (editIsAdmin !== f.is_admin) data.is_admin = editIsAdmin;
      } else {
        // If changed to guest, strip admin rights and set fallback color
        if (f.is_admin) data.is_admin = false;
        data.color = '#9E9E9E';
      }

      if (editVkId !== (f.vk_id ?? '')) data.vk_id = editVkId;

      if (Object.keys(data).length > 0 || ((editRole === 'fighter' || editRole === 'retired') && editAvatarFile)) {
        if (Object.keys(data).length > 0) {
          await patchUser(f.id, data);
        }
        if ((editRole === 'fighter' || editRole === 'retired') && editAvatarFile) {
          await uploadUserAvatar(f.id, editAvatarFile);
        }
        await loadUsers();
        await refreshFightersStore();
      }
      editingId = null;
    } catch (e) {
      editError = e instanceof Error ? e.message : 'Ошибка при сохранении';
    } finally {
      saving = false;
    }
  }

  async function handleDelete(f: Fighter) {
    if (!confirm(`Точно ли вы хотите удалить пользователя «${f.display_name}»? Все метаданные этого пользователя будут очищены из базы данных.`)) return;
    try {
      await deleteUser(f.id);
      await loadUsers();
      await refreshFightersStore();
      if (editingId === f.id) editingId = null;
    } catch (e) {
      alert(e instanceof Error ? e.message : 'Ошибка при удалении');
    }
  }

  // ── Backdrop close ──────────────────────────────────────────────────────────
  let backdropDown = false;

  function onBackdropMousedown(e: MouseEvent) {
    backdropDown = (e.target as HTMLElement).classList.contains('modal-backdrop');
  }

  function onBackdropClick(e: MouseEvent) {
    if (backdropDown && (e.target as HTMLElement).classList.contains('modal-backdrop')) {
      onclose?.();
    }
    backdropDown = false;
  }
</script>

<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<div
  class="modal-backdrop"
  role="presentation"
  onmousedown={onBackdropMousedown}
  onclick={onBackdropClick}
>
  <div class="modal" role="dialog" aria-modal="true" aria-label="Управление пользователями">

    <div class="modal-header">
      <h2>Управление пользователями</h2>
      <button class="close-btn" onclick={onclose} aria-label="Закрыть">
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none">
          <path d="M18 6L6 18M6 6l12 12" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
        </svg>
      </button>
    </div>

    <!-- TABS NAVIGATION -->
    <div class="tabs-header">
      <button class="tab-btn" class:active={activeTab === 'fighters'} onclick={() => { activeTab = 'fighters'; showCreateForm = false; }}>
        Бойцы ({fightersList.length})
      </button>
      <button class="tab-btn" class:active={activeTab === 'guests'} onclick={() => { activeTab = 'guests'; showCreateForm = false; }}>
        Гости ({guestsList.length})
      </button>
    </div>

    <div class="modal-body">

      <!-- ── Create section (collapsible) ─────────────────────────────────── -->
      {#if showCreateForm}
        <section class="section create-section glass-card-solid">
          <div class="row-between">
            <h3 class="section-title">Новый {newRole === 'fighter' ? 'боец' : 'гость'}</h3>
            <button class="btn-text-sm" onclick={() => showCreateForm = false}>Скрыть форму</button>
          </div>

          <div class="profile-top-row">
            {#if newRole === 'fighter'}
              <!-- Left: Avatar pick (Fighters only) -->
              <label class="avatar-wrap-modern" title="Загрузить аватар">
                <div class="avatar-preview-modern" style:background={newColor}>
                  {#if newAvatarPrev}
                    <img src={newAvatarPrev} alt="preview" />
                  {:else}
                    <svg class="avatar-fallback-modern" width="24" height="24" viewBox="0 0 24 24" fill="none">
                      <circle cx="12" cy="8" r="4" stroke="#fff" stroke-width="1.5"/>
                      <path d="M4 20c0-4 3.6-7 8-7s8 3 8 7" stroke="#fff" stroke-width="1.5" stroke-linecap="round"/>
                    </svg>
                  {/if}
                  <div class="avatar-overlay">
                    <span>Загруз.</span>
                  </div>
                </div>
                <input type="file" accept="image/*" class="file-hidden" onchange={handleNewAvatar} />
              </label>
            {/if}

            <!-- Right: fields -->
            <div class="identity-fields">
              <div class="field">
                <label class="label-sm" for="new-display">Отображаемое имя</label>
                <input id="new-display" class="input-glass" type="text" bind:value={newDisplayName} placeholder="Имя Фамилия" autocomplete="off" />
              </div>

              <div class="row-fields">
                <div class="field flex-grow">
                  <label class="label-sm" for="new-username">Логин</label>
                  <input id="new-username" class="input-glass" type="text" bind:value={newUsername} placeholder="Имя пользователя" autocomplete="off" />
                </div>

                {#if newRole === 'fighter'}
                  <div class="field color-field">
                    <label class="label-sm" for="new-color">Цвет</label>
                    <div class="color-picker-wrapper">
                      <input id="new-color" type="color" class="color-input-modern" bind:value={newColor} />
                      <span class="color-preview-dot" style:background={newColor}></span>
                    </div>
                  </div>
                {/if}
              </div>
            </div>
          </div>

          <div class="row-fields align-center">
            <div class="field flex-grow">
              <label class="label-sm" for="new-pw">Пароль</label>
              <input id="new-pw" class="input-glass" type="password" bind:value={newPassword} autocomplete="new-password" placeholder="••••••••" />
            </div>

            {#if newRole === 'fighter'}
              <label class="checkbox-row" style="margin-top: 18px; margin-left: auto;">
                <input type="checkbox" bind:checked={newIsAdmin} />
                <span>Администратор</span>
              </label>
            {/if}
          </div>

          {#if createError}
            <p class="error">{createError}</p>
          {/if}

          <div class="row-end">
            <button class="btn btn-primary btn-sm" onclick={handleCreate} disabled={!canCreate}>
              {creating ? 'Создание…' : 'Создать'}
            </button>
          </div>
        </section>
      {/if}

      <!-- ── List title with action button ────────────────────────────────── -->
      {#if !showCreateForm}
        <div class="row-between">
          <span class="section-title">Список {activeTab === 'fighters' ? 'бойцов' : 'гостей'}</span>
          <button class="btn btn-primary btn-sm btn-vk" onclick={() => { showCreateForm = true; newRole = activeTab === 'fighters' ? 'fighter' : 'guest'; }}>
            + Добавить {activeTab === 'fighters' ? 'бойца' : 'гостя'}
          </button>
        </div>
      {/if}

      <!-- ── Existing users list ────────────────────────────────────────── -->
      <section class="section list-section">
        {#if loadingUsers && allUsers.length === 0}
          <p class="empty">Загрузка пользователей...</p>
        {/if}

        {#each activeTab === 'fighters' ? fightersList : guestsList as f (f.id)}
          <div class="user-row">

            {#if editingId === f.id}
              <!-- Inline edit form (matches Profile design) -->
              <div class="edit-form-modern">
                {#if editRole === 'fighter'}
                  <!-- Left: Avatar pick (Fighters only) -->
                  <label class="avatar-wrap-modern avatar-wrap-edit" title="Загрузить аватар">
                    <div class="avatar-preview-modern avatar-preview-edit" style:background={editColor}>
                      {#if editAvatarPrev}
                        <img src={editAvatarPrev} alt="preview" />
                      {:else if f.avatar_url}
                        <img src={f.avatar_url} alt={f.display_name} />
                      {:else}
                        <svg class="avatar-fallback-modern" width="20" height="20" viewBox="0 0 24 24" fill="none">
                          <circle cx="12" cy="8" r="4" stroke="#fff" stroke-width="1.5"/>
                          <path d="M4 20c0-4 3.6-7 8-7s8 3 8 7" stroke="#fff" stroke-width="1.5" stroke-linecap="round"/>
                        </svg>
                      {/if}
                      <div class="avatar-overlay">
                        <span>Изм.</span>
                      </div>
                    </div>
                    <input type="file" accept="image/*" class="file-hidden" onchange={handleEditAvatar} />
                  </label>
                {/if}

                <!-- Right: Fields -->
                <div class="identity-fields">
                  <div class="field">
                    <label class="label-sm">Отображаемое имя</label>
                    <input class="input-glass" type="text" bind:value={editName} placeholder="Имя" />
                  </div>

                  <div class="row-fields">
                    <div class="field flex-grow">
                      <label class="label-sm">VK ID (при наличии)</label>
                      <input class="input-glass" type="text" bind:value={editVkId} placeholder="Например, 12345678" />
                    </div>
                    <div class="field flex-grow">
                      <label class="label-sm">Новый пароль</label>
                      <input class="input-glass" type="password" bind:value={editPassword} placeholder="Оставьте пустым" autocomplete="new-password" />
                    </div>
                  </div>

                  <div class="row-fields align-center">
                    <div class="field flex-grow">
                      <label class="label-sm">Роль</label>
                      <select class="input-glass select-glass-inline-modern" bind:value={editRole}>
                        <option value="fighter">Боец</option>
                        <option value="guest">Гость</option>
                        <option value="retired">На пенсии</option>
                      </select>
                    </div>

                    {#if editRole === 'fighter' || editRole === 'retired'}
                      <div class="field color-field">
                        <label class="label-sm">Цвет</label>
                        <div class="color-picker-wrapper">
                          <input type="color" class="color-input-modern" bind:value={editColor} />
                          <span class="color-preview-dot" style:background={editColor}></span>
                        </div>
                      </div>

                      <label class="checkbox-row" style="margin-top: 18px; margin-left: auto;">
                        <input type="checkbox" bind:checked={editIsAdmin} />
                        <span>Администратор</span>
                      </label>
                    {/if}
                  </div>

                  {#if editError}
                    <p class="error">{editError}</p>
                  {/if}

                  <div class="edit-actions">
                    <button class="btn btn-primary btn-sm" onclick={() => handleSaveEdit(f)} disabled={saving}>
                      {saving ? 'Сохранение…' : 'Сохранить'}
                    </button>
                    <button class="btn btn-outline btn-sm" onclick={cancelEdit}>Отмена</button>
                  </div>
                </div>
              </div>

            {:else}
              <!-- Collapsed row -->
              <div class="collapsed-row-content">
                <div class="user-info">
                  {#if f.role === 'fighter' || f.role === 'retired'}
                    <div class="user-dot" style:background={resolveColor(f.id, f.color)}></div>
                    <div class="user-avatar-wrap">
                      <svg class="user-avatar-icon" width="14" height="14" viewBox="0 0 24 24" fill="none" aria-hidden="true">
                        <circle cx="12" cy="8" r="4" stroke="#fff" stroke-width="1.5"/>
                        <path d="M4 20c0-4 3.6-7 8-7s8 3 8 7" stroke="#fff" stroke-width="1.5" stroke-linecap="round"/>
                      </svg>
                      <img class="user-avatar-img" src={f.avatar_url} alt={f.display_name} onerror={(e) => { (e.target as HTMLImageElement).style.display = 'none'; }} />
                    </div>
                  {/if}
                  <div class="user-names">
                    <span class="user-display">
                      {f.display_name}
                      {#if f.vk_id}
                        <span class="vk-badge-sm" title="Привязан ВК">VK</span>
                      {/if}
                    </span>
                    <span class="user-login">
                      @{f.username}{f.is_admin ? ' · Администратор' : ''}{f.role === 'retired' ? ' · На пенсии' : ''}
                    </span>
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
              </div>
            {/if}

          </div>
        {/each}

        {#if !loadingUsers && (activeTab === 'fighters' ? fightersList : guestsList).length === 0}
          <p class="empty">Нет зарегистрированных {activeTab === 'fighters' ? 'бойцов' : 'гостей'}</p>
        {/if}
      </section>

    </div>
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
    width: 100%;
    max-width: 520px;
    display: flex;
    flex-direction: column;
    box-shadow: var(--shadow-lg);
    overflow: hidden;
  }

  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 20px 24px 14px;
    border-bottom: 1px solid var(--border-color);
    flex-shrink: 0;
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

  /* Tabs */
  .tabs-header {
    display: flex;
    border-bottom: 1px solid var(--border-color);
    background: rgba(255, 255, 255, 0.01);
  }

  .tab-btn {
    flex: 1;
    background: transparent;
    border: none;
    border-bottom: 2px solid transparent;
    padding: 12px 16px;
    font-size: 0.88rem;
    font-weight: 600;
    color: var(--text-secondary);
    cursor: pointer;
    transition: var(--transition);
  }

  .tab-btn:hover {
    color: var(--text-primary);
    background: rgba(255, 255, 255, 0.02);
  }

  .tab-btn.active {
    color: var(--accent-blue);
    border-bottom-color: var(--accent-blue);
    background: rgba(255, 255, 255, 0.03);
  }

  .modal-body {
    padding: 20px 24px;
    display: flex;
    flex-direction: column;
    gap: 16px;
    max-height: calc(100vh - 160px);
    overflow-y: auto;
  }

  .section {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .create-section {
    padding: 16px;
    border-radius: var(--radius-md);
    background: rgba(255, 255, 255, 0.02);
    border: 1px solid var(--border-color);
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .list-section {
    max-height: 400px;
    overflow-y: auto;
  }

  .section-title {
    font-size: 0.76rem;
    font-weight: 700;
    letter-spacing: 0.09em;
    text-transform: uppercase;
    color: var(--text-secondary);
    margin: 0;
    display: flex;
    align-items: center;
  }

  /* Identity top row layout */
  .profile-top-row {
    display: flex;
    gap: 20px;
    align-items: center;
    width: 100%;
  }

  .identity-fields {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  /* Modern Avatar Pick (consitent with Profile) */
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

  .avatar-preview-modern img {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .avatar-fallback-modern {
    opacity: 0.85;
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

  .avatar-wrap-edit .avatar-preview-edit {
    width: 70px;
    height: 70px;
  }

  .file-hidden { display: none; }

  /* Row layouts */
  .row-fields {
    display: flex;
    gap: 12px;
    width: 100%;
  }

  .align-center {
    align-items: center;
  }

  .flex-grow {
    flex: 1;
    min-width: 0;
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

  .checkbox-row {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 0.82rem;
    color: var(--text-secondary);
    cursor: pointer;
    user-select: none;
  }

  .checkbox-row input[type='checkbox'] {
    accent-color: var(--accent-blue);
    cursor: pointer;
  }

  .row-end { display: flex; justify-content: flex-end; margin-top: 8px; }
  .row-between { display: flex; align-items: center; justify-content: space-between; }

  .btn-text-sm {
    background: transparent;
    border: none;
    font-size: 0.78rem;
    color: var(--text-secondary);
    cursor: pointer;
    transition: color 0.2s;
  }
  .btn-text-sm:hover {
    color: var(--text-primary);
  }

  .error {
    font-size: 0.8rem;
    color: #e05252;
    margin: 0;
  }

  .empty {
    font-size: 0.83rem;
    color: var(--text-secondary);
    text-align: center;
    padding: 20px 0;
  }

  .user-row {
    display: block;
    background: var(--surface-hover);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-md);
    margin-bottom: 6px;
  }

  .collapsed-row-content {
    display: flex;
    align-items: center;
    width: 100%;
  }

  .user-info {
    display: flex;
    align-items: center;
    gap: 10px;
    flex: 1;
    min-width: 0;
    padding: 8px 12px;
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
    display: flex;
    align-items: center;
    gap: 6px;
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

  /* Modern Inline Edit Form */
  .edit-form-modern {
    display: flex;
    align-items: flex-start;
    gap: 20px;
    padding: 16px;
    width: 100%;
    background: rgba(0, 0, 0, 0.12);
  }

  .edit-actions {
    display: flex;
    gap: 8px;
    margin-top: 4px;
  }

  /* Buttons */
  .btn-sm { padding: 6px 12px; font-size: 0.82rem; }

  .btn-vk {
    background: var(--accent-blue);
    color: white !important;
    border: none;
  }
  .btn-vk:hover {
    opacity: 0.9;
  }

  .vk-badge-sm {
    background: #0077FF;
    color: white;
    font-size: 0.6rem;
    font-weight: 700;
    padding: 1px 4px;
    border-radius: var(--radius-sm);
    text-transform: uppercase;
    vertical-align: middle;
  }

  /* Select */
  .select-glass-inline-modern {
    padding: 0 12px;
    font-size: 0.85rem;
    border-radius: var(--radius-md);
    background: rgba(255, 255, 255, 0.05);
    color: var(--text-primary);
    cursor: pointer;
    border: 1px solid var(--border-color);
    height: 38px;
    width: 100%;
  }

  .select-glass-inline-modern option {
    background: #1a1a1f;
    color: var(--text-primary);
  }

  @media (max-width: 768px) {
    .modal {
      border-radius: var(--radius-lg) var(--radius-lg) 0 0;
      margin-top: auto;
      max-width: 100vw;
    }
    .profile-top-row {
      flex-direction: column;
      gap: 12px;
    }
    .edit-form-modern {
      flex-direction: column;
    }
    .avatar-wrap-edit {
      align-self: center;
    }
  }
</style>
