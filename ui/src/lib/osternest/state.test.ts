import { get } from 'svelte/store';
import { vi } from 'vitest';

const KEY = 'osternest-baeckermeister';

beforeEach(() => {
  localStorage.clear();
  vi.resetModules();
});

afterEach(() => {
  vi.restoreAllMocks();
  vi.useRealTimers();
});

test('countVersionClick zählt bis 7 und schaltet dann um', async () => {
  const { countVersionClick, VERSION_CLICKS } = await import('./state');
  expect(VERSION_CLICKS).toBe(7);
  expect(countVersionClick(0)).toEqual({ count: 1, toggled: false });
  expect(countVersionClick(5)).toEqual({ count: 6, toggled: false });
  expect(countVersionClick(6)).toEqual({ count: 0, toggled: true });
});

test('setBakerMode speichert, ein neuer Start liest es wieder', async () => {
  const first = await import('./state');
  expect(get(first.bakerMode)).toBe(false);
  first.setBakerMode(true);
  expect(localStorage.getItem(KEY)).toBe('1');
  vi.resetModules();
  const second = await import('./state');
  expect(get(second.bakerMode)).toBe(true);
  second.setBakerMode(false);
  expect(localStorage.getItem(KEY)).toBe('0');
});

test('badge: besonderer Tag vor der Brezel, sonst nur mit Modus', async () => {
  const { badge } = await import('./state');
  expect(badge(new Date(2027, 9, 16), true)?.icon).toBe('🍞');
  expect(badge(new Date(2027, 9, 17), true)).toEqual({ icon: '🥨', text: 'Bäckermeister-Modus. Gut Teig!' });
  expect(badge(new Date(2027, 9, 17), false)).toBeNull();
  expect(badge(new Date(2027, 9, 16), false)?.icon).toBe('🍞');
});

test('gesperrter Speicher: alles funktioniert, nur ohne Merken', async () => {
  vi.spyOn(Storage.prototype, 'getItem').mockImplementation(() => { throw new Error('gesperrt'); });
  vi.spyOn(Storage.prototype, 'setItem').mockImplementation(() => { throw new Error('gesperrt'); });
  const { bakerMode, setBakerMode } = await import('./state');
  expect(get(bakerMode)).toBe(false);
  expect(() => setBakerMode(true)).not.toThrow();
  expect(get(bakerMode)).toBe(true);
});

test('now tickt jede Minute, auch über Mitternacht', async () => {
  vi.useFakeTimers();
  vi.setSystemTime(new Date(2027, 9, 15, 23, 59, 30));
  const { now } = await import('./state');
  const seen: Date[] = [];
  const stop = now.subscribe((d) => seen.push(d));
  expect(seen[seen.length - 1]?.getDate()).toBe(15);
  vi.advanceTimersByTime(60_000);
  expect(seen[seen.length - 1]?.getDate()).toBe(16);
  stop();
});
