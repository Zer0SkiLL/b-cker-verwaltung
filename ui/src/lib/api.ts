// Typisierte Aufrufe der Rust-Befehle. Fehler kommen als deutscher Text (Error.message).
import { invoke } from '@tauri-apps/api/core';
import type * as T from './types';

async function call<R>(command: string, args?: Record<string, unknown>): Promise<R> {
  try {
    return await invoke<R>(command, args);
  } catch (err) {
    throw new Error(typeof err === 'string' ? err : String(err));
  }
}

// Sitzung
export const lastPath = () => call<string | null>('last_path');
export const openFile = (path: string) => call<T.OpenResult>('open_file', { path });
export const createFile = (path: string) => call<T.OpenResult>('create_file', { path });
export const takeOver = () => call<T.SessionInfo>('take_over');
export const sessionInfo = () => call<T.SessionInfo>('session_info');
export const closeFile = () => call<T.SecondaryResultDto | null>('close_file');

// Adressen und Gruppen
export const searchAddresses = (filter: T.Filter) =>
  call<T.SearchResult>('search_addresses', { filter });
export const exportAddresses = (filter: T.Filter, path: string, format: T.ExportFormat) =>
  call<void>('export_addresses', { filter, path, format });
export const listGroups = () => call<T.GroupDto[]>('list_groups');
export const saveGroup = (name: string, filter: T.Filter) =>
  call<number>('save_group', { name, filter });
export const loadGroup = (id: number) => call<T.Filter>('load_group', { id });
export const deleteGroup = (id: number) => call<void>('delete_group', { id });
export const renameGroup = (id: number, name: string) => call<void>('rename_group', { id, name });
export const filterOptions = () => call<T.FilterOptions>('filter_options');

// Betriebe und Personen
export const organisationDetail = (id: number, includePast: boolean) =>
  call<T.OrganisationDetail>('organisation_detail', { id, includePast });
export const personDetail = (id: number, includePast: boolean) =>
  call<T.PersonDetail>('person_detail', { id, includePast });
export const listPeople = () => call<T.OptionDto[]>('list_people');
export const listOrganisations = () => call<T.OptionDto[]>('list_organisations');
export const createOrganisation = (input: T.OrganisationInput) =>
  call<T.Organisation>('create_organisation', { input });
export const updateOrganisation = (id: number, input: T.OrganisationInput) =>
  call<T.Organisation>('update_organisation', { id, input });
export const deleteOrganisation = (id: number) => call<void>('delete_organisation', { id });
export const createPerson = (input: T.PersonInput) => call<T.Person>('create_person', { input });
export const updatePerson = (id: number, input: T.PersonInput) =>
  call<T.Person>('update_person', { id, input });
export const deletePerson = (id: number) => call<void>('delete_person', { id });
export const createAffiliation = (input: T.AffiliationInput) =>
  call<T.Affiliation>('create_affiliation', { input });
export const updateAffiliation = (id: number, input: T.AffiliationInput) =>
  call<T.Affiliation>('update_affiliation', { id, input });
export const deleteAffiliation = (id: number) => call<void>('delete_affiliation', { id });
export const createMembership = (input: T.MembershipInput) =>
  call<T.Membership>('create_membership', { input });
export const updateMembership = (id: number, input: T.MembershipInput) =>
  call<T.Membership>('update_membership', { id, input });
export const deleteMembership = (id: number) => call<void>('delete_membership', { id });
export const assignLabel = (input: T.AssignmentInput) =>
  call<T.Assignment>('assign_label', { input });
export const unassignLabel = (id: number) => call<void>('unassign_label', { id });

// Listen, Labels, Teams
export const listLookups = (kind: T.LookupKind) => call<T.LookupUsage[]>('list_lookups', { kind });
export const createLookup = (kind: T.LookupKind, name: string) =>
  call<T.LookupValue>('create_lookup', { kind, name });
export const renameLookup = (id: number, name: string) =>
  call<T.LookupValue>('rename_lookup', { id, name });
export const deleteLookup = (id: number, confirm: boolean) =>
  call<void>('delete_lookup', { id, confirm });
export const listLabelTree = () => call<T.LabelGroupTree[]>('list_label_tree');
export const createLabelGroup = (name: string, appliesTo: T.AppliesTo) =>
  call<T.LabelGroup>('create_label_group', { name, appliesTo });
export const createLabel = (groupId: number, name: string) =>
  call<T.Label>('create_label', { groupId, name });
export const renameLabel = (id: number, name: string) => call<T.Label>('rename_label', { id, name });
export const deleteLabel = (id: number, confirm: boolean) =>
  call<void>('delete_label', { id, confirm });
export const deleteLabelGroup = (id: number, confirm: boolean) =>
  call<void>('delete_label_group', { id, confirm });
export const renameLabelGroup = (id: number, name: string) =>
  call<T.LabelGroup>('rename_label_group', { id, name });
export const listTeams = () => call<T.TeamSummary[]>('list_teams');
export const teamMembers = (teamId: number) => call<T.TeamMemberView[]>('team_members', { teamId });
export const createTeam = (name: string, description: string | null) =>
  call<T.Team>('create_team', { name, description });
export const updateTeam = (id: number, name: string, description: string | null) =>
  call<T.Team>('update_team', { id, name, description });
export const deleteTeam = (id: number) => call<void>('delete_team', { id });
export const addTeamMember = (teamId: number, personId: number, role: string | null) =>
  call<void>('add_team_member', { teamId, personId, role });
export const removeTeamMember = (teamId: number, personId: number) =>
  call<void>('remove_team_member', { teamId, personId });

// Sicherung, Export, Import
export const backupOverview = () => call<T.BackupOverview>('backup_overview');
export const setSecondaryDir = (dir: string | null) => call<void>('set_secondary_dir', { dir });
export const backupNow = () => call<T.BackupNowDto>('backup_now');
export const restoreBackup = (path: string) => call<void>('restore_backup', { path });
export const exportAll = (path: string) => call<void>('export_all', { path });
export const importPreview = (path: string, mapping: string[] | null) =>
  call<T.ImportPreview>('import_preview', { path, mapping });
export const importApply = (path: string, mapping: string[]) =>
  call<T.ImportSummary>('import_apply', { path, mapping });
export const importUndo = (backup: string) => call<void>('import_undo', { backup });
export const exportImportPreview = (path: string, mapping: string[] | null, out: string) =>
  call<void>('export_import_preview', { path, mapping, out });
export const exportReport = (report: T.ReportEntry[], path: string) =>
  call<void>('export_report', { report, path });
