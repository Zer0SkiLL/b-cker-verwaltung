// Osternest: versteckte Überraschungen fürs Sekretariat (bewusst nicht im Handbuch). Siehe docs/superpowers/specs/2026-11-26-osternest-design.md
// Meilenstein: Nach einer Neuaufnahme, die eine neue volle Hundertermarke erreicht, fällt kurz Brot vom Himmel.
// Gemerkt wird nur die höchste erreichte Marke, pro PC; die Vereinsdatei bleibt unberührt.
import { writable } from 'svelte/store';
import { searchAddresses } from '../api';
import { emptyFilter } from '../filters';

const KEY = 'osternest-meilenstein';
export const RAIN_MS = 3000;

/** Gefeierte Zahl, solange der Brot-Regen läuft; sonst null. */
export const rain = writable<number | null>(null);

/** Erste Zählung (stored = null, etwa nach dem Excel-Import) feiert nicht; Austritte feiern nie nochmal. */
export function nextMilestone(stored: number | null, count: number): { store: number; celebrate: number | null } {
  const mark = Math.floor(count / 100) * 100;
  if (stored === null) return { store: mark, celebrate: null };
  if (mark > stored && mark >= 100) return { store: mark, celebrate: mark };
  return { store: Math.max(stored, mark), celebrate: null };
}

function loadStored(): number | null {
  const raw = localStorage.getItem(KEY);
  const value = raw === null ? Number.NaN : Number.parseInt(raw, 10);
  return Number.isFinite(value) ? value : null;
}

/** Wird nach einer Neuaufnahme aufgerufen. Fehler bleiben still: Ein Easter Egg darf nie stören. */
export async function checkMilestone(countActive: () => Promise<number>): Promise<void> {
  try {
    const count = await countActive();
    const result = nextMilestone(loadStored(), count);
    localStorage.setItem(KEY, String(result.store));
    if (result.celebrate !== null) rain.set(result.celebrate);
  } catch {
    // Ohne Zählung oder Speicher gibt es eben keinen Regen.
  }
}

export async function countActiveMembers(): Promise<number> {
  const result = await searchAddresses({ ...emptyFilter(), membership: 'Active' });
  return result.rows.length;
}
