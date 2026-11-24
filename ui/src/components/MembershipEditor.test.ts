import { render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { vi } from 'vitest';

vi.mock('../lib/api', () => ({
  createMembership: vi.fn(async () => {
    throw new Error('Austritt liegt vor Eintritt');
  }),
  updateMembership: vi.fn(),
  sessionInfo: vi.fn(async () => null),
}));

import MembershipEditor from './MembershipEditor.svelte';

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
});
