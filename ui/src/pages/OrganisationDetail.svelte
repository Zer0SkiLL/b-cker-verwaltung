<script lang="ts">
  import { untrack } from 'svelte';
  import AffiliationList from '../components/AffiliationList.svelte';
  import LabelAssignments from '../components/LabelAssignments.svelte';
  import MembershipList from '../components/MembershipList.svelte';
  import OrganisationFields from '../components/OrganisationFields.svelte';
  import * as api from '../lib/api';
  import { confirmAction } from '../lib/desktop';
  import { emptyOrganisationForm, organisationFormFrom, organisationInputFromForm, type Errors } from '../lib/forms';
  import { back, replace } from '../lib/router';
  import { dataVersion, runWrite, sessionInfo } from '../lib/session';
  import type { FilterOptions, OptionDto, OrganisationDetail } from '../lib/types';

  let { id }: { id: number | null } = $props();

  let detail = $state<OrganisationDetail | null>(null);
  let form = $state(emptyOrganisationForm());
  let errors = $state<Errors>({});
  let failure = $state<string | null>(null);
  let saved = $state(false);
  let showPast = $state(false);
  let options = $state<FilterOptions | null>(null);
  let people = $state<OptionDto[]>([]);
  let loaded = '';

  const readOnly = $derived($sessionInfo?.readOnly ?? false);

  async function load(current: number | null, past: boolean) {
    try {
      options = await api.filterOptions();
      people = await api.listPeople();
      if (current !== null) {
        detail = await api.organisationDetail(current, past);
        // Formular nur zurücksetzen, wenn sich die Stammdaten geändert haben (ungespeicherte Eingaben behalten).
        const snapshot = JSON.stringify(detail.organisation);
        if (snapshot !== loaded) {
          loaded = snapshot;
          form = organisationFormFrom(detail.organisation);
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
    const parsed = organisationInputFromForm(form);
    if ('errors' in parsed) {
      errors = parsed.errors;
      return;
    }
    errors = {};
    try {
      const org = await runWrite(() =>
        id === null ? api.createOrganisation(parsed.input) : api.updateOrganisation(id, parsed.input),
      );
      failure = null;
      saved = true;
      if (id === null) replace({ page: 'organisation', id: org.id });
    } catch (err) {
      failure = (err as Error).message;
    }
  }

  async function remove() {
    if (id === null || !(await confirmAction(`Betrieb „${form.name}“ mit allen Verknüpfungen löschen?`, 'Löschen'))) return;
    try {
      await runWrite(() => api.deleteOrganisation(id));
      back();
    } catch (err) {
      failure = (err as Error).message;
    }
  }
</script>

<div class="toolbar"><button class="link" onclick={back}>← Zurück</button></div>
<h1>{id === null ? 'Neuer Betrieb' : form.name || 'Betrieb'}</h1>
{#if failure}<div class="form-error">{failure}</div>{/if}

<section class="card">
  <OrganisationFields bind:form {errors} orgTypes={options?.orgTypes ?? []} cantonal={options?.cantonal ?? []} />
  <div class="toolbar">
    <button class="primary" disabled={readOnly} onclick={save}>Speichern</button>
    {#if id !== null}<button class="danger" disabled={readOnly} onclick={remove}>Betrieb löschen</button>{/if}
    {#if saved}<span class="muted">Gespeichert.</span>{/if}
  </div>
</section>

{#if id !== null && detail && options}
  <h2>Personen</h2>
  <AffiliationList mode="organisation" ownerId={id} items={detail.people} {people} organisations={[]} functions={options.functions} bind:showPast {readOnly} />
  <h2>Mitgliedschaft</h2>
  <MembershipList member={{ Organisation: id }} items={detail.memberships} categories={options.categories} {readOnly} />
  <h2>Labels</h2>
  <LabelAssignments target={{ Organisation: id }} kind="Organisation" assigned={detail.labels} labelGroups={options.labelGroups} {readOnly} />
{/if}
