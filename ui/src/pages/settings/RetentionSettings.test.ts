import { render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { vi } from 'vitest';

vi.mock('../../lib/api', () => ({
  getRetention: vi.fn(async () => ({ keepAllDays: 3, dailyDays: 20, weeklyWeeks: 8, monthlyMonths: 12, intervalMinutes: 30 })),
  setRetention: vi.fn(async () => undefined),
  sessionInfo: vi.fn(async () => null),
}));

import * as api from '../../lib/api';
import RetentionSettings from './RetentionSettings.svelte';

test('Felder vorbelegt, Speichern sendet die Werte', async () => {
  const user = userEvent.setup();
  render(RetentionSettings, { readOnly: false });
  const minutes = await screen.findByLabelText('Während der Arbeit sichern alle … Minuten (0 = aus)');
  expect(minutes).toHaveValue(30);
  await user.clear(minutes);
  await user.type(minutes, '45');
  await user.click(screen.getByRole('button', { name: 'Speichern' }));
  expect(api.setRetention).toHaveBeenCalledWith({ keepAllDays: 3, dailyDays: 20, weeklyWeeks: 8, monthlyMonths: 12, intervalMinutes: 45 });
  expect(await screen.findByText('Gespeichert.')).toBeInTheDocument();
});

test('Meldung aus Rust bei ungültigen Werten', async () => {
  vi.mocked(api.setRetention).mockRejectedValueOnce(new Error('„Während der Arbeit sichern“: 0 (aus) oder 15 bis 480 Minuten.'));
  const user = userEvent.setup();
  render(RetentionSettings, { readOnly: false });
  await screen.findByLabelText('Eine pro Tag für … Tage');
  await user.click(screen.getByRole('button', { name: 'Speichern' }));
  expect(await screen.findByText('„Während der Arbeit sichern“: 0 (aus) oder 15 bis 480 Minuten.')).toBeInTheDocument();
});

test('Standard wiederherstellen setzt 2/14/8/12/60', async () => {
  const user = userEvent.setup();
  render(RetentionSettings, { readOnly: false });
  await screen.findByLabelText('Alle Sicherungen behalten für … Tage');
  await user.click(screen.getByRole('button', { name: 'Standard wiederherstellen' }));
  expect(screen.getByLabelText('Alle Sicherungen behalten für … Tage')).toHaveValue(2);
  expect(screen.getByLabelText('Eine pro Tag für … Tage')).toHaveValue(14);
  expect(screen.getByLabelText('Eine pro Woche für … Wochen')).toHaveValue(8);
  expect(screen.getByLabelText('Eine pro Monat für … Monate')).toHaveValue(12);
  expect(screen.getByLabelText('Während der Arbeit sichern alle … Minuten (0 = aus)')).toHaveValue(60);
});

test('nur lesend: gesperrt', async () => {
  render(RetentionSettings, { readOnly: true });
  await screen.findByLabelText('Eine pro Tag für … Tage');
  expect(screen.getByRole('button', { name: 'Speichern' })).toBeDisabled();
  expect(screen.getByLabelText('Eine pro Tag für … Tage')).toBeDisabled();
});

test('leere oder ungültige Eingabe: deutsche Meldung, nichts gesendet', async () => {
  vi.mocked(api.setRetention).mockClear();
  const user = userEvent.setup();
  render(RetentionSettings, { readOnly: false });
  const days = await screen.findByLabelText('Eine pro Tag für … Tage');
  await user.clear(days);
  await user.click(screen.getByRole('button', { name: 'Speichern' }));
  expect(screen.getByText('Bitte in jedes Feld eine ganze Zahl ab 0 eingeben.')).toBeInTheDocument();
  expect(api.setRetention).not.toHaveBeenCalled();
});
