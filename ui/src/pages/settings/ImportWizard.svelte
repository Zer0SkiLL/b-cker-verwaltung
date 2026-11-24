<script lang="ts">
  import { readOnly as readOnlyStore } from '../../lib/session';
  import * as api from '../../lib/api';
  import { pickFile } from '../../lib/desktop';
  import { fieldLabel, setColumn } from '../../lib/importMapping';
  import { runWrite, sessionInfo } from '../../lib/session';
  import type { ImportPreview, ImportSummary } from '../../lib/types';
  import ImportReport from './ImportReport.svelte';

  let path = $state<string | null>(null);
  let preview = $state<ImportPreview | null>(null);
  let summary = $state<ImportSummary | null>(null);
  let error = $state<string | null>(null);
  let busy = $state(false);

  const readOnly = $derived($readOnlyStore);

  async function run(task: () => Promise<void>) {
    busy = true;
    error = null;
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

  const apply = () =>
    run(async () => {
      if (!path || !preview) return;
      const file = path;
      const mapping = preview.mapping;
      summary = await runWrite(() => api.importApply(file, mapping));
      preview = null;
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
  <h3>2. Spalten zuordnen und prüfen</h3>
  <div class="scroll">
    <table>
      <thead><tr><th>Spalte in der Datei</th><th>Wird übernommen als</th></tr></thead>
      <tbody>
        {#each preview.headers as header, i (i)}
          <tr>
            <td>{header || `(Spalte ${i + 1} ohne Namen)`}</td>
            <td>
              <select value={preview.mapping[i]} disabled={busy} aria-label="Zuordnung für {header}" onchange={(e) => remap(i, e.currentTarget.value)}>
                {#each preview.fields as key (key)}<option value={key}>{fieldLabel(key)}</option>{/each}
              </select>
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  </div>

  <p>
    <strong>{preview.rows}</strong> Zeilen → {preview.organisations} Betriebe, {preview.persons} Personen.
  </p>
  <h3>Vorschau (erste {preview.sample.length} Zeilen)</h3>
  <div class="scroll">
    <table>
      <thead><tr><th>Zeile</th><th>Betrieb</th><th>Person</th><th>Ort</th><th>Kategorie</th></tr></thead>
      <tbody>
        {#each preview.sample as r (r.line)}
          <tr><td>{r.line}</td><td>{r.organisation}</td><td>{r.person ?? ''}</td><td>{r.city ?? ''}</td><td>{r.category ?? ''}</td></tr>
        {/each}
      </tbody>
    </table>
  </div>
  <ImportReport report={preview.report} />

  <h3>3. Übernehmen</h3>
  {#if !preview.databaseEmpty}
    <div class="form-error">Import nur in eine leere Datei möglich. Bitte neue Datei anlegen.</div>
  {/if}
  <button class="primary" disabled={busy || readOnly || !preview.databaseEmpty} onclick={apply}>Übernehmen</button>
{/if}

{#if summary}
  <div class="card">
    <strong>Import abgeschlossen:</strong> {summary.organisations} Betriebe und {summary.persons} Personen übernommen.
    Unklare Zeilen bitte anhand des Berichts nachbearbeiten.
  </div>
  <ImportReport report={summary.report} />
{/if}

<style>
  .scroll { max-height: 320px; overflow: auto; margin-bottom: 0.75rem; }
</style>
