import { render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { vi } from 'vitest';
import type { ImportPreview } from '../../lib/types';

const preview: ImportPreview = {
  headers: ['Firma', 'Name'], mapping: ['company', 'last_name'], fields: ['company', 'last_name', 'ignore'],
  rows: 4, merged: 1, persons: 2,
  organisations: [
    { lines: [2, 3], name: 'Beck AG', city: 'Bern', category: 'Kein Mitglied', labels: [], notes: [],
      persons: [
        { line: 2, name: 'Anna Keller', function: 'Kontaktperson', labels: [], notes: [] },
        { line: 3, name: 'Beat Huber', function: 'Kontaktperson', labels: [], notes: ['Mail: Abweichend'] },
      ] },
    { lines: [5], name: 'Ruhig GmbH', city: null, category: null, labels: [], notes: [], persons: [] },
  ],
  individuals: [{ person: { line: 4, name: 'Hans Einzeln', function: 'Einzelmitglied', labels: ['Mandat: Präsident'], notes: [] }, city: null, category: 'Ehrenmitglied' }],
  skipped: [],
  newEntries: { categories: ['Ehrenmitglied'], cantonal: [], functions: ['Kontaktperson'], labels: ['Mandat: Präsident'] },
  reconcileError: null, report: [{ line: 3, column: 'Mail', message: 'Abweichend' }], databaseEmpty: true,
};

vi.mock('../../lib/api', () => ({
  importPreview: vi.fn(async () => preview),
  importApply: vi.fn(async () => ({ organisations: 2, persons: 2, individuals: 1, report: [], undoBackup: '/d/backups/v.sqlite' })),
  importUndo: vi.fn(async () => undefined),
  exportImportPreview: vi.fn(async () => undefined),
  exportReport: vi.fn(async () => undefined),
  sessionInfo: vi.fn(async () => null),
}));
vi.mock('../../lib/desktop', () => ({
  pickFile: vi.fn(async () => '/x/liste.xlsx'),
  pickSavePath: vi.fn(async () => '/x/vorschau.xlsx'),
  confirmAction: vi.fn(async () => true),
  todayStamp: () => '2026-11-25',
}));

import * as api from '../../lib/api';
import * as desktop from '../../lib/desktop';
import { sessionInfo } from '../../lib/session';
import ImportWizard from './ImportWizard.svelte';

const writable = { path: '/d/v.sqlite', readOnly: false, heldBy: null, backup: { secondaryDir: null, lastSecondary: null, reminderDue: false } };

async function open() {
  sessionInfo.set(writable);
  vi.mocked(api.importApply).mockClear();
  vi.mocked(desktop.confirmAction).mockClear();
  const user = userEvent.setup();
  render(ImportWizard);
  await user.click(screen.getByRole('button', { name: '1. Excel-Datei wählen…' }));
  await screen.findByText(/4 Zeilen →/);
  return user;
}

test('Zusammenfassung, neue Einträge und gegliederte Vorschau', async () => {
  await open();
  expect(screen.getByText('4 Zeilen → 2 Betriebe (davon 1 aus mehreren Zeilen zusammengefasst), 2 Personen, 1 Einzelmitglied, 0 Zeilen übersprungen')).toBeInTheDocument();
  expect(screen.getByText('Mitgliederkategorien: Ehrenmitglied')).toBeInTheDocument();
  expect(screen.getByText('Beck AG')).toBeInTheDocument();
  expect(screen.getByText('Anna Keller')).toBeInTheDocument();
  expect(screen.getByText('Hans Einzeln')).toBeInTheDocument();
});

test('Filter „Nur mit Hinweisen“ blendet Betriebe ohne Hinweise aus', async () => {
  const user = await open();
  expect(screen.getByText('Ruhig GmbH')).toBeInTheDocument();
  await user.click(screen.getByLabelText('Nur mit Hinweisen'));
  expect(screen.queryByText('Ruhig GmbH')).not.toBeInTheDocument();
  expect(screen.getByText('Beck AG')).toBeInTheDocument();
});

test('Übernehmen fragt mit den Zahlen nach; Abbrechen speichert nichts', async () => {
  const user = await open();
  vi.mocked(desktop.confirmAction).mockResolvedValueOnce(false);
  await user.click(screen.getByRole('button', { name: 'Übernehmen' }));
  expect(desktop.confirmAction).toHaveBeenCalledWith('2 Betriebe, 2 Personen und 1 Einzelmitglied anlegen?', 'Übernehmen');
  expect(api.importApply).not.toHaveBeenCalled();
});

test('nach dem Import lässt er sich rückgängig machen', async () => {
  const user = await open();
  await user.click(screen.getByRole('button', { name: 'Übernehmen' }));
  await user.click(await screen.findByRole('button', { name: 'Import rückgängig machen' }));
  expect(api.importUndo).toHaveBeenCalledWith('/d/backups/v.sqlite');
  expect(await screen.findByText('Import rückgängig gemacht.')).toBeInTheDocument();
});

test('Vorschau als Excel speichern', async () => {
  const user = await open();
  await user.click(screen.getByRole('button', { name: 'Vorschau als Excel speichern…' }));
  expect(api.exportImportPreview).toHaveBeenCalledWith('/x/liste.xlsx', ['company', 'last_name'], '/x/vorschau.xlsx');
});

test('geht der Zeilenabgleich nicht auf, ist Übernehmen gesperrt', async () => {
  vi.mocked(api.importPreview).mockResolvedValueOnce({ ...preview, reconcileError: 'Der Abgleich der Zeilen geht nicht auf – bitte Problem melden' });
  await open();
  expect(screen.getByText('Der Abgleich der Zeilen geht nicht auf – bitte Problem melden')).toBeInTheDocument();
  expect(screen.getByRole('button', { name: 'Übernehmen' })).toBeDisabled();
});
