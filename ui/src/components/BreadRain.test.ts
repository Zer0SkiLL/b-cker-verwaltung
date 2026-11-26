import { fireEvent, render, screen } from '@testing-library/svelte';
import { flushSync } from 'svelte';
import { get } from 'svelte/store';
import { vi } from 'vitest';
import { rain, RAIN_MS } from '../lib/osternest/milestone';
import BreadRain from './BreadRain.svelte';

beforeEach(() => rain.set(null));
afterEach(() => vi.useRealTimers());

test('ohne Meilenstein ist nichts zu sehen', () => {
  const { container } = render(BreadRain);
  expect(container.querySelector('.rain')).toBeNull();
});

test('Meilenstein zeigt die Zahl und endet nach RAIN_MS', () => {
  vi.useFakeTimers();
  render(BreadRain);
  rain.set(400);
  flushSync();
  expect(screen.getByText(/400 aktive Mitglieder!/)).toBeInTheDocument();
  vi.advanceTimersByTime(RAIN_MS);
  flushSync();
  expect(get(rain)).toBeNull();
  expect(screen.queryByText(/aktive Mitglieder!/)).not.toBeInTheDocument();
});

test('ein Klick irgendwo beendet den Regen sofort', async () => {
  render(BreadRain);
  rain.set(100);
  flushSync();
  await fireEvent.pointerDown(window);
  expect(get(rain)).toBeNull();
});
