import { render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { vi } from 'vitest';

const readOnlyInfo = {
  path: '/x',
  readOnly: true,
  heldBy: { user: 'anna', host: 'pc1', since: '2026-11-24T07:00:00Z', likelyCrash: false },
  backup: { secondaryDir: '/s', lastSecondary: null, reminderDue: false },
};

vi.mock('../lib/api', () => ({
  takeOver: vi.fn(async () => {
    throw new Error('Übernehmen nicht möglich: Zugriff verweigert');
  }),
  sessionInfo: vi.fn(async () => readOnlyInfo),
}));
vi.mock('../lib/desktop', () => ({ confirmAction: vi.fn(async () => true) }));

import * as api from '../lib/api';
import { bumpData, sessionInfo } from '../lib/session';
import NoticeBar from './NoticeBar.svelte';

test('gescheitertes Übernehmen prüft die Sitzung neu; Meldung verschwindet nach nächster Aktion', async () => {
  sessionInfo.set(readOnlyInfo);
  const user = userEvent.setup();
  render(NoticeBar);
  await user.click(screen.getByRole('button', { name: 'Übernehmen' }));
  expect(await screen.findByText('Übernehmen nicht möglich: Zugriff verweigert')).toBeInTheDocument();
  expect(api.sessionInfo).toHaveBeenCalled();
  bumpData();
  await vi.waitFor(() => expect(screen.queryByText(/Übernehmen nicht möglich/)).toBeNull());
});
