<script lang="ts">
  import type { Fighter } from '../api/types';

  interface Props {
    fighters: Fighter[];
    selectedId: string | null;
    onselect: (fighter: Fighter) => void;
  }

  let { fighters, selectedId, onselect }: Props = $props();
</script>

<aside class="sidebar">
  <h3 class="title">Бойцы</h3>

  {#each fighters as fighter (fighter.id)}
    <button
      class="row"
      class:active={fighter.id === selectedId}
      onclick={() => onselect(fighter)}
    >
      {#if fighter.avatar_url}
        <img class="avatar" src={fighter.avatar_url} alt={fighter.display_name} />
      {:else}
        <div class="avatar-fallback">{fighter.display_name.charAt(0).toUpperCase()}</div>
      {/if}
      <span class="name">{fighter.display_name}</span>
    </button>
  {/each}

  {#if fighters.length === 0}
    <p class="empty">Нет бойцов</p>
  {/if}
</aside>

<style>
  .sidebar {
    width: 216px;
    flex-shrink: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .title {
    font-size: 0.68rem;
    font-weight: 600;
    letter-spacing: 0.09em;
    text-transform: uppercase;
    color: #4a6280;
    margin-bottom: 8px;
    padding: 0 6px;
  }

  .row {
    display: flex;
    align-items: center;
    gap: 9px;
    background: none;
    border: none;
    color: #a0b4c8;
    font-size: 0.875rem;
    text-align: left;
    cursor: pointer;
    padding: 7px 8px;
    border-radius: 7px;
    transition: background 0.12s, color 0.12s;
    width: 100%;
  }

  .row:hover {
    background: #1a3050;
    color: #e8edf2;
  }

  .row.active {
    background: rgba(219, 132, 31, 0.12);
    color: #DB841F;
  }

  .avatar {
    width: 26px;
    height: 26px;
    border-radius: 50%;
    object-fit: cover;
    flex-shrink: 0;
    border: 1px solid #2a4f73;
  }

  .avatar-fallback {
    width: 26px;
    height: 26px;
    border-radius: 50%;
    background: #1a3050;
    border: 1px solid #2a4f73;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 0.7rem;
    font-weight: 600;
    color: #a0b4c8;
    flex-shrink: 0;
  }

  .name {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    flex: 1;
    min-width: 0;
  }

  .empty {
    font-size: 0.8rem;
    color: #4a6280;
    padding: 4px 8px;
  }
</style>
