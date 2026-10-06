<script lang="ts">
  import { onMount } from 'svelte';
  import { currentUser, unreadNotificationsCount, refreshUnreadNotificationsCount, notificationsRevision } from '../stores';
  import { fetchNotifications, markReadAll, markReadByVideo, markReadBout, markReadComment, markReadMultiple } from '../lib/api/notifications';
  import { searchComments } from '../lib/api/comments';
  import type { NotificationItem, SearchResult } from '../lib/api/types';
  import { resolveColor, getContrastColor } from '../lib/api/types';

  // ── Filters & LocalStorage ──────────────────────────────────────────────────
  interface FilterState {
    replies: boolean;
    videoTags: boolean;
    bouts: boolean;
  }

  const STORAGE_KEY = 'ef_activity_filters';

  function loadFilters(): FilterState {
    if (typeof localStorage === 'undefined') return { replies: true, videoTags: true, bouts: true };
    try {
      const saved = localStorage.getItem(STORAGE_KEY);
      if (saved) return JSON.parse(saved);
    } catch {}
    return { replies: true, videoTags: true, bouts: true };
  }

  let filters = $state<FilterState>(loadFilters());

  function toggleFilter(key: keyof FilterState) {
    filters[key] = !filters[key];
    if (typeof localStorage !== 'undefined') {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(filters));
    }
  }

  // ── Data & Loading ──────────────────────────────────────────────────────────
  let loading = $state(true);
  let notifications = $state<NotificationItem[]>([]);
  let total = $state(0);

  async function loadData() {
    loading = true;
    try {
      const res = await fetchNotifications({ limit: 150 });
      notifications = res.items;
      total = res.total;
      unreadNotificationsCount.set(res.unread_count);
    } catch (e) {
      console.error('Failed to load notifications:', e);
    } finally {
      loading = false;
    }
  }

  async function loadDataSilently() {
    try {
      const res = await fetchNotifications({ limit: 150 });
      notifications = res.items;
      total = res.total;
    } catch (e) {
      console.error('Failed to silently reload notifications:', e);
    }
  }

  $effect(() => {
    const rev = $notificationsRevision;
    if (rev > 0) {
      loadDataSilently();
    }
  });

  onMount(() => {
    loadData();
  });

  // ── Search Mode (subtle inline tool) ────────────────────────────────────────
  let searchQuery = $state('');
  let searchResults = $state<SearchResult[]>([]);
  let searching = $state(false);
  let searched = $state(false);

  let searchTimeout: any;
  function handleSearchInput() {
    clearTimeout(searchTimeout);
    const q = searchQuery.trim();
    if (!q) {
      searchResults = [];
      searched = false;
      return;
    }
    searchTimeout = setTimeout(async () => {
      searching = true;
      try {
        searchResults = await searchComments(q);
        searched = true;
      } catch (e) {
        console.error(e);
      } finally {
        searching = false;
      }
    }, 300);
  }

  function clearSearch() {
    searchQuery = '';
    searchResults = [];
    searched = false;
  }

  // ── Formatters ──────────────────────────────────────────────────────────────
  function fmtMs(ms: number): string {
    const t = Math.floor(ms / 1000);
    const m = Math.floor(t / 60);
    const s = t % 60;
    return `${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}`;
  }

  function fmtDate(iso: string): string {
    if (!iso) return '';
    try {
      const d = new Date(iso);
      return d.toLocaleDateString('ru-RU', {
        day: '2-digit',
        month: '2-digit',
        hour: '2-digit',
        minute: '2-digit',
      });
    } catch {
      return iso;
    }
  }

  // ── Filtering & Grouping ───────────────────────────────────────────────────
  let filteredNotifications = $derived.by(() => {
    return notifications.filter(n => {
      if (n.event_type === 'comment_reply' || n.event_type === 'video_comment') {
        return filters.replies;
      }
      if (n.event_type === 'video_tag' || n.event_type === 'video_labeled') {
        return filters.videoTags;
      }
      if (n.event_type === 'bout_added' || n.event_type === 'bout_updated') {
        return filters.bouts;
      }
      return true;
    });
  });

  let unreadNotifications = $derived(filteredNotifications.filter(n => !n.is_read));
  let readNotifications = $derived(filteredNotifications.filter(n => n.is_read));

  // ── Group Types ────────────────────────────────────────────────────────────
  interface GroupedBouts {
    kind: 'bouts_group';
    groupId: string;
    video_id: string;
    video_title: string;
    items: NotificationItem[];
    actor: { id: string; display_name: string; avatar_url?: string | null; color?: string | null };
    created_at: string;
    is_read: boolean;
  }

  interface GroupedComments {
    kind: 'comments_group';
    groupId: string;
    video_id: string;
    video_title: string;
    items: NotificationItem[];
    actor: { id: string; display_name: string; avatar_url?: string | null; color?: string | null };
    created_at: string;
    is_read: boolean;
  }

  interface SingleNotification {
    kind: 'single';
    notification: NotificationItem;
  }

  type ActivityItem = GroupedBouts | GroupedComments | SingleNotification;

  let expandedGroupIds = $state<Set<string>>(new Set());

  function toggleGroup(groupId: string, e?: MouseEvent) {
    if (e) e.stopPropagation();
    const next = new Set(expandedGroupIds);
    if (next.has(groupId)) {
      next.delete(groupId);
    } else {
      next.add(groupId);
    }
    expandedGroupIds = next;
  }

  function groupNotificationList(items: NotificationItem[]): ActivityItem[] {
    const boutGroups = new Map<string, NotificationItem[]>();
    const commentGroups = new Map<string, NotificationItem[]>();

    for (const item of items) {
      if (item.event_type === 'bout_added' || item.event_type === 'bout_updated') {
        const list = boutGroups.get(item.video_id) || [];
        list.push(item);
        boutGroups.set(item.video_id, list);
      } else if (item.event_type === 'comment_reply' || item.event_type === 'video_comment') {
        const list = commentGroups.get(item.video_id) || [];
        list.push(item);
        commentGroups.set(item.video_id, list);
      }
    }

    const result: ActivityItem[] = [];
    const processedBouts = new Set<string>();
    const processedComments = new Set<string>();

    for (const item of items) {
      if (item.event_type === 'bout_added' || item.event_type === 'bout_updated') {
        const list = boutGroups.get(item.video_id)!;
        if (list.length >= 2) {
          if (!processedBouts.has(item.video_id)) {
            processedBouts.add(item.video_id);
            result.push({
              kind: 'bouts_group',
              groupId: `bouts_${item.video_id}_${item.is_read ? 'read' : 'unread'}`,
              video_id: item.video_id,
              video_title: item.video_title,
              items: list,
              actor: list[0].actor,
              created_at: list[0].created_at,
              is_read: list.every(x => x.is_read),
            });
          }
        } else {
          result.push({ kind: 'single', notification: item });
        }
      } else if (item.event_type === 'comment_reply' || item.event_type === 'video_comment') {
        const list = commentGroups.get(item.video_id)!;
        if (list.length >= 2) {
          if (!processedComments.has(item.video_id)) {
            processedComments.add(item.video_id);
            result.push({
              kind: 'comments_group',
              groupId: `comments_${item.video_id}_${item.is_read ? 'read' : 'unread'}`,
              video_id: item.video_id,
              video_title: item.video_title,
              items: list,
              actor: list[0].actor,
              created_at: list[0].created_at,
              is_read: list.every(x => x.is_read),
            });
          }
        } else {
          result.push({ kind: 'single', notification: item });
        }
      } else {
        result.push({ kind: 'single', notification: item });
      }
    }

    return result;
  }

  let groupedUnread = $derived(groupNotificationList(unreadNotifications));
  let groupedRead = $derived(groupNotificationList(readNotifications));

  // ── Navigation & Actions ────────────────────────────────────────────────────
  function navigateToNotification(n: NotificationItem) {
    if (n.event_type === 'video_tag' || n.event_type === 'video_labeled') {
      window.location.hash = `#/player/${n.video_id}`;
    } else if (n.event_type === 'bout_added' || n.event_type === 'bout_updated') {
      const t = n.payload?.time_start_ms ?? 0;
      const boutParam = n.bout_id ? `&bout_id=${n.bout_id}` : '';
      window.location.hash = `#/player/${n.video_id}?t=${t}${boutParam}`;
    } else if (n.event_type === 'comment_reply' || n.event_type === 'video_comment') {
      const t = n.payload?.timestamp_ms ?? 0;
      const cParam = n.comment_id ? `&comment_id=${n.comment_id}` : '';
      window.location.hash = `#/player/${n.video_id}?t=${t}${cParam}`;
    } else {
      window.location.hash = `#/player/${n.video_id}`;
    }
  }

  async function handleSingleCardClick(n: NotificationItem) {
    if (!n.is_read) {
      try {
        await markReadMultiple([n.id]);
        notifications = notifications.map(x => x.id === n.id ? { ...x, is_read: true } : x);
        unreadNotificationsCount.update(c => Math.max(0, c - 1));
      } catch (e) {
        console.error('Failed to mark notification read:', e);
      }
    }
    navigateToNotification(n);
  }

  async function handleGroupCardClick(group: GroupedBouts | GroupedComments) {
    const unreadIds = group.items.filter(x => !x.is_read).map(x => x.id);
    if (unreadIds.length > 0) {
      try {
        await markReadMultiple(unreadIds);
        const unreadSet = new Set(unreadIds);
        notifications = notifications.map(x => unreadSet.has(x.id) ? { ...x, is_read: true } : x);
        unreadNotificationsCount.update(c => Math.max(0, c - unreadIds.length));
      } catch (e) {
        console.error('Failed to mark group notifications read:', e);
      }
    }
    window.location.hash = `#/player/${group.video_id}`;
  }

  async function handleSubItemClick(group: GroupedBouts | GroupedComments, item: NotificationItem, e?: MouseEvent) {
    if (e) e.stopPropagation();
    if (!item.is_read) {
      try {
        await markReadMultiple([item.id]);
        notifications = notifications.map(x => x.id === item.id ? { ...x, is_read: true } : x);
        unreadNotificationsCount.update(c => Math.max(0, c - 1));
      } catch (e) {
        console.error('Failed to mark sub-item read:', e);
      }
    }
    navigateToNotification(item);
  }

  function navigateToSearch(r: SearchResult) {
    window.location.hash = `#/player/${r.video_id}?t=${r.timestamp_ms}&comment_id=${r.comment_id}`;
  }

  async function handleMarkAllRead() {
    try {
      await markReadAll();
      notifications = notifications.map(n => ({ ...n, is_read: true }));
      unreadNotificationsCount.set(0);
    } catch (e) {
      console.error('Failed to mark all read:', e);
    }
  }
</script>

<div class="activity-page">
  <!-- Top Bar -->
  <header class="top-bar">
    <div class="title-col">
      <h1 class="page-title">
        Активность
        {#if unreadNotifications.length > 0}
          <span class="unread-pill">{unreadNotifications.length}</span>
        {/if}
      </h1>
      <span class="subtitle">Центр уведомлений и история изменений</span>
    </div>

    <!-- Right: Search input & Mark all read -->
    <div class="actions-col">
      <div class="search-box">
        <svg class="search-icon" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <circle cx="11" cy="11" r="8"></circle>
          <line x1="21" y1="21" x2="16.65" y2="16.65"></line>
        </svg>
        <input
          type="text"
          bind:value={searchQuery}
          oninput={handleSearchInput}
          placeholder="Поиск по сообщениям…"
          class="search-input"
        />
        {#if searchQuery}
          <button class="search-clear-btn" onclick={clearSearch}>✕</button>
        {/if}
      </div>

      {#if unreadNotifications.length > 0}
        <button class="btn btn-secondary btn-mark-all" onclick={handleMarkAllRead}>
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <polyline points="20 6 9 17 4 12"></polyline>
          </svg>
          Прочитать всё
        </button>
      {/if}
    </div>
  </header>

  <!-- Filter chips -->
  {#if !searchQuery}
    <div class="filter-chips">
      <button
        class="chip"
        class:active={filters.replies}
        onclick={() => toggleFilter('replies')}
      >
        <span class="chip-dot" style:background="#38bdf8"></span>
        <span>Ответы и сообщения</span>
      </button>
      <button
        class="chip"
        class:active={filters.videoTags}
        onclick={() => toggleFilter('videoTags')}
      >
        <span class="chip-dot" style:background="#ec4899"></span>
        <span>Отметки в видео</span>
      </button>
      <button
        class="chip"
        class:active={filters.bouts}
        onclick={() => toggleFilter('bouts')}
      >
        <span class="chip-dot" style:background="#eab308"></span>
        <span>Сходы и правки</span>
      </button>
    </div>
  {/if}

  <!-- Body Content -->
  {#if searchQuery}
    <!-- Search Results View -->
    <div class="search-results-section">
      <div class="section-title">
        Результаты поиска ({searchResults.length})
      </div>
      {#if searching}
        <div class="loading-state">Поиск сообщений…</div>
      {:else if searched && searchResults.length === 0}
        <div class="empty-state">Ничего не найдено по запросу «{searchQuery}»</div>
      {:else}
        <div class="results-grid">
          {#each searchResults as r (r.comment_id)}
            <!-- svelte-ignore a11y_click_events_have_key_events -->
            <!-- svelte-ignore a11y_no_static_element_interactions -->
            <div class="notif-card search-card" onclick={() => navigateToSearch(r)}>
              <div class="card-icon" style:background="rgba(56, 189, 248, 0.15)" style:color="#38bdf8">
                💬
              </div>
              <div class="card-body">
                <div class="card-meta">
                  <span class="actor-name">{r.author_name}</span>
                  <span class="meta-dot">•</span>
                  <span class="video-ref">{r.fighter_a_name ?? '?'} vs {r.fighter_b_name ?? '?'}</span>
                  <span class="meta-dot">•</span>
                  <span class="time-badge">{fmtMs(r.timestamp_ms)}</span>
                </div>
                <div class="comment-text-preview">«{r.comment_text}»</div>
              </div>
            </div>
          {/each}
        </div>
      {/if}
    </div>
  {:else}
    <!-- Notifications Feed View -->
    {#if loading}
      <div class="loading-state">Загрузка ленты активности…</div>
    {:else if filteredNotifications.length === 0}
      <div class="empty-state">
        <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.2">
          <circle cx="12" cy="12" r="10"></circle>
          <line x1="12" y1="8" x2="12" y2="12"></line>
          <line x1="12" y1="16" x2="12.01" y2="16"></line>
        </svg>
        <p>Пока нет уведомлений в выбранных категориях</p>
      </div>
    {:else}
      <!-- UNREAD SECTION -->
      {#if unreadNotifications.length > 0}
        <section class="notif-section">
          <div class="section-title">
            <span class="dot-unread"></span>
            Новые ({unreadNotifications.length})
          </div>
          <div class="notif-list">
            {#each groupedUnread as entry (entry.kind === 'single' ? entry.notification.id : entry.groupId)}
              {#if entry.kind === 'single'}
                {@const n = entry.notification}
                <!-- svelte-ignore a11y_click_events_have_key_events -->
                <!-- svelte-ignore a11y_no_static_element_interactions -->
                <div class="notif-card unread" onclick={() => handleSingleCardClick(n)}>
                  <div class="unread-bar"></div>

                  <!-- Event Icon / Avatar -->
                  <div class="card-avatar-wrap">
                    <div class="card-avatar-box" style:background={resolveColor(n.actor.id, n.actor.color)}>
                      <svg class="card-avatar-svg" width="20" height="20" viewBox="0 0 24 24" fill="none" aria-hidden="true">
                        <circle cx="12" cy="8" r="4" stroke={getContrastColor(n.actor.color)} stroke-width="1.6"/>
                        <path d="M4 20c0-4 3.6-7 8-7s8 3 8 7" stroke={getContrastColor(n.actor.color)} stroke-width="1.6" stroke-linecap="round"/>
                      </svg>
                      {#if n.actor.avatar_url}
                        <img
                          src={n.actor.avatar_url}
                          alt=""
                          class="card-avatar-img"
                          onerror={(e) => { (e.target as HTMLElement).style.display = 'none'; }}
                        />
                      {/if}
                    </div>
                    <span class="event-badge">
                      {#if n.event_type === 'video_tag'}⚔️
                      {:else if n.event_type === 'bout_added'}➕
                      {:else if n.event_type === 'bout_updated'}✏️
                      {:else if n.event_type === 'comment_reply'}💬
                      {:else if n.event_type === 'video_comment'}💭
                      {:else}🔔{/if}
                    </span>
                  </div>

                  <!-- Content -->
                  <div class="card-body">
                    <div class="card-meta">
                      <span class="actor-name">{n.actor.display_name}</span>
                      <span class="action-desc">
                        {#if n.event_type === 'video_tag'}отметил(а) вас в ролике
                        {:else if n.event_type === 'video_labeled'}завершил(а) разметку ролика
                        {:else if n.event_type === 'bout_added'}добавил(а) новый сход в
                        {:else if n.event_type === 'bout_updated'}изменил(а) сход #{n.payload?.order_index ?? ''} в
                        {:else if n.event_type === 'comment_reply'}ответил(а) на ваш комментарий в
                        {:else if n.event_type === 'video_comment'}оставил(а) комментарий в вашем бою
                        {:else}обновил(а) событие в{/if}
                      </span>
                      <span class="video-ref">{n.video_title}</span>
                      <span class="meta-dot">•</span>
                      <span class="timestamp">{fmtDate(n.created_at)}</span>
                    </div>

                    <!-- Details Payload -->
                    {#if n.event_type === 'comment_reply'}
                      <div class="reply-container">
                        {#if n.payload?.original_text}
                          <div class="quote-text">«{n.payload.original_text}»</div>
                        {/if}
                        <div class="reply-text">↳ «{n.payload?.reply_text ?? ''}»</div>
                      </div>
                    {:else if n.event_type === 'video_comment'}
                      <div class="comment-text-preview">«{n.payload?.comment_text ?? ''}»</div>
                    {:else if n.event_type === 'bout_updated' && n.payload?.changes}
                      <div class="bout-changes">
                        {#each n.payload.changes as change}
                          <span class="change-tag">{change}</span>
                        {/each}
                      </div>
                    {:else if n.event_type === 'bout_added'}
                      <div class="bout-time-tag">
                        Хронометраж: {fmtMs(n.payload?.time_start_ms ?? 0)} — {fmtMs(n.payload?.time_end_ms ?? 0)}
                      </div>
                    {/if}
                  </div>
                </div>
              {:else if entry.kind === 'bouts_group'}
                <!-- svelte-ignore a11y_click_events_have_key_events -->
                <!-- svelte-ignore a11y_no_static_element_interactions -->
                <div class="notif-card group-card unread" onclick={() => handleGroupCardClick(entry)}>
                  <div class="unread-bar"></div>

                  <div class="group-main-row">
                    <div class="card-avatar-wrap">
                      <div class="card-avatar-box" style:background={resolveColor(entry.actor.id, entry.actor.color)}>
                        <svg class="card-avatar-svg" width="20" height="20" viewBox="0 0 24 24" fill="none" aria-hidden="true">
                          <circle cx="12" cy="8" r="4" stroke={getContrastColor(entry.actor.color)} stroke-width="1.6"/>
                          <path d="M4 20c0-4 3.6-7 8-7s8 3 8 7" stroke={getContrastColor(entry.actor.color)} stroke-width="1.6" stroke-linecap="round"/>
                        </svg>
                        {#if entry.actor.avatar_url}
                          <img
                            src={entry.actor.avatar_url}
                            alt=""
                            class="card-avatar-img"
                            onerror={(e) => { (e.target as HTMLElement).style.display = 'none'; }}
                          />
                        {/if}
                      </div>
                      <span class="event-badge">✏️</span>
                    </div>

                    <div class="card-body">
                      <div class="group-title-row">
                        <span class="group-title-text">В видео «{entry.video_title}» обновлены сходы</span>
                        <span class="group-count-pill">{entry.items.length}</span>
                      </div>
                      <div class="card-meta">
                        <span class="actor-name">{entry.actor.display_name}</span>
                        <span class="meta-dot">•</span>
                        <span class="timestamp">{fmtDate(entry.created_at)}</span>
                      </div>
                    </div>

                    <button
                      class="group-toggle-btn"
                      class:open={expandedGroupIds.has(entry.groupId)}
                      onclick={(e) => toggleGroup(entry.groupId, e)}
                      title={expandedGroupIds.has(entry.groupId) ? 'Свернуть список' : 'Показать сходы'}
                      aria-label="Переключить список сходов"
                    >
                      <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                        <polyline points="6 9 12 15 18 9"></polyline>
                      </svg>
                    </button>
                  </div>

                  {#if expandedGroupIds.has(entry.groupId)}
                    <!-- svelte-ignore a11y_click_events_have_key_events -->
                    <!-- svelte-ignore a11y_no_static_element_interactions -->
                    <div class="group-accordion-list" onclick={(e) => e.stopPropagation()}>
                      {#each entry.items as sub (sub.id)}
                        <div class="group-subitem" onclick={(e) => handleSubItemClick(entry, sub, e)}>
                          <div class="subitem-left">
                            <span class="subitem-order">
                              {#if sub.event_type === 'bout_added'}➕ Сход #{sub.payload?.order_index ?? ''}
                              {:else}✏️ Сход #{sub.payload?.order_index ?? ''}{/if}
                            </span>
                            <span class="subitem-timeframe">
                              {fmtMs(sub.payload?.time_start_ms ?? 0)} — {fmtMs(sub.payload?.time_end_ms ?? 0)}
                            </span>
                            {#if sub.payload?.changes && sub.payload.changes.length > 0}
                              <div class="subitem-changes">
                                {#each sub.payload.changes as ch}
                                  <span class="change-tag-mini">{ch}</span>
                                {/each}
                              </div>
                            {/if}
                          </div>
                          <div class="subitem-right">
                            <span class="subitem-actor">{sub.actor.display_name}</span>
                            <span class="subitem-date">{fmtDate(sub.created_at)}</span>
                            <svg class="subitem-arrow" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                              <polyline points="9 18 15 12 9 6"></polyline>
                            </svg>
                          </div>
                        </div>
                      {/each}
                    </div>
                  {/if}
                </div>
              {:else if entry.kind === 'comments_group'}
                <!-- svelte-ignore a11y_click_events_have_key_events -->
                <!-- svelte-ignore a11y_no_static_element_interactions -->
                <div class="notif-card group-card unread" onclick={() => handleGroupCardClick(entry)}>
                  <div class="unread-bar"></div>

                  <div class="group-main-row">
                    <div class="card-avatar-wrap">
                      <div class="card-avatar-box" style:background={resolveColor(entry.actor.id, entry.actor.color)}>
                        <svg class="card-avatar-svg" width="20" height="20" viewBox="0 0 24 24" fill="none" aria-hidden="true">
                          <circle cx="12" cy="8" r="4" stroke={getContrastColor(entry.actor.color)} stroke-width="1.6"/>
                          <path d="M4 20c0-4 3.6-7 8-7s8 3 8 7" stroke={getContrastColor(entry.actor.color)} stroke-width="1.6" stroke-linecap="round"/>
                        </svg>
                        {#if entry.actor.avatar_url}
                          <img
                            src={entry.actor.avatar_url}
                            alt=""
                            class="card-avatar-img"
                            onerror={(e) => { (e.target as HTMLElement).style.display = 'none'; }}
                          />
                        {/if}
                      </div>
                      <span class="event-badge">💬</span>
                    </div>

                    <div class="card-body">
                      <div class="group-title-row">
                        <span class="group-title-text">В видео «{entry.video_title}» новые комментарии</span>
                        <span class="group-count-pill">{entry.items.length}</span>
                      </div>
                      <div class="card-meta">
                        <span class="actor-name">{entry.actor.display_name}</span>
                        <span class="meta-dot">•</span>
                        <span class="timestamp">{fmtDate(entry.created_at)}</span>
                      </div>
                    </div>

                    <button
                      class="group-toggle-btn"
                      class:open={expandedGroupIds.has(entry.groupId)}
                      onclick={(e) => toggleGroup(entry.groupId, e)}
                      title={expandedGroupIds.has(entry.groupId) ? 'Свернуть список' : 'Показать комментарии'}
                      aria-label="Переключить список комментариев"
                    >
                      <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                        <polyline points="6 9 12 15 18 9"></polyline>
                      </svg>
                    </button>
                  </div>

                  {#if expandedGroupIds.has(entry.groupId)}
                    <!-- svelte-ignore a11y_click_events_have_key_events -->
                    <!-- svelte-ignore a11y_no_static_element_interactions -->
                    <div class="group-accordion-list" onclick={(e) => e.stopPropagation()}>
                      {#each entry.items as sub (sub.id)}
                        <div class="group-subitem" onclick={(e) => handleSubItemClick(entry, sub, e)}>
                          <div class="subitem-left">
                            <span class="subitem-order">💬 {sub.actor.display_name}</span>
                            <span class="subitem-timeframe">{fmtMs(sub.payload?.timestamp_ms ?? 0)}</span>
                            <div class="subitem-comment-text">
                              {#if sub.payload?.reply_text}↳ «{sub.payload.reply_text}»
                              {:else if sub.payload?.comment_text}«{sub.payload.comment_text}»{/if}
                            </div>
                          </div>
                          <div class="subitem-right">
                            <span class="subitem-date">{fmtDate(sub.created_at)}</span>
                            <svg class="subitem-arrow" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                              <polyline points="9 18 15 12 9 6"></polyline>
                            </svg>
                          </div>
                        </div>
                      {/each}
                    </div>
                  {/if}
                </div>
              {/if}
            {/each}
          </div>
        </section>
      {/if}

      <!-- READ / PREVIOUS SECTION -->
      {#if readNotifications.length > 0}
        <section class="notif-section">
          <div class="section-title section-title--muted">
            Ранее просмотренные ({readNotifications.length})
          </div>
          <div class="notif-list">
            {#each groupedRead as entry (entry.kind === 'single' ? entry.notification.id : entry.groupId)}
              {#if entry.kind === 'single'}
                {@const n = entry.notification}
                <!-- svelte-ignore a11y_click_events_have_key_events -->
                <!-- svelte-ignore a11y_no_static_element_interactions -->
                <div class="notif-card read" onclick={() => handleSingleCardClick(n)}>
                  <!-- Event Icon / Avatar -->
                  <div class="card-avatar-wrap">
                    <div class="card-avatar-box muted" style:background={resolveColor(n.actor.id, n.actor.color)}>
                      <svg class="card-avatar-svg" width="20" height="20" viewBox="0 0 24 24" fill="none" aria-hidden="true">
                        <circle cx="12" cy="8" r="4" stroke={getContrastColor(n.actor.color)} stroke-width="1.6"/>
                        <path d="M4 20c0-4 3.6-7 8-7s8 3 8 7" stroke={getContrastColor(n.actor.color)} stroke-width="1.6" stroke-linecap="round"/>
                      </svg>
                      {#if n.actor.avatar_url}
                        <img
                          src={n.actor.avatar_url}
                          alt=""
                          class="card-avatar-img"
                          onerror={(e) => { (e.target as HTMLElement).style.display = 'none'; }}
                        />
                      {/if}
                    </div>
                    <span class="event-badge muted">
                      {#if n.event_type === 'video_tag'}⚔️
                      {:else if n.event_type === 'bout_added'}➕
                      {:else if n.event_type === 'bout_updated'}✏️
                      {:else if n.event_type === 'comment_reply'}💬
                      {:else if n.event_type === 'video_comment'}💭
                      {:else}🔔{/if}
                    </span>
                  </div>

                  <!-- Content -->
                  <div class="card-body">
                    <div class="card-meta">
                      <span class="actor-name muted">{n.actor.display_name}</span>
                      <span class="action-desc">
                        {#if n.event_type === 'video_tag'}отметил(а) вас в ролике
                        {:else if n.event_type === 'video_labeled'}разметил(а) ролик
                        {:else if n.event_type === 'bout_added'}добавил(а) сход в
                        {:else if n.event_type === 'bout_updated'}изменил(а) сход #{n.payload?.order_index ?? ''} в
                        {:else if n.event_type === 'comment_reply'}ответил(а) на комментарий в
                        {:else if n.event_type === 'video_comment'}оставил(а) комментарий к
                        {:else}обновил(а) событие в{/if}
                      </span>
                      <span class="video-ref">{n.video_title}</span>
                      <span class="meta-dot">•</span>
                      <span class="timestamp">{fmtDate(n.created_at)}</span>
                    </div>

                    <!-- Details Payload -->
                    {#if n.event_type === 'comment_reply'}
                      <div class="reply-container muted">
                        <div class="reply-text">↳ «{n.payload?.reply_text ?? ''}»</div>
                      </div>
                    {:else if n.event_type === 'video_comment'}
                      <div class="comment-text-preview muted">«{n.payload?.comment_text ?? ''}»</div>
                    {:else if n.event_type === 'bout_updated' && n.payload?.changes}
                      <div class="bout-changes muted">
                        {#each n.payload.changes as change}
                          <span class="change-tag">{change}</span>
                        {/each}
                      </div>
                    {/if}
                  </div>
                </div>
              {:else if entry.kind === 'bouts_group'}
                <!-- svelte-ignore a11y_click_events_have_key_events -->
                <!-- svelte-ignore a11y_no_static_element_interactions -->
                <div class="notif-card group-card read" onclick={() => handleGroupCardClick(entry)}>
                  <div class="group-main-row">
                    <div class="card-avatar-wrap">
                      <div class="card-avatar-box muted" style:background={resolveColor(entry.actor.id, entry.actor.color)}>
                        <svg class="card-avatar-svg" width="20" height="20" viewBox="0 0 24 24" fill="none" aria-hidden="true">
                          <circle cx="12" cy="8" r="4" stroke={getContrastColor(entry.actor.color)} stroke-width="1.6"/>
                          <path d="M4 20c0-4 3.6-7 8-7s8 3 8 7" stroke={getContrastColor(entry.actor.color)} stroke-width="1.6" stroke-linecap="round"/>
                        </svg>
                        {#if entry.actor.avatar_url}
                          <img
                            src={entry.actor.avatar_url}
                            alt=""
                            class="card-avatar-img"
                            onerror={(e) => { (e.target as HTMLElement).style.display = 'none'; }}
                          />
                        {/if}
                      </div>
                      <span class="event-badge muted">✏️</span>
                    </div>

                    <div class="card-body">
                      <div class="group-title-row">
                        <span class="group-title-text muted">В видео «{entry.video_title}» обновлены сходы</span>
                        <span class="group-count-pill muted">{entry.items.length}</span>
                      </div>
                      <div class="card-meta">
                        <span class="actor-name muted">{entry.actor.display_name}</span>
                        <span class="meta-dot">•</span>
                        <span class="timestamp">{fmtDate(entry.created_at)}</span>
                      </div>
                    </div>

                    <button
                      class="group-toggle-btn"
                      class:open={expandedGroupIds.has(entry.groupId)}
                      onclick={(e) => toggleGroup(entry.groupId, e)}
                      title={expandedGroupIds.has(entry.groupId) ? 'Свернуть список' : 'Показать сходы'}
                      aria-label="Переключить список сходов"
                    >
                      <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                        <polyline points="6 9 12 15 18 9"></polyline>
                      </svg>
                    </button>
                  </div>

                  {#if expandedGroupIds.has(entry.groupId)}
                    <!-- svelte-ignore a11y_click_events_have_key_events -->
                    <!-- svelte-ignore a11y_no_static_element_interactions -->
                    <div class="group-accordion-list" onclick={(e) => e.stopPropagation()}>
                      {#each entry.items as sub (sub.id)}
                        <div class="group-subitem" onclick={(e) => handleSubItemClick(entry, sub, e)}>
                          <div class="subitem-left">
                            <span class="subitem-order muted">
                              {#if sub.event_type === 'bout_added'}➕ Сход #{sub.payload?.order_index ?? ''}
                              {:else}✏️ Сход #{sub.payload?.order_index ?? ''}{/if}
                            </span>
                            <span class="subitem-timeframe muted">
                              {fmtMs(sub.payload?.time_start_ms ?? 0)} — {fmtMs(sub.payload?.time_end_ms ?? 0)}
                            </span>
                            {#if sub.payload?.changes && sub.payload.changes.length > 0}
                              <div class="subitem-changes">
                                {#each sub.payload.changes as ch}
                                  <span class="change-tag-mini muted">{ch}</span>
                                {/each}
                              </div>
                            {/if}
                          </div>
                          <div class="subitem-right">
                            <span class="subitem-actor muted">{sub.actor.display_name}</span>
                            <span class="subitem-date">{fmtDate(sub.created_at)}</span>
                            <svg class="subitem-arrow" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                              <polyline points="9 18 15 12 9 6"></polyline>
                            </svg>
                          </div>
                        </div>
                      {/each}
                    </div>
                  {/if}
                </div>
              {:else if entry.kind === 'comments_group'}
                <!-- svelte-ignore a11y_click_events_have_key_events -->
                <!-- svelte-ignore a11y_no_static_element_interactions -->
                <div class="notif-card group-card read" onclick={() => handleGroupCardClick(entry)}>
                  <div class="group-main-row">
                    <div class="card-avatar-wrap">
                      <div class="card-avatar-box muted" style:background={resolveColor(entry.actor.id, entry.actor.color)}>
                        <svg class="card-avatar-svg" width="20" height="20" viewBox="0 0 24 24" fill="none" aria-hidden="true">
                          <circle cx="12" cy="8" r="4" stroke={getContrastColor(entry.actor.color)} stroke-width="1.6"/>
                          <path d="M4 20c0-4 3.6-7 8-7s8 3 8 7" stroke={getContrastColor(entry.actor.color)} stroke-width="1.6" stroke-linecap="round"/>
                        </svg>
                        {#if entry.actor.avatar_url}
                          <img
                            src={entry.actor.avatar_url}
                            alt=""
                            class="card-avatar-img"
                            onerror={(e) => { (e.target as HTMLElement).style.display = 'none'; }}
                          />
                        {/if}
                      </div>
                      <span class="event-badge muted">💬</span>
                    </div>

                    <div class="card-body">
                      <div class="group-title-row">
                        <span class="group-title-text muted">В видео «{entry.video_title}» новые комментарии</span>
                        <span class="group-count-pill muted">{entry.items.length}</span>
                      </div>
                      <div class="card-meta">
                        <span class="actor-name muted">{entry.actor.display_name}</span>
                        <span class="meta-dot">•</span>
                        <span class="timestamp">{fmtDate(entry.created_at)}</span>
                      </div>
                    </div>

                    <button
                      class="group-toggle-btn"
                      class:open={expandedGroupIds.has(entry.groupId)}
                      onclick={(e) => toggleGroup(entry.groupId, e)}
                      title={expandedGroupIds.has(entry.groupId) ? 'Свернуть список' : 'Показать комментарии'}
                      aria-label="Переключить список комментариев"
                    >
                      <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                        <polyline points="6 9 12 15 18 9"></polyline>
                      </svg>
                    </button>
                  </div>

                  {#if expandedGroupIds.has(entry.groupId)}
                    <!-- svelte-ignore a11y_click_events_have_key_events -->
                    <!-- svelte-ignore a11y_no_static_element_interactions -->
                    <div class="group-accordion-list" onclick={(e) => e.stopPropagation()}>
                      {#each entry.items as sub (sub.id)}
                        <div class="group-subitem" onclick={(e) => handleSubItemClick(entry, sub, e)}>
                          <div class="subitem-left">
                            <span class="subitem-order muted">💬 {sub.actor.display_name}</span>
                            <span class="subitem-timeframe muted">{fmtMs(sub.payload?.timestamp_ms ?? 0)}</span>
                            <div class="subitem-comment-text muted">
                              {#if sub.payload?.reply_text}↳ «{sub.payload.reply_text}»
                              {:else if sub.payload?.comment_text}«{sub.payload.comment_text}»{/if}
                            </div>
                          </div>
                          <div class="subitem-right">
                            <span class="subitem-date">{fmtDate(sub.created_at)}</span>
                            <svg class="subitem-arrow" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                              <polyline points="9 18 15 12 9 6"></polyline>
                            </svg>
                          </div>
                        </div>
                      {/each}
                    </div>
                  {/if}
                </div>
              {/if}
            {/each}
          </div>
        </section>
      {/if}
    {/if}
  {/if}
</div>

<style>
  .activity-page {
    max-width: 960px;
    margin: 0 auto;
    padding-bottom: 60px;
  }

  /* ── Header ───────────────────────────────────────────────────────────────── */
  .top-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    margin-bottom: 20px;
    flex-wrap: wrap;
  }

  .title-col {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .page-title {
    font-size: 1.6rem;
    font-weight: 700;
    color: var(--text-primary);
    margin: 0;
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .unread-pill {
    font-size: 0.8rem;
    background: #f97316;
    color: #fff;
    padding: 2px 8px;
    border-radius: var(--radius-pill);
    font-weight: 600;
  }

  .subtitle {
    font-size: 0.85rem;
    color: var(--text-secondary);
  }

  .actions-col {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .search-box {
    position: relative;
    display: flex;
    align-items: center;
  }

  .search-icon {
    position: absolute;
    left: 10px;
    color: var(--text-secondary);
    pointer-events: none;
  }

  .search-input {
    background: var(--surface-glass);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-pill);
    padding: 7px 32px 7px 32px;
    color: var(--text-primary);
    font-size: 0.85rem;
    width: 210px;
    transition: var(--transition);
  }

  .search-input:focus {
    outline: none;
    border-color: var(--border-hover);
    width: 260px;
    background: var(--surface-solid);
  }

  .search-clear-btn {
    position: absolute;
    right: 8px;
    background: none;
    border: none;
    color: var(--text-secondary);
    cursor: pointer;
    font-size: 0.85rem;
  }

  .btn-mark-all {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 0.85rem;
    padding: 7px 14px;
    border-radius: var(--radius-pill);
    background: var(--surface-solid);
    border: 1px solid var(--border-color);
    color: var(--text-primary);
    cursor: pointer;
    transition: var(--transition);
  }

  .btn-mark-all:hover {
    background: var(--surface-hover);
    border-color: var(--border-hover);
  }

  /* ── Filter Chips ─────────────────────────────────────────────────────────── */
  .filter-chips {
    display: flex;
    gap: 8px;
    margin-bottom: 24px;
    flex-wrap: wrap;
  }

  .chip {
    display: flex;
    align-items: center;
    gap: 7px;
    padding: 6px 14px;
    border-radius: var(--radius-pill);
    background: var(--surface-glass);
    border: 1px solid var(--border-color);
    color: var(--text-secondary);
    font-size: 0.82rem;
    cursor: pointer;
    transition: var(--transition);
  }

  .chip:hover {
    background: var(--surface-hover);
    color: var(--text-primary);
  }

  .chip.active {
    background: rgba(234, 179, 8, 0.12);
    border-color: rgba(234, 179, 8, 0.4);
    color: var(--text-primary);
    font-weight: 500;
  }

  .chip-dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
  }

  /* ── Sections ─────────────────────────────────────────────────────────────── */
  .notif-section {
    margin-bottom: 32px;
  }

  .section-title {
    font-size: 0.95rem;
    font-weight: 600;
    color: var(--text-primary);
    margin-bottom: 12px;
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .section-title--muted {
    color: var(--text-secondary);
    font-size: 0.85rem;
  }

  .dot-unread {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: #f97316;
  }

  .notif-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  /* ── Cards ────────────────────────────────────────────────────────────────── */
  .notif-card {
    position: relative;
    display: flex;
    align-items: flex-start;
    gap: 14px;
    padding: 14px 18px;
    background: var(--surface-solid);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-md);
    cursor: pointer;
    transition: var(--transition);
    text-decoration: none;
    overflow: hidden;
  }

  .notif-card:hover {
    background: var(--surface-hover);
    border-color: var(--border-hover);
    transform: translateY(-1px);
  }

  .notif-card.unread {
    background: rgba(255, 255, 255, 0.03);
    border-left: 3px solid #f97316;
  }

  .notif-card.read {
    opacity: 0.82;
    background: rgba(0, 0, 0, 0.2);
  }

  .unread-bar {
    position: absolute;
    left: 0;
    top: 0;
    bottom: 0;
    width: 3px;
    background: #f97316;
  }

  /* Avatar / Icon */
  .card-avatar-wrap {
    position: relative;
    flex-shrink: 0;
  }

  .card-avatar-box {
    width: 36px;
    height: 36px;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    position: relative;
    overflow: hidden;
    border: 1px solid rgba(255, 255, 255, 0.15);
  }

  .card-avatar-box.muted {
    opacity: 0.75;
    filter: grayscale(0.4);
  }

  .card-avatar-svg {
    position: absolute;
    pointer-events: none;
  }

  .card-avatar-img {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .event-badge {
    position: absolute;
    bottom: -4px;
    right: -4px;
    font-size: 0.75rem;
    background: var(--surface-solid);
    border-radius: 50%;
    width: 18px;
    height: 18px;
    display: flex;
    align-items: center;
    justify-content: center;
    border: 1px solid var(--border-color);
  }

  .card-body {
    flex: 1;
    min-width: 0;
  }

  .card-meta {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 0.88rem;
    flex-wrap: wrap;
    line-height: 1.4;
  }

  .actor-name {
    font-weight: 600;
    color: var(--text-primary);
  }

  .action-desc {
    color: var(--text-secondary);
  }

  .video-ref {
    color: var(--accent-yellow);
    font-weight: 500;
  }

  .meta-dot {
    color: var(--text-secondary);
    font-size: 0.7rem;
  }

  .timestamp {
    color: var(--text-secondary);
    font-size: 0.78rem;
    margin-left: auto;
  }

  /* Specific payload renders */
  .reply-container {
    margin-top: 8px;
    padding: 8px 12px;
    background: rgba(0, 0, 0, 0.25);
    border-radius: var(--radius-sm);
    font-size: 0.84rem;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .quote-text {
    color: var(--text-secondary);
    font-style: italic;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .reply-text {
    color: var(--text-primary);
    font-weight: 500;
  }

  .comment-text-preview {
    margin-top: 6px;
    font-size: 0.85rem;
    color: var(--text-primary);
  }

  .bout-changes {
    display: flex;
    gap: 6px;
    flex-wrap: wrap;
    margin-top: 8px;
  }

  .change-tag {
    font-size: 0.78rem;
    background: rgba(234, 179, 8, 0.12);
    color: var(--accent-yellow);
    border: 1px solid rgba(234, 179, 8, 0.25);
    padding: 2px 8px;
    border-radius: var(--radius-pill);
  }

  .bout-time-tag {
    margin-top: 6px;
    font-size: 0.8rem;
    color: var(--text-secondary);
  }

  /* ── Search Mode Cards ────────────────────────────────────────────────────── */
  .search-results-section {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .results-grid {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .card-icon {
    width: 36px;
    height: 36px;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 1.1rem;
    flex-shrink: 0;
  }

  .time-badge {
    background: rgba(255, 255, 255, 0.08);
    padding: 1px 6px;
    border-radius: var(--radius-pill);
    font-size: 0.78rem;
    color: var(--text-secondary);
  }

  /* ── States ───────────────────────────────────────────────────────────────── */
  .loading-state, .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 60px 20px;
    color: var(--text-secondary);
    gap: 12px;
  }

  /* ── Group Cards & Accordion ──────────────────────────────────────────────── */
  .notif-card.group-card {
    cursor: pointer;
    position: relative;
    padding: 0;
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }

  .group-main-row {
    display: flex;
    align-items: center;
    gap: 14px;
    padding: 14px 16px;
    width: 100%;
    box-sizing: border-box;
  }

  .group-title-row {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
    margin-bottom: 2px;
  }

  .group-title-text {
    font-size: 0.95rem;
    font-weight: 600;
    color: var(--text-primary);
  }

  .group-title-text.muted {
    color: var(--text-secondary);
  }

  .group-count-pill {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 22px;
    height: 20px;
    padding: 0 7px;
    border-radius: var(--radius-pill);
    background: rgba(245, 158, 11, 0.2);
    border: 1px solid rgba(245, 158, 11, 0.35);
    color: #f59e0b;
    font-size: 0.76rem;
    font-weight: 700;
  }

  .group-count-pill.muted {
    background: rgba(255, 255, 255, 0.08);
    border-color: rgba(255, 255, 255, 0.15);
    color: var(--text-secondary);
  }

  .group-toggle-btn {
    margin-left: auto;
    background: transparent;
    border: 1px solid transparent;
    color: var(--text-secondary);
    width: 32px;
    height: 32px;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition: background 0.15s, transform 0.2s, color 0.15s, border-color 0.15s;
    flex-shrink: 0;
  }

  .group-toggle-btn:hover {
    background: var(--surface-hover);
    border-color: var(--border-color);
    color: var(--text-primary);
  }

  .group-toggle-btn.open svg {
    transform: rotate(180deg);
  }

  .group-accordion-list {
    border-top: 1px solid var(--border-color);
    background: rgba(0, 0, 0, 0.18);
    display: flex;
    flex-direction: column;
  }

  .group-subitem {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 10px 16px 10px 58px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.04);
    cursor: pointer;
    transition: background 0.15s;
    box-sizing: border-box;
  }

  .group-subitem:last-child {
    border-bottom: none;
  }

  .group-subitem:hover {
    background: var(--surface-hover);
  }

  .subitem-left {
    display: flex;
    align-items: center;
    gap: 10px;
    flex-wrap: wrap;
    min-width: 0;
  }

  .subitem-order {
    font-weight: 600;
    font-size: 0.85rem;
    color: var(--text-primary);
    white-space: nowrap;
  }

  .subitem-order.muted {
    color: var(--text-secondary);
  }

  .subitem-timeframe {
    font-size: 0.8rem;
    color: var(--text-secondary);
    background: var(--surface-solid);
    padding: 2px 7px;
    border-radius: 4px;
    font-variant-numeric: tabular-nums;
    white-space: nowrap;
  }

  .subitem-timeframe.muted {
    opacity: 0.75;
  }

  .subitem-changes {
    display: flex;
    gap: 4px;
    flex-wrap: wrap;
  }

  .change-tag-mini {
    font-size: 0.72rem;
    padding: 2px 6px;
    border-radius: 4px;
    background: rgba(234, 179, 8, 0.12);
    border: 1px solid rgba(234, 179, 8, 0.25);
    color: #eab308;
    white-space: nowrap;
  }

  .change-tag-mini.muted {
    background: rgba(255, 255, 255, 0.05);
    border-color: rgba(255, 255, 255, 0.1);
    color: var(--text-secondary);
  }

  .subitem-comment-text {
    font-size: 0.83rem;
    color: var(--text-primary);
    font-style: italic;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    max-width: 460px;
  }

  .subitem-comment-text.muted {
    color: var(--text-secondary);
  }

  .subitem-right {
    display: flex;
    align-items: center;
    gap: 10px;
    flex-shrink: 0;
    color: var(--text-secondary);
    font-size: 0.8rem;
  }

  .subitem-actor {
    color: var(--text-secondary);
  }

  .subitem-actor.muted {
    opacity: 0.75;
  }

  .subitem-date {
    font-size: 0.75rem;
    opacity: 0.8;
  }

  .subitem-arrow {
    opacity: 0.5;
    transition: transform 0.15s, opacity 0.15s;
  }

  .group-subitem:hover .subitem-arrow {
    opacity: 1;
    transform: translateX(2px);
  }
</style>
