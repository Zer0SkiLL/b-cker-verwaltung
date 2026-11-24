<script lang="ts">
  import * as api from '../lib/api';
  import { runWrite } from '../lib/session';
  import type { AssignedLabel, EntryKind, FilterOptions, MemberRef } from '../lib/types';

  let {
    target,
    kind,
    assigned,
    labelGroups,
    readOnly,
  }: {
    target: MemberRef;
    kind: EntryKind;
    assigned: AssignedLabel[];
    labelGroups: FilterOptions['labelGroups'];
    readOnly: boolean;
  } = $props();

  let choice = $state('');
  let error = $state<string | null>(null);

  const assignedIds = $derived(new Set(assigned.map((a) => a.assignment.label_id)));
  const groups = $derived(
    labelGroups
      .filter((g) => g.group.applies_to === kind || g.group.applies_to === 'Both')
      .map((g) => ({ ...g, labels: g.labels.filter((l) => !assignedIds.has(l.id)) }))
      .filter((g) => g.labels.length > 0),
  );

  async function add() {
    if (choice === '') return;
    try {
      await runWrite(() =>
        api.assignLabel({ label_id: Number(choice), target, assigned_on: null, note: null }),
      );
      choice = '';
      error = null;
    } catch (err) {
      error = (err as Error).message;
    }
  }

  async function remove(id: number) {
    try {
      await runWrite(() => api.unassignLabel(id));
    } catch (err) {
      error = (err as Error).message;
    }
  }
</script>

<div>
  {#each assigned as a (a.assignment.id)}
    <span class="chip" title={a.groupName}>
      {a.labelName}
      {#if !readOnly}
        <button class="link" aria-label="Label {a.labelName} entfernen" onclick={() => remove(a.assignment.id)}>×</button>
      {/if}
    </span>
  {:else}
    <span class="muted">Keine Labels</span>
  {/each}
</div>
{#if !readOnly}
  <div class="toolbar add">
    <select bind:value={choice} aria-label="Label hinzufügen">
      <option value="">Label hinzufügen…</option>
      {#each groups as g (g.group.id)}
        <optgroup label={g.group.name}>
          {#each g.labels as l (l.id)}<option value={String(l.id)}>{l.name}</option>{/each}
        </optgroup>
      {/each}
    </select>
    <button disabled={choice === ''} onclick={add}>Hinzufügen</button>
  </div>
{/if}
{#if error}<div class="form-error">{error}</div>{/if}

<style>
  .add { margin-top: 0.5rem; }
</style>
