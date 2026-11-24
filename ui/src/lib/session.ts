// Sitzungszustand für Hinweisleiste und Neuladen der Ansichten.
import { derived, writable } from 'svelte/store';
import * as api from './api';
import { formatDateTime } from './dates';
import type { SessionInfo } from './types';

export const sessionInfo = writable<SessionInfo | null>(null);
/** Wird nach Datenänderungen erhöht; Seiten laden dann neu. */
export const dataVersion = writable(0);
/** Nur lesen – auch solange der Zustand noch unbekannt ist (sicherer Standard). */
export const readOnly = derived(sessionInfo, (info) => info?.readOnly ?? true);

export async function refreshSession(): Promise<void> {
  try {
    sessionInfo.set(await api.sessionInfo());
  } catch {
    sessionInfo.set(null);
  }
}

export function bumpData(): void {
  dataVersion.update((v) => v + 1);
}

export type Notice = { kind: 'readOnly' | 'backup'; text: string };

export function noticesFor(info: SessionInfo): Notice[] {
  const notices: Notice[] = [];
  if (info.readOnly) {
    const who = info.heldBy;
    const text = who
      ? `Nur lesen – geöffnet von ${who.user} (${who.host}) seit ${formatDateTime(who.since)}` +
        (who.likelyCrash ? ' – wahrscheinlich nach einem Absturz, bitte „Übernehmen“' : '')
      : 'Nur lesen';
    notices.push({ kind: 'readOnly', text });
  }
  if (info.backup.reminderDue) {
    const text = info.backup.secondaryDir
      ? 'Letzte Sicherung am zweiten Ort ist älter als 30 Tage'
      : 'Kein zweiter Sicherungsort eingerichtet';
    notices.push({ kind: 'backup', text });
  }
  return notices;
}

/**
 * Führt eine Schreibaktion aus. Erfolg: Ansichten neu laden. Fehler: Sitzung neu prüfen
 * (z.B. Sperre verloren → Hinweisleiste) und Fehler weitergeben.
 */
export async function runWrite<T>(action: () => Promise<T>): Promise<T> {
  try {
    const value = await action();
    bumpData();
    return value;
  } catch (err) {
    await refreshSession();
    throw err;
  }
}
