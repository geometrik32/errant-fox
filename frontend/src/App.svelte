<script lang="ts">
  import { token, currentUser, initStores } from './stores';
  import Auth from './routes/Auth.svelte';
  import Header from './lib/ui/Header.svelte';
  import Gallery from './routes/Gallery.svelte';
  import Stats from './routes/Stats.svelte';
  import Player from './routes/Player.svelte';

  let hash = $state(typeof window !== 'undefined' ? (window.location.hash || '#/gallery') : '#/gallery');
  let initialized = $state(false);

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

  let routeName = $derived(
    hash.startsWith('#/player/') ? 'player' : hash === '#/stats' ? 'stats' : 'gallery'
  );

  let playerId = $derived(
    hash.startsWith('#/player/') ? hash.slice('#/player/'.length).split('?')[0] : ''
  );

  let initialTimeMs = $derived.by(() => {
    if (!hash.startsWith('#/player/')) return 0;
    const match = hash.match(/[?&]t=(\d+)/);
    return match ? parseInt(match[1], 10) : 0;
  });
</script>

{#if !$token}
  <Auth />
{:else}
  <div class="app">
    <Header {hash} />
    <main class="main" class:main--player={routeName === 'player'}>
      {#if routeName === 'gallery'}
        <Gallery />
      {:else if routeName === 'stats'}
        <Stats />
      {:else if routeName === 'player' && playerId}
        {#key playerId + '-' + initialTimeMs}
        <Player videoId={playerId} {initialTimeMs} />
        {/key}
      {/if}
    </main>
  </div>
{/if}

<style>
  :global(*, *::before, *::after) {
    box-sizing: border-box;
    margin: 0;
    padding: 0;
  }

  :global(body) {
    background: #0d1b2a;
    color: #e8edf2;
    font-family: system-ui, -apple-system, sans-serif;
    min-height: 100vh;
  }

  .app {
    display: flex;
    flex-direction: column;
    min-height: 100vh;
  }

  .main {
    flex: 1;
    padding: 24px;
  }

  .main--player {
    padding: 0;
    overflow: hidden;
  }

</style>
