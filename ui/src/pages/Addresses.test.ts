import { fireEvent, render, screen } from '@testing-library/svelte';
import { get } from 'svelte/store';
import { vi } from 'vitest';

vi.mock('../lib/api', () => ({
  searchAddresses: vi.fn(async () => ({ rows: [], emailCount: 0, missingEmail: [], joinedEmails: '', mailto: null })),
  filterOptions: vi.fn(async () => null),
  sessionInfo: vi.fn(async () => null),
}));
vi.mock('../lib/desktop', () => ({}));

import * as api from '../lib/api';
import { replace, route } from '../lib/router';
import { addressSort } from '../lib/sorting';
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

test('Klick auf einen Spaltenkopf sortiert, zweiter Klick dreht die Richtung', async () => {
  const contact = (id: number, display_name: string, city: string) => ({
    kind: 'Organisation' as const, id, display_name, organisation: null, street: null, zip: null,
    city, po_box: null, email: null, phone: null, mobile: null,
  });
  vi.mocked(api.searchAddresses).mockResolvedValueOnce({
    rows: [contact(1, 'Aebi', 'Zürich'), contact(2, 'Brot AG', 'Bern'), contact(3, 'Christen', 'Aarau')],
    emailCount: 0, missingEmail: [], joinedEmails: '', mailto: null,
  });
  addressSort.set(null);
  render(Addresses, {});
  await screen.findByText('Aebi');
  const firstNames = () => screen.getAllByRole('row').slice(1).map((r) => r.querySelectorAll('td')[1].textContent);

  await fireEvent.click(screen.getByRole('button', { name: /^Ort/ }));
  expect(firstNames()).toEqual(['Christen', 'Brot AG', 'Aebi']);
  expect(screen.getByRole('columnheader', { name: /Ort/ })).toHaveAttribute('aria-sort', 'ascending');

  await fireEvent.click(screen.getByRole('button', { name: /^Ort/ }));
  expect(firstNames()).toEqual(['Aebi', 'Brot AG', 'Christen']);
  expect(screen.getByRole('columnheader', { name: /Ort/ })).toHaveAttribute('aria-sort', 'descending');
  addressSort.set(null);
});

describe('Osternest in Adressen', () => {
  afterEach(() => vi.useRealTimers());
  const at = (date: Date) => {
    vi.useFakeTimers({ toFake: ['Date'] });
    vi.setSystemTime(date);
  };

  test('spät abends steht unter „Keine Treffer“ ein Zusatz', async () => {
    at(new Date(2027, 2, 10, 23));
    render(Addresses, { filter: { ...emptyFilter(), text: 'xyz' } });
    expect(await screen.findByText('Keine Treffer für diesen Filter.')).toBeInTheDocument();
    expect(screen.getByText('Um diese Zeit backen nur noch die Bäcker.')).toBeInTheDocument();
  });

  test('tagsüber nur der normale Hinweis', async () => {
    at(new Date(2027, 2, 10, 10));
    render(Addresses, { filter: { ...emptyFilter(), text: 'xyz' } });
    expect(await screen.findByText('Keine Treffer für diesen Filter.')).toBeInTheDocument();
    expect(screen.queryByText(/backen nur noch/)).not.toBeInTheDocument();
  });

  test('an Dreikönig liegt eine Krone unten auf der Seite', async () => {
    at(new Date(2027, 0, 6, 10));
    render(Addresses, {});
    await fireEvent.click(await screen.findByRole('button', { name: 'Krone' }));
    expect(screen.getByText('Sie sind heute König.')).toBeInTheDocument();
  });

  test('nach Dreikönig keine Krone', async () => {
    at(new Date(2027, 0, 10, 10));
    render(Addresses, {});
    await screen.findByText(/Einträge/);
    expect(screen.queryByRole('button', { name: 'Krone' })).not.toBeInTheDocument();
  });
});
