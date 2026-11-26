import { render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { get } from 'svelte/store';
import { vi } from 'vitest';
import { reportOpen } from '../lib/diagnostics';
import { bakerMode } from '../lib/osternest/state';
import Sidebar from './Sidebar.svelte';

test('Problem melden in der linken Leiste öffnet den Dialog', async () => {
  reportOpen.set(false);
  const user = userEvent.setup();
  render(Sidebar);
  await user.click(screen.getByRole('button', { name: 'Problem melden' }));
  expect(get(reportOpen)).toBe(true);
});

describe('Zeichen neben dem Namen', () => {
  afterEach(() => vi.useRealTimers());

  test('am Welttag des Brotes mit Tooltip', () => {
    vi.useFakeTimers({ toFake: ['Date'] });
    vi.setSystemTime(new Date(2027, 9, 16, 10));
    render(Sidebar);
    const sign = screen.getByText('🍞');
    expect(sign).toHaveAttribute('title', expect.stringMatching(/^Heute ist Welttag des Brotes\./));
    expect(sign.closest('.brand')).toHaveTextContent('Bäckervereinigung');
  });

  test('an einem gewöhnlichen Tag ohne Modus kein Zeichen', () => {
    vi.useFakeTimers({ toFake: ['Date'] });
    vi.setSystemTime(new Date(2027, 9, 17, 10));
    bakerMode.set(false);
    const { container } = render(Sidebar);
    expect(container.querySelector('.osternest')).toBeNull();
  });
});
