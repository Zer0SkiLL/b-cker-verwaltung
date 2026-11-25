// Zustand der Hilfe-Seitenleiste. Bewusst unabhängig vom Router: Ein Seitenwechsel lässt das
// gelesene Kapitel stehen, damit man einen Ablauf zu Ende lesen kann.
import { get, writable } from 'svelte/store';
import { chapterForRoute } from './handbook';
import type { Route } from './router';

export type HelpView = { kind: 'chapter'; slug: string } | { kind: 'toc' } | { kind: 'search'; query: string };
export type HelpState = { open: boolean; view: HelpView; history: HelpView[] };

export const helpState = writable<HelpState>({ open: false, view: { kind: 'toc' }, history: [] });

/** Zeigt eine Ansicht; war die Leiste offen, kommt die bisherige Ansicht in den Verlauf. */
function show(view: HelpView, remember = true): void {
  helpState.update((s) => ({
    open: true,
    view,
    history: s.open && remember ? [...s.history, s.view] : s.open ? s.history : [],
  }));
}

export function openHelp(slug: string): void {
  show({ kind: 'chapter', slug });
}

export function openHelpForRoute(route: Route): void {
  openHelp(chapterForRoute(route));
}

export function showToc(): void {
  show({ kind: 'toc' });
}

/** Suche ersetzt die Ansicht ohne Verlaufseintrag (jeder Tastendruck wäre sonst ein Eintrag). */
export function showSearch(query: string): void {
  const current = get(helpState);
  show({ kind: 'search', query }, current.view.kind !== 'search');
}

export function helpBack(): void {
  helpState.update((s) => {
    const previous = s.history[s.history.length - 1];
    return previous ? { ...s, view: previous, history: s.history.slice(0, -1) } : s;
  });
}

export function closeHelp(): void {
  helpState.update((s) => ({ ...s, open: false }));
}

/** F1 öffnet die Hilfe zur Seite, Esc schliesst sie. Liefert true, wenn die Taste behandelt wurde. */
export function handleHelpKey(event: KeyboardEvent, route: Route): boolean {
  if (event.key === 'F1') {
    openHelpForRoute(route);
    return true;
  }
  if (event.key === 'Escape' && get(helpState).open && !document.querySelector('dialog[open]')) {
    closeHelp();
    return true;
  }
  return false;
}
