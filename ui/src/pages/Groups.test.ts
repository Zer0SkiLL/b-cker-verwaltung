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
