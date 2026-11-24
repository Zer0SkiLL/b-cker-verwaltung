<script lang="ts">
  import * as api from '../lib/api';
  import { formatDate } from '../lib/dates';
  import { membershipInputFromForm, type Errors } from '../lib/forms';
  import { runWrite } from '../lib/session';
  import type { LookupValue, MemberRef, Membership } from '../lib/types';
  import DateInput from './DateInput.svelte';

  let {
    member,
    categories,
    existing,
    onDone,
  }: {
    member: MemberRef;
    categories: LookupValue[];
    existing: Membership | null;
    onDone: () => void;
  } = $props();

  const initial = () => existing;
  let category = $state(String(initial()?.category_id ?? ''));
  let joined = $state(formatDate(initial()?.joined_on ?? null));
  let left = $state(formatDate(initial()?.left_on ?? null));
  let errors = $state<Errors>({});
  let failure = $state<string | null>(null);

  async function save() {
    const parsed = membershipInputFromForm({
      member,
      category_id: category === '' ? null : Number(category),
      joined_on: joined,
      left_on: left,
    });
    if ('errors' in parsed) {
      errors = parsed.errors;
      return;
    }
    errors = {};
    try {
      await runWrite(() =>
        existing ? api.updateMembership(existing.id, parsed.input) : api.createMembership(parsed.input),
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
    <div class="field">
      <label for="mem-cat">Kategorie</label>
      <select id="mem-cat" bind:value={category}>
        <option value="">– keine –</option>
        {#each categories as c (c.id)}<option value={String(c.id)}>{c.name}</option>{/each}
      </select>
    </div>
    <div></div>
    <DateInput id="mem-joined" label="Eintritt" bind:value={joined} error={errors.joined_on} />
    <DateInput id="mem-left" label="Austritt" bind:value={left} error={errors.left_on} />
  </div>
  <div class="toolbar">
    <button class="primary" onclick={save}>Speichern</button>
    <button onclick={onDone}>Abbrechen</button>
  </div>
</div>
