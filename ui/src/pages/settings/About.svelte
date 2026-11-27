<script lang="ts">
  import { onDestroy, onMount } from 'svelte';
  import changelog from '../../../../CHANGELOG.md?raw';
  import { openWebsite } from '../../lib/desktop';
  import { renderMarkdown } from '../../lib/markdown';
  import { checkForUpdates, updateState } from '../../lib/updates';
  import { bakerMode, countVersionClick, setBakerMode } from '../../lib/osternest/state';
  import DiagnoseSwitch from './DiagnoseSwitch.svelte';

  const REPO = 'https://github.com/Zer0SkiLL/b-cker-verwaltung';
  let version = $state('');
  // Osternest: 7 Klicks auf die Version schalten den Bäckermeister-Modus um (bewusst versteckt).
  let clicks = 0;
  let bakerNote = $state<string | null>(null);
  let noteTimer: ReturnType<typeof setTimeout> | undefined;

  function onVersionClick() {
    const result = countVersionClick(clicks);
    clicks = result.count;
    if (!result.toggled) return;
    const on = !$bakerMode;
    setBakerMode(on);
    bakerNote = on ? 'Sie sind jetzt Bäckermeister.' : 'Bäckermeister-Modus aus. Zurück an den Ofen.';
    clearTimeout(noteTimer);
    noteTimer = setTimeout(() => (bakerNote = null), 4000);
  }

  onDestroy(() => clearTimeout(noteTimer));

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
  <div>
    <strong>Mitgliederverwaltung</strong> –
    <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions (bewusst versteckt) -->
    <span onclick={onVersionClick}>Version {version}</span>
    {#if bakerNote}<span class="muted baker-note">{bakerNote}</span>{/if}
  </div>
  <div class="muted">
    Quellcode und alle Versionen:
    <button class="link" onclick={() => openWebsite(REPO).catch(() => undefined)}>{REPO.replace('https://', '')}</button>
  </div>
  <div class="toolbar actions">
    <button class="primary" disabled={$updateState.kind === 'checking'} onclick={() => checkForUpdates({ silent: false })}>
      Nach Updates suchen
    </button>
    {#if status}<span class="muted">{status}</span>{/if}
  </div>
</div>
<DiagnoseSwitch />

<h2>Was ist neu?</h2>
<div class="changelog">{@html renderMarkdown(changelog)}</div>

<style>
  .baker-note { margin-left: 0.6rem; }
  .actions { margin-top: 0.75rem; }
  .changelog :global(h1) { display: none; }
  .changelog :global(h2) { font-size: 1.05rem; margin-top: 1.25rem; }
  .changelog :global(h3) { font-size: 0.95rem; margin: 0.75rem 0 0.25rem; }
</style>
