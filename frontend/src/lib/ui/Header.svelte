<script lang="ts">
  import { token, currentUser } from '../../stores';
  import CreateUserModal from '../admin/CreateUserModal.svelte';
  import TechniquesModal from '../admin/TechniquesModal.svelte';
  import ProfileModal from './ProfileModal.svelte';

  interface Props {
    hash: string;
  }

  let { hash }: Props = $props();

  let dropdownOpen = $state(false);
  let showCreateUser = $state(false);
  let showTechniques = $state(false);
  let showProfile = $state(false);

  let activeNav = $derived(
    hash === '#/stats' ? 'stats' : 'gallery'
  );

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

  let avatarFallback = $derived(
    $currentUser?.display_name?.charAt(0).toUpperCase() ?? '?'
  );

  let avatarColor = $derived($currentUser?.color ?? '#1f3a57');
</script>

<header class="header">
  <!-- Left: logo -->
  <button class="logo" onclick={() => navigate('#/gallery')}>
    <svg width="24" height="24" viewBox="0 0 24 24" fill="none" aria-hidden="true">
      <path d="M12 2L3 7v10l9 5 9-5V7L12 2z" stroke="#DB841F" stroke-width="1.5" stroke-linejoin="round"/>
      <path d="M12 2v20M3 7l9 5 9-5" stroke="#DB841F" stroke-width="1.5" stroke-linejoin="round"/>
    </svg>
    <span>Errant Fox</span>
  </button>

  <!-- Center: nav -->
  <nav class="nav">
    <button
      class="nav-btn"
      class:active={activeNav === 'gallery'}
      onclick={() => navigate('#/gallery')}
    >
      Видео
    </button>
    <button
      class="nav-btn"
      class:active={activeNav === 'stats'}
      onclick={() => navigate('#/stats')}
    >
      Бойцы
    </button>
  </nav>

  <!-- Right: user menu -->
  <div class="user-menu">
    <button
      class="menu-trigger"
      onclick={(e) => { e.stopPropagation(); dropdownOpen = !dropdownOpen; }}
      aria-label="Меню пользователя"
      aria-expanded={dropdownOpen}
    >
      <div class="avatar" style:background={avatarColor}>
        {#if $currentUser?.avatar_url}
          <img src={$currentUser.avatar_url} alt={$currentUser.display_name} />
        {:else}
          <span>{avatarFallback}</span>
        {/if}
      </div>
    </button>

    {#if dropdownOpen}
      <div class="dropdown" role="menu">
        <button class="dropdown-item" role="menuitem" onclick={() => { dropdownOpen = false; showProfile = true; }}>
          Профиль
        </button>

        {#if $currentUser?.is_admin}
          <div class="dropdown-divider"></div>
          <button class="dropdown-item" role="menuitem" onclick={() => { dropdownOpen = false; showCreateUser = true; }}>
            Создать бойца
          </button>
          <button class="dropdown-item" role="menuitem" onclick={() => { dropdownOpen = false; showTechniques = true; }}>
            Техники
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

{#if showProfile}
  <ProfileModal onclose={() => { showProfile = false; }} />
{/if}

{#if showCreateUser}
  <CreateUserModal onclose={() => { showCreateUser = false; }} />
{/if}

{#if showTechniques}
  <TechniquesModal onclose={() => { showTechniques = false; }} />
{/if}

<style>
  .header {
    height: 56px;
    background: #0f2035;
    border-bottom: 1px solid #1f3a57;
    display: flex;
    align-items: center;
    padding: 0 20px;
    gap: 16px;
    position: sticky;
    top: 0;
    z-index: 100;
  }

  /* Logo */
  .logo {
    display: flex;
    align-items: center;
    gap: 10px;
    background: none;
    border: none;
    cursor: pointer;
    color: #e8edf2;
    font-size: 1rem;
    font-weight: 700;
    letter-spacing: 0.02em;
    padding: 0;
    flex-shrink: 0;
    text-decoration: none;
    transition: color 0.15s;
  }

  .logo:hover {
    color: #DB841F;
  }

  /* Center nav */
  .nav {
    flex: 1;
    display: flex;
    justify-content: center;
    gap: 4px;
  }

  .nav-btn {
    background: none;
    border: none;
    color: #6b8aab;
    font-size: 0.9rem;
    font-weight: 500;
    cursor: pointer;
    padding: 6px 18px;
    border-radius: 6px;
    transition: color 0.15s, background 0.15s;
  }

  .nav-btn:hover {
    color: #e8edf2;
    background: #1a3050;
  }

  .nav-btn.active {
    color: #DB841F;
    background: rgba(219, 132, 31, 0.12);
  }

  /* User menu */
  .user-menu {
    position: relative;
    flex-shrink: 0;
  }

  .menu-trigger {
    display: flex;
    align-items: center;
    gap: 10px;
    background: none;
    border: none;
    cursor: pointer;
    color: #6b8aab;
    padding: 4px;
    border-radius: 6px;
    transition: background 0.15s;
  }

  .menu-trigger:hover {
    background: #1a3050;
    color: #a0b4c8;
  }

  .avatar {
    width: 32px;
    height: 32px;
    border-radius: 50%;
    background: #1f3a57;
    border: 1.5px solid #2a4f73;
    overflow: hidden;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 0.875rem;
    font-weight: 600;
    color: #a0b4c8;
    flex-shrink: 0;
  }

  .avatar img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  /* Dropdown */
  .dropdown {
    position: absolute;
    top: calc(100% + 8px);
    right: 0;
    background: #0f2035;
    border: 1px solid #1f3a57;
    border-radius: 8px;
    padding: 4px;
    min-width: 176px;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.4);
    z-index: 200;
  }

  .dropdown-item {
    display: block;
    width: 100%;
    background: none;
    border: none;
    color: #a0b4c8;
    font-size: 0.875rem;
    text-align: left;
    padding: 8px 12px;
    cursor: pointer;
    border-radius: 5px;
    transition: background 0.12s, color 0.12s;
  }

  .dropdown-item:hover {
    background: #1a3050;
    color: #e8edf2;
  }

  .dropdown-item.danger {
    color: #e05252;
  }

  .dropdown-item.danger:hover {
    background: rgba(224, 82, 82, 0.1);
    color: #e05252;
  }

  .dropdown-divider {
    height: 1px;
    background: #1f3a57;
    margin: 4px 0;
  }
</style>
