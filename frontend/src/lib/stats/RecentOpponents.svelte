<script lang="ts">
  import type { FighterBout } from '../api/types';
  import { fighters } from '../../stores';
  import { resolveColor } from '../api/types';

  interface Props {
    bouts: FighterBout[];
    currentFighterId?: string;
    selectedOpponentId?: string;
    onfilter?: (opponentId: string) => void;
  }

  let { bouts, currentFighterId, selectedOpponentId = '', onfilter }: Props = $props();

  let recentOpponents = $derived.by(() => {
    type FightKey = string;
    const fights = new Map<FightKey, { oppId: string; myScore: number; oppScore: number }>();
    for (const b of bouts) {
      const key: FightKey = `${b.opponent_id}::${b.video_id}`;
      const f = fights.get(key);
      if (f) { f.myScore += b.my_score; f.oppScore += b.opponent_score; }
      else fights.set(key, { oppId: b.opponent_id, myScore: b.my_score, oppScore: b.opponent_score });
    }

    const fightStats = new Map<string, { wins: number; losses: number; total: number }>();
    for (const { oppId, myScore, oppScore } of fights.values()) {
      if (!fightStats.has(oppId)) fightStats.set(oppId, { wins: 0, losses: 0, total: 0 });
      const s = fightStats.get(oppId)!;
      s.total += 1;
      if (myScore > oppScore) s.wins += 1;
      else if (myScore < oppScore) s.losses += 1;
    }

    return $fighters
      .filter(f => f.id !== currentFighterId)
      .map(f => {
        const stats = fightStats.get(f.id) || { wins: 0, losses: 0, total: 0 };
        const balance = stats.wins - stats.losses;
        return {
          id: f.id,
          name: f.display_name || f.username,
          wins: stats.wins,
          losses: stats.losses,
          total: stats.total,
          balance,
          avatar_url: f.avatar_url,
          color: resolveColor(f.id, f.color)
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
      <div class="opp-item" class:selected={opp.id === selectedOpponentId} onclick={() => onfilter?.(opp.id === selectedOpponentId ? '' : opp.id)}>
        <div class="avatar-wrap" style:background={opp.color}>
          {#if opp.avatar_url}
            <img class="avatar-img" src={opp.avatar_url} alt={opp.name}
              onerror={(e) => { (e.target as HTMLImageElement).style.display = 'none'; }} />
          {:else}
            <svg class="avatar-icon" width="24" height="24" viewBox="0 0 24 24" fill="none" aria-hidden="true">
              <circle cx="12" cy="8" r="4" stroke="#fff" stroke-width="1.5"/>
              <path d="M4 20c0-4 3.6-7 8-7s8 3 8 7" stroke="#fff" stroke-width="1.5" stroke-linecap="round"/>
            </svg>
          {/if}
        </div>
        <div class="opp-name" title={opp.name}>{opp.name}</div>
        <div class="opp-stats">
          <div>Побед: <span class="stat-val">{opp.wins}</span></div>
          <div>Поражений: <span class="stat-val">{opp.losses}</span></div>
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
    backdrop-filter: blur(var(--blur-amount));
    -webkit-backdrop-filter: blur(var(--blur-amount));
    border: 1px solid var(--border-color);
    border-radius: var(--radius-2xl);
    padding: 20px;
    display: flex;
    flex-direction: column;
    height: 100%;
  }

  .card-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 16px;
  }

  .card-title {
    font-size: 1rem;
    font-weight: 700;
    color: var(--text-primary);
    margin: 0;
  }

  .opponents-list {
    display: grid;
    grid-template-columns: repeat(5, 1fr);
    grid-auto-rows: 1fr;
    gap: 20px 12px;
    overflow-y: auto;
    padding-bottom: 12px;
    flex: 1;
    scrollbar-width: thin;
    scrollbar-color: rgba(255,255,255,0.1) transparent;
  }

  .opponents-list::-webkit-scrollbar { height: 6px; }
  .opponents-list::-webkit-scrollbar-thumb {
    background: rgba(255, 255, 255, 0.1);
    border-radius: 3px;
  }

  .opp-item {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 6px;
    min-width: 110px;
    padding: 4px;
    border-radius: var(--radius-xl);
    cursor: pointer;
    transition: var(--transition);
    border: 1px solid transparent;
  }

  .opp-item:hover {
    background: var(--surface-hover);
    border-color: var(--border-color);
  }
 
  .opp-item.selected {
    background: rgba(219, 132, 31, 0.12);
    border-color: rgba(219, 132, 31, 0.4);
    box-shadow: 0 4px 12px rgba(219, 132, 31, 0.1);
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
    flex-shrink: 0;
  }

  .avatar-icon { position: absolute; opacity: 0.6; }
  .avatar-img {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .opp-name {
    font-size: 1rem;
    font-weight: 700;
    color: var(--text-primary);
    text-align: center;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    width: 100px;
  }

  .opp-stats {
    font-size: 0.75rem;
    color: var(--text-secondary);
    text-align: center;
    line-height: 1.4;
    display: flex;
    flex-direction: column;
    opacity: 0.8;
  }

  .stat-val {
    color: var(--text-secondary);
    font-weight: 600;
  }

  .opp-score {
    font-size: 0.75rem;
    font-weight: 800;
    padding: 4px 10px;
    border-radius: 6px;
    background: rgba(255,255,255,0.05);
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    margin-top: 4px;
  }

  .opp-score.positive {
    background: var(--accent-green);
    color: #fff;
  }

  .opp-score.negative {
    background: var(--accent-red);
    color: #fff;
  }

  .empty {
    font-size: 0.85rem;
    color: var(--text-secondary);
    text-align: center;
    width: 100%;
    padding: 40px 0;
  }
</style>
