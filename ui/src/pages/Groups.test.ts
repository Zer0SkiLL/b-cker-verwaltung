import { render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { get } from 'svelte/store';
import { vi } from 'vitest';
import { emptyFilter } from '../lib/filters';

const vorstand = { ...emptyFilter(), label_ids: [4] };

vi.mock('../lib/api', () => ({
  listGroups: vi.fn(async () => [{ id: 7, name: 'Vorstand' }]),
  loadGroup: vi.fn(async () => vorstand),
  deleteGroup: vi.fn(),
  renameGroup: vi.fn(async () => undefined),
  sessionInfo: vi.fn(async () => null),
}));
vi.mock('../lib/desktop', () => ({ confirmAction: vi.fn(async () => true) }));

import { resetRouter, route } from '../lib/router';
import Groups from './Groups.svelte';

test('Klick auf eine Gruppe öffnet Adressen mit deren Filter', async () => {
  resetRouter();
  const user = userEvent.setup();
  render(Groups);
  await user.click(await screen.findByRole('button', { name: 'Vorstand' }));
  expect(get(route)).toEqual({ page: 'addresses', filter: vorstand });
});

import * as api from '../lib/api';
import { sessionInfo } from '../lib/session';

test('Gruppe umbenennen', async () => {
  sessionInfo.set({
    path: '/x',
    readOnly: false,
    heldBy: null,
    backup: { secondaryDir: '/s', lastSecondary: null, reminderDue: false },
  });
  const user = userEvent.setup();
  render(Groups);
  await user.click(await screen.findByRole('button', { name: 'Umbenennen' }));
  const input = screen.getByLabelText('Neuer Name');
  await user.clear(input);
  await user.type(input, 'Vorstand 2027');
  await user.click(screen.getByRole('button', { name: 'Speichern' }));
  expect(api.renameGroup).toHaveBeenCalledWith(7, 'Vorstand 2027');
});

test('ohne Gruppen erklärt ein Hinweis, wie man eine anlegt', async () => {
  vi.mocked(api.listGroups).mockResolvedValueOnce([]);
  render(Groups);
  expect(await screen.findByText(/Noch keine Gruppen\. Stellen Sie unter „Adressen“ einen Filter zusammen/)).toBeInTheDocument();
  expect(screen.getByRole('button', { name: 'Wie stelle ich einen Versand zusammen?' })).toBeInTheDocument();
});

test('wird die Datei beim Umbenennen nur lesend, verschwindet die Eingabe', async () => {
  const info = {
    path: '/x',
    readOnly: false,
    heldBy: null,
    backup: { secondaryDir: '/s', lastSecondary: null, reminderDue: false },
  };
  sessionInfo.set(info);
  const user = userEvent.setup();
  render(Groups);
  await user.click(await screen.findByRole('button', { name: 'Umbenennen' }));
  expect(screen.getByLabelText('Neuer Name')).toBeInTheDocument();
  sessionInfo.set({ ...info, readOnly: true });
  await Promise.resolve();
  expect(screen.queryByLabelText('Neuer Name')).not.toBeInTheDocument();
  expect(screen.queryByRole('button', { name: 'Speichern' })).not.toBeInTheDocument();
});
