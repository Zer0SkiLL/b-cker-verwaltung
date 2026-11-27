import { get } from 'svelte/store';
import { vi } from 'vitest';

vi.mock('./api', () => ({
  diagnosticsSnapshot: vi.fn(),
  selfTest: vi.fn(),
  getRetentionQuiet: vi.fn(async () => ({ keepAllDays: 2, dailyDays: 14, weeklyWeeks: 8, monthlyMonths: 12, intervalMinutes: 60 })),
  diagnosticsPublish: vi.fn(async () => undefined),
  diagnosticsWriteFolder: vi.fn(async () => undefined),
  diagnoseHeartbeatDue: vi.fn(async () => false),
}));
let eventHandler: ((e: { payload: { kind: string } }) => void) | null = null;
vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(async (_name: string, handler: (e: { payload: { kind: string } }) => void) => {
    eventHandler = handler;
    return () => undefined;
  }),
}));

import * as api from './api';
import {
  autoReport, checkCrash, checkOpenedFile, crashNotice, createCollector, dismissNotice, earlyNotices, runSelfTest, startEarlyWarning,
  statusBlock, WINDOW_MS,
} from './earlyWarning';
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

test('Collector fasst Warnungen im Fenster zusammen', () => {
  vi.useFakeTimers();
  const publish = vi.fn();
  const collect = createCollector(publish);
  collect('warnung');
  vi.advanceTimersByTime(10_000);
  collect('warnung');
  collect('selbsttest');
  expect(publish).not.toHaveBeenCalled();
  vi.advanceTimersByTime(WINDOW_MS);
  expect(publish).toHaveBeenCalledTimes(1);
  expect(publish).toHaveBeenCalledWith('selbsttest');
  vi.useRealTimers();
});

test('Absturz wird sofort gemeldet und ersetzt wartende Warnungen', () => {
  vi.useFakeTimers();
  const publish = vi.fn();
  const collect = createCollector(publish);
  collect('warnung');
  collect('absturz');
  expect(publish).toHaveBeenCalledWith('absturz');
  vi.advanceTimersByTime(WINDOW_MS * 2);
  expect(publish).toHaveBeenCalledTimes(1);
  vi.useRealTimers();
});

const fullSnapshot = {
  system: { os: 'Windows 11', arch: 'x86_64', version: '0.6.0' },
  host: 'PC-TEST',
  resources: { cpuTotal: 5, cpuProgram: 1, memUsed: 1, memTotal: 2, memProgram: 1, memProgramPeak: 1, programSince: null, computerSince: null },
  file: null,
  starts: [],
  problemsToday: 0,
  log: [],
} as SnapshotDto;

test('statusBlock: Status und Computer aus dem Bericht', () => {
  const block = statusBlock('Kopf\n\n  STATUS AUF EINEN BLICK\n  [ OK ] x\n\n  COMPUTER\n  Prozessor\n\n  LAUFZEITEN\n  y');
  expect(block).toBe('  STATUS AUF EINEN BLICK\n  [ OK ] x\n\n  COMPUTER\n  Prozessor');
});

test('autoReport baut den Bericht und übergibt ihn', async () => {
  vi.mocked(api.diagnosticsSnapshot).mockResolvedValue(fullSnapshot);
  await autoReport('warnung');
  const calls = vi.mocked(api.diagnosticsPublish).mock.calls;
  const [kind, block, report] = calls[calls.length - 1];
  expect(kind).toBe('warnung');
  expect(block).toContain('STATUS AUF EINEN BLICK');
  expect(report).toContain('» (automatisch: Warnung)');
});

test('autoReport schluckt Fehler', async () => {
  vi.mocked(api.diagnosticsSnapshot).mockRejectedValueOnce(new Error('weg'));
  vi.mocked(api.diagnosticsPublish).mockRejectedValueOnce(new Error('weg'));
  await expect(autoReport('warnung')).resolves.toBeUndefined();
});

test('Start: Absturz einmal melden, sonst Lebenszeichen, Ereignisse sammeln', async () => {
  vi.mocked(api.diagnosticsPublish).mockClear();
  vi.mocked(api.diagnosticsSnapshot).mockResolvedValue({ ...fullSnapshot, starts: [start('2026-11-27T08:00:00+01:00', false)] });
  await startEarlyWarning();
  expect(vi.mocked(api.diagnosticsPublish).mock.calls.map((c) => c[0])).toEqual(['absturz']);
  await startEarlyWarning();
  expect(vi.mocked(api.diagnosticsPublish).mock.calls.map((c) => c[0])).toEqual(['absturz']);
  vi.mocked(api.diagnosticsSnapshot).mockResolvedValue(fullSnapshot);
  vi.mocked(api.diagnoseHeartbeatDue).mockResolvedValueOnce(true);
  await startEarlyWarning();
  expect(vi.mocked(api.diagnosticsPublish).mock.calls.map((c) => c[0])).toEqual(['absturz', 'lebenszeichen']);
  expect(eventHandler).not.toBeNull();
});

test('Selbsttest nur einmal pro Datei', async () => {
  vi.mocked(api.selfTest).mockClear().mockResolvedValue({ integrityProblem: false, lowSpace: false, backupStale: false, slowOpen: false });
  vi.mocked(api.diagnosticsSnapshot).mockResolvedValue(fullSnapshot);
  await checkOpenedFile('/a.sqlite');
  await checkOpenedFile('/a.sqlite');
  expect(api.selfTest).toHaveBeenCalledTimes(1);
  expect(api.diagnosticsWriteFolder).toHaveBeenCalled();
  await checkOpenedFile('/b.sqlite');
  expect(api.selfTest).toHaveBeenCalledTimes(2);
});
