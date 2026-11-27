import { render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { vi } from 'vitest';

vi.mock('../../lib/api', () => ({
  diagnoseSettings: vi.fn(),
  setDiagnoseEnabled: vi.fn(async () => undefined),
}));
vi.mock('../../lib/earlyWarning', () => ({
  currentReport: vi.fn(async () => ({ report: 'PROBLEMBERICHT · Vorschau', statusBlock: '' })),
}));

import * as api from '../../lib/api';
import DiagnoseSwitch from './DiagnoseSwitch.svelte';

const info = (over = {}) => ({ available: true, enabled: true, until: '2027-05-31', lastSent: null, ...over });

test('ohne Webhook im Build: nichts zu sehen', async () => {
  vi.mocked(api.diagnoseSettings).mockResolvedValue(info({ available: false }));
  render(DiagnoseSwitch);
  await Promise.resolve();
  await Promise.resolve();
  expect(screen.queryByLabelText('Analysedaten senden')).toBeNull();
});

test('Schalter mit Erklärung, Enddatum und Ausschalten', async () => {
  vi.mocked(api.diagnoseSettings).mockResolvedValue(info());
  const user = userEvent.setup();
  render(DiagnoseSwitch);
  const box = (await screen.findByLabelText('Analysedaten senden')) as HTMLInputElement;
  expect(box.checked).toBe(true);
  expect(screen.getByText(/Endet automatisch am 31\.05\.2027/)).toBeInTheDocument();
  expect(screen.getByText(/keine Mitgliederdaten/)).toBeInTheDocument();
  expect(screen.getByText('Noch nichts gesendet.')).toBeInTheDocument();
  await user.click(box);
  expect(api.setDiagnoseEnabled).toHaveBeenCalledWith(false);
});

test('Was wird gesendet? zeigt den Bericht', async () => {
  vi.mocked(api.diagnoseSettings).mockResolvedValue(info({ lastSent: new Date(2026, 10, 27, 14, 5).toISOString() }));
  const user = userEvent.setup();
  render(DiagnoseSwitch);
  expect(await screen.findByText('Zuletzt gesendet: 27.11.2026 14:05')).toBeInTheDocument();
  await user.click(screen.getByRole('button', { name: 'Was wird gesendet?' }));
  expect(((await screen.findByLabelText('Gesendeter Bericht')) as HTMLTextAreaElement).value).toContain('PROBLEMBERICHT · Vorschau');
});
