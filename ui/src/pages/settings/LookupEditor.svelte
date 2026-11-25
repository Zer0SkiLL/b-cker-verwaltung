<script lang="ts">
  import * as api from '../../lib/api';
  import { confirmAction } from '../../lib/desktop';
  import { dataVersion, runWrite } from '../../lib/session';
  import type { LookupKind, LookupUsage } from '../../lib/types';

  let { kind, readOnly }: { kind: LookupKind; readOnly: boolean } = $props();

  let items = $state<LookupUsage[]>([]);
  let newName = $state('');
  let editing = $state<{ id: number; name: string } | null>(null);
  let error = $state<string | null>(null);

  $effect(() => {
    const current = kind;
    void $dataVersion;
    api.listLookups(current).then((l) => (items = l)).catch((err) => (error = (err as Error).message));
  });

  async function act(task: () => Promise<unknown>) {
    try {
      await runWrite(task);
      error = null;
      return true;
    } catch (err) {
      error = (err as Error).message;
      return false;
    }
  }

  async function create() {
    const name = newName.trim();
    if (name && (await act(() => api.createLookup(kind, name)))) newName = '';
  }

  async function rename() {
    if (!editing) return;
    const { id, name } = editing;
    if (await act(() => api.renameLookup(id, name))) editing = null;
  }

  async function remove(item: LookupUsage) {
    const question =
      item.usage > 0
        ? `„${item.value.name}“ wird ${item.usage}-mal verwendet. Trotzdem löschen? Die Verwendungen werden geleert.`
        : `„${item.value.name}“ löschen?`;
    if (await confirmAction(question, 'Löschen')) await act(() => api.deleteLookup(item.value.id, item.usage > 0));
  }

  // Wird die Datei nur lesend (Sperre übernommen), offene Bearbeitungen verwerfen.
  $effect(() => {
    if (readOnly) {
      editing = null;
    }
  });
</script>

{#if error}<div class="form-error">{error}</div>{/if}
<table>
  <tbody>
    {#each items as item (item.value.id)}
      <tr>
        <td>
          {#if editing?.id === item.value.id}
            <input bind:value={editing.name} aria-label="Neuer Name" onkeydown={(e) => e.key === 'Enter' && rename()} />
            <button class="primary" onclick={rename}>Speichern</button>
            <button onclick={() => (editing = null)}>Abbrechen</button>
          {:else}
            {item.value.name}
          {/if}
        </td>
        <td class="muted">{item.usage}× verwendet</td>
        <td class="right">
          {#if !readOnly && editing?.id !== item.value.id}
            <button onclick={() => (editing = { id: item.value.id, name: item.value.name })}>Umbenennen</button>
            <button class="danger" onclick={() => remove(item)}>Löschen</button>
          {/if}
        </td>
      </tr>
    {:else}
      <tr><td class="muted">Noch keine Einträge.</td></tr>
    {/each}
  </tbody>
</table>
{#if !readOnly}
  <div class="toolbar add">
    <input placeholder="Neuer Eintrag" bind:value={newName} onkeydown={(e) => e.key === 'Enter' && create()} />
    <button onclick={create}>Hinzufügen</button>
  </div>
{/if}

<style>
  .right { text-align: right; white-space: nowrap; }
  .add { margin-top: 0.75rem; }
</style>
