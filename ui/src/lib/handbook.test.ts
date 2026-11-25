import { CHAPTERS, chapterForRoute, pdfUrl, renderChapter, searchChapters, slugFromHref } from './handbook';
import type { Route, SettingsTab } from './router';

const slugs = () => CHAPTERS.map((c) => c.slug);

test('jede Seite und jeder Einstellungs-Tab hat ein vorhandenes Kapitel', () => {
  const tabs: SettingsTab[] = ['catalog', 'backup', 'import', 'export', 'about'];
  const routes: Route[] = [
    { page: 'start' },
    { page: 'addresses' },
    { page: 'organisation', id: 1 },
    { page: 'person', id: null },
    { page: 'teams' },
    { page: 'groups' },
    { page: 'settings' },
    ...tabs.map((tab): Route => ({ page: 'settings', tab })),
  ];
  for (const route of routes) expect(slugs()).toContain(chapterForRoute(route));
  expect(chapterForRoute({ page: 'addresses' })).toBe('10-adressen');
  expect(chapterForRoute({ page: 'settings' })).toBe('15-listen-und-labels');
  expect(chapterForRoute({ page: 'settings', tab: 'about' })).toBe('19-ueber-das-programm');
});

test('Kapitel folgen inhalt.json und sind nicht leer', () => {
  expect(CHAPTERS).toHaveLength(20);
  expect(slugs()[0]).toBe('01-willkommen');
  expect(slugs()[19]).toBe('20-haeufige-fragen');
  expect(CHAPTERS[2].part).toBe('Abläufe');
  for (const c of CHAPTERS) expect(c.markdown.trim()).not.toBe('');
});

test('Suche ignoriert Gross-/Kleinschreibung und Umlaut-Schreibweisen', () => {
  expect(searchChapters('AUSTRITT').map((h) => h.slug)).toContain('05-austritt');
  expect(searchChapters('uberprufen').map((h) => h.slug)).toContain('06-daten-ueberpruefen');
  expect(searchChapters('ueberpruefen').map((h) => h.slug)).toContain('06-daten-ueberpruefen');
});

test('Suche verträgt Sonderzeichen', () => {
  for (const q of ['(', '*', '[', '\\']) expect(() => searchChapters(q)).not.toThrow();
  const chapters = [{ slug: 'x', title: 'X', part: 'P', markdown: 'Text (mit Klammer)' }];
  expect(searchChapters('(', chapters).map((h) => h.slug)).toEqual(['x']);
  expect(searchChapters('[', chapters)).toEqual([]);
});

test('leere Suche liefert nichts', () => {
  expect(searchChapters('')).toEqual([]);
  expect(searchChapters('   ')).toEqual([]);
});

test('Ausschnitt ist kurz und enthält das Suchwort', () => {
  const chapters = [{ slug: 'x', title: 'X', part: 'P', markdown: `${'Vorlauf '.repeat(40)}Zielwort ${'Nachlauf '.repeat(40)}` }];
  const [hit] = searchChapters('zielwort', chapters);
  expect(hit.snippet.length).toBeLessThanOrEqual(130);
  expect(hit.snippet).toContain('Zielwort');
});

test('Titeltreffer zählen auch', () => {
  const chapters = [{ slug: 'x', title: 'Teams', part: 'P', markdown: 'Ohne Wort.' }];
  expect(searchChapters('teams', chapters)).toHaveLength(1);
});

test('slugFromHref erkennt nur vorhandene Kapitel', () => {
  expect(slugFromHref('05-austritt.md#x')).toBe('05-austritt');
  expect(slugFromHref('05-austritt.md')).toBe('05-austritt');
  expect(slugFromHref('https://x.ch')).toBeNull();
  expect(slugFromHref('unbekannt.md')).toBeNull();
});

test('pdfUrl zeigt auf das Release der Version', () => {
  expect(pdfUrl('0.2.0')).toBe(
    'https://github.com/Zer0SkiLL/b-cker-verwaltung/releases/download/v0.2.0/Handbuch.pdf',
  );
});

test('renderChapter behält Kapitel-Links und zeigt den Titel', () => {
  const html = renderChapter('05-austritt');
  expect(html).toContain('<h1>Austritt</h1>');
  expect(renderChapter('gibt-es-nicht')).toContain('nicht gefunden');
});
