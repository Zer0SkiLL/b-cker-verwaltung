import { vi } from 'vitest';

const logged: unknown[][] = [];
vi.mock('./logFile', () => ({ logToFile: (...args: unknown[]) => logged.push(args) }));
let duration = 0;
vi.mock('@tauri-apps/api/core', () => ({
  invoke: async () => {
    vi.spyOn(performance, 'now').mockReturnValue(duration);
    return null;
  },
}));

import * as api from './api';

beforeEach(() => {
  logged.length = 0;
  vi.spyOn(performance, 'now').mockReturnValue(0);
});

test('langsame Aktion wird als Warnung protokolliert', async () => {
  duration = 2500;
  await api.sessionInfo();
  expect(logged).toEqual([['warn', 'Langsam: session_info 2500 ms']]);
});

test('schnelle Aktion nicht', async () => {
  duration = 2000;
  await api.sessionInfo();
  expect(logged).toEqual([]);
});
