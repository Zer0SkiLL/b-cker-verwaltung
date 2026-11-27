<script lang="ts">
  import BreadRain from './components/BreadRain.svelte';
  import HelpPanel from './components/HelpPanel.svelte';
  import NoticeBar from './components/NoticeBar.svelte';
  import ReportDialog from './components/ReportDialog.svelte';
  import Sidebar from './components/Sidebar.svelte';
  import Addresses from './pages/Addresses.svelte';
  import Groups from './pages/Groups.svelte';
  import OrganisationDetail from './pages/OrganisationDetail.svelte';
  import PersonDetail from './pages/PersonDetail.svelte';
  import Settings from './pages/Settings.svelte';
  import Start from './pages/Start.svelte';
  import Teams from './pages/Teams.svelte';
  import { onDestroy, onMount } from 'svelte';
  import { get } from 'svelte/store';
  import * as api from './lib/api';
  import { startAutoBackup } from './lib/autosave';
  import { refreshSession, sessionInfo } from './lib/session';
  import { handleHelpKey, helpState } from './lib/help';
  import { clampWidth, helpWidth } from './lib/helpWidth';
  import { route } from './lib/router';
  import { checkForUpdates } from './lib/updates';
  import { checkOpenedFile, startEarlyWarning } from './lib/earlyWarning';

  // Im Hintergrund nach Updates suchen; ohne Verbindung bleibt es still.
  // Sicherung während der Arbeit: nur mit schreibend geöffneter Datei, Intervall aus den Einstellungen.
  const stopAutoBackup = startAutoBackup({
    getInterval: async () => {
      const info = get(sessionInfo);
      return info && !info.readOnly ? (await api.getRetention()).intervalMinutes : 0;
    },
    backup: () => api.backupIfChanged(),
    refresh: refreshSession,
  });
  onDestroy(stopAutoBackup);

  onMount(() => {
    checkForUpdates({ silent: true });
    void startEarlyWarning();
  });

  // Einmal pro geöffneter Datei im Hintergrund: Selbsttest, danach Diagnose-Ordner aktualisieren.
  $effect(() => {
    const path = $sessionInfo?.path;
    if (path) void checkOpenedFile(path);
  });

  // Die Hilfe nimmt nie so viel Platz, dass die Seite daneben unbrauchbar wird (auch bei kleinem Fenster).
  let viewport = $state(window.innerWidth);
  const columns = $derived($helpState.open ? `minmax(0, 1fr) ${clampWidth($helpWidth, viewport)}px` : undefined);

  // F1 öffnet die Hilfe zur aktuellen Seite (auch aus Eingabefeldern), Esc schliesst sie.
  function onKey(event: KeyboardEvent) {
    if (handleHelpKey(event, $route)) event.preventDefault();
  }
</script>

<svelte:window onkeydown={onKey} bind:innerWidth={viewport} />

<div class="shell" style:grid-template-columns={columns}>
{#if $route.page === 'start'}
  <main class="start-page"><Start /></main>
{:else}
  <div class="layout">
    <aside><Sidebar /></aside>
    <main>
      <NoticeBar />
      {#if $route.page === 'addresses'}
        <Addresses filter={$route.filter} />
      {:else if $route.page === 'organisation'}
        {#key $route.id}<OrganisationDetail id={$route.id} />{/key}
      {:else if $route.page === 'person'}
        {#key $route.id}<PersonDetail id={$route.id} />{/key}
      {:else if $route.page === 'teams'}
        <Teams />
      {:else if $route.page === 'groups'}
        <Groups />
      {:else if $route.page === 'settings'}
        <Settings tab={$route.tab} />
      {/if}
    </main>
  </div>
{/if}
{#if $helpState.open}<HelpPanel />{/if}
</div>
<ReportDialog />
<BreadRain />

<style>
  .shell { display: grid; grid-template-columns: 1fr; height: 100vh; }
  .start-page { overflow: auto; min-width: 0; }
  .layout { display: grid; grid-template-columns: 210px 1fr; height: 100vh; min-width: 0; }
  aside { border-right: 1px solid var(--border); }
  main { overflow: auto; padding: 1rem 1.5rem; }
</style>
