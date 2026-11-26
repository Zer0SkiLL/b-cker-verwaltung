// Sortierung der Adressliste per Klick auf die Spaltenköpfe. Rein im Programm, ohne neue Abfrage.
import { writable } from 'svelte/store';
import type { ContactRow } from './types';

export type SortKey = 'kind' | 'name' | 'organisation' | 'place' | 'email' | 'phone';
export type SortDir = 'asc' | 'desc';
export type SortState = { key: SortKey; dir: SortDir } | null;

/** Gewählte Sortierung; bleibt beim Wechsel zu einem Betrieb und zurück erhalten. */
export const addressSort = writable<SortState>(null);

const collator = new Intl.Collator('de-CH', { numeric: true, sensitivity: 'base' });

// Gleiche Texte wie in der Tabelle, damit die Reihenfolge zum Gezeigten passt.
const KEYS: Record<SortKey, (row: ContactRow) => string[]> = {
  kind: (r) => [r.kind === 'Person' ? 'Person' : 'Betrieb'],
  name: (r) => [r.display_name],
  organisation: (r) => [r.organisation ?? ''],
  place: (r) => [r.city ?? '', r.zip ?? ''],
  email: (r) => [r.email ?? ''],
  phone: (r) => [r.phone ?? r.mobile ?? ''],
};

function compareValues(a: string[], b: string[], dir: SortDir): number {
  // Leere Werte stehen in beiden Richtungen am Schluss.
  const emptyA = a[0].trim() === '';
  const emptyB = b[0].trim() === '';
  if (emptyA !== emptyB) return emptyA ? 1 : -1;
  const sign = dir === 'asc' ? 1 : -1;
  for (let i = 0; i < a.length; i++) {
    const result = collator.compare(a[i], b[i]);
    if (result !== 0) return result * sign;
  }
  return 0;
}

/** Neue, sortierte Liste; ohne Sortierung eine Kopie in der bisherigen Reihenfolge. */
export function sortRows(rows: readonly ContactRow[], state: SortState): ContactRow[] {
  if (!state) return [...rows];
  const key = KEYS[state.key];
  return [...rows].sort(
    (a, b) =>
      compareValues(key(a), key(b), state.dir) ||
      collator.compare(a.display_name, b.display_name) ||
      a.id - b.id,
  );
}

/** Klick auf einen Spaltenkopf: neue Spalte aufsteigend, dieselbe Spalte wechselt die Richtung. */
export function nextSort(state: SortState, key: SortKey): SortState {
  if (state?.key === key) return { key, dir: state.dir === 'asc' ? 'desc' : 'asc' };
  return { key, dir: 'asc' };
}
