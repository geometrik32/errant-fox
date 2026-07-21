<script lang="ts">
  import { onMount, onDestroy, untrack } from 'svelte';
  import { currentUser } from '../../stores';
  import type { Comment, Bout } from '../api/types';
  import { createComment, updateComment, deleteComment, reactComment, deleteReact, createSharedComment } from '../api/comments';
  import ConfirmModal from '../ui/ConfirmModal.svelte';
  import type { Stroke } from './DrawingCanvas.svelte';

  interface Props {
    videoId: string;
    comments?: Comment[];
    currentTime?: number;
    highlightedId?: number | null;
    readonly?: boolean;
    shareToken?: string;
    sharedBoutId?: number | null;
    isDrawingMode?: boolean;
    drawingStrokes?: Stroke[];
    onseek?: (timestamp_ms: number) => void;
    oncommentschange?: (comments: Comment[]) => void;
    onstartdrawing?: () => void;
    onclosedrawing?: () => void;
    onselectcommentdrawing?: (strokes: Stroke[] | null, timestampMs?: number) => void;
    bouts?: Bout[];
  }

  let {
    videoId,
    comments: initComments = [],
    currentTime = 0,
    highlightedId = null,
    readonly = false,
    shareToken = '',
    sharedBoutId = null,
    isDrawingMode = false,
    drawingStrokes = $bindable([]),
    bouts = [],
    onseek,
    oncommentschange,
    onstartdrawing,
    onclosedrawing,
    onselectcommentdrawing,
  }: Props = $props();

  let comments = $state<Comment[]>([...untrack(() => initComments)]);
  $effect(() => {
    comments = [...initComments];
  });

  let sharedBout = $derived(
    sharedBoutId
      ? bouts.find(b => b.id === sharedBoutId)
      : null
  );
  let text = $state('');
  let replyTo = $state<Comment | null>(null);
  let sending = $state(false);
  let listEl: HTMLDivElement;
  let textareaEl = $state<HTMLTextAreaElement | null>(null);

  let filterByActiveBout = $state(false);

  let currentTimeMs = $derived(currentTime * 1000);

  let sortedBouts = $derived(
    [...bouts].sort((a, b) => a.time_start_ms - b.time_start_ms)
  );

  let currentBout = $derived(
    sortedBouts.find(b => currentTimeMs >= b.time_start_ms && currentTimeMs <= b.time_end_ms)
  );

  let currentBoutIndex = $derived(
    currentBout ? sortedBouts.findIndex(b => b.id === currentBout.id) : -1
  );

  let sortedComments = $derived.by(() => {
    let filtered = comments;
    if (sharedBout) {
      filtered = comments.filter(c => {
        if (highlightedId && (c.id === highlightedId || c.reply_to_id === highlightedId)) return true;
        return c.timestamp_ms >= sharedBout.time_start_ms && c.timestamp_ms <= sharedBout.time_end_ms;
      });
    } else if (filterByActiveBout) {
      if (currentBout) {
        filtered = comments.filter(c => {
          if (highlightedId && (c.id === highlightedId || c.reply_to_id === highlightedId)) return true;
          return c.timestamp_ms >= currentBout.time_start_ms && c.timestamp_ms <= currentBout.time_end_ms;
        });
      } else {
        filtered = highlightedId ? comments.filter(c => c.id === highlightedId || c.reply_to_id === highlightedId) : [];
      }
    }

    const topLevel = filtered.filter(c => c.reply_to_id === null).sort((a, b) => a.id - b.id);
    const result: Comment[] = [];
    for (const c of topLevel) {
      result.push(c);
      const replies = filtered.filter(r => r.reply_to_id === c.id).sort((a, b) => a.id - b.id);
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

  let isGuestMode = $derived(!!shareToken || !$currentUser);
  let guestNickname = $state(typeof localStorage !== 'undefined' ? (localStorage.getItem('ef_guest_nickname') || 'Гость') : 'Гость');

  function onGuestNicknameChange(e: Event) {
    const val = (e.target as HTMLInputElement).value;
    guestNickname = val;
    if (typeof localStorage !== 'undefined') {
      localStorage.setItem('ef_guest_nickname', val);
    }
  }

  function parseDrawing(drawingStr?: string | null): Stroke[] | null {
    if (!drawingStr) return null;
    try {
      const data = JSON.parse(drawingStr);
      if (data && Array.isArray(data.strokes)) {
        return data.strokes;
      }
    } catch (e) {
      // ignore
    }
    return null;
  }

  function handleCommentClick(c: Comment) {
    onseek?.(c.timestamp_ms);
    const strokes = parseDrawing(c.drawing);
    onselectcommentdrawing?.(strokes, c.timestamp_ms);
  }

  function toggleDrawingMode() {
    if (isDrawingMode) {
      onclosedrawing?.();
    } else {
      onseek?.(currentTime * 1000);
      onstartdrawing?.();
    }
  }

  function clearDrawing() {
    drawingStrokes = [];
    onclosedrawing?.();
  }

  async function submit() {
    const t = text.trim();
    if (!t || sending) return;
    sending = true;
    try {
      let created: Comment;
      let commentTimeMs = replyTo ? replyTo.timestamp_ms : Math.round(currentTime * 1000);
      if (sharedBout) {
        commentTimeMs = Math.max(sharedBout.time_start_ms, Math.min(sharedBout.time_end_ms, commentTimeMs));
      }
      let drawingDataStr: string | null = null;
      if (drawingStrokes && drawingStrokes.length > 0) {
        drawingDataStr = JSON.stringify({ version: 1, strokes: drawingStrokes });
      }

      if (shareToken || !$currentUser) {
        created = await createSharedComment({
          videoId,
          token: shareToken || localStorage.getItem('ef_token') || '',
          nickname: guestNickname || 'Гость',
          text: t,
          timestamp_ms: commentTimeMs,
          reply_to_id: replyTo?.id ?? null,
          bout_id: sharedBoutId,
          drawing: drawingDataStr,
        });
      } else {
        created = await createComment({
          video_id: videoId,
          timestamp_ms: commentTimeMs,
          text: t,
          reply_to_id: replyTo?.id ?? null,
          drawing: drawingDataStr,
        });
      }
      const idx = comments.findIndex(c => c.id === created.id);
      if (idx >= 0) {
        comments = comments.map((c, i) => i === idx ? created : c);
      } else {
        comments = [...comments, created];
      }
      oncommentschange?.(comments);
      text = '';
      replyTo = null;
      drawingStrokes = [];
      onclosedrawing?.();
      requestAnimationFrame(() => {
        if (listEl) listEl.scrollTop = listEl.scrollHeight;
      });
    } catch (err) {
      alert(err instanceof Error ? err.message : 'Ошибка отправки комментария');
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

  let deleteTargetId = $state<number | null>(null);

  function promptDelete(id: number) {
    deleteTargetId = id;
  }

  async function confirmDelete() {
    if (deleteTargetId === null) return;
    const id = deleteTargetId;
    deleteTargetId = null;
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

  // WebSocket handler (called by parent Player)
  export function handleWsMessage(msg: Record<string, unknown>) {
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
    } else if (msg.type === 'update_comment' && msg.video_id === videoId) {
      const { type: _t, video_id: _v, ...fields } = msg;
      const incoming = fields as unknown as Comment;
      comments = comments.map(c => c.id === incoming.id ? incoming : c);
      oncommentschange?.(comments);
    } else if (msg.type === 'delete_comment' && msg.video_id === videoId) {
      const id = msg.id as number;
      comments = comments.filter(c => c.id !== id);
      oncommentschange?.(comments);
    }
  }
</script>

<div class="chat">

  <!-- Chat Header -->
  <div class="chat-header">
    <span class="chat-title">Комментарии</span>
    {#if !sharedBoutId}
      <label class="filter-switch" title="Показывать только комментарии внутри текущего схода">
        <div class="switch-container">
          <input 
            type="checkbox" 
            bind:checked={filterByActiveBout} 
          />
          <span class="slider"></span>
        </div>
        <span class="switch-label">
          {#if currentBoutIndex !== -1}
            По сходу №{currentBoutIndex + 1}
          {:else}
            По сходу
          {/if}
        </span>
      </label>
    {/if}
  </div>

  <!-- Message list -->
  <div class="list" bind:this={listEl}>
    {#each sortedComments as c (c.id)}
      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div
        class="msg"
        class:highlighted={highlightedId === c.id}
        data-comment-id={c.id}
        style={c.reply_to_id !== null ? 'margin-left: 16px' : ''}
        onclick={() => handleCommentClick(c)}
      >
        <div class="msg-head">
          <div class="avatar">
            <svg class="avatar-icon" width="14" height="14" viewBox="0 0 24 24" fill="none" aria-hidden="true">
              <circle cx="12" cy="8" r="4" stroke="currentColor" stroke-width="1.5"/>
              <path d="M4 20c0-4 3.6-7 8-7s8 3 8 7" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
            </svg>
            <img src={c.author.avatar_url} alt={c.author.display_name} onerror={(e) => { (e.target as HTMLImageElement).style.display = 'none'; }} />
          </div>
          <span class="name">{c.author.display_name}</span>
          {#if c.drawing}
            <button class="drawing-badge" onclick={(e) => { e.stopPropagation(); handleCommentClick(c); }} title="Посмотреть рисунок">
              🎨
            </button>
          {/if}
          <button class="ts" onclick={(e) => { e.stopPropagation(); handleCommentClick(c); }}>
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

        <div class="msg-footer" onclick={(e) => e.stopPropagation()}>
          <button
            class="react-btn"
            class:active={c.my_reaction === 'like'}
            onclick={(e) => { e.stopPropagation(); handleReact(c, 'like'); }}
            title="Нравится"
          >👍 {#if c.likes > 0}<span class="react-count">{c.likes}</span>{/if}</button>
          <button
            class="react-btn"
            class:active={c.my_reaction === 'dislike'}
            onclick={(e) => { e.stopPropagation(); handleReact(c, 'dislike'); }}
            title="Не нравится"
          >👎 {#if c.dislikes > 0}<span class="react-count">{c.dislikes}</span>{/if}</button>
          <button class="reply-link" onclick={(e) => { e.stopPropagation(); replyTo = c; textareaEl?.focus(); }}>Ответить</button>
          {#if $currentUser?.id === c.author.id || ($currentUser && c.author.id === 'guest')}
            {#if $currentUser?.id === c.author.id}
              <button class="edit-link" onclick={(e) => { e.stopPropagation(); startEdit(c); }}>Ред.</button>
            {/if}
            <button class="del-link" onclick={(e) => { e.stopPropagation(); promptDelete(c.id); }} title="Удалить">
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
    {#if isGuestMode}
      <div class="guest-name-bar">
        <span class="guest-label">Имя гостя:</span>
        <input
          type="text"
          class="guest-input"
          value={guestNickname}
          oninput={onGuestNicknameChange}
          placeholder="Гость"
        />
      </div>
    {/if}
    {#if replyTo}
      <div class="reply-to-bar">
        <span class="reply-to-label">
          Ответ для: {replyTo.text.length > 40 ? replyTo.text.slice(0, 40) + '…' : replyTo.text}
        </span>
        <button class="reply-cancel" onclick={() => { replyTo = null; }}>✕</button>
      </div>
    {/if}
    <textarea
      bind:this={textareaEl}
      class="input-glass"
      bind:value={text}
      onkeydown={onKeydown}
      placeholder="Комментарий… (Enter — отправить)"
      rows="2"
      disabled={sending}
    ></textarea>

    <div class="input-actions-row">
      <button
        type="button"
        class="pencil-btn"
        class:active={isDrawingMode || (drawingStrokes && drawingStrokes.length > 0)}
        onclick={toggleDrawingMode}
        title="Нарисовать поверх кадра видео"
      >
        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M12 20h9" />
          <path d="M16.5 3.5a2.121 2.121 0 0 1 3 3L7 19l-4 1 1-4L16.5 3.5z" />
        </svg>
        <span>Рисовать</span>
      </button>
    </div>
  </div>

</div>

{#if deleteTargetId !== null}
  <ConfirmModal
    title="Удаление сообщения"
    message="Вы действительно хотите удалить этот комментарий?"
    confirmText="Удалить"
    cancelText="Отмена"
    danger={true}
    onconfirm={confirmDelete}
    oncancel={() => (deleteTargetId = null)}
  />
{/if}

<style>
  .chat {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: transparent;
    overflow: hidden;
  }

  /* ── Header ── */
  .chat-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 16px;
    border-bottom: 1px solid var(--border-color);
    background: rgba(0, 0, 0, 0.15);
    flex-shrink: 0;
  }

  .chat-title {
    font-size: 0.8rem;
    font-weight: 600;
    color: var(--text-primary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  /* ── Switch Slider ── */
  .filter-switch {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    cursor: pointer;
    font-size: 0.8rem;
    color: var(--text-secondary);
    user-select: none;
    transition: var(--transition);
  }

  .switch-container {
    position: relative;
    display: inline-block;
    width: 28px;
    height: 16px;
    flex-shrink: 0;
  }

  .switch-container input {
    opacity: 0;
    width: 0;
    height: 0;
  }

  .slider {
    position: absolute;
    cursor: pointer;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background-color: rgba(255, 255, 255, 0.1);
    border: 1px solid var(--border-color);
    transition: .2s;
    border-radius: var(--radius-pill);
  }

  .slider:before {
    position: absolute;
    content: "";
    height: 10px;
    width: 10px;
    left: 2px;
    bottom: 2px;
    background-color: var(--text-secondary);
    transition: .2s;
    border-radius: 50%;
  }

  .switch-container input:checked + .slider {
    background-color: var(--accent-yellow);
    border-color: var(--accent-yellow);
  }

  .switch-container input:checked + .slider:before {
    transform: translateX(12px);
    background-color: #000;
  }

  .switch-label {
    font-weight: 500;
    transition: var(--transition);
  }

  .filter-switch:hover .switch-label {
    color: var(--text-primary);
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
    transition: border-color 0.2s, background 0.2s, transform 0.15s;
    cursor: pointer;
  }

  .msg:hover {
    background: rgba(255, 255, 255, 0.07);
    border-color: rgba(255, 255, 255, 0.15);
  }

  .msg:active {
    transform: scale(0.985);
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

  .guest-name-bar {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 8px;
  }

  .guest-label {
    font-size: 0.8rem;
    color: var(--text-secondary);
    white-space: nowrap;
  }

  .guest-input {
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm);
    color: var(--text-primary);
    padding: 4px 8px;
    font-size: 0.85rem;
    width: 100%;
    outline: none;
  }

  .guest-input:focus {
    border-color: var(--accent-yellow);
  }

  .drawing-badge {
    background: rgba(245, 158, 11, 0.15);
    color: var(--accent-yellow, #f59e0b);
    border: 1px solid rgba(245, 158, 11, 0.35);
    font-size: 0.8rem;
    width: 22px;
    height: 22px;
    border-radius: 50%;
    cursor: pointer;
    margin-left: 4px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    padding: 0;
    line-height: 1;
    transition: var(--transition);
  }

  .drawing-badge:hover {
    background: rgba(245, 158, 11, 0.35);
    transform: scale(1.15);
  }

  .input-actions-row {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-top: 4px;
  }

  .pencil-btn {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    padding: 5px 10px;
    font-size: 0.78rem;
    font-weight: 500;
    color: var(--text-secondary);
    background: var(--surface-hover);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm);
    cursor: pointer;
    transition: var(--transition);
  }

  .pencil-btn:hover {
    color: var(--text-primary);
    background: rgba(255, 255, 255, 0.1);
  }

  .pencil-btn.active {
    color: #0f172a;
    background: var(--accent-yellow, #f59e0b);
    border-color: var(--accent-yellow, #f59e0b);
    font-weight: 600;
  }


</style>
