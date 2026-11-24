<script lang="ts">
  let {
    label,
    value = $bindable(''),
    error = null,
    id,
    multiline = false,
    type = 'text',
  }: {
    label: string;
    value?: string;
    error?: string | null;
    id: string;
    multiline?: boolean;
    type?: string;
  } = $props();
</script>

<div class="field">
  <label for={id}>{label}</label>
  {#if multiline}
    <textarea {id} bind:value rows="3"></textarea>
  {:else}
    <input {id} {type} bind:value />
  {/if}
  {#if error}<span class="error">{error}</span>{/if}
</div>
