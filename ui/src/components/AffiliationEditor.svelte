<script lang="ts">
  import * as api from '../lib/api';
  import { formatDate } from '../lib/dates';
  import { affiliationInputFromForm, type Errors } from '../lib/forms';
  import { runWrite } from '../lib/session';
  import type { Affiliation, LookupValue, OptionDto } from '../lib/types';
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

  const toId = (v: string) => (v === '' ? null : Number(v));
  const initial = () => existing;
  let person = $state(String(initial()?.person_id ?? ''));
  let organisation = $state(String(initial()?.organisation_id ?? ''));
  let fn = $state(String(initial()?.function_id ?? ''));
  let from = $state(formatDate(initial()?.valid_from ?? null));
  let to = $state(formatDate(initial()?.valid_to ?? null));
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
    if ('errors' in parsed) {
      errors = parsed.errors;
      return;
    }
    errors = {};
    try {
      await runWrite(() =>
        existing ? api.updateAffiliation(existing.id, parsed.input) : api.createAffiliation(parsed.input),
      );
      onDone();
    } catch (err) {
      failure = (err as Error).message;
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
        </select>
        {#if errors.person_id}<span class="error">{errors.person_id}</span>{/if}
      </div>
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
