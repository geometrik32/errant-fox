<script lang="ts">
  import { techniques } from '../../stores';
  import { createTechnique, renameTechnique, deleteTechnique } from '../api/techniques';

  interface Props {
    onclose?: () => void;
  }

  let { onclose }: Props = $props();

  let newName = $state('');
  let adding = $state(false);
  let addError = $state('');
  let deleteErrors: Record<number, string> = $state({});
  let editingId = $state<number | null>(null);
  let editName = $state('');

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

  function startEdit(id: number, name: string) {
    editingId = id;
    editName = name;
  }

  async function saveEdit(id: number) {
    const name = editName.trim();
    if (!name) { editingId = null; return; }
    try {
      const updated = await renameTechnique(id, name);
      techniques.update((list) => list.map(t => t.id === id ? updated : t));
      editingId = null;
    } catch (e) {
      addError = e instanceof Error ? e.message : 'Ошибка переименования';
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

  function handleAddKey(e: KeyboardEvent) {
    if (e.key === 'Enter') add();
  }

  function handleBackdrop(e: MouseEvent) {
    if ((e.target as HTMLElement).classList.contains('backdrop')) {
      onclose?.();
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') onclose?.();
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="backdrop" onclick={handleBackdrop} role="dialog" aria-modal="true" aria-label="Управление техниками">
  <div class="modal">
    <h2 class="title">Техники</h2>

    <div class="list">
      {#if $techniques.length === 0}
        <p class="empty">Техники не добавлены</p>
      {:else}
        {#each $techniques as t (t.id)}
          <div class="technique-row">
            {#if editingId === t.id}
              <input
                class="edit-inp"
                type="text"
                bind:value={editName}
                onkeydown={(e) => { if (e.key === 'Enter') saveEdit(t.id); if (e.key === 'Escape') editingId = null; }}
                autofocus
              />
              <button class="btn-save-edit" onclick={() => saveEdit(t.id)}>✓</button>
              <button class="btn-cancel-edit" onclick={() => { editingId = null; }}>✕</button>
            {:else}
              <span class="technique-name">{t.name}</span>
              <div class="row-right">
                {#if deleteErrors[t.id]}
                  <span class="row-error">{deleteErrors[t.id]}</span>
                {/if}
                <button
                  class="btn-edit"
                  onclick={() => startEdit(t.id, t.name)}
                  aria-label="Переименовать {t.name}"
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
        {/each}
      {/if}
    </div>

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
    width: 420px;
    max-width: calc(100vw - 32px);
    display: flex;
    flex-direction: column;
    gap: 16px;
    box-shadow: 0 16px 48px rgba(0, 0, 0, 0.5);
  }

  .title {
    font-size: 1.05rem;
    font-weight: 600;
    color: #e8edf2;
    margin: 0;
  }

  .list {
    display: flex;
    flex-direction: column;
    gap: 2px;
    max-height: 300px;
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

  .technique-name {
    font-size: 0.9rem;
    color: #c8d8e8;
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
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

  .edit-inp {
    flex: 1;
    background: #060e18;
    border: 1px solid #2a4f73;
    border-radius: 4px;
    color: #e8edf2;
    font-size: 0.88rem;
    padding: 4px 8px;
    outline: none;
  }

  .btn-save-edit, .btn-cancel-edit {
    background: none;
    border: none;
    cursor: pointer;
    padding: 4px 6px;
    border-radius: 4px;
    font-size: 0.85rem;
    transition: background 0.12s;
  }

  .btn-save-edit { color: #52d47a; }
  .btn-save-edit:hover { background: rgba(82, 212, 122, 0.12); }
  .btn-cancel-edit { color: #e05252; }
  .btn-cancel-edit:hover { background: rgba(224, 82, 82, 0.1); }

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
  }

  .actions {
    display: flex;
    justify-content: flex-end;
    margin-top: 4px;
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
