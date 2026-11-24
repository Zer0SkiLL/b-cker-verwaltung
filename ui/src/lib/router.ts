// Einfache Navigation mit Verlauf (für „Zurück", das den Adressen-Filter erhält).
import { get, writable } from 'svelte/store';
import type { Filter } from './types';

export type SettingsTab = 'catalog' | 'backup' | 'import' | 'export';

export type Route =
  | { page: 'start' }
  | { page: 'addresses'; filter?: Filter }
  | { page: 'organisation'; id: number | null }
  | { page: 'person'; id: number | null }
  | { page: 'teams' }
  | { page: 'groups' }
  | { page: 'settings'; tab?: SettingsTab };

const START: Route = { page: 'start' };

export const route = writable<Route>(START);
let history: Route[] = [];

export function navigate(next: Route): void {
  history = [...history, get(route)];
  route.set(next);
}

/** Ersetzt die aktuelle Seite ohne Verlaufseintrag (z.B. Filter geändert, neu gespeichert). */
export function replace(next: Route): void {
  route.set(next);
}

export function back(): void {
  const previous = history[history.length - 1];
  if (!previous) return;
  history = history.slice(0, -1);
  route.set(previous);
}

export function resetRouter(): void {
  history = [];
  route.set(START);
}
