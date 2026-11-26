import { render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { vi } from 'vitest';

vi.mock('../../lib/desktop', () => ({ openWebsite: vi.fn(async () => undefined) }));
vi.mock('../../lib/updates', async () => {
  const { writable } = await import('svelte/store');
  return { updateState: writable({ kind: 'idle' }), checkForUpdates: vi.fn() };
});

import { get } from 'svelte/store';
import * as desktop from '../../lib/desktop';
import { bakerMode } from '../../lib/osternest/state';
import About from './About.svelte';

test('Link zum Repo öffnet den Browser', async () => {
  const user = userEvent.setup();
  render(About);
  await user.click(screen.getByRole('button', { name: 'github.com/Zer0SkiLL/b-cker-verwaltung' }));
  expect(desktop.openWebsite).toHaveBeenCalledWith('https://github.com/Zer0SkiLL/b-cker-verwaltung');
});

test('7 Klicks auf die Version schalten den Bäckermeister-Modus ein, 7 weitere wieder aus', async () => {
  bakerMode.set(false);
  const user = userEvent.setup();
  render(About);
  const versionText = await screen.findByText(/^Version /);
  for (let i = 0; i < 6; i++) await user.click(versionText);
  expect(screen.queryByText('Sie sind jetzt Bäckermeister.')).not.toBeInTheDocument();
  await user.click(versionText);
  expect(screen.getByText('Sie sind jetzt Bäckermeister.')).toBeInTheDocument();
  expect(get(bakerMode)).toBe(true);
  for (let i = 0; i < 7; i++) await user.click(versionText);
  expect(screen.getByText('Bäckermeister-Modus aus. Zurück an den Ofen.')).toBeInTheDocument();
  expect(get(bakerMode)).toBe(false);
});
