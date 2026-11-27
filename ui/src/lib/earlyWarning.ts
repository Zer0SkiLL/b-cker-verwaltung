// Frühwarnung: Hinweis nach einem Absturz und Befunde des Selbsttests. Alles hier scheitert still –
// die Arbeit im Programm geht immer vor.
import { get, writable } from 'svelte/store';
import * as api from './api';
import { buildReport, recentActions } from './diagnostics';
import { route } from './router';
import { sessionInfo } from './session';
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

// --- Automatische Berichte (Diagnose-Ordner und Analysedaten) ---

export type EventKind = 'absturz' | 'warnung' | 'selbsttest' | 'lebenszeichen';

const KIND_LABEL: Record<EventKind, string> = {
  absturz: 'Absturz',
  warnung: 'Warnung',
  selbsttest: 'Selbsttest',
  lebenszeichen: 'Lebenszeichen',
};
/** Rangfolge, wenn im Sammelfenster mehrere Arten zusammenkommen. */
const RANK: Record<EventKind, number> = { lebenszeichen: 0, warnung: 1, selbsttest: 2, absturz: 3 };

export const WINDOW_MS = 60_000;
const SENT_KEY = 'diagnose-absturz-gesendet';

/** Erstes Ereignis öffnet ein Fenster von 60 s; danach ein gemeinsamer Bericht. Absturz sofort. */
export function createCollector(publish: (kind: EventKind) => void, setTimer = setTimeout, clearTimer = clearTimeout) {
  let pending: EventKind | null = null;
  let timer: ReturnType<typeof setTimeout> | null = null;
  return (kind: EventKind) => {
    if (kind === 'absturz') {
      if (timer) clearTimer(timer);
      timer = null;
      pending = null;
      publish('absturz');
      return;
    }
    pending = pending && RANK[pending] >= RANK[kind] ? pending : kind;
    if (timer) return;
    timer = setTimer(() => {
      const k = pending ?? kind;
      pending = null;
      timer = null;
      publish(k);
    }, WINDOW_MS);
  };
}

/** Abschnitte STATUS und COMPUTER (für die Discord-Nachricht). */
export function statusBlock(report: string): string {
  const lines = report.split('\n');
  const from = lines.indexOf('  STATUS AUF EINEN BLICK');
  const to = lines.indexOf('  LAUFZEITEN');
  if (from < 0) return '';
  const block = lines.slice(from, to > from ? to : undefined);
  while (block.length && block[block.length - 1] === '') block.pop();
  return block.join('\n');
}

/** Bericht wie in „Problem melden“, mit Beschreibung „(automatisch: …)“. */
export async function currentReport(description: string): Promise<{ report: string; statusBlock: string }> {
  const session = get(sessionInfo);
  const [snapshot, retention] = await Promise.all([
    api.diagnosticsSnapshot().catch(() => null),
    session ? api.getRetentionQuiet().catch(() => null) : Promise.resolve(null),
  ]);
  const report = buildReport({
    description,
    now: new Date(),
    route: get(route),
    session,
    retention,
    actions: recentActions(),
    snapshot,
  });
  return { report, statusBlock: statusBlock(report) };
}

export async function autoReport(kind: EventKind): Promise<void> {
  try {
    const { report, statusBlock: block } = await currentReport(`(automatisch: ${KIND_LABEL[kind]})`);
    await api.diagnosticsPublish(kind, block, report);
  } catch {
    // Frühwarnung darf nie stören.
  }
}

/** Diagnose-Ordner nach dem Öffnen einer Datei aktualisieren (nichts wird gesendet). */
export async function writeFolderNow(): Promise<void> {
  try {
    const { report } = await currentReport('(automatisch: Datei geöffnet)');
    await api.diagnosticsWriteFolder(report);
  } catch {
    // still
  }
}

function readSent(): string | null {
  try {
    return localStorage.getItem(SENT_KEY);
  } catch {
    return null;
  }
}

function writeSent(at: string): void {
  try {
    localStorage.setItem(SENT_KEY, at);
  } catch {
    // dann höchstens ein zweites Mal
  }
}

let listening = false;
const collect = createCollector((kind) => void autoReport(kind));

/** Beim Programmstart: Absturz-Hinweis, einmalige Meldung, Lebenszeichen, Ereignisse abonnieren. */
export async function startEarlyWarning(): Promise<void> {
  const crash = await checkCrash();
  if (crash && crash.at !== readSent()) {
    writeSent(crash.at);
    await autoReport('absturz');
  } else if (await api.diagnoseHeartbeatDue().catch(() => false)) {
    await autoReport('lebenszeichen');
  }
  if (listening) return;
  listening = true;
  try {
    const { listen } = await import('@tauri-apps/api/event');
    await listen<{ kind: EventKind }>('diagnose-ereignis', (e) => collect(e.payload.kind));
  } catch {
    listening = false;
  }
}

let checkedPath: string | null = null;

/** Nach dem Öffnen: Selbsttest und Diagnose-Ordner, einmal pro Datei (auch bei mehrfacher Meldung). */
export async function checkOpenedFile(path: string): Promise<void> {
  if (path === checkedPath) return;
  checkedPath = path;
  await runSelfTest();
  await writeFolderNow();
}
