<script lang="ts">
  import { readOnly as readOnlyStore } from '../lib/session';
  import { untrack } from 'svelte';
  import * as api from '../lib/api';
  import { confirmAction } from '../lib/desktop';
  import { emptyFilter } from '../lib/filters';
  import { emptyToNull } from '../lib/forms';
  import { navigate } from '../lib/router';
  import { dataVersion, runWrite, sessionInfo } from '../lib/session';
  import type { OptionDto, TeamMemberView, TeamSummary } from '../lib/types';

  let teams = $state<TeamSummary[]>([]);
  let selectedId = $state<number | null>(null);
  let members = $state<TeamMemberView[]>([]);
  let people = $state<OptionDto[]>([]);
  let error = $state<string | null>(null);
  let newName = $state('');
  let edit = $state<{ name: string; description: string } | null>(null);
  let addPerson = $state('');
  let addRole = $state('');

  const readOnly = $derived($readOnlyStore);
  const selected = $derived(teams.find((t) => t.team.id === selectedId) ?? null);

  async function load(teamId: number | null) {
    try {
      teams = await api.listTeams();
      people = await api.listPeople();
      members = teamId === null ? [] : await api.teamMembers(teamId);
    } catch (err) {
      error = (err as Error).message;
    }
  }

  $effect(() => {
    const id = selectedId;
    void $dataVersion;
    untrack(() => load(id));
  });

  async function act(task: () => Promise<unknown>): Promise<boolean> {
    try {
      await runWrite(task);
      error = null;
      return true;
    } catch (err) {
      error = (err as Error).message;
      return false;
    }
  }

  async function create() {
    const name = newName.trim();
    if (!name) return;
    const ok = await act(async () => {
      const team = await api.createTeam(name, null);
      selectedId = team.id;
    });
    if (ok) newName = '';
  }

  async function saveEdit() {
    if (!selected || !edit) return;
    const { name, description } = edit;
    if (await act(() => api.updateTeam(selected.team.id, name, emptyToNull(description)))) edit = null;
  }

  async function remove() {
    if (!selected || !(await confirmAction(`Team „${selected.team.name}“ löschen?`, 'Löschen'))) return;
    const id = selected.team.id;
    if (await act(() => api.deleteTeam(id))) selectedId = null;
  }

  async function add() {
    if (!selected || addPerson === '') return;
    const teamId = selected.team.id;
    if (await act(() => api.addTeamMember(teamId, Number(addPerson), emptyToNull(addRole)))) {
      addPerson = '';
      addRole = '';
    }
  }

  const setRole = (m: TeamMemberView, role: string) =>
    selected && act(() => api.addTeamMember(selected.team.id, m.personId, emptyToNull(role)));
  const removeMember = (m: TeamMemberView) =>
    selected && act(() => api.removeTeamMember(selected.team.id, m.personId));
</script>

<h1>Teams</h1>
{#if error}<div class="form-error">{error}</div>{/if}
<div class="split">
  <div class="list">
    {#each teams as t (t.team.id)}
      <button class:active={t.team.id === selectedId} onclick={() => (selectedId = t.team.id)}>
        {t.team.name} <span class="muted">({t.memberCount})</span>
      </button>
    {:else}
      <p class="muted">Noch keine Teams.</p>
    {/each}
    {#if !readOnly}
      <div class="toolbar new">
        <input placeholder="Neues Team" bind:value={newName} onkeydown={(e) => e.key === 'Enter' && create()} />
        <button onclick={create}>Anlegen</button>
      </div>
    {/if}
  </div>

  <div class="detail">
    {#if selected}
      {#if edit}
        <div class="card">
          <div class="field"><label for="t-name">Name</label><input id="t-name" bind:value={edit.name} /></div>
          <div class="field"><label for="t-desc">Beschreibung</label><input id="t-desc" bind:value={edit.description} /></div>
          <div class="toolbar"><button class="primary" onclick={saveEdit}>Speichern</button><button onclick={() => (edit = null)}>Abbrechen</button></div>
        </div>
      {:else}
        <h2>{selected.team.name}</h2>
        {#if selected.team.description}<p class="muted">{selected.team.description}</p>{/if}
      {/if}
      <div class="toolbar">
        <button class="primary" onclick={() => selected && navigate({ page: 'addresses', filter: { ...emptyFilter(), team_ids: [selected.team.id] } })}>Team anschreiben</button>
        {#if !readOnly && !edit}
          <button onclick={() => selected && (edit = { name: selected.team.name, description: selected.team.description ?? '' })}>Umbenennen</button>
          <button class="danger" onclick={remove}>Team löschen</button>
        {/if}
      </div>
      <table>
        <thead><tr><th>Person</th><th>Rolle</th><th></th></tr></thead>
        <tbody>
          {#each members as m (m.personId)}
            <tr>
              <td><button class="link" onclick={() => navigate({ page: 'person', id: m.personId })}>{m.name}</button></td>
              <td><input value={m.role ?? ''} disabled={readOnly} aria-label="Rolle von {m.name}" onchange={(e) => setRole(m, e.currentTarget.value)} /></td>
              <td class="actions">{#if !readOnly}<button class="danger" onclick={() => removeMember(m)}>Entfernen</button>{/if}</td>
            </tr>
          {:else}
            <tr><td colspan="3" class="muted">Noch keine Mitglieder.</td></tr>
          {/each}
        </tbody>
      </table>
      {#if !readOnly}
        <div class="toolbar add">
          <select bind:value={addPerson} aria-label="Person hinzufügen">
            <option value="">Person hinzufügen…</option>
            {#each people.filter((p) => !members.some((m) => m.personId === p.id)) as p (p.id)}
              <option value={String(p.id)}>{p.name}</option>
            {/each}
          </select>
          <input placeholder="Rolle (optional)" bind:value={addRole} />
          <button disabled={addPerson === ''} onclick={add}>Hinzufügen</button>
        </div>
      {/if}
    {:else}
      <p class="muted">Team links auswählen.</p>
    {/if}
  </div>
</div>

<style>
  .split { display: grid; grid-template-columns: 260px 1fr; gap: 1.5rem; }
  .list { display: flex; flex-direction: column; gap: 0.2rem; }
  .list > button { text-align: left; border: none; background: none; padding: 0.4rem 0.6rem; border-radius: 6px; }
  .list > button.active { background: var(--bg-muted); font-weight: 600; }
  .new, .add { margin-top: 0.75rem; }
  .actions { text-align: right; }
</style>
