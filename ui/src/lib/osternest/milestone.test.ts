import { get } from 'svelte/store';
import { vi } from 'vitest';

vi.mock('../api', () => ({ countActiveMembers: vi.fn() }));

import * as api from '../api';
import { checkMilestone, countActiveMembers, nextMilestone, rain, RAIN_MS } from './milestone';

const KEY = 'osternest-meilenstein';

beforeEach(() => {
  localStorage.clear();
  rain.set(null);
});
afterEach(() => vi.restoreAllMocks());

test.each<[number | null, number, number, number | null]>([
  [null, 312, 300, null],
  [null, 50, 0, null],
  [300, 399, 300, null],
  [300, 400, 400, 400],
  [400, 380, 400, null],
  [0, 100, 100, 100],
  [0, 99, 0, null],
])('nextMilestone(%s, %i) → speichert %i, feiert %s', (stored, count, store, celebrate) => {
  expect(nextMilestone(stored, count)).toEqual({ store, celebrate });
});

test('RAIN_MS ist 3 Sekunden', () => {
  expect(RAIN_MS).toBe(3000);
});

test('erreichte Marke: Regen und gespeichert', async () => {
  localStorage.setItem(KEY, '300');
  await checkMilestone(async () => 400);
  expect(get(rain)).toBe(400);
  expect(localStorage.getItem(KEY)).toBe('400');
});

test('erste Zählung wird still gemerkt', async () => {
  await checkMilestone(async () => 312);
  expect(get(rain)).toBeNull();
  expect(localStorage.getItem(KEY)).toBe('300');
});

test('Fehler beim Zählen bleibt still', async () => {
  await expect(checkMilestone(async () => { throw new Error('x'); })).resolves.toBeUndefined();
  expect(get(rain)).toBeNull();
});

test('gesperrter Speicher: keine Ausnahme, kein Regen', async () => {
  vi.spyOn(Storage.prototype, 'getItem').mockImplementation(() => { throw new Error('gesperrt'); });
  vi.spyOn(Storage.prototype, 'setItem').mockImplementation(() => { throw new Error('gesperrt'); });
  await expect(checkMilestone(async () => 400)).resolves.toBeUndefined();
  expect(get(rain)).toBeNull();
});

test('countActiveMembers fragt den Datenkern (Mitglieder, nicht Kontaktpersonen)', async () => {
  vi.mocked(api.countActiveMembers).mockResolvedValueOnce(60);
  expect(await countActiveMembers()).toBe(60);
});
