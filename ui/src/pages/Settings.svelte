<script lang="ts">
  import { replace, type SettingsTab } from '../lib/router';
  import Backup from './settings/Backup.svelte';
  import Catalog from './settings/Catalog.svelte';
  import FullExport from './settings/FullExport.svelte';
  import ImportWizard from './settings/ImportWizard.svelte';
  import About from './settings/About.svelte';

  let { tab = 'catalog' }: { tab?: SettingsTab } = $props();

  const TABS: { key: SettingsTab; label: string }[] = [
    { key: 'catalog', label: 'Listen und Labels' },
    { key: 'backup', label: 'Datensicherung' },
    { key: 'import', label: 'Excel-Import' },
    { key: 'export', label: 'Gesamtexport' },
    { key: 'about', label: 'Über das Programm' },
  ];
</script>

<h1>Einstellungen</h1>
<div class="tabs">
  {#each TABS as t (t.key)}
    <button class:active={tab === t.key} onclick={() => replace({ page: 'settings', tab: t.key })}>{t.label}</button>
  {/each}
</div>

{#if tab === 'catalog'}
  <Catalog />
{:else if tab === 'backup'}
  <Backup />
{:else if tab === 'import'}
  <ImportWizard />
{:else if tab === 'export'}
  <FullExport />
{:else}
  <About />
{/if}

<style>
  .tabs { display: flex; gap: 0.5rem; margin-bottom: 1rem; }
  .tabs button.active { background: var(--accent); border-color: var(--accent); color: var(--accent-text); }
</style>
