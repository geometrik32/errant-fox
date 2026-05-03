<script lang="ts">
  import { login } from '$lib/api/auth';
  import { token, currentUser } from '../stores';

  let username = $state('');
  let password = $state('');
  let error = $state('');
  let loading = $state(false);

  async function handleSubmit(e: Event) {
    e.preventDefault();
    error = '';
    loading = true;
    try {
      const result = await login(username, password);
      token.set(result.token);
      currentUser.set(result.user);
      window.location.href = '/';
    } catch (e) {
      error = e instanceof Error ? e.message : 'Ошибка входа';
    } finally {
      loading = false;
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
