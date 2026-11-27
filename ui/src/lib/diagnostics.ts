// „Problem melden“: Aktionsliste (Ringpuffer) und Bericht. Es werden nur Namen von Seiten und
// Befehlen, Ergebnis, Dauer und Fehlermeldungen aufgezeichnet – nie Argumente oder Rückgabewerte.
import { writable } from 'svelte/store';
import { formatDateTime } from './dates';
import { logToFile } from './logFile';
import type { Route } from './router';
import { bar, bytesGb, bytesMb, duration, formatStatus, frame, shortDateTime, statusLines } from './reportLayout';
import type { RetentionPolicy, SessionInfo, SnapshotDto } from './types';

export type ActionEntry = {
  at: string;
  kind: 'page' | 'command' | 'error';
  name: string;
  ok?: boolean;
  ms?: number;
  message?: string;
};

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
  logToFile('error', `Oberfläche: ${sanitize(message)}`);
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

// Wie `sanitize` in Rust (crates/app/src/commands/logfile.rs): absolute Pfade und den Namen des
// Sperr-Inhabers entfernen, bevor Texte den PC verlassen.
const PATH = /(^|[^A-Za-z0-9])((?:[A-Za-z]:[\\/]|\\\\|\/[\w.~-]+\/)(?:[^"'“”‘’«»<>|():,;\n]|[:,;][^ \n])*)/g;
const LOCK_HOLDER = /(bereits von ).+?( auf )/g;

export function sanitize(text: string): string {
  return text.replace(PATH, '$1‹Pfad›').replace(LOCK_HOLDER, '$1‹Person›$2').trimEnd();
}

function actionLine(a: ActionEntry): string {
  if (a.kind === 'page') return `${time(a.at)} Seite: ${a.name}`;
  if (a.kind === 'error') return `${time(a.at)} Fehler: ${sanitize(a.message ?? '')}`;
  const result = a.ok ? `OK (${a.ms ?? 0} ms)` : `Fehler: ${sanitize(a.message ?? '')}`;
  return `${time(a.at)} ${a.name} – ${result}`;
}

const INDENT = '  ';
const UNKNOWN = 'unbekannt';

function percent(value: number | null): string {
  return value === null ? UNKNOWN : `${Math.round(value)} %`;
}

function since(now: Date, rfc3339: string | null): string {
  if (!rfc3339) return UNKNOWN;
  const seconds = (now.getTime() - new Date(rfc3339).getTime()) / 1000;
  return `seit ${shortDateTime(rfc3339)}   ${duration(seconds)}`;
}

function computerLines(snapshot: SnapshotDto | null): string[] {
  const r = snapshot?.resources;
  const total = r?.memTotal ?? null;
  const ratio = (part: number | null | undefined) => (part != null && total ? part / total : null);
  const cpu = r?.cpuTotal != null ? `${percent(r.cpuTotal)} (Programm ${percent(r.cpuProgram)})` : UNKNOWN;
  const mem = r?.memUsed != null && total ? `${bytesGb(r.memUsed)} / ${bytesGb(total)} GB` : UNKNOWN;
  const peak = r?.memProgramPeak != null ? ` (höchster Wert ${bytesMb(r.memProgramPeak)})` : '';
  const program = r?.memProgram != null ? `${bytesMb(r.memProgram)}${peak}` : UNKNOWN;
  return [
    `${INDENT}COMPUTER`,
    `${INDENT}Prozessor  ${bar(r?.cpuTotal != null ? r.cpuTotal / 100 : null)}  ${cpu}`,
    `${INDENT}Speicher   ${bar(ratio(r?.memUsed))}  ${mem}`,
    `${INDENT}Programm   ${bar(ratio(r?.memProgram))}  ${program}`,
  ];
}

/** Neueste zuerst; „(!)“ markiert einen Lauf, der nicht sauber endete. */
function startsText(snapshot: SnapshotDto | null): string {
  const starts = snapshot?.starts ?? [];
  if (!starts.length) return UNKNOWN;
  return starts
    .map((s, i) => `${shortDateTime(s.at)}${starts[i - 1]?.cleanBefore === false ? ' (!)' : ''}`)
    .join(' · ');
}

function runtimeLines(now: Date, snapshot: SnapshotDto | null): string[] {
  return [
    `${INDENT}LAUFZEITEN`,
    `${INDENT}Programm   ${since(now, snapshot?.resources.programSince ?? null)}`,
    `${INDENT}Computer   ${since(now, snapshot?.resources.computerSince ?? null)}`,
    `${INDENT}Starts     ${startsText(snapshot)}`,
  ];
}

function retentionText(r: RetentionPolicy | null): string {
  return r ? `${r.keepAllDays}/${r.dailyDays}/${r.weeklyWeeks}/${r.monthlyMonths}, alle ${r.intervalMinutes} min` : UNKNOWN;
}

function fileLines(session: SessionInfo | null, snapshot: SnapshotDto | null, retention: RetentionPolicy | null): string[] {
  if (!session) return [];
  const f = snapshot?.file;
  const n = (v: number | null | undefined) => (v == null ? '?' : String(v));
  const size = f?.size != null ? bytesMb(f.size, 1) : UNKNOWN;
  return [
    `${INDENT}DATEI`,
    `${INDENT}${size} · Datenbank-Version ${n(f?.schemaVersion)} · Aufbewahrung ${retentionText(retention)}`,
    `${INDENT}${n(f?.organisations)} Betriebe · ${n(f?.persons)} Personen · ${n(f?.activeMembers)} aktive Mitglieder`,
    '',
  ];
}

function description(text: string): string {
  const lines = (text.trim() || '(keine Angabe)').split('\n');
  return [`» ${lines[0]}`, ...lines.slice(1).map((l) => `${INDENT}${l}`)].join('\n');
}

function listOrNone(lines: string[]): string[] {
  return (lines.length ? lines : ['(keine)']).map((l) => `${INDENT}${l}`);
}

export function buildReport(input: {
  description: string;
  now: Date;
  route: Route;
  session: SessionInfo | null;
  retention: RetentionPolicy | null;
  actions: ActionEntry[];
  snapshot: SnapshotDto | null;
}): string {
  const { now, session, snapshot } = input;
  const system = snapshot?.system ?? { os: UNKNOWN, arch: UNKNOWN, version: UNKNOWN };
  const header = frame([
    `PROBLEMBERICHT · Mitgliederverwaltung ${system.version}`,
    `${formatDateTime(now.toISOString())} · ${system.os} (${system.arch})`,
  ]);
  return [
    ...header,
    '',
    description(input.description),
    '',
    `${INDENT}STATUS AUF EINEN BLICK`,
    ...statusLines({ session, snapshot }).map(formatStatus),
    '',
    ...computerLines(snapshot),
    '',
    ...runtimeLines(now, snapshot),
    '',
    ...fileLines(session, snapshot, input.retention),
    `${INDENT}SEITE`,
    `${INDENT}${pageName(input.route)}`,
    '',
    'LETZTE AKTIONEN (neueste unten)',
    ...listOrNone(input.actions.map(actionLine)),
    '',
    'PROTOKOLL (neueste unten)',
    ...listOrNone(snapshot?.log ?? []),
  ].join('\n');
}
