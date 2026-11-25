<script lang="ts">
  import { openHelpForRoute } from '../lib/help';
  import { route } from '../lib/router';
</script>

<button class="help-button" title="Hilfe zu dieser Seite (F1)" aria-label="Hilfe" onclick={() => openHelpForRoute($route)}>?</button>

<style>
  .help-button { width: 1.9rem; height: 1.9rem; padding: 0; border-radius: 50%; font-weight: 700; flex: none; }
</style>
