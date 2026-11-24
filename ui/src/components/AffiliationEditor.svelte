<script lang="ts">
  import * as api from '../lib/api';
  import { formatDate } from '../lib/dates';
  import { affiliationInputFromForm, emptyPersonForm, personInputFromForm, type Errors } from '../lib/forms';
  import { runWrite } from '../lib/session';
  import type { Affiliation, AffiliationInput, LookupValue, OptionDto, PersonInput } from '../lib/types';
  import DateInput from './DateInput.svelte';

  let {
    organisationId,
    personId,
    people,
    organisations,
    functions,
    existing,
    onDone,
  }: {
    organisationId: number | null;
    personId: number | null;
    people: OptionDto[];
    organisations: OptionDto[];
    functions: LookupValue[];
    existing: Affiliation | null;
    onDone: () => void;
  } = $props();

  const NEW_PERSON = 'new';
  // „Neue Person“: Platzhalter-ID 0, damit die Formularprüfung durchläuft; die echte ID setzt saveWithNewPerson.
  const toId = (v: string) => (v === '' ? null : v === NEW_PERSON ? 0 : Number(v));
  const initial = () => existing;
  let person = $state(String(initial()?.person_id ?? ''));
  let organisation = $state(String(initial()?.organisation_id ?? ''));
  let fn = $state(String(initial()?.function_id ?? ''));
  let from = $state(formatDate(initial()?.valid_from ?? null));
  let to = $state(formatDate(initial()?.valid_to ?? null));
  let newLast = $state('');
  // Bereits angelegte neue Person (nach gescheiterter Verknüpfung nicht nochmals anlegen).
  let createdId = $state<number | null>(null);
  let newFirst = $state('');
  let errors = $state<Errors>({});
  let failure = $state<string | null>(null);

  async function save() {
    const parsed = affiliationInputFromForm({
      person_id: personId ?? toId(person),
      organisation_id: organisationId ?? toId(organisation),
      function_id: toId(fn),
      valid_from: from,
      valid_to: to,
    });
    const creating = personId === null && person === NEW_PERSON;
    const newPerson = creating
      ? personInputFromForm({ ...emptyPersonForm(), last_name: newLast, first_name: newFirst })
      : null;
    const personErrors = newPerson && 'errors' in newPerson ? newPerson.errors : {};
    if ('errors' in parsed || Object.keys(personErrors).length > 0) {
      errors = { ...('errors' in parsed ? parsed.errors : {}), ...personErrors };
      return;
    }
    errors = {};
    if (newPerson && 'input' in newPerson) {
      await saveWithNewPerson(newPerson.input, parsed.input);
      return;
    }
    try {
      await runWrite(() =>
        existing ? api.updateAffiliation(existing.id, parsed.input) : api.createAffiliation(parsed.input),
      );
      onDone();
    } catch (err) {
      failure = (err as Error).message;
    }
  }

  /** Legt die Person an und verknüpft sie; meldet klar, wenn nur der erste Schritt geklappt hat. */
  async function saveWithNewPerson(personInput: PersonInput, input: AffiliationInput) {
    if (createdId === null) {
      try {
        createdId = (await runWrite(() => api.createPerson(personInput))).id;
      } catch (err) {
        failure = (err as Error).message;
        return;
      }
    }
    const id = createdId;
    try {
      await runWrite(() => api.createAffiliation({ ...input, person_id: id }));
      onDone();
    } catch (err) {
      failure = `Person wurde angelegt, die Verknüpfung aber nicht: ${(err as Error).message}`;
    }
  }
</script>

<div class="card editor">
  {#if failure}<div class="form-error">{failure}</div>{/if}
  <div class="grid-2">
    {#if personId === null}
      <div class="field">
        <label for="aff-person">Person</label>
        <select id="aff-person" bind:value={person}>
          <option value="">– wählen –</option>
          {#each people as p (p.id)}<option value={String(p.id)}>{p.name}</option>{/each}
          {#if !existing}<option value={NEW_PERSON}>+ Neue Person anlegen…</option>{/if}
        </select>
        {#if errors.person_id}<span class="error">{errors.person_id}</span>{/if}
      </div>
      {#if person === NEW_PERSON}
        <div class="field">
          <label for="aff-new-last">Nachname *</label>
          <input id="aff-new-last" bind:value={newLast} />
          {#if errors.last_name}<span class="error">{errors.last_name}</span>{/if}
        </div>
        <div class="field">
          <label for="aff-new-first">Vorname</label>
          <input id="aff-new-first" bind:value={newFirst} />
        </div>
      {/if}
    {/if}
    {#if organisationId === null}
      <div class="field">
        <label for="aff-org">Betrieb</label>
        <select id="aff-org" bind:value={organisation}>
          <option value="">– wählen –</option>
          {#each organisations as o (o.id)}<option value={String(o.id)}>{o.name}</option>{/each}
        </select>
        {#if errors.organisation_id}<span class="error">{errors.organisation_id}</span>{/if}
      </div>
    {/if}
    <div class="field">
      <label for="aff-fn">Funktion</label>
      <select id="aff-fn" bind:value={fn}>
        <option value="">– wählen –</option>
        {#each functions as f (f.id)}<option value={String(f.id)}>{f.name}</option>{/each}
      </select>
      {#if errors.function_id}<span class="error">{errors.function_id}</span>{/if}
      {#if functions.length === 0}
        <span class="muted">Funktionen unter Einstellungen › Listen anlegen.</span>
      {/if}
    </div>
    <div></div>
    <DateInput id="aff-from" label="Von" bind:value={from} error={errors.valid_from} />
    <DateInput id="aff-to" label="Bis" bind:value={to} error={errors.valid_to} />
  </div>
  <div class="toolbar">
    <button class="primary" onclick={save}>Speichern</button>
    <button onclick={onDone}>Abbrechen</button>
  </div>
</div>
