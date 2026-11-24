<script lang="ts">
  import type { Filter } from '../lib/types';
  let { filter }: { filter?: Filter } = $props();
</script>

<h1>Adressen</h1>
<p>{filter ? "gefiltert" : ""}</p>
