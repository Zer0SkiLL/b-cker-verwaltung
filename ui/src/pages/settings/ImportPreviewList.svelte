<script lang="ts">
  // Vorschau gegliedert wie nachher im Programm: Betriebe mit Personen, Einzelmitglieder, übersprungene Zeilen.
  import { hasNotes, linesText } from '../../lib/importSummary';
  import type { ImportPreview } from '../../lib/types';

  let { preview }: { preview: ImportPreview } = $props();
  let onlyNotes = $state(false);

  const orgs = $derived(onlyNotes ? preview.organisations.filter(hasNotes) : preview.organisations);
  const individuals = $derived(
    onlyNotes ? preview.individuals.filter((i) => i.person.notes.length > 0) : preview.individuals,
  );
</script>

<label class="filter"><input type="checkbox" bind:checked={onlyNotes} /> Nur mit Hinweisen</label>

<table class="preview">
    <colgroup>
      <col style="width: 24%" /><col style="width: 11%" /><col style="width: 14%" />
      <col style="width: 20%" /><col style="width: 23%" /><col style="width: 6.5rem" />
    </colgroup>
    <thead><tr><th>Betrieb / Person</th><th>Ort</th><th>Kategorie</th><th>Labels</th><th>Hinweise</th><th>Quelle</th></tr></thead>
    <tbody>
      {#each orgs as o (o.lines[0])}
        <tr class="org">
          <td><strong>{o.name}</strong></td>
          <td>{o.city ?? ''}</td>
          <td>{o.category ?? ''}</td>
          <td>{o.labels.join(', ')}</td>
          <td class="notes">{o.notes.join('; ')}</td>
          <td class="muted">{linesText(o.lines)}</td>
        </tr>
        {#each o.persons as p (p.line)}
          <tr>
            <td class="person">{p.name} <span class="muted">({p.function})</span></td>
            <td></td><td></td>
            <td>{p.labels.join(', ')}</td>
            <td class="notes">{p.notes.join('; ')}</td>
            <td class="muted">Zeile {p.line}</td>
          </tr>
        {/each}
      {/each}
      {#if individuals.length > 0}
        <tr><th colspan="6">Einzelmitglieder</th></tr>
        {#each individuals as i (i.person.line)}
          <tr>
            <td>{i.person.name}</td>
            <td>{i.city ?? ''}</td>
            <td>{i.category ?? ''}</td>
            <td>{i.person.labels.join(', ')}</td>
            <td class="notes">{i.person.notes.join('; ')}</td>
            <td class="muted">Zeile {i.person.line}</td>
          </tr>
        {/each}
      {/if}
      {#if preview.skipped.length > 0}
        <tr><th colspan="6">Übersprungene Zeilen</th></tr>
        {#each preview.skipped as s (s.line)}
          <tr><td colspan="4"></td><td class="notes">{s.reason}</td><td class="muted">Zeile {s.line}</td></tr>
        {/each}
      {/if}
    </tbody>
  </table>

<style>
  .preview { table-layout: fixed; width: 100%; margin-bottom: 0.75rem; }
  .preview td { overflow-wrap: break-word; hyphens: auto; vertical-align: top; }
  .filter { display: inline-flex; gap: 0.4rem; align-items: center; margin: 0.25rem 0 0.5rem; }
  .person { padding-left: 1.5rem; }
  .org td { border-top: 1px solid var(--border); }
  .notes { color: var(--danger); font-size: 0.9em; }
</style>
