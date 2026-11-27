<script lang="ts">
  // „Problem melden“: Bericht zum Kopieren, ohne Vereinsdaten.
  import * as api from '../lib/api';
  import { copyText, pickSavePath, todayStamp } from '../lib/desktop';
  import { buildReport, recentActions, reportOpen, type ActionEntry } from '../lib/diagnostics';
  import { route } from '../lib/router';
  import { sessionInfo } from '../lib/session';
  import type { RetentionPolicy, SnapshotDto } from '../lib/types';

  let description = $state('');
  let copied = $state(false);
  let saved = $state<string | null>(null);
  let saveError = $state<string | null>(null);
  let snapshot = $state<SnapshotDto | null>(null);
  let retention = $state<RetentionPolicy | null>(null);
  let actions = $state<ActionEntry[]>([]);

  // Beim Öffnen frisch einlesen; Fehler beim Lesen dürfen den Bericht nicht verhindern.
  async function load() {
    actions = recentActions();
    copied = false;
    saved = null;
    saveError = null;
    try {
      snapshot = await api.diagnosticsSnapshot();
    } catch {
      snapshot = null; // Bericht zeigt dann „unbekannt“
    }
    try {
      retention = $sessionInfo ? await api.getRetention() : null;
    } catch {
      retention = null;
    }
  }

  $effect(() => {
    if ($reportOpen) void load();
  });

  const report = $derived(
    buildReport({ description, now: new Date(), route: $route, session: $sessionInfo, retention, actions, snapshot }),
  );

  function close() {
    reportOpen.set(false);
    description = '';
  }

  async function copy() {
    await copyText(report);
    copied = true;
  }

  async function saveFile() {
    const path = await pickSavePath(`Problembericht-${todayStamp()}.txt`, 'txt');
    if (!path) return;
    try {
      await api.saveReport(path, report);
      saved = `Gespeichert: ${path}`;
      saveError = null;
    } catch (err) {
      saveError = (err as Error).message;
    }
  }

  function onKey(event: KeyboardEvent) {
    if ($reportOpen && event.key === 'Escape') close();
  }
</script>

<svelte:window onkeydown={onKey} />

{#if $reportOpen}
  <div class="backdrop">
    <dialog open aria-labelledby="report-title">
      <h2 id="report-title">Problem melden</h2>
      <p class="muted">
        Fügen Sie den Bericht in Ihre Nachricht an den Entwickler ein oder speichern Sie ihn als Datei und hängen Sie diese an. Ein Bildschirmfoto machen Sie unter Windows mit
        <strong>Windows + Umschalt + S</strong> und fügen es ebenfalls ein. Der Bericht enthält keine Mitgliederdaten.
      </p>
      <label for="report-description">Was ist passiert?</label>
      <textarea id="report-description" rows="3" bind:value={description} placeholder="z.B. Beim Speichern eines Betriebs kam eine Fehlermeldung"></textarea>
      <label for="report-text">Bericht</label>
      <textarea id="report-text" rows="16" readonly value={report}></textarea>
      <div class="toolbar">
        <button class="primary" onclick={copy}>Bericht kopieren</button>
        <button onclick={saveFile}>Als Datei speichern…</button>
        <button onclick={close}>Schliessen</button>
        {#if copied}<span class="muted">Kopiert.</span>{/if}
      </div>
      {#if saved}<p class="muted">{saved}</p>{/if}
      {#if saveError}<p class="error" role="alert">{saveError}</p>{/if}
    </dialog>
  </div>
{/if}

<style>
  .backdrop { position: fixed; inset: 0; background: rgb(0 0 0 / 0.25); display: grid; place-items: center; z-index: 50; }
  dialog { position: static; width: min(720px, 92vw); max-height: 90vh; overflow: auto; border: 1px solid var(--border); border-radius: 8px; padding: 1rem 1.25rem; display: flex; flex-direction: column; gap: 0.4rem; }
  textarea { width: 100%; font: inherit; }
  #report-text { font-family: ui-monospace, Menlo, Consolas, monospace; font-size: 0.85em; }
</style>
