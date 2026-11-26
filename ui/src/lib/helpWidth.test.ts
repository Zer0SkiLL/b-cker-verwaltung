import { get } from 'svelte/store';
import { vi } from 'vitest';
import { HELP_WIDTH, clampWidth, helpWidth, isWide, loadWidth, setHelpWidth, toggleWide } from './helpWidth';

const VIEWPORT = 1600;

beforeEach(() => {
  localStorage.clear();
  helpWidth.set(HELP_WIDTH.normal);
});

test('clampWidth: nicht schmaler als das Minimum, Programm behält genug Platz', () => {
  expect(clampWidth(100, VIEWPORT)).toBe(HELP_WIDTH.min);
  expect(clampWidth(500, VIEWPORT)).toBe(500);
  expect(clampWidth(5000, VIEWPORT)).toBe(VIEWPORT - HELP_WIDTH.keepForApp);
  // Sehr kleines Fenster: Minimum gilt trotzdem.
  expect(clampWidth(500, 600)).toBe(HELP_WIDTH.min);
  expect(clampWidth(Number.NaN, VIEWPORT)).toBe(HELP_WIDTH.normal);
});

test('setHelpWidth begrenzt, setzt und merkt sich die Breite', () => {
  setHelpWidth(9999, VIEWPORT);
  expect(get(helpWidth)).toBe(VIEWPORT - HELP_WIDTH.keepForApp);
  expect(loadWidth()).toBe(VIEWPORT - HELP_WIDTH.keepForApp);
});

test('loadWidth: ohne oder mit unbrauchbarem Wert die normale Breite', () => {
  expect(loadWidth()).toBe(HELP_WIDTH.normal);
  localStorage.setItem('hilfe-breite', 'abc');
  expect(loadWidth()).toBe(HELP_WIDTH.normal);
  localStorage.setItem('hilfe-breite', '50');
  expect(loadWidth()).toBe(HELP_WIDTH.min);
});

test('ohne Speicher (gesperrt) funktioniert alles, nur ohne Merken', () => {
  const getItem = vi.spyOn(Storage.prototype, 'getItem').mockImplementation(() => { throw new Error('gesperrt'); });
  const setItem = vi.spyOn(Storage.prototype, 'setItem').mockImplementation(() => { throw new Error('gesperrt'); });
  try {
    expect(loadWidth()).toBe(HELP_WIDTH.normal);
    setHelpWidth(500, VIEWPORT);
    expect(get(helpWidth)).toBe(500);
  } finally {
    getItem.mockRestore();
    setItem.mockRestore();
  }
});

test('toggleWide wechselt zwischen normal und breit', () => {
  expect(isWide(get(helpWidth))).toBe(false);
  toggleWide(VIEWPORT);
  expect(get(helpWidth)).toBe(HELP_WIDTH.wide);
  expect(isWide(get(helpWidth))).toBe(true);
  toggleWide(VIEWPORT);
  expect(get(helpWidth)).toBe(HELP_WIDTH.normal);
});

test('toggleWide im kleinen Fenster: so breit wie möglich', () => {
  toggleWide(1000);
  expect(get(helpWidth)).toBe(1000 - HELP_WIDTH.keepForApp);
});
