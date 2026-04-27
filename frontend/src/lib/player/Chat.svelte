<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { currentUser } from '../../stores';
  import type { Comment } from '../api/types';
  import { createComment, updateComment, deleteComment, reactComment, deleteReact } from '../api/comments';

  interface Props {
    videoId: string;
    comments?: Comment[];
    currentTime?: number;
    onseek?: (timestamp_ms: number) => void;
    oncommentschange?: (comments: Comment[]) => void;
  }

  let { videoId, comments: initComments = [], currentTime = 0, onseek, oncommentschange }: Props = $props();

  let comments = $state<Comment[]>([...initComments]);
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
      comments = [...comments, created];
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

  // WebSocket
  let ws: WebSocket | null = null;

  function connectWS() {
    ws = new WebSocket('ws://localhost:8080/ws');
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
          comments = [...comments, fields as unknown as Comment];
          oncommentschange?.(comments);
          requestAnimationFrame(() => {
            if (listEl) listEl.scrollTop = listEl.scrollHeight;
          });
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
      <div class="msg" style={c.reply_to_id !== null ? 'margin-left: 16px' : ''}>
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
              class="edit-inp"
              bind:value={editText}
              rows="2"
              onkeydown={(e) => { if (e.key === 'Enter' && !e.shiftKey) { e.preventDefault(); submitEdit(c.id); } if (e.key === 'Escape') editingId = null; }}
            ></textarea>
            <div class="edit-actions">
              <button class="edit-save" onclick={() => submitEdit(c.id)}>Сохранить</button>
              <button class="edit-cancel" onclick={() => { editingId = null; }}>Отмена</button>
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
            <button class="edit-link" onclick={() => startEdit(c)}>Редактировать</button>
            <button class="del-link" onclick={() => handleDelete(c.id)}>Удалить</button>
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
    background: #08101f;
    border-left: 1px solid #1a3050;
    overflow: hidden;
  }

  /* ── List ── */
  .list {
    flex: 1;
    overflow-y: auto;
    padding: 10px 8px;
    display: flex;
    flex-direction: column;
    gap: 8px;
    scrollbar-width: thin;
    scrollbar-color: #1a3050 transparent;
  }

  .list::-webkit-scrollbar { width: 4px; }
  .list::-webkit-scrollbar-track { background: transparent; }
  .list::-webkit-scrollbar-thumb { background: #1a3050; border-radius: 2px; }

  .msg {
    display: flex;
    flex-direction: column;
    gap: 4px;
    padding: 6px 8px;
    border-radius: 6px;
    background: #0d1e35;
    border: 1px solid #1a3050;
  }

  .msg-head {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .avatar {
    width: 24px;
    height: 24px;
    border-radius: 50%;
    background: #1f3a57;
    border: 1px solid #2a4f73;
    overflow: hidden;
    display: flex;
    align-items: center;
    justify-content: center;
    color: #4a6280;
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
    font-size: 0.75rem;
    font-weight: 600;
    color: #a0b4c8;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .ts {
    font-size: 0.68rem;
    font-variant-numeric: tabular-nums;
    color: #3a94c8;
    background: none;
    border: none;
    cursor: pointer;
    padding: 1px 4px;
    border-radius: 3px;
    white-space: nowrap;
    flex-shrink: 0;
    transition: background 0.1s, color 0.1s;
  }

  .ts:hover {
    background: rgba(58, 148, 200, 0.15);
    color: #60b8e8;
  }

  .reply-preview {
    font-size: 0.68rem;
    color: #4a6280;
    background: #0a1628;
    border-left: 2px solid #2a4f73;
    padding: 2px 6px;
    border-radius: 0 3px 3px 0;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .msg-text {
    font-size: 0.78rem;
    color: #c8d8e8;
    word-break: break-word;
    line-height: 1.4;
  }

  .msg-footer {
    display: flex;
    gap: 8px;
  }

  .reply-link, .edit-link, .del-link {
    background: none;
    border: none;
    font-size: 0.65rem;
    cursor: pointer;
    padding: 0;
    transition: color 0.1s;
  }

  .reply-link { color: #3a6080; }
  .reply-link:hover { color: #5a9ab8; }

  .edit-link { color: #3a6080; }
  .edit-link:hover { color: #5a9ab8; }

  .del-link { color: #5a3030; }
  .del-link:hover { color: #e05252; }

  .edit-area {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .edit-inp {
    width: 100%;
    background: #0d1e35;
    border: 1px solid #2a4f73;
    border-radius: 4px;
    color: #c8d8e8;
    font-size: 0.78rem;
    padding: 5px;
    resize: none;
    outline: none;
    font-family: inherit;
    line-height: 1.4;
  }

  .edit-actions {
    display: flex;
    gap: 6px;
  }

  .edit-save, .edit-cancel {
    flex: 1;
    padding: 3px 0;
    border-radius: 3px;
    font-size: 0.72rem;
    cursor: pointer;
    border: none;
  }

  .edit-save { background: #DB841F; color: #fff; }
  .edit-save:hover { background: #e8941f; }

  .edit-cancel { background: #0a1628; border: 1px solid #1a3050; color: #7090a8; }
  .edit-cancel:hover { background: #0f2035; }

  .empty {
    font-size: 0.78rem;
    color: #3a5470;
    text-align: center;
    padding: 24px 0;
  }

  /* ── Input ── */
  .input-area {
    flex-shrink: 0;
    border-top: 1px solid #1a3050;
    background: #060e1a;
    padding: 8px;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .reply-to-bar {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 4px 6px;
    background: #0d1e35;
    border-radius: 4px;
    border-left: 2px solid #2a4f73;
  }

  .reply-to-label {
    flex: 1;
    font-size: 0.68rem;
    color: #5a7a96;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .reply-cancel {
    background: none;
    border: none;
    color: #3a5470;
    cursor: pointer;
    font-size: 0.7rem;
    padding: 0 2px;
    line-height: 1;
    transition: color 0.1s;
  }

  .reply-cancel:hover { color: #e05252; }

  textarea {
    width: 100%;
    background: #0d1e35;
    border: 1px solid #1a3050;
    border-radius: 5px;
    color: #c8d8e8;
    font-size: 0.78rem;
    padding: 8px;
    resize: none;
    outline: none;
    font-family: inherit;
    line-height: 1.4;
    transition: border-color 0.12s;
  }

  textarea:focus { border-color: #2a4f73; }
  textarea:disabled { opacity: 0.5; cursor: default; }
  textarea::placeholder { color: #3a5470; }

  .react-btn {
    background: none;
    border: none;
    font-size: 0.7rem;
    cursor: pointer;
    padding: 1px 4px;
    border-radius: 3px;
    color: #3a5470;
    line-height: 1.4;
    transition: background 0.1s, color 0.1s;
    display: inline-flex;
    align-items: center;
    gap: 2px;
  }

  .react-btn:hover { background: #0f2035; color: #6b8aab; }
  .react-btn.active { color: #DB841F; }

  .react-count {
    font-size: 0.65rem;
    font-variant-numeric: tabular-nums;
    color: inherit;
  }
</style>
