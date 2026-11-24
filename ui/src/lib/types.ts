// Spiegel der Rust-Typen. Core-Strukturen in snake_case, App-DTOs in camelCase.
// Datum: "YYYY-MM-DD", Zeitpunkte: RFC 3339.

export type IsoDate = string;

export type EntryKind = 'Person' | 'Organisation';
export type MembershipStatus = 'Any' | 'Active' | 'Exited';
export type LookupKind = 'OrgType' | 'Function' | 'MembershipCategory' | 'CantonalAssociation';
export type AppliesTo = 'Person' | 'Organisation' | 'Both';
export type MemberRef = { Organisation: number } | { Person: number };

export interface Filter {
  text: string | null;
  kinds: EntryKind[];
  label_ids: number[];
  team_ids: number[];
  function_ids: number[];
  org_type_ids: number[];
  category_ids: number[];
  cantonal_ids: number[];
  membership: MembershipStatus;
}

export type FilterIdKey =
  | 'label_ids'
  | 'team_ids'
  | 'function_ids'
  | 'org_type_ids'
  | 'category_ids'
  | 'cantonal_ids';

export interface ContactRow {
  kind: EntryKind;
  id: number;
  display_name: string;
  organisation: string | null;
  street: string | null;
  zip: string | null;
  city: string | null;
  po_box: string | null;
  email: string | null;
  phone: string | null;
  mobile: string | null;
}

export interface SearchResult {
  rows: ContactRow[];
  emailCount: number;
  missingEmail: ContactRow[];
  joinedEmails: string;
  mailto: string | null;
}

export interface LockDto {
  user: string;
  host: string;
  since: string;
  likelyCrash: boolean;
}

export interface BackupStatusDto {
  secondaryDir: string | null;
  lastSecondary: string | null;
  reminderDue: boolean;
}

export interface SessionInfo {
  path: string;
  readOnly: boolean;
  heldBy: LockDto | null;
  backup: BackupStatusDto;
}

export interface OpenResult {
  info: SessionInfo;
  warning: string | null;
}

export type SecondaryResultDto =
  | { kind: 'notConfigured' }
  | { kind: 'done'; path: string }
  | { kind: 'unavailable'; reason: string };

export interface LookupValue {
  id: number;
  kind: LookupKind;
  name: string;
}

export interface LabelGroup {
  id: number;
  name: string;
  applies_to: AppliesTo;
}

export interface Label {
  id: number;
  group_id: number;
  name: string;
}

export interface Team {
  id: number;
  name: string;
  description: string | null;
}

export interface FilterOptions {
  labelGroups: { group: LabelGroup; labels: Label[] }[];
  teams: Team[];
  functions: LookupValue[];
  orgTypes: LookupValue[];
  categories: LookupValue[];
  cantonal: LookupValue[];
}

export interface GroupDto {
  id: number;
  name: string;
}

export interface OrganisationInput {
  name: string;
  addition: string | null;
  street: string | null;
  po_box: string | null;
  zip: string | null;
  city: string | null;
  phone: string | null;
  email: string | null;
  website: string | null;
  branches: number | null;
  apprentice_count: number | null;
  notes: string | null;
  legacy_id: string | null;
  org_type_id: number | null;
  cantonal_association_id: number | null;
}

export interface Organisation extends OrganisationInput {
  id: number;
}

export interface PersonInput {
  salutation: string | null;
  first_name: string | null;
  last_name: string;
  street: string | null;
  zip: string | null;
  city: string | null;
  phone: string | null;
  mobile: string | null;
  email: string | null;
  compensation: string | null;
  notes: string | null;
}

export interface Person extends PersonInput {
  id: number;
}

export interface AffiliationInput {
  person_id: number;
  organisation_id: number;
  function_id: number;
  valid_from: IsoDate | null;
  valid_to: IsoDate | null;
}

export interface Affiliation extends AffiliationInput {
  id: number;
}

export interface MembershipInput {
  member: MemberRef;
  category_id: number | null;
  joined_on: IsoDate | null;
  left_on: IsoDate | null;
}

export interface Membership extends MembershipInput {
  id: number;
}

export interface AssignmentInput {
  label_id: number;
  target: MemberRef;
  assigned_on: IsoDate | null;
  note: string | null;
}

export interface Assignment extends AssignmentInput {
  id: number;
}

export interface AffiliationView {
  affiliation: Affiliation;
  personName: string;
  organisationName: string;
  functionName: string;
  current: boolean;
}

export interface AssignedLabel {
  assignment: Assignment;
  groupName: string;
  labelName: string;
}

export interface OrganisationDetail {
  organisation: Organisation;
  people: AffiliationView[];
  memberships: Membership[];
  labels: AssignedLabel[];
}

export interface PersonDetail {
  person: Person;
  organisations: AffiliationView[];
  memberships: Membership[];
  labels: AssignedLabel[];
  teams: { team: Team; role: string | null }[];
}

export interface OptionDto {
  id: number;
  name: string;
}

export interface LookupUsage {
  value: LookupValue;
  usage: number;
}

export interface LabelGroupTree {
  group: LabelGroup;
  labels: { label: Label; usage: number }[];
}

export interface TeamSummary {
  team: Team;
  memberCount: number;
}

export interface TeamMemberView {
  personId: number;
  name: string;
  role: string | null;
}

export interface BackupFileDto {
  path: string;
  created: string;
}

export interface BackupOverview {
  status: BackupStatusDto;
  localDir: string;
  files: BackupFileDto[];
}

export interface BackupNowDto {
  local: string;
  secondary: SecondaryResultDto;
}

export interface ReportEntry {
  line: number;
  column: string | null;
  message: string;
}

export interface PreviewRow {
  line: number;
  organisation: string;
  person: string | null;
  city: string | null;
  category: string | null;
}

export interface ImportPreview {
  headers: string[];
  mapping: string[];
  fields: string[];
  rows: number;
  organisations: number;
  persons: number;
  sample: PreviewRow[];
  report: ReportEntry[];
  databaseEmpty: boolean;
}

export interface ImportSummary {
  organisations: number;
  persons: number;
  report: ReportEntry[];
}

export type ExportFormat = 'csv' | 'xlsx';
