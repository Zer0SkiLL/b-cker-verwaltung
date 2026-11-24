import { render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { vi } from 'vitest';

vi.mock('../../lib/desktop', () => ({ openWebsite: vi.fn(async () => undefined) }));
vi.mock('../../lib/updates', async () => {
  const { writable } = await import('svelte/store');
  return { updateState: writable({ kind: 'idle' }), checkForUpdates: vi.fn() };
});

import * as desktop from '../../lib/desktop';
import About from './About.svelte';

test('Link zum Repo öffnet den Browser', async () => {
  const user = userEvent.setup();
  render(About);
  await user.click(screen.getByRole('button', { name: 'github.com/Zer0SkiLL/b-cker-verwaltung' }));
  expect(desktop.openWebsite).toHaveBeenCalledWith('https://github.com/Zer0SkiLL/b-cker-verwaltung');
});
