import { fireEvent, render, screen } from '@testing-library/svelte';
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
        : slug === 'mit-bild'
          ? '<h1>Mit Bild</h1><p><img src="data:image/png;base64,AA" alt="Die Seite Adressen"></p>'
          : actual.renderChapter(slug),
  };
});

import * as desktop from '../lib/desktop';
import { helpState, openHelp } from '../lib/help';
import { HELP_WIDTH, helpWidth } from '../lib/helpWidth';
import HelpPanel from './HelpPanel.svelte';

beforeEach(() => {
  helpState.set({ open: false, view: { kind: 'toc' }, history: [] });
  helpWidth.set(HELP_WIDTH.normal);
});

test('Klick auf ein Bild zeigt es gross; Schliessen, Klick oder Esc schliesst nur das Bild', async () => {
  const user = userEvent.setup();
  openHelp('mit-bild');
  render(HelpPanel);
  await user.click(screen.getByRole('button', { name: 'Bild vergrössern: Die Seite Adressen' }));
  const dialog = screen.getByRole('dialog', { name: 'Die Seite Adressen' });
  expect(dialog).toBeInTheDocument();
  await user.click(screen.getByRole('button', { name: 'Bild schliessen' }));
  expect(screen.queryByRole('dialog')).not.toBeInTheDocument();

  await user.click(screen.getByRole('button', { name: 'Bild vergrössern: Die Seite Adressen' }));
  await user.keyboard('{Escape}');
  expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
  expect(get(helpState).open).toBe(true);

  await user.click(screen.getByRole('button', { name: 'Bild vergrössern: Die Seite Adressen' }));
  await user.click(screen.getByRole('img', { name: 'Die Seite Adressen' , hidden: false }).closest('.lightbox')!);
  expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
});

test('Breiter/Schmaler ändert die Breite der Hilfe', async () => {
  const user = userEvent.setup();
  openHelp('05-austritt');
  render(HelpPanel);
  await user.click(screen.getByRole('button', { name: 'Breiter' }));
  expect(get(helpWidth)).toBeGreaterThan(HELP_WIDTH.normal);
  await user.click(screen.getByRole('button', { name: 'Schmaler' }));
  expect(get(helpWidth)).toBe(HELP_WIDTH.normal);
});

// jsdom kennt kein PointerEvent; MouseEvent liefert clientX genauso.
if (!('PointerEvent' in window)) Object.assign(window, { PointerEvent: MouseEvent });

test('Ziehen am Rand ändert die Breite, Doppelklick setzt sie zurück', async () => {
  openHelp('05-austritt');
  render(HelpPanel);
  const handle = screen.getByRole('separator', { name: 'Breite der Hilfe ändern' });
  const width = window.innerWidth;
  await fireEvent.pointerDown(handle, { clientX: width - HELP_WIDTH.normal, pointerId: 1 });
  await fireEvent.pointerMove(window, { clientX: width - 500, pointerId: 1 });
  await fireEvent.pointerUp(window, { pointerId: 1 });
  expect(get(helpWidth)).toBe(Math.min(500, Math.max(HELP_WIDTH.min, width - HELP_WIDTH.keepForApp)));
  await fireEvent.pointerMove(window, { clientX: width - 700, pointerId: 1 });
  expect(get(helpWidth)).toBe(Math.min(500, Math.max(HELP_WIDTH.min, width - HELP_WIDTH.keepForApp)));
  await fireEvent.dblClick(handle);
  expect(get(helpWidth)).toBe(HELP_WIDTH.normal);
});

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

test('Problem melden in der Hilfe-Leiste öffnet den Dialog', async () => {
  const { reportOpen } = await import('../lib/diagnostics');
  reportOpen.set(false);
  const user = userEvent.setup();
  openHelp('05-austritt');
  render(HelpPanel);
  await user.click(screen.getByRole('button', { name: 'Problem melden' }));
  expect(get(reportOpen)).toBe(true);
});

test('Zurück zur Suche zeigt den Suchbegriff wieder im Suchfeld', async () => {
  const user = userEvent.setup();
  render(HelpPanel);
  const field = screen.getByPlaceholderText('Im Handbuch suchen');
  await user.type(field, 'Austritt');
  await user.click(screen.getAllByRole('button', { name: /Austritt/ })[0]);
  expect(field).toHaveValue('');
  await user.click(screen.getByRole('button', { name: 'Zurück' }));
  expect(field).toHaveValue('Austritt');
});

test('Esc im Suchfeld leert zuerst die Suche, die Hilfe bleibt offen', async () => {
  const user = userEvent.setup();
  render(HelpPanel);
  const field = screen.getByPlaceholderText('Im Handbuch suchen');
  await user.type(field, 'Austritt');
  const escape = new KeyboardEvent('keydown', { key: 'Escape', bubbles: true, cancelable: true });
  field.dispatchEvent(escape);
  await Promise.resolve();
  expect(field).toHaveValue('');
  expect(escape.defaultPrevented).toBe(true);
  expect(get(helpState).view.kind).toBe('toc');
});

test('Esc im leeren Suchfeld wird nicht abgefangen', () => {
  render(HelpPanel);
  const field = screen.getByPlaceholderText('Im Handbuch suchen');
  const escape = new KeyboardEvent('keydown', { key: 'Escape', bubbles: true, cancelable: true });
  field.dispatchEvent(escape);
  expect(escape.defaultPrevented).toBe(false);
});
