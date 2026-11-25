import { get } from 'svelte/store';
import { closeHelp, handleHelpKey, helpBack, helpState, openHelp, openHelpForRoute, showSearch, showToc, toggleHelp } from './help';

beforeEach(() => {
  closeHelp();
  helpState.set({ open: false, view: { kind: 'toc' }, history: [] });
  document.body.innerHTML = '';
});

const key = (k: string, target: Element = document.body) => {
  const event = new KeyboardEvent('keydown', { key: k, bubbles: true });
  Object.defineProperty(event, 'target', { value: target });
  return event;
};

test('openHelpForRoute öffnet das Kapitel der Seite', () => {
  openHelpForRoute({ page: 'addresses' });
  expect(get(helpState)).toMatchObject({ open: true, view: { kind: 'chapter', slug: '10-adressen' } });
});

test('Kapitel bleibt beim Seitenwechsel stehen (kein Router-Bezug)', () => {
  openHelpForRoute({ page: 'addresses' });
  openHelp('05-austritt');
  expect(get(helpState).view).toEqual({ kind: 'chapter', slug: '05-austritt' });
});

test('openHelp merkt sich den Verlauf, helpBack geht zurück', () => {
  openHelp('03-mitglied-aufnehmen');
  openHelp('05-austritt');
  helpBack();
  expect(get(helpState).view).toEqual({ kind: 'chapter', slug: '03-mitglied-aufnehmen' });
  helpBack();
  helpBack();
  expect(get(helpState).history).toEqual([]);
});

test('showToc und showSearch öffnen die Leiste', () => {
  showToc();
  expect(get(helpState)).toMatchObject({ open: true, view: { kind: 'toc' } });
  showSearch('austritt');
  expect(get(helpState).view).toEqual({ kind: 'search', query: 'austritt' });
});

test('F1 öffnet die Hilfe auch aus einem Eingabefeld', () => {
  const input = document.createElement('input');
  document.body.append(input);
  expect(handleHelpKey(key('F1', input), { page: 'teams' })).toBe(true);
  expect(get(helpState)).toMatchObject({ open: true, view: { kind: 'chapter', slug: '14-teams' } });
});

test('Esc schliesst nur, wenn die Hilfe offen ist', () => {
  expect(handleHelpKey(key('Escape'), { page: 'teams' })).toBe(false);
  showToc();
  expect(handleHelpKey(key('Escape'), { page: 'teams' })).toBe(true);
  expect(get(helpState).open).toBe(false);
});

test('Esc bei offenem Dialog lässt die Hilfe offen', () => {
  showToc();
  const dialog = document.createElement('dialog');
  dialog.setAttribute('open', '');
  document.body.append(dialog);
  expect(handleHelpKey(key('Escape'), { page: 'teams' })).toBe(false);
  expect(get(helpState).open).toBe(true);
});

test('andere Tasten werden nicht behandelt', () => {
  expect(handleHelpKey(key('a'), { page: 'teams' })).toBe(false);
});

test('? auf der Seite mit offenem Seitenkapitel schliesst die Hilfe', () => {
  openHelpForRoute({ page: 'teams' });
  openHelpForRoute({ page: 'teams' });
  expect(get(helpState).open).toBe(false);
  openHelpForRoute({ page: 'teams' });
  expect(get(helpState)).toMatchObject({ open: true, history: [] });
});

test('? auf einem anderen Kapitel wechselt zum Seitenkapitel statt zu schliessen', () => {
  openHelpForRoute({ page: 'teams' });
  openHelp('05-austritt');
  openHelpForRoute({ page: 'teams' });
  expect(get(helpState)).toMatchObject({ open: true, view: { kind: 'chapter', slug: '14-teams' } });
});

test('dasselbe Kapitel erneut öffnen erzeugt keinen Verlaufseintrag', () => {
  openHelp('05-austritt');
  openHelp('05-austritt');
  expect(get(helpState).history).toEqual([]);
});

test('toggleHelp öffnet das Inhaltsverzeichnis und schliesst wieder', () => {
  toggleHelp();
  expect(get(helpState)).toMatchObject({ open: true, view: { kind: 'toc' } });
  toggleHelp();
  expect(get(helpState).open).toBe(false);
});

test('F1 zweimal schliesst die Hilfe wieder', () => {
  handleHelpKey(key('F1'), { page: 'groups' });
  handleHelpKey(key('F1'), { page: 'groups' });
  expect(get(helpState).open).toBe(false);
});

test('bereits behandeltes Esc schliesst die Hilfe nicht', () => {
  showToc();
  const event = new KeyboardEvent('keydown', { key: 'Escape', cancelable: true });
  event.preventDefault();
  expect(handleHelpKey(event, { page: 'teams' })).toBe(false);
  expect(get(helpState).open).toBe(true);
});
