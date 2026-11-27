// Frühwarnung: Hinweis nach einem Absturz und Befunde des Selbsttests. Alles hier scheitert still –
// die Arbeit im Programm geht immer vor.
import { writable } from 'svelte/store';
import * as api from './api';
import type { SelfTestDto, StartDto } from './types';

export type EarlyNotice = { kind: 'crash' | 'integrity' | 'lowSpace'; text: string };

export const CRASH_TEXT = 'Das Programm wurde letztes Mal unerwartet beendet.';
const INTEGRITY_TEXT = 'Die Datei hat einen Fehler. Bitte melden Sie das Problem.';
const LOW_SPACE_TEXT = 'Auf dem Laufwerk der Datei ist nur noch wenig Platz frei.';
const SEEN_KEY = 'diagnose-absturz-gesehen';

export const earlyNotices = writable<EarlyNotice[]>([]);

function readSeen(): string | null {
  try {
    return localStorage.getItem(SEEN_KEY);
  } catch {
    return null;
  }
}

function writeSeen(at: string): void {
  try {
    localStorage.setItem(SEEN_KEY, at);
  } catch {
    // Ohne Speicher erscheint der Hinweis beim nächsten Start noch einmal.
  }
}

/** Neuester Start, vor dem das Programm nicht sauber endete, sofern noch nicht ausgeblendet. */
export function crashNotice(starts: StartDto[], dismissed: string | null): StartDto | null {
  const latest = starts[0];
  return latest && latest.cleanBefore === false && latest.at !== dismissed ? latest : null;
}

let crashAt: string | null = null;

function setNotice(kind: EarlyNotice['kind'], text: string | null): void {
  earlyNotices.update((list) => {
    const rest = list.filter((n) => n.kind !== kind);
    return text ? [...rest, { kind, text }] : rest;
  });
}

/** Beim Start: war der vorige Lauf ein Absturz? Gibt den betroffenen Start zurück. */
export async function checkCrash(): Promise<StartDto | null> {
  try {
    const snapshot = await api.diagnosticsSnapshot();
    const crash = crashNotice(snapshot.starts, readSeen());
    crashAt = crash?.at ?? null;
    setNotice('crash', crash ? CRASH_TEXT : null);
    return crash;
  } catch {
    return null;
  }
}

export function dismissNotice(kind: EarlyNotice['kind']): void {
  if (kind === 'crash' && crashAt) writeSeen(crashAt);
  setNotice(kind, null);
}

/** Nach dem Öffnen einer Datei; zeigt nur Befunde, die der Nutzer selbst angehen kann. */
export async function runSelfTest(): Promise<SelfTestDto | null> {
  try {
    const result = await api.selfTest();
    setNotice('integrity', result.integrityProblem ? INTEGRITY_TEXT : null);
    setNotice('lowSpace', result.lowSpace ? LOW_SPACE_TEXT : null);
    return result;
  } catch {
    return null;
  }
}
