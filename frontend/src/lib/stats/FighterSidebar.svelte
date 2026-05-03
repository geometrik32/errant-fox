<script lang="ts">
  import type { Fighter } from '../api/types';
  import { resolveColor } from '../api/types';

  interface Props {
    fighters: Fighter[];
    selectedId: string | null;
    onselect: (fighter: Fighter) => void;
  }

  let { fighters, selectedId, onselect }: Props = $props();
</script>

<aside class="sidebar">
  <h3 class="title">Бойцы</h3>

  <div class="list">
    {#each fighters as fighter (fighter.id)}
      <button
        class="row"
        class:active={fighter.id === selectedId}
        onclick={() => onselect(fighter)}
      >
        <div class="avatar-wrap" style:background={resolveColor(fighter.id, fighter.color)}>
          <svg class="avatar-icon" width="14" height="14" viewBox="0 0 24 24" fill="none">
            <circle cx="12" cy="8" r="4" stroke="#fff" stroke-width="1.5"/>
            <path d="M4 20c0-4 3.6-7 8-7s8 3 8 7" stroke="#fff" stroke-width="1.5" stroke-linecap="round"/>
          </svg>
          <img class="avatar-img" src={fighter.avatar_url} alt="" onerror={(e) => { (e.target as HTMLImageElement).style.display = 'none'; }} />
        </div>
        <div class="info">
          <span class="name">{fighter.display_name}</span>
          <span class="login">@{fighter.username}</span>
        </div>
      </button>
    {/each}
  </div>

  {#if fighters.length === 0}
    <p class="empty">Нет бойцов</p>
  {/if}
</aside>

<style>
  .sidebar {
    width: 280px;
    flex-shrink: 0;
    display: flex;
    flex-direction: column;
    gap: 16px;
    background: var(--surface);
    backdrop-filter: var(--glass-blur);
    -webkit-backdrop-filter: var(--glass-blur);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-md);
    box-shadow: var(--shadow-sm);
    padding: 20px;
    max-height: calc(100vh - 80px);
    overflow-y: auto;
  }

  .title {
    font-size: 0.8rem;
    font-weight: 600;
    letter-spacing: 0.09em;
    text-transform: uppercase;
    color: var(--text-secondary);
    margin: 0;
  }

  .list {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .row {
    display: flex;
    align-items: center;
    gap: 12px;
    background: transparent;
    border: 1px solid transparent;
    color: var(--text-primary);
    text-align: left;
    cursor: pointer;
    padding: 10px 12px;
    border-radius: var(--radius-sm);
    transition: var(--transition);
    width: 100%;
  }

  .row:hover {
    background: var(--surface-hover);
    border-color: var(--border-color);
  }

  .row.active {
    background: rgba(219, 132, 31, 0.12);
    border-color: rgba(219, 132, 31, 0.3);
    color: var(--accent-yellow);
    box-shadow: 0 4px 12px rgba(219, 132, 31, 0.1);
  }

  .avatar-wrap {
    width: 32px;
    height: 32px;
    border-radius: 50%;
    position: relative;
    overflow: hidden;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    border: 2px solid rgba(255, 255, 255, 0.2);
  }

  .avatar-icon {
    position: absolute;
    pointer-events: none;
  }

  .avatar-img {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .info {
    display: flex;
    flex-direction: column;
    min-width: 0;
  }

  .name {
    font-size: 0.95rem;
    font-weight: 600;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .login {
    font-size: 0.75rem;
    color: var(--text-secondary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .empty {
    font-size: 0.85rem;
    color: var(--text-secondary);
    padding: 4px 0;
  }

  @media (max-width: 1024px) {
    .sidebar {
      width: 100%;
      max-height: none;
      overflow-y: visible;
    }
    .list {
      flex-direction: row;
      flex-wrap: nowrap;
      overflow-x: auto;
      padding-bottom: 8px;
    }
    .row {
      width: auto;
      flex-shrink: 0;
    }
    .info {
      display: none;
    }
    .avatar-wrap {
      width: 44px;
      height: 44px;
    }
  }
</style>
