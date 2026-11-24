<script lang="ts">
  import { onMount } from 'svelte';
  import changelog from '../../../../CHANGELOG.md?raw';
  import { renderMarkdown } from '../../lib/markdown';
  import { checkForUpdates, updateState } from '../../lib/updates';

  const REPO = 'https://github.com/Zer0SkiLL/b-cker-verwaltung';
  let version = $state('');

  onMount(async () => {
    try {
      version = await (await import('@tauri-apps/api/app')).getVersion();
    } catch {
      version = 'unbekannt';
    }
  });

  const status = $derived.by(() => {
    const s = $updateState;
    if (s.kind === 'checking') return 'Suche nach Updates …';
    if (s.kind === 'none') return 'Das Programm ist auf dem neuesten Stand.';
    if (s.kind === 'available') return `Version ${s.version} ist verfügbar – siehe gelbe Leiste oben.`;
    if (s.kind === 'failed') return s.message;
    return '';
  });
</script>

<h2>Über das Programm</h2>
<div class="card">
  <div><strong>Mitgliederverwaltung</strong> – Version {version}</div>
  <div class="muted">Quellcode und alle Versionen: <span class="mono">{REPO}</span></div>
  <div class="toolbar actions">
    <button class="primary" disabled={$updateState.kind === 'checking'} onclick={() => checkForUpdates({ silent: false })}>
      Nach Updates suchen
    </button>
    {#if status}<span class="muted">{status}</span>{/if}
  </div>
</div>

<h2>Was ist neu?</h2>
<div class="changelog">{@html renderMarkdown(changelog)}</div>

<style>
  .actions { margin-top: 0.75rem; }
  .mono { font-family: ui-monospace, monospace; }
  .changelog :global(h1) { display: none; }
  .changelog :global(h2) { font-size: 1.05rem; margin-top: 1.25rem; }
  .changelog :global(h3) { font-size: 0.95rem; margin: 0.75rem 0 0.25rem; }
</style>
