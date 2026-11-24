<script lang="ts">
  import NoticeBar from './components/NoticeBar.svelte';
  import Sidebar from './components/Sidebar.svelte';
  import Addresses from './pages/Addresses.svelte';
  import Groups from './pages/Groups.svelte';
  import OrganisationDetail from './pages/OrganisationDetail.svelte';
  import PersonDetail from './pages/PersonDetail.svelte';
  import Settings from './pages/Settings.svelte';
  import Start from './pages/Start.svelte';
  import Teams from './pages/Teams.svelte';
  import { onMount } from 'svelte';
  import { route } from './lib/router';
  import { checkForUpdates } from './lib/updates';

  // Im Hintergrund nach Updates suchen; ohne Verbindung bleibt es still.
  onMount(() => {
    checkForUpdates({ silent: true });
  });
</script>

{#if $route.page === 'start'}
  <Start />
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

<style>
  .layout { display: grid; grid-template-columns: 210px 1fr; height: 100vh; }
  aside { border-right: 1px solid var(--border); }
  main { overflow: auto; padding: 1rem 1.5rem; }
</style>
