// Bausteine für den Problembericht im Dashboard-Layout: Balken, Dauer, Rahmen, Statuszeilen.
import type { SessionInfo, SnapshotDto } from './types';

export const BAR_WIDTH = 20;
/** Ab hier gilt der Speicherverbrauch des Programms als auffällig. */
export const MEM_WARN_BYTES = 1024 ** 3;

const MB = 1024 ** 2;
const GB = 1024 ** 3;

/** Anteil als Balken; auf halbe Zeichen gerundet, kleinster sichtbarer Wert ist ein halbes Zeichen. */
export function bar(ratio: number | null, width = BAR_WIDTH): string {
  const r = ratio === null || !Number.isFinite(ratio) ? 0 : Math.min(Math.max(ratio, 0), 1);
  let halves = Math.round(r * width * 2);
  if (r > 0 && halves === 0) halves = 1;
  const full = Math.floor(halves / 2);
  const half = halves % 2;
  return '█'.repeat(full) + '▌'.repeat(half) + '░'.repeat(width - full - half);
}

/** 45 s · 10 min · 6 h 31 min · 1 Tag 3 h · 2 Tage 6 h */
export function duration(seconds: number): string {
  const s = Math.max(0, Math.round(seconds));
  if (s < 60) return `${s} s`;
  const minutes = Math.floor(s / 60) % 60;
  const hours = Math.floor(s / 3600) % 24;
  const days = Math.floor(s / 86400);
  if (days > 0) return `${days} ${days === 1 ? 'Tag' : 'Tage'} ${hours} h`;
  if (hours > 0) return `${hours} h ${minutes} min`;
  return `${minutes} min`;
}

/** Doppelrahmen um die Zeilen; Breite = längste Zeile + 4. */
export function frame(lines: string[]): string[] {
  const width = Math.max(...lines.map((l) => [...l].length)) + 4;
  const pad = (l: string) => `  ${l}${' '.repeat(width - 2 - [...l].length)}`;
  return ['╔' + '═'.repeat(width) + '╗', ...lines.map((l) => `║${pad(l)}║`), '╚' + '═'.repeat(width) + '╝'];
}

export function bytesGb(bytes: number): string {
  return (bytes / GB).toFixed(1);
}

export function bytesMb(bytes: number, decimals = 0): string {
  return `${(bytes / MB).toFixed(decimals)} MB`;
}

const pad2 = (n: number) => String(n).padStart(2, '0');

/** 27.11. 08:01 */
export function shortDateTime(rfc3339: string): string {
  const t = new Date(rfc3339);
  return `${pad2(t.getDate())}.${pad2(t.getMonth() + 1)}. ${pad2(t.getHours())}:${pad2(t.getMinutes())}`;
}

export type Status = 'ok' | 'warn' | 'none';
export type StatusLine = { status: Status; label: string; text: string };
export type StatusInput = { session: SessionInfo | null; snapshot: SnapshotDto | null };

const UNKNOWN = 'unbekannt';

function fileLine({ session, snapshot }: StatusInput): StatusLine {
  if (!session) return { status: 'none', label: 'Datei', text: 'keine Datei geöffnet' };
  const file = snapshot?.file;
  if (file && !file.reachable) return { status: 'warn', label: 'Datei', text: 'nicht erreichbar' };
  return { status: 'ok', label: 'Datei', text: file?.location ?? UNKNOWN };
}

function accessLine(session: SessionInfo, snapshot: SnapshotDto | null): StatusLine {
  if (!session.readOnly) return { status: 'ok', label: 'Schreibzugriff', text: 'schreibend' };
  const file = snapshot?.file;
  const holder = file?.lockHost
    ? ` – gesperrt von ${file.lockHost}${file.lockSince ? ` seit ${shortDateTime(file.lockSince)}` : ''}`
    : '';
  const crash = file?.lockLikelyCrash ? ' (vermutlich nach Absturz)' : '';
  return { status: 'warn', label: 'Schreibzugriff', text: `nur lesend${holder}${crash}` };
}

function backupLine(session: SessionInfo, snapshot: SnapshotDto | null): StatusLine {
  const b = session.backup;
  const local = snapshot?.file?.lastLocalBackup;
  const localText = snapshot?.file ? (local ? shortDateTime(local) : 'noch keine') : UNKNOWN;
  const secondary = b.secondaryError
    ? `Fehler: ${b.secondaryError}`
    : !b.secondaryDir
      ? 'nicht eingerichtet'
      : b.lastSecondary
        ? shortDateTime(b.lastSecondary)
        : 'noch nie';
  const warn = Boolean(b.secondaryError) || b.reminderDue || (snapshot?.file != null && !local);
  return { status: warn ? 'warn' : 'ok', label: 'Sicherung', text: `neben der Datei ${localText} · zweiter Ort ${secondary}` };
}

function startLine(snapshot: SnapshotDto | null): StatusLine {
  const label = 'Letzter Start';
  if (!snapshot) return { status: 'none', label, text: UNKNOWN };
  const clean = snapshot.starts[0]?.cleanBefore ?? null;
  if (clean === null) return { status: 'none', label, text: 'erster Start im Protokoll' };
  return clean
    ? { status: 'ok', label, text: 'vorher sauber beendet' }
    : { status: 'warn', label, text: 'vorher nicht sauber beendet' };
}

function memoryLine(snapshot: SnapshotDto | null): StatusLine {
  const label = 'Arbeitsspeicher';
  const mem = snapshot?.resources.memProgram ?? null;
  if (mem === null) return { status: 'none', label, text: UNKNOWN };
  return { status: mem >= MEM_WARN_BYTES ? 'warn' : 'ok', label, text: `Programm ${bytesMb(mem)}` };
}

function logLine(snapshot: SnapshotDto | null): StatusLine {
  const label = 'Protokoll';
  if (!snapshot) return { status: 'none', label, text: UNKNOWN };
  const n = snapshot.problemsToday;
  return n > 0
    ? { status: 'warn', label, text: `${n} Warnungen/Fehler heute` }
    : { status: 'ok', label, text: 'keine Warnungen/Fehler heute' };
}

/** Status auf einen Blick; ohne Datei entfallen Schreibzugriff und Sicherung. */
export function statusLines(input: StatusInput): StatusLine[] {
  const { session, snapshot } = input;
  const fileLines = session ? [accessLine(session, snapshot), backupLine(session, snapshot)] : [];
  return [fileLine(input), ...fileLines, startLine(snapshot), memoryLine(snapshot), logLine(snapshot)];
}

const MARK: Record<Status, string> = { ok: '[ OK ]', warn: '[ !! ]', none: '[ -- ]' };

export function formatStatus(line: StatusLine): string {
  return `  ${MARK[line.status]} ${line.label.padEnd(19)}${line.text}`;
}
