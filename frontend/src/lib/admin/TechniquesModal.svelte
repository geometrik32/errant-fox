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
                  class="edit-inp"
                  type="text"
                  bind:value={editName}
                  placeholder="Название"
                  onkeydown={(e) => { if (e.key === 'Enter') saveEdit(t.id); if (e.key === 'Escape') editingId = null; }}
                  autofocus
                />
                <textarea
                  class="edit-desc"
                  bind:value={editDescription}
                  placeholder="Описание (HTML, включая изображения и embed-видео YouTube/VK)"
                  rows="6"
                ></textarea>
                <div class="edit-actions">
                  <button class="btn-save-edit" onclick={() => saveEdit(t.id)}>Сохранить</button>
                  <button class="btn-cancel-edit" onclick={() => { editingId = null; }}>Отмена</button>
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
          class="input"
          type="text"
          bind:value={newName}
          placeholder="Название техники"
          onkeydown={handleAddKey}
          autocomplete="off"
        />
        <button class="btn-add" onclick={add} disabled={!canAdd}>
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
    background: #0f2035;
    border: 1px solid #1f3a57;
    border-radius: 12px;
    padding: 28px;
    width: 480px;
    max-width: calc(100vw - 32px);
    max-height: calc(100vh - 64px);
    display: flex;
    flex-direction: column;
    gap: 16px;
    box-shadow: 0 16px 48px rgba(0, 0, 0, 0.5);
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
    gap: 2px;
    max-height: 420px;
    overflow-y: auto;
    border: 1px solid #1f3a57;
    border-radius: 8px;
    padding: 4px;
    background: #060e18;
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
    padding: 7px 10px;
    border-radius: 5px;
    gap: 8px;
  }

  .technique-row:hover {
    background: #0f2035;
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
    font-size: 0.9rem;
    color: #c8d8e8;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .desc-indicator {
    font-size: 0.7rem;
    color: #4a6280;
    flex-shrink: 0;
    transition: transform 0.15s;
    display: inline-block;
  }

  .desc-indicator.expanded {
    transform: rotate(90deg);
  }

  .desc-panel {
    padding: 10px 12px;
    background: #0a1628;
    border-top: 1px solid #1a3050;
    font-size: 0.85rem;
    color: #a0b4c8;
    line-height: 1.6;
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
    gap: 8px;
    padding: 10px;
    background: #0a1628;
  }

  .edit-inp {
    background: #060e18;
    border: 1px solid #2a4f73;
    border-radius: 4px;
    color: #e8edf2;
    font-size: 0.88rem;
    padding: 6px 8px;
    outline: none;
  }

  .edit-desc {
    background: #060e18;
    border: 1px solid #1f3a57;
    border-radius: 4px;
    color: #a0b4c8;
    font-size: 0.8rem;
    padding: 6px 8px;
    outline: none;
    resize: vertical;
    font-family: monospace;
    line-height: 1.5;
    transition: border-color 0.12s;
  }

  .edit-desc:focus {
    border-color: #2a4f73;
  }

  .edit-actions {
    display: flex;
    gap: 6px;
  }

  .btn-save-edit {
    background: #1a4030;
    border: 1px solid #2a6048;
    color: #52d47a;
    font-size: 0.82rem;
    padding: 5px 12px;
    border-radius: 4px;
    cursor: pointer;
    transition: background 0.12s;
  }

  .btn-save-edit:hover { background: #1f5040; }

  .btn-cancel-edit {
    background: none;
    border: 1px solid #2a3a50;
    color: #6b8aab;
    font-size: 0.82rem;
    padding: 5px 12px;
    border-radius: 4px;
    cursor: pointer;
    transition: background 0.12s;
  }

  .btn-cancel-edit:hover { background: #1a3050; }

  .btn-edit {
    background: none;
    border: none;
    color: #4a6280;
    cursor: pointer;
    padding: 4px;
    border-radius: 4px;
    display: flex;
    align-items: center;
    transition: color 0.12s, background 0.12s;
  }

  .btn-edit:hover {
    color: #DB841F;
    background: rgba(219, 132, 31, 0.1);
  }

  .btn-delete {
    background: none;
    border: none;
    color: #4a6280;
    cursor: pointer;
    padding: 4px;
    border-radius: 4px;
    display: flex;
    align-items: center;
    transition: color 0.12s, background 0.12s;
  }

  .btn-delete:hover {
    color: #e05252;
    background: rgba(224, 82, 82, 0.1);
  }

  .add-row {
    display: flex;
    gap: 8px;
    flex-shrink: 0;
  }

  .input {
    background: #060e18;
    border: 1px solid #1f3a57;
    border-radius: 6px;
    color: #e8edf2;
    font-size: 0.9rem;
    padding: 8px 10px;
    outline: none;
    flex: 1;
    transition: border-color 0.12s;
  }

  .input:focus {
    border-color: #2a4f73;
  }

  .btn-add {
    background: #1a3050;
    border: none;
    color: #a0b4c8;
    font-size: 0.88rem;
    font-weight: 500;
    padding: 8px 16px;
    border-radius: 6px;
    cursor: pointer;
    white-space: nowrap;
    transition: background 0.12s, color 0.12s;
    flex-shrink: 0;
  }

  .btn-add:hover:not(:disabled) {
    background: #DB841F;
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
    margin-top: 4px;
    flex-shrink: 0;
  }

  .btn-close {
    background: #1a3050;
    border: none;
    color: #a0b4c8;
    font-size: 0.88rem;
    font-weight: 500;
    padding: 8px 18px;
    border-radius: 6px;
    cursor: pointer;
    transition: background 0.12s;
  }

  .btn-close:hover {
    background: #1f3a57;
  }
</style>
