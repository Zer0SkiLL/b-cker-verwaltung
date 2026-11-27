import { vi } from 'vitest';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(async (command: string) => {
    if (command === 'create_label') throw 'Name ist bereits vergeben';
    return { id: 1, name: 'Bäckerei Geheim' };
  }),
}));

import * as api from './api';
import {
  buildReport, clearActions, MAX_ENTRIES, recentActions, recordCommand, recordError, recordPage, reportOpen, sanitize,
} from './diagnostics';
import type { SnapshotDto } from './types';

const snapshot: SnapshotDto = {
  system: { os: 'Windows 11', arch: 'x86_64', version: '0.3.0' },
  host: 'PC-TEST',
  resources: {
    cpuTotal: 12, cpuProgram: 1, memUsed: 9.1 * 1024 ** 3, memTotal: 16 * 1024 ** 3, memProgram: 310 * 1024 ** 2,
    memProgramPeak: 420 * 1024 ** 2, programSince: new Date(2026, 10, 25, 8, 1).toISOString(),
    computerSince: new Date(2026, 10, 23, 7, 45).toISOString(),
  },
  file: {
    location: 'Netzlaufwerk (S:)', reachable: true, size: 2.4 * 1024 ** 2, schemaVersion: 7, organisations: 120, persons: 340,
    activeMembers: 410, lastLocalBackup: new Date(2026, 10, 25, 14, 0).toISOString(), lockHost: null, lockSince: null, lockLikelyCrash: false,
  },
  starts: [
    { at: new Date(2026, 10, 25, 8, 1).toISOString(), cleanBefore: false },
    { at: new Date(2026, 10, 24, 21, 33).toISOString(), cleanBefore: true },
    { at: new Date(2026, 10, 24, 7, 58).toISOString(), cleanBefore: null },
  ],
  problemsToday: 1,
  log: ['25.11. 14:00:00  WARNUNG  Zweiter Sicherungsort nicht erreichbar: os error 2'],
};
const session = {
  path: '/x/v.sqlite', readOnly: false, heldBy: null,
  backup: { secondaryDir: 'E:\\', lastSecondary: '2026-11-25T12:10:00Z', reminderDue: false, secondaryError: null },
};
const retention = { keepAllDays: 2, dailyDays: 14, weeklyWeeks: 8, monthlyMonths: 12, intervalMinutes: 60 };
const report = (over: Partial<Parameters<typeof buildReport>[0]> = {}) =>
  buildReport({
    description: '', now: new Date(2026, 10, 25, 14, 31), route: { page: 'addresses' },
    session, retention, actions: recentActions(), snapshot, ...over,
  });

beforeEach(() => clearActions());

test('Ringpuffer hält höchstens 50 Einträge', () => {
  for (let i = 0; i < 60; i += 1) recordCommand(`c${i}`, true, 1);
  const all = recentActions(100);
  expect(all).toHaveLength(MAX_ENTRIES);
  expect(all[0].name).toBe('c10');
  expect(all[all.length - 1].name).toBe('c59');
});

test('werfendes localStorage stört nicht', () => {
  const spy = vi.spyOn(Storage.prototype, 'setItem').mockImplementation(() => {
    throw new Error('voll');
  });
  expect(() => recordCommand('x', true, 1)).not.toThrow();
  expect(recentActions().slice(-1)[0].name).toBe('x');
  spy.mockRestore();
});

test('Seitennamen ohne IDs', () => {
  recordPage({ page: 'organisation', id: 42 });
  recordPage({ page: 'settings', tab: 'backup' });
  const names = recentActions().map((a) => a.name);
  expect(names).toEqual(['Betrieb (Detail)', 'Einstellungen › Datensicherung']);
});

test('Datenschutz: keine Argumente oder Rückgabewerte im Bericht', async () => {
  await api.createOrganisation({ name: 'Bäckerei Geheim', email: 'x@y.ch' } as never);
  await api.searchAddresses({ text: 'Geheim' } as never);
  await expect(api.createLabel(1, 'Geheim-Label')).rejects.toThrow('Name ist bereits vergeben');
  const text = report();
  expect(text).not.toContain('Geheim');
  expect(text).not.toContain('x@y.ch');
  expect(text).toMatch(/create_organisation – OK \(\d+ ms\)/);
  expect(text).toContain('create_label – Fehler: Name ist bereits vergeben');
});

test('unerwartete Fehler: Objekte werden nicht ausgegeben', () => {
  recordError({ name: 'Geheim' });
  recordError(new TypeError('x is undefined'));
  const text = report();
  expect(text).not.toContain('Geheim');
  expect(text).toContain('Fehler: TypeError: x is undefined');
});

test('Bericht im Dashboard-Layout, Abschnitte in fester Reihenfolge', () => {
  const text = report({ description: 'Speichern ging nicht' });
  const lines = text.split('\n');
  expect(lines[0]).toMatch(/^╔═+╗$/);
  expect(lines[1]).toMatch(/^║  PROBLEMBERICHT · Mitgliederverwaltung 0\.3\.0 +║$/);
  expect(lines[2]).toMatch(/^║  25\.11\.2026 14:31 · Windows 11 \(x86_64\) +║$/);
  expect(text).toContain('» Speichern ging nicht');
  const order = ['STATUS AUF EINEN BLICK', 'COMPUTER', 'LAUFZEITEN', 'DATEI', 'SEITE', 'LETZTE AKTIONEN (neueste unten)', 'PROTOKOLL (neueste unten)'];
  const positions = order.map((h) => lines.indexOf(h.startsWith('LETZTE') || h.startsWith('PROTOKOLL') ? h : `  ${h}`));
  expect(positions.every((p) => p > 0)).toBe(true);
  expect([...positions].sort((a, b) => a - b)).toEqual(positions);
  expect(text).toContain('  [ OK ] Datei              Netzlaufwerk (S:)');
  expect(text).toContain('  [ !! ] Letzter Start      vorher nicht sauber beendet');
  expect(text).toContain('  [ !! ] Protokoll          1 Warnungen/Fehler heute');
  expect(text).toContain('  Prozessor  ██▌░░░░░░░░░░░░░░░░░  12 % (Programm 1 %)');
  expect(text).toContain('  Speicher   ');
  expect(text).toContain('9.1 / 16.0 GB');
  expect(text).toContain('310 MB (höchster Wert 420 MB)');
  expect(text).toContain('  Programm   seit 25.11. 08:01   6 h 30 min');
  expect(text).toContain('  Computer   seit 23.11. 07:45   2 Tage 6 h');
  expect(text).toContain('  Starts     25.11. 08:01 · 24.11. 21:33 (!) · 24.11. 07:58');
  expect(text).toContain('  2.4 MB · Datenbank-Version 7 · Aufbewahrung 2/14/8/12, alle 60 min');
  expect(text).toContain('  120 Betriebe · 340 Personen · 410 aktive Mitglieder');
  expect(text).toContain('  Adressen');
  expect(text).toContain('  25.11. 14:00:00  WARNUNG  Zweiter Sicherungsort nicht erreichbar');
  expect(text).not.toContain('/x/v.sqlite');
});

test('Bericht ohne Angaben, ohne Datei und ohne Momentaufnahme', () => {
  const text = report({ session: null, snapshot: null, retention: null });
  expect(text).toContain('» (keine Angabe)');
  expect(text).toContain('[ -- ] Datei              keine Datei geöffnet');
  expect(text).not.toContain('  DATEI');
  expect(text).not.toContain('Schreibzugriff');
  expect(text).toContain('PROBLEMBERICHT · Mitgliederverwaltung unbekannt');
  expect(text).toContain('  Prozessor  ░░░░░░░░░░░░░░░░░░░░  unbekannt');
  expect(text).toContain('PROTOKOLL (neueste unten)\n  (keine)');
});

test('mehrzeilige Beschreibung bleibt eingerückt', () => {
  expect(report({ description: 'Zeile eins\nZeile zwei' })).toContain('» Zeile eins\n  Zeile zwei');
});

test('reportOpen ist ein Store', () => {
  reportOpen.set(true);
  let value = false;
  reportOpen.subscribe((v) => (value = v))();
  expect(value).toBe(true);
});

test('Aktionen im Bericht ohne Pfade und ohne Namen des Sperr-Inhabers', () => {
  recordCommand('open_file', false, 5, 'Datei nicht gefunden: S:\\Verein\\Geheimordner\\x.sqlite');
  recordCommand('take_over', false, 5, 'Die Datei wird bereits von Hans Muster auf PC-1 verwendet.');
  recordError(new Error('kaputt bei /Users/hans/Geheim/a.sqlite, sonst nichts'));
  const text = report();
  expect(text).toContain('open_file – Fehler: Datei nicht gefunden: ‹Pfad›');
  expect(text).toContain('take_over – Fehler: Die Datei wird bereits von ‹Person› auf PC-1 verwendet.');
  expect(text).toContain('Fehler: Error: kaputt bei ‹Pfad›, sonst nichts');
  for (const secret of ['Geheimordner', 'Hans', 'Geheim/']) expect(text).not.toContain(secret);
});

test('sanitize wie in Rust', () => {
  expect(sanitize('Öffnen von \\\\server\\freigabe\\a.sqlite: Zugriff verweigert')).toBe('Öffnen von ‹Pfad›: Zugriff verweigert');
  expect(sanitize('Aufbewahrung 2/14/8/12, 9.1 / 16.0 GB')).toBe('Aufbewahrung 2/14/8/12, 9.1 / 16.0 GB');
  expect(sanitize('C:\\Users\\Hans Muster\\a.sqlite')).toBe('‹Pfad›');
});
