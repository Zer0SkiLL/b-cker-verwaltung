import { render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { vi } from 'vitest';

const info = {
  path: '/x', readOnly: false, heldBy: null,
  backup: { secondaryDir: 'E:\\', lastSecondary: null, reminderDue: true, secondaryError: 'nicht erreichbar' },
};

vi.mock('../lib/api', () => ({
  backupNow: vi.fn(async () => ({ local: '/x/b.sqlite', secondary: { kind: 'done', path: 'E:\\b' } })),
  sessionInfo: vi.fn(async () => ({ ...info, backup: { ...info.backup, secondaryError: null, reminderDue: false } })),
}));
vi.mock('../lib/desktop', () => ({ confirmAction: vi.fn(async () => true) }));

import * as api from '../lib/api';
import { sessionInfo } from '../lib/session';
import NoticeBar from './NoticeBar.svelte';

test('Fehler am zweiten Ort: „Jetzt sichern“ sichert und prüft die Sitzung neu', async () => {
  sessionInfo.set(info);
  const user = userEvent.setup();
  render(NoticeBar);
  expect(screen.getByText('Zweiter Sicherungsort nicht erreichbar – letzte Sicherung dort: noch nie')).toBeInTheDocument();
  await user.click(screen.getByRole('button', { name: 'Jetzt sichern' }));
  expect(api.backupNow).toHaveBeenCalled();
  expect(api.sessionInfo).toHaveBeenCalled();
  expect(screen.queryByText(/nicht erreichbar/)).not.toBeInTheDocument();
});
