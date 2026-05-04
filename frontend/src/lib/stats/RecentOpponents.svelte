<script lang="ts">
  import type { FighterBout } from '../api/types';
  import { fighters } from '../../stores';
  import { resolveColor } from '../api/types';

  interface Props {
    bouts: FighterBout[];
    onfilter?: (opponentId: string) => void;
  }

  let { bouts, onfilter }: Props = $props();

  let recentOpponents = $derived.by(() => {
    // Group bouts by (opponent_id, video_id) to get per-fight scores
    type FightKey = string;
    const fights = new Map<FightKey, { oppId: string; oppName: string; myScore: number; oppScore: number }>();
    for (const b of bouts) {
      const key: FightKey = `${b.opponent_id}::${b.video_id}`;
      const f = fights.get(key);
      if (f) { f.myScore += b.my_score; f.oppScore += b.opponent_score; }
      else fights.set(key, { oppId: b.opponent_id, oppName: b.opponent_name, myScore: b.my_score, oppScore: b.opponent_score });
    }

    const uniqueMap = new Map<string, { id: string; name: string; wins: number; losses: number; total: number }>();
    for (const { oppId, oppName, myScore, oppScore } of fights.values()) {
      if (!uniqueMap.has(oppId)) {
        uniqueMap.set(oppId, { id: oppId, name: oppName, wins: 0, losses: 0, total: 0 });
      }
      const opp = uniqueMap.get(oppId)!;
      opp.total += 1;
      if (myScore > oppScore) opp.wins += 1;
      else if (myScore < oppScore) opp.losses += 1;
    }

    return [...uniqueMap.values()].map(opp => {
      const f = $fighters.find(f => f.id === opp.id);
      const balance = opp.wins - opp.losses;
      return {
        ...opp,
        balance,
        avatar_url: f?.avatar_url,
        color: resolveColor(opp.id, f?.color)
      };
    });
  });
</script>

<div class="recent-opponents glass-card">
  <div class="card-header">
    <h3 class="card-title">Оппоненты</h3>
  </div>
  
  <div class="opponents-list">
    {#each recentOpponents as opp (opp.id)}
      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div class="opp-item" onclick={() => onfilter?.(opp.id)}>
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
        <div class="opp-stats">
          <div>Побед: <span class="stat-val">{opp.wins}</span></div>
          <div>Поражений: <span class="stat-val">{opp.losses}</span></div>
          <div>Всего: <span class="stat-val">{opp.total}</span></div>
        </div>
        <div class="opp-score" class:positive={opp.balance > 0} class:negative={opp.balance < 0}>
          Баланс: {opp.balance > 0 ? '+' : ''}{opp.balance}
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
    box-shadow: none;
    display: flex;
    flex-direction: column;
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
    min-width: 100px;
    flex-shrink: 0;
    padding: 8px;
    border-radius: var(--radius-md);
    cursor: pointer;
    transition: background 0.2s;
  }

  .opp-item:hover {
    background: var(--surface-hover);
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
    font-size: 0.85rem;
    font-weight: 600;
    color: var(--text-primary);
    text-align: center;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    width: 100%;
  }

  .opp-stats {
    font-size: 0.7rem;
    color: var(--text-secondary);
    text-align: center;
    line-height: 1.4;
    margin-bottom: 4px;
  }

  .stat-val {
    color: var(--text-primary);
    font-weight: 600;
  }

  .opp-score {
    font-size: 0.75rem;
    font-weight: 600;
    padding: 4px 8px;
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
