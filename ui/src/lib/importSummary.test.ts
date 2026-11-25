import { hasNotes, linesText, mappingText, newEntriesText, plural, summaryText } from './importSummary';
import type { ImportPreview } from './types';

const preview = (over: Partial<ImportPreview> = {}): ImportPreview => ({
  headers: [], mapping: [], fields: [], rows: 4, merged: 1, persons: 2,
  organisations: [{ lines: [2, 3], name: 'Beck AG', city: null, category: null, labels: [], notes: [],
    persons: [{ line: 2, name: 'Anna Keller', function: 'Kontaktperson', labels: [], notes: [] }] }],
  individuals: [{ person: { line: 4, name: 'Hans', function: 'Einzelmitglied', labels: [], notes: [] }, city: null, category: null }],
  skipped: [{ line: 5, reason: 'Weder Firma noch Name, Zeile übersprungen' }],
  newEntries: { categories: [], cantonal: [], functions: [], labels: [] },
  reconcileError: null, report: [], databaseEmpty: true, ...over,
});

test('plural wählt Einzahl oder Mehrzahl', () => {
  expect(plural(1, 'Betrieb', 'Betriebe')).toBe('1 Betrieb');
  expect(plural(0, 'Betrieb', 'Betriebe')).toBe('0 Betriebe');
  expect(plural(3, 'Zeile', 'Zeilen')).toBe('3 Zeilen');
});

test('Zusammenfassung mit allen Zahlen', () => {
  expect(summaryText(preview())).toBe(
    '4 Zeilen → 1 Betrieb (davon 1 aus mehreren Zeilen zusammengefasst), 2 Personen, 1 Einzelmitglied, 1 Zeile übersprungen',
  );
});

test('neu angelegt: pro Liste, sonst „nichts Neues“', () => {
  expect(newEntriesText(preview().newEntries)).toEqual([]);
  const lines = newEntriesText({ categories: ['Ehrenmitglied'], cantonal: ['FL', 'SG'], functions: ['Kontaktperson'], labels: ['Mandat: Präsident'] });
  expect(lines).toEqual([
    'Mitgliederkategorien: Ehrenmitglied',
    'Kantonalverbände: FL, SG',
    'Funktionen: Kontaktperson',
    'Labels: Mandat: Präsident',
  ]);
});

test('hasNotes erkennt Hinweise beim Betrieb oder seinen Personen', () => {
  const org = preview().organisations[0];
  expect(hasNotes(org)).toBe(false);
  expect(hasNotes({ ...org, persons: [{ ...org.persons[0], notes: ['x'] }] })).toBe(true);
  expect(hasNotes({ ...org, notes: ['y'] })).toBe(true);
});

test('Quelle fasst aufeinanderfolgende Zeilen zusammen', () => {
  expect(linesText([4])).toBe('Zeile 4');
  expect(linesText([3, 4, 5])).toBe('Zeilen 3–5');
  expect(linesText([3, 4, 9])).toBe('Zeilen 3–4, 9');
  expect(linesText([2, 7])).toBe('Zeilen 2, 7');
});

test('Zuordnung: Zahl der Spalten und der nicht übernommenen', () => {
  expect(mappingText(['company', 'ignore', 'zip'])).toBe('3 Spalten, 1 wird nicht übernommen');
  expect(mappingText(['company'])).toBe('1 Spalte, alle werden übernommen');
  expect(mappingText(['ignore', 'ignore'])).toBe('2 Spalten, 2 werden nicht übernommen');
});
