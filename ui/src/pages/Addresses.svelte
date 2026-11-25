<script lang="ts">
  import HelpButton from '../components/HelpButton.svelte';
  import { onDestroy, onMount, untrack } from 'svelte';
  import MultiSelect from '../components/MultiSelect.svelte';
  import * as api from '../lib/api';
  import { contactActions } from '../lib/contact';
  import { copyText, openMail, pickSavePath, todayStamp } from '../lib/desktop';
  import { activeFilterCount, emptyFilter, setKinds, setMembership, toggleId, withText } from '../lib/filters';
  import { navigate, replace } from '../lib/router';
  import { dataVersion, readOnly, runWrite } from '../lib/session';
  import type { EntryKind, ExportFormat, Filter, FilterIdKey, FilterOptions, MembershipStatus, SearchResult } from '../lib/types';

  let { filter: initial }: { filter?: Filter } = $props();

  // Startwert bewusst einmalig aus der Route übernehmen; danach führt die Seite den Filter selbst.
  const start = untrack(() => initial ?? emptyFilter());
  let filter = $state<Filter>(start);
  let text = $state(start.text ?? '');
  let result = $state<SearchResult | null>(null);
  let options = $state<FilterOptions | null>(null);
  let error = $state<string | null>(null);
  let info = $state<string | null>(null);
  let groupName = $state<string | null>(null);
  let timer: ReturnType<typeof setTimeout> | undefined;

  const actions = $derived(result ? contactActions(result) : null);
  const kindValue = $derived(filter.kinds.length === 1 ? filter.kinds[0] : 'all');

  async function load(current: Filter) {
    try {
      result = await api.searchAddresses(current);
      error = null;
    } catch (err) {
      error = (err as Error).message;
    }
  }

  function update(next: Filter) {
    filter = next;
    replace({ page: 'addresses', filter: next });
  }

  function onText() {
    clearTimeout(timer);
    timer = setTimeout(() => update(withText(filter, text)), 300);
  }

  const toggle = (key: FilterIdKey) => (id: number) => update(toggleId(filter, key, id));
  const onKind = (value: string) => update(setKinds(filter, value === 'all' ? [] : [value as EntryKind]));
  const onMembership = (value: string) => update(setMembership(filter, value as MembershipStatus));

  async function copyMails() {
    if (!result) return;
    try {
      await copyText(result.joinedEmails);
      info = `${result.emailCount} Adressen kopiert – in Outlook unter „Bcc“ einfügen.`;
    } catch (err) {
      error = `Kopieren nicht möglich: ${(err as Error).message ?? err}`;
    }
  }

  async function openMailProgram() {
    if (!result?.mailto) return;
    try {
      await openMail(result.mailto);
    } catch (err) {
      error = `Mailprogramm konnte nicht geöffnet werden: ${String(err)}. Bitte „Mails kopieren“ verwenden.`;
    }
  }

  async function exportList(format: ExportFormat) {
    const path = await pickSavePath(`Adressen-${todayStamp()}.${format}`, format);
    if (!path) return;
    try {
      await api.exportAddresses(filter, path, format);
      info = `Gespeichert: ${path}`;
    } catch (err) {
      error = (err as Error).message;
    }
  }

  async function saveGroup() {
    const name = (groupName ?? '').trim();
    if (!name) return;
    try {
      await runWrite(() => api.saveGroup(name, filter));
      info = `Gruppe „${name}“ gespeichert.`;
      groupName = null;
    } catch (err) {
      error = (err as Error).message;
    }
  }

  function reset() {
    text = '';
    update(emptyFilter());
  }

  onDestroy(() => clearTimeout(timer));

  onMount(async () => {
    options = await api.filterOptions().catch(() => null);
  });

  $effect(() => {
    const current = filter;
    void $dataVersion;
    untrack(() => load(current));
  });
</script>

<div class="head">
  <div class="title-row"><h1>Adressen</h1><HelpButton /></div>
  <div class="toolbar">
    <button disabled={$readOnly} onclick={() => navigate({ page: 'organisation', id: null })}>Neuer Betrieb</button>
    <button disabled={$readOnly} onclick={() => navigate({ page: 'person', id: null })}>Neue Person</button>
  </div>
</div>

<div class="toolbar filters">
  <input class="search" type="search" placeholder="Suche Name, Ort, Mail…" bind:value={text} oninput={onText} />
  <select value={kindValue} onchange={(e) => onKind(e.currentTarget.value)} aria-label="Typ">
    <option value="all">Alle</option>
    <option value="Organisation">Betriebe</option>
    <option value="Person">Personen</option>
  </select>
  <select value={filter.membership} onchange={(e) => onMembership(e.currentTarget.value)} aria-label="Mitgliedschaft">
    <option value="Any">Mitgliedschaft: alle</option>
    <option value="Active">Aktive Mitglieder</option>
    <option value="Exited">Ausgetretene</option>
  </select>
  {#if options}
    <MultiSelect
      label="Labels"
      options={options.labelGroups.flatMap((g) => g.labels.map((l) => ({ id: l.id, name: l.name, group: g.group.name })))}
      selected={filter.label_ids}
      onToggle={toggle('label_ids')}
    />
    <MultiSelect label="Funktion" options={options.functions} selected={filter.function_ids} onToggle={toggle('function_ids')} />
    <MultiSelect label="Teams" options={options.teams} selected={filter.team_ids} onToggle={toggle('team_ids')} />
    <MultiSelect label="Kategorie" options={options.categories} selected={filter.category_ids} onToggle={toggle('category_ids')} />
    <MultiSelect label="Betriebsart" options={options.orgTypes} selected={filter.org_type_ids} onToggle={toggle('org_type_ids')} />
    <MultiSelect label="Kt. Verband" options={options.cantonal} selected={filter.cantonal_ids} onToggle={toggle('cantonal_ids')} />
  {/if}
  {#if activeFilterCount(filter) > 0}
    <button class="link" onclick={reset}>Filter zurücksetzen</button>
  {/if}
</div>

{#if error}<div class="form-error">{error}</div>{/if}

{#if result}
  <div class="table-wrap">
    <table>
      <thead>
        <tr><th>Typ</th><th>Name</th><th>Betrieb</th><th>Ort</th><th>Mail</th><th>Telefon</th></tr>
      </thead>
      <tbody>
        {#each result.rows as row (row.kind + row.id)}
          <tr class="clickable" onclick={() => navigate({ page: row.kind === 'Person' ? 'person' : 'organisation', id: row.id })}>
            <td>{row.kind === 'Person' ? 'Person' : 'Betrieb'}</td>
            <td>{row.display_name}</td>
            <td>{row.organisation ?? ''}</td>
            <td>{[row.zip, row.city].filter(Boolean).join(' ')}</td>
            <td>{#if row.email}{row.email}{:else}<span class="error">fehlt</span>{/if}</td>
            <td>{row.phone ?? row.mobile ?? ''}</td>
          </tr>
        {:else}
          <tr><td colspan="6" class="muted">Keine Einträge gefunden.</td></tr>
        {/each}
      </tbody>
    </table>
  </div>

  <div class="footer">
    <div>
      {result.rows.length} Einträge · {result.missingEmail.length} ohne Mail
      {#if result.missingEmail.length > 0}
        <details>
          <summary class="muted">Ohne Mail anzeigen</summary>
          <ul>{#each result.missingEmail as m (m.kind + m.id)}<li>{m.display_name}</li>{/each}</ul>
        </details>
      {/if}
    </div>
    <div class="toolbar">
      <button class="primary" disabled={!actions?.canCopy} onclick={copyMails}>Mails kopieren</button>
      <button disabled={!actions?.canMailto} onclick={openMailProgram}>Mail öffnen</button>
      <button disabled={result.rows.length === 0} onclick={() => exportList('xlsx')}>Excel</button>
      <button disabled={result.rows.length === 0} onclick={() => exportList('csv')}>CSV</button>
      {#if groupName === null}
        <button disabled={$readOnly} onclick={() => (groupName = '')}>Als Gruppe speichern</button>
      {:else}
        <input placeholder="Name der Gruppe" bind:value={groupName} onkeydown={(e) => e.key === 'Enter' && saveGroup()} />
        <button class="primary" onclick={saveGroup}>Speichern</button>
        <button onclick={() => (groupName = null)}>Abbrechen</button>
      {/if}
    </div>
  </div>
  {#if actions?.hint}<p class="muted">{actions.hint}</p>{/if}
  {#if info}<p class="muted">{info}</p>{/if}
{/if}

<style>
  .head { display: flex; justify-content: space-between; align-items: center; }
  .filters { margin-bottom: 0.75rem; }
  .search { min-width: 260px; }
  .table-wrap { max-height: calc(100vh - 290px); overflow: auto; }
  .footer { display: flex; justify-content: space-between; align-items: flex-start; gap: 1rem; padding-top: 0.75rem; border-top: 1px solid var(--border); margin-top: 0.5rem; }
  ul { margin: 0.3rem 0 0; padding-left: 1.2rem; }
</style>
