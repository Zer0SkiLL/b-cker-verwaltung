import { render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { vi } from 'vitest';

vi.mock('../lib/api', () => ({
  createMembership: vi.fn(async () => {
    throw new Error('Austritt liegt vor Eintritt');
  }),
  updateMembership: vi.fn(async () => ({})),
  sessionInfo: vi.fn(async () => null),
}));

vi.mock('../lib/osternest/milestone', () => ({
  checkMilestone: vi.fn(async () => undefined),
  countActiveMembers: vi.fn(),
}));

import * as api from '../lib/api';
import { checkMilestone } from '../lib/osternest/milestone';
import MembershipEditor from './MembershipEditor.svelte';

beforeEach(() => vi.mocked(checkMilestone).mockClear());

test('Fehler aus dem Datenkern wird im Formular angezeigt', async () => {
  const user = userEvent.setup();
  const onDone = vi.fn();
  render(MembershipEditor, {
    member: { Organisation: 1 },
    categories: [],
    existing: null,
    onDone,
  });
  await user.type(screen.getByLabelText('Eintritt'), '1.1.2020');
  await user.type(screen.getByLabelText('Austritt'), '1.1.2019');
  await user.click(screen.getByRole('button', { name: 'Speichern' }));
  expect(await screen.findByText('Austritt liegt vor Eintritt')).toBeInTheDocument();
  expect(onDone).not.toHaveBeenCalled();
  expect(checkMilestone).not.toHaveBeenCalled();
});

test('erfolgreiche Neuaufnahme prüft den Meilenstein', async () => {
  vi.mocked(api.createMembership).mockResolvedValueOnce({} as never);
  const user = userEvent.setup();
  const onDone = vi.fn();
  render(MembershipEditor, { member: { Organisation: 1 }, categories: [], existing: null, onDone });
  await user.type(screen.getByLabelText('Eintritt'), '1.1.2020');
  await user.click(screen.getByRole('button', { name: 'Speichern' }));
  await vi.waitFor(() => expect(onDone).toHaveBeenCalled());
  expect(checkMilestone).toHaveBeenCalledTimes(1);
});

test('Ändern einer bestehenden Mitgliedschaft prüft keinen Meilenstein', async () => {
  const user = userEvent.setup();
  const onDone = vi.fn();
  const existing = { id: 7, member: { Organisation: 1 }, category_id: null, joined_on: '2020-01-01', left_on: null };
  render(MembershipEditor, { member: { Organisation: 1 }, categories: [], existing, onDone });
  await user.click(screen.getByRole('button', { name: 'Speichern' }));
  await vi.waitFor(() => expect(onDone).toHaveBeenCalled());
  expect(api.updateMembership).toHaveBeenCalled();
  expect(checkMilestone).not.toHaveBeenCalled();
});
