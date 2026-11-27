import { bar, bytesMb, bytesGb, duration, frame, shortDateTime, statusLines, type StatusInput } from './reportLayout';
import type { SnapshotDto } from './types';

test('bar rundet auf halbe Zeichen', () => {
  expect(bar(0.12)).toBe('██▌' + '░'.repeat(17));
  expect(bar(0)).toBe('░'.repeat(20));
  expect(bar(1)).toBe('█'.repeat(20));
  expect(bar(0.0001)).toBe('▌' + '░'.repeat(19));
  expect(bar(null)).toBe('░'.repeat(20));
  expect(bar(2)).toBe('█'.repeat(20));
  for (const r of [0.33, 0.5, 0.77]) expect([...bar(r)].length).toBe(20);
});

test('duration in Schweizer Kurzform', () => {
  expect(duration(45)).toBe('45 s');
  expect(duration(600)).toBe('10 min');
  expect(duration(23460)).toBe('6 h 31 min');
  expect(duration(194400)).toBe('2 Tage 6 h');
  expect(duration(97200)).toBe('1 Tag 3 h');
});

test('frame: alle Zeilen gleich lang', () => {
  const lines = frame(['kurz', 'etwas länger']);
  expect(lines[0]).toBe('╔' + '═'.repeat(16) + '╗');
  expect(lines[1]).toBe('║  kurz          ║');
  expect(new Set(lines.map((l) => [...l].length)).size).toBe(1);
});

test('Zahlen und Zeiten', () => {
  expect(bytesGb(9.1 * 1024 ** 3)).toBe('9.1');
  expect(bytesMb(310 * 1024 ** 2)).toBe('310 MB');
  expect(bytesMb(2.4 * 1024 ** 2, 1)).toBe('2.4 MB');
  expect(shortDateTime(new Date(2026, 10, 27, 8, 1).toISOString())).toBe('27.11. 08:01');
});

const snapshot: SnapshotDto = {
  system: { os: 'Windows 11', arch: 'x86_64', version: '0.6.0' },
  host: 'PC-SEKRETARIAT',
  resources: {
    cpuTotal: 12, cpuProgram: 1, memUsed: 9e9, memTotal: 16e9, memProgram: 310e6, memProgramPeak: 420e6,
    programSince: new Date(2026, 10, 27, 8, 1).toISOString(), computerSince: new Date(2026, 10, 25, 7, 45).toISOString(),
  },
  file: {
    location: 'Netzlaufwerk (S:)', reachable: true, size: 2_500_000, schemaVersion: 7, organisations: 120, persons: 340,
    activeMembers: 410, lastLocalBackup: new Date(2026, 10, 27, 14, 0).toISOString(), lockHost: null, lockSince: null, lockLikelyCrash: false,
  },
  starts: [{ at: new Date(2026, 10, 27, 8, 1).toISOString(), cleanBefore: true }],
  problemsToday: 0,
  log: [],
};
const session = {
  path: '/geheim/v.sqlite', readOnly: false, heldBy: null,
  backup: { secondaryDir: 'E:\\', lastSecondary: new Date(2026, 10, 27, 14, 0).toISOString(), reminderDue: false, secondaryError: null },
};
const status = (over: Partial<StatusInput> = {}) => statusLines({ session, snapshot, ...over });
const line = (over: Partial<StatusInput>, label: string) => status(over).find((s) => s.label === label);

test('alles in Ordnung', () => {
  const lines = status();
  expect(lines.map((l) => l.label)).toEqual(['Datei', 'Schreibzugriff', 'Sicherung', 'Letzter Start', 'Arbeitsspeicher', 'Protokoll']);
  expect(lines.every((l) => l.status === 'ok')).toBe(true);
  expect(lines[0].text).toBe('Netzlaufwerk (S:)');
  expect(lines[2].text).toBe('neben der Datei 27.11. 14:00 · zweiter Ort 27.11. 14:00');
});

test('nur lesend nennt den PC, nie den Benutzer', () => {
  const ro = {
    session: { ...session, readOnly: true, heldBy: { user: 'hans', host: 'PC-SEKRETARIAT', since: '', likelyCrash: true } },
    snapshot: { ...snapshot, file: { ...snapshot.file!, lockHost: 'PC-SEKRETARIAT', lockSince: new Date(2026, 10, 27, 8, 2).toISOString(), lockLikelyCrash: true } },
  };
  const l = line(ro, 'Schreibzugriff')!;
  expect(l.status).toBe('warn');
  expect(l.text).toBe('nur lesend – gesperrt von PC-SEKRETARIAT seit 27.11. 08:02 (vermutlich nach Absturz)');
  expect(JSON.stringify(statusLines({ ...ro }))).not.toContain('hans');
});

test('Warnungen nach Spec', () => {
  expect(line({ session: { ...session, backup: { ...session.backup, secondaryError: 'weg' } } }, 'Sicherung')!.status).toBe('warn');
  expect(line({ session: { ...session, backup: { ...session.backup, reminderDue: true } } }, 'Sicherung')!.status).toBe('warn');
  expect(line({ snapshot: { ...snapshot, file: { ...snapshot.file!, lastLocalBackup: null } } }, 'Sicherung')!.text).toContain('neben der Datei noch keine');
  expect(line({ snapshot: { ...snapshot, starts: [{ at: snapshot.starts[0].at, cleanBefore: false }] } }, 'Letzter Start')).toMatchObject({ status: 'warn', text: 'vorher nicht sauber beendet' });
  expect(line({ snapshot: { ...snapshot, starts: [{ at: snapshot.starts[0].at, cleanBefore: null }] } }, 'Letzter Start')!.status).toBe('none');
  expect(line({ snapshot: { ...snapshot, resources: { ...snapshot.resources, memProgram: 1024 ** 3 } } }, 'Arbeitsspeicher')!.status).toBe('warn');
  expect(line({ snapshot: { ...snapshot, problemsToday: 3 } }, 'Protokoll')).toMatchObject({ status: 'warn', text: '3 Warnungen/Fehler heute' });
  expect(line({ snapshot: { ...snapshot, file: { ...snapshot.file!, reachable: false } } }, 'Datei')).toMatchObject({ status: 'warn', text: 'nicht erreichbar' });
});

test('ohne Datei und ohne Momentaufnahme', () => {
  const lines = status({ session: null, snapshot: null });
  expect(lines[0]).toEqual({ status: 'none', label: 'Datei', text: 'keine Datei geöffnet' });
  expect(lines.map((l) => l.label)).toEqual(['Datei', 'Letzter Start', 'Arbeitsspeicher', 'Protokoll']);
  expect(lines.slice(1).every((l) => l.status === 'none' && l.text === 'unbekannt')).toBe(true);
});
