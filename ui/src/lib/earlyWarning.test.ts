import { get } from 'svelte/store';
import { vi } from 'vitest';

vi.mock('./api', () => ({
  diagnosticsSnapshot: vi.fn(),
  selfTest: vi.fn(),
}));

import * as api from './api';
import { checkCrash, crashNotice, dismissNotice, earlyNotices, runSelfTest } from './earlyWarning';
import type { SnapshotDto, StartDto } from './types';

const start = (at: string, cleanBefore: boolean | null): StartDto => ({ at, cleanBefore });
const snap = (starts: StartDto[]) => ({ starts }) as unknown as SnapshotDto;

beforeEach(() => {
  earlyNotices.set([]);
  localStorage.clear();
});

test('crashNotice: nur wenn der vorige Lauf nicht sauber endete und noch nicht ausgeblendet', () => {
  expect(crashNotice([start('a', false)], null)).toEqual(start('a', false));
  expect(crashNotice([start('a', false)], 'a')).toBeNull();
  expect(crashNotice([start('a', true)], null)).toBeNull();
  expect(crashNotice([start('a', null)], null)).toBeNull();
  expect(crashNotice([], null)).toBeNull();
});

test('checkCrash zeigt den Hinweis einmal pro Vorfall', async () => {
  vi.mocked(api.diagnosticsSnapshot).mockResolvedValue(snap([start('2026-11-27T08:00:00+01:00', false)]));
  await checkCrash();
  expect(get(earlyNotices)).toEqual([{ kind: 'crash', text: 'Das Programm wurde letztes Mal unerwartet beendet.' }]);
  dismissNotice('crash');
  expect(get(earlyNotices)).toEqual([]);
  await checkCrash();
  expect(get(earlyNotices)).toEqual([]);
});

test('checkCrash schluckt Fehler', async () => {
  vi.mocked(api.diagnosticsSnapshot).mockRejectedValue(new Error('weg'));
  await expect(checkCrash()).resolves.toBeNull();
});

test('Selbsttest: Nutzer sieht nur, was er beheben kann', async () => {
  vi.mocked(api.selfTest).mockResolvedValue({ integrityProblem: true, lowSpace: true, backupStale: true, slowOpen: true });
  const result = await runSelfTest();
  expect(result?.backupStale).toBe(true);
  expect(get(earlyNotices).map((n) => n.text)).toEqual([
    'Die Datei hat einen Fehler. Bitte melden Sie das Problem.',
    'Auf dem Laufwerk der Datei ist nur noch wenig Platz frei.',
  ]);
  vi.mocked(api.selfTest).mockResolvedValue({ integrityProblem: false, lowSpace: false, backupStale: false, slowOpen: false });
  await runSelfTest();
  expect(get(earlyNotices)).toEqual([]);
});

test('Selbsttest scheitert still', async () => {
  vi.mocked(api.selfTest).mockRejectedValue(new Error('keine Datei'));
  await expect(runSelfTest()).resolves.toBeNull();
});
