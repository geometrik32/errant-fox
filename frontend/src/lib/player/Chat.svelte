<script lang="ts">
  import { onMount, onDestroy, untrack } from 'svelte';
  import { currentUser } from '../../stores';
  import type { Comment } from '../api/types';
  import { createComment, updateComment, deleteComment, reactComment, deleteReact } from '../api/comments';

  interface Props {
    videoId: string;
    comments?: Comment[];
    currentTime?: number;
    highlightedId?: number | null;
    onseek?: (timestamp_ms: number) => void;
    oncommentschange?: (comments: Comment[]) => void;
  }

  let { videoId, comments: initComments = [], currentTime = 0, highlightedId = null, onseek, oncommentschange }: Props = $props();

  let comments = $state<Comment[]>([...untrack(() => initComments)]);
  let text = $state('');
  let replyTo = $state<Comment | null>(null);
  let sending = $state(false);
  let listEl: HTMLDivElement;

  let sortedComments = $derived.by(() => {
    const topLevel = comments.filter(c => c.reply_to_id === null).sort((a, b) => a.id - b.id);
    const result: Comment[] = [];
    for (const c of topLevel) {
      result.push(c);
      const replies = comments.filter(r => r.reply_to_id === c.id).sort((a, b) => a.id - b.id);
      result.push(...replies);
    }
    return result;
  });

  let editingId = $state<number | null>(null);
  let editText = $state('');

  function fmtMs(ms: number): string {
    const total = Math.floor(ms / 1000);
    const h = Math.floor(total / 3600);
    const m = Math.floor((total % 3600) / 60);
    const s = total % 60;
    return h > 0
      ? `${String(h).padStart(2, '0')}:${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}`
      : `${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}`;
  }

  function getReplyPreview(replyToId: number | null): string {
    if (replyToId === null) return '';
    const c = comments.find(x => x.id === replyToId);
    if (!c) return '';
    return c.text.length > 60 ? c.text.slice(0, 60) + '…' : c.text;
  }

  async function submit() {
    const t = text.trim();
    if (!t || sending) return;
    sending = true;
    try {
      const created = await createComment({
        video_id: videoId,
        timestamp_ms: replyTo ? replyTo.timestamp_ms : Math.round(currentTime * 1000),
        text: t,
        reply_to_id: replyTo?.id ?? null,
      });
      const idx = comments.findIndex(c => c.id === created.id);
      if (idx >= 0) {
        comments = comments.map((c, i) => i === idx ? created : c);
      } else {
        comments = [...comments, created];
      }
      oncommentschange?.(comments);
      text = '';
      replyTo = null;
      requestAnimationFrame(() => {
        if (listEl) listEl.scrollTop = listEl.scrollHeight;
      });
    } finally {
      sending = false;
    }
  }

  function startEdit(c: Comment) {
    editingId = c.id;
    editText = c.text;
  }

  async function submitEdit(id: number) {
    const t = editText.trim();
    if (!t) return;
    try {
      const updated = await updateComment(id, t);
      comments = comments.map(c => c.id === id ? updated : c);
      oncommentschange?.(comments);
    } finally {
      editingId = null;
    }
  }

  async function handleDelete(id: number) {
    if (!confirm('Удалить сообщение?')) return;
    await deleteComment(id);
    comments = comments.filter(c => c.id !== id && c.reply_to_id !== id);
    oncommentschange?.(comments);
  }

  async function handleReact(c: (typeof comments)[0], kind: 'like' | 'dislike') {
    const prev = c.my_reaction;
    if (prev === kind) {
      await deleteReact(c.id);
      comments = comments.map(x => x.id !== c.id ? x : {
        ...x,
        my_reaction: null,
        likes:    kind === 'like'    ? x.likes    - 1 : x.likes,
        dislikes: kind === 'dislike' ? x.dislikes - 1 : x.dislikes,
      });
    } else {
      await reactComment(c.id, kind);
      comments = comments.map(x => x.id !== c.id ? x : {
        ...x,
        my_reaction: kind,
        likes:    x.likes    + (kind === 'like'    ? 1 : 0) - (prev === 'like'    ? 1 : 0),
        dislikes: x.dislikes + (kind === 'dislike' ? 1 : 0) - (prev === 'dislike' ? 1 : 0),
      });
    }
    oncommentschange?.(comments);
  }

  function onKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      submit();
    }
  }

  // Scroll to + flash highlighted comment from timeline click
  $effect(() => {
    const id = highlightedId;
    if (!id || !listEl) return;
    const el = listEl.querySelector<HTMLElement>(`[data-comment-id="${id}"]`);
    if (!el) return;
    el.scrollIntoView({ block: 'nearest', behavior: 'smooth' });
    el.classList.remove('msg--flash');
    void el.offsetWidth; // reflow to restart animation
    el.classList.add('msg--flash');
  });

  // WebSocket
  let ws: WebSocket | null = null;

  function connectWS() {
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    ws = new WebSocket(`${protocol}//${window.location.host}/ws`);
    ws.onopen = () => {
      const token = localStorage.getItem('ef_token');
      if (!token) return;
      ws!.send(JSON.stringify({ token }));
      ws!.send(JSON.stringify({ watching: videoId }));
    };
    ws.onmessage = (e) => {
      try {
        const msg = JSON.parse(e.data as string) as Record<string, unknown>;
        if (msg.type === 'new_comment' && msg.video_id === videoId) {
          const { type: _t, video_id: _v, ...fields } = msg;
          const incoming = fields as unknown as Comment;
          const idx = comments.findIndex(c => c.id === incoming.id);
          if (idx >= 0) {
            comments = comments.map((c, i) => i === idx ? incoming : c);
          } else {
            comments = [...comments, incoming];
            requestAnimationFrame(() => {
              if (listEl) listEl.scrollTop = listEl.scrollHeight;
            });
          }
          oncommentschange?.(comments);
        }
      } catch { /* ignore malformed */ }
    };
    ws.onclose = () => {
      setTimeout(() => { if (ws !== null) connectWS(); }, 4000);
    };
  }

  onMount(connectWS);
  onDestroy(() => {
    const w = ws;
    ws = null;
    w?.close();
  });
</script>

<div class="chat">

  <!-- Message list -->
  <div class="list" bind:this={listEl}>
    {#each sortedComments as c (c.id)}
      <div class="msg" data-comment-id={c.id} style={c.reply_to_id !== null ? 'margin-left: 16px' : ''}>
        <div class="msg-head">
          <div class="avatar">
            <svg class="avatar-icon" width="14" height="14" viewBox="0 0 24 24" fill="none" aria-hidden="true">
              <circle cx="12" cy="8" r="4" stroke="currentColor" stroke-width="1.5"/>
              <path d="M4 20c0-4 3.6-7 8-7s8 3 8 7" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
            </svg>
            <img src={c.author.avatar_url} alt={c.author.display_name} onerror={(e) => { (e.target as HTMLImageElement).style.display = 'none'; }} />
          </div>
          <span class="name">{c.author.display_name}</span>
          <button class="ts" onclick={() => onseek?.(c.timestamp_ms)}>
            {fmtMs(c.timestamp_ms)}
          </button>
        </div>

        {#if c.reply_to_id !== null}
          <div class="reply-preview">{getReplyPreview(c.reply_to_id)}</div>
        {/if}

        {#if editingId === c.id}
          <div class="edit-area">
            <textarea
              class="input-glass edit-inp"
              bind:value={editText}
              rows="2"
              onkeydown={(e) => { if (e.key === 'Enter' && !e.shiftKey) { e.preventDefault(); submitEdit(c.id); } if (e.key === 'Escape') editingId = null; }}
            ></textarea>
            <div class="edit-actions">
              <button class="btn btn-primary btn-sm" onclick={() => submitEdit(c.id)}>Сохранить</button>
              <button class="btn btn-outline btn-sm" onclick={() => { editingId = null; }}>Отмена</button>
            </div>
          </div>
        {:else}
          <div class="msg-text">{c.text}</div>
        {/if}

        <div class="msg-footer">
          <button
            class="react-btn"
            class:active={c.my_reaction === 'like'}
            onclick={() => handleReact(c, 'like')}
            title="Нравится"
          >👍 {#if c.likes > 0}<span class="react-count">{c.likes}</span>{/if}</button>
          <button
            class="react-btn"
            class:active={c.my_reaction === 'dislike'}
            onclick={() => handleReact(c, 'dislike')}
            title="Не нравится"
          >👎 {#if c.dislikes > 0}<span class="react-count">{c.dislikes}</span>{/if}</button>
          <button class="reply-link" onclick={() => { replyTo = c; }}>Ответить</button>
          {#if $currentUser?.id === c.author.id}
            <button class="edit-link" onclick={() => startEdit(c)}>Ред.</button>
            <button class="del-link" onclick={() => handleDelete(c.id)} title="Удалить">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <polyline points="3 6 5 6 21 6"></polyline>
                <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path>
                <line x1="10" y1="11" x2="10" y2="17"></line>
                <line x1="14" y1="11" x2="14" y2="17"></line>
              </svg>
            </button>
          {/if}
        </div>
      </div>
    {:else}
      <div class="empty">Нет комментариев</div>
    {/each}
  </div>

  <!-- Input area -->
  <div class="input-area">
    {#if replyTo}
      <div class="reply-to-bar">
        <span class="reply-to-label">
          Ответ для: {replyTo.text.length > 40 ? replyTo.text.slice(0, 40) + '…' : replyTo.text}
        </span>
        <button class="reply-cancel" onclick={() => { replyTo = null; }}>✕</button>
      </div>
    {/if}
    <textarea
      class="input-glass"
      bind:value={text}
      onkeydown={onKeydown}
      placeholder="Комментарий… (Enter — отправить)"
      rows="2"
      disabled={sending}
    ></textarea>
  </div>

</div>

<style>
  .chat {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: transparent;
    overflow: hidden;
  }

  /* ── List ── */
  .list {
    flex: 1;
    overflow-y: auto;
    padding: 12px;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .msg {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 10px 12px;
    border-radius: var(--radius-sm);
    background: var(--surface-hover);
    border: 1px solid var(--border-color);
    transition: border-color 0.2s;
  }

  @keyframes comment-flash {
    0%   { border-color: var(--accent-yellow); background: rgba(219, 132, 31, 0.15); }
    100% { border-color: var(--border-color);  background: var(--surface-hover); }
  }

  :global(.msg--flash) {
    animation: comment-flash 1.2s ease-out forwards;
  }

  .msg-head {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .avatar {
    width: 28px;
    height: 28px;
    border-radius: 50%;
    background: var(--surface-solid);
    border: 1px solid var(--border-color);
    overflow: hidden;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-secondary);
    flex-shrink: 0;
    position: relative;
  }

  .avatar-icon {
    position: absolute;
  }

  .avatar img {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .name {
    flex: 1;
    font-size: 0.85rem;
    font-weight: 600;
    color: var(--text-primary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .ts {
    font-size: 0.75rem;
    font-variant-numeric: tabular-nums;
    color: var(--accent-yellow);
    background: none;
    border: none;
    cursor: pointer;
    padding: 2px 6px;
    border-radius: var(--radius-sm);
    white-space: nowrap;
    flex-shrink: 0;
    transition: var(--transition);
  }

  .ts:hover {
    background: rgba(219, 132, 31, 0.15);
    color: #e8941f;
  }

  .reply-preview {
    font-size: 0.8rem;
    color: var(--text-secondary);
    background: var(--surface-solid);
    border-left: 2px solid var(--accent-yellow);
    padding: 4px 8px;
    border-radius: 0 var(--radius-sm) var(--radius-sm) 0;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .msg-text {
    font-size: 0.9rem;
    color: var(--text-primary);
    word-break: break-word;
    line-height: 1.5;
  }

  .msg-footer {
    display: flex;
    gap: 12px;
    align-items: center;
    margin-top: 2px;
    white-space: nowrap;
  }

  .reply-link, .edit-link, .del-link {
    background: none;
    border: none;
    font-size: 0.75rem;
    cursor: pointer;
    padding: 2px 4px;
    border-radius: var(--radius-sm);
    transition: var(--transition);
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .reply-link { color: var(--text-secondary); font-weight: 500; }
  .reply-link:hover { color: var(--text-primary); background: var(--surface-solid); }

  .edit-link { color: var(--text-secondary); font-weight: 500; }
  .edit-link:hover { color: var(--text-primary); background: var(--surface-solid); }

  .del-link { color: var(--text-secondary); }
  .del-link:hover { color: #ef4444; background: rgba(239, 68, 68, 0.1); }

  .edit-area {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .edit-inp {
    width: 100%;
    padding: 8px;
  }

  .edit-actions {
    display: flex;
    gap: 8px;
  }

  .empty {
    font-size: 0.9rem;
    color: var(--text-secondary);
    text-align: center;
    padding: 32px 0;
  }

  /* ── Input ── */
  .input-area {
    flex-shrink: 0;
    border-top: 1px solid var(--border-color);
    background: transparent;
    padding: 12px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .reply-to-bar {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 10px;
    background: var(--surface-hover);
    border-radius: var(--radius-sm);
    border-left: 2px solid var(--accent-yellow);
  }

  .reply-to-label {
    flex: 1;
    font-size: 0.8rem;
    color: var(--text-secondary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .reply-cancel {
    background: none;
    border: none;
    color: var(--text-secondary);
    cursor: pointer;
    font-size: 0.9rem;
    padding: 0 4px;
    line-height: 1;
    transition: var(--transition);
  }

  .reply-cancel:hover { color: #ef4444; }

  textarea {
    width: 100%;
    padding: 10px;
    resize: none;
  }

  .react-btn {
    background: none;
    border: none;
    font-size: 0.85rem;
    cursor: pointer;
    padding: 2px 6px;
    border-radius: var(--radius-sm);
    color: var(--text-secondary);
    line-height: 1.4;
    transition: var(--transition);
    display: inline-flex;
    align-items: center;
    gap: 4px;
  }

  .react-btn:hover { background: var(--surface-hover); color: var(--text-primary); }
  .react-btn.active { color: var(--accent-yellow); }

  .react-count {
    font-size: 0.8rem;
    font-variant-numeric: tabular-nums;
    color: inherit;
  }
</style>
