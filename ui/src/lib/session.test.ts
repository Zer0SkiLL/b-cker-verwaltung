import { get } from 'svelte/store';
import { vi } from 'vitest';

vi.mock('./api', () => ({
  sessionInfo: vi.fn(async () => ({
    path: '/x/verband.sqlite',
    readOnly: true,
    heldBy: { user: 'anna', host: 'pc1', since: '2026-11-24T07:12:00Z' },
    backup: { secondaryDir: null, lastSecondary: null, reminderDue: true },
  })),
}));

import { bumpData, dataVersion, noticesFor, refreshSession, sessionInfo } from './session';

test('refreshSession setzt den Store', async () => {
  await refreshSession();
  expect(get(sessionInfo)?.readOnly).toBe(true);
});

test('bumpData erhöht die Version', () => {
  const before = get(dataVersion);
  bumpData();
  expect(get(dataVersion)).toBe(before + 1);
});

test('Hinweise: nur lesen und fehlender zweiter Ort', () => {
  const notices = noticesFor({
    path: '/x',
    readOnly: true,
    heldBy: { user: 'anna', host: 'pc1', since: new Date(2026, 10, 24, 8, 12).toISOString(), likelyCrash: false },
    backup: { secondaryDir: null, lastSecondary: null, reminderDue: true },
  });
  expect(notices.map((n) => n.kind)).toEqual(['readOnly', 'backup']);
  expect(notices[0].text).toBe('Nur lesen – geöffnet von anna (pc1) seit 24.11.2026 08:12');
  expect(notices[1].text).toBe('Kein zweiter Sicherungsort eingerichtet');
});

test('Hinweise: alte Sicherung am zweiten Ort', () => {
  const notices = noticesFor({
    path: '/x',
    readOnly: false,
    heldBy: null,
    backup: { secondaryDir: '/stick', lastSecondary: '2026-01-01T00:00:00Z', reminderDue: true },
  });
  expect(notices).toEqual([
    { kind: 'backup', text: 'Letzte Sicherung am zweiten Ort ist älter als 30 Tage' },
  ]);
});

test('Hinweise: alles in Ordnung', () => {
  expect(
    noticesFor({
      path: '/x',
      readOnly: false,
      heldBy: null,
      backup: { secondaryDir: '/stick', lastSecondary: '2026-11-20T00:00:00Z', reminderDue: false },
    }),
  ).toEqual([]);
});

import { runWrite } from './session';
import * as api from './api';

test('runWrite: Erfolg erhöht die Datenversion', async () => {
  const before = get(dataVersion);
  const value = await runWrite(async () => 42);
  expect(value).toBe(42);
  expect(get(dataVersion)).toBe(before + 1);
});

test('runWrite: Fehler aktualisiert die Sitzung und gibt den Text weiter', async () => {
  vi.mocked(api.sessionInfo).mockClear();
  const before = get(dataVersion);
  await expect(
    runWrite(async () => {
      throw new Error('Die Datei ist nur lesend geöffnet. Änderungen sind nicht möglich.');
    }),
  ).rejects.toThrow('nur lesend');
  expect(api.sessionInfo).toHaveBeenCalled();
  expect(get(dataVersion)).toBe(before);
});

import { readOnly } from './session';

test('readOnly: unbekannter Zustand gilt als nur lesen', () => {
  sessionInfo.set(null);
  expect(get(readOnly)).toBe(true);
  sessionInfo.set({
    path: '/x',
    readOnly: false,
    heldBy: null,
    backup: { secondaryDir: '/s', lastSecondary: null, reminderDue: false },
  });
  expect(get(readOnly)).toBe(false);
});

test('Hinweis nach Absturz', () => {
  const [notice] = noticesFor({
    path: '/x',
    readOnly: true,
    heldBy: { user: 'anna', host: 'pc1', since: new Date(2026, 10, 24, 8, 12).toISOString(), likelyCrash: true },
    backup: { secondaryDir: '/s', lastSecondary: null, reminderDue: false },
  });
  expect(notice.text).toBe(
    'Nur lesen – geöffnet von anna (pc1) seit 24.11.2026 08:12 – wahrscheinlich nach einem Absturz, bitte „Übernehmen“',
  );
});
