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
          class="search-inp"
          aria-label="Поиск"
        />
        <button class="btn-search" onclick={doSearch} disabled={searching || !query.trim()}>
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
    inset: 56px 0 0 0;
    z-index: 90;
    pointer-events: none;
    display: flex;
    flex-direction: column;
    align-items: center;
  }

  .panel {
    pointer-events: all;
    background: #0f2035;
    border: 1px solid #1f3a57;
    border-top: none;
    border-radius: 0 0 10px 10px;
    width: min(760px, 96vw);
    box-shadow: 0 12px 32px rgba(0,0,0,0.45);
    display: flex;
    flex-direction: column;
    max-height: calc(100vh - 80px);
    overflow: hidden;
  }

  .panel-head {
    padding: 12px 14px;
    border-bottom: 1px solid #1f3a57;
    flex-shrink: 0;
  }

  .search-row {
    display: flex;
    gap: 8px;
    align-items: center;
  }

  .search-inp {
    flex: 1;
    background: #0a1628;
    border: 1px solid #2a4f73;
    border-radius: 6px;
    color: #d0dde8;
    font-size: 0.9rem;
    padding: 7px 12px;
    outline: none;
    transition: border-color 0.12s;
  }

  .search-inp:focus { border-color: #DB841F; }
  .search-inp::placeholder { color: #3a5470; }

  .btn-search {
    padding: 7px 18px;
    background: #DB841F;
    border: none;
    border-radius: 6px;
    color: #fff;
    font-size: 0.85rem;
    font-weight: 600;
    cursor: pointer;
    transition: background 0.12s, opacity 0.12s;
    flex-shrink: 0;
  }

  .btn-search:hover:not(:disabled) { background: #e8941f; }
  .btn-search:disabled { opacity: 0.45; cursor: default; }

  .btn-close {
    background: none;
    border: none;
    color: #4a6280;
    font-size: 1rem;
    cursor: pointer;
    padding: 4px 8px;
    border-radius: 4px;
    transition: color 0.1s, background 0.1s;
    flex-shrink: 0;
  }

  .btn-close:hover { color: #e05252; background: rgba(224,82,82,0.1); }

  .msg-info {
    padding: 20px;
    text-align: center;
    font-size: 0.85rem;
    color: #4a6280;
  }

  .msg-error {
    padding: 12px 14px;
    font-size: 0.82rem;
    color: #e08080;
  }

  .results-wrap {
    overflow-y: auto;
    scrollbar-width: thin;
    scrollbar-color: #1a3050 transparent;
  }

  .results-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 0.82rem;
  }

  thead th {
    padding: 8px 12px;
    text-align: left;
    font-size: 0.68rem;
    font-weight: 600;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: #4a6280;
    border-bottom: 1px solid #1a3050;
    position: sticky;
    top: 0;
    background: #0f2035;
  }

  tbody tr {
    border-bottom: 1px solid #0d1e35;
    transition: background 0.1s;
  }

  tbody tr:hover { background: #0d1e35; }

  td {
    padding: 8px 12px;
    vertical-align: middle;
    color: #a0b4c8;
  }

  .col-text { width: 50%; }
  .col-author { width: 15%; white-space: nowrap; }
  .col-fighters { width: 20%; color: #6b8aab; font-size: 0.78rem; }
  .col-action { width: 15%; text-align: right; }

  .msg-text {
    display: block;
    color: #c8d8e8;
    word-break: break-word;
  }

  .msg-ts {
    display: block;
    font-size: 0.68rem;
    color: #3a6080;
    margin-top: 2px;
    font-variant-numeric: tabular-nums;
  }

  .btn-goto {
    background: rgba(219,132,31,0.12);
    border: 1px solid rgba(219,132,31,0.3);
    color: #DB841F;
    border-radius: 5px;
    padding: 4px 10px;
    font-size: 0.75rem;
    font-weight: 600;
    cursor: pointer;
    white-space: nowrap;
    transition: background 0.12s;
  }

  .btn-goto:hover { background: rgba(219,132,31,0.22); }
</style>
