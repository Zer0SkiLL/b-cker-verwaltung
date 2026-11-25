import { render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { get } from 'svelte/store';
import { reportOpen } from '../lib/diagnostics';
import Sidebar from './Sidebar.svelte';

test('Problem melden in der linken Leiste öffnet den Dialog', async () => {
  reportOpen.set(false);
  const user = userEvent.setup();
  render(Sidebar);
  await user.click(screen.getByRole('button', { name: 'Problem melden' }));
  expect(get(reportOpen)).toBe(true);
});
