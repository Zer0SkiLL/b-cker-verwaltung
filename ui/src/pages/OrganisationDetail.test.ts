import { render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { get } from 'svelte/store';
import { vi } from 'vitest';

const org = {
  id: 3, name: 'Beck AG', addition: null, street: null, po_box: null, zip: null, city: null,
  phone: null, email: null, website: null, branches: null, apprentice_count: null, notes: null,
  legacy_id: null, org_type_id: null, cantonal_association_id: null,
};

vi.mock('../lib/api', () => ({
  filterOptions: vi.fn(async () => ({ labelGroups: [], teams: [], functions: [], orgTypes: [], categories: [], cantonal: [] })),
  listPeople: vi.fn(async () => []),
  organisationDetail: vi.fn(async () => ({ organisation: org, people: [], memberships: [], labels: [] })),
  sessionInfo: vi.fn(async () => null),
}));
const confirm = vi.fn(async (..._args: unknown[]) => false);
vi.mock('../lib/desktop', () => ({ confirmAction: (...args: unknown[]) => confirm(...args) }));

import { navigate, resetRouter, route } from '../lib/router';
import OrganisationDetail from './OrganisationDetail.svelte';

test('ungespeicherte Änderung: Zurück fragt nach und bleibt bei Abbruch', async () => {
  resetRouter();
  await navigate({ page: 'addresses' });
  await navigate({ page: 'organisation', id: 3 });
  const user = userEvent.setup();
  render(OrganisationDetail, { id: 3 });
  const name = await screen.findByDisplayValue('Beck AG');
  await user.click(screen.getByRole('button', { name: '← Zurück' }));
  expect(confirm).not.toHaveBeenCalled();
  expect(get(route)).toEqual({ page: 'addresses' });

  await navigate({ page: 'organisation', id: 3 });
  await user.type(name, ' Neu');
  await user.click(screen.getByRole('button', { name: '← Zurück' }));
  expect(confirm).toHaveBeenCalled();
  expect(get(route)).toEqual({ page: 'organisation', id: 3 });
});
