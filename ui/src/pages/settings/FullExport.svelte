<script lang="ts">
  import * as api from '../../lib/api';
  import { pickSavePath, todayStamp } from '../../lib/desktop';

  let info = $state<string | null>(null);
  let error = $state<string | null>(null);

  async function run() {
    const path = await pickSavePath(`Mitgliederverwaltung-${todayStamp()}.xlsx`, 'xlsx');
    if (!path) return;
    try {
      await api.exportAll(path);
      info = `Gespeichert: ${path}`;
      error = null;
    } catch (err) {
      error = (err as Error).message;
    }
  }
</script>

<h2>Gesamtexport</h2>
<p class="muted">
  Alle Daten in einer Excel-Datei mit je einem Blatt pro Bereich. Lesbar auch ohne dieses Programm,
  z.B. als Archiv oder für einen späteren Wechsel.
</p>
<button class="primary" onclick={run}>Alle Daten als Excel exportieren…</button>
{#if info}<p>{info}</p>{/if}
{#if error}<div class="form-error">{error}</div>{/if}
