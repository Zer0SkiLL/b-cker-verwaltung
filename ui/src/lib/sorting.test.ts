import { get } from 'svelte/store';
import { addressSort, nextSort, sortRows, type SortState } from './sorting';
import type { ContactRow } from './types';

const row = (id: number, over: Partial<ContactRow> = {}): ContactRow => ({
  kind: 'Organisation',
  id,
  display_name: `Eintrag ${id}`,
  organisation: null,
  street: null,
  zip: null,
  city: null,
  po_box: null,
  email: null,
  phone: null,
  mobile: null,
  ...over,
});

const names = (rows: ContactRow[]) => rows.map((r) => r.display_name);

test('ohne Sortierung bleibt die Reihenfolge, die Liste wird nicht verändert', () => {
  const rows = [row(2, { display_name: 'B' }), row(1, { display_name: 'A' })];
  const sorted = sortRows(rows, null);
  expect(names(sorted)).toEqual(['B', 'A']);
  expect(sorted).not.toBe(rows);
});

test('Name: alphabetisch mit Umlauten, ohne Gross/Klein, absteigend umgekehrt', () => {
  const rows = [row(1, { display_name: 'Zürcher' }), row(2, { display_name: 'äbi' }), row(3, { display_name: 'Bäckerei' })];
  const original = names(rows);
  expect(names(sortRows(rows, { key: 'name', dir: 'asc' }))).toEqual(['äbi', 'Bäckerei', 'Zürcher']);
  expect(names(sortRows(rows, { key: 'name', dir: 'desc' }))).toEqual(['Zürcher', 'Bäckerei', 'äbi']);
  expect(names(rows)).toEqual(original);
});

test('leere Werte stehen immer am Schluss', () => {
  const rows = [row(1, { display_name: 'A' }), row(2, { display_name: 'B', email: 'b@x.ch' }), row(3, { display_name: 'C', email: 'a@x.ch' })];
  expect(names(sortRows(rows, { key: 'email', dir: 'asc' }))).toEqual(['C', 'B', 'A']);
  expect(names(sortRows(rows, { key: 'email', dir: 'desc' }))).toEqual(['B', 'C', 'A']);
});

test('Ort: nach Ortsname, bei gleichem Ort nach PLZ', () => {
  const rows = [
    row(1, { display_name: 'A', zip: '8004', city: 'Zürich' }),
    row(2, { display_name: 'B', zip: '3000', city: 'Bern' }),
    row(3, { display_name: 'C', zip: '8001', city: 'Zürich' }),
  ];
  expect(names(sortRows(rows, { key: 'place', dir: 'asc' }))).toEqual(['B', 'C', 'A']);
});

test('Telefon fällt wie in der Tabelle auf das Mobiltelefon zurück', () => {
  const rows = [row(1, { display_name: 'A', phone: '044 2' }), row(2, { display_name: 'B', mobile: '044 1' })];
  expect(names(sortRows(rows, { key: 'phone', dir: 'asc' }))).toEqual(['B', 'A']);
});

test('Typ und Betrieb; Gleichstand nach Name', () => {
  const rows = [
    row(1, { display_name: 'Z', kind: 'Person', organisation: 'Beck AG' }),
    row(2, { display_name: 'Y', kind: 'Organisation' }),
    row(3, { display_name: 'X', kind: 'Person', organisation: 'Beck AG' }),
  ];
  expect(names(sortRows(rows, { key: 'kind', dir: 'asc' }))).toEqual(['Y', 'X', 'Z']);
  expect(names(sortRows(rows, { key: 'organisation', dir: 'asc' }))).toEqual(['X', 'Z', 'Y']);
});

test('Zahlen in Texten werden natürlich sortiert', () => {
  const rows = [row(1, { display_name: 'Filiale 10' }), row(2, { display_name: 'Filiale 9' })];
  expect(names(sortRows(rows, { key: 'name', dir: 'asc' }))).toEqual(['Filiale 9', 'Filiale 10']);
});

test('nextSort: neue Spalte aufsteigend, gleiche Spalte wechselt die Richtung', () => {
  const asc: SortState = { key: 'place', dir: 'asc' };
  expect(nextSort(null, 'name')).toEqual({ key: 'name', dir: 'asc' });
  expect(nextSort({ key: 'name', dir: 'asc' }, 'name')).toEqual({ key: 'name', dir: 'desc' });
  expect(nextSort({ key: 'name', dir: 'desc' }, 'name')).toEqual({ key: 'name', dir: 'asc' });
  expect(nextSort(asc, 'email')).toEqual({ key: 'email', dir: 'asc' });
});

test('addressSort startet ohne Sortierung', () => {
  expect(get(addressSort)).toBeNull();
});
