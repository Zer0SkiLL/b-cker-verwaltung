<script lang="ts">
  // „Analysedaten senden“ in der Einführungsphase. Erscheint nur, wenn der Build einen Webhook hat
  // und das Enddatum nicht überschritten ist.
  import { onMount } from 'svelte';
  import * as api from '../../lib/api';
  import { formatDate, formatDateTime } from '../../lib/dates';
  import { currentReport } from '../../lib/earlyWarning';
  import type { DiagnoseInfoDto } from '../../lib/types';

  let info = $state<DiagnoseInfoDto | null>(null);
  let preview = $state<string | null>(null);
  let error = $state<string | null>(null);

  onMount(async () => {
    try {
      info = await api.diagnoseSettings();
    } catch {
      info = null;
    }
  });

  async function toggle(event: Event) {
    const enabled = (event.currentTarget as HTMLInputElement).checked;
    try {
      await api.setDiagnoseEnabled(enabled);
      if (info) info = { ...info, enabled };
      error = null;
    } catch (err) {
      error = (err as Error).message;
    }
  }

  async function showPreview() {
    preview = (await currentReport('(automatisch: Beispiel)')).report;
  }
</script>

{#if info?.available}
  <div class="card diagnose">
    <label class="switch">
      <input type="checkbox" checked={info.enabled} onchange={toggle} />
      Analysedaten senden
    </label>
    <p class="muted">
      Hilft dem Entwickler in der Einführungsphase, Probleme früh zu erkennen. Es werden keine Mitgliederdaten gesendet.
      Endet automatisch am {formatDate(info.until)}.
    </p>
    <div class="toolbar">
      <button onclick={showPreview}>Was wird gesendet?</button>
      <span class="muted">{info.lastSent ? `Zuletzt gesendet: ${formatDateTime(info.lastSent)}` : 'Noch nichts gesendet.'}</span>
    </div>
    {#if error}<p class="error" role="alert">{error}</p>{/if}
    {#if preview}
      <textarea aria-label="Gesendeter Bericht" rows="14" readonly value={preview}></textarea>
    {/if}
  </div>
{/if}

<style>
  .diagnose { margin-top: 1rem; }
  .switch { display: flex; gap: 0.5rem; align-items: center; font-weight: 600; }
  textarea { width: 100%; margin-top: 0.5rem; font-family: ui-monospace, Menlo, Consolas, monospace; font-size: 0.85em; }
</style>
