import { vi } from 'vitest';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(async (command: string) => {
    if (command === 'create_label') throw 'Name ist bereits vergeben';
    return { id: 1, name: 'Bäckerei Geheim' };
  }),
}));

import * as api from './api';
import {
  buildReport, clearActions, MAX_ENTRIES, recentActions, recordCommand, recordError, recordPage, reportOpen,
} from './diagnostics';

const system = { os: 'Windows 11', arch: 'x86_64', version: '0.3.0' };
const session = {
  path: '/x/v.sqlite', readOnly: false, heldBy: null,
  backup: { secondaryDir: 'E:\\', lastSecondary: '2026-11-25T12:10:00Z', reminderDue: false, secondaryError: null },
};
const retention = { keepAllDays: 2, dailyDays: 14, weeklyWeeks: 8, monthlyMonths: 12, intervalMinutes: 60 };
const report = (over: Partial<Parameters<typeof buildReport>[0]> = {}) =>
  buildReport({
    description: '', now: new Date(2026, 10, 25, 14, 3), system, route: { page: 'addresses' },
    session, retention, actions: recentActions(), log: ['[x][WARN] Zweiter Sicherungsort nicht erreichbar: os error 2'], ...over,
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

test('Bericht enthält alle Abschnitte', () => {
  const text = report({ description: 'Speichern ging nicht' });
  expect(text).toContain('Problembericht Mitgliederverwaltung');
  expect(text).toContain('Zeit: 25.11.2026 14:03');
  expect(text).toContain('Version: 0.3.0 · Windows 11 (x86_64)');
  expect(text).toContain('Was ist passiert?\nSpeichern ging nicht');
  expect(text).toContain('- Seite: Adressen');
  expect(text).toContain('- Datei: geöffnet, schreibend');
  expect(text).toContain('- Zweiter Sicherungsort: eingerichtet, letzte Sicherung');
  expect(text).toContain('- Aufbewahrung: 2/14/8/12, Intervall 60 min');
  expect(text).toContain('Letzte Aktionen (neueste zuletzt)');
  expect(text).toContain('Programm-Protokoll (letzte Einträge)\n[x][WARN] Zweiter Sicherungsort nicht erreichbar');
});

test('Bericht ohne Angaben, ohne Datei, mit Fehler am zweiten Ort', () => {
  expect(report({ session: null })).toContain('(keine Angabe)');
  expect(report({ session: null })).toContain('- Datei: keine Datei');
  const err = report({ session: { ...session, backup: { ...session.backup, secondaryError: 'nicht erreichbar' } } });
  expect(err).toContain('- Zweiter Sicherungsort: Fehler: nicht erreichbar');
  const ro = report({ session: { ...session, readOnly: true } });
  expect(ro).toContain('- Datei: nur lesend');
});

test('reportOpen ist ein Store', () => {
  reportOpen.set(true);
  let value = false;
  reportOpen.subscribe((v) => (value = v))();
  expect(value).toBe(true);
});
