import { render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { vi } from 'vitest';

vi.mock('../../lib/api', () => ({
  listLookups: vi.fn(async () => [{ value: { id: 3, name: 'Kontaktperson', sort: 0 }, usage: 0 }]),
  renameLookup: vi.fn(async () => undefined),
  sessionInfo: vi.fn(async () => null),
}));
vi.mock('../../lib/desktop', () => ({ confirmAction: vi.fn(async () => true) }));

import LookupEditor from './LookupEditor.svelte';

test('wird die Datei beim Umbenennen nur lesend, verschwindet die Eingabe', async () => {
  const user = userEvent.setup();
  const view = render(LookupEditor, { kind: 'Function', readOnly: false });
  await user.click(await screen.findByRole('button', { name: 'Umbenennen' }));
  expect(screen.getByLabelText('Neuer Name')).toBeInTheDocument();
  await view.rerender({ kind: 'Function', readOnly: true });
  expect(screen.queryByLabelText('Neuer Name')).not.toBeInTheDocument();
});
