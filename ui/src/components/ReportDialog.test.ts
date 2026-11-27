import { render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { get } from 'svelte/store';
import { vi } from 'vitest';

vi.mock('../lib/api', () => ({
  diagnosticsSnapshot: vi.fn(async () => ({
    system: { os: 'Windows 11', arch: 'x86_64', version: '0.3.0' },
    host: 'PC-TEST',
    resources: {
      cpuTotal: 5, cpuProgram: 1, memUsed: 1, memTotal: 2, memProgram: 1, memProgramPeak: 1,
      programSince: null, computerSince: null,
    },
    file: null,
    starts: [],
    problemsToday: 0,
    log: ['26.11. 10:00:00  WARNUNG  Achtung'],
  })),
  saveReport: vi.fn(async () => undefined),
  getRetention: vi.fn(async () => ({ keepAllDays: 2, dailyDays: 14, weeklyWeeks: 8, monthlyMonths: 12, intervalMinutes: 60 })),
  sessionInfo: vi.fn(async () => null),
}));
vi.mock('../lib/desktop', () => ({
  copyText: vi.fn(async () => undefined),
  pickSavePath: vi.fn(async () => '/ablage/Problembericht.txt'),
  todayStamp: () => '2026-11-27',
}));

import * as api from '../lib/api';
import * as desktop from '../lib/desktop';
import { reportOpen } from '../lib/diagnostics';
import { helpState, showToc } from '../lib/help';
import { handleHelpKey } from '../lib/help';
import ReportDialog from './ReportDialog.svelte';

beforeEach(() => reportOpen.set(false));

test('zeigt Beschreibung und Bericht; Tippen aktualisiert den Bericht', async () => {
  const user = userEvent.setup();
  render(ReportDialog);
  reportOpen.set(true);
  const field = await screen.findByLabelText('Was ist passiert?');
  const report = (await screen.findByLabelText('Bericht')) as HTMLTextAreaElement;
  await vi.waitFor(() => expect(report.value).toContain('Windows 11 (x86_64)'));
  expect(report.value).toContain('WARNUNG  Achtung');
  await user.type(field, 'Speichern geht nicht');
  expect(report.value).toContain('Speichern geht nicht');
});

test('Bericht kopieren', async () => {
  const user = userEvent.setup();
  render(ReportDialog);
  reportOpen.set(true);
  await screen.findByLabelText('Bericht');
  await vi.waitFor(() => expect((screen.getByLabelText('Bericht') as HTMLTextAreaElement).value).toContain('0.3.0'));
  await user.click(screen.getByRole('button', { name: 'Bericht kopieren' }));
  expect(vi.mocked(desktop.copyText).mock.calls[0][0]).toContain('PROBLEMBERICHT · Mitgliederverwaltung 0.3.0');
  expect(await screen.findByText('Kopiert.')).toBeInTheDocument();
});

test('Esc schliesst den Dialog, die Hilfe bleibt offen', async () => {
  const user = userEvent.setup();
  showToc();
  render(ReportDialog);
  reportOpen.set(true);
  await screen.findByLabelText('Bericht');
  // Die Hilfe ignoriert Esc, solange ein <dialog open> da ist.
  const event = new KeyboardEvent('keydown', { key: 'Escape' });
  expect(handleHelpKey(event, { page: 'addresses' })).toBe(false);
  await user.keyboard('{Escape}');
  expect(get(reportOpen)).toBe(false);
  expect(get(helpState).open).toBe(true);
});

test('Schliessen-Knopf', async () => {
  const user = userEvent.setup();
  render(ReportDialog);
  reportOpen.set(true);
  await user.click(await screen.findByRole('button', { name: 'Schliessen' }));
  expect(get(reportOpen)).toBe(false);
});

test('Bericht als Datei speichern', async () => {
  const user = userEvent.setup();
  render(ReportDialog);
  reportOpen.set(true);
  await vi.waitFor(() => expect((screen.getByLabelText('Bericht') as HTMLTextAreaElement).value).toContain('0.3.0'));
  await user.click(screen.getByRole('button', { name: 'Als Datei speichern…' }));
  expect(desktop.pickSavePath).toHaveBeenCalledWith('Problembericht-2026-11-27.txt', 'txt');
  const [path, text] = vi.mocked(api.saveReport).mock.calls[0];
  expect(path).toBe('/ablage/Problembericht.txt');
  expect(text).toContain('PROBLEMBERICHT');
  expect(await screen.findByText('Gespeichert: /ablage/Problembericht.txt')).toBeInTheDocument();
});

test('Speichern abgebrochen: nichts passiert', async () => {
  const user = userEvent.setup();
  vi.mocked(desktop.pickSavePath).mockResolvedValueOnce(null);
  vi.mocked(api.saveReport).mockClear();
  render(ReportDialog);
  reportOpen.set(true);
  await screen.findByLabelText('Bericht');
  await user.click(screen.getByRole('button', { name: 'Als Datei speichern…' }));
  expect(api.saveReport).not.toHaveBeenCalled();
});

test('Speichern scheitert: Meldung sichtbar', async () => {
  const user = userEvent.setup();
  vi.mocked(api.saveReport).mockRejectedValueOnce(new Error('Dateifehler: Zugriff verweigert'));
  render(ReportDialog);
  reportOpen.set(true);
  await screen.findByLabelText('Bericht');
  await user.click(screen.getByRole('button', { name: 'Als Datei speichern…' }));
  expect(await screen.findByText('Dateifehler: Zugriff verweigert')).toBeInTheDocument();
});
