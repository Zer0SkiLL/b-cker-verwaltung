// Texte der Import-Vorschau: Zusammenfassung, „neu angelegt“, Filter „nur mit Hinweisen“.
import type { ImportPreview, NewEntries, PreviewOrganisation } from './types';

export function plural(n: number, one: string, many: string): string {
  return `${n} ${n === 1 ? one : many}`;
}

export function summaryText(p: ImportPreview): string {
  return (
    `${plural(p.rows, 'Zeile', 'Zeilen')} → ${plural(p.organisations.length, 'Betrieb', 'Betriebe')} ` +
    `(davon ${p.organisations.filter((o) => o.lines.length > 1).length} aus mehreren Zeilen zusammengefasst), ` +
    `${plural(p.persons, 'Person', 'Personen')}, ${plural(p.individuals.length, 'Einzelmitglied', 'Einzelmitglieder')}, ` +
    `${plural(p.skipped.length, 'Zeile', 'Zeilen')} übersprungen`
  );
}

const LISTS: [keyof NewEntries, string][] = [
  ['categories', 'Mitgliederkategorien'],
  ['cantonal', 'Kantonalverbände'],
  ['functions', 'Funktionen'],
  ['labels', 'Labels'],
];

/** Eine Zeile pro Liste mit neuen Einträgen; leer, wenn nichts Neues entsteht. */
export function newEntriesText(entries: NewEntries): string[] {
  return LISTS.filter(([key]) => entries[key].length > 0).map(
    ([key, label]) => `${label}: ${entries[key].join(', ')}`,
  );
}

export function hasNotes(org: PreviewOrganisation): boolean {
  return org.notes.length > 0 || org.persons.some((p) => p.notes.length > 0);
}

/** „Zeile 4“, „Zeilen 3–5“, „Zeilen 3–4, 9“: aufeinanderfolgende Zeilen als Bereich. */
export function linesText(lines: number[]): string {
  if (lines.length === 1) return `Zeile ${lines[0]}`;
  const ranges: string[] = [];
  let start = lines[0];
  let prev = lines[0];
  for (const n of [...lines.slice(1), NaN]) {
    if (n === prev + 1) {
      prev = n;
      continue;
    }
    ranges.push(start === prev ? `${start}` : `${start}–${prev}`);
    start = n;
    prev = n;
  }
  return `Zeilen ${ranges.join(', ')}`;
}

/** Kurzfassung der Spaltenzuordnung für die eingeklappte Überschrift. */
export function mappingText(mapping: string[]): string {
  const ignored = mapping.filter((m) => m === 'ignore').length;
  const rest =
    ignored === 0
      ? 'alle werden übernommen'
      : `${ignored} ${ignored === 1 ? 'wird' : 'werden'} nicht übernommen`;
  return `${plural(mapping.length, 'Spalte', 'Spalten')}, ${rest}`;
}
