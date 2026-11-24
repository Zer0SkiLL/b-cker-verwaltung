<script lang="ts">
  import type { Errors, PersonForm } from '../lib/forms';
  import TextField from './TextField.svelte';

  let { form = $bindable(), errors }: { form: PersonForm; errors: Errors } = $props();
</script>

<div class="grid-2">
  <TextField id="p-salutation" label="Anrede" bind:value={form.salutation} />
  <div></div>
  <TextField id="p-first" label="Vorname" bind:value={form.first_name} />
  <TextField id="p-last" label="Nachname *" bind:value={form.last_name} error={errors.last_name} />
  <TextField id="p-street" label="Adresse" bind:value={form.street} />
  <div></div>
  <TextField id="p-zip" label="PLZ" bind:value={form.zip} />
  <TextField id="p-city" label="Ort" bind:value={form.city} />
  <TextField id="p-phone" label="Telefon" bind:value={form.phone} />
  <TextField id="p-mobile" label="Mobiltelefon" bind:value={form.mobile} />
  <TextField id="p-email" label="Mail" type="email" bind:value={form.email} />
  <TextField id="p-comp" label="Honorar / Lohn (Info)" bind:value={form.compensation} />
</div>
<TextField id="p-notes" label="Bemerkungen" multiline bind:value={form.notes} />
