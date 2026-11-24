import { render } from '@testing-library/svelte';
import { get } from 'svelte/store';
import { vi } from 'vitest';

const info = { path: '/x.sqlite', readOnly: false, heldBy: null, backup: { secondaryDir: null, lastSecondary: null, reminderDue: true } };
vi.mock('../lib/api', () => ({
  lastPath: vi.fn(async () => null),
  sessionInfo: vi.fn(async () => info),
}));
vi.mock('../lib/desktop', () => ({}));

import { resetRouter, route } from '../lib/router';
import Start from './Start.svelte';

test('ist bereits eine Datei offen, geht es direkt zu den Adressen', async () => {
  resetRouter();
  render(Start);
  await vi.waitFor(() => expect(get(route)).toEqual({ page: 'addresses' }));
});
