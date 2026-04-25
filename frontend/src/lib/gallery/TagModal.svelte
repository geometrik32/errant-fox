<script lang="ts">
  import { fighters } from '../../stores';
  import { patchVideo } from '../api/videos';
  import type { Video, VideoFull } from '../api/types';

  interface Props {
    video: Video;
    onsaved?: (updated: VideoFull) => void;
    onclose?: () => void;
  }

  let { video, onsaved, onclose }: Props = $props();

  let fighterAId = $state(video.fighter_a?.id ?? '');
  let fighterBId = $state(video.fighter_b?.id ?? '');
  let saving = $state(false);
  let errorMsg = $state('');

  let optionsForA = $derived($fighters.filter((f) => f.id !== fighterBId));
  let optionsForB = $derived($fighters.filter((f) => f.id !== fighterAId));

  let canSave = $derived(!!fighterAId && !!fighterBId && !saving);

  let formattedDate = $derived(
    new Date(video.date + 'T00:00:00').toLocaleDateString('ru-RU', {
      day: '2-digit',
      month: '2-digit',
      year: 'numeric',
    })
  );

  async function save() {
    if (!canSave) return;
    saving = true;
    errorMsg = '';
    try {
      const updated = await patchVideo(video.id, {
        fighter_a_id: fighterAId,
        fighter_b_id: fighterBId,
      });
      onsaved?.(updated);
    } catch (e) {
      errorMsg = e instanceof Error ? e.message : 'Ошибка при сохранении';
    } finally {
      saving = false;
    }
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

<div class="backdrop" onclick={handleBackdrop} role="dialog" aria-modal="true" aria-label="Разметка видео">
  <div class="modal">
    <h2 class="title">Разметка видео</h2>

    <div class="field">
      <label class="label" for="tag-date">Дата тренировки</label>
      <input id="tag-date" class="input readonly" type="text" value={formattedDate} readonly />
    </div>

    <div class="field">
      <label class="label" for="fighter-a">Боец A</label>
      <select id="fighter-a" class="select" bind:value={fighterAId}>
        <option value="">— выберите бойца —</option>
        {#each optionsForA as f (f.id)}
          <option value={f.id}>{f.display_name}</option>
        {/each}
      </select>
    </div>

    <div class="field">
      <label class="label" for="fighter-b">Боец B</label>
      <select id="fighter-b" class="select" bind:value={fighterBId}>
        <option value="">— выберите бойца —</option>
        {#each optionsForB as f (f.id)}
          <option value={f.id}>{f.display_name}</option>
        {/each}
      </select>
    </div>

    {#if errorMsg}
      <p class="error">{errorMsg}</p>
    {/if}

    <div class="actions">
      <button class="btn-cancel" onclick={onclose} disabled={saving}>Отмена</button>
      <button class="btn-save" onclick={save} disabled={!canSave}>
        {saving ? 'Сохранение…' : 'Сохранить'}
      </button>
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
    width: 380px;
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

  .field {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .label {
    font-size: 0.78rem;
    color: #6b8aab;
    font-weight: 500;
  }

  .input,
  .select {
    background: #060e18;
    border: 1px solid #1f3a57;
    border-radius: 6px;
    color: #e8edf2;
    font-size: 0.9rem;
    padding: 8px 10px;
    outline: none;
    width: 100%;
    transition: border-color 0.12s;
  }

  .input:focus,
  .select:focus {
    border-color: #2a4f73;
  }

  .input.readonly {
    color: #4a6280;
    cursor: default;
  }

  .select option {
    background: #0f2035;
  }

  .error {
    font-size: 0.83rem;
    color: #e05252;
    margin: 0;
  }

  .actions {
    display: flex;
    gap: 10px;
    justify-content: flex-end;
    margin-top: 4px;
  }

  .btn-cancel,
  .btn-save {
    border: none;
    border-radius: 6px;
    font-size: 0.88rem;
    font-weight: 500;
    padding: 8px 18px;
    cursor: pointer;
    transition: background 0.12s;
  }

  .btn-cancel {
    background: #1a3050;
    color: #a0b4c8;
  }

  .btn-cancel:hover:not(:disabled) {
    background: #1f3a57;
  }

  .btn-save {
    background: #DB841F;
    color: #fff;
  }

  .btn-save:hover:not(:disabled) {
    background: #c4731a;
  }

  .btn-cancel:disabled,
  .btn-save:disabled {
    opacity: 0.45;
    cursor: not-allowed;
  }
</style>
