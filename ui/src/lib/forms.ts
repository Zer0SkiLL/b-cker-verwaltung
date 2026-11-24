// Umwandlung Formular (Texte) → Eingabe für Rust. Keine Fachregeln, nur Form und Format.
import { parseDate } from './dates';
import type {
  AffiliationInput,
  MemberRef,
  MembershipInput,
  Organisation,
  OrganisationInput,
  Person,
  PersonInput,
} from './types';

export type Errors = Record<string, string>;
export type FormResult<T> = { input: T } | { errors: Errors };

export const INT_ERROR = 'Bitte eine ganze Zahl eingeben';

export function emptyToNull(value: string): string | null {
  const text = value.trim();
  return text === '' ? null : text;
}

export function toIntOrNull(value: string): { ok: true; value: number | null } | { ok: false; error: string } {
  const text = value.trim();
  if (text === '') return { ok: true, value: null };
  if (!/^-?\d+$/.test(text)) return { ok: false, error: INT_ERROR };
  return { ok: true, value: Number(text) };
}

const textOf = (v: string | null) => v ?? '';
const intText = (v: number | null) => (v === null ? '' : String(v));

function result<T>(input: T, errors: Errors): FormResult<T> {
  return Object.keys(errors).length > 0 ? { errors } : { input };
}

function dateField(errors: Errors, key: string, value: string): string | null {
  const parsed = parseDate(value);
  if (!parsed.ok) {
    errors[key] = parsed.error;
    return null;
  }
  return parsed.iso;
}

// ---------- Betrieb ----------

const ORG_TEXT = [
  'addition', 'street', 'po_box', 'zip', 'city', 'phone', 'email', 'website', 'notes', 'legacy_id',
] as const;

export interface OrganisationForm {
  name: string;
  addition: string;
  street: string;
  po_box: string;
  zip: string;
  city: string;
  phone: string;
  email: string;
  website: string;
  branches: string;
  apprentice_count: string;
  notes: string;
  legacy_id: string;
  org_type_id: number | null;
  cantonal_association_id: number | null;
}

export function emptyOrganisationForm(): OrganisationForm {
  return {
    name: '', addition: '', street: '', po_box: '', zip: '', city: '', phone: '', email: '',
    website: '', branches: '', apprentice_count: '', notes: '', legacy_id: '',
    org_type_id: null, cantonal_association_id: null,
  };
}

export function organisationFormFrom(org: Organisation): OrganisationForm {
  const texts = Object.fromEntries(ORG_TEXT.map((k) => [k, textOf(org[k])]));
  return {
    ...emptyOrganisationForm(),
    ...texts,
    name: org.name,
    branches: intText(org.branches),
    apprentice_count: intText(org.apprentice_count),
    org_type_id: org.org_type_id,
    cantonal_association_id: org.cantonal_association_id,
  };
}

export function organisationInputFromForm(form: OrganisationForm): FormResult<OrganisationInput> {
  const errors: Errors = {};
  const name = form.name.trim();
  if (name === '') errors.name = 'Name ist erforderlich';
  const ints = (['branches', 'apprentice_count'] as const).map((key) => {
    const parsed = toIntOrNull(form[key]);
    if (!parsed.ok) errors[key] = parsed.error;
    return parsed.ok ? parsed.value : null;
  });
  const texts = Object.fromEntries(ORG_TEXT.map((k) => [k, emptyToNull(form[k])]));
  const input = {
    ...texts,
    name,
    branches: ints[0],
    apprentice_count: ints[1],
    org_type_id: form.org_type_id,
    cantonal_association_id: form.cantonal_association_id,
  } as OrganisationInput;
  return result(input, errors);
}

// ---------- Person ----------

const PERSON_TEXT = [
  'salutation', 'first_name', 'street', 'zip', 'city', 'phone', 'mobile', 'email', 'compensation', 'notes',
] as const;

export type PersonForm = Record<(typeof PERSON_TEXT)[number] | 'last_name', string>;

export function emptyPersonForm(): PersonForm {
  const texts = Object.fromEntries(PERSON_TEXT.map((k) => [k, '']));
  return { ...texts, last_name: '' } as PersonForm;
}

export function personFormFrom(person: Person): PersonForm {
  const texts = Object.fromEntries(PERSON_TEXT.map((k) => [k, textOf(person[k])]));
  return { ...texts, last_name: person.last_name } as PersonForm;
}

export function personInputFromForm(form: PersonForm): FormResult<PersonInput> {
  const errors: Errors = {};
  const last_name = form.last_name.trim();
  if (last_name === '') errors.last_name = 'Nachname ist erforderlich';
  const texts = Object.fromEntries(PERSON_TEXT.map((k) => [k, emptyToNull(form[k])]));
  return result({ ...texts, last_name } as PersonInput, errors);
}

// ---------- Zugehörigkeit und Mitgliedschaft ----------

export interface AffiliationForm {
  person_id: number | null;
  organisation_id: number | null;
  function_id: number | null;
  valid_from: string;
  valid_to: string;
}

export function affiliationInputFromForm(form: AffiliationForm): FormResult<AffiliationInput> {
  const errors: Errors = {};
  if (form.person_id === null) errors.person_id = 'Bitte eine Person wählen';
  if (form.organisation_id === null) errors.organisation_id = 'Bitte einen Betrieb wählen';
  if (form.function_id === null) errors.function_id = 'Bitte eine Funktion wählen';
  const valid_from = dateField(errors, 'valid_from', form.valid_from);
  const valid_to = dateField(errors, 'valid_to', form.valid_to);
  const input = {
    person_id: form.person_id ?? 0,
    organisation_id: form.organisation_id ?? 0,
    function_id: form.function_id ?? 0,
    valid_from,
    valid_to,
  };
  return result(input, errors);
}

export interface MembershipForm {
  member: MemberRef;
  category_id: number | null;
  joined_on: string;
  left_on: string;
}

export function membershipInputFromForm(form: MembershipForm): FormResult<MembershipInput> {
  const errors: Errors = {};
  const joined_on = dateField(errors, 'joined_on', form.joined_on);
  const left_on = dateField(errors, 'left_on', form.left_on);
  return result({ member: form.member, category_id: form.category_id, joined_on, left_on }, errors);
}
