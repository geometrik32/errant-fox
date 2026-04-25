<script lang="ts">
  import { login } from '$lib/api/auth';
  import { token, currentUser } from '../stores';

  let username = $state('');
  let password = $state('');
  let error = $state('');
  let loading = $state(false);

  async function handleSubmit() {
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

<div class="auth-bg">
  <div class="auth-card">
    <h1>Errant Fox</h1>
    <form onsubmit={handleSubmit}>
      <label>
        Username
        <input type="text" bind:value={username} required autocomplete="username" />
      </label>
      <label>
        Password
        <input type="password" bind:value={password} required autocomplete="current-password" />
      </label>
      {#if error}
        <p class="error">{error}</p>
      {/if}
      <button type="submit" disabled={loading}>
        {loading ? 'Вход...' : 'Sign In'}
      </button>
    </form>
  </div>
</div>

<style>
  .auth-bg {
    min-height: 100vh;
    background: #0d1b2a;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .auth-card {
    background: #16283f;
    border: 1px solid #1f3a57;
    border-radius: 12px;
    padding: 48px 40px;
    width: 100%;
    max-width: 380px;
    display: flex;
    flex-direction: column;
    gap: 24px;
  }

  h1 {
    color: #DB841F;
    font-size: 1.75rem;
    font-weight: 700;
    text-align: center;
    margin: 0;
  }

  form {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  label {
    display: flex;
    flex-direction: column;
    gap: 6px;
    color: #a0b4c8;
    font-size: 0.875rem;
    font-weight: 500;
  }

  input {
    background: #0d1b2a;
    border: 1px solid #1f3a57;
    border-radius: 6px;
    color: #e8edf2;
    padding: 10px 12px;
    font-size: 1rem;
    outline: none;
    transition: border-color 0.2s;
  }

  input:focus {
    border-color: #DB841F;
  }

  button {
    background: #DB841F;
    color: #fff;
    border: none;
    border-radius: 6px;
    padding: 12px;
    font-size: 1rem;
    font-weight: 600;
    cursor: pointer;
    margin-top: 4px;
    transition: background 0.2s;
  }

  button:hover:not(:disabled) {
    background: #c4741a;
  }

  button:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .error {
    color: #e05252;
    font-size: 0.875rem;
    margin: 0;
    padding: 8px 12px;
    background: rgba(224, 82, 82, 0.1);
    border-radius: 6px;
    border: 1px solid rgba(224, 82, 82, 0.2);
  }
</style>
