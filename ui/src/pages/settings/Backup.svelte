<script lang="ts">
  import * as api from '../../lib/api';
  import { formatDateTime } from '../../lib/dates';
  import { confirmAction, pickFolder } from '../../lib/desktop';
  import { dataVersion, refreshSession, runWrite, sessionInfo } from '../../lib/session';
  import type { BackupFileDto, BackupOverview, SecondaryResultDto } from '../../lib/types';

  let overview = $state<BackupOverview | null>(null);
  let error = $state<string | null>(null);
  let info = $state<string | null>(null);
  const readOnly = $derived($sessionInfo?.readOnly ?? true);

  $effect(() => {
    void $dataVersion;
    api.backupOverview().then((o) => (overview = o)).catch((err) => (error = (err as Error).message));
  });

  function secondaryText(result: SecondaryResultDto): string {
    if (result.kind === 'done') return `Zweiter Ort: ${result.path}`;
    if (result.kind === 'unavailable') return result.reason;
    return 'Kein zweiter Ort eingerichtet.';
  }

  async function act(task: () => Promise<string | null>) {
    error = null;
    try {
      info = await task();
    } catch (err) {
      error = (err as Error).message;
    }
  }

  const backupNow = () =>
    act(async () => {
      const done = await runWrite(() => api.backupNow());
      await refreshSession();
      return `Gesichert: ${done.local}. ${secondaryText(done.secondary)}`;
    });

  const chooseSecondary = () =>
    act(async () => {
      const dir = await pickFolder();
      if (!dir) return null;
      await runWrite(() => api.setSecondaryDir(dir));
      await refreshSession();
      return `Zweiter Sicherungsort: ${dir}. Gesichert wird beim Schliessen und mit „Jetzt sichern“.`;
    });

  const removeSecondary = () =>
    act(async () => {
      await runWrite(() => api.setSecondaryDir(null));
      await refreshSession();
      return 'Zweiter Sicherungsort entfernt.';
    });

  const restore = (file: BackupFileDto) =>
    act(async () => {
      const ok = await confirmAction(
        `Stand vom ${file.created} wiederherstellen? Der aktuelle Stand wird vorher gesichert.`,
        'Wiederherstellen',
      );
      if (!ok) return null;
      await runWrite(() => api.restoreBackup(file.path));
      await refreshSession();
      return `Stand vom ${file.created} wiederhergestellt.`;
    });
</script>

<h2>Datensicherung</h2>
<p class="muted">
  Beim Öffnen wird automatisch eine Sicherung neben der Datei angelegt (die letzten 20 bleiben).
  Ein zweiter Ort (z.B. USB-Stick oder anderes Laufwerk) schützt auch vor Defekten des Speicherorts.
</p>
{#if error}<div class="form-error">{error}</div>{/if}
{#if info}<p>{info}</p>{/if}

{#if overview}
  <div class="card">
    <div><strong>Zweiter Ort:</strong> {overview.status.secondaryDir ?? 'nicht eingerichtet'}</div>
    <div><strong>Letzte Sicherung dort:</strong> {overview.status.lastSecondary ? formatDateTime(overview.status.lastSecondary) : '–'}</div>
    {#if overview.status.reminderDue}<div class="error">Bitte zweiten Ort einrichten bzw. sichern.</div>{/if}
    <div class="toolbar actions">
      <button class="primary" disabled={readOnly} onclick={backupNow}>Jetzt sichern</button>
      <button disabled={readOnly} onclick={chooseSecondary}>Zweiten Ort wählen…</button>
      {#if overview.status.secondaryDir}
        <button disabled={readOnly} onclick={removeSecondary}>Zweiten Ort entfernen</button>
      {/if}
    </div>
  </div>

  <h2>Sicherungen in {overview.localDir}</h2>
  <table>
    <tbody>
      {#each overview.files as file (file.path)}
        <tr>
          <td>{file.created}</td>
          <td class="muted path">{file.path}</td>
          <td class="right"><button disabled={readOnly} onclick={() => restore(file)}>Wiederherstellen</button></td>
        </tr>
      {:else}
        <tr><td class="muted">Noch keine Sicherungen.</td></tr>
      {/each}
    </tbody>
  </table>
{/if}

<style>
  .actions { margin-top: 0.75rem; }
  .path { font-size: 0.85rem; word-break: break-all; }
  .right { text-align: right; }
</style>
