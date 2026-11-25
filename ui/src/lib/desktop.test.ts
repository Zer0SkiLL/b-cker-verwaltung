import { vi } from 'vitest';

vi.mock('@tauri-apps/plugin-dialog', () => ({
  open: vi.fn(async () => '/vom/dialog.xlsx'),
  save: vi.fn(async () => '/vom/dialog'),
  ask: vi.fn(),
  message: vi.fn(),
}));

import { open } from '@tauri-apps/plugin-dialog';
import { pickFile } from './desktop';

type Hooked = Window & { __mvDialogAntworten?: { open?: string } };

afterEach(() => {
  delete (window as Hooked).__mvDialogAntworten;
  vi.mocked(open).mockClear();
});

test('pickFile öffnet normalerweise den Dialog', async () => {
  expect(await pickFile('Excel', ['xlsx'])).toBe('/vom/dialog.xlsx');
  expect(open).toHaveBeenCalledOnce();
});

test('pickFile nimmt eine vorbereitete Antwort (automatische Bildschirmfotos)', async () => {
  (window as Hooked).__mvDialogAntworten = { open: '/vorbereitet/liste.xlsx' };
  expect(await pickFile('Excel', ['xlsx'])).toBe('/vorbereitet/liste.xlsx');
  expect(open).not.toHaveBeenCalled();
});
