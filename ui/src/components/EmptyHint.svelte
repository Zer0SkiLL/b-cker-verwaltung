<script lang="ts">
  // Hinweis für leere Listen: was als Nächstes zu tun ist, optional mit Link ins Handbuch.
  import { openHelp } from '../lib/help';

  let { text, help }: { text: string; help?: { label: string; slug: string } } = $props();
</script>

<p class="muted empty-hint">
  {text}
  {#if help}<button class="link" onclick={() => openHelp(help.slug)}>{help.label}</button>{/if}
</p>
