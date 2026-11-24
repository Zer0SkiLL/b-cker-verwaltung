import { render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { vi } from 'vitest';

vi.mock('../lib/api', () => ({
  createPerson: vi.fn(async (input) => ({ id: 42, ...input })),
  createAffiliation: vi.fn(async (input) => ({ id: 9, ...input })),
  updateAffiliation: vi.fn(),
  sessionInfo: vi.fn(async () => null),
}));

import * as api from '../lib/api';
import AffiliationEditor from './AffiliationEditor.svelte';

const props = () => ({
  organisationId: 2,
  personId: null,
  people: [{ id: 1, name: 'Muster Hans' }],
  organisations: [],
  functions: [{ id: 5, kind: 'Function' as const, name: 'Inhaber' }],
  existing: null,
  onDone: vi.fn(),
});

async function fillNewPerson(user: ReturnType<typeof userEvent.setup>) {
  await user.selectOptions(screen.getByLabelText('Person'), 'new');
  await user.type(screen.getByLabelText('Nachname *'), 'Keller');
  await user.type(screen.getByLabelText('Vorname'), 'Eva');
  await user.selectOptions(screen.getByLabelText('Funktion'), '5');
  await user.click(screen.getByRole('button', { name: 'Speichern' }));
}

test('neue Person wird angelegt und verknüpft', async () => {
  const user = userEvent.setup();
  const p = props();
  render(AffiliationEditor, p);
  await fillNewPerson(user);
  expect(api.createPerson).toHaveBeenCalledWith(expect.objectContaining({ last_name: 'Keller', first_name: 'Eva' }));
  expect(api.createAffiliation).toHaveBeenCalledWith(expect.objectContaining({ person_id: 42, organisation_id: 2, function_id: 5 }));
  expect(p.onDone).toHaveBeenCalled();
});

test('neue Person ohne Nachname wird nicht angelegt', async () => {
  vi.mocked(api.createPerson).mockClear();
  const user = userEvent.setup();
  render(AffiliationEditor, props());
  await user.selectOptions(screen.getByLabelText('Person'), 'new');
  await user.selectOptions(screen.getByLabelText('Funktion'), '5');
  await user.click(screen.getByRole('button', { name: 'Speichern' }));
  expect(screen.getByText('Nachname ist erforderlich')).toBeInTheDocument();
  expect(api.createPerson).not.toHaveBeenCalled();
});

test('scheitert die Verknüpfung, sagt die Meldung, dass die Person angelegt ist', async () => {
  vi.mocked(api.createAffiliation).mockRejectedValueOnce(new Error('Datenbankfehler: x'));
  const user = userEvent.setup();
  render(AffiliationEditor, props());
  await fillNewPerson(user);
  expect(
    await screen.findByText('Person wurde angelegt, die Verknüpfung aber nicht: Datenbankfehler: x'),
  ).toBeInTheDocument();
});

test('erneutes Speichern nach gescheiterter Verknüpfung legt die Person nicht doppelt an', async () => {
  vi.mocked(api.createPerson).mockClear();
  vi.mocked(api.createAffiliation).mockClear();
  vi.mocked(api.createAffiliation).mockRejectedValueOnce(new Error('Datenbankfehler: x'));
  const user = userEvent.setup();
  render(AffiliationEditor, props());
  await fillNewPerson(user);
  await screen.findByText(/Person wurde angelegt/);
  await user.click(screen.getByRole('button', { name: 'Speichern' }));
  expect(api.createPerson).toHaveBeenCalledTimes(1);
  expect(api.createAffiliation).toHaveBeenLastCalledWith(expect.objectContaining({ person_id: 42 }));
});
