import { render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { vi } from 'vitest';

vi.mock('../lib/api', () => ({
  createAffiliation: vi.fn(async (input) => ({ id: 9, ...input })),
  updateAffiliation: vi.fn(),
  sessionInfo: vi.fn(async () => null),
}));

import * as api from '../lib/api';
import AffiliationEditor from './AffiliationEditor.svelte';

const props = {
  organisationId: 2,
  personId: null,
  people: [{ id: 1, name: 'Muster Hans' }],
  organisations: [],
  functions: [{ id: 5, kind: 'Function' as const, name: 'Inhaber' }],
  existing: null,
  onDone: vi.fn(),
};

test('ungültiges Von-Datum wird gemeldet und nicht gespeichert, gültiges wird ISO', async () => {
  const user = userEvent.setup();
  render(AffiliationEditor, props);
  await user.selectOptions(screen.getByLabelText('Person'), '1');
  await user.selectOptions(screen.getByLabelText('Funktion'), '5');
  await user.type(screen.getByLabelText('Von'), '31.02.2024');
  await user.click(screen.getByRole('button', { name: 'Speichern' }));
  expect(screen.getByText('Ungültiges Datum (TT.MM.JJJJ)')).toBeInTheDocument();
  expect(api.createAffiliation).not.toHaveBeenCalled();

  await user.clear(screen.getByLabelText('Von'));
  await user.type(screen.getByLabelText('Von'), '1.3.2024');
  await user.click(screen.getByRole('button', { name: 'Speichern' }));
  expect(api.createAffiliation).toHaveBeenCalledWith({
    person_id: 1,
    organisation_id: 2,
    function_id: 5,
    valid_from: '2024-03-01',
    valid_to: null,
  });
  expect(props.onDone).toHaveBeenCalled();
});
