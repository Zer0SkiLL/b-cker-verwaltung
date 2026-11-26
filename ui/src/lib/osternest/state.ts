// Osternest: versteckte Überraschungen fürs Sekretariat (bewusst nicht im Handbuch). Siehe docs/superpowers/specs/2026-11-26-osternest-design.md
// Bäckermeister-Modus (7 Klicks auf die Version) und das Zeichen neben dem Namen in der linken Leiste.
import { readable, writable } from 'svelte/store';
import { specialDay } from './calendar';

const KEY = 'osternest-baeckermeister';
export const VERSION_CLICKS = 7;

const BAKER_BADGE = { icon: '🥨', text: 'Bäckermeister-Modus. Gut Teig!' };

function load(): boolean {
  try {
    return localStorage.getItem(KEY) === '1';
  } catch {
    return false;
  }
}

export const bakerMode = writable<boolean>(load());

export function setBakerMode(on: boolean): void {
  bakerMode.set(on);
  try {
    localStorage.setItem(KEY, on ? '1' : '0');
  } catch {
    // Ohne Speicher gilt der Modus nur bis zum Neustart.
  }
}

/** Aktuelle Zeit, jede Minute neu; so wechselt das Zeichen auch, wenn das Programm über Mitternacht läuft. */
export const now = readable<Date>(new Date(), (set) => {
  set(new Date());
  const timer = setInterval(() => set(new Date()), 60_000);
  return () => clearInterval(timer);
});

export function countVersionClick(count: number): { count: number; toggled: boolean } {
  const next = count + 1;
  return next >= VERSION_CLICKS ? { count: 0, toggled: true } : { count: next, toggled: false };
}

/** Ein besonderer Tag hat Vorrang vor der Brezel des Bäckermeister-Modus. */
export function badge(date: Date, baker: boolean): { icon: string; text: string } | null {
  const day = specialDay(date);
  if (day) return { icon: day.icon, text: day.text };
  return baker ? BAKER_BADGE : null;
}
