<script lang="ts">
  import type { SettingsTab } from '../lib/router';
  let { tab }: { tab?: SettingsTab } = $props();
</script>

<h1>Einstellungen {tab ?? ""}</h1>
