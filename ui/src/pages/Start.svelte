<script lang="ts">
  import { onMount } from 'svelte';
  import * as api from '../lib/api';
  import { pickDatabase, pickNewDatabase, showInfo } from '../lib/desktop';
  import { replace } from '../lib/router';
  import { bumpData, sessionInfo } from '../lib/session';
  import type { OpenResult } from '../lib/types';

  let last = $state<string | null>(null);
  let error = $state<string | null>(null);
  let busy = $state(false);

  onMount(async () => {
    last = await api.lastPath().catch(() => null);
  });

  async function run(task: () => Promise<OpenResult | null>) {
    busy = true;
    error = null;
    try {
      const result = await task();
      if (result) {
        sessionInfo.set(result.info);
        bumpData();
        replace({ page: 'addresses' });
        if (result.warning) await showInfo(result.warning);
      }
    } catch (err) {
      error = (err as Error).message;
    } finally {
      busy = false;
    }
  }

  const openLast = () => run(() => api.openFile(last!));
  const openOther = () => run(async () => {
    const path = await pickDatabase();
    return path ? api.openFile(path) : null;
  });
  const createNew = () => run(async () => {
    const path = await pickNewDatabase();
    return path ? api.createFile(path) : null;
  });
</script>

<div class="start">
  <h1>Mitgliederverwaltung</h1>
  <p class="muted">Bitte die Datei mit den Mitgliederdaten öffnen oder eine neue anlegen.</p>
  {#if error}<div class="form-error">{error}</div>{/if}
  {#if last}
    <div class="card">
      <div class="muted">Zuletzt geöffnet</div>
      <div class="path">{last}</div>
      <button class="primary" disabled={busy} onclick={openLast}>Öffnen</button>
    </div>
  {/if}
  <div class="toolbar">
    <button disabled={busy} onclick={openOther}>Datei öffnen…</button>
    <button disabled={busy} onclick={createNew}>Neue Datei anlegen…</button>
  </div>
</div>

<style>
  .start { max-width: 560px; margin: 12vh auto; }
  .path { font-family: ui-monospace, monospace; margin: 0.3rem 0 0.8rem; word-break: break-all; }
</style>
