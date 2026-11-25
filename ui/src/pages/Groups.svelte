<script lang="ts">
  import EmptyHint from '../components/EmptyHint.svelte';
  import HelpButton from '../components/HelpButton.svelte';
  import { readOnly as readOnlyStore } from '../lib/session';
  import * as api from '../lib/api';
  import { confirmAction } from '../lib/desktop';
  import { navigate } from '../lib/router';
  import { dataVersion, runWrite, sessionInfo } from '../lib/session';
  import type { GroupDto } from '../lib/types';

  let groups = $state<GroupDto[]>([]);
  let error = $state<string | null>(null);
  let editing = $state<{ id: number; name: string } | null>(null);
  const readOnly = $derived($readOnlyStore);

  $effect(() => {
    void $dataVersion;
    api.listGroups().then((g) => (groups = g)).catch((err) => (error = (err as Error).message));
  });

  async function openGroup(group: GroupDto) {
    try {
      navigate({ page: 'addresses', filter: await api.loadGroup(group.id) });
    } catch (err) {
      error = (err as Error).message;
    }
  }

  async function rename() {
    if (!editing) return;
    const { id, name } = editing;
    try {
      await runWrite(() => api.renameGroup(id, name));
      editing = null;
      error = null;
    } catch (err) {
      error = (err as Error).message;
    }
  }

  async function remove(group: GroupDto) {
    if (!(await confirmAction(`Gruppe „${group.name}“ löschen? Die Adressen bleiben erhalten.`, 'Löschen'))) return;
    try {
      await runWrite(() => api.deleteGroup(group.id));
    } catch (err) {
      error = (err as Error).message;
    }
  }

  // Wird die Datei nur lesend (Sperre übernommen), offene Bearbeitungen verwerfen.
  $effect(() => {
    if (readOnly) {
      editing = null;
    }
  });
</script>

<div class="title-row"><h1>Gruppen</h1><HelpButton /></div>
<p class="muted">Gespeicherte Filter, z.B. „Alle Prüfungsexperten“. Neue Gruppen unter „Adressen“ › „Als Gruppe speichern“.</p>
{#if error}<div class="form-error">{error}</div>{/if}
<table>
  <tbody>
    {#each groups as group (group.id)}
      <tr>
        <td>
          {#if editing?.id === group.id}
            <input bind:value={editing.name} aria-label="Neuer Name" onkeydown={(e) => e.key === 'Enter' && rename()} />
            <button class="primary" onclick={rename}>Speichern</button>
            <button onclick={() => (editing = null)}>Abbrechen</button>
          {:else}
            <button class="link" onclick={() => openGroup(group)}>{group.name}</button>
          {/if}
        </td>
        <td class="actions">
          {#if !readOnly && editing?.id !== group.id}
            <button onclick={() => (editing = { id: group.id, name: group.name })}>Umbenennen</button>
            <button class="danger" onclick={() => remove(group)}>Löschen</button>
          {/if}
        </td>
      </tr>
    {:else}
      <tr><td><EmptyHint
        text="Noch keine Gruppen. Stellen Sie unter „Adressen“ einen Filter zusammen und klicken Sie auf „Als Gruppe speichern“."
        help={{ label: 'Wie stelle ich einen Versand zusammen?', slug: '07-versand' }}
      /></td></tr>
    {/each}
  </tbody>
</table>

<style>
  .actions { text-align: right; }
</style>
