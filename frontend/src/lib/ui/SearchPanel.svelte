<script lang="ts">
  import { searchComments } from '../api/comments';
  import type { SearchResult } from '../api/types';

  interface Props {
    onclose?: () => void;
  }

  let { onclose }: Props = $props();

  let query = $state('');
  let results = $state<SearchResult[]>([]);
  let searching = $state(false);
  let searched = $state(false);
  let error = $state<string | null>(null);
  let inputEl: HTMLInputElement;

  function fmtMs(ms: number): string {
    const t = Math.floor(ms / 1000);
    const m = Math.floor(t / 60);
    const s = t % 60;
    return `${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}`;
  }

  async function doSearch() {
    const q = query.trim();
    if (!q) return;
    searching = true;
    searched = false;
    error = null;
    try {
      results = await searchComments(q);
      searched = true;
    } catch (e) {
      error = e instanceof Error ? e.message : 'Ошибка поиска';
    } finally {
      searching = false;
    }
  }

  function onKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') doSearch();
    if (e.key === 'Escape') onclose?.();
  }

  function navigate(r: SearchResult) {
    const hash = `#/player/${r.video_id}?t=${r.timestamp_ms}`;
    // Force navigation even if we're on the same video — reset the hash to trigger hashchange
    if (window.location.hash === hash) {
      window.location.hash = '';
      requestAnimationFrame(() => { window.location.hash = hash; });
    } else {
      window.location.hash = hash;
    }
    onclose?.();
  }

  function fighters(r: SearchResult): string {
    const a = r.fighter_a_name ?? '?';
    const b = r.fighter_b_name ?? '?';
    return `${a} vs ${b}`;
  }

  $effect(() => {
    inputEl?.focus();
  });
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="overlay" onkeydown={(e) => e.key === 'Escape' && onclose?.()}>
  <div class="panel">
    <div class="panel-head">
      <div class="search-row">
        <input
          bind:this={inputEl}
          bind:value={query}
          onkeydown={onKeydown}
          placeholder="Поиск по сообщениям…"
          class="input-glass search-inp"
          aria-label="Поиск"
        />
        <button class="btn btn-primary btn-search" onclick={doSearch} disabled={searching || !query.trim()}>
          {searching ? '…' : 'Поиск'}
        </button>
        <button class="btn-close" onclick={onclose} aria-label="Закрыть">✕</button>
      </div>
    </div>

    {#if error}
      <div class="msg-error">{error}</div>
    {:else if searching}
      <div class="msg-info">Поиск…</div>
    {:else if searched && results.length === 0}
      <div class="msg-info">Ничего не найдено.</div>
    {:else if searched}
      <div class="results-wrap">
        <table class="results-table">
          <thead>
            <tr>
              <th class="col-text">Сообщение</th>
              <th class="col-author">Автор</th>
              <th class="col-fighters">Бойцы</th>
              <th class="col-action"></th>
            </tr>
          </thead>
          <tbody>
            {#each results as r (r.comment_id)}
              <tr>
                <td class="col-text">
                  <span class="msg-text">{r.comment_text}</span>
                  <span class="msg-ts">{fmtMs(r.timestamp_ms)}</span>
                </td>
                <td class="col-author">{r.author_name}</td>
                <td class="col-fighters">{fighters(r)}</td>
                <td class="col-action">
                  <button class="btn-goto" onclick={() => navigate(r)}>
                    {#if r.bout_order_index !== null}
                      Сход {r.bout_order_index}
                    {:else}
                      Видео
                    {/if}
                    →
                  </button>
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    {/if}
  </div>
</div>

<style>
  .overlay {
    position: fixed;
    inset: 64px 0 0 0;
    z-index: 90;
    pointer-events: none;
    display: flex;
    flex-direction: column;
    align-items: center;
  }

  .panel {
    pointer-events: all;
    background: var(--surface-solid);
    border: 1px solid var(--border-color);
    border-top: none;
    border-radius: 0 0 var(--radius-lg) var(--radius-lg);
    width: min(760px, 96vw);
    box-shadow: none;
    display: flex;
    flex-direction: column;
    max-height: calc(100vh - 80px);
    overflow: hidden;
  }

  .panel-head {
    padding: 16px 20px;
    border-bottom: 1px solid var(--border-color);
    flex-shrink: 0;
  }

  .search-row {
    display: flex;
    gap: 12px;
    align-items: center;
  }

  .search-inp {
    flex: 1;
    font-size: 1rem;
    padding: 10px 16px;
  }

  .btn-search {
    padding: 10px 24px;
    flex-shrink: 0;
  }

  .btn-close {
    background: var(--surface-solid);
    border: 1px solid var(--border-color);
    color: var(--text-secondary);
    font-size: 1.2rem;
    cursor: pointer;
    width: 36px;
    height: 36px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 50%;
    transition: var(--transition);
    flex-shrink: 0;
  }

  .btn-close:hover {
    color: #ef4444;
    border-color: #ef4444;
    background: rgba(239, 68, 68, 0.1);
  }

  .msg-info {
    padding: 32px;
    text-align: center;
    font-size: 0.9rem;
    color: var(--text-secondary);
  }

  .msg-error {
    padding: 16px 20px;
    font-size: 0.9rem;
    color: #ef4444;
  }

  .results-wrap {
    overflow-y: auto;
  }

  .results-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 0.85rem;
  }

  thead th {
    padding: 12px 16px;
    text-align: left;
    font-size: 0.75rem;
    font-weight: 600;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--text-secondary);
    border-bottom: 1px solid var(--border-color);
    position: sticky;
    top: 0;
    background: var(--surface-solid);
  }

  tbody tr {
    border-bottom: 1px solid var(--border-color);
    transition: var(--transition);
  }

  tbody tr:hover { background: var(--surface-hover); }

  td {
    padding: 12px 16px;
    vertical-align: middle;
    color: var(--text-primary);
  }

  .col-text { width: 50%; }
  .col-author { width: 15%; white-space: nowrap; }
  .col-fighters { width: 20%; color: var(--text-secondary); font-size: 0.8rem; }
  .col-action { width: 15%; text-align: right; }

  .msg-text {
    display: block;
    color: var(--text-primary);
    word-break: break-word;
    font-size: 0.9rem;
  }

  .msg-ts {
    display: block;
    font-size: 0.75rem;
    color: var(--text-secondary);
    margin-top: 4px;
    font-variant-numeric: tabular-nums;
  }

  .btn-goto {
    background: rgba(219, 132, 31, 0.15);
    border: 1px solid rgba(219, 132, 31, 0.4);
    color: var(--accent-yellow);
    border-radius: var(--radius-sm);
    padding: 6px 12px;
    font-size: 0.8rem;
    font-weight: 600;
    cursor: pointer;
    white-space: nowrap;
    transition: var(--transition);
  }

  .btn-goto:hover {
    background: var(--accent-yellow);
    color: #000;
  }

  @media (max-width: 768px) {
    .search-row {
      flex-wrap: wrap;
    }
    .btn-search {
      flex: 1;
    }
    .col-author, .col-fighters {
      display: none;
    }
    .col-text { width: 70%; }
    .col-action { width: 30%; }
  }
</style>
