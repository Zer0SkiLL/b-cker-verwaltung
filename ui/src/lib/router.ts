// Einfache Navigation mit Verlauf (für „Zurück", das den Adressen-Filter erhält).
// Ein Schutz (z.B. ungespeicherte Eingaben) kann das Verlassen der Seite abfragen.
import { get, writable } from 'svelte/store';
import type { Filter } from './types';

export type SettingsTab = 'catalog' | 'backup' | 'import' | 'export' | 'about';

export type Route =
  | { page: 'start' }
  | { page: 'addresses'; filter?: Filter }
  | { page: 'organisation'; id: number | null }
  | { page: 'person'; id: number | null }
  | { page: 'teams' }
  | { page: 'groups' }
  | { page: 'settings'; tab?: SettingsTab };

export type LeaveGuard = () => Promise<boolean>;

const START: Route = { page: 'start' };

export const route = writable<Route>(START);
let history: Route[] = [];
let guard: LeaveGuard | null = null;

export function setLeaveGuard(next: LeaveGuard | null): void {
  guard = next;
}

async function mayLeave(): Promise<boolean> {
  return guard === null ? true : guard();
}

export async function navigate(next: Route): Promise<void> {
  if (!(await mayLeave())) return;
  guard = null;
  history = [...history, get(route)];
  route.set(next);
}

/** Ersetzt die aktuelle Seite ohne Verlaufseintrag und ohne Schutzabfrage (z.B. neu gespeichert). */
export function replace(next: Route): void {
  route.set(next);
}

export async function back(): Promise<void> {
  const previous = history[history.length - 1];
  if (!previous || !(await mayLeave())) return;
  guard = null;
  history = history.slice(0, -1);
  route.set(previous);
}

export function resetRouter(): void {
  history = [];
  guard = null;
  route.set(START);
}
