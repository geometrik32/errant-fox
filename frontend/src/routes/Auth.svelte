<script lang="ts">
  import { login, getVkConfig } from '$lib/api/auth';
  import { token, currentUser } from '../stores';
  import { generateCodeVerifier, generateCodeChallenge } from '$lib/utils/pkce';

  let username = $state('');
  let password = $state('');
  let error = $state('');
  let loading = $state(false);
  let vkAppId = $state<string | null>(null);

  $effect(() => {
    getVkConfig().then(cfg => {
      vkAppId = cfg.client_id;
    }).catch(err => {
      console.error('Не удалось загрузить конфигурацию VK:', err);
    });
  });

  async function handleSubmit(e: Event) {
    e.preventDefault();
    error = '';
    loading = true;
    try {
      const result = await login(username, password);
      token.set(result.token);
      currentUser.set(result.user);
      if (window.location.hash && window.location.hash !== '#/' && window.location.hash !== '') {
        window.location.reload();
      } else {
        window.location.href = '/';
      }
    } catch (e) {
      error = e instanceof Error ? e.message : 'Ошибка входа';
    } finally {
      loading = false;
    }
  }

  async function handleVkLogin() {
    if (!vkAppId) return;
    if (window.location.hash && window.location.hash !== '#/' && window.location.hash !== '') {
      sessionStorage.setItem('post_auth_hash', window.location.hash);
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
      console.error('Ошибка подготовки PKCE для VK ID:', e);
      error = 'Не удалось запустить авторизацию VK ID';
    }
  }
</script>

<div class="auth-wrap">
  <div class="auth-card glass-card">
    <h1>Errant Fox</h1>
    <form onsubmit={handleSubmit}>
      <label>
        Username
        <input class="input-glass" type="text" bind:value={username} required autocomplete="username" />
      </label>
      <label>
        Password
        <input class="input-glass" type="password" bind:value={password} required autocomplete="current-password" />
      </label>
      {#if error}
        <p class="error">{error}</p>
      {/if}
      <button class="btn btn-primary" type="submit" disabled={loading}>
        {loading ? 'Вход...' : 'Sign In'}
      </button>

      {#if vkAppId}
        <div class="vk-divider">
          <span>или</span>
        </div>
        <button class="btn btn-vk" type="button" onclick={handleVkLogin} disabled={loading}>
          <svg class="vk-icon" width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
            <path d="M15.96 2H8.04C3.6 2 2 3.6 2 8.04v7.92C2 20.4 3.6 22 8.04 22h7.92c4.44 0 6.04-1.6 6.04-6.04V8.04C22 3.6 20.4 2 15.96 2zm3.32 12.3c.09.28.18.57.26.85.11.36.19.72.06 1.09-.12.34-.41.48-.73.51-1.12.1-2.24.04-3.36.03-.31 0-.58-.1-.78-.36-.59-.72-1.14-1.47-1.74-2.18-.18-.21-.38-.28-.62-.17-.23.11-.31.32-.33.56-.04.47-.02.94-.03 1.41 0 .42-.2.66-.6.69-.76.06-1.52.06-2.28-.05-1.5-.22-2.73-.97-3.75-2.07-1.39-1.5-2.45-3.21-3.38-5-.14-.26-.11-.47.15-.59.23-.11.48-.12.73-.12 1 .01 2.01 0 3.01.01.27 0 .49.12.61.37.42.86.9 1.7 1.45 2.5.17.25.35.42.66.32.28-.09.35-.33.38-.6.09-.94.04-1.87-.19-2.79-.08-.34-.28-.53-.61-.59-.14-.02-.13-.08-.07-.15.11-.14.28-.21.48-.21h3.33c.36.05.54.24.59.6.14.97.11 1.95.04 2.92 0 .07 0 .15-.02.22-.05.35-.01.59.35.73.23.09.39-.03.54-.19.67-.73 1.23-1.54 1.75-2.39.15-.24.32-.42.63-.42.99.01 1.99 0 2.99.01.42 0 .58.21.43.6-.33.86-.78 1.66-1.31 2.41-.27.38-.27.6.08 1 .49.56 1.02 1.1 1.49 1.68.32.41.6.86.87 1.31z"/>
          </svg>
          Войти через VK ID
        </button>
      {/if}
    </form>
  </div>
</div>

<style>
  .auth-wrap {
    min-height: 100vh;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 24px;
  }

  .auth-card {
    width: 100%;
    max-width: 400px;
    display: flex;
    flex-direction: column;
    gap: 32px;
    padding: 48px;
  }

  h1 {
    color: var(--text-primary);
    font-size: 2rem;
    font-weight: 700;
    text-align: center;
    margin: 0;
  }

  form {
    display: flex;
    flex-direction: column;
    gap: 20px;
  }

  label {
    display: flex;
    flex-direction: column;
    gap: 8px;
    color: var(--text-secondary);
    font-size: 0.9rem;
    font-weight: 500;
  }

  .btn {
    margin-top: 8px;
    padding: 12px;
    font-size: 1rem;
    font-weight: 600;
  }

  .btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .vk-divider {
    display: flex;
    align-items: center;
    text-align: center;
    color: var(--text-secondary);
    font-size: 0.85rem;
    margin: 4px 0;
  }

  .vk-divider::before,
  .vk-divider::after {
    content: '';
    flex: 1;
    border-bottom: 1px solid rgba(255, 255, 255, 0.1);
  }

  .vk-divider:not(:empty)::before {
    margin-right: .8em;
  }

  .vk-divider:not(:empty)::after {
    margin-left: .8em;
  }

  .btn-vk {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 10px;
    background: #0077FF;
    color: #fff;
    border: none;
    padding: 12px;
    font-size: 1rem;
    font-weight: 600;
    border-radius: var(--radius-md);
    cursor: pointer;
    transition: background 0.2s ease, transform 0.1s ease;
  }

  .btn-vk:hover:not(:disabled) {
    background: #0066DD;
  }

  .btn-vk:active:not(:disabled) {
    transform: scale(0.98);
  }

  .btn-vk:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .vk-icon {
    flex-shrink: 0;
  }

  .error {
    color: #ef4444;
    font-size: 0.875rem;
    margin: 0;
    padding: 10px 14px;
    background: rgba(239, 68, 68, 0.1);
    border-radius: var(--radius-sm);
    border: 1px solid rgba(239, 68, 68, 0.2);
  }
</style>
