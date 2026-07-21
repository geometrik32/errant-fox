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
    hoveredCommentId?: number | null;
    oncommenthover?: (id: number) => void;
    oncommentleave?: () => void;
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
    hoveredCommentId = null,
    readonly = false,
    shareToken = '',
    sharedBoutId = null,
    isDrawingMode = false,
    drawingStrokes = $bindable([]),
    bouts = [],
    oncommenthover,
    oncommentleave,
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

  let activeThreadParentId = $state<number | null>(null);

  // Auto-open thread if highlightedId is a reply
  $effect(() => {
    if (highlightedId) {
      const target = comments.find(c => c.id === highlightedId);
      if (target && target.reply_to_id) {
        activeThreadParentId = target.reply_to_id;
      }
    }
  });

  let activeThreadParent = $derived(
    activeThreadParentId != null ? comments.find(c => c.id === activeThreadParentId) : null
  );

  let activeThreadReplies = $derived(
    activeThreadParentId != null
      ? comments.filter(c => c.reply_to_id === activeThreadParentId).sort((a, b) => a.id - b.id)
      : []
  );

  let sharedBout = $derived(
    sharedBoutId ? bouts.find(b => b.id === sharedBoutId) : null
  );
  let text = $state('');
  let threadText = $state('');
  let sending = $state(false);
  let listEl = $state<HTMLDivElement | null>(null);
  let threadListEl = $state<HTMLDivElement | null>(null);
  let textareaEl = $state<HTMLTextAreaElement | null>(null);
  let threadTextareaEl = $state<HTMLTextAreaElement | null>(null);

  let filterByActiveBout = $state(false);
  let filterByTimeline = $state(false);
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

  let topLevelComments = $derived.by(() => {
    let filtered = comments.filter(c => !c.reply_to_id);
    if (sharedBout) {
      filtered = filtered.filter(c => {
        if (highlightedId && c.id === highlightedId) return true;
        return c.timestamp_ms >= sharedBout.time_start_ms && c.timestamp_ms <= sharedBout.time_end_ms;
      });
    } else if (filterByTimeline) {
      filtered = filtered.filter(c => {
        if (highlightedId && c.id === highlightedId) return true;
        const diff = currentTimeMs - c.timestamp_ms;
        return diff >= -500 && diff <= 15000;
      });
    } else if (filterByActiveBout) {
      if (currentBout) {
        filtered = filtered.filter(c => {
          if (highlightedId && c.id === highlightedId) return true;
          return c.timestamp_ms >= currentBout.time_start_ms && c.timestamp_ms <= currentBout.time_end_ms;
        });
      } else {
        filtered = highlightedId ? filtered.filter(c => c.id === highlightedId) : [];
      }
    }
    return filtered.sort((a, b) => {
      if (a.timestamp_ms !== b.timestamp_ms) {
        return a.timestamp_ms - b.timestamp_ms;
      }
      return a.id - b.id;
    });
  });

  function getReplyCount(parentId: number): number {
    return comments.filter(c => c.reply_to_id === parentId).length;
  }

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

  function openThread(c: Comment) {
    const rootId = c.reply_to_id || c.id;
    activeThreadParentId = rootId;
    const parentComment = comments.find(x => x.id === rootId) || c;
    handleCommentClick(parentComment);
  }

  function closeThread() {
    activeThreadParentId = null;
    drawingStrokes = [];
    onclosedrawing?.();
  }

  function toggleDrawingMode() {
    if (isDrawingMode) {
      onclosedrawing?.();
    } else {
      onseek?.(currentTime * 1000);
      onstartdrawing?.();
    }
  }

  async function submit(isThreadSubmit: boolean = false) {
    const inputText = isThreadSubmit ? threadText : text;
    const t = inputText.trim();
    if (!t || sending) return;
    sending = true;
    try {
      let created: Comment;
      let commentTimeMs = Math.round(currentTime * 1000);
      if (sharedBout) {
        commentTimeMs = Math.max(sharedBout.time_start_ms, Math.min(sharedBout.time_end_ms, commentTimeMs));
      }
      let drawingDataStr: string | null = null;
      if (drawingStrokes && drawingStrokes.length > 0) {
        drawingDataStr = JSON.stringify({ version: 1, strokes: drawingStrokes });
      }

      const targetReplyId = isThreadSubmit ? activeThreadParentId : null;

      if (shareToken || !$currentUser) {
        created = await createSharedComment({
          videoId,
          token: shareToken || localStorage.getItem('ef_token') || '',
          nickname: guestNickname || 'Гость',
          text: t,
          timestamp_ms: commentTimeMs,
          reply_to_id: targetReplyId,
          bout_id: sharedBoutId,
          drawing: drawingDataStr,
        });
      } else {
        created = await createComment({
          video_id: videoId,
          timestamp_ms: commentTimeMs,
          text: t,
          reply_to_id: targetReplyId,
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
      if (isThreadSubmit) {
        threadText = '';
      } else {
        text = '';
      }
      drawingStrokes = [];
      onclosedrawing?.();
      requestAnimationFrame(() => {
        const targetEl = isThreadSubmit ? threadListEl : listEl;
        if (targetEl) targetEl.scrollTop = targetEl.scrollHeight;
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
    onseek?.(c.timestamp_ms);
    const strokes = parseDrawing(c.drawing);
    drawingStrokes = strokes ? [...strokes] : [];
    onstartdrawing?.();
  }

  function cancelEdit() {
    editingId = null;
    editText = '';
    drawingStrokes = [];
    onclosedrawing?.();
  }

  async function submitEdit(id: number) {
    const t = editText.trim();
    if (!t) return;
    try {
      let drawingDataStr: string | null = null;
      if (drawingStrokes && drawingStrokes.length > 0) {
        drawingDataStr = JSON.stringify({ version: 1, strokes: drawingStrokes });
      }
      const updated = await updateComment(id, t, drawingDataStr);
      comments = comments.map(c => c.id === id ? updated : c);
      oncommentschange?.(comments);
      editingId = null;
      editText = '';
      drawingStrokes = [];
      onclosedrawing?.();
    } catch (err) {
      alert(err instanceof Error ? err.message : 'Ошибка обновления комментария');
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
    if (activeThreadParentId === id) {
      activeThreadParentId = null;
    }
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

  function onKeydown(e: KeyboardEvent, isThreadSubmit: boolean = false) {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      submit(isThreadSubmit);
    }
  }

  // Scroll to + flash highlighted comment
  $effect(() => {
    const id = highlightedId;
    if (!id || (filterByTimeline && activeThreadParentId === null)) return;
    const targetEl = activeThreadParentId !== null ? threadListEl : listEl;
    if (!targetEl) return;
    const el = targetEl.querySelector<HTMLElement>(`[data-comment-id="${id}"]`);
    if (!el) return;
    el.scrollIntoView({ block: 'nearest', behavior: 'smooth' });
    el.classList.remove('msg--flash');
    void el.offsetWidth;
    el.classList.add('msg--flash');
  });

  // Scroll to hovered comment from timeline marker
  $effect(() => {
    const id = hoveredCommentId;
    if (!id || (filterByTimeline && activeThreadParentId === null)) return;
    const targetEl = activeThreadParentId !== null ? threadListEl : listEl;
    if (!targetEl) return;
    const el = targetEl.querySelector<HTMLElement>(`[data-comment-id="${id}"]`);
    if (el) {
      el.scrollIntoView({ block: 'nearest', behavior: 'smooth' });
    }
  });

  // Auto-scroll to bottom in Live Stream mode
  $effect(() => {
    if (filterByTimeline && activeThreadParentId === null && listEl) {
      topLevelComments;
      requestAnimationFrame(() => {
        if (listEl) listEl.scrollTop = listEl.scrollHeight;
      });
    }
  });

  // WebSocket handler
  export function handleWsMessage(msg: Record<string, unknown>) {
    if (msg.type === 'new_comment' && msg.video_id === videoId) {
      const { type: _t, video_id: _v, ...fields } = msg;
      const incoming = fields as unknown as Comment;
      const idx = comments.findIndex(c => c.id === incoming.id);
      if (idx >= 0) {
        comments = comments.map((c, i) => i === idx ? incoming : c);
      } else {
        comments = [...comments, incoming];
      }
      oncommentschange?.(comments);
    } else if (msg.type === 'update_comment' && msg.video_id === videoId) {
      const { type: _t, video_id: _v, ...fields } = msg;
      const incoming = fields as unknown as Comment;
      comments = comments.map(c => c.id === incoming.id ? incoming : c);
      oncommentschange?.(comments);
    } else if (msg.type === 'delete_comment' && msg.video_id === videoId) {
      const id = msg.id as number;
      if (activeThreadParentId === id) {
        activeThreadParentId = null;
      }
      comments = comments.filter(c => c.id !== id);
      oncommentschange?.(comments);
    }
  }
</script>

<div class="chat">
  <div class="chat-slider-viewport">
    <div class="chat-views-container" class:in-thread={activeThreadParentId !== null}>
      
      <!-- VIEW 1: Main Comments List (1st order comments) -->
      <div class="chat-view view-main">
        <!-- Header -->
        <div class="chat-header">
          <span class="chat-title">Комментарии ({topLevelComments.length})</span>
          {#if !sharedBoutId}
            <label class="filter-switch" title="Живой чат: комментарии всплывают снизу по мере воспроизведения">
              <div class="switch-container">
                <input 
                  type="checkbox" 
                  bind:checked={filterByTimeline} 
                />
                <span class="slider"></span>
              </div>
              <span class="switch-label">
                Живой чат
              </span>
            </label>
          {/if}
        </div>

        <!-- Message list -->
        <div class="list" class:live-stream-mode={filterByTimeline} bind:this={listEl}>
          {#if filterByTimeline && topLevelComments.length === 0}
            <div class="live-stream-empty">
              <span class="live-stream-empty-icon">💬</span>
              <span>Комментарии появятся по мере таймлайна...</span>
            </div>
          {/if}
          {#each topLevelComments as c (c.id)}
            <!-- svelte-ignore a11y_click_events_have_key_events -->
            <!-- svelte-ignore a11y_no_static_element_interactions -->
            <div
              class="msg"
              class:highlighted={highlightedId === c.id}
              class:hovered={hoveredCommentId === c.id}
              data-comment-id={c.id}
              onclick={() => handleCommentClick(c)}
              onmouseenter={() => oncommenthover?.(c.id)}
              onmouseleave={() => oncommentleave?.()}
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
                  <span class="drawing-badge" title="Прикреплен рисунок">🎨</span>
                {/if}
                <span class="ts">{fmtMs(c.timestamp_ms)}</span>
              </div>

              {#if editingId === c.id}
                <div class="edit-area" onclick={(e) => e.stopPropagation()}>
                  <textarea
                    class="input-glass edit-inp"
                    bind:value={editText}
                    rows="2"
                    onkeydown={(e) => { if (e.key === 'Enter' && !e.shiftKey) { e.preventDefault(); submitEdit(c.id); } if (e.key === 'Escape') cancelEdit(); }}
                  ></textarea>
                  <div class="edit-actions-row">
                    <button class="btn btn-primary btn-sm edit-btn" onclick={() => submitEdit(c.id)}>Сохранить</button>
                    <button class="btn btn-outline btn-sm edit-btn" onclick={cancelEdit}>Отмена</button>
                    <button
                      type="button"
                      class="pencil-btn edit-btn"
                      class:active={isDrawingMode || (drawingStrokes && drawingStrokes.length > 0)}
                      onclick={toggleDrawingMode}
                      title="Нарисовать / отредактировать рисунок"
                    >
                      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <path d="M12 20h9" />
                        <path d="M16.5 3.5a2.121 2.121 0 0 1 3 3L7 19l-4 1 1-4L16.5 3.5z" />
                      </svg>
                      <span>{drawingStrokes && drawingStrokes.length > 0 ? 'Рисунок' : 'Рисовать'}</span>
                    </button>
                  </div>
                </div>
              {:else}
                <div class="msg-text">{c.text}</div>
              {/if}

              {#if editingId !== c.id}
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

                  {#if $currentUser?.id === c.author.id}
                    <button class="edit-link" onclick={(e) => { e.stopPropagation(); startEdit(c); }} title="Редактировать">
                      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                        <path d="M12 20h9" />
                        <path d="M16.5 3.5a2.121 2.121 0 0 1 3 3L7 19l-4 1 1-4L16.5 3.5z" />
                      </svg>
                    </button>
                  {/if}
                  
                  <div class="msg-footer-center">
                    <button
                      class="thread-btn"
                      onclick={(e) => { e.stopPropagation(); openThread(c); }}
                    >
                      {#if getReplyCount(c.id) > 0}
                        Ответы ({getReplyCount(c.id)})
                      {:else}
                        Ответить
                      {/if}
                    </button>
                  </div>

                  {#if $currentUser?.id === c.author.id || ($currentUser && c.author.id === 'guest')}
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
              {/if}
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
          <textarea
            bind:this={textareaEl}
            class="input-glass"
            bind:value={text}
            onkeydown={(e) => onKeydown(e, false)}
            placeholder="Оставить главный комментарий… (Enter — отправить)"
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

      <!-- VIEW 2: Thread View (1st order comment + 2nd order replies) -->
      <div class="chat-view view-thread">
        {#if activeThreadParent}
          <div class="chat-header thread-header">
            <button class="back-btn" onclick={closeThread} title="Вернуться ко всем комментариям">
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <line x1="19" y1="12" x2="5" y2="12"></line>
                <polyline points="12 19 5 12 12 5"></polyline>
              </svg>
              <span>Назад</span>
            </button>
            <span class="chat-title">Ветка обсуждения</span>
          </div>

          <div class="list thread-list" bind:this={threadListEl}>
            <!-- Pinned Parent Comment (1st Order) -->
            <!-- svelte-ignore a11y_click_events_have_key_events -->
            <!-- svelte-ignore a11y_no_static_element_interactions -->
            <div
              class="msg msg-parent glass-card"
              onclick={() => handleCommentClick(activeThreadParent!)}
            >
              <div class="msg-head">
                <div class="avatar">
                  <svg class="avatar-icon" width="14" height="14" viewBox="0 0 24 24" fill="none" aria-hidden="true">
                    <circle cx="12" cy="8" r="4" stroke="currentColor" stroke-width="1.5"/>
                    <path d="M4 20c0-4 3.6-7 8-7s8 3 8 7" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
                  </svg>
                  <img src={activeThreadParent.author.avatar_url} alt={activeThreadParent.author.display_name} onerror={(e) => { (e.target as HTMLImageElement).style.display = 'none'; }} />
                </div>
                <span class="name">{activeThreadParent.author.display_name}</span>
                {#if activeThreadParent.drawing}
                  <span class="drawing-badge" title="Прикреплен рисунок">🎨</span>
                {/if}
                <span class="ts">{fmtMs(activeThreadParent.timestamp_ms)}</span>
              </div>

              {#if editingId === activeThreadParent.id}
                <div class="edit-area" onclick={(e) => e.stopPropagation()}>
                  <textarea
                    class="input-glass edit-inp"
                    bind:value={editText}
                    rows="2"
                    onkeydown={(e) => { if (e.key === 'Enter' && !e.shiftKey) { e.preventDefault(); submitEdit(activeThreadParent!.id); } if (e.key === 'Escape') cancelEdit(); }}
                  ></textarea>
                  <div class="edit-actions-row">
                    <button class="btn btn-primary btn-sm edit-btn" onclick={() => submitEdit(activeThreadParent!.id)}>Сохранить</button>
                    <button class="btn btn-outline btn-sm edit-btn" onclick={cancelEdit}>Отмена</button>
                    <button
                      type="button"
                      class="pencil-btn edit-btn"
                      class:active={isDrawingMode || (drawingStrokes && drawingStrokes.length > 0)}
                      onclick={toggleDrawingMode}
                      title="Нарисовать / отредактировать рисунок"
                    >
                      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <path d="M12 20h9" />
                        <path d="M16.5 3.5a2.121 2.121 0 0 1 3 3L7 19l-4 1 1-4L16.5 3.5z" />
                      </svg>
                      <span>{drawingStrokes && drawingStrokes.length > 0 ? 'Рисунок' : 'Рисовать'}</span>
                    </button>
                  </div>
                </div>
              {:else}
                <div class="msg-text">{activeThreadParent.text}</div>
              {/if}

              {#if editingId !== activeThreadParent.id}
                <div class="msg-footer" onclick={(e) => e.stopPropagation()}>
                  <button
                    class="react-btn"
                    class:active={activeThreadParent.my_reaction === 'like'}
                    onclick={(e) => { e.stopPropagation(); handleReact(activeThreadParent!, 'like'); }}
                    title="Нравится"
                  >👍 {#if activeThreadParent.likes > 0}<span class="react-count">{activeThreadParent.likes}</span>{/if}</button>
                  <button
                    class="react-btn"
                    class:active={activeThreadParent.my_reaction === 'dislike'}
                    onclick={(e) => { e.stopPropagation(); handleReact(activeThreadParent!, 'dislike'); }}
                    title="Не нравится"
                  >👎 {#if activeThreadParent.dislikes > 0}<span class="react-count">{activeThreadParent.dislikes}</span>{/if}</button>
                  {#if $currentUser?.id === activeThreadParent.author.id || ($currentUser && activeThreadParent.author.id === 'guest')}
                    {#if $currentUser?.id === activeThreadParent.author.id}
                      <button class="edit-link" onclick={(e) => { e.stopPropagation(); startEdit(activeThreadParent!); }} title="Редактировать">
                        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                          <path d="M12 20h9" />
                          <path d="M16.5 3.5a2.121 2.121 0 0 1 3 3L7 19l-4 1 1-4L16.5 3.5z" />
                        </svg>
                      </button>
                    {/if}
                    <button class="del-link" onclick={(e) => { e.stopPropagation(); promptDelete(activeThreadParent!.id); }} title="Удалить" style="margin-left: auto;">
                      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                        <polyline points="3 6 5 6 21 6"></polyline>
                        <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path>
                        <line x1="10" y1="11" x2="10" y2="17"></line>
                        <line x1="14" y1="11" x2="14" y2="17"></line>
                      </svg>
                    </button>
                  {/if}
                </div>
              {/if}
            </div>

            <!-- Replies Divider -->
            <div class="thread-divider">
              <span>Ответы ({activeThreadReplies.length})</span>
            </div>

            <!-- Replies List (2nd Order) -->
            {#each activeThreadReplies as r (r.id)}
              <!-- svelte-ignore a11y_click_events_have_key_events -->
              <!-- svelte-ignore a11y_no_static_element_interactions -->
              <div
                class="msg msg-reply"
                class:highlighted={highlightedId === r.id}
                data-comment-id={r.id}
                onclick={() => handleCommentClick(r)}
              >
                <div class="msg-head">
                  <div class="avatar avatar-split">
                    {#if activeThreadParent?.author.avatar_url}
                      <img class="avatar-half avatar-left" src={activeThreadParent.author.avatar_url} alt={activeThreadParent.author.display_name} onerror={(e) => { (e.target as HTMLImageElement).style.display = 'none'; }} />
                    {:else}
                      <div class="avatar-half avatar-left avatar-fallback">
                        <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><circle cx="12" cy="8" r="4"/><path d="M4 20c0-4 3.6-7 8-7s8 3 8 7"/></svg>
                      </div>
                    {/if}
                    {#if r.author.avatar_url}
                      <img class="avatar-half avatar-right" src={r.author.avatar_url} alt={r.author.display_name} onerror={(e) => { (e.target as HTMLImageElement).style.display = 'none'; }} />
                    {:else}
                      <div class="avatar-half avatar-right avatar-fallback">
                        <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><circle cx="12" cy="8" r="4"/><path d="M4 20c0-4 3.6-7 8-7s8 3 8 7"/></svg>
                      </div>
                    {/if}
                  </div>
                  <span class="name">{r.author.display_name}</span>
                  {#if r.drawing}
                    <span class="drawing-badge" title="Прикреплен рисунок">🎨</span>
                  {/if}
                  <span class="ts">{fmtMs(r.timestamp_ms)}</span>
                </div>

                {#if editingId === r.id}
                  <div class="edit-area" onclick={(e) => e.stopPropagation()}>
                    <textarea
                      class="input-glass edit-inp"
                      bind:value={editText}
                      rows="2"
                      onkeydown={(e) => { if (e.key === 'Enter' && !e.shiftKey) { e.preventDefault(); submitEdit(r.id); } if (e.key === 'Escape') cancelEdit(); }}
                    ></textarea>
                    <div class="edit-actions-row">
                      <button class="btn btn-primary btn-sm edit-btn" onclick={() => submitEdit(r.id)}>Сохранить</button>
                      <button class="btn btn-outline btn-sm edit-btn" onclick={cancelEdit}>Отмена</button>
                      <button
                        type="button"
                        class="pencil-btn edit-btn"
                        class:active={isDrawingMode || (drawingStrokes && drawingStrokes.length > 0)}
                        onclick={toggleDrawingMode}
                        title="Нарисовать / отредактировать рисунок"
                      >
                        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                          <path d="M12 20h9" />
                          <path d="M16.5 3.5a2.121 2.121 0 0 1 3 3L7 19l-4 1 1-4L16.5 3.5z" />
                        </svg>
                        <span>{drawingStrokes && drawingStrokes.length > 0 ? 'Рисунок' : 'Рисовать'}</span>
                      </button>
                    </div>
                  </div>
                {:else}
                  <div class="msg-text">{r.text}</div>
                {/if}

                {#if editingId !== r.id}
                  <div class="msg-footer" onclick={(e) => e.stopPropagation()}>
                    <button
                      class="react-btn"
                      class:active={r.my_reaction === 'like'}
                      onclick={(e) => { e.stopPropagation(); handleReact(r, 'like'); }}
                      title="Нравится"
                    >👍 {#if r.likes > 0}<span class="react-count">{r.likes}</span>{/if}</button>
                    <button
                      class="react-btn"
                      class:active={r.my_reaction === 'dislike'}
                      onclick={(e) => { e.stopPropagation(); handleReact(r, 'dislike'); }}
                      title="Не нравится"
                    >👎 {#if r.dislikes > 0}<span class="react-count">{r.dislikes}</span>{/if}</button>

                    {#if $currentUser?.id === r.author.id}
                      <button class="edit-link" onclick={(e) => { e.stopPropagation(); startEdit(r); }} title="Редактировать">
                        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                          <path d="M12 20h9" />
                          <path d="M16.5 3.5a2.121 2.121 0 0 1 3 3L7 19l-4 1 1-4L16.5 3.5z" />
                        </svg>
                      </button>
                    {/if}

                    {#if $currentUser?.id === r.author.id || ($currentUser && r.author.id === 'guest')}
                      <button class="del-link" onclick={(e) => { e.stopPropagation(); promptDelete(r.id); }} title="Удалить" style="margin-left: auto;">
                        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                          <polyline points="3 6 5 6 21 6"></polyline>
                          <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path>
                          <line x1="10" y1="11" x2="10" y2="17"></line>
                          <line x1="14" y1="11" x2="14" y2="17"></line>
                        </svg>
                      </button>
                    {/if}
                  </div>
                {/if}
              </div>
            {:else}
              <div class="empty">В этой ветке пока нет ответов. Будьте первым!</div>
            {/each}
          </div>

          <!-- Thread Input area -->
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
            <textarea
              bind:this={threadTextareaEl}
              class="input-glass"
              bind:value={threadText}
              onkeydown={(e) => onKeydown(e, true)}
              placeholder="Написать ответ в тред… (Enter — отправить)"
              rows="2"
              disabled={sending}
            ></textarea>

            <div class="input-actions-row">
              <button
                type="button"
                class="pencil-btn"
                class:active={isDrawingMode || (drawingStrokes && drawingStrokes.length > 0)}
                onclick={toggleDrawingMode}
                title="Нарисовать поверх кадра видео для этого ответа"
              >
                <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M12 20h9" />
                  <path d="M16.5 3.5a2.121 2.121 0 0 1 3 3L7 19l-4 1 1-4L16.5 3.5z" />
                </svg>
                <span>Рисовать</span>
              </button>
            </div>
          </div>
        {/if}
      </div>

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

  .chat-slider-viewport {
    position: relative;
    width: 100%;
    height: 100%;
    overflow: hidden;
  }

  .chat-views-container {
    display: flex;
    width: 200%;
    height: 100%;
    transition: transform 0.35s cubic-bezier(0.4, 0, 0.2, 1);
  }

  .chat-views-container.in-thread {
    transform: translateX(-50%);
  }

  .chat-view {
    width: 50%;
    height: 100%;
    display: flex;
    flex-direction: column;
    box-sizing: border-box;
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

  .thread-header {
    justify-content: flex-start;
    gap: 12px;
  }

  .back-btn {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    background: rgba(255, 255, 255, 0.08);
    border: 1px solid var(--border-color);
    color: var(--text-primary);
    padding: 4px 10px;
    border-radius: var(--radius-sm, 6px);
    font-size: 0.8rem;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .back-btn:hover {
    background: rgba(255, 255, 255, 0.15);
    border-color: var(--color-primary);
    color: #fff;
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

  input:checked + .slider {
    background-color: var(--color-primary);
    border-color: var(--color-primary);
  }

  input:checked + .slider:before {
    transform: translateX(12px);
    background-color: #fff;
  }

  .switch-label {
    font-size: 0.75rem;
    white-space: nowrap;
  }

  /* ── List ── */
  .list {
    flex: 1;
    overflow-y: auto;
    padding: 12px;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .thread-list {
    gap: 12px;
  }

  .thread-divider {
    display: flex;
    align-items: center;
    margin: 8px 0 4px;
    font-size: 0.75rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--color-primary);
    border-bottom: 1px dashed rgba(255, 255, 255, 0.15);
    padding-bottom: 4px;
  }

  .msg-parent {
    border-left: 3px solid var(--color-primary) !important;
    background: rgba(255, 255, 255, 0.05);
  }

  .msg-reply {
    margin-left: 8px;
    border-left: 2px solid rgba(255, 255, 255, 0.15);
  }

  .msg {
    padding: 10px 12px;
    border-radius: var(--radius-md);
    background: var(--surface-color);
    border: 1px solid var(--border-color);
    cursor: pointer;
    transition: background 0.15s ease, border-color 0.15s ease, box-shadow 0.15s ease;
    display: flex;
    flex-direction: column;
    gap: 6px;

    &:hover, &.hovered {
      background: var(--surface-hover-color);
      border-color: rgba(255, 255, 255, 0.3);
    }

    &.highlighted {
      border-color: var(--color-primary);
      box-shadow: 0 0 12px rgba(var(--color-primary-rgb, 99, 102, 241), 0.35);
    }

    :global(&.msg--flash) {
      animation: comment-white-flash 1.2s ease-out;
    }
  }

  @keyframes comment-white-flash {
    0% {
      border-color: #ffffff;
      box-shadow: 0 0 16px rgba(255, 255, 255, 0.9);
    }
    50% {
      border-color: #ffffff;
      box-shadow: 0 0 10px rgba(255, 255, 255, 0.6);
    }
    100% {
      border-color: var(--border-color);
      box-shadow: none;
    }
  }

  .msg-head {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .avatar {
    width: 24px;
    height: 24px;
    border-radius: 50%;
    overflow: hidden;
    background: rgba(255, 255, 255, 0.1);
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    position: relative;
  }

  .avatar img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .avatar-split {
    display: flex;
    align-items: center;
    justify-content: space-between;
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(255, 255, 255, 0.12);
  }

  .avatar-half {
    width: 50%;
    height: 100%;
    object-fit: cover;
  }

  .avatar-left {
    border-right: 1px solid rgba(0, 0, 0, 0.4);
  }

  .avatar-right {
    border-left: 1px solid rgba(255, 255, 255, 0.2);
  }

  .avatar-fallback {
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(255, 255, 255, 0.08);
    color: var(--text-tertiary);
  }

  .name {
    font-size: 0.8rem;
    font-weight: 600;
    color: var(--text-primary);
  }

  .drawing-badge {
    font-size: 0.85rem;
    cursor: help;
  }

  .ts {
    font-size: 0.75rem;
    color: var(--color-primary);
    font-weight: 500;
    margin-left: auto;
    background: rgba(99, 102, 241, 0.1);
    padding: 2px 6px;
    border-radius: var(--radius-sm);
  }

  .msg-text {
    font-size: 0.85rem;
    color: var(--text-secondary);
    line-height: 1.4;
    white-space: pre-wrap;
    word-break: break-word;
  }

  .msg-footer {
    display: flex;
    align-items: center;
    gap: 6px;
    margin-top: 2px;
  }

  .msg-footer-center {
    flex: 1;
    display: flex;
    justify-content: center;
  }

  .react-btn {
    background: transparent;
    border: none;
    font-size: 0.75rem;
    color: var(--text-tertiary);
    cursor: pointer;
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 2px 4px;
    border-radius: var(--radius-sm);
    transition: background 0.15s ease;

    &:hover {
      background: rgba(255, 255, 255, 0.08);
      color: var(--text-primary);
    }

    &.active {
      color: var(--color-primary);
    }
  }

  .react-count {
    font-weight: 600;
  }

  .thread-btn {
    background: rgba(99, 102, 241, 0.12);
    border: 1px solid rgba(99, 102, 241, 0.25);
    color: var(--color-primary);
    font-size: 0.75rem;
    font-weight: 600;
    cursor: pointer;
    padding: 3px 10px;
    border-radius: var(--radius-pill);
    transition: all 0.15s ease;
    display: inline-flex;
    align-items: center;
    justify-content: center;

    &:hover {
      background: var(--color-primary);
      color: #fff;
      border-color: var(--color-primary);
    }
  }

  .edit-link,
  .del-link {
    background: transparent;
    border: none;
    font-size: 0.75rem;
    color: var(--text-tertiary);
    cursor: pointer;
    padding: 2px 4px;

    &:hover {
      color: var(--text-primary);
    }
  }

  .del-link:hover {
    color: #ef4444;
  }

  .empty {
    text-align: center;
    color: var(--text-tertiary);
    font-size: 0.85rem;
    padding: 32px 16px;
  }

  /* ── Edit Area ── */
  .edit-area {
    display: flex;
    flex-direction: column;
    gap: 6px;
    margin-top: 4px;
  }

  .edit-inp {
    width: 100%;
    box-sizing: border-box;
  }

  .edit-actions-row {
    display: flex;
    align-items: center;
    gap: 6px;
    margin-top: 6px;
    width: 100%;
    box-sizing: border-box;
  }

  .edit-actions-row .edit-btn {
    flex: 1 1 0px;
    min-width: 0;
    padding: 4px 6px;
    font-size: 0.75rem;
    white-space: nowrap;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    box-sizing: border-box;
    height: 28px;
  }

  /* ── Input Area ── */
  .input-area {
    padding: 12px 16px;
    border-top: 1px solid var(--border-color);
    background: rgba(0, 0, 0, 0.2);
    display: flex;
    flex-direction: column;
    gap: 8px;
    flex-shrink: 0;
  }

  .guest-name-bar {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 0.75rem;
    color: var(--text-secondary);
  }

  .guest-input {
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid var(--border-color);
    color: var(--text-primary);
    padding: 2px 8px;
    border-radius: var(--radius-sm);
    font-size: 0.75rem;
    width: 120px;
  }

  .input-glass {
    width: 100%;
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-md);
    color: var(--text-primary);
    padding: 8px 12px;
    font-size: 0.85rem;
    resize: none;
    box-sizing: border-box;

    &:focus {
      outline: none;
      border-color: var(--color-primary);
      background: rgba(255, 255, 255, 0.08);
    }
  }

  .input-actions-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .pencil-btn {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    background: rgba(255, 255, 255, 0.06);
    border: 1px solid var(--border-color);
    color: var(--text-secondary);
    padding: 4px 10px;
    border-radius: var(--radius-sm);
    font-size: 0.75rem;
    cursor: pointer;
    transition: all 0.15s ease;

    &:hover {
      background: rgba(255, 255, 255, 0.12);
      color: var(--text-primary);
    }

    &.active {
      background: rgba(99, 102, 241, 0.2);
      border-color: var(--color-primary);
      color: var(--color-primary);
    }
  }

  /* ── Live Stream Mode Styles ── */
  .list.live-stream-mode {
    display: flex;
    flex-direction: column;
    justify-content: flex-end;
    gap: 10px;
    padding: 12px;
    overflow-y: auto;
    height: 100%;
  }

  .list.live-stream-mode .msg {
    margin-bottom: 0;
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.35);
    backdrop-filter: blur(12px);
    border: 1px solid rgba(255, 255, 255, 0.12);
    border-left: 3px solid var(--color-primary, #6366f1);
    animation: liveMsgSlideUp 0.3s cubic-bezier(0.16, 1, 0.3, 1);
  }

  @keyframes liveMsgSlideUp {
    from {
      opacity: 0;
      transform: translateY(20px) scale(0.97);
    }
    to {
      opacity: 1;
      transform: translateY(0) scale(1);
    }
  }

  .live-stream-empty {
    margin: auto;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    color: var(--text-muted, rgba(255, 255, 255, 0.4));
    font-size: 0.85rem;
    text-align: center;
  }

  .live-stream-empty-icon {
    font-size: 1.5rem;
  }
</style>
