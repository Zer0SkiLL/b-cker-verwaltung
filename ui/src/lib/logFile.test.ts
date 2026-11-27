// Ohne vi.fn als Rückgabe: dessen Ergebnis-Verfolgung meldet abgelehnte Promises selbst.
const calls: unknown[][] = [];
let impl: () => Promise<unknown> = () => Promise.resolve();
vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: unknown[]) => {
    calls.push(args);
    return impl();
  },
}));

import { logToFile } from './logFile';
import { recentActions, recordError } from './diagnostics';

beforeEach(() => {
  calls.length = 0;
  impl = () => Promise.resolve();
});

test('logToFile ruft den Befehl direkt auf, nicht über die Aktionsliste', () => {
  const before = recentActions().length;
  logToFile('warn', 'Langsam: x 2500 ms');
  expect(calls).toEqual([['diagnostics_log', { level: 'warn', message: 'Langsam: x 2500 ms' }]]);
  expect(recentActions().length).toBe(before);
});

test('logToFile schluckt Fehler', async () => {
  impl = () => Promise.reject(new Error('kaputt'));
  expect(() => logToFile('info', 'x')).not.toThrow();
  await new Promise((r) => setTimeout(r, 0));
});

test('unerwartete Fehler gehen auch ins Protokoll', () => {
  recordError(new TypeError('x is undefined'));
  expect(calls).toContainEqual(['diagnostics_log', { level: 'error', message: 'Oberfläche: TypeError: x is undefined' }]);
});
