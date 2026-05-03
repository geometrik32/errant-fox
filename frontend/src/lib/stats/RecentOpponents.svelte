<script lang="ts">
  import type { FighterBout } from '../api/types';
  import { fighters } from '../../stores';
  import { resolveColor } from '../api/types';

  interface Props {
    bouts: FighterBout[];
    limit?: number;
  }

  let { bouts, limit = 5 }: Props = $props();

  let recentOpponents = $derived.by(() => {
    // Sort bouts by date descending
    const sorted = [...bouts].sort((a, b) => b.video_date.localeCompare(a.video_date));
    
    const uniqueMap = new Map<string, { id: string, name: string, scoreDiff: number }>();
    for (const b of sorted) {
      if (!uniqueMap.has(b.opponent_id)) {
        uniqueMap.set(b.opponent_id, {
          id: b.opponent_id,
          name: b.opponent_name,
          scoreDiff: b.my_score - b.opponent_score
        });
      }
      if (uniqueMap.size >= limit) break;
    }

    return [...uniqueMap.values()].map(opp => {
      const f = $fighters.find(f => f.id === opp.id);
      return {
        ...opp,
        avatar_url: f?.avatar_url,
        color: resolveColor(opp.id, f?.color)
      };
    });
  });
</script>

<div class="recent-opponents glass-card">
  <div class="card-header">
    <h3 class="card-title">Недавние оппоненты</h3>
    <span class="card-subtitle">быстрый доступ</span>
  </div>
  
  <div class="opponents-list">
    {#each recentOpponents as opp (opp.id)}
      <div class="opp-item">
        <div class="avatar-wrap" style:background={opp.color}>
          {#if opp.avatar_url}
            <img class="avatar-img" src={opp.avatar_url} alt={opp.name} onerror={(e) => { (e.target as HTMLImageElement).style.display = 'none'; }} />
          {:else}
            <svg class="avatar-icon" width="24" height="24" viewBox="0 0 24 24" fill="none" aria-hidden="true">
              <circle cx="12" cy="8" r="4" stroke="#fff" stroke-width="1.5"/>
              <path d="M4 20c0-4 3.6-7 8-7s8 3 8 7" stroke="#fff" stroke-width="1.5" stroke-linecap="round"/>
            </svg>
          {/if}
        </div>
        <div class="opp-name">{opp.name}</div>
        <div class="opp-score" class:positive={opp.scoreDiff > 0} class:negative={opp.scoreDiff < 0}>
          {opp.scoreDiff > 0 ? '+' : ''}{opp.scoreDiff}
        </div>
      </div>
    {:else}
      <div class="empty">Нет данных</div>
    {/each}
  </div>
</div>

<style>
  .recent-opponents {
    background: var(--surface);
    backdrop-filter: var(--glass-blur);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-lg);
    padding: 24px;
    box-shadow: var(--shadow-sm);
    display: flex;
    flex-direction: column;
    height: 100%;
  }

  .card-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 24px;
  }

  .card-title {
    font-size: 1rem;
    font-weight: 700;
    color: var(--text-primary);
    margin: 0;
  }

  .card-subtitle {
    font-size: 0.8rem;
    color: var(--text-secondary);
  }

  .opponents-list {
    display: flex;
    gap: 16px;
    overflow-x: auto;
    padding-bottom: 8px;
    flex: 1;
  }
  
  .opponents-list::-webkit-scrollbar {
    height: 4px;
  }
  
  .opponents-list::-webkit-scrollbar-thumb {
    background: rgba(255, 255, 255, 0.1);
    border-radius: 2px;
  }

  .opp-item {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    min-width: 70px;
    flex-shrink: 0;
  }

  .avatar-wrap {
    width: 56px;
    height: 56px;
    border-radius: 50%;
    border: 2px solid rgba(255,255,255,0.1);
    position: relative;
    display: flex;
    align-items: center;
    justify-content: center;
    overflow: hidden;
  }

  .avatar-icon {
    position: absolute;
    opacity: 0.6;
  }

  .avatar-img {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .opp-name {
    font-size: 0.8rem;
    color: var(--text-primary);
    text-align: center;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    width: 100%;
  }

  .opp-score {
    font-size: 0.75rem;
    font-weight: 600;
    padding: 2px 6px;
    border-radius: var(--radius-sm);
    background: var(--surface-solid);
    color: var(--text-secondary);
  }
  
  .opp-score.positive {
    background: rgba(39, 174, 96, 0.1);
    color: #27ae60;
  }
  
  .opp-score.negative {
    background: rgba(239, 68, 68, 0.1);
    color: #ef4444;
  }

  .empty {
    font-size: 0.85rem;
    color: var(--text-secondary);
    text-align: center;
    width: 100%;
    padding: 20px 0;
  }
</style>
