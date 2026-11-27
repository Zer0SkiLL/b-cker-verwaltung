// „Problem melden“: Aktionsliste (Ringpuffer) und Bericht. Es werden nur Namen von Seiten und
// Befehlen, Ergebnis, Dauer und Fehlermeldungen aufgezeichnet – nie Argumente oder Rückgabewerte.
import { writable } from 'svelte/store';
import { formatDateTime } from './dates';
import { logToFile } from './logFile';
import type { Route } from './router';
import type { RetentionPolicy, SessionInfo } from './types';

export type ActionEntry = {
  at: string;
  kind: 'page' | 'command' | 'error';
  name: string;
  ok?: boolean;
  ms?: number;
  message?: string;
};
export type SystemInfo = { os: string; arch: string; version: string };

export const MAX_ENTRIES = 50;
const STORAGE_KEY = 'mv.aktionen';
const REPORT_ACTIONS = 30;

/** Ob der Dialog „Problem melden“ offen ist. */
export const reportOpen = writable(false);

let memory: ActionEntry[] = load();

function load(): ActionEntry[] {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    const parsed = raw ? JSON.parse(raw) : [];
    return Array.isArray(parsed) ? parsed.slice(-MAX_ENTRIES) : [];
  } catch {
    return [];
  }
}

function push(entry: Omit<ActionEntry, 'at'>): void {
  memory = [...memory, { at: new Date().toISOString(), ...entry }].slice(-MAX_ENTRIES);
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(memory));
  } catch {
    // Kein Speicher (privat, voll): die Liste lebt dann nur im Programm.
  }
}

const PAGE_NAMES: Record<Route['page'], string> = {
  start: 'Start',
  addresses: 'Adressen',
  organisation: 'Betrieb (Detail)',
  person: 'Person (Detail)',
  teams: 'Teams',
  groups: 'Gruppen',
  settings: 'Einstellungen',
};
const TAB_NAMES: Record<string, string> = {
  catalog: 'Listen und Labels',
  backup: 'Datensicherung',
  import: 'Excel-Import',
  export: 'Gesamtexport',
  about: 'Über das Programm',
};

export function pageName(route: Route): string {
  if (route.page === 'settings') return `Einstellungen › ${TAB_NAMES[route.tab ?? 'catalog']}`;
  return PAGE_NAMES[route.page];
}

export function recordPage(route: Route): void {
  push({ kind: 'page', name: pageName(route) });
}

export function recordCommand(name: string, ok: boolean, ms: number, message?: string): void {
  push({ kind: 'command', name, ok, ms: Math.round(ms), message });
}

/** Unerwartete Fehler: nur Texte und Fehler-Objekte; andere Objekte werden nicht ausgegeben. */
export function recordError(reason: unknown): void {
  const message =
    reason instanceof Error
      ? `${reason.name}: ${reason.message}`
      : typeof reason === 'string'
        ? reason
        : `unbekannter Fehler (${Object.prototype.toString.call(reason)})`;
  push({ kind: 'error', name: 'Programmfehler', message });
  logToFile('error', `Oberfläche: ${message}`);
}

export function recentActions(limit = REPORT_ACTIONS): ActionEntry[] {
  return memory.slice(-limit);
}

export function clearActions(): void {
  memory = [];
  try {
    localStorage.removeItem(STORAGE_KEY);
  } catch {
    // ignorieren
  }
}

function time(iso: string): string {
  const d = new Date(iso);
  const pad = (n: number) => String(n).padStart(2, '0');
  return `${pad(d.getHours())}:${pad(d.getMinutes())}:${pad(d.getSeconds())}`;
}

function actionLine(a: ActionEntry): string {
  if (a.kind === 'page') return `${time(a.at)} Seite: ${a.name}`;
  if (a.kind === 'error') return `${time(a.at)} Fehler: ${a.message ?? ''}`;
  const result = a.ok ? `OK (${a.ms ?? 0} ms)` : `Fehler: ${a.message ?? ''}`;
  return `${time(a.at)} ${a.name} – ${result}`;
}

function fileState(session: SessionInfo | null): string {
  if (!session) return 'keine Datei';
  return session.readOnly ? 'nur lesend' : 'geöffnet, schreibend';
}

function secondaryState(session: SessionInfo | null): string {
  const b = session?.backup;
  if (!b) return '–';
  if (b.secondaryError) return `Fehler: ${b.secondaryError}`;
  if (!b.secondaryDir) return 'nicht eingerichtet';
  return `eingerichtet, letzte Sicherung ${b.lastSecondary ? formatDateTime(b.lastSecondary) : 'noch nie'}`;
}

export function buildReport(input: {
  description: string;
  now: Date;
  system: SystemInfo;
  route: Route;
  session: SessionInfo | null;
  retention: RetentionPolicy | null;
  actions: ActionEntry[];
  log: string[];
}): string {
  const r = input.retention;
  const retention = r
    ? `${r.keepAllDays}/${r.dailyDays}/${r.weeklyWeeks}/${r.monthlyMonths}, Intervall ${r.intervalMinutes} min`
    : '–';
  return [
    'Problembericht Mitgliederverwaltung',
    `Zeit: ${formatDateTime(input.now.toISOString())}`,
    `Version: ${input.system.version} · ${input.system.os} (${input.system.arch})`,
    '',
    'Was ist passiert?',
    input.description.trim() || '(keine Angabe)',
    '',
    'Zustand',
    `- Seite: ${pageName(input.route)}`,
    `- Datei: ${fileState(input.session)}`,
    `- Zweiter Sicherungsort: ${secondaryState(input.session)}`,
    `- Aufbewahrung: ${retention}`,
    '',
    'Letzte Aktionen (neueste zuletzt)',
    ...(input.actions.length ? input.actions.map(actionLine) : ['(keine)']),
    '',
    'Programm-Protokoll (letzte Einträge)',
    ...(input.log.length ? input.log : ['(keine)']),
  ].join('\n');
}
