<script lang="ts">
  import { techniques, currentUser } from '../../stores';
  import { createTechnique, patchTechnique, deleteTechnique } from '../api/techniques';

  interface Props {
    onclose?: () => void;
  }

  let { onclose }: Props = $props();

  let isAdmin = $derived($currentUser?.is_admin ?? false);

  let newName = $state('');
  let adding = $state(false);
  let addError = $state('');
  let deleteErrors: Record<number, string> = $state({});
  let editingId = $state<number | null>(null);
  let editName = $state('');
  let editDescription = $state('');
  let expandedId = $state<number | null>(null);

  let canAdd = $derived(newName.trim().length > 0 && !adding);

  async function add() {
    if (!canAdd) return;
    adding = true;
    addError = '';
    try {
      const created = await createTechnique(newName.trim());
      techniques.update((list) => [...list, created]);
      newName = '';
    } catch (e) {
      addError = e instanceof Error ? e.message : 'Ошибка при добавлении';
    } finally {
      adding = false;
    }
  }

  function startEdit(id: number, name: string, description: string | null | undefined) {
    editingId = id;
    editName = name;
    editDescription = description ?? '';
    expandedId = null;
  }

  async function saveEdit(id: number) {
    const name = editName.trim();
    if (!name) { editingId = null; return; }
    try {
      const updated = await patchTechnique(id, { name, description: editDescription });
      techniques.update((list) => list.map(t => t.id === id ? updated : t));
      editingId = null;
    } catch (e) {
      addError = e instanceof Error ? e.message : 'Ошибка сохранения';
    }
  }

  async function remove(id: number, name: string) {
    if (!confirm(`Удалить технику «${name}»?\n\nЕсли техника записана в сходах, данные о ней там сотрутся.`)) return;
    deleteErrors = { ...deleteErrors, [id]: '' };
    try {
      await deleteTechnique(id);
      techniques.update((list) => list.filter((t) => t.id !== id));
    } catch (e) {
      const msg = e instanceof Error ? e.message : 'Ошибка';
      deleteErrors = {
        ...deleteErrors,
        [id]: msg.includes('409') ? 'Используется в сходах — удалите привязку' : msg,
      };
    }
  }

  function toggleExpanded(id: number) {
    expandedId = expandedId === id ? null : id;
  }

  function handleAddKey(e: KeyboardEvent) {
    if (e.key === 'Enter') add();
  }

  function handleBackdrop(e: MouseEvent) {
    if ((e.target as HTMLElement).classList.contains('backdrop')) {
      onclose?.();
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      if (editingId !== null) { editingId = null; return; }
      onclose?.();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="backdrop" onclick={handleBackdrop} role="dialog" aria-modal="true" aria-label="Техники">
  <div class="modal">
    <h2 class="title">Техники</h2>

    <div class="list">
      {#if $techniques.length === 0}
        <p class="empty">Техники не добавлены</p>
      {:else}
        {#each $techniques as t (t.id)}
          <div class="technique-item">
            {#if editingId === t.id}
              <!-- Edit mode (admin only) -->
              <div class="edit-block">
                <input
                  class="input-glass edit-inp"
                  type="text"
                  bind:value={editName}
                  placeholder="Название"
                  onkeydown={(e) => { if (e.key === 'Enter') saveEdit(t.id); if (e.key === 'Escape') editingId = null; }}
                  autofocus
                />
                <textarea
                  class="input-glass edit-desc"
                  bind:value={editDescription}
                  placeholder="Описание (HTML, включая изображения и embed-видео YouTube/VK)"
                  rows="6"
                ></textarea>
                <div class="edit-actions">
                  <button class="btn btn-primary btn-sm" onclick={() => saveEdit(t.id)}>Сохранить</button>
                  <button class="btn btn-outline btn-sm" onclick={() => { editingId = null; }}>Отмена</button>
                </div>
              </div>
            {:else}
              <!-- View mode -->
              <div class="technique-row">
                <button
                  class="technique-name-btn"
                  onclick={() => toggleExpanded(t.id)}
                  title={t.description ? 'Нажмите для просмотра описания' : undefined}
                >
                  <span class="technique-name">{t.name}</span>
                  {#if t.description}
                    <span class="desc-indicator" class:expanded={expandedId === t.id}>▸</span>
                  {/if}
                </button>
                {#if isAdmin}
                  <div class="row-right">
                    {#if deleteErrors[t.id]}
                      <span class="row-error">{deleteErrors[t.id]}</span>
                    {/if}
                    <button
                      class="btn-edit"
                      onclick={() => startEdit(t.id, t.name, t.description)}
                      aria-label="Редактировать {t.name}"
                    >
                      <svg width="13" height="13" viewBox="0 0 24 24" fill="none" aria-hidden="true">
                        <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
                        <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
                      </svg>
                    </button>
                    <button
                      class="btn-delete"
                      onclick={() => remove(t.id, t.name)}
                      aria-label="Удалить технику {t.name}"
                    >
                      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" aria-hidden="true">
                        <path d="M3 6h18M8 6V4h8v2M19 6l-1 14H6L5 6" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
                      </svg>
                    </button>
                  </div>
                {/if}
              </div>
              {#if expandedId === t.id && t.description}
                <!-- Description panel -->
                <div class="desc-panel">
                  <!-- eslint-disable-next-line svelte/no-at-html-tags -->
                  {@html t.description}
                </div>
              {/if}
            {/if}
          </div>
        {/each}
      {/if}
    </div>

    {#if isAdmin}
      <div class="add-row">
        <input
          class="input-glass"
          type="text"
          bind:value={newName}
          placeholder="Название техники"
          onkeydown={handleAddKey}
          autocomplete="off"
        />
        <button class="btn btn-primary" onclick={add} disabled={!canAdd}>
          {adding ? '…' : 'Добавить'}
        </button>
      </div>
    {/if}

    {#if addError}
      <p class="error">{addError}</p>
    {/if}

    <div class="actions">
      <button class="btn-close" onclick={onclose}>Закрыть</button>
    </div>
  </div>
</div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.6);
    backdrop-filter: blur(3px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 500;
  }

  .modal {
    background: var(--surface);
    backdrop-filter: var(--glass-blur);
    -webkit-backdrop-filter: var(--glass-blur);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-lg);
    padding: 32px;
    width: 480px;
    max-width: calc(100vw - 32px);
    max-height: calc(100vh - 64px);
    display: flex;
    flex-direction: column;
    gap: 20px;
    box-shadow: var(--shadow-lg);
    overflow: hidden;
  }

  .title {
    font-size: 1.05rem;
    font-weight: 600;
    color: #e8edf2;
    margin: 0;
    flex-shrink: 0;
  }

  .list {
    display: flex;
    flex-direction: column;
    gap: 4px;
    max-height: 420px;
    overflow-y: auto;
    border: 1px solid var(--border-color);
    border-radius: var(--radius-md);
    padding: 6px;
    background: var(--surface-solid);
  }

  .empty {
    font-size: 0.85rem;
    color: #4a6280;
    text-align: center;
    padding: 16px;
    margin: 0;
  }

  .technique-item {
    border-radius: 5px;
    overflow: hidden;
  }

  .technique-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 10px 14px;
    border-radius: var(--radius-sm);
    gap: 12px;
  }

  .technique-row:hover {
    background: var(--surface-hover);
  }

  .technique-name-btn {
    flex: 1;
    min-width: 0;
    background: none;
    border: none;
    cursor: pointer;
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 0;
    text-align: left;
  }

  .technique-name {
    font-size: 0.95rem;
    font-weight: 500;
    color: var(--text-primary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .desc-indicator {
    font-size: 0.8rem;
    color: var(--text-secondary);
    flex-shrink: 0;
    transition: var(--transition);
    display: inline-block;
  }

  .desc-indicator.expanded {
    transform: rotate(90deg);
  }

  .desc-panel {
    padding: 14px 18px;
    background: var(--surface-hover);
    border-top: 1px solid var(--border-color);
    font-size: 0.9rem;
    color: var(--text-secondary);
    line-height: 1.6;
    border-radius: 0 0 var(--radius-sm) var(--radius-sm);
  }

  .desc-panel :global(img) {
    max-width: 100%;
    border-radius: 4px;
    margin: 6px 0;
  }

  .desc-panel :global(iframe) {
    max-width: 100%;
    border-radius: 4px;
    margin: 6px 0;
  }

  .desc-panel :global(p) {
    margin: 0 0 8px;
  }

  .desc-panel :global(a) {
    color: #DB841F;
  }

  .row-right {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-shrink: 0;
  }

  .row-error {
    font-size: 0.75rem;
    color: #e05252;
  }

  /* Edit block */
  .edit-block {
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 14px;
    background: var(--surface-hover);
    border-radius: var(--radius-sm);
  }

  .edit-inp {
    background: var(--surface-solid);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm);
    color: var(--text-primary);
    font-size: 0.88rem;
    padding: 6px 8px;
    outline: none;
  }

  .edit-desc {
    background: var(--surface-solid);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm);
    color: var(--text-primary);
    font-size: 0.8rem;
    padding: 6px 8px;
    outline: none;
    resize: vertical;
    font-family: monospace;
    line-height: 1.5;
    transition: var(--transition);
  }

  .edit-desc:focus {
    border-color: var(--accent-yellow);
  }

  .edit-actions {
    display: flex;
    gap: 6px;
  }

  .btn-save-edit {
    background: rgba(76, 175, 130, 0.12);
    border: 1px solid rgba(76, 175, 130, 0.3);
    color: #4caf82;
    font-size: 0.82rem;
    padding: 5px 12px;
    border-radius: var(--radius-sm);
    cursor: pointer;
    transition: var(--transition);
  }

  .btn-save-edit:hover { background: rgba(76, 175, 130, 0.2); }

  .btn-cancel-edit {
    background: none;
    border: 1px solid var(--border-color);
    color: var(--text-secondary);
    font-size: 0.82rem;
    padding: 5px 12px;
    border-radius: var(--radius-sm);
    cursor: pointer;
    transition: var(--transition);
  }

  .btn-cancel-edit:hover { background: var(--surface-hover); color: var(--text-primary); }

  .btn-edit {
    background: none;
    border: none;
    color: var(--text-secondary);
    cursor: pointer;
    padding: 6px;
    border-radius: var(--radius-sm);
    display: flex;
    align-items: center;
    transition: var(--transition);
  }

  .btn-edit:hover {
    color: var(--accent-yellow);
    background: rgba(219, 132, 31, 0.1);
  }

  .btn-delete {
    background: none;
    border: none;
    color: var(--text-secondary);
    cursor: pointer;
    padding: 6px;
    border-radius: var(--radius-sm);
    display: flex;
    align-items: center;
    transition: var(--transition);
  }

  .btn-delete:hover {
    color: #ef4444;
    background: rgba(239, 68, 68, 0.1);
  }

  .add-row {
    display: flex;
    gap: 8px;
    flex-shrink: 0;
  }

  .input {
    background: var(--surface-solid);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm);
    color: var(--text-primary);
    font-size: 0.9rem;
    padding: 8px 10px;
    outline: none;
    flex: 1;
    transition: var(--transition);
  }

  .input:focus {
    border-color: var(--accent-yellow);
  }

  .btn-add {
    background: var(--surface-hover);
    border: 1px solid var(--border-color);
    color: var(--text-secondary);
    font-size: 0.88rem;
    font-weight: 500;
    padding: 8px 16px;
    border-radius: var(--radius-sm);
    cursor: pointer;
    white-space: nowrap;
    transition: var(--transition);
    flex-shrink: 0;
  }

  .btn-add:hover:not(:disabled) {
    background: var(--accent-yellow);
    border-color: var(--accent-yellow);
    color: #fff;
  }

  .btn-add:disabled {
    opacity: 0.45;
    cursor: not-allowed;
  }

  .error {
    font-size: 0.83rem;
    color: #e05252;
    margin: 0;
    flex-shrink: 0;
  }

  .actions {
    display: flex;
    justify-content: flex-end;
    margin-top: 8px;
    flex-shrink: 0;
  }

  .btn-close {
    background: var(--surface-solid);
    border: 1px solid var(--border-color);
    color: var(--text-secondary);
    font-size: 0.9rem;
    font-weight: 500;
    padding: 10px 24px;
    border-radius: var(--radius-sm);
    cursor: pointer;
    transition: var(--transition);
  }

  .btn-close:hover {
    background: var(--surface-hover);
    color: var(--text-primary);
  }

  .btn-sm {
    padding: 6px 12px;
    font-size: 0.85rem;
  }

  @media (max-width: 768px) {
    .modal {
      border-radius: var(--radius-lg) var(--radius-lg) 0 0;
      margin-top: auto;
      max-width: 100vw;
    }
  }
</style>
