import { render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { get } from 'svelte/store';
import { vi } from 'vitest';

vi.mock('../lib/api', () => ({
  diagnosticsSystem: vi.fn(async () => ({ os: 'Windows 11', arch: 'x86_64', version: '0.3.0' })),
  diagnosticsLogTail: vi.fn(async () => ['[x][WARN] Achtung']),
  getRetention: vi.fn(async () => ({ keepAllDays: 2, dailyDays: 14, weeklyWeeks: 8, monthlyMonths: 12, intervalMinutes: 60 })),
  sessionInfo: vi.fn(async () => null),
}));
vi.mock('../lib/desktop', () => ({ copyText: vi.fn(async () => undefined) }));

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
  await vi.waitFor(() => expect(report.value).toContain('Version: 0.3.0 · Windows 11 (x86_64)'));
  expect(report.value).toContain('[x][WARN] Achtung');
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
  expect(vi.mocked(desktop.copyText).mock.calls[0][0]).toContain('Problembericht Mitgliederverwaltung');
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
