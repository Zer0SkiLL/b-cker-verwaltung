import { render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { vi } from 'vitest';

vi.mock('../lib/api', () => ({
  updateMembership: vi.fn(async (id, input) => ({ id, ...input })),
  createMembership: vi.fn(),
  deleteMembership: vi.fn(),
  sessionInfo: vi.fn(async () => null),
}));
vi.mock('../lib/desktop', () => ({ confirmAction: vi.fn(async () => true) }));

import * as api from '../lib/api';
import MembershipList from './MembershipList.svelte';

const member = { Organisation: 1 };
const items = [
  { id: 10, member, category_id: null, joined_on: '2010-01-01', left_on: null },
  { id: 20, member, category_id: null, joined_on: '2015-06-01', left_on: '2020-12-31' },
];

test('Wechsel von Zeile A zu Zeile B bearbeitet B mit den Werten von B', async () => {
  const user = userEvent.setup();
  render(MembershipList, { member, items, categories: [], readOnly: false });
  const edit = screen.getAllByRole('button', { name: 'Bearbeiten' });
  await user.click(edit[0]);
  await user.click(edit[1]);
  expect(screen.getByLabelText('Eintritt')).toHaveValue('01.06.2015');
  await user.click(screen.getByRole('button', { name: 'Speichern' }));
  expect(api.updateMembership).toHaveBeenCalledWith(20, {
    member,
    category_id: null,
    joined_on: '2015-06-01',
    left_on: '2020-12-31',
  });
});
