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

<h3>Import-Bericht ({report.length} Hinweise)</h3>
{#if report.length === 0}
  <p class="muted">Keine Hinweise.</p>
{:else}
  <div class="scroll">
    <table>
      <thead><tr><th>Zeile</th><th>Spalte</th><th>Meldung</th></tr></thead>
      <tbody>
        {#each report as r, i (i)}
          <tr><td>{r.line}</td><td>{r.column ?? ''}</td><td>{r.message}</td></tr>
        {/each}
      </tbody>
    </table>
  </div>
  <button onclick={save}>Bericht als CSV speichern…</button>
{/if}
{#if info}<p class="muted">{info}</p>{/if}
{#if error}<div class="form-error">{error}</div>{/if}

<style>
  .scroll { max-height: 260px; overflow: auto; margin-bottom: 0.5rem; }
</style>
