<script lang="ts">
  import * as api from '../lib/api';
  import { confirmAction } from '../lib/desktop';
  import { navigate } from '../lib/router';
  import { untrack } from 'svelte';
  import { bumpData, dataVersion, noticesFor, refreshSession, sessionInfo } from '../lib/session';
  import { renderMarkdown } from '../lib/markdown';
  import { installUpdate, reopenLastFile, updateState } from '../lib/updates';
  import { replace } from '../lib/router';

  let error = $state<string | null>(null);
  const notices = $derived($sessionInfo ? noticesFor($sessionInfo) : []);
  let errorVersion = 0;

  // Meldung verschwindet nach der nächsten erfolgreichen Aktion (Datenversion ändert sich).
  $effect(() => {
    const version = $dataVersion;
    untrack(() => {
      if (error !== null && version !== errorVersion) error = null;
    });
  });

  async function takeOver() {
    const ok = await confirmAction(
      'Wer die Datei noch offen hat, kann danach nicht mehr speichern. Übernehmen?',
      'Übernehmen',
    );
    if (!ok) return;
    try {
      sessionInfo.set(await api.takeOver());
      error = null;
      bumpData();
    } catch (err) {
      await refreshSession();
      errorVersion = $dataVersion;
      error = (err as Error).message;
    }
  }
</script>

{#if $updateState.kind === 'available'}
  <div class="notice update" role="status">
    <div class="grow">
      <span>Version {$updateState.version} ist verfügbar</span>
      {#if $updateState.notes}
        <details>
          <summary>Was ist neu?</summary>
          <div class="notes">{@html renderMarkdown($updateState.notes)}</div>
        </details>
      {/if}
    </div>
    <button class="primary" onclick={() => installUpdate()}>Jetzt installieren</button>
  </div>
{:else if $updateState.kind === 'installing'}
  <div class="notice update" role="status">
    <span>Update wird installiert{$updateState.percent !== null ? ` (${$updateState.percent} %)` : ''} … Das Programm startet danach neu.</span>
    {#if $updateState.note}<span class="muted">{$updateState.note}</span>{/if}
  </div>
{:else if $updateState.kind === 'failed' && $updateState.fileClosed}
  <div class="form-error">
    {$updateState.message}
    <div class="toolbar">
      <button onclick={() => installUpdate()}>Erneut versuchen</button>
      <button onclick={() => reopenLastFile().then((page) => replace({ page }))}>Datei wieder öffnen</button>
    </div>
  </div>
{/if}

{#each notices as notice (notice.kind)}
  <div class="notice" role="status">
    <span>{notice.text}</span>
    {#if notice.kind === 'readOnly'}
      <button onclick={takeOver}>Übernehmen</button>
    {:else}
      <button class="link" onclick={() => navigate({ page: 'settings', tab: 'backup' })}>
        Datensicherung einrichten
      </button>
    {/if}
  </div>
{/each}
{#if error}<div class="form-error">{error}</div>{/if}

<style>
  .grow { flex: 1; }
  .notes { margin-top: 0.4rem; }
  .notice { display: flex; gap: 1rem; align-items: center; justify-content: space-between; background: var(--warn-bg); border: 1px solid var(--warn-border); border-radius: 6px; padding: 0.45rem 0.75rem; margin-bottom: 0.5rem; }
</style>
