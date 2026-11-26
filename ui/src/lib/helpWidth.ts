// Breite der Hilfe-Seitenleiste: per Knopf „Breiter“/„Schmaler“ oder durch Ziehen am linken Rand.
// Die Breite gilt pro PC und bleibt über Neustarts erhalten (nur eine Bequemlichkeit).
import { get, writable } from 'svelte/store';

export const HELP_WIDTH = {
  normal: 380,
  wide: 680,
  min: 320,
  /** So viel Platz bleibt links für Seitenleiste und Seite. */
  keepForApp: 560,
} as const;

const STORAGE_KEY = 'hilfe-breite';

export function clampWidth(width: number, viewport: number): number {
  if (!Number.isFinite(width)) return HELP_WIDTH.normal;
  const max = Math.max(HELP_WIDTH.min, viewport - HELP_WIDTH.keepForApp);
  return Math.round(Math.min(Math.max(width, HELP_WIDTH.min), max));
}

export function loadWidth(): number {
  try {
    const stored = Number.parseInt(localStorage.getItem(STORAGE_KEY) ?? '', 10);
    return Number.isFinite(stored) ? Math.max(stored, HELP_WIDTH.min) : HELP_WIDTH.normal;
  } catch {
    return HELP_WIDTH.normal;
  }
}

function saveWidth(width: number): void {
  try {
    localStorage.setItem(STORAGE_KEY, String(width));
  } catch {
    // Ohne Speicher gilt die Breite nur bis zum Neustart.
  }
}

export const helpWidth = writable<number>(loadWidth());

export function setHelpWidth(width: number, viewport: number): void {
  const next = clampWidth(width, viewport);
  helpWidth.set(next);
  saveWidth(next);
}

/** Breiter als die normale Breite (z.B. nach Ziehen) zählt als „breit“. */
export const isWide = (width: number) => width > HELP_WIDTH.normal;

export function toggleWide(viewport: number): void {
  setHelpWidth(isWide(get(helpWidth)) ? HELP_WIDTH.normal : HELP_WIDTH.wide, viewport);
}
