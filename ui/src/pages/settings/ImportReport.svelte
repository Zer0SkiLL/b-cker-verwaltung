<script lang="ts">
  import * as api from '../../lib/api';
  import { pickSavePath, todayStamp } from '../../lib/desktop';
  import type { ReportEntry } from '../../lib/types';

  let { report }: { report: ReportEntry[] } = $props();
  let info = $state<string | null>(null);
  let error = $state<string | null>(null);

  async function save() {
    const path = await pickSavePath(`Import-Bericht-${todayStamp()}.csv`, 'csv');
    if (!path) return;
    try {
      await api.exportReport(report, path);
      info = `Gespeichert: ${path}`;
    } catch (err) {
      error = (err as Error).message;
    }
  }
</script>

{#if report.length === 0}
  <h3>Import-Bericht</h3>
  <p class="muted">Keine Hinweise.</p>
{:else}
  <details class="report">
    <summary><h3>Import-Bericht ({report.length} {report.length === 1 ? 'Hinweis' : 'Hinweise'})</h3></summary>
    <p class="muted">Alle Hinweise als Liste, zum Nachbearbeiten auch als CSV.</p>
    <table>
      <thead><tr><th>Zeile</th><th>Spalte</th><th>Meldung</th></tr></thead>
      <tbody>
        {#each report as r, i (i)}
          <tr><td>{r.line}</td><td>{r.column ?? ''}</td><td>{r.message}</td></tr>
        {/each}
      </tbody>
    </table>
    <button onclick={save}>Bericht als CSV speichern…</button>
  </details>
{/if}
{#if info}<p class="muted">{info}</p>{/if}
{#if error}<div class="form-error">{error}</div>{/if}

<style>
  .report { margin: 0.5rem 0 1rem; }
  .report summary { cursor: pointer; }
  .report summary h3 { display: inline; }
  .report table { margin-bottom: 0.5rem; }
</style>
