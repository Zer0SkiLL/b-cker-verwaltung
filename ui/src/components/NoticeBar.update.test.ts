import { render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { vi } from 'vitest';

vi.mock('../lib/api', () => ({ sessionInfo: vi.fn(async () => null) }));
vi.mock('../lib/desktop', () => ({ confirmAction: vi.fn(async () => true) }));
vi.mock('../lib/updates', async () => {
  const { writable } = await import('svelte/store');
  return {
    updateState: writable({ kind: 'available', version: '0.3.0', notes: '### Neu\n- Mailvorlagen' }),
    installUpdate: vi.fn(async () => undefined),
    reopenLastFile: vi.fn(async () => undefined),
    checkForUpdates: vi.fn(async () => undefined),
  };
});

import * as updates from '../lib/updates';
import NoticeBar from './NoticeBar.svelte';

test('Update-Hinweis: Was ist neu und Jetzt installieren', async () => {
  const user = userEvent.setup();
  render(NoticeBar);
  expect(screen.getByText('Version 0.3.0 ist verfügbar')).toBeInTheDocument();
  await user.click(screen.getByText('Was ist neu?'));
  expect(screen.getByText('Mailvorlagen')).toBeInTheDocument();
  await user.click(screen.getByRole('button', { name: 'Jetzt installieren' }));
  expect(updates.installUpdate).toHaveBeenCalled();
});
