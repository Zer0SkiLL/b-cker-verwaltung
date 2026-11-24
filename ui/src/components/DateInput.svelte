<script lang="ts">
  let {
    label,
    value = $bindable(''),
    error = null,
    id,
  }: { label: string; value?: string; error?: string | null; id: string } = $props();
</script>

<div class="field">
  <label for={id}>{label}</label>
  <input {id} bind:value placeholder="TT.MM.JJJJ" inputmode="numeric" autocomplete="off" size="12" />
  {#if error}<span class="error">{error}</span>{/if}
</div>
