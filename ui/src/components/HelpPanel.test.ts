import { render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { get } from 'svelte/store';
import { vi } from 'vitest';

vi.mock('../lib/desktop', () => ({ openWebsite: vi.fn(async () => undefined) }));
vi.mock('@tauri-apps/api/app', () => ({ getVersion: vi.fn(async () => '0.2.0') }));
vi.mock('../lib/handbook', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../lib/handbook')>();
  return {
    ...actual,
    renderChapter: (slug: string) =>
      slug === '03-mitglied-aufnehmen'
        ? '<h1>Mitglied aufnehmen</h1><p>Siehe <a href="05-austritt.md">Austritt</a> und <a>extern</a>.</p>'
        : actual.renderChapter(slug),
  };
});

import * as desktop from '../lib/desktop';
import { helpState, openHelp } from '../lib/help';
import HelpPanel from './HelpPanel.svelte';

beforeEach(() => helpState.set({ open: false, view: { kind: 'toc' }, history: [] }));

test('zeigt das geöffnete Kapitel', () => {
  openHelp('05-austritt');
  render(HelpPanel);
  expect(screen.getByRole('heading', { name: 'Austritt' })).toBeInTheDocument();
});

test('Kapitel-Link öffnet das Ziel, Zurück führt zurück', async () => {
  const user = userEvent.setup();
  openHelp('03-mitglied-aufnehmen');
  render(HelpPanel);
  await user.click(screen.getByText('Austritt'));
  expect(screen.getByRole('heading', { name: 'Austritt' })).toBeInTheDocument();
  await user.click(screen.getByRole('button', { name: 'Zurück' }));
  expect(screen.getByRole('heading', { name: 'Mitglied aufnehmen' })).toBeInTheDocument();
});

test('Link ohne Kapitel-Ziel ändert nichts', async () => {
  const user = userEvent.setup();
  openHelp('03-mitglied-aufnehmen');
  render(HelpPanel);
  await user.click(screen.getByText('extern'));
  expect(get(helpState).view).toEqual({ kind: 'chapter', slug: '03-mitglied-aufnehmen' });
});

test('Suche zeigt Treffer und öffnet sie; ohne Treffer ein Hinweis', async () => {
  const user = userEvent.setup();
  openHelp('01-willkommen');
  render(HelpPanel);
  const field = screen.getByPlaceholderText('Im Handbuch suchen');
  await user.type(field, 'Austritt');
  await user.click(screen.getByRole('button', { name: /^Austritt/ }));
  expect(screen.getByRole('heading', { name: 'Austritt' })).toBeInTheDocument();
  await user.clear(screen.getByPlaceholderText('Im Handbuch suchen'));
  await user.type(screen.getByPlaceholderText('Im Handbuch suchen'), 'xyzxyz');
  expect(
    screen.getByText('Keine Treffer. Versuchen Sie ein anderes Wort oder öffnen Sie das Inhaltsverzeichnis.'),
  ).toBeInTheDocument();
});

test('Inhalt zeigt die drei Teile', async () => {
  const user = userEvent.setup();
  openHelp('05-austritt');
  render(HelpPanel);
  await user.click(screen.getByRole('button', { name: 'Inhalt' }));
  for (const part of ['Erste Schritte', 'Abläufe', 'Nachschlagen']) {
    expect(screen.getByRole('heading', { name: part })).toBeInTheDocument();
  }
  await user.click(screen.getByRole('button', { name: 'Teams' }));
  expect(screen.getByRole('heading', { name: 'Teams' })).toBeInTheDocument();
});

test('Handbuch als PDF öffnet die Datei zur Version', async () => {
  const user = userEvent.setup();
  openHelp('05-austritt');
  render(HelpPanel);
  await user.click(screen.getByRole('button', { name: 'Handbuch als PDF' }));
  expect(desktop.openWebsite).toHaveBeenCalledWith(
    'https://github.com/Zer0SkiLL/b-cker-verwaltung/releases/download/v0.2.0/Handbuch.pdf',
  );
});

test('Schliessen setzt open auf false', async () => {
  const user = userEvent.setup();
  openHelp('05-austritt');
  render(HelpPanel);
  await user.click(screen.getByRole('button', { name: 'Hilfe schliessen' }));
  expect(get(helpState).open).toBe(false);
});
