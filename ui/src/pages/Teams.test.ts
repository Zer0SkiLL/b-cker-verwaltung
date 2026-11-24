import { render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { vi } from 'vitest';

vi.mock('../lib/api', () => ({
  listTeams: vi.fn(async () => []),
  listPeople: vi.fn(async () => []),
  teamMembers: vi.fn(async () => []),
  createTeam: vi.fn(async () => {
    throw new Error('Name ist bereits vergeben');
  }),
  sessionInfo: vi.fn(async () => ({
    path: '/x', readOnly: false, heldBy: null,
    backup: { secondaryDir: '/s', lastSecondary: null, reminderDue: false },
  })),
}));
vi.mock('../lib/desktop', () => ({ confirmAction: vi.fn(async () => true) }));

import { sessionInfo } from '../lib/session';
import Teams from './Teams.svelte';

test('fehlgeschlagenes Anlegen behält die Eingabe', async () => {
  sessionInfo.set({
    path: '/x', readOnly: false, heldBy: null,
    backup: { secondaryDir: '/s', lastSecondary: null, reminderDue: false },
  });
  const user = userEvent.setup();
  render(Teams);
  const input = screen.getByPlaceholderText('Neues Team');
  await user.type(input, 'Vorstand');
  await user.click(screen.getByRole('button', { name: 'Anlegen' }));
  expect(await screen.findByText('Name ist bereits vergeben')).toBeInTheDocument();
  expect(input).toHaveValue('Vorstand');
});
