<script lang="ts">
  import * as api from '../lib/api';
  import { confirmAction } from '../lib/desktop';
  import { navigate } from '../lib/router';
  import { untrack } from 'svelte';
  import { bumpData, dataVersion, noticesFor, refreshSession, sessionInfo } from '../lib/session';

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
  .notice { display: flex; gap: 1rem; align-items: center; justify-content: space-between; background: var(--warn-bg); border: 1px solid var(--warn-border); border-radius: 6px; padding: 0.45rem 0.75rem; margin-bottom: 0.5rem; }
</style>
