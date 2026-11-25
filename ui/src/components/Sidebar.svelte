<script lang="ts">
  import { reportOpen } from '../lib/diagnostics';
  import { helpState, toggleHelp } from '../lib/help';
  import { navigate, route, type Route } from '../lib/router';

  const items: { label: string; target: Route; pages: string[] }[] = [
    { label: 'Adressen', target: { page: 'addresses' }, pages: ['addresses', 'organisation', 'person'] },
    { label: 'Teams', target: { page: 'teams' }, pages: ['teams'] },
    { label: 'Gruppen', target: { page: 'groups' }, pages: ['groups'] },
  ];
</script>

<nav>
  <div class="brand">Bäckervereinigung</div>
  {#each items as item (item.label)}
    <button class:active={item.pages.includes($route.page)} onclick={() => navigate(item.target)}>
      {item.label}
    </button>
  {/each}
  <div class="spacer"></div>
  <button class:active={$helpState.open} onclick={toggleHelp} title="Handbuch: Abläufe und Erklärungen zu allen Seiten">Hilfe</button>
  <button onclick={() => reportOpen.set(true)} title="Bericht für den Entwickler erstellen, wenn etwas nicht funktioniert">Problem melden</button>
  <button class:active={$route.page === 'settings'} onclick={() => navigate({ page: 'settings' })}>
    Einstellungen
  </button>
</nav>

<style>
  nav { display: flex; flex-direction: column; gap: 0.25rem; padding: 1rem 0.75rem; background: var(--bg-sidebar); height: 100%; }
  .brand { font-weight: 700; margin-bottom: 1rem; padding: 0 0.5rem; }
  button { text-align: left; border: none; background: none; padding: 0.45rem 0.6rem; }
  button.active { background: var(--bg); font-weight: 600; }
  .spacer { flex: 1; }
</style>
