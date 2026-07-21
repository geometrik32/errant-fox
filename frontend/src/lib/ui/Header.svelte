<script lang="ts">
  import { token, currentUser, gallerySidebarOpen } from '../../stores';
  import { resolveColor } from '../api/types';
  import CreateUserModal from '../admin/CreateUserModal.svelte';
  import TechniquesModal from '../admin/TechniquesModal.svelte';
  import ProfileModal from './ProfileModal.svelte';
  import HotkeysModal from './HotkeysModal.svelte';
  import SearchPanel from './SearchPanel.svelte';
  import SyncModal from './SyncModal.svelte';
  import BatchAiModal from './BatchAiModal.svelte';

  interface Props {
    hash: string;
  }

  let { hash }: Props = $props();

  let dropdownOpen = $state(false);
  let showCreateUser = $state(false);
  let showTechniques = $state(false);
  let showProfile = $state(false);
  let showHotkeys = $state(false);
  let showSyncDatabase = $state(false);
  let showBatchAi = $state(false);
  let batchAiMode = $state<'new' | 'relabel'>('new');

  let activeNav = $derived(
    hash === '#/stats' ? 'stats' : hash === '#/search' ? 'search' : 'gallery'
  );

  let isGalleryRoute = $derived(
    hash === '#/gallery' || hash === '' || hash === '#'
  );

  let showSearch = $state(false);
  let searchQuery = $state('');

  $effect(() => {
    showSearch = hash === '#/search';
  });

  function toggleSearch() {
    if (hash === '#/search') {
      window.location.hash = '#/gallery';
    } else {
      window.location.hash = '#/search';
    }
  }

  function navigate(path: string) {
    window.location.hash = path;
  }

  function logout() {
    dropdownOpen = false;
    token.set(null);
    currentUser.set(null);
    window.location.hash = '#/gallery';
  }

  function handleClickOutside(e: MouseEvent) {
    const target = e.target as HTMLElement;
    if (!target.closest('.user-menu')) {
      dropdownOpen = false;
    }
  }



  $effect(() => {
    if (dropdownOpen) {
      document.addEventListener('click', handleClickOutside);
      return () => document.removeEventListener('click', handleClickOutside);
    }
  });

  let avatarColor = $derived(resolveColor($currentUser?.id ?? '', $currentUser?.color ?? null));
</script>

<header class="header">
  <!-- Left: logo -->
  <button class="logo" onclick={() => navigate('#/gallery')}>
    <img src="/logo.png" alt="" class="logo-img" aria-hidden="true" />
    <span>Errant Fox</span>
  </button>

  <!-- Filter toggle (gallery only) -->
  {#if isGalleryRoute}
    <button
      class="filter-toggle"
      class:active={$gallerySidebarOpen}
      onclick={() => gallerySidebarOpen.update(v => !v)}
      aria-label="Фильтры"
      title={$gallerySidebarOpen ? 'Скрыть фильтры' : 'Показать фильтры'}
    >
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
        <polygon points="22 3 2 3 10 12.46 10 19 14 21 14 12.46 22 3"></polygon>
      </svg>
    </button>
  {/if}

  <!-- Center: nav (absolutely centered) -->
  {#if $currentUser?.role !== 'guest'}
    <nav class="nav">
      <button
        class="nav-btn"
        class:active={activeNav === 'gallery'}
        onclick={() => { navigate('#/gallery'); showSearch = false; }}
      >
        Видео
      </button>
      <button
        class="nav-btn"
        class:active={activeNav === 'stats'}
        onclick={() => { navigate('#/stats'); showSearch = false; }}
      >
        Бойцы
      </button>
      <button
        class="nav-btn"
        class:active={activeNav === 'search'}
        onclick={toggleSearch}
      >
        Поиск
      </button>
    </nav>
  {/if}

  <!-- Right: user menu -->
  <div class="user-menu">
    <button
      class="menu-trigger"
      onclick={(e) => { e.stopPropagation(); dropdownOpen = !dropdownOpen; }}
      aria-label="Меню пользователя"
      aria-expanded={dropdownOpen}
    >
      <div class="avatar" style:background={avatarColor}>
        <svg class="avatar-icon" width="16" height="16" viewBox="0 0 24 24" fill="none" aria-hidden="true">
          <circle cx="12" cy="8" r="4" stroke="#fff" stroke-width="1.5"/>
          <path d="M4 20c0-4 3.6-7 8-7s8 3 8 7" stroke="#fff" stroke-width="1.5" stroke-linecap="round"/>
        </svg>
        <img src={$currentUser?.avatar_url} alt={$currentUser?.display_name} onerror={(e) => { (e.target as HTMLImageElement).style.display = 'none'; }} />
      </div>
    </button>

    {#if dropdownOpen}
      <div class="dropdown" role="menu">
        <button class="dropdown-item" role="menuitem" onclick={() => { dropdownOpen = false; showProfile = true; }}>
          Профиль
        </button>

        {#if $currentUser?.role !== 'guest'}
          <div class="dropdown-divider"></div>
          <button class="dropdown-item" role="menuitem" onclick={() => { dropdownOpen = false; showTechniques = true; }}>
            Техники
          </button>
        {/if}
        <button class="dropdown-item" role="menuitem" onclick={() => { dropdownOpen = false; showHotkeys = true; }}>
          Горячие клавиши
        </button>
        {#if $currentUser?.is_admin}
          <button class="dropdown-item" role="menuitem" onclick={() => { dropdownOpen = false; showCreateUser = true; }}>
            Пользователи
          </button>
          <button class="dropdown-item" role="menuitem" onclick={() => { dropdownOpen = false; showSyncDatabase = true; }}>
            Актуализировать базу
          </button>
          <button class="dropdown-item" role="menuitem" onclick={() => { dropdownOpen = false; batchAiMode = 'new'; showBatchAi = true; }}>
            ИИ-разметка видео
          </button>
          <button class="dropdown-item" role="menuitem" onclick={() => { dropdownOpen = false; batchAiMode = 'relabel'; showBatchAi = true; }}>
            ИИ-переразметка видео
          </button>
        {/if}

        <div class="dropdown-divider"></div>
        <button class="dropdown-item danger" role="menuitem" onclick={logout}>
          Выйти
        </button>
      </div>
    {/if}
  </div>
</header>

{#if showSearch}
  <SearchPanel onclose={() => { if (window.location.hash === '#/search') window.location.hash = '#/gallery'; }} />
{/if}

{#if showProfile}
  <ProfileModal onclose={() => { showProfile = false; }} />
{/if}

{#if showHotkeys}
  <HotkeysModal onclose={() => { showHotkeys = false; }} />
{/if}

{#if showCreateUser}
  <CreateUserModal onclose={() => { showCreateUser = false; }} />
{/if}

{#if showTechniques}
  <TechniquesModal onclose={() => { showTechniques = false; }} />
{/if}

{#if showSyncDatabase}
  <SyncModal onclose={() => { showSyncDatabase = false; }} />
{/if}

{#if showBatchAi}
  <BatchAiModal mode={batchAiMode} onclose={() => { showBatchAi = false; }} />
{/if}

<style>
  .header {
    height: 64px;
    background: var(--surface);
    border-bottom: 1px solid var(--border-color);
    display: flex;
    align-items: center;
    padding: 0 24px;
    gap: 16px;
    position: sticky;
    top: 0;
    z-index: 100;
  }

  /* Logo */
  .logo {
    display: flex;
    align-items: center;
    gap: 12px;
    background: none;
    border: none;
    cursor: pointer;
    color: var(--text-primary);
    font-size: 1.1rem;
    font-weight: 700;
    letter-spacing: 0.02em;
    padding: 0;
    flex-shrink: 0;
    text-decoration: none;
    transition: color 0.15s;
  }

  .logo:hover {
    color: var(--accent-yellow-hover);
  }

  /* Center nav */
  .nav {
    position: absolute;
    left: 50%;
    transform: translateX(-50%);
    display: flex;
    gap: 8px;
    background: var(--surface-solid);
    padding: 6px;
    border-radius: var(--radius-pill);
    box-shadow: var(--shadow-sm);
    border: 1px solid var(--border-color);
  }

  .nav-btn {
    background: none;
    border: none;
    color: var(--text-secondary);
    font-size: 0.9rem;
    font-weight: 500;
    cursor: pointer;
    padding: 8px 20px;
    border-radius: var(--radius-pill);
    transition: var(--transition);
  }

  .nav-btn:hover {
    color: var(--text-primary);
    background: var(--surface-hover);
  }

  .nav-btn.active {
    color: #000;
    background: var(--accent-yellow);
  }

  .logo-img {
    width: 32px;
    height: 32px;
    object-fit: contain;
  }

  /* Filter toggle button */
  .filter-toggle {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    background: none;
    border: 1px solid transparent;
    border-radius: 50%;
    color: var(--text-secondary);
    cursor: pointer;
    transition: var(--transition);
    flex-shrink: 0;
    margin-left: 8px;
  }

  .filter-toggle:hover {
    color: var(--text-primary);
    background: var(--surface-hover);
    border-color: var(--border-color);
  }

  .filter-toggle.active {
    color: var(--accent-yellow);
    background: rgba(245, 158, 11, 0.08);
    border-color: rgba(245, 158, 11, 0.25);
  }

  /* User menu */
  .user-menu {
    position: relative;
    flex-shrink: 0;
    margin-left: auto;
  }

  .menu-trigger {
    display: flex;
    align-items: center;
    gap: 10px;
    background: var(--surface-solid);
    border: 1px solid var(--border-color);
    box-shadow: var(--shadow-sm);
    cursor: pointer;
    padding: 4px;
    border-radius: var(--radius-pill);
    transition: var(--transition);
  }

  .menu-trigger:hover {
    transform: translateY(-1px);
    box-shadow: var(--shadow-md);
  }

  .avatar {
    width: 36px;
    height: 36px;
    border-radius: 50%;
    background: var(--surface-hover);
    border: 1.5px solid var(--border-color);
    overflow: hidden;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    position: relative;
  }

  .avatar-icon {
    position: absolute;
    pointer-events: none;
  }
  .avatar-icon circle, .avatar-icon path {
    stroke: var(--text-secondary);
  }

  .avatar img {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  /* Dropdown */
  .dropdown {
    position: absolute;
    top: calc(100% + 12px);
    right: 0;
    background: var(--surface-solid);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-md);
    padding: 8px;
    min-width: 200px;
    box-shadow: var(--shadow-lg);
    z-index: 200;
  }

  .dropdown-item {
    display: block;
    width: 100%;
    background: none;
    border: none;
    color: var(--text-secondary);
    font-size: 0.9rem;
    font-weight: 500;
    text-align: left;
    padding: 10px 16px;
    cursor: pointer;
    border-radius: var(--radius-sm);
    transition: var(--transition);
  }

  .dropdown-item:hover {
    background: var(--surface-hover);
    color: var(--text-primary);
  }

  .dropdown-item.danger {
    color: #ef4444;
  }

  .dropdown-item.danger:hover {
    background: #fef2f2;
    color: #dc2626;
  }

  .dropdown-divider {
    height: 1px;
    background: var(--border-color);
    margin: 8px 0;
  }

  @media (max-width: 768px) {
    .nav {
      position: static;
      transform: none;
      background: transparent;
      box-shadow: none;
      border: none;
      margin-left: auto;
      margin-right: 16px;
    }
    .logo span {
      display: none;
    }
    .nav-btn {
      padding: 6px 12px;
      font-size: 0.85rem;
    }
    .user-menu {
      margin-left: 0;
    }
  }
</style>
