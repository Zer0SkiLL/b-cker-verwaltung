<script lang="ts">
  import { readOnly as readOnlyStore } from '../../lib/session';
  import * as api from '../../lib/api';
  import { confirmAction, pickFile, pickSavePath, todayStamp } from '../../lib/desktop';
  import { fieldLabel, setColumn } from '../../lib/importMapping';
  import { mappingText, newEntriesText, plural, summaryText } from '../../lib/importSummary';
  import { runWrite } from '../../lib/session';
  import type { ImportPreview, ImportSummary } from '../../lib/types';
  import ImportPreviewList from './ImportPreviewList.svelte';
  import ImportReport from './ImportReport.svelte';

  let path = $state<string | null>(null);
  let preview = $state<ImportPreview | null>(null);
  let summary = $state<ImportSummary | null>(null);
  let error = $state<string | null>(null);
  let info = $state<string | null>(null);
  let busy = $state(false);

  const readOnly = $derived($readOnlyStore);
  const newLines = $derived(preview ? newEntriesText(preview.newEntries) : []);

  async function run(task: () => Promise<void>) {
    busy = true;
    error = null;
    info = null;
    try {
      await task();
    } catch (err) {
      error = (err as Error).message;
    } finally {
      busy = false;
    }
  }

  const choose = () =>
    run(async () => {
      const picked = await pickFile('Excel', ['xlsx', 'xlsm', 'xls', 'ods']);
      if (!picked) return;
      path = picked;
      summary = null;
      preview = await api.importPreview(picked, null);
    });

  const remap = (index: number, key: string) =>
    run(async () => {
      if (!path || !preview) return;
      preview = await api.importPreview(path, setColumn(preview.mapping, index, key));
    });

  const exportPreview = () =>
    run(async () => {
      if (!path || !preview) return;
      const out = await pickSavePath(`Import-Vorschau-${todayStamp()}.xlsx`, 'xlsx');
      if (!out) return;
      await api.exportImportPreview(path, preview.mapping, out);
      info = `Gespeichert: ${out}`;
    });

  const apply = () =>
    run(async () => {
      if (!path || !preview) return;
      const question =
        `${plural(preview.organisations.length, 'Betrieb', 'Betriebe')}, ${plural(preview.persons, 'Person', 'Personen')} ` +
        `und ${plural(preview.individuals.length, 'Einzelmitglied', 'Einzelmitglieder')} anlegen?`;
      if (!(await confirmAction(question, 'Übernehmen'))) return;
      const file = path;
      const mapping = preview.mapping;
      summary = await runWrite(() => api.importApply(file, mapping));
      preview = null;
    });

  const undo = () =>
    run(async () => {
      if (!summary) return;
      const ok = await confirmAction(
        'Import rückgängig machen? Die Datei wird auf den Stand unmittelbar vor dem Import zurückgesetzt.',
        'Rückgängig machen',
      );
      if (!ok) return;
      const backup = summary.undoBackup;
      await runWrite(() => api.importUndo(backup));
      summary = null;
      info = 'Import rückgängig gemacht.';
    });
</script>

<h2>Excel-Import</h2>
<p class="muted">
  Übernimmt die bisherige Mitgliederliste (erstes Tabellenblatt, erste Zeile = Spaltennamen).
  Zuerst wird nur geprüft; übernommen wird erst mit „Übernehmen“ – und nur in eine leere Datei.
</p>
{#if error}<div class="form-error">{error}</div>{/if}

<div class="toolbar">
  <button class="primary" disabled={busy || readOnly} onclick={choose}>1. Excel-Datei wählen…</button>
  {#if path}<span class="muted">{path}</span>{/if}
</div>

{#if preview}
  <details class="mapping">
    <summary><h3>2. Spalten zuordnen – {mappingText(preview.mapping)}</h3></summary>
    <p class="muted">Nur ändern, wenn eine Spalte falsch erkannt wurde. Die Vorschau passt sich sofort an.</p>
    <div class="mapping-grid">
      {#each preview.headers as header, i (i)}
        <label class="map">
          <span>{header || `(Spalte ${i + 1} ohne Namen)`}</span>
          <select value={preview.mapping[i]} disabled={busy} aria-label="Zuordnung für {header}" onchange={(e) => remap(i, e.currentTarget.value)}>
            {#each preview.fields as key (key)}<option value={key}>{fieldLabel(key)}</option>{/each}
          </select>
        </label>
      {/each}
    </div>
  </details>

  <div class="card summary">
    <strong>{summaryText(preview)}</strong>
    <div>
      Neu angelegt werden:
      {#if newLines.length === 0}<span class="muted">nichts Neues</span>{/if}
    </div>
    {#if newLines.length > 0}
      <ul>{#each newLines as line (line)}<li>{line}</li>{/each}</ul>
    {/if}
    {#if preview.reconcileError}
      <div class="form-error">{preview.reconcileError}</div>
    {/if}
    {#if !preview.databaseEmpty}
      <div class="form-error">Import nur in eine leere Datei möglich. Bitte neue Datei anlegen.</div>
    {/if}
    <div class="toolbar actions">
      <button
        class="primary"
        disabled={busy || readOnly || !preview.databaseEmpty || preview.reconcileError !== null}
        onclick={apply}
      >Übernehmen</button>
      <button disabled={busy} onclick={exportPreview}>Vorschau als Excel speichern…</button>
    </div>
    <p class="muted hint">Prüfen Sie vorher die Vorschau unten. Übernehmen fragt nach und lässt sich rückgängig machen.</p>
  </div>

  <h3>Vorschau</h3>
  <ImportPreviewList {preview} />
  <ImportReport report={preview.report} />
{/if}

{#if info}<p>{info}</p>{/if}

{#if summary}
  <div class="card">
    <strong>Import abgeschlossen:</strong>
    {plural(summary.organisations, 'Betrieb', 'Betriebe')}, {plural(summary.persons, 'Person', 'Personen')} und
    {plural(summary.individuals, 'Einzelmitglied', 'Einzelmitglieder')} übernommen.
    Unklare Zeilen bitte anhand des Berichts nachbearbeiten.
    <div class="toolbar actions">
      <button disabled={busy || readOnly} onclick={undo}>Import rückgängig machen</button>
    </div>
  </div>
  <ImportReport report={summary.report} />
{/if}

<style>
  .summary ul { margin: 0.25rem 0 0; }
  .actions { margin-top: 0.5rem; }
  .mapping { margin: 0.5rem 0 1rem; }
  .mapping summary { cursor: pointer; }
  .mapping summary h3 { display: inline; }
  .mapping-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
    gap: 0.35rem 2rem;
  }
  .map { display: flex; align-items: center; justify-content: space-between; gap: 0.75rem; }
  .map select { width: 13rem; flex: none; }
  .hint { margin: 0.25rem 0 0; }
</style>
