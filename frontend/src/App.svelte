<script lang="ts">
  import { token, currentUser, initStores } from './stores';
  import Auth from './routes/Auth.svelte';
  import Header from './lib/ui/Header.svelte';
  import Gallery from './routes/Gallery.svelte';
  import Stats from './routes/Stats.svelte';
  import Player from './routes/Player.svelte';
  import { loginWithVk } from './lib/api/auth';

  let hash = $state(typeof window !== 'undefined' ? (window.location.hash || '#/gallery') : '#/gallery');
  let initialized = $state(false);
  let vkAuthLoading = $state(false);
  let vkAuthError = $state('');



  $effect(() => {
    const onHashChange = () => {
      hash = window.location.hash || '#/gallery';
    };
    window.addEventListener('hashchange', onHashChange);
    return () => window.removeEventListener('hashchange', onHashChange);
  });

  $effect(() => {
    if ($token && !initialized) {
      initStores().finally(() => {
        initialized = true;
      });
    }
    if (!$token) {
      initialized = false;
    }
  });

  $effect(() => {
    if (typeof window !== 'undefined') {
      const params = new URLSearchParams(window.location.search);
      const code = params.get('code');
      const deviceId = params.get('device_id') || '';
      if (code && !vkAuthLoading) {
        vkAuthLoading = true;
        vkAuthError = '';
        const redirectUri = window.location.origin;
        const codeVerifier = sessionStorage.getItem('vk_code_verifier') || '';
        loginWithVk(code, redirectUri, codeVerifier, deviceId)
          .then((result) => {
             if ($token) {
               currentUser.set(result.user);
             } else {
               token.set(result.token);
               currentUser.set(result.user);
             }
             const url = new URL(window.location.href);
             url.searchParams.delete('code');
             url.searchParams.delete('device_id');
             url.searchParams.delete('state');
             url.searchParams.delete('expires_in');
             window.history.replaceState({}, '', url.pathname + url.hash);
             sessionStorage.removeItem('vk_code_verifier');
             sessionStorage.removeItem('vk_state');
          })
          .catch((err) => {
             vkAuthError = err.message || 'Ошибка авторизации через VK ID';
             const url = new URL(window.location.href);
             url.searchParams.delete('code');
             url.searchParams.delete('device_id');
             url.searchParams.delete('state');
             url.searchParams.delete('expires_in');
             window.history.replaceState({}, '', url.pathname + url.hash);
          })
          .finally(() => {
             vkAuthLoading = false;
          });
      }
    }
  });

  let routeName = $derived(
    hash.startsWith('#/player/') || hash.startsWith('#/share/video/') ? 'player' : hash === '#/stats' ? 'stats' : 'gallery'
  );

  let isShareRoute = $derived(hash.startsWith('#/share/video/'));

  let playerId = $derived.by(() => {
    if (hash.startsWith('#/player/')) {
      return hash.slice('#/player/'.length).split('?')[0];
    }
    if (hash.startsWith('#/share/video/')) {
      return hash.slice('#/share/video/'.length).split('?')[0];
    }
    return '';
  });

  let shareToken = $derived.by(() => {
    if (!hash.startsWith('#/share/video/')) return '';
    const match = hash.match(/[?&]token=([^&]+)/);
    return match ? decodeURIComponent(match[1]) : '';
  });

  let sharedBoutId = $derived.by(() => {
    const match = hash.match(/[?&]bout_id=(\d+)/);
    return match ? parseInt(match[1], 10) : null;
  });

  let initialTimeMs = $derived.by(() => {
    const match = hash.match(/[?&]t=(\d+)/);
    return match ? parseInt(match[1], 10) : 0;
  });
</script>

{#if vkAuthLoading}
  <div class="vk-loading-overlay">
    <div class="vk-loading-card glass-card">
      <div class="spinner"></div>
      <p>Авторизация через VK ID...</p>
    </div>
  </div>
{:else if !$token && !isShareRoute}
  <Auth />
  {#if vkAuthError}
    <div class="toast-error">{vkAuthError}</div>
  {/if}
{:else}
  <div class="app">
    {#if !isShareRoute}
      <Header {hash} />
    {/if}
    <main class="main" class:main--player={routeName === 'player'} class:main--stats={routeName === 'stats'}>
      {#if $currentUser?.role === 'guest' && !isShareRoute}
        <div class="guest-restricted-container">
          <div class="guest-restricted-card glass-card">
            <svg class="lock-icon" viewBox="0 0 24 24" width="48" height="48" fill="none" stroke="currentColor" stroke-width="1.5">
              <rect x="3" y="11" width="18" height="11" rx="2" ry="2" />
              <path d="M7 11V7a5 5 0 0 1 10 0v4" />
            </svg>
            <h2>Доступ ограничен</h2>
            <p>Ваш аккаунт имеет роль <strong>Гость</strong>.</p>
            <p>Доступ к общей медиатеке ограничен. Вы можете просматривать видео только по прямым ссылкам общего доступа, которыми с вами поделились.</p>
          </div>
        </div>
      {:else}
        {#if routeName === 'gallery'}
          <Gallery />
        {:else if routeName === 'stats'}
          <Stats />
        {:else if routeName === 'player' && playerId}
          {#key playerId + '-' + initialTimeMs + '-' + shareToken + '-' + sharedBoutId}
          <Player videoId={playerId} {initialTimeMs} {shareToken} {sharedBoutId} />
          {/key}
        {/if}
      {/if}
    </main>
  </div>
{/if}

<style>
  .app {
    display: flex;
    flex-direction: column;
    height: 100vh;
  }

  .main {
    flex: 1;
    padding: 24px;
    overflow: auto;
  }

  .main--player {
    padding: 0;
    overflow: hidden;
  }

  .main--stats {
    padding: 0;
  }

  .vk-loading-overlay {
    position: fixed;
    inset: 0;
    background: rgba(10, 10, 12, 0.8);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .vk-loading-card {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 20px;
    padding: 32px 48px;
    border-radius: var(--radius-lg);
    color: var(--text-primary);
  }

  .spinner {
    width: 40px;
    height: 40px;
    border: 3px solid rgba(0, 119, 255, 0.2);
    border-top-color: #0077FF;
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .toast-error {
    position: fixed;
    bottom: 24px;
    right: 24px;
    background: #ef4444;
    color: #fff;
    padding: 12px 24px;
    border-radius: var(--radius-md);
    font-weight: 500;
    box-shadow: 0 4px 12px rgba(0,0,0,0.3);
    z-index: 999;
    animation: slideUp 0.3s ease;
  }

  @keyframes slideUp {
    from { transform: translateY(20px); opacity: 0; }
    to { transform: translateY(0); opacity: 1; }
  }

  .guest-restricted-container {
    display: flex;
    justify-content: center;
    align-items: center;
    height: 100%;
    min-height: 400px;
    padding: 24px;
  }

  .guest-restricted-card {
    max-width: 480px;
    padding: 40px;
    text-align: center;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 16px;
    border-radius: var(--radius-lg);
  }

  .guest-restricted-card h2 {
    font-size: 1.5rem;
    font-weight: 600;
    margin: 0;
    color: var(--text-primary);
  }

  .guest-restricted-card p {
    font-size: 0.95rem;
    color: var(--text-secondary);
    line-height: 1.5;
    margin: 0;
  }

  .lock-icon {
    color: var(--accent-blue);
    margin-bottom: 8px;
  }
</style>
