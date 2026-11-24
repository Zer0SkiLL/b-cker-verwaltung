<script lang="ts">
  import * as api from '../../lib/api';
  import { confirmAction } from '../../lib/desktop';
  import { dataVersion, runWrite } from '../../lib/session';
  import type { AppliesTo, LabelGroupTree } from '../../lib/types';

  let { readOnly }: { readOnly: boolean } = $props();

  const APPLIES: Record<AppliesTo, string> = { Person: 'Personen', Organisation: 'Betriebe', Both: 'Personen und Betriebe' };

  let tree = $state<LabelGroupTree[]>([]);
  let error = $state<string | null>(null);
  let groupName = $state('');
  let groupApplies = $state<AppliesTo>('Person');
  let newLabel = $state<Record<number, string>>({});
  let editing = $state<{ id: number; name: string } | null>(null);

  $effect(() => {
    void $dataVersion;
    api.listLabelTree().then((t) => (tree = t)).catch((err) => (error = (err as Error).message));
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

  async function createGroup() {
    const name = groupName.trim();
    if (name && (await act(() => api.createLabelGroup(name, groupApplies)))) groupName = '';
  }

  async function addLabel(groupId: number) {
    const name = (newLabel[groupId] ?? '').trim();
    if (name && (await act(() => api.createLabel(groupId, name)))) newLabel = { ...newLabel, [groupId]: '' };
  }

  async function rename() {
    if (!editing) return;
    const { id, name } = editing;
    if (await act(() => api.renameLabel(id, name))) editing = null;
  }

  async function removeLabel(id: number, name: string, usage: number) {
    const question = usage > 0 ? `Label „${name}“ ist ${usage}-mal vergeben. Trotzdem löschen?` : `Label „${name}“ löschen?`;
    if (await confirmAction(question, 'Löschen')) await act(() => api.deleteLabel(id, usage > 0));
  }

  async function removeGroup(g: LabelGroupTree) {
    const usage = g.labels.reduce((sum, l) => sum + l.usage, 0);
    const question = usage > 0
      ? `Gruppe „${g.group.name}“ mit allen Labels löschen? Sie sind ${usage}-mal vergeben.`
      : `Gruppe „${g.group.name}“ löschen?`;
    if (await confirmAction(question, 'Löschen')) await act(() => api.deleteLabelGroup(g.group.id, usage > 0));
  }
</script>

{#if error}<div class="form-error">{error}</div>{/if}
{#each tree as g (g.group.id)}
  <div class="card">
    <div class="head">
      <strong>{g.group.name}</strong>
      <span class="muted">gilt für {APPLIES[g.group.applies_to]}</span>
      {#if !readOnly}<button class="danger" onclick={() => removeGroup(g)}>Gruppe löschen</button>{/if}
    </div>
    <table>
      <tbody>
        {#each g.labels as l (l.label.id)}
          <tr>
            <td>
              {#if editing?.id === l.label.id}
                <input bind:value={editing.name} aria-label="Neuer Name" onkeydown={(e) => e.key === 'Enter' && rename()} />
                <button class="primary" onclick={rename}>Speichern</button>
                <button onclick={() => (editing = null)}>Abbrechen</button>
              {:else}
                {l.label.name}
              {/if}
            </td>
            <td class="muted">{l.usage}× vergeben</td>
            <td class="right">
              {#if !readOnly && editing?.id !== l.label.id}
                <button onclick={() => (editing = { id: l.label.id, name: l.label.name })}>Umbenennen</button>
                <button class="danger" onclick={() => removeLabel(l.label.id, l.label.name, l.usage)}>Löschen</button>
              {/if}
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
    {#if !readOnly}
      <div class="toolbar add">
        <input placeholder="Neues Label" value={newLabel[g.group.id] ?? ''} oninput={(e) => (newLabel = { ...newLabel, [g.group.id]: e.currentTarget.value })} onkeydown={(e) => e.key === 'Enter' && addLabel(g.group.id)} />
        <button onclick={() => addLabel(g.group.id)}>Hinzufügen</button>
      </div>
    {/if}
  </div>
{:else}
  <p class="muted">Noch keine Label-Gruppen. Beispiele: „Rollen“ (Vorstand, Prüfungsexperte), „Ausbildung“ (Lehrbetrieb, ÜK-Zentrum).</p>
{/each}
{#if !readOnly}
  <div class="toolbar">
    <input placeholder="Neue Gruppe" bind:value={groupName} />
    <select bind:value={groupApplies} aria-label="Gilt für">
      <option value="Person">für Personen</option>
      <option value="Organisation">für Betriebe</option>
      <option value="Both">für beide</option>
    </select>
    <button onclick={createGroup}>Gruppe anlegen</button>
  </div>
{/if}

<style>
  .head { display: flex; gap: 0.75rem; align-items: center; margin-bottom: 0.5rem; }
  .head button { margin-left: auto; }
  .right { text-align: right; white-space: nowrap; }
  .add { margin-top: 0.5rem; }
</style>
