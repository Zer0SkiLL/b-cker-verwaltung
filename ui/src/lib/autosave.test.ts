import { vi } from 'vitest';
import { startAutoBackup } from './autosave';

afterEach(() => vi.useRealTimers());

test('sichert nach dem Intervall und prüft danach die Sitzung', async () => {
  vi.useFakeTimers();
  const backup = vi.fn(async () => null);
  const refresh = vi.fn(async () => undefined);
  const stop = startAutoBackup({ getInterval: async () => 60, backup, refresh });
  await vi.advanceTimersByTimeAsync(59 * 60_000);
  expect(backup).not.toHaveBeenCalled();
  await vi.advanceTimersByTimeAsync(60_000);
  expect(backup).toHaveBeenCalledTimes(1);
  expect(refresh).toHaveBeenCalledTimes(1);
  await vi.advanceTimersByTimeAsync(60 * 60_000);
  expect(backup).toHaveBeenCalledTimes(2);
  stop();
  await vi.advanceTimersByTimeAsync(120 * 60_000);
  expect(backup).toHaveBeenCalledTimes(2);
});

test('Intervall 0 sichert nie, liest die Einstellung aber später erneut', async () => {
  vi.useFakeTimers();
  let interval = 0;
  const backup = vi.fn(async () => null);
  const stop = startAutoBackup({ getInterval: async () => interval, backup, refresh: async () => undefined });
  await vi.advanceTimersByTimeAsync(240 * 60_000);
  expect(backup).not.toHaveBeenCalled();
  interval = 15;
  await vi.advanceTimersByTimeAsync(30 * 60_000);
  expect(backup).toHaveBeenCalled();
  stop();
});

test('Fehler beim Sichern stoppen den Zeitgeber nicht', async () => {
  vi.useFakeTimers();
  const backup = vi.fn(async () => {
    throw new Error('Keine Datei geöffnet.');
  });
  const stop = startAutoBackup({ getInterval: async () => 15, backup, refresh: async () => undefined });
  await vi.advanceTimersByTimeAsync(31 * 60_000);
  expect(backup).toHaveBeenCalledTimes(2);
  stop();
});
