<script lang="ts">
  import * as api from '../lib/api';
  import { formatDate, todayIso } from '../lib/dates';
  import { confirmAction } from '../lib/desktop';
  import { navigate } from '../lib/router';
  import { runWrite } from '../lib/session';
  import type { Affiliation, AffiliationView, LookupValue, OptionDto } from '../lib/types';
  import AffiliationEditor from './AffiliationEditor.svelte';

  let {
    mode,
    ownerId,
    items,
    people,
    organisations,
    functions,
    showPast = $bindable(false),
    readOnly,
  }: {
    mode: 'organisation' | 'person';
    ownerId: number;
    items: AffiliationView[];
    people: OptionDto[];
    organisations: OptionDto[];
    functions: LookupValue[];
    showPast?: boolean;
    readOnly: boolean;
  } = $props();

  let editing = $state<Affiliation | 'new' | null>(null);
  let error = $state<string | null>(null);

  const period = (a: Affiliation) =>
    [formatDate(a.valid_from), formatDate(a.valid_to)].some(Boolean)
      ? `${formatDate(a.valid_from) || '…'} – ${formatDate(a.valid_to) || 'heute'}`
      : '';

  async function act(task: () => Promise<unknown>) {
    try {
      await runWrite(task);
      error = null;
    } catch (err) {
      error = (err as Error).message;
    }
  }

  const end = (a: Affiliation) => act(() => api.updateAffiliation(a.id, { ...a, valid_to: todayIso() }));

  async function remove(view: AffiliationView) {
    const name = mode === 'organisation' ? view.personName : view.organisationName;
    if (await confirmAction(`Verknüpfung mit ${name} löschen? Besser „Beenden“, wenn sie früher bestand.`)) {
      await act(() => api.deleteAffiliation(view.affiliation.id));
    }
  }

  function open(view: AffiliationView) {
    const a = view.affiliation;
    navigate(mode === 'organisation' ? { page: 'person', id: a.person_id } : { page: 'organisation', id: a.organisation_id });
  }
</script>

<label class="past"><input type="checkbox" bind:checked={showPast} /> Frühere anzeigen</label>
<table>
  <tbody>
    {#each items as view (view.affiliation.id)}
      <tr class:past-row={!view.current}>
        <td><button class="link" onclick={() => open(view)}>{mode === 'organisation' ? view.personName : view.organisationName}</button></td>
        <td>{view.functionName}</td>
        <td class="muted">{period(view.affiliation)}</td>
        <td class="actions">
          {#if !readOnly}
            <button onclick={() => (editing = view.affiliation)}>Bearbeiten</button>
            {#if view.current}<button onclick={() => end(view.affiliation)}>Beenden</button>{/if}
            <button class="danger" onclick={() => remove(view)}>Löschen</button>
          {/if}
        </td>
      </tr>
    {:else}
      <tr><td class="muted">Keine Einträge</td></tr>
    {/each}
  </tbody>
</table>
{#if error}<div class="form-error">{error}</div>{/if}
{#if editing}
  <AffiliationEditor
    organisationId={mode === 'organisation' ? ownerId : null}
    personId={mode === 'person' ? ownerId : null}
    {people}
    {organisations}
    {functions}
    existing={editing === 'new' ? null : editing}
    onDone={() => (editing = null)}
  />
{:else if !readOnly}
  <button onclick={() => (editing = 'new')}>
    {mode === 'organisation' ? 'Person verknüpfen' : 'Betrieb verknüpfen'}
  </button>
{/if}

<style>
  .past { display: inline-flex; gap: 0.3rem; align-items: center; margin-bottom: 0.4rem; font-size: 0.9rem; }
  .past-row td { color: var(--text-muted); }
  .actions { text-align: right; white-space: nowrap; }
  .actions button { margin-left: 0.3rem; }
</style>
