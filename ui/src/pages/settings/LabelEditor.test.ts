import { render, screen } from '@testing-library/svelte';
import { vi } from 'vitest';

vi.mock('../../lib/api', () => ({
  listLabelTree: vi.fn(async () => []),
  sessionInfo: vi.fn(async () => null),
}));
vi.mock('../../lib/desktop', () => ({ confirmAction: vi.fn(async () => true) }));

import LabelEditor from './LabelEditor.svelte';

test('ohne Label-Gruppen ein Hinweis mit Link ins Handbuch', async () => {
  render(LabelEditor);
  expect(await screen.findByText(/Noch keine Label-Gruppen\./)).toBeInTheDocument();
  expect(screen.getByRole('button', { name: 'Mehr zu Labels' })).toBeInTheDocument();
});
