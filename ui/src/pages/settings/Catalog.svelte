<script lang="ts">
  import { readOnly as readOnlyStore } from '../../lib/session';
  import { sessionInfo } from '../../lib/session';
  import type { LookupKind } from '../../lib/types';
  import LabelEditor from './LabelEditor.svelte';
  import LookupEditor from './LookupEditor.svelte';

  const TABS: { key: LookupKind | 'labels'; label: string }[] = [
    { key: 'labels', label: 'Labels' },
    { key: 'Function', label: 'Funktionen' },
    { key: 'OrgType', label: 'Betriebsarten' },
    { key: 'MembershipCategory', label: 'Mitgliederkategorien' },
    { key: 'CantonalAssociation', label: 'Kantonalverbände' },
  ];

  let active = $state<LookupKind | 'labels'>('labels');
  const readOnly = $derived($readOnlyStore);
</script>

<h2>Listen und Labels</h2>
<div class="tabs">
  {#each TABS as tab (tab.key)}
    <button class:active={active === tab.key} onclick={() => (active = tab.key)}>{tab.label}</button>
  {/each}
</div>
{#if active === 'labels'}
  <LabelEditor {readOnly} />
{:else}
  {#key active}<LookupEditor kind={active} {readOnly} />{/key}
{/if}

<style>
  .tabs { display: flex; gap: 0.25rem; margin-bottom: 0.75rem; border-bottom: 1px solid var(--border); }
  .tabs button { border: none; border-radius: 6px 6px 0 0; background: none; }
  .tabs button.active { background: var(--bg-muted); font-weight: 600; }
</style>
