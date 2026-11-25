<script lang="ts">
  import type { Errors, OrganisationForm } from '../lib/forms';
  import type { LookupValue } from '../lib/types';
  import TextField from './TextField.svelte';

  let {
    form = $bindable(),
    errors,
    orgTypes,
    cantonal,
  }: { form: OrganisationForm; errors: Errors; orgTypes: LookupValue[]; cantonal: LookupValue[] } = $props();

  const toId = (v: string) => (v === '' ? null : Number(v));
</script>

<div class="grid-2">
  <TextField id="org-name" label="Name / Firma *" bind:value={form.name} error={errors.name} />
  <TextField id="org-addition" label="Zusatz" bind:value={form.addition} />
  <TextField id="org-street" label="Adresse" bind:value={form.street} />
  <TextField id="org-pobox" label="Postfach" bind:value={form.po_box} />
  <TextField id="org-zip" label="PLZ" bind:value={form.zip} />
  <TextField id="org-city" label="Ort" bind:value={form.city} />
  <TextField id="org-phone" label="Telefon" bind:value={form.phone} />
  <TextField id="org-email" label="Mail" type="email" bind:value={form.email} />
  <TextField id="org-web" label="Website" bind:value={form.website} />
  <TextField id="org-legacy" label="Alt-ID (alte Liste)" bind:value={form.legacy_id} />
  <TextField id="org-hr" label="Handelsregister-Nr" bind:value={form.commercial_register_no} />
  <TextField id="org-kst" label="Kostenstellen-Nr" bind:value={form.cost_center_no} />
  <TextField id="org-branches" label="Filialen" bind:value={form.branches} error={errors.branches} />
  <TextField id="org-apprentices" label="Anzahl Lernende" bind:value={form.apprentice_count} error={errors.apprentice_count} />
  <div class="field">
    <label for="org-type">Betriebsart</label>
    <select id="org-type" value={String(form.org_type_id ?? '')} onchange={(e) => (form.org_type_id = toId(e.currentTarget.value))}>
      <option value="">– keine –</option>
      {#each orgTypes as t (t.id)}<option value={String(t.id)}>{t.name}</option>{/each}
    </select>
  </div>
  <div class="field">
    <label for="org-kt">Kantonalverband</label>
    <select id="org-kt" value={String(form.cantonal_association_id ?? '')} onchange={(e) => (form.cantonal_association_id = toId(e.currentTarget.value))}>
      <option value="">– keiner –</option>
      {#each cantonal as k (k.id)}<option value={String(k.id)}>{k.name}</option>{/each}
    </select>
  </div>
</div>
<TextField id="org-notes" label="Bemerkungen" multiline bind:value={form.notes} />
