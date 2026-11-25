import { render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { get } from 'svelte/store';
import { helpState } from '../lib/help';
import EmptyHint from './EmptyHint.svelte';

test('Hilfe-Link öffnet das Kapitel', async () => {
  const user = userEvent.setup();
  render(EmptyHint, { text: 'Noch nichts da.', help: { label: 'Mehr dazu', slug: '07-versand' } });
  expect(screen.getByText(/Noch nichts da\./)).toBeInTheDocument();
  await user.click(screen.getByRole('button', { name: 'Mehr dazu' }));
  expect(get(helpState)).toMatchObject({ open: true, view: { kind: 'chapter', slug: '07-versand' } });
});
