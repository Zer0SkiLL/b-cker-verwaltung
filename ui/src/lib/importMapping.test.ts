import { FIELD_LABELS, fieldLabel, setColumn } from './importMapping';

const ALL_KEYS = [
  'legacy_id', 'apprentice_count', 'cantonal_association', 'name', 'first_name', 'addition',
  'street', 'zip', 'city', 'category', 'email', 'website', 'apprenticeship', 'fr_baeck', 'fr_conf',
  'dh', 'notes', 'joined', 'left', 'branches', 'phone', 'mobile', 'po_box', 'ignore',
];

test('jedes Feld hat einen deutschen Namen', () => {
  expect(ALL_KEYS).toHaveLength(24);
  for (const key of ALL_KEYS) expect(FIELD_LABELS[key]).toBeTruthy();
  expect(FIELD_LABELS.name).toBe('Name/Firma');
  expect(FIELD_LABELS.ignore).toBe('– nicht übernehmen –');
});

test('unbekannter Schlüssel wird unverändert angezeigt', () => {
  expect(fieldLabel('neu')).toBe('neu');
});

test('setColumn liefert ein neues Array', () => {
  const original = Object.freeze(['name', 'ignore']) as string[];
  const next = setColumn(original, 1, 'city');
  expect(next).toEqual(['name', 'city']);
  expect(original).toEqual(['name', 'ignore']);
});
