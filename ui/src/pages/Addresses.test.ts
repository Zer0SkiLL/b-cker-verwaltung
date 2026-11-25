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
import { emptyFilter } from '../lib/filters';

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

test('leere Datei: Hinweis auf den ersten Eintrag', async () => {
  render(Addresses, {});
  expect(await screen.findByText(/Noch keine Adressen\. Legen Sie mit „Neuer Betrieb“ oder „Neue Person“/)).toBeInTheDocument();
  expect(screen.getByRole('button', { name: 'Wie nehme ich ein Mitglied auf?' })).toBeInTheDocument();
});

test('aktiver Filter ohne Treffer: eigener Hinweis', async () => {
  render(Addresses, { filter: { ...emptyFilter(), text: 'xyz' } });
  expect(await screen.findByText('Keine Treffer für diesen Filter.')).toBeInTheDocument();
  expect(screen.queryByText(/Noch keine Adressen/)).not.toBeInTheDocument();
});
