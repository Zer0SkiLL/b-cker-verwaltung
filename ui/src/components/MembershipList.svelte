<script lang="ts">
  import * as api from '../lib/api';
  import { formatDate, todayIso } from '../lib/dates';
  import { confirmAction } from '../lib/desktop';
  import { runWrite } from '../lib/session';
  import type { LookupValue, MemberRef, Membership } from '../lib/types';
  import MembershipEditor from './MembershipEditor.svelte';

  let {
    member,
    items,
    categories,
    readOnly,
  }: { member: MemberRef; items: Membership[]; categories: LookupValue[]; readOnly: boolean } = $props();

  let editing = $state<Membership | 'new' | null>(null);
  let error = $state<string | null>(null);

  const categoryName = (id: number | null) => categories.find((c) => c.id === id)?.name ?? '–';
  const isActive = (m: Membership) => m.left_on === null || m.left_on > todayIso();

  async function remove(m: Membership) {
    if (!(await confirmAction('Mitgliedschaft löschen?'))) return;
    try {
      await runWrite(() => api.deleteMembership(m.id));
    } catch (err) {
      error = (err as Error).message;
    }
  }
</script>

<table>
  <tbody>
    {#each items as m (m.id)}
      <tr>
        <td>{categoryName(m.category_id)}</td>
        <td>Eintritt {formatDate(m.joined_on) || '–'}</td>
        <td>Austritt {formatDate(m.left_on) || '–'}</td>
        <td>{isActive(m) ? 'aktiv' : 'ausgetreten'}</td>
        <td class="actions">
          {#if !readOnly}
            <button onclick={() => (editing = m)}>Bearbeiten</button>
            <button class="danger" onclick={() => remove(m)}>Löschen</button>
          {/if}
        </td>
      </tr>
    {:else}
      <tr><td class="muted">Keine Mitgliedschaft erfasst</td></tr>
    {/each}
  </tbody>
</table>
{#if error}<div class="form-error">{error}</div>{/if}
{#if editing}
  {#key editing}
    <MembershipEditor {member} {categories} existing={editing === 'new' ? null : editing} onDone={() => (editing = null)} />
  {/key}
{:else if !readOnly}
  <button onclick={() => (editing = 'new')}>Mitgliedschaft erfassen</button>
{/if}

<style>
  .actions { text-align: right; white-space: nowrap; }
  .actions button { margin-left: 0.3rem; }
</style>
