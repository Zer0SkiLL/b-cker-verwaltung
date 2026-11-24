import { fireEvent, render, screen } from '@testing-library/svelte';
import { get } from 'svelte/store';
import { vi } from 'vitest';

vi.mock('../lib/api', () => ({
  searchAddresses: vi.fn(async () => ({ rows: [], emailCount: 0, missingEmail: [], joinedEmails: '', mailto: null })),
  filterOptions: vi.fn(async () => null),
  sessionInfo: vi.fn(async () => null),
}));
vi.mock('../lib/desktop', () => ({}));

import { replace, route } from '../lib/router';
import Addresses from './Addresses.svelte';

test('verlassene Seite: ausstehende Suche holt nicht zurück', async () => {
  vi.useFakeTimers();
  try {
    const { unmount } = render(Addresses, {});
    await fireEvent.input(screen.getByPlaceholderText('Suche Name, Ort, Mail…'), { target: { value: 'beck' } });
    replace({ page: 'organisation', id: 1 });
    unmount();
    vi.advanceTimersByTime(500);
    expect(get(route)).toEqual({ page: 'organisation', id: 1 });
  } finally {
    vi.useRealTimers();
  }
});
