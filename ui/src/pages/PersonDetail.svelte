<script lang="ts">
  import { readOnly as readOnlyStore } from '../lib/session';
  import { onDestroy, untrack } from 'svelte';
  import AffiliationList from '../components/AffiliationList.svelte';
  import LabelAssignments from '../components/LabelAssignments.svelte';
  import MembershipList from '../components/MembershipList.svelte';
  import PersonFields from '../components/PersonFields.svelte';
  import * as api from '../lib/api';
  import { confirmAction } from '../lib/desktop';
  import { emptyPersonForm, personFormFrom, personInputFromForm, type Errors } from '../lib/forms';
  import { back, navigate, replace, setLeaveGuard } from '../lib/router';
  import { dataVersion, runWrite, sessionInfo } from '../lib/session';
  import type { FilterOptions, OptionDto, PersonDetail } from '../lib/types';

  let { id }: { id: number | null } = $props();

  let detail = $state<PersonDetail | null>(null);
  let form = $state(emptyPersonForm());
  let errors = $state<Errors>({});
  let failure = $state<string | null>(null);
  let saved = $state(false);
  let showPast = $state(false);
  let options = $state<FilterOptions | null>(null);
  let organisations = $state<OptionDto[]>([]);
  let loaded = '';
  // Stand des Formulars nach Laden/Speichern; Abweichung = ungespeicherte Änderungen.
  let baseline = $state(JSON.stringify(emptyPersonForm()));
  const dirty = $derived(JSON.stringify(form) !== baseline);

  $effect(() => {
    setLeaveGuard(
      dirty
        ? () => confirmAction('Es gibt ungespeicherte Änderungen. Seite trotzdem verlassen?', 'Verlassen')
        : null,
    );
  });
  onDestroy(() => setLeaveGuard(null));

  const readOnly = $derived($readOnlyStore);
  const title = $derived([form.first_name, form.last_name].filter((v) => v.trim()).join(' '));

  async function load(current: number | null, past: boolean) {
    try {
      failure = null;
      options = await api.filterOptions();
      organisations = await api.listOrganisations();
      if (current !== null) {
        detail = await api.personDetail(current, past);
        // Formular nur zurücksetzen, wenn sich die Stammdaten geändert haben (ungespeicherte Eingaben behalten).
        const snapshot = JSON.stringify(detail.person);
        if (snapshot !== loaded) {
          loaded = snapshot;
          form = personFormFrom(detail.person);
          baseline = JSON.stringify(form);
        }
      }
    } catch (err) {
      failure = (err as Error).message;
    }
  }

  $effect(() => {
    const past = showPast;
    void $dataVersion;
    untrack(() => load(id, past));
  });

  async function save() {
    const parsed = personInputFromForm(form);
    if ('errors' in parsed) {
      errors = parsed.errors;
      return;
    }
    errors = {};
    try {
      const person = await runWrite(() =>
        id === null ? api.createPerson(parsed.input) : api.updatePerson(id, parsed.input),
      );
      failure = null;
      saved = true;
      baseline = JSON.stringify(form);
      if (id === null) replace({ page: 'person', id: person.id });
    } catch (err) {
      failure = (err as Error).message;
    }
  }

  async function remove() {
    if (id === null || !(await confirmAction(`${title || 'Person'} mit allen Verknüpfungen löschen?`, 'Löschen'))) return;
    try {
      await runWrite(() => api.deletePerson(id));
      setLeaveGuard(null);
      back();
    } catch (err) {
      failure = (err as Error).message;
    }
  }
</script>

<div class="toolbar"><button class="link" onclick={back}>← Zurück</button></div>
<h1>{id === null ? 'Neue Person' : title || 'Person'}</h1>
{#if failure}<div class="form-error">{failure}</div>{/if}

<section class="card">
  <PersonFields bind:form {errors} />
  <div class="toolbar">
    <button class="primary" disabled={readOnly} onclick={save}>Speichern</button>
    {#if id !== null}<button class="danger" disabled={readOnly} onclick={remove}>Person löschen</button>{/if}
    {#if dirty}<span class="muted">Ungespeicherte Änderungen</span>{:else if saved}<span class="muted">Gespeichert.</span>{/if}
  </div>
</section>

{#if id !== null && detail && options}
  <h2>Betriebe und Funktionen</h2>
  <AffiliationList mode="person" ownerId={id} items={detail.organisations} people={[]} {organisations} functions={options.functions} bind:showPast {readOnly} />
  <h2>Mitgliedschaft (Einzelmitglied)</h2>
  <MembershipList member={{ Person: id }} items={detail.memberships} categories={options.categories} {readOnly} />
  <h2>Labels</h2>
  <LabelAssignments target={{ Person: id }} kind="Person" assigned={detail.labels} labelGroups={options.labelGroups} {readOnly} />
  <h2>Teams</h2>
  {#each detail.teams as t (t.team.id)}
    <span class="chip">{t.team.name}{t.role ? ` – ${t.role}` : ''}</span>
  {:else}
    <span class="muted">In keinem Team</span>
  {/each}
  <div><button class="link" onclick={() => navigate({ page: 'teams' })}>Teams bearbeiten</button></div>
{/if}
