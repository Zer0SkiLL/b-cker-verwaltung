import {
  affiliationInputFromForm,
  emptyOrganisationForm,
  emptyPersonForm,
  emptyToNull,
  membershipInputFromForm,
  organisationFormFrom,
  organisationInputFromForm,
  personInputFromForm,
  toIntOrNull,
} from './forms';

test('emptyToNull trimmt und macht Leeres zu null', () => {
  expect(emptyToNull('  ')).toBeNull();
  expect(emptyToNull(' a ')).toBe('a');
});

test('toIntOrNull', () => {
  expect(toIntOrNull('')).toEqual({ ok: true, value: null });
  expect(toIntOrNull(' 3 ')).toEqual({ ok: true, value: 3 });
  expect(toIntOrNull('2,5')).toEqual({ ok: false, error: 'Bitte eine ganze Zahl eingeben' });
});

test('Betrieb: Werte werden bereinigt', () => {
  const form = { ...emptyOrganisationForm(), name: ' Beck AG ', email: '' };
  const result = organisationInputFromForm(form);
  if (!('input' in result)) throw new Error('erwartet input');
  expect(result.input.name).toBe('Beck AG');
  expect(result.input.email).toBeNull();
});

test('Betrieb: leerer Name und Kommazahl ergeben Feldfehler', () => {
  const form = { ...emptyOrganisationForm(), name: '  ', branches: '2,5' };
  const result = organisationInputFromForm(form);
  if (!('errors' in result)) throw new Error('erwartet errors');
  expect(result.errors.name).toBe('Name ist erforderlich');
  expect(result.errors.branches).toBe('Bitte eine ganze Zahl eingeben');
});

test('Betrieb: Formular aus bestehendem Datensatz und zurück', () => {
  const org = {
    id: 1,
    name: 'Beck AG',
    addition: null,
    street: 'Hauptgasse 1',
    po_box: null,
    zip: '3000',
    city: 'Bern',
    phone: null,
    email: null,
    website: null,
    branches: 2,
    apprentice_count: null,
    notes: null,
    legacy_id: null,
    org_type_id: 4,
    cantonal_association_id: null,
  };
  const form = organisationFormFrom(org);
  expect(form.branches).toBe('2');
  const result = organisationInputFromForm(form);
  if (!('input' in result)) throw new Error('erwartet input');
  const { id: _id, ...input } = org;
  expect(result.input).toEqual(input);
});

test('Person: Nachname erforderlich, Rest bereinigt', () => {
  const bad = personInputFromForm({ ...emptyPersonForm(), last_name: '' });
  expect('errors' in bad && bad.errors.last_name).toBe('Nachname ist erforderlich');
  const ok = personInputFromForm({ ...emptyPersonForm(), last_name: 'Keller', compensation: ' CHF 300 ' });
  if (!('input' in ok)) throw new Error('erwartet input');
  expect(ok.input.compensation).toBe('CHF 300');
  expect(ok.input.first_name).toBeNull();
});

test('Zugehörigkeit: ungültiges Datum und fehlende Funktion', () => {
  const result = affiliationInputFromForm({
    person_id: 1,
    organisation_id: 2,
    function_id: null,
    valid_from: '31.02.2024',
    valid_to: '',
  });
  if (!('errors' in result)) throw new Error('erwartet errors');
  expect(result.errors.function_id).toBe('Bitte eine Funktion wählen');
  expect(result.errors.valid_from).toBe('Ungültiges Datum (TT.MM.JJJJ)');
});

test('Mitgliedschaft: ungültiger Austritt', () => {
  const result = membershipInputFromForm({
    member: { Organisation: 1 },
    category_id: null,
    joined_on: '1.1.2010',
    left_on: '31.02.2024',
  });
  if (!('errors' in result)) throw new Error('erwartet errors');
  expect(result.errors.left_on).toBe('Ungültiges Datum (TT.MM.JJJJ)');
});

test('Mitgliedschaft: gültige Daten werden ISO', () => {
  const result = membershipInputFromForm({
    member: { Person: 3 },
    category_id: 2,
    joined_on: '1.1.2010',
    left_on: '',
  });
  if (!('input' in result)) throw new Error('erwartet input');
  expect(result.input).toEqual({
    member: { Person: 3 },
    category_id: 2,
    joined_on: '2010-01-01',
    left_on: null,
  });
});
