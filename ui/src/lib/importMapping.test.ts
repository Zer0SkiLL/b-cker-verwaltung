import { FIELD_LABELS, fieldLabel, setColumn } from './importMapping';

// Gleiche Schlüssel wie ALL_FIELDS in crates/app/src/commands/import.rs.
const ALL_KEYS = [
  'company', 'last_name', 'first_name', 'addition', 'legacy_id', 'cantonal_association', 'street',
  'zip', 'city', 'category', 'email', 'website', 'phone', 'mobile', 'apprenticeship', 'fr_baeck',
  'fr_conf', 'dh', 'notes', 'joined', 'left', 'branches', 'apprentice_count', 'cost_center_no',
  'commercial_register_no', 'mandate', 'ignore',
];

test('jedes Feld hat einen deutschen Namen', () => {
  expect(ALL_KEYS).toHaveLength(27);
  for (const key of ALL_KEYS) expect(FIELD_LABELS[key]).toBeTruthy();
  expect(FIELD_LABELS.company).toBe('Firma (Betrieb)');
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
