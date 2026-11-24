import { render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { get } from 'svelte/store';
import { vi } from 'vitest';

const overview = {
  status: { secondaryDir: null, lastSecondary: null, reminderDue: true },
  localDir: '/daten/backups',
  files: [{ path: '/daten/backups/verband-20261124-081200.sqlite', created: '24.11.2026 08:12' }],
};

vi.mock('../../lib/api', () => ({
  backupOverview: vi.fn(async () => overview),
  restoreBackup: vi.fn(async () => undefined),
  backupNow: vi.fn(),
  setSecondaryDir: vi.fn(),
  sessionInfo: vi.fn(async () => ({
    path: '/daten/verband.sqlite',
    readOnly: false,
    heldBy: null,
    backup: overview.status,
  })),
}));
vi.mock('../../lib/desktop', () => ({
  confirmAction: vi.fn(async () => true),
  pickFolder: vi.fn(async () => null),
  pickFile: vi.fn(async () => '/stick/verband-20261101-090000.sqlite'),
}));

import * as api from '../../lib/api';
import { dataVersion, sessionInfo } from '../../lib/session';
import Backup from './Backup.svelte';

const writable = {
  path: '/daten/verband.sqlite',
  readOnly: false,
  heldBy: null,
  backup: overview.status,
};

test('Wiederherstellen nach Rückfrage lädt Daten und Sitzung neu', async () => {
  sessionInfo.set(writable);
  const user = userEvent.setup();
  const before = get(dataVersion);
  render(Backup);
  await user.click(await screen.findByRole('button', { name: 'Wiederherstellen' }));
  expect(api.restoreBackup).toHaveBeenCalledWith(overview.files[0].path);
  expect(get(dataVersion)).toBe(before + 1);
  expect(api.sessionInfo).toHaveBeenCalled();
});

test('nur lesend: Sichern und Wiederherstellen gesperrt', async () => {
  sessionInfo.set({ ...writable, readOnly: true, heldBy: { user: 'anna', host: 'pc1', since: '2026-11-24T07:00:00Z', likelyCrash: false } });
  render(Backup);
  expect(await screen.findByRole('button', { name: 'Wiederherstellen' })).toBeDisabled();
  expect(screen.getByRole('button', { name: 'Jetzt sichern' })).toBeDisabled();
});

test('Wiederherstellen aus einer gewählten Datei (z.B. USB-Stick)', async () => {
  sessionInfo.set(writable);
  vi.mocked(api.restoreBackup).mockClear();
  const user = userEvent.setup();
  render(Backup);
  await user.click(await screen.findByRole('button', { name: 'Aus Datei wiederherstellen…' }));
  expect(api.restoreBackup).toHaveBeenCalledWith('/stick/verband-20261101-090000.sqlite');
});
